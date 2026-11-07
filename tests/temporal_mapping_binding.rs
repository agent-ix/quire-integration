// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350 input binding through the public QSL producer and strict IR reader.

mod support;

#[path = "support/observation_revision/mod.rs"]
#[allow(dead_code)] // This test uses only the observation slice of the shared fixture.
mod observation_support;

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
use quire_integration::temporal::{admit_native_tl_input, NativeInputCode, NativeInputLimits};
use quire_integration::temporal::{validate_mapping_binding, MappingBindingRefusal};
use quire_semantic_value::checking::CheckingLimits;

/// Emit once and admit twice: the packages have equal contents and distinct owners.
fn packages() -> (CheckedPackageV2, CheckedPackageV2, CheckedNodeId) {
    packages_with_clock(
        "quire.temporal.event-position.false-extension/v1",
        ClockConfiguration::EventPosition {
            sequence_authority: "events".into(),
        },
    )
}

fn packages_with_clock(
    profile: &str,
    clock: ClockConfiguration,
) -> (CheckedPackageV2, CheckedPackageV2, CheckedNodeId) {
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
"#
    .replace("quire.temporal.event-position.false-extension/v1", profile);
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
        .map(|clause| qsl_package::TemporalClockSelection {
            source: graph.source().clone(),
            declaration_span: clause.clause().span(),
            clock: clock.clone(),
        })
        .collect();
    let emitted = qsl_package::emit_checked_with_temporal_selections(
        &qsl_package::CheckedPackage::link(graph),
        &selections,
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
#[trace("TC-461")]
#[test]
fn equal_admissions_do_not_authorize_a_foreign_subject_instance() {
    let (first, second, clause) = packages();
    let subject = temporal_subject(&first, &clause).expect("first subject reads");
    let other = temporal_subject(&second, &clause).expect("second subject reads");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let other_classification = mapping_support(&other, Closure::Complete).expect("classification");
    assert_eq!(
        classification, other_classification,
        "equal content/name is insufficient"
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
}

/// IR-1378: the native request is retained and its opaque anchor is matched
/// to an independently admitted observation before exposing a numeric anchor.
#[trace(
    "TC-461",
    "IT-022-SC-01",
    "IT-022-SC-02",
    "IT-022-SC-03",
    "IT-022-SC-04",
    "IT-022-SC-05",
    "IT-022-SC-06"
)]
#[test]
fn native_request_selects_exact_observation_and_keeps_valuations() {
    use qsl_eval::temporal::evidence::{
        EvidencePosition, EvidenceRef, EvidenceRequest, ExecutionDisposition, Relation,
        SemanticTriggerIdentity, TemporalEvidenceLimits, TraceBinding, TriggerEvidence,
    };
    use qsl_eval::temporal::{ClockCover, ProgressClosure, Valuation};

    let (package, foreign, clause) = packages_with_clock(
        "quire.temporal.timestamped-event.finite-window/v1",
        ClockConfiguration::TimestampedEvent {
            timestamp_unit: "nanosecond".into(),
        },
    );
    let subject = temporal_subject(&package, &clause).unwrap();
    let fixture = observation_support::Fixture::new();
    let observation = fixture.observation(fixture.observation_document(), 1, None);
    let identity = observation.payload().observation_identity().to_owned();
    let trigger = SemanticTriggerIdentity::new(b"instance-a".as_slice()).unwrap();
    let position = EvidencePosition {
        observation: EvidenceRef::named(identity.clone()),
        position: 5,
        valuations: [(7, Valuation::False)].into(),
        instance_valuations: [(trigger.clone(), [(9, Valuation::True)].into())].into(),
        ..EvidencePosition::default()
    };
    let clock = subject.clock().unwrap();
    let request = EvidenceRequest {
        trigger: Some(trigger.clone()),
        correspondence: EvidenceRef::named("correspondence"),
        positions: vec![position],
        anchor: Some(identity.clone()),
        deliveries: vec![],
        origins: vec![],
        trigger_evidence: TriggerEvidence::Admitted,
        trigger_scope: ProgressClosure::Closed,
        decision_progress: None,
        decision_closure: None,
        surrounding_progress: None,
        surrounding_closure: None,
        execution: ExecutionDisposition::Completed,
        completeness: None,
        authoritative_origin: None,
        evicted: vec![],
        cover: ClockCover::Exact,
        progress_ledger: None,
        ledger: None,
        relation: Relation::Original,
        control: None,
        binding: TraceBinding {
            profile: subject.profile().definition.identity.clone(),
            clock_name: clock.clock_name.clone(),
            parameters: clock.clock.clone(),
        },
        occurrence: None,
        limits: TemporalEvidenceLimits::default(),
    };

    let limits = NativeInputLimits::default();
    let admitted =
        admit_native_tl_input(&package, &subject, &request, &observation, limits).unwrap();
    assert_eq!(admitted.limits(), limits);
    assert_eq!(admitted.selected_model_packages(), &["acme/orders"]);
    assert_eq!(
        admit_native_tl_input(
            &package,
            &subject,
            &request,
            &observation,
            NativeInputLimits { max_positions: 1 },
        )
        .unwrap()
        .anchor(),
        Some(5)
    );
    assert_eq!(
        admit_native_tl_input(
            &package,
            &subject,
            &request,
            &observation,
            NativeInputLimits { max_positions: 0 },
        )
        .unwrap_err()
        .code(),
        NativeInputCode::LimitReached
    );
    assert_eq!(admitted.anchor(), Some(5));
    assert_eq!(admitted.occurrence(), &subject.occurrences()[0]);
    assert!(std::ptr::eq(admitted.request(), &request));
    assert!(std::ptr::eq(admitted.observation(), &observation));
    assert_eq!(admitted.trigger(), Some(&trigger));
    assert_eq!(
        admitted.positions()[0].valuations.get(&7),
        Some(&Valuation::False)
    );
    assert_eq!(
        admitted.positions()[0].instance_valuations[&trigger].get(&9),
        Some(&Valuation::True)
    );
    let mut explicit = request.clone();
    explicit.occurrence = Some(quire_contract_ir::CheckedOccurrence {
        role: subject.occurrences()[0].role.clone(),
        ordinal: subject.occurrences()[0].ordinal,
    });
    assert_eq!(
        admit_native_tl_input(&package, &subject, &explicit, &observation, limits)
            .unwrap()
            .occurrence(),
        &subject.occurrences()[0]
    );
    assert_eq!(
        admit_native_tl_input(&foreign, &subject, &request, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::ForeignSubject
    );

    let mut changed = request.clone();
    changed.anchor = Some("another-observation".into());
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::Anchor
    );
    changed = request.clone();
    changed.positions[0].observation.identity = "another-observation".into();
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::Anchor
    );
    changed = request.clone();
    changed.positions.push(changed.positions[0].clone());
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::Anchor
    );
    changed = request.clone();
    changed.positions[0].position = 6;
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::ObservationAnchor
    );
    changed = request.clone();
    changed.binding.profile = "foreign-profile".into();
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::Binding
    );
    changed = request.clone();
    changed.occurrence = Some(quire_contract_ir::CheckedOccurrence {
        role: subject.occurrences()[0].role.clone(),
        ordinal: subject.occurrences()[0].ordinal + 1,
    });
    assert_eq!(
        admit_native_tl_input(&package, &subject, &changed, &observation, limits)
            .unwrap_err()
            .code(),
        NativeInputCode::Occurrence
    );

    let (event_package, _, event_clause) = packages();
    let event_subject = temporal_subject(&event_package, &event_clause).unwrap();
    let event_clock = event_subject.clock().unwrap();
    let mut event_request = request.clone();
    event_request.binding = TraceBinding {
        profile: event_subject.profile().definition.identity.clone(),
        clock_name: event_clock.clock_name.clone(),
        parameters: event_clock.clock.clone(),
    };
    assert_eq!(
        admit_native_tl_input(
            &event_package,
            &event_subject,
            &event_request,
            &observation,
            limits,
        )
        .unwrap_err()
        .code(),
        NativeInputCode::ObservationAnchor
    );
}
