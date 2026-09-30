// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-008 / IT-010: QSL's emitted IR bytes, read by contract IR and fed to
//! quire-contract-codegen, compiled and executed against QSL's native runtime.

mod support;

use std::{fmt::Write as _, fs, path::Path, path::PathBuf, process::Command};

use ix_trace_rs::trace;
use qsl_foundation::ByteDigest;
use quire_contract_codegen as codegen;
use quire_contract_ir as ir;
use quire_spec_language::{
    command,
    formal_source::FormalSource,
    lowering::{self, LoweringLimits, ProjectionTarget},
    native_model::NativeModel,
    package::NativePackage,
    runtime::{
        evaluate, validate, EvaluationLimits, EvaluationOutcome, ExecutionSelection,
        ImplicationEventKind, InvocationRef, ObjectIdentity, ObservationSelection, RuntimeInput,
        Snapshot, SnapshotRef, ValidationLimits, ValueNode,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use support::{
    bound_clause, parameter_names, public_function, public_functions, run, GeneratedCrate,
    CONFIG_DOMAIN,
};

// ---------------------------------------------------------------------------
// native-compile/1 and native-run/1 request files, as QSL's command reads them.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SourceFile<'a> {
    file: &'a str,
    authority: &'a str,
    identity: &'a str,
    revision_namespace: &'a str,
    revision: &'a str,
    digest: String,
    document: &'a str,
    formal_revision: u64,
}

impl<'a> SourceFile<'a> {
    fn new(source: &'a FormalSource, file: &'a str) -> Self {
        let identity = source.source().identity();
        Self {
            file,
            authority: &identity.authority,
            identity: &identity.identity,
            revision_namespace: &identity.revision_namespace,
            revision: &identity.revision,
            digest: source.source().digest().to_string(),
            document: source.identity().document().as_str(),
            formal_revision: source.identity().revision().get(),
        }
    }
}

#[derive(Serialize)]
struct ModelFile<'a> {
    format: &'a str,
    source: SourceFile<'a>,
}

#[derive(Serialize)]
struct Owner<'a> {
    package: &'a str,
    requirement: &'a str,
    revision: u64,
}

impl<'a> Owner<'a> {
    fn new(requirement: &'a ir::RequirementRef) -> Self {
        Self {
            package: requirement.package().as_str(),
            requirement: requirement.requirement().as_str(),
            revision: requirement.revision().get(),
        }
    }
}

#[derive(Serialize)]
struct Binding<'a> {
    name: &'a str,
    owner: Owner<'a>,
    clause: &'a str,
    point: &'a ir::ExecutionPoint,
}

#[derive(Serialize)]
struct Program<'a> {
    source: SourceFile<'a>,
    clauses: Vec<Binding<'a>>,
}

#[derive(Serialize)]
struct FileSelection<R> {
    file: String,
    reference: R,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Observation<'a> {
    Current {
        snapshot: &'a SnapshotRef,
        self_object: &'a ObjectIdentity,
    },
}

#[derive(Serialize)]
struct Selection<'a> {
    owner: Owner<'a>,
    clause: &'a str,
    observation: Observation<'a>,
}

#[derive(Serialize)]
struct Request<'a> {
    models: Vec<ModelFile<'a>>,
    program: Program<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshots: Option<Vec<FileSelection<SnapshotRef>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invocations: Option<Vec<FileSelection<InvocationRef>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection: Option<Selection<'a>>,
}

#[derive(Serialize)]
struct Envelope<'a> {
    format: &'a str,
    request: Request<'a>,
}

