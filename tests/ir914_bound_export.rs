// SPDX-License-Identifier: AGPL-3.0-or-later
//! IR-914: a QSL export refusal is withheld while bound siblings reach CG.
//! The injected refusal names one unbound model element because this
//! QSL-compiled fixture has no attachable abstraction relation. The test
//! checks that the driver preserves the exact typed refusal it receives.

mod support;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::num::NonZeroU64;
use std::time::Duration;

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::SourceIdentity;
use qsl_package::{BoundItem, CheckedPackage, ExportRefusal, UnboundElement};
use qsl_replay::spine::{AdmittedModels, CheckedUnit, EmittedUnit, SpineLimits};
use qsl_route::routing::Disposition;
use qsl_route::Candidate;
use qsl_semantics::check::AbstractionKey;
use qsl_semantics::model::intake::{package_input, PackageInput};
use qsl_semantics::value::DefinitionRole;
use quire_checked::key::DeclarationKey;
use quire_contract_codegen::{
    execute_kani_obligation, execute_kani_scalar_native, BackendDescriptor, BackendKind,
    BackendProviderEnvelope, ContractItemSettlement, ContractNegotiationError,
    ContractRequestPosition, GenerationContexts, KaniExecutionRequest, KaniInstallation,
    KaniRunOutcome, KaniScalarNativeRequest, KindOutput, ObligationDisposition, RoutedGeneration,
    RoutedGenerationError, RoutedGenerationItem, ScalarLimits,
};
use quire_contract_ir::{
    CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, ContractPackageV1,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverRefusal, GenerationInput, KaniInput,
    ProviderManifest, QslCg, Upstream,
};
use quire_eval::FamilyOutcome;
use quire_exact::{Cancel, Integer, Meter, Outcome, ScalarLimits as QslScalarLimits, Value};

struct ExportVector {
    envelope: RefCell<Option<BackendProviderEnvelope>>,
    positions: RefCell<Option<Vec<ContractRequestPosition>>>,
    mode: ExportMode,
    export_calls: RefCell<usize>,
    export_results: RefCell<Vec<Result<BoundItem, ExportRefusal>>>,
}

#[derive(Clone, Copy)]
enum ExportMode {
    Injected,
    Natural,
    ForceRelation,
}

impl ExportVector {
    fn observed(mode: ExportMode) -> Self {
        Self {
            envelope: RefCell::new(None),
            positions: RefCell::new(None),
            mode,
            export_calls: RefCell::new(0),
            export_results: RefCell::new(Vec::new()),
        }
    }

    fn natural() -> Self {
        Self::observed(ExportMode::Natural)
    }

    fn force_relation() -> Self {
        Self::observed(ExportMode::ForceRelation)
    }

    fn export_calls(&self) -> usize {
        *self.export_calls.borrow()
    }

    fn export_results(&self) -> Vec<Result<BoundItem, ExportRefusal>> {
        self.export_results.borrow().clone()
    }

    fn envelope(&self) -> BackendProviderEnvelope {
        self.envelope.borrow().clone().expect("CG negotiated")
    }
}

fn unbound_fixture(item: &OccurrenceKey) -> ExportRefusal {
    let key = AbstractionKey::Object(DeclarationKey {
        package: "it013/domain".to_owned(),
        node: "missing-object".to_owned(),
    });
    ExportRefusal::Unbound {
        item: item.clone(),
        elements: vec![UnboundElement {
            package: key.package().to_owned(),
            key,
        }],
    }
}

impl Upstream for ExportVector {
    fn has_abstraction_relation(&self, package: &CheckedPackage) -> bool {
        match self.mode {
            ExportMode::Injected | ExportMode::ForceRelation => true,
            ExportMode::Natural => QslCg.has_abstraction_relation(package),
        }
    }

