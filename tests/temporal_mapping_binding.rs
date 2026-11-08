// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350 input binding through the public QSL producer and strict IR reader.

mod support;

use ix_trace_rs::trace;
use qsl_package::checked_views::{mapping_support, temporal_subject, Closure, Support, Target};
use qsl_semantics::{
    check::PackageDeclarations,
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedNodeKind, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, ClockConfiguration,
    TemporalForm,
};
use quire_integration::temporal::{validate_mapping_binding, MappingBindingRefusal};
use quire_semantic_value::checking::CheckingLimits;

/// Emit once and admit twice: the packages have equal contents and distinct owners.
fn packages() -> (CheckedPackageV2, CheckedPackageV2, CheckedNodeId) {
    let document =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("the selected QSL owner fixture exists");
    let source = r#"language "ix:native" edition "1-draft";
profile v = "quire.model.complete/v1";
profile t = "quire.temporal.event-position.false-extension/v1";
model M = "acme/orders";
temporal Due using t over (g: M::Gadget) clock "events" on origin {
    eventually[0,2] holds(true)
}
"#;
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "test:temporal-binding"),
        "temporal-binding.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the temporal source parses");
    assert!(parsed.diagnostics().is_empty());
    let raw = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("the temporal unit forms");
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document.as_slice()]).expect("the domain document reads"),
        ModelNormalizationLimits::default(),
    )
    .expect("the selected model admits");
    let graph = PackageDeclarations::assemble(
        raw,
        "agent-ix/temporal-binding".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("the package assembles")
    .check(CheckingLimits::default())
    .expect("the temporal package checks");
    let selections: Vec<_> = graph
        .temporal_clauses()
        .iter()
        .map(|clause| {
            qsl_package::TemporalClockSelection::new(
                graph.source().clone(),
                clause.clause().span(),
                ClockConfiguration::EventPosition {
                    sequence_authority: "events".into(),
                },
            )
        })
        .collect();
    let emitted = qsl_package::emit_checked_with_temporal_selections(
        &qsl_package::CheckedPackage::link(graph),
        &selections,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("the producer emits the temporal package");
    assert!(emitted.omitted().is_empty());
    let bytes = emitted.package().bytes();
    let wire: serde_json::Value = serde_json::from_slice(bytes).expect("emitted JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature identity"));
    }
    evidence
        .insert_domain_package_document("acme/orders", document.as_slice())
        .expect("the original model document is reader evidence");
    let admit = || match read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence)
    {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("the strict reader admits the producer output: {other:?}"),
    };
    let first = admit();
    let second = admit();
    let clauses: Vec<_> = first
        .graph()
        .nodes
        .iter()
        .zip(first.node_kinds())
        .filter(|(_, kind)| **kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
        .map(|(node, _)| node.node_id.clone())
        .collect();
    let [clause] = clauses.as_slice() else {
        panic!("exactly one temporal clause");
    };
    (first, second, clause.clone())
}

/// This is the instance-binding precondition, not whole FR-350 projection evidence.
#[trace("IT-022-SC-01", "FR-350-AC-2")]
#[test]
fn equal_admissions_do_not_authorize_a_foreign_subject_instance() {
    let (first, second, clause) = packages();
    let subject = temporal_subject(&first, &clause).expect("first subject reads");
    let other = temporal_subject(&second, &clause).expect("second subject reads");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let other_classification = mapping_support(&other, Closure::Complete).expect("classification");
    assert_eq!(classification.support, other_classification.support);
    assert_eq!(classification.retained, other_classification.retained);
    assert_eq!(
        classification.selection().package(),
        other_classification.selection().package()
    );
    assert_eq!(
        classification.selection().definition_artifact(),
        other_classification.selection().definition_artifact()
    );
    assert_ne!(
        classification.selection().admission_binding(),
        other_classification.selection().admission_binding()
    );
    assert!(subject.belongs_to(&first));
    assert!(!subject.belongs_to(&second));
    assert_eq!(
        validate_mapping_binding(&first, &subject, &classification, Closure::Complete),
        Ok(())
    );
    assert_eq!(
        validate_mapping_binding(&second, &subject, &classification, Closure::Complete),
        Err(MappingBindingRefusal::ForeignSubject)
    );
    assert_eq!(
        validate_mapping_binding(&first, &subject, &other_classification, Closure::Complete),
        Err(MappingBindingRefusal::ForeignClassification)
    );
    assert_eq!(
        validate_mapping_binding(&first, &other, &classification, Closure::Complete),
        Err(MappingBindingRefusal::ForeignSubject)
    );
    assert_eq!(
        validate_mapping_binding(&second, &other, &other_classification, Closure::Complete),
        Ok(())
    );

    let mut altered = classification.clone();
    altered.support = Support::Unsupported { dimensions: vec![] };
    assert_eq!(
        validate_mapping_binding(&first, &subject, &altered, Closure::Complete),
        Err(MappingBindingRefusal::ClassificationMismatch)
    );
    let mut altered = classification.clone();
    altered.retained.profile.definition.authority = "foreign".into();
    assert_eq!(
        validate_mapping_binding(&first, &subject, &altered, Closure::Complete),
        Err(MappingBindingRefusal::ClassificationMismatch)
    );
    assert_eq!(
        validate_mapping_binding(&first, &subject, &classification, Closure::Open),
        Err(MappingBindingRefusal::ClassificationMismatch)
    );
    assert!(matches!(
        classification.support,
        Support::Supported {
            target: Target::ClosedTrace,
            ..
        }
    ));
    let cloned = first.clone();
    let clone_subject = temporal_subject(&cloned, &clause).expect("clone subject");
    assert_eq!(
        validate_mapping_binding(&cloned, &clone_subject, &classification, Closure::Complete),
        Ok(())
    );
    let moved = cloned;
    let moved_subject = temporal_subject(&moved, &clause).expect("moved subject");
    assert_eq!(
        validate_mapping_binding(&moved, &moved_subject, &classification, Closure::Complete),
        Ok(())
    );
}