/// Write `native`'s model and program files plus `compile.json`, and, when
/// `run` is given, the runtime inputs plus `request.json`.
fn write_job(
    directory: &Path,
    native: &NativePackage<'_>,
    model: &NativeModel,
    run: Option<(&Snapshot, &ExecutionSelection)>,
) {
    let program = &native.checked().bindings().source;
    fs::write(directory.join("model.json"), model.source().source().text()).unwrap();
    fs::write(directory.join("program.native"), program.source().text()).unwrap();
    let bindings = &native.checked().bindings().clauses;
    let request = || Request {
        models: vec![ModelFile {
            format: quire_spec_language::model_source::FORMAT,
            source: SourceFile::new(model.source(), "model.json"),
        }],
        program: Program {
            source: SourceFile::new(program, "program.native"),
            clauses: bindings
                .iter()
                .map(|binding| Binding {
                    name: &binding.name,
                    owner: Owner::new(&binding.requirement),
                    clause: binding.clause.as_str(),
                    point: &binding.execution_point,
                })
                .collect(),
        },
        snapshots: None,
        invocations: None,
        selection: None,
    };
    let compile = Envelope {
        format: "native-compile/1",
        request: request(),
    };
    fs::write(
        directory.join("compile.json"),
        serde_json::to_vec(&compile).unwrap(),
    )
    .unwrap();
    let Some((snapshot, selection)) = run else {
        return;
    };
    fs::write(directory.join("snapshot-0.json"), snapshot.bytes()).unwrap();
    let ObservationSelection::Current {
        snapshot: selected,
        self_object,
    } = &selection.observation
    else {
        panic!("rule runs select a current observation")
    };
    let mut request = request();
    request.snapshots = Some(vec![FileSelection {
        file: "snapshot-0.json".into(),
        reference: snapshot.reference(),
    }]);
    request.invocations = Some(Vec::new());
    request.selection = Some(Selection {
        owner: Owner::new(&selection.requirement),
        clause: selection.clause.as_str(),
        observation: Observation::Current {
            snapshot: selected,
            self_object,
        },
    });
    let job = Envelope {
        format: "native-run/1",
        request,
    };
    fs::write(
        directory.join("request.json"),
        serde_json::to_vec(&job).unwrap(),
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// IT-010-SC-02: a plain integer comparison through command export and codegen.
// ---------------------------------------------------------------------------

fn integer_model() -> NativeModel {
    support::rule_model(&[
        ("amount", json!({"kind": "scalar", "name": "Version"})),
        ("delta", json!({"kind": "scalar", "name": "Signed"})),
        ("distance", json!({"kind": "scalar", "name": "Distance"})),
        ("flag", json!({"kind": "boolean"})),
    ])
}

#[test]
#[trace("IT-010-SC-02")]
fn actual_command_exports_selected_bytes_and_backend_generates_numeric_oracle() {
    const TARGET: ProjectionTarget = ProjectionTarget::IntegerIrV1;
    let directory = tempfile::tempdir().unwrap();
    let models = [integer_model()];
    let native = support::rule_package(&models, "amount < 7");
    write_job(directory.path(), &native, &models[0], None);
    let exported = command::lower_for(&directory.path().join("compile.json"), TARGET)
        .unwrap_or_else(|error| panic!("{error}"));
    let projection = lowering::lower_for(&native, TARGET, LoweringLimits::default()).unwrap();
    assert_eq!(exported, projection.bytes());

    let consumer = ir::BoundPackage::from_json_bytes(&exported).unwrap();
    let codegen::BoundOracleGeneration::Generated(generated) =
        codegen::generate_bound_oracles(&consumer).unwrap()
    else {
        panic!("numeric comparison must produce an executable oracle");
    };
    let generated_clause = generated
        .clauses()
        .iter()
        .find(|clause| clause.identity().clause().as_str() == "population_rule")
        .expect("selected numeric clause");
    let bound = consumer
        .clauses()
        .iter()
        .find(|clause| clause.identity() == generated_clause.identity())
        .expect("generated clause retains a bound source clause");
    let oracle_source = &generated_clause.bundle().rust.contents;
    let function = public_function(oracle_source);
    assert_eq!(function.sig.inputs.len(), 1);
    assert!(
        matches!(function.sig.output, syn::ReturnType::Type(_, ref ty)
        if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("bool")))
    );

    let oracle = GeneratedCrate::new("integer-oracle", &[], "");
    oracle.write("src/oracle.rs", oracle_source);
    oracle.write(
        "src/main.rs",
        &format!(
            "#![deny(warnings)]\n#[allow(dead_code)] #[path = \"oracle.rs\"] mod oracle;\nfn main() {{ let value: i64 = std::env::args().nth(1).unwrap().parse().unwrap(); println!(\"{{}}\", oracle::{}(value)); }}\n",
            function.sig.ident
        ),
    );
    let executable = oracle.build_binary();
    for (value, expected) in [(0, true), (1000, false)] {
        let executed = run(Command::new(&executable).arg(value.to_string()));
        assert_eq!(
            String::from_utf8(executed.stdout).unwrap().trim(),
            expected.to_string()
        );
    }

    let strategy = codegen::generate_bound_strategy(&codegen::BoundStrategyRequest {
        package: &consumer,
        clause: bound.identity(),
        population: codegen::BoundStrategyPopulation::Boundary,
        minimum_accepted_cases: 1,
        minimum_rejected_cases: 0,
        maximum_discarded_cases: 0,
    })
    .unwrap();
    syn::parse_file(&strategy.rust.contents).expect("generated integer strategy is Rust");
    assert!(strategy.rust.contents.contains("run_census"));

    let kani = support::absent_precondition_kani_bundle(
        bound,
        "amount_absent_precondition",
        "it-010-plain-integer",
    );
    syn::parse_file(&kani.rust.contents).expect("generated integer Kani adapter is Rust");
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&kani.proof_graph.contents).unwrap();
    assert_eq!(graph.proof_id, "it-010-plain-integer");
    assert_eq!(graph.subject_arguments.len(), 1);
    assert!(graph.subject_results.is_empty());
    let bounds = graph.subject_arguments[0].integer_bounds.as_ref().unwrap();
    assert_eq!((bounds.minimum, bounds.maximum), (0, 1000));

    assert!("future/v9".parse::<ProjectionTarget>().is_err());
    let unsupported = support::rule_package(&models, "self.n < 7");
    write_job(directory.path(), &unsupported, &models[0], None);
    let error = command::lower_for(&directory.path().join("compile.json"), TARGET)
        .expect_err("an unsupported clause exports nothing");
    assert_eq!(error.exit_code(), 21);
    let error = error.value().unwrap();
    assert_eq!(error["code"], "unsupported_projection");
    assert_eq!(error["details"]["profile"], "integer-ir/v1");
}