    fn export_bindings(
        &self,
        package: &CheckedPackage,
        items: &[OccurrenceKey],
    ) -> Result<Vec<Result<BoundItem, ExportRefusal>>, ExportRefusal> {
        *self.export_calls.borrow_mut() += 1;
        let results = match self.mode {
            ExportMode::Natural | ExportMode::ForceRelation => {
                qsl_package::export_bindings(package, items)
            }
            ExportMode::Injected => {
                assert_eq!(items.len(), 3);
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        if index == 1 {
                            Err(unbound_fixture(item))
                        } else {
                            Ok(BoundItem {
                                item: item.clone(),
                                bindings: Vec::new(),
                            })
                        }
                    })
                    .collect()
            }
        };
        *self.export_results.borrow_mut() = results.clone();
        Ok(results)
    }

    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        package_name: &str,
        packages: &PackageInput,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, AdmittedModels, EmittedUnit), DriverRefusal> {
        QslCg.compile(source, path, bytes, package_name, packages, limits, cancel)
    }

    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        QslCg.read_package(emitted, limits, evidence)
    }

    fn backend_kind(&self, descriptor: &BackendDescriptor) -> Option<BackendKind> {
        QslCg.backend_kind(descriptor)
    }

    fn negotiate(
        &self,
        admitted: &CheckedPackageV2,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ContractItemSettlement>, Box<ContractNegotiationError>> {
        *self.envelope.borrow_mut() = Some(envelope.clone());
        *self.positions.borrow_mut() = Some(positions.to_vec());
        QslCg.negotiate(admitted, package, positions, envelope)
    }

    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>> {
        QslCg.route(dispositions)
    }

    fn generate(
        &self,
        admitted: &CheckedPackageV2,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        QslCg.generate(admitted, package, positions, routed, contexts)
    }
}

#[test]
#[trace("IT-013-SC-01")]
fn refused_binding_is_withheld_as_bound_siblings_flow_to_codegen() {
    let root = DefinitionRole::Root.identity();
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{root}\";\n\
         function a using v(x: Int[0, 9]): Int[-9, 0] pure {{ -x }}\n\
         function b using v(y: Int[0, 9]): Int[-9, 0] pure {{ -y }}\n\
         function c using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n"
    );
    let registry = kani_registry();
    let export = ExportVector::observed(ExportMode::Injected);
    let driven = drive(
        &export,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "ir914-integration"),
            path: "ir914.native",
            bytes: source.as_bytes(),
            package_name: "agent-ix/it-013",
            packages: &BTreeMap::new(),
            limits: SpineLimits::default(),
            cancel: Cancel::new(),
        },
        &registry,
        None,
        GenerationInput {
            kani: Some(KaniInput {
                ceilings: quire_contract_codegen::ProofCeilings {
                    memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive ceiling"),
                    wall_clock: Duration::from_secs(30),
                },
                subject_path: "crate::subject",
                unwind: 1,
            }),
            scalar_execution: None,
        },
    )
    .expect("a per-item export refusal does not refuse the run");
    let envelope = export.envelope.borrow();
    assert_eq!(envelope.as_ref().expect("CG negotiated").items.len(), 2);
    let positions = export.positions.borrow();
    let positions = positions.as_ref().expect("CG received request positions");
    let driver_roles = positions
        .iter()
        .filter_map(|position| match position {
            ContractRequestPosition::DriverItem { .. } => Some("bound"),
            ContractRequestPosition::Excluded { .. } => Some("excluded"),
            ContractRequestPosition::ContextRoot { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(driver_roles, ["bound", "excluded", "bound"]);
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item
                .backend
                .as_ref()
                .expect("bound item settles")
                .request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(driven.item_refusals.len(), 1);
    assert_eq!(driven.item_refusals[0].request_index, 1);
    assert_eq!(
        driven.item_refusals[0].refusal,
        unbound_fixture(driven.request_items[1].occurrence())
    );
    assert_eq!(
        driven.item_refusals[0].occurrence,
        *driven.request_items[1].occurrence()
    );
    assert_eq!(
        driven
            .generation()
            .items
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2],
        "both bound siblings generate and the refused item does not"
    );
}

fn kani_registry() -> quire_driver::AdmittedRegistry {
    build_registry(&[ProviderManifest {
        identity: "kani".to_owned(),
        advertises: vec![(
            Some("value-validity".to_owned()),
            Some("bounded".to_owned()),
        )],
        domains: Some(vec!["integer".to_owned()]),
        bounds: None,
        wire: None,
    }])
    .expect("Kani provider registers")
}

fn generation_input() -> GenerationInput<'static> {
    GenerationInput {
        kani: Some(KaniInput {
            ceilings: quire_contract_codegen::ProofCeilings {
                memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive ceiling"),
                wall_clock: Duration::from_secs(30),
            },
            subject_path: "crate::subject",
            unwind: 1,
        }),
        scalar_execution: None,
    }
}

fn compile_input<'a>(source: &'a str, packages: &'a PackageInput) -> CompileInput<'a> {
    CompileInput {
        source: SourceIdentity::new("agent-ix", "ir914-export-observation"),
        path: "ir914.native",
        bytes: source.as_bytes(),
        package_name: "agent-ix/it-013",
        packages,
        limits: SpineLimits::default(),
        cancel: Cancel::new(),
    }
}

