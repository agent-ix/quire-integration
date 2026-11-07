// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original-source packages for capture and shared-declaration correspondence.

use qsl_eval::temporal::evidence::{CaptureFact, CapturedValue, TraceBinding};
use qsl_package::checked_views::{mapping_support, temporal_subject, ClauseOccurrence, Closure};
use qsl_semantics::{
    check::PackageDeclarations,
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedNodeKind, CheckedOccurrence, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, ClockConfiguration,
    TemporalForm,
};
use quire_integration::temporal::NativeProjectionInput;
use quire_semantic_value::checking::CheckingLimits;
use quire_spec_language::temporal::{ValuationEvidence, ValuationRow};

use crate::{
    projection_support::{self, Case, EVENT},
    support,
};

pub(super) fn declarations(body: &str) -> String {
    format!("temporal Due using t over (g: M::Gadget) clock \"events\" on origin {{ {body} }}")
}

pub(super) fn package(declarations: &str) -> CheckedPackageV2 {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.model.complete/v1\";\n\
         profile t = \"{EVENT}\";\n\
         model M = \"acme/orders\";\n{declarations}\n"
    );
    let model =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("original model evidence");
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "test:temporal-binding-axes"),
        "temporal-binding-axes.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original temporal source parses");
    assert!(parsed.diagnostics().is_empty());
    let reference = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("native forms");
    let models = admit_unit(
        &unit.selections().models,
        &package_input([model.as_slice()]).expect("model input"),
        ModelNormalizationLimits::default(),
    )
    .expect("model admission");
    let graph = PackageDeclarations::assemble(
        reference,
        "agent-ix/temporal-binding-axes".into(),
        unit,
        models,
        Vec::new(),
    )
    .expect("package assembly")
    .check(CheckingLimits::default())
    .expect("native checker");
    let clocks = graph
        .temporal_clauses()
        .iter()
        .map(|declaration| {
            qsl_package::TemporalClockSelection::new(
                graph.source().clone(),
                declaration.clause().span(),
                ClockConfiguration::EventPosition {
                    sequence_authority: "events".into(),
                },
            )
        })
        .collect::<Vec<_>>();
    let emitted = qsl_package::emit_checked_with_temporal_selections(
        &qsl_package::CheckedPackage::link(graph),
        &clocks,
        qsl_package::EmitLimits::new(16 * 1024 * 1024),
    )
    .expect("actual native emitter");
    assert!(emitted.omitted().is_empty());
    let bytes = emitted.package().bytes();
    let wire: serde_json::Value = serde_json::from_slice(bytes).expect("emitted package JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature identity"));
    }
    evidence
        .insert_domain_package_document("acme/orders", &model)
        .expect("reader model evidence");
    match read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("strict IR admission: {other:?}"),
    }
}

pub(super) fn with_source(
    declarations: &str,
    selected_name: &str,
    captures: &[&[u8]],
    test: impl FnOnce(Case<'_>),
) {
    projection_support::with_case("holds(true)", false, |base| {
        let package = package(declarations);
        let mut selected: Option<ClauseOccurrence> = None;
        for (node, kind) in package.graph().nodes.iter().zip(package.node_kinds()) {
            if *kind != CheckedNodeKind::Temporal(TemporalForm::TemporalClause) {
                continue;
            }
            let subject = temporal_subject(&package, &node.node_id).expect("actual temporal view");
            for occurrence in subject.occurrences() {
                if subject.name_of(occurrence) == Some(selected_name) {
                    assert!(
                        selected.replace(occurrence.clone()).is_none(),
                        "one selected authored occurrence"
                    );
                }
            }
        }
        let occurrence = selected.expect("authored occurrence exists");
        let subject = temporal_subject(&package, &occurrence.node).expect("selected view");
        let classification = mapping_support(&subject, Closure::Complete).expect("classification");
        let mut request = base.native.request.clone();
        request.occurrence = Some(CheckedOccurrence {
            role: occurrence.role.clone(),
            ordinal: occurrence.ordinal,
        });
        let clock = subject
            .clock_of(&occurrence)
            .expect("actual selected clock row");
        request.binding = TraceBinding {
            profile: subject.profile().definition.identity.clone(),
            clock_name: clock.clock_name.clone(),
            parameters: clock.clock.clone(),
        };
        request.origins[0].captures = captures
            .iter()
            .map(|value| CaptureFact::Value {
                anchor: base.native.instance.anchor.into(),
                value: CapturedValue::new(value.to_vec()),
            })
            .collect();
        let rows = subject
            .predicate_leaves()
            .iter()
            .map(|operand| ValuationRow {
                subject: occurrence.clone(),
                position: 0,
                anchor: base.native.instance.anchor.into(),
                capture_instance: base.native.instance.capture_instance.into(),
                revision: base.native.observation.revision.clone(),
                operand: operand.clone(),
                value: true,
            })
            .collect();
        let evidence = ValuationEvidence::admit(
            &subject,
            &projection_support::native_trace(EVENT, base.native.instance.anchor),
            base.native.instance,
            &base.native.observation.revision,
            rows,
        )
        .expect("actual native valuation owner");
        test(Case {
            package: &package,
            subject: &subject,
            classification: &classification,
            surrounding: Closure::Complete,
            native: NativeProjectionInput {
                request: &request,
                valuations: &evidence,
                occurrence: &occurrence,
                instance: base.native.instance,
                observation: base.native.observation,
            },
            selection: base.selection,
        });
    });
}