// ---------------------------------------------------------------------------
// IT-008-SC-01/02: exported Boolean bytes reach the complete backend population.
// ---------------------------------------------------------------------------

#[test]
#[trace("IT-008-SC-01", "IT-008-SC-02")]
fn exported_boolean_bytes_reach_the_ir_reader_and_the_complete_backend_population() {
    let directory = tempfile::tempdir().unwrap();
    let models = [support::rule_model(&[("flag", json!({"kind": "boolean"}))])];
    let native = support::rule_package(&models, "true implies flag");
    let snapshot =
        support::node_snapshot(&models[0], &[("flag", ValueNode::Boolean { value: true })]);
    let selection = support::rule_selection(&models[0], &snapshot);
    write_job(
        directory.path(),
        &native,
        &models[0],
        Some((&snapshot, &selection)),
    );
    let ran = command::run(&directory.path().join("request.json"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(ran.exit_code, 0);
    assert_eq!(ran.value.as_value()["truth"], true);
    fs::remove_file(directory.path().join("snapshot-0.json")).unwrap();
    fs::remove_file(directory.path().join("request.json")).unwrap();

    let exported = command::lower(&directory.path().join("compile.json"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(!exported.ends_with(b"\n"));

    // Rebuild the model from the exported model file, as a consumer would.
    let model_text = fs::read_to_string(directory.path().join("model.json")).unwrap();
    assert_eq!(model_text, models[0].source().source().text());
    let expected = lowering::lower(&native, LoweringLimits::default()).unwrap();
    assert_eq!(exported, expected.bytes());
    let consumer = ir::BoundPackage::from_json_bytes(&exported).unwrap();
    assert_eq!(consumer.digest(), expected.bound().digest());
    assert_eq!(&consumer, expected.bound());

    let codegen::BoundOracleGeneration::Generated(population) =
        codegen::generate_bound_oracles(&consumer).unwrap()
    else {
        panic!("expected complete generated population");
    };
    let names: Vec<_> = population
        .clauses()
        .iter()
        .map(|clause| clause.identity().clause().as_str())
        .collect();
    assert_eq!(names, ["other_rule", "population_rule"]);
    for clause in population.clauses() {
        assert_eq!(public_functions(&clause.bundle().rust.contents).len(), 1);
        let regions: Vec<codegen::SourceRegion> =
            serde_json::from_str(&clause.bundle().source_map.contents).unwrap();
        assert!(!regions.is_empty());
        assert!(regions
            .iter()
            .all(|region| region.package_id == "example/runtime-rules"
                && region.requirement_id == "PopulationRule"
                && region.requirement_revision == 7
                && region.clause_id == clause.identity().clause().as_str()));
    }
    let compiled = command::compile(&directory.path().join("compile.json"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_ne!(compiled, exported);
    assert_eq!(ByteDigest::of(&compiled), native.digest());
}

// ---------------------------------------------------------------------------
// IT-008 / TC-094: generated truth and measured activation against native events.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LlvmExport {
    #[serde(rename = "type")]
    kind: String,
    version: String,
    cargo_llvm_cov: LlvmExportProducer,
    data: Vec<LlvmExportData>,
}

#[derive(Deserialize)]
struct LlvmExportProducer {
    manifest_path: String,
}

#[derive(Deserialize)]
struct LlvmExportData {
    files: Vec<LlvmExportFile>,
}

#[derive(Deserialize)]
struct LlvmExportFile {
    filename: String,
    /// LLVM export 3.x segments: line, column, count, has-count,
    /// region-entry, gap-region.
    segments: Vec<(u32, u32, u64, bool, bool, bool)>,
}

impl LlvmExport {
    fn parse(bytes: &[u8], source_root: &Path) -> Self {
        assert!(bytes.len() <= codegen::MAX_COVERAGE_BYTES);
        let export: Self = serde_json::from_slice(bytes).unwrap();
        assert_eq!(export.kind, "llvm.coverage.json.export");
        // The segment tuple layout read below is the 3.1.0 export format's.
        assert_eq!(export.version, "3.1.0");
        assert_eq!(
            Path::new(&export.cargo_llvm_cov.manifest_path),
            source_root.join("Cargo.toml")
        );
        assert!(!export.data.is_empty());
        export
    }

    /// Execution count at a generated source-map probe.
    fn probe_count(&self, source_root: &Path, region: &codegen::SourceRegion) -> u64 {
        let expected = source_root.join(&region.artifact_path);
        let file = self
            .data
            .iter()
            .flat_map(|data| &data.files)
            .find(|file| Path::new(&file.filename) == expected)
            .unwrap_or_else(|| panic!("missing generated coverage file {}", expected.display()));
        let probe = region.probe.expect("generated semantic probe");
        let start = (probe.line, probe.start_column);
        let end = (probe.line, probe.end_column);
        file.segments
            .windows(2)
            .find_map(|pair| {
                let (segment, next) = (pair[0], pair[1]);
                let covers = (segment.0, segment.1) <= start && end <= (next.0, next.1);
                (covers && segment.3 && !segment.5).then_some(segment.2)
            })
            .unwrap_or_else(|| {
                panic!(
                    "unmeasured generated probe {}:{}:{}",
                    region.artifact_path, probe.line, probe.start_column
                )
            })
    }
}

#[test]
#[trace(
    "IT-008-SC-01",
    "IT-008-SC-02",
    "IT-008-SC-03",
    "IT-008-SC-04",
    "TC-094"
)]
fn generated_proptest_and_old_profile_activation_match_reference() {
    // Left-antecedent implication, outer implication, then right-consequent
    // implication: this catches source-map order accidentally using preorder.
    const EXPRESSION: &str = "(a implies b) implies (c implies (not a or not b and c))";
    let boolean = json!({"kind": "boolean"});
    let models = [support::rule_model(&[
        ("a", boolean.clone()),
        ("b", boolean.clone()),
        ("c", boolean),
    ])];
    let package = support::rule_package(&models, EXPRESSION);
    let unit = package.checked().linked().unit();
    assert_eq!(unit.language().value, "ix:native");
    assert_eq!(unit.edition().value, "0-draft");
    assert_eq!(
        unit.source().text().lines().nth(1),
        Some("profile \"state-finite/0-draft\";")
    );
    let projection = lowering::lower(&package, LoweringLimits::default()).unwrap();
    assert_eq!(projection.profile(), "boolean-oracle/v1");
    let consumer = ir::BoundPackage::from_json_bytes(projection.bytes()).unwrap();
    assert_eq!(consumer.digest(), projection.bound().digest());
    let generated = codegen::generate_bound_oracles(&consumer).unwrap();
    let codegen::BoundOracleGeneration::Generated(generated_clauses) = &generated else {
        panic!("complete executable population required")
    };
    assert_eq!(generated_clauses.clauses().len(), 2);
    let strategy = codegen::generate_i64_strategy(&codegen::StrategyRequest {
        requirement: generated_clauses.clauses()[0].identity().requirement(),
        strategy_id: "boolean-oracle-v1-complete-domain",
        constraint: codegen::StrategyConstraint::Membership {
            values: &[0, 1, 2, 3, 4, 5, 6, 7],
        },
        campaign: codegen::StrategyCampaign::Broad,
    })
    .unwrap();

    let root = tempfile::tempdir().unwrap();
    let source_root = root.path().join("package");
    codegen::write_bundle_atomic(generated_clauses.bundle(), &source_root).unwrap();
    fs::write(
        source_root.join(&strategy.rust.path),
        &strategy.rust.contents,
    )
    .unwrap();
    let mut program = String::from("// SPDX-License-Identifier: AGPL-3.0-or-later\n");
    let mut calls = Vec::new();
    for (index, clause) in generated_clauses.clauses().iter().enumerate() {
        let artifact = &clause.bundle().rust;
        let function = public_function(&artifact.contents);
        let arguments = parameter_names(&function)
            .iter()
            .map(|name| match name.as_str() {
                "a_current" => "a",
                "b_current" => "b",
                "c_current" => "c",
                name => panic!("unexpected generated parameter {name}"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            program,
            "#[allow(dead_code)] #[path = {:?}] mod clause_{index};",
            artifact.path.strip_prefix("src/").unwrap()
        )
        .unwrap();
        calls.push(format!(
            "clause_{index}::{}({arguments})",
            function.sig.ident
        ));
        let map: Vec<codegen::SourceRegion> =
            serde_json::from_str(&clause.bundle().source_map.contents).unwrap();
        assert!(map
            .iter()
            .all(|region| region.package_id == "example/runtime-rules"
                && region.requirement_id == "PopulationRule"
                && region.requirement_revision == 7
                && region.clause_id == clause.identity().clause().as_str()));
    }
    let strategy_function = public_function(&strategy.rust.contents).sig.ident;
    writeln!(
        program,
        "#[path = {:?}] mod boolean_oracle_v1_domain;",
        strategy.rust.path.strip_prefix("src/").unwrap()
    )
    .unwrap();
    write!(
        program,
        r#"fn evaluate(a: bool, b: bool, c: bool) -> [bool; 2] {{ [{}, {}] }}
fn main() {{
    let values = std::env::args().skip(1).collect::<Vec<_>>();
    if values.is_empty() {{
        use proptest::strategy::Strategy as _;
        let mut runner = proptest::test_runner::TestRunner::deterministic();
        let strategy = boolean_oracle_v1_domain::{strategy_function}();
        runner.run(&strategy, |case| {{
            proptest::prop_assert!(!case.expected.expects_rejection());
            proptest::prop_assert_eq!(case.related, None);
            let bits = u8::try_from(case.primary).unwrap();
            let [a, b, c] = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0];
            let [constant, rule] = evaluate(a, b, c);
            println!("{{bits}} {{constant}} {{rule}}");
            Ok(())
        }}).unwrap();
        return;
    }}
    assert_eq!(values.len(), 3);
    let a: bool = values[0].parse().unwrap();
    let b: bool = values[1].parse().unwrap();
    let c: bool = values[2].parse().unwrap();
    println!("{{:?}}", evaluate(a, b, c));
}}
"#,
        calls[0], calls[1]
    )
    .unwrap();
    fs::write(source_root.join("src/main.rs"), program).unwrap();
    fs::write(
        source_root.join("Cargo.toml"),
        support::runtime_manifest(
            "native-boolean-parity",
            &[],
            "proptest = { version = \"=1.5.0\", default-features = false, features = [\"std\"] }\n",
        ),
    )
    .unwrap();

    let version = String::from_utf8(run(Command::new("rustc").arg("-vV")).stdout).unwrap();
    let host = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap();
    let sysroot =
        String::from_utf8(run(Command::new("rustc").args(["--print", "sysroot"])).stdout).unwrap();
    let llvm = PathBuf::from(sysroot.trim())
        .join("lib/rustlib")
        .join(host)
        .join("bin");
    for tool in ["llvm-cov", "llvm-profdata"] {
        assert!(
            llvm.join(tool).is_file(),
            "{tool} is missing: install the llvm-tools component"
        );
    }
    let target = root.path().join("target");
    run(Command::new(env!("CARGO"))
        .args(["build", "--offline", "-j", "1", "--target-dir"])
        .arg(&target)
        .current_dir(&source_root)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-C instrument-coverage"));
    let executable = target.join("debug/native-boolean-parity");
    let property_raw = root.path().join("property.profraw");
    let property_output =
        run(Command::new(&executable).env("LLVM_PROFILE_FILE", &property_raw)).stdout;
    fs::remove_file(property_raw).unwrap();
    let mut property_truth = [None; 8];
    for line in String::from_utf8(property_output).unwrap().lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        let bits = fields[0].parse::<usize>().unwrap();
        assert!(bits < property_truth.len());
        assert!(fields[1].parse::<bool>().unwrap());
        let truth = fields[2].parse::<bool>().unwrap();
        if let Some(previous) = property_truth[bits] {
            assert_eq!(previous, truth, "generated proptest assignment {bits}");
        }
        property_truth[bits] = Some(truth);
    }
    assert!(property_truth.iter().all(Option::is_some));
    run(Command::new(env!("CARGO")).args(["llvm-cov", "--version"]));
    // Isolate this run's measured binary and profiles from the build target so
    // stale executables or profiles cannot participate in the report.
    let report_target = root.path().join("coverage");
    fs::create_dir_all(report_target.join("debug")).unwrap();
    fs::copy(
        &executable,
        report_target.join("debug/native-boolean-parity"),
    )
    .unwrap();
    // Re-read the published bytes compilation used, including maps and
    // unsealed generation bodies, before asking codegen's coverage consumer.
    let artifacts = generated_clauses
        .bundle()
        .artifacts()
        .iter()
        .map(|artifact| {
            (
                artifact.path.clone(),
                fs::read(source_root.join(&artifact.path)).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let artifacts = artifacts
        .iter()
        .map(|(path, bytes)| codegen::ArtifactBytes { path, bytes })
        .collect::<Vec<_>>();
    for (bits, property_expected) in property_truth.into_iter().enumerate() {
        let [a, b, c] = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0];
        // Independent truth and control equations for this fixed fixture.
        let antecedent = !a || b;
        let expected = !antecedent || !c || !a || !b && c;
        let expected_entries = [
            u64::from(a),
            u64::from(antecedent),
            u64::from(antecedent && c),
        ];
        let snapshot = support::node_snapshot(
            &models[0],
            &[
                ("a", ValueNode::Boolean { value: a }),
                ("b", ValueNode::Boolean { value: b }),
                ("c", ValueNode::Boolean { value: c }),
            ],
        );
        let selected = support::rule_selection(&models[0], &snapshot);
        let context = validate(
            package.checked(),
            RuntimeInput {
                snapshots: vec![snapshot],
                invocations: Vec::new(),
            },
            selected,
            ValidationLimits::default(),
            || false,
        )
        .unwrap();
        let native = evaluate(&context, EvaluationLimits::default(), || false);
        assert_eq!(
            native.outcome(),
            &EvaluationOutcome::Completed(expected),
            "assignment {bits}"
        );
        assert_eq!(
            property_expected,
            Some(expected),
            "generated proptest assignment {bits}"
        );
        let mut entries = [0_u64; 3];
        for event in native
            .events()
            .iter()
            .filter(|event| event.kind == ImplicationEventKind::ConsequentEntered)
        {
            let expression = unit.expression(event.implication).unwrap();
            let ordinal = match &unit.source().text()[expression.span.start..expression.span.end] {
                "a implies b" => 0,
                EXPRESSION => 1,
                "c implies (not a or not b and c)" => 2,
                other => panic!("foreign implication source: {other}"),
            };
            entries[ordinal] += 1;
        }
        assert_eq!(
            entries, expected_entries,
            "native activation assignment {bits}"
        );

        let raw = report_target.join("case.profraw");
        let output = run(Command::new(&executable)
            .args([a.to_string(), b.to_string(), c.to_string()])
            .env("LLVM_PROFILE_FILE", &raw))
        .stdout;
        assert_eq!(
            serde_json::from_slice::<Vec<bool>>(&output).unwrap(),
            [true, expected]
        );
        let export = run(Command::new(env!("CARGO"))
            .args(["llvm-cov", "report", "--json", "--offline"])
            .current_dir(&source_root)
            .env("CARGO_LLVM_COV_TARGET_DIR", &report_target)
            .env("LLVM_COV", llvm.join("llvm-cov"))
            .env("LLVM_PROFDATA", llvm.join("llvm-profdata"))
            .env("LLVM_PROFDATA_FLAGS", "-num-threads=1")
            .env("CARGO_BUILD_JOBS", "1"))
        .stdout;
        fs::remove_file(&raw).unwrap();
        let exact_profile = LlvmExport::parse(&export, &source_root);
        let rule_map: Vec<codegen::SourceRegion> =
            serde_json::from_str(&generated_clauses.clauses()[1].bundle().source_map.contents)
                .unwrap();
        let generated_entries = rule_map[2..]
            .iter()
            .map(|region| exact_profile.probe_count(&source_root, region))
            .collect::<Vec<_>>();
        assert_eq!(
            generated_entries, entries,
            "generated activation assignment {bits}"
        );
        // Codegen's coverage consumer knows only the bound-oracle bundle's own
        // files, so the strategy file measured beside them is foreign to it.
        let report = codegen::analyze_bound_coverage(
            &consumer,
            &generated,
            codegen::BoundCoverageInputs {
                source_root: source_root.to_str().unwrap(),
                artifacts: &artifacts,
                llvm_export: Some(&export),
            },
        );
        let report: Value = serde_json::from_slice(&report.to_json_bytes().unwrap()).unwrap();
        assert_eq!(report["state"], "invalid_input", "{report}");
        assert_eq!(report["diagnostics"][0]["code"], "foreign_generated_file");
        let message = report["diagnostics"][0]["message"].as_str().unwrap();
        assert!(message.contains("foreign generated source"));
    }
}

// ---------------------------------------------------------------------------
// IT-010-SC-02..04: ConfigVersion VersionUnchanged through state-scalar IR.
// ---------------------------------------------------------------------------

const CORPUS: [i64; 4] = [-1, 0, 1000, 1001];

fn config_bound(native: &NativePackage<'_>) -> ir::BoundPackage {
    let projection = lowering::lower_for(
        native,
        ProjectionTarget::StateScalarIrV1,
        LoweringLimits::default(),
    )
    .unwrap();
    let bound = ir::BoundPackage::from_json_bytes(projection.bytes()).unwrap();
    assert_eq!(&bound, projection.bound());
    bound
}

/// A compiled generated ConfigVersion oracle, invoked only on in-domain input.
struct OracleExecutable {
    _crate: GeneratedCrate,
    path: PathBuf,
    calls: std::cell::Cell<u64>,
}

impl OracleExecutable {
    fn compile(source: &str) -> Self {
        let function = public_function(source);
        let arguments = parameter_names(&function)
            .iter()
            .map(|name| {
                if name.ends_with("_pre") {
                    "pre"
                } else if name.ends_with("_post") {
                    "post"
                } else {
                    panic!("unexpected ConfigVersion oracle argument {name}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let generated = GeneratedCrate::new("configversion-oracle", &[], "");
        generated.write("src/oracle.rs", source);
        generated.write(
            "src/main.rs",
            &format!(
                "#![deny(warnings)]\n#[allow(dead_code)] #[path = \"oracle.rs\"] mod oracle;\nfn main() {{ let mut values = std::env::args().skip(1); let pre: i64 = values.next().unwrap().parse().unwrap(); let post: i64 = values.next().unwrap().parse().unwrap(); assert!(values.next().is_none()); println!(\"{{}}\", oracle::{}({arguments})); }}\n",
                function.sig.ident
            ),
        );
        let path = generated.build_binary();
        Self {
            _crate: generated,
            path,
            calls: std::cell::Cell::new(0),
        }
    }

    /// The generated verdict, or `None` without a call for out-of-domain input.
    fn bounded_verdict(&self, pre: i64, post: i64) -> Option<bool> {
        if !CONFIG_DOMAIN.contains(&pre) || !CONFIG_DOMAIN.contains(&post) {
            return None;
        }
        self.calls.set(self.calls.get() + 1);
        let output = run(Command::new(&self.path).args([pre.to_string(), post.to_string()]));
        Some(String::from_utf8(output.stdout).unwrap().trim() == "true")
    }
}

#[test]
#[trace("IT-010-SC-02", "IT-010-SC-03")]
fn compiled_state_oracle_and_native_execute_agree_on_complete_corpus() {
    let models = [support::config_model()];
    let native = support::config_package(&models);
    let consumer = config_bound(&native);
    let codegen::BoundOracleGeneration::Generated(generated) =
        codegen::generate_bound_oracles(&consumer).unwrap()
    else {
        panic!("ConfigVersion has one executable state comparison")
    };
    let clause = bound_clause(&consumer, "version_unchanged");
    let generated_clause = generated
        .clauses()
        .iter()
        .find(|candidate| candidate.identity() == clause.identity())
        .unwrap();
    let map: Vec<codegen::SourceRegion> =
        serde_json::from_str(&generated_clause.bundle().source_map.contents).unwrap();
    assert!(!map.is_empty());
    assert!(map.iter().all(|region| {
        region.package_id == "example/config-version"
            && region.requirement_id == "VersionUnchanged"
            && region.requirement_revision == 1
            && region.clause_id == "version_unchanged"
    }));
    let oracle = OracleExecutable::compile(&generated_clause.bundle().rust.contents);
    for pre in CORPUS {
        for post in CORPUS {
            let calls = oracle.calls.get();
            let generated = oracle.bounded_verdict(pre, post);
            let native_result = support::config_native_verdict(&native, &models[0], pre, post);
            assert_eq!(generated, native_result, "pre={pre}, post={post}");
            if CONFIG_DOMAIN.contains(&pre) && CONFIG_DOMAIN.contains(&post) {
                assert_eq!(generated, Some(pre == post));
                assert_eq!(oracle.calls.get(), calls + 1);
            } else {
                assert_eq!(generated, None);
                assert_eq!(
                    oracle.calls.get(),
                    calls,
                    "invalid input invoked Boolean oracle"
                );
            }
        }
    }
}

/// The name of the first generated item on a line starting with `prefix`.
fn generated_item(source: &str, prefix: &str) -> String {
    source
        .lines()
        .find_map(|line| {
            line.starts_with(prefix).then(|| {
                line.split_whitespace()
                    .nth(2)
                    .expect("generated public item name")
                    .split(['(', '<', ' ', '{', ':'])
                    .next()
                    .unwrap()
                    .to_owned()
            })
        })
        .unwrap_or_else(|| panic!("generated source has no item beginning {prefix:?}"))
}

/// The generated campaign runner whose signature continues with `marker`.
fn campaign_runner(source: &str, marker: &str, end: char) -> String {
    source
        .lines()
        .find_map(|line| {
            line.strip_prefix("pub fn bound_campaign_")
                .filter(|tail| tail.contains(marker))
                .map(|tail| format!("bound_campaign_{}", tail.split(end).next().unwrap()))
        })
        .unwrap_or_else(|| panic!("generated source has no {marker} campaign runner"))
}

/// Rename the generated oracle and wrap it so every call records its
/// `(pre, post, verdict)` observation, leaving the generated body untouched.
fn instrumented_strategy_source(source: &str) -> String {
    let oracle = generated_item(source, "pub fn oracle_");
    let function = syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == oracle => Some(function),
            _ => None,
        })
        .unwrap();
    let arguments = parameter_names(&function);
    assert_eq!(arguments.len(), 2);
    let pre = arguments
        .iter()
        .find(|argument| argument.ends_with("_pre"))
        .expect("generated state strategy has a pre observation");
    let post = arguments
        .iter()
        .find(|argument| argument.ends_with("_post"))
        .expect("generated state strategy has a post observation");
    let mut instrumented = source.replacen(
        &format!("pub fn {oracle}("),
        &format!("fn {oracle}_uninstrumented("),
        1,
    );
    writeln!(
        instrumented,
        r#"
thread_local! {{
    static OBSERVATIONS: core::cell::RefCell<Vec<(i64, i64, bool)>> = const {{ core::cell::RefCell::new(Vec::new()) }};
}}

/// Instrumented call preserving the generated oracle body.
pub fn {oracle}({first}: i64, {second}: i64) -> bool {{
    let result = {oracle}_uninstrumented({first}, {second});
    OBSERVATIONS.with(|values| values.borrow_mut().push(({pre}, {post}, result)));
    result
}}

/// Drain the exact values observed by this generated campaign.
pub fn take_observations() -> Vec<(i64, i64, bool)> {{
    OBSERVATIONS.with(|values| core::mem::take(&mut *values.borrow_mut()))
}}
"#,
        first = arguments[0],
        second = arguments[1],
    )
    .unwrap();
    instrumented
}

const CAMPAIGN_REPORT: &str = r#"quire_contract_runtime::CampaignReport::new(
            quire_contract_runtime::ContractIdentity::new(
                quire_contract_runtime::RequirementId::new("VersionUnchanged"),
                quire_contract_runtime::RevisionId::new("1"),
            ),
        )"#;

#[test]
#[trace("IT-010-SC-02", "IT-010-SC-04")]
fn generated_proptest_populations_execute_in_domain_with_zero_discards() {
    let models = [support::config_model()];
    let native = support::config_package(&models);
    let bound = config_bound(&native);
    let clause = bound_clause(&bound, "version_unchanged");

    let generated_crate = GeneratedCrate::new(
        "configversion-strategies",
        &[],
        "proptest = { version = \"=1.5.0\", default-features = false, features = [\"std\"] }\n",
    );
    let mut root =
        String::from("#![deny(warnings)]\n//! Executed IT-010 strategy populations.\n\n");
    let mut checks = String::from("#[cfg(test)]\nmod checks {\n");
    for (module, population) in [
        ("satisfying", codegen::BoundStrategyPopulation::Satisfying),
        ("violating", codegen::BoundStrategyPopulation::Violating),
        ("broad", codegen::BoundStrategyPopulation::Broad),
        ("boundary", codegen::BoundStrategyPopulation::Boundary),
    ] {
        let generated = codegen::generate_bound_strategy(&codegen::BoundStrategyRequest {
            package: &bound,
            clause: clause.identity(),
            population,
            minimum_accepted_cases: 1,
            minimum_rejected_cases: 0,
            maximum_discarded_cases: 0,
        })
        .unwrap();
        let source = &generated.rust.contents;
        generated_crate.write(
            &format!("src/{module}.rs"),
            &instrumented_strategy_source(source),
        );
        writeln!(
            root,
            "#[allow(dead_code)] #[path = \"{module}.rs\"] pub mod {module};"
        )
        .unwrap();
        let failure_assertion = match population {
            codegen::BoundStrategyPopulation::Satisfying => "assert_eq!(summary.failed, 0);",
            codegen::BoundStrategyPopulation::Violating => {
                "assert_eq!(summary.failed, summary.attempted);"
            }
            codegen::BoundStrategyPopulation::Broad
            | codegen::BoundStrategyPopulation::Boundary => {
                "assert!(summary.failed > 0 && summary.failed < summary.attempted);"
            }
        };
        if population == codegen::BoundStrategyPopulation::Boundary {
            let runner = campaign_runner(source, "_run_census(", '(');
            writeln!(
                checks,
                r#"
    #[test]
    fn boundary_census_executes_exact_in_domain_cases() {{
        let mut report = {CAMPAIGN_REPORT};
        let summary = super::boundary::{runner}(&mut report).unwrap();
        let observed = super::boundary::take_observations();
        assert_eq!(observed.len(), usize::try_from(summary.attempted).unwrap());
        assert!(observed.iter().all(|(pre, post, _)| (0..=1000).contains(pre) && (0..=1000).contains(post)));
        for (pre, post, generated) in observed {{ println!("IT010-PAIR:{{pre}}:{{post}}:{{generated}}"); }}
        assert_eq!(summary.accepted, summary.attempted);
        {failure_assertion}
        assert_eq!(summary.discarded, 0);
        assert_eq!(summary.discard_rate(), Some((0, summary.attempted)));
        println!("IT010-BOUNDARY-COUNT:{{}}", summary.attempted);
    }}
"#
            )
            .unwrap();
        } else {
            let strategy = generated_item(source, "pub fn bound_strategy_");
            let runner = campaign_runner(source, "_run<Strategy>", '<');
            writeln!(
                checks,
                r#"
    #[test]
    fn {module}_sampled_campaign_is_in_domain_and_reports_zero_discards() {{
        let config = proptest::test_runner::Config {{
            cases: 256,
            max_global_rejects: 0,
            failure_persistence: None,
            ..proptest::test_runner::Config::default()
        }};
        let mut runner = proptest::test_runner::TestRunner::new_with_rng(
            config,
            proptest::test_runner::TestRng::deterministic_rng(
                proptest::test_runner::RngAlgorithm::ChaCha,
            ),
        );
        let strategy = super::{module}::{strategy}();
        let mut report = {CAMPAIGN_REPORT};
        let summary = super::{module}::{runner}(&mut runner, &strategy, &mut report).unwrap();
        let observed = super::{module}::take_observations();
        assert_eq!(observed.len(), 256);
        assert!(observed.iter().all(|(pre, post, _)| (0..=1000).contains(pre) && (0..=1000).contains(post)));
        for (pre, post, generated) in observed {{ println!("IT010-PAIR:{{pre}}:{{post}}:{{generated}}"); }}
        assert_eq!(summary.attempted, 256);
        assert_eq!(summary.accepted, 256);
        assert_eq!(summary.rejected, 0);
        {failure_assertion}
        assert_eq!(summary.discarded, 0);
        assert_eq!(summary.discard_rate(), Some((0, 256)));
    }}
"#
            )
            .unwrap();
        }
    }
    checks.push_str("}\n");
    root.push_str(&checks);
    generated_crate.write("src/lib.rs", &root);
    let output = generated_crate.cargo("test", &["--nocapture", "--test-threads=1"]);
    assert!(
        output.status.success(),
        "generated strategy campaigns failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = String::from_utf8(output.stdout).unwrap();
    let mut compared = 0_u64;
    let mut boundary_count = None;
    for line in output.lines() {
        if let Some((_, value)) = line.split_once("IT010-BOUNDARY-COUNT:") {
            boundary_count = Some(value.parse::<u64>().unwrap());
            continue;
        }
        let Some((_, values)) = line.split_once("IT010-PAIR:") else {
            continue;
        };
        let [pre, post, generated] = values.split(':').collect::<Vec<_>>()[..] else {
            panic!("malformed generated observation {line}")
        };
        let pre = pre.parse().unwrap();
        let post = post.parse().unwrap();
        let generated = generated.parse().unwrap();
        assert_eq!(
            support::config_native_verdict(&native, &models[0], pre, post),
            Some(generated),
            "generated strategy/native disagreement for pre={pre}, post={post}"
        );
        compared += 1;
    }
    assert_eq!(
        compared,
        3 * 256 + boundary_count.expect("generated boundary count was reported")
    );
}