fn integer_siblings_source() -> String {
    let root = DefinitionRole::Root.identity();
    format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{root}\";\n\
         function a using v(x: Int[0, 9]): Int[1, 10] pure {{ x + 1 }}\n\
         function b using v(y: Int[0, 9]): Int[1, 10] pure {{ y + 1 }}\n\
         function c using v(z: Int[0, 9]): Int[1, 10] pure {{ z + 1 }}\n"
    )
}

fn mixed_model_source() -> (String, PackageInput) {
    let root = support::qsl_root().join("tests/fixtures");
    let path = root.join("spine-model.semantic-ir.json");
    let document = fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let packages =
        package_input([document.as_slice()]).expect("QSL fixture declares one model identity");
    let source_path = root.join("spine-model.native");
    let selected_source =
        fs::read_to_string(&source_path).unwrap_or_else(|error| panic!("{source_path:?}: {error}"));
    let selection = selected_source
        .split_once("function keep")
        .expect("QSL fixture contains its model selection")
        .0;
    let source = format!(
        "{selection}\
         function a using v(x: Int[0, 9]): Int[1, 10] pure {{ x + 1 }}\n\
         function b using v(y: M::Widget): Integer pure {{ deref(y).code + 1 }}\n\
         function c using v(z: Int[0, 9]): Int[1, 10] pure {{ z + 1 }}\n"
    );
    (source, packages)
}

#[test]
#[trace("IT-013-SC-07")]
fn relationless_production_flow_skips_export_and_sends_every_item_to_codegen() {
    let source = integer_siblings_source();
    let registry = kani_registry();
    let upstream = ExportVector::natural();
    let driven = drive(
        &upstream,
        compile_input(&source, &BTreeMap::new()),
        &registry,
        None,
        generation_input(),
    )
    .expect("relationless production flow completes");
    assert!(!QslCg.has_abstraction_relation(driven.package().package()));
    assert_eq!(upstream.export_calls(), 0);
    assert!(driven.item_refusals.is_empty());
    assert_eq!(driven.request_items.len(), 3);
    assert_eq!(upstream.envelope().items.len(), 3);
    assert_eq!(driven.generation().items.len(), 3);
}

#[test]
#[trace("IT-013-SC-08")]
fn real_qsl_export_refusal_is_withheld_before_codegen() {
    let (source, packages) = mixed_model_source();
    let registry = kani_registry();
    let upstream = ExportVector::force_relation();
    let driven = drive(
        &upstream,
        compile_input(&source, &packages),
        &registry,
        None,
        generation_input(),
    )
    .expect("real per-item QSL export refusal leaves siblings running");
    assert!(!QslCg.has_abstraction_relation(driven.package().package()));
    assert_eq!(upstream.export_calls(), 1);
    let results = upstream.export_results();
    assert_eq!(
        results.len(),
        3,
        "QSL export results: {results:?}; driver items: {:?}",
        driven.request_items
    );
    // FR-075 orders requests by occurrence key, not function source position.
    // Identify the model-reading item from the admitted fixture, independently
    // of the export result and the driver's refusal partition.
    let graph = driven.package().package().graph();
    let ordered_keys = graph.requirements().keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        driven
            .request_items
            .iter()
            .map(|item| item.occurrence().clone())
            .collect::<Vec<_>>(),
        ordered_keys
    );
    let widget = AbstractionKey::Object(DeclarationKey {
        package: "acme/orders".to_owned(),
        node: "ix://acme/orders/Widget".to_owned(),
    });
    let referenced_elements = ordered_keys
        .iter()
        .map(|key| {
            graph
                .item_elements(key)
                .expect("the occurrence names an admitted requirement")
                .expect("the fixture's referenced declarations are admitted")
        })
        .collect::<Vec<_>>();
    let widget_indices = referenced_elements
        .iter()
        .enumerate()
        .filter_map(|(index, elements)| elements.contains(&widget).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(widget_indices.len(), 1, "exactly function b reads Widget");
    let refused_index = widget_indices[0];
    assert_eq!(
        referenced_elements[refused_index]
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        std::slice::from_ref(&widget)
    );
    let expected_refusal = ExportRefusal::Unbound {
        item: ordered_keys[refused_index].clone(),
        elements: vec![UnboundElement {
            package: "acme/orders".to_owned(),
            key: widget,
        }],
    };
    assert_eq!(results[refused_index], Err(expected_refusal.clone()));
    let healthy_indices = (0..ordered_keys.len())
        .filter(|index| *index != refused_index)
        .collect::<Vec<_>>();
    assert_eq!(healthy_indices.len(), 2);
    for index in &healthy_indices {
        assert!(referenced_elements[*index].is_empty());
        assert_eq!(
            results[*index],
            Ok(BoundItem {
                item: ordered_keys[*index].clone(),
                bindings: Vec::new(),
            })
        );
    }
    assert_eq!(upstream.envelope().items.len(), 2);
    assert_eq!(driven.item_refusals.len(), 1);
    assert_eq!(driven.item_refusals[0].request_index, refused_index);
    assert_eq!(
        driven.item_refusals[0].occurrence,
        ordered_keys[refused_index]
    );
    assert_eq!(driven.item_refusals[0].refusal, expected_refusal);
    assert_eq!(
        driven.item_refusals[0].refusal,
        results[refused_index].as_ref().unwrap_err().clone()
    );
    assert_eq!(
        driven
            .generation()
            .items
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        healthy_indices,
        "both model-free siblings generate and the Widget refusal is withheld"
    );
}

fn drive_integer_step() -> Driven {
    let root = DefinitionRole::Root.identity();
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{root}\";\n\
         function step using v(x: Int[0, 9]): Int[1, 10] pure {{ x + 1 }}\n"
    );
    let registry = kani_registry();
    drive(
        &QslCg,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "it013-kani"),
            path: "it013.native",
            bytes: source.as_bytes(),
            package_name: "agent-ix/it-013",
            packages: &BTreeMap::new(),
            limits: SpineLimits::default(),
            cancel: Cancel::new(),
        },
        &registry,
        None,
        GenerationInput {
            kani: Some(KaniInput {
                ceilings: quire_contract_codegen::ProofCeilings {
                    memory_bytes: NonZeroU64::new(16 * 1024 * 1024 * 1024)
                        .expect("positive ceiling"),
                    wall_clock: Duration::from_secs(600),
                },
                subject_path: "crate::subject",
                unwind: 4,
            }),
            scalar_execution: None,
        },
    )
    .expect("integer addition drives")
}

#[test]
#[trace("IT-013-SC-04")]
fn qsl_integer_addition_reaches_a_supported_kani_harness() {
    let driven = drive_integer_step();
    let [item] = driven.generation().items.as_slice() else {
        panic!("one generated Kani item: {:?}", driven.generation().items);
    };
    let KindOutput::Kani { record, harness } = &item.output else {
        panic!("expected Kani output, got {:?}", item.output);
    };
    assert!(matches!(
        record.disposition,
        ObligationDisposition::Supported { .. }
    ));
    assert!(
        harness.is_some(),
        "supported item retains the scalar harness"
    );

    let qsl_limits = QslScalarLimits {
        integer_bits: 64,
        decimal_digits: 20,
        scale_expansion: 20,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 100,
        value_occurrences: 100,
        work_units: 1000,
        result_units: 1000,
    };
    let native = driven
        .package()
        .package()
        .call(
            &QualifiedName::unqualified("step".to_owned()).expect("checked function name"),
            vec![Value::Integer(Integer::from(4_i64))],
            &qsl_semantics::model::object_environment::ObjectEnvironment::default(),
            &mut Meter::new(qsl_limits),
        )
        .expect("QSL evaluates the checked function");
    assert!(matches!(
        native.outcome,
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value)))
            if value == Integer::from(5_i64)
    ));
}

#[test]
#[trace("IT-013-SC-05")]
#[ignore = "real Kani lane: run serially after the normal integration gate"]
fn qsl_integer_addition_proves_and_matches_native_execution() {
    let driven = drive_integer_step();
    let [item] = driven.generation().items.as_slice() else {
        panic!("one generated Kani item: {:?}", driven.generation().items);
    };
    let KindOutput::Kani {
        harness: Some(harness),
        ..
    } = &item.output
    else {
        panic!("expected retained Kani harness, got {:?}", item.output);
    };
    let artifacts = driven
        .generation()
        .oracle_artifacts
        .as_ref()
        .expect("the routed call generated an oracle crate");
    let limits = ScalarLimits {
        integer_bits: 64,
        decimal_digits: 20,
        scale_expansion: 20,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 100,
        value_occurrences: 100,
        work_units: 1000,
        result_units: 1000,
    };
    let installation = KaniInstallation::discover().expect("cargo-kani is installed");
    let cancel = Cancel::new();
    let operands = [4_i64, 1_i64];
    let target = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("it013-kani");
    let receipt = execute_kani_scalar_native(&KaniScalarNativeRequest {
        cancel: &cancel,
        installation: &installation,
        harness: harness.as_ref(),
        oracle_artifacts: artifacts,
        operands: &operands,
        scalar_limits: limits,
        target_directory: &target,
    })
    .unwrap_or_else(|error| panic!("generated scalar proof/native run: {:?}", error.class()));
    assert_eq!(
        receipt.proof_evidence().outcome,
        quire_contract_codegen::KaniRunOutcome::Verified
    );
    assert!(receipt.proof_evidence().success_checks > 0);
    let qsl = driven
        .package()
        .package()
        .call(
            &QualifiedName::unqualified("step".to_owned()).expect("checked function name"),
            vec![Value::Integer(Integer::from(4_i64))],
            &qsl_semantics::model::object_environment::ObjectEnvironment::default(),
            &mut Meter::new(QslScalarLimits {
                integer_bits: limits.integer_bits,
                decimal_digits: limits.decimal_digits,
                scale_expansion: limits.scale_expansion,
                text_input_bytes: limits.text_input_bytes,
                text_scalars: limits.text_scalars,
                normalized_scalars: limits.normalized_scalars,
                unit_edges: limits.unit_edges,
                value_occurrences: limits.value_occurrences,
                work_units: limits.work_units,
                result_units: limits.result_units,
            }),
        )
        .expect("QSL evaluates the checked function");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(qsl_value))) = qsl.outcome
    else {
        panic!("QSL must complete integer addition");
    };
    assert_eq!(qsl_value, Integer::from(5_i64));
    let native_value = receipt
        .native_outcome()
        .clone()
        .completed()
        .expect("staged native oracle completes");
    assert_eq!(native_value.to_string(), qsl_value.to_string());
}

#[test]
#[trace("IT-013-SC-06")]
#[ignore = "real Kani lane: run serially after the normal integration gate"]
fn qsl_integer_addition_mutant_falsifies_the_generated_scalar_proof() {
    let driven = drive_integer_step();
    let [item] = driven.generation().items.as_slice() else {
        panic!("one generated Kani item: {:?}", driven.generation().items);
    };
    let KindOutput::Kani {
        harness: Some(harness),
        ..
    } = &item.output
    else {
        panic!("expected retained Kani harness, got {:?}", item.output);
    };
    let manifest = driven
        .generation()
        .oracle_artifacts
        .as_ref()
        .expect("the routed call generated an oracle crate")
        .iter()
        .find(|artifact| artifact.path == "Cargo.toml")
        .expect("the oracle crate includes Cargo.toml");
    let add_call = "rt::IntegerArithmetic::Add(left, right)";
    assert_eq!(harness.rust.contents.matches(add_call).count(), 1);
    let mut mutant = harness.as_ref().clone();
    mutant.rust.contents = mutant
        .rust
        .contents
        .replace(add_call, "rt::IntegerArithmetic::Subtract(left, right)");

    let scratch = tempfile::Builder::new()
        .prefix("it013-mutated-scalar-")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .expect("temporary Kani crate");
    std::fs::create_dir_all(scratch.path().join("src")).expect("crate source directory");
    std::fs::write(scratch.path().join("Cargo.toml"), &manifest.contents).expect("crate manifest");
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        scratch.path().join("Cargo.lock"),
    )
    .expect("use this integration cohort's resolved dependency lock");
    std::fs::write(scratch.path().join("src/lib.rs"), &mutant.rust.contents)
        .expect("mutated harness source");
    std::fs::write(
        scratch.path().join("build.rs"),
        "fn main() { println!(\"cargo:rustc-check-cfg=cfg(kani)\"); }\n",
    )
    .expect("Kani cfg declaration");
    let installation = KaniInstallation::discover().expect("cargo-kani is installed");
    let cancel = Cancel::new();
    let evidence = execute_kani_obligation(&KaniExecutionRequest {
        cancel: &cancel,
        installation: &installation,
        harness: (&mutant).into(),
        crate_directory: scratch.path(),
        target_directory: &scratch.path().join("target"),
        home_directory: None,
    })
    .unwrap_or_else(|refusal| panic!("mutated Kani run: {refusal}"));
    let KaniRunOutcome::Falsified { counterexample } = &evidence.outcome else {
        panic!(
            "mutated arithmetic must falsify, got {:?}; failure {:?}; diagnostics {:?}",
            evidence.outcome, evidence.failure_cause, evidence.diagnostics
        );
    };
    assert!(counterexample.contains(mutant.identity.harness_symbol.as_str()));
    assert!(counterexample.contains("Check for `assertion`"));
    assert!(counterexample
        .contains("the oracle must complete with exactly the native quire.op.integer.add result"));
}
