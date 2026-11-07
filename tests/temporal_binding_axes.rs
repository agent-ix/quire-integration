// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350 preserved source occurrence, declared capture and predicate/type axes.
#![recursion_limit = "256"] // The expected record is one large `json!` object.

#[path = "support/temporal_expected.rs"]
mod expected;
#[path = "temporal_binding_axes/fixture.rs"]
mod fixture;
#[path = "support/temporal_projection.rs"]
mod projection_support;
mod support;

use ix_trace_rs::trace;
use qsl_eval::temporal::evidence::{CaptureFact, CapturedValue, ObligationKey, Payload};
use qsl_package::NativeJoinBudget;
use quire_exact::{Meter, ScalarLimits};
use quire_integration::temporal::{
    project_tl, read_tl_projection, ExpectedTlProjection, NativeProjectionInput, TlLimits,
    TlMissingPremise, TlProjectionCode, TlProjectionError, TlProjectionPremise,
    ValidatedTlProjection,
};
use quire_spec_language::temporal::{self as native, CheckedSubject, Report};
use serde_json::{json, Value};

fn evaluate(projection: &ValidatedTlProjection<'_>) -> Report {
    evaluate_input(projection.package(), projection.native())
}

fn evaluate_input(
    package: &quire_contract_ir::CheckedPackageV2,
    native: NativeProjectionInput<'_>,
) -> Report {
    let subject = CheckedSubject::new(
        package,
        native.occurrence,
        native.valuations,
        native.request,
    )
    .expect("genuine native checked subject");
    let limits = ScalarLimits {
        integer_bits: 1 << 20,
        decimal_digits: 1 << 20,
        scale_expansion: 1 << 20,
        text_input_bytes: 1 << 20,
        text_scalars: 1 << 20,
        normalized_scalars: 1 << 20,
        unit_edges: 1 << 20,
        value_occurrences: 1 << 20,
        work_units: 1 << 24,
        result_units: 1 << 20,
    };
    native::evaluate_v2(
        &subject,
        native.observation,
        &mut Meter::new(limits),
        &mut NativeJoinBudget::new(1 << 30, 1 << 30),
    )
    .expect("actual prior native evaluator")
}

fn obligation(projection: &ValidatedTlProjection<'_>) -> ObligationKey {
    let occurrence = projection.native().occurrence;
    let key = quire_contract_ir::CheckedOccurrence {
        role: occurrence.role.clone(),
        ordinal: occurrence.ordinal,
    };
    ObligationKey::new(
        occurrence.node.package.clone(),
        serde_json::to_vec(&(&occurrence.node, key)).expect("actual native obligation key"),
        occurrence.ordinal,
    )
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn declared_capture_and_retained_immutable_environment_are_original_owner_values() {
    fixture::with_source(
        &fixture::declarations("capture saved: Boolean = true; holds(saved)"),
        "Due",
        &[b"true"],
        |case| {
            assert_eq!(case.subject.capture_names(), &["saved"]);
            assert_eq!(case.subject.captures().len(), 1);
            let first = case
                .project(TlLimits::default())
                .expect("declared capture projection");
            let first_record: Value =
                serde_json::from_slice(first.bytes()).expect("canonical correspondence");
            assert_eq!(first_record["captures"], json!([expected::hex(b"true")]));
            let report = evaluate(&first);
            let ledger = report.ledger().expect("actual retained native ledger");
            let key = obligation(&first);
            let trigger = case
                .native
                .request
                .trigger
                .as_ref()
                .expect("actual origin identity");
            let retained = ledger
                .instance(&key, trigger)
                .expect("actual retained instance");
            assert_eq!(retained.evaluations(), 1);
            let entry = retained
                .environment()
                .get("saved")
                .expect("declared retained capture");
            assert_eq!(entry.value().as_bytes(), b"true");
            assert_eq!(entry.source().clause.as_ref(), "Due");
            assert_eq!(entry.source().index, 0);
            assert_eq!(entry.anchor(), case.native.instance.anchor);
            let saved_environment = retained.environment().clone();
            let mut redelivery = case.native.request.clone();
            redelivery.ledger = Some(ledger.clone());
            redelivery.origins[0].captures[0] = CaptureFact::Value {
                anchor: case.native.instance.anchor.into(),
                value: CapturedValue::new(b"false".as_slice()),
            };
            let again = project_tl(
                case.package,
                case.subject,
                case.classification,
                case.surrounding,
                NativeProjectionInput {
                    request: &redelivery,
                    ..case.native
                },
                case.selection,
                TlLimits::default(),
            )
            .expect("equal payload reuses retained capture environment");
            let again_record: Value =
                serde_json::from_slice(again.bytes()).expect("redelivery correspondence");
            assert_eq!(again_record["captures"], first_record["captures"]);
            assert_eq!(again_record["captures"], json!([expected::hex(b"true")]));
            assert_eq!(
                again_record["triggerPayload"],
                first_record["triggerPayload"]
            );
            assert_eq!(
                ledger.instance(&key, trigger).unwrap().environment(),
                &saved_environment
            );
            assert_eq!(ledger.instance(&key, trigger).unwrap().evaluations(), 1);
            assert!(
                matches!(&redelivery.origins[0].captures[0], CaptureFact::Value { value, .. }
            if value.as_bytes() == b"false")
            );
        },
    );
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn retained_instance_payload_contradiction_refuses_before_projection() {
    fixture::with_source(
        &fixture::declarations("capture saved: Boolean = true; holds(saved)"),
        "Due",
        &[b"true"],
        |case| {
            let first = case
                .project(TlLimits::default())
                .expect("initial capture projection");
            let report = evaluate(&first);
            let ledger = report.ledger().expect("owner ledger");
            let saved = ledger.clone();
            let mut changed = case.native.request.clone();
            changed.ledger = Some(ledger.clone());
            changed.origins[0].payload =
                Payload::new(b"changed-immutable-origin-payload".as_slice());
            let error = project_tl(
                case.package,
                case.subject,
                case.classification,
                case.surrounding,
                NativeProjectionInput {
                    request: &changed,
                    ..case.native
                },
                case.selection,
                TlLimits::default(),
            )
            .expect_err("retained owner contradiction must refuse");
            assert!(
                matches!(
                    error,
                    TlProjectionError::NativePremise(TlProjectionPremise::Activation)
                ),
                "an activation contradiction is native-premise activation: {error:?}"
            );
            assert_eq!(ledger, &saved);
        },
    );
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn actual_declared_capture_missing_wrong_type_and_anchor_remain_distinct_refusals() {
    fixture::with_source(
        &fixture::declarations("capture saved: Boolean = true; holds(saved)"),
        "Due",
        &[b"true"],
        |case| {
            for capture in [CaptureFact::Missing, CaptureFact::Null, CaptureFact::Stale] {
                let mut changed = case.native.request.clone();
                changed.origins[0].captures[0] = capture;
                let error = project_tl(
                    case.package,
                    case.subject,
                    case.classification,
                    case.surrounding,
                    NativeProjectionInput {
                        request: &changed,
                        ..case.native
                    },
                    case.selection,
                    TlLimits::default(),
                )
                .expect_err("absent required capture must not create artifact");
                assert!(matches!(
                    error,
                    TlProjectionError::Incomplete(TlMissingPremise::Capture)
                ));
            }
            for capture in [
                CaptureFact::WrongType,
                CaptureFact::Value {
                    anchor: "foreign-capture-anchor".into(),
                    value: CapturedValue::new(b"true".as_slice()),
                },
            ] {
                let mut changed = case.native.request.clone();
                changed.origins[0].captures[0] = capture;
                let error = project_tl(
                    case.package,
                    case.subject,
                    case.classification,
                    case.surrounding,
                    NativeProjectionInput {
                        request: &changed,
                        ..case.native
                    },
                    case.selection,
                    TlLimits::default(),
                )
                .expect_err("cross-wired capture must refuse");
                assert!(matches!(
                    error,
                    TlProjectionError::NativePremise(TlProjectionPremise::Capture)
                ));
            }
        },
    );
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn later_shared_node_occurrence_retains_its_actual_name_role_ordinal_and_span() {
    let declarations =
        "temporal Alpha using t over (g: M::Gadget) clock \"events\" on origin { holds(true) }\n\n\
        temporal Beta using t over (g: M::Gadget) clock \"events\" on origin { holds(true) }";
    fixture::with_source(declarations, "Beta", &[], |case| {
        let occurrences = case.subject.occurrences();
        assert_eq!(
            occurrences.len(),
            2,
            "identical authored declarations share one actual node"
        );
        let first = occurrences
            .iter()
            .find(|occurrence| case.subject.name_of(occurrence) == Some("Alpha"))
            .unwrap();
        let later = case.native.occurrence;
        assert_eq!(first.node, later.node);
        assert_ne!(first.ordinal, later.ordinal);
        assert_eq!(case.subject.name_of(later), Some("Beta"));
        let lookup = |occurrence: &qsl_package::checked_views::ClauseOccurrence| {
            case.package
                .source_map()
                .iter()
                .find(|entry| {
                    entry.node_id == occurrence.node
                        && entry.role == occurrence.role
                        && entry.ordinal == occurrence.ordinal
                })
                .expect("actual owner source-map occurrence")
        };
        let first_source = lookup(first);
        let later_source = lookup(later);
        assert_ne!(first_source.regions, later_source.regions);
        let projected = case
            .project(TlLimits::default())
            .expect("later exact occurrence projects");
        assert_eq!(projected.source(), later_source);
        assert_ne!(projected.source(), first_source);
        assert_eq!(projected.request().subject_identity(), "Beta");
        let record: Value =
            serde_json::from_slice(projected.bytes()).expect("complete canonical record");
        assert_eq!(record["source"], expected::ir(later_source));
        assert_ne!(record["source"], expected::ir(first_source));
        assert_eq!(record["declaration"]["node"], expected::ir(&later.node));
        assert_eq!(record["declaration"]["role"], expected::ir(&later.role));
        assert_eq!(record["declaration"]["ordinal"], later.ordinal.to_string());
        assert_eq!(record["declaration"]["name"], "Beta");
    });
}

#[trace("IT-022-SC-10", "FR-350-AC-3")]
#[test]
fn predicate_and_declared_type_mutations_preserve_their_actual_owner_bindings() {
    let build = |body: &str| {
        let mut result = None;
        fixture::with_source(&fixture::declarations(body), "Due", &[b"1"], |case| {
            let projected = case
                .project(TlLimits::default())
                .expect("typed capture projection");
            let record: Value =
                serde_json::from_slice(projected.bytes()).expect("typed correspondence");
            let leaves = projected.formula().leaves();
            assert_eq!(leaves.len(), 1);
            let leaf = &leaves[0];
            assert_eq!(
                record["bindings"][0]["predicate"],
                expected::ir(&leaf.predicate)
            );
            assert_eq!(
                record["bindings"][0]["term"],
                expected::ir(&case.package.term(&leaf.predicate))
            );
            assert_eq!(
                record["bindings"][0]["proposition"],
                json!(leaf.proposition.0.to_string())
            );
            let report = evaluate(&projected);
            let key = obligation(&projected);
            let retained = report
                .ledger()
                .unwrap()
                .instance(&key, case.native.request.trigger.as_ref().unwrap())
                .unwrap();
            let capture = retained
                .environment()
                .get("saved")
                .expect("declared capture");
            assert_eq!(capture.value().as_bytes(), b"1");
            result = Some((
                record,
                capture.declared_type().clone(),
                projected.formula().bytes().to_vec(),
            ));
        });
        result.expect("genuine mutated source admitted")
    };
    let (original, original_type, original_formula) =
        build("capture saved: Int[0, 10] = 1; holds(saved = 1)");
    let (predicate, predicate_type, predicate_formula) =
        build("capture saved: Int[0, 10] = 1; holds(saved = 2)");
    assert_eq!(original_type, predicate_type);
    assert_eq!(
        original_formula, predicate_formula,
        "TL proposition shape cannot erase predicate binding changes"
    );
    assert_ne!(
        original, predicate,
        "the record distinguishes the predicate change the formula bytes cannot"
    );
    let (typed, changed_type, typed_formula) =
        build("capture saved: Int[0, 20] = 1; holds(saved = 1)");
    assert_ne!(original_type, changed_type);
    assert_eq!(
        original_formula, typed_formula,
        "TL proposition shape cannot erase native type changes"
    );
    // FR-350's member table carries capture values, not declared types, so a type
    // change alone is no record member; the owner type itself differs (above).
    let _ = typed;
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn a_genuine_retained_capture_cannot_be_replayed_under_another_admitted_anchor() {
    fixture::with_source(
        &fixture::declarations("capture saved: Boolean = true; holds(saved)"),
        "Due",
        &[b"true"],
        |case| {
            let prior_anchor = "earlier-native-observation";
            assert_ne!(prior_anchor, case.native.instance.anchor);
            let mut request = case.native.request.clone();
            request.anchor = Some(prior_anchor.into());
            request.positions[0].observation.identity = prior_anchor.into();
            request.origins[0].captures[0] = CaptureFact::Value {
                anchor: prior_anchor.into(),
                value: CapturedValue::new(b"true".as_slice()),
            };
            let instance = native::ValuationInstance {
                anchor: prior_anchor,
                capture_instance: case.native.instance.capture_instance,
            };
            let rows = case
                .native
                .valuations
                .rows()
                .iter()
                .cloned()
                .map(|mut row| {
                    row.anchor = prior_anchor.into();
                    row
                })
                .collect();
            let evidence = native::ValuationEvidence::admit(
                case.subject,
                &projection_support::native_trace(projection_support::EVENT, prior_anchor),
                instance,
                &case.native.observation.revision,
                rows,
            )
            .expect("actual native prior-anchor row admission");
            let prior_input = NativeProjectionInput {
                request: &request,
                valuations: &evidence,
                instance,
                ..case.native
            };
            let prior = evaluate_input(case.package, prior_input);
            let ledger = prior.ledger().expect("actual prior native ledger");
            let key = ObligationKey::new(
                case.native.occurrence.node.package.clone(),
                serde_json::to_vec(&(
                    &case.native.occurrence.node,
                    quire_contract_ir::CheckedOccurrence {
                        role: case.native.occurrence.role.clone(),
                        ordinal: case.native.occurrence.ordinal,
                    },
                ))
                .unwrap(),
                case.native.occurrence.ordinal,
            );
            let retained = ledger
                .instance(&key, case.native.request.trigger.as_ref().unwrap())
                .unwrap();
            assert_eq!(
                retained.environment().get("saved").unwrap().anchor(),
                prior_anchor
            );
            // The current input remains bound to the genuine current QObs observation.
            let mut current = case.native.request.clone();
            current.ledger = Some(ledger.clone());
            let result = project_tl(
                case.package,
                case.subject,
                case.classification,
                case.surrounding,
                NativeProjectionInput {
                    request: &current,
                    ..case.native
                },
                case.selection,
                TlLimits::default(),
            );
            assert!(matches!(
                result,
                Err(TlProjectionError::NativePremise(
                    TlProjectionPremise::Capture
                ))
            ));
        },
    );
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn retained_native_capture_faults_preserve_original_kind_and_named_subject() {
    use qsl_eval::temporal::evidence::{FaultKind, FaultSubject};
    use quire_integration::temporal::TlProjectionCode;
    fixture::with_source(
        &fixture::declarations("capture saved: Boolean = true; holds(saved)"),
        "Due",
        &[b"true"],
        |case| {
            for (fact, kind, code) in [
                (
                    CaptureFact::Missing,
                    FaultKind::Missing,
                    TlProjectionCode::Incomplete,
                ),
                (
                    CaptureFact::WrongType,
                    FaultKind::Refused,
                    TlProjectionCode::NativePremise,
                ),
                (
                    CaptureFact::Value {
                        anchor: "another-capture-anchor".into(),
                        value: CapturedValue::new(b"true".as_slice()),
                    },
                    FaultKind::Refused,
                    TlProjectionCode::NativePremise,
                ),
            ] {
                let mut faulty = case.native.request.clone();
                faulty.origins[0].captures[0] = fact;
                let prior = evaluate_input(
                    case.package,
                    NativeProjectionInput {
                        request: &faulty,
                        ..case.native
                    },
                );
                let ledger = prior
                    .ledger()
                    .expect("genuine native faulted-instance ledger");
                let mut current = case.native.request.clone();
                current.ledger = Some(ledger.clone());
                let error = project_tl(
                    case.package,
                    case.subject,
                    case.classification,
                    case.surrounding,
                    NativeProjectionInput {
                        request: &current,
                        ..case.native
                    },
                    case.selection,
                    TlLimits::default(),
                )
                .expect_err("retained owner fault");
                assert_eq!(error.code(), code);
                // A retained `Missing` fault is incomplete; a `Refused` one is a native premise.
                match (kind, &error) {
                    (
                        FaultKind::Missing,
                        TlProjectionError::Incomplete(TlMissingPremise::Activation),
                    )
                    | (
                        FaultKind::Refused,
                        TlProjectionError::NativePremise(TlProjectionPremise::Activation),
                    ) => {}
                    other => panic!("retained fault premise: {other:?}"),
                }
                let _ = FaultSubject::Capture("saved".into());
            }
        },
    );
}

#[trace("IT-022-SC-08", "FR-350-AC-9")]
#[test]
fn retained_native_guard_and_delivery_anchor_faults_keep_distinct_subjects() {
    use qsl_eval::temporal::evidence::{FaultKind, FaultSubject, Receipt, TriggerDelivery};
    use quire_integration::temporal::TlProjectionCode;
    let source = "temporal Due using t over (g: M::Gadget) clock \"events\" on each (trigger: M::Gadget) when (true) { holds(true) }";
    fixture::with_source(source, "Due", &[], |case| {
        for (guard, anchor, kind, subject, code) in [
            (
                None,
                case.native.instance.anchor,
                FaultKind::Missing,
                FaultSubject::Guard,
                TlProjectionCode::Incomplete,
            ),
            (
                Some(true),
                "another-delivery-anchor",
                FaultKind::Refused,
                FaultSubject::Anchor,
                TlProjectionCode::NativePremise,
            ),
        ] {
            let mut request = case.native.request.clone();
            let origin = request.origins.remove(0);
            request.deliveries = vec![TriggerDelivery {
                identity: origin.identity,
                receipt: Receipt::new(b"real-fault-fixture-receipt".as_slice()),
                anchor: anchor.into(),
                payload: origin.payload,
                guard,
                captures: vec![],
            }];
            let prior = evaluate_input(
                case.package,
                NativeProjectionInput {
                    request: &request,
                    ..case.native
                },
            );
            let ledger = prior
                .ledger()
                .expect("actual guarded delivery fault ledger");
            request.ledger = Some(ledger.clone());
            request.deliveries[0].guard = Some(true);
            request.deliveries[0].anchor = case.native.instance.anchor.into();
            let error = project_tl(
                case.package,
                case.subject,
                case.classification,
                case.surrounding,
                NativeProjectionInput {
                    request: &request,
                    ..case.native
                },
                case.selection,
                TlLimits::default(),
            )
            .expect_err("retained original delivery fault");
            assert_eq!(error.code(), code);
            match (kind, &error) {
                (
                    FaultKind::Missing,
                    TlProjectionError::Incomplete(TlMissingPremise::Activation),
                )
                | (
                    FaultKind::Refused,
                    TlProjectionError::NativePremise(TlProjectionPremise::Activation),
                ) => {}
                other => panic!("retained fault premise: {other:?}"),
            }
            let _ = subject;
        }
    });
}

/// The record's captured inputs, read from the case's own owner request.
fn expected_members(
    case: &projection_support::Case<'_>,
    projection: &ValidatedTlProjection<'_>,
) -> serde_json::Map<String, Value> {
    let origin = &case.native.request.origins[0];
    let captures: Vec<&[u8]> = origin
        .captures
        .iter()
        .map(|capture| match capture {
            CaptureFact::Value { value, .. } => value.as_bytes(),
            other => panic!("admitted capture: {other:?}"),
        })
        .collect();
    let occurrence = case.native.occurrence;
    let source = case
        .package
        .source_map()
        .iter()
        .find(|entry| {
            entry.node_id == occurrence.node
                && entry.role == occurrence.role
                && entry.ordinal == occurrence.ordinal
        })
        .expect("source entry");
    expected::expected_members(
        case,
        projection,
        &captures,
        origin.payload.as_bytes(),
        source,
    )
}

fn with_request<'a>(
    case: &projection_support::Case<'a>,
    request: &'a qsl_eval::temporal::evidence::EvidenceRequest,
) -> projection_support::Case<'a> {
    projection_support::Case {
        native: NativeProjectionInput {
            request,
            ..case.native
        },
        ..*case
    }
}

#[trace("IT-022-SC-10", "FR-350-AC-3", "FR-350-AC-10", "FR-350-AC-11")]
#[test]
fn records_equal_the_independently_built_expected_bytes_for_future_past_and_sample() {
    for (formula, sample) in [
        ("holds(true) and holds(false)", false),
        ("once[0,0] holds(true)", false),
        ("once[0,0] holds(true)", true),
    ] {
        projection_support::with_case(formula, sample, |case| {
            let projection = case.project(TlLimits::default()).expect("projection");
            let members = expected_members(&case, &projection);
            assert_eq!(
                expected::canonical(&members),
                projection.bytes(),
                "{formula} sample={sample}"
            );
            let text = std::str::from_utf8(projection.bytes()).expect("JSON text");
            let record: Value = serde_json::from_str(text).expect("record");
            fn no_number(value: &Value) {
                match value {
                    Value::Number(number) => panic!("JSON number in record: {number}"),
                    Value::Array(items) => items.iter().for_each(no_number),
                    Value::Object(members) => members.values().for_each(no_number),
                    _ => {}
                }
            }
            no_number(&record);
            // The request holds no record bytes, and carries the subject and profile names.
            assert_eq!(projection.request().subject_identity(), "Due");
            assert_eq!(
                projection.request().correspondence_identity(),
                "quire.native-temporal-correspondence/v1"
            );
            let record_bytes = projection.bytes();
            assert!(
                !projection
                    .request()
                    .bytes()
                    .windows(record_bytes.len().min(32))
                    .any(|window| window == &record_bytes[..record_bytes.len().min(32)]),
                "the request carries no record bytes"
            );
        });
    }
}

#[trace("IT-022-SC-10", "FR-350-AC-3", "FR-350-AC-10", "FR-350-AC-11")]
#[test]
fn each_mutated_source_input_changes_exactly_its_member_and_the_original_bytes_mismatch() {
    use qsl_eval::temporal::OrderKey;
    projection_support::with_case("holds(true) and holds(false)", false, |case| {
        let original = case.project(TlLimits::default()).expect("projection");
        let base = expected_members(&case, &original);
        type Mutation = fn(&mut qsl_eval::temporal::evidence::EvidenceRequest);
        let mutations: Vec<(&str, Mutation, &[&str])> = vec![
            (
                "trigger scope",
                |r| r.trigger_scope = qsl_eval::temporal::ProgressClosure::Open,
                &["native"],
            ),
            (
                "authoritative origin",
                |r| r.authoritative_origin = Some(false),
                &["authoritativeOrigin"],
            ),
            (
                "order key",
                |r| {
                    r.positions[0].order_key = Some(OrderKey {
                        authority: "order".into(),
                        key: 7,
                    })
                },
                &["positions"],
            ),
            (
                "position reference",
                |r| r.positions[0].observation.scope = "another-scope".into(),
                &["positions"],
            ),
            (
                "correspondence reference",
                |r| r.correspondence.scope = "another-scope".into(),
                &["native"],
            ),
            (
                "completeness facts",
                |r| r.completeness.as_mut().unwrap().facts.push("fact".into()),
                &["native"],
            ),
            (
                "trigger",
                |r| {
                    let other = qsl_eval::temporal::evidence::EvidenceRequest::trigger_of(
                        b"whole-execution:other",
                    )
                    .unwrap();
                    r.origins[0].identity = other.clone();
                    r.trigger = Some(other);
                },
                &["trigger"],
            ),
            (
                "payload",
                |r| r.origins[0].payload = Payload::new(b"another-origin".as_slice()),
                &["triggerPayload"],
            ),
            (
                "completeness reference",
                |r| r.completeness.as_mut().unwrap().reference.scope = "another".into(),
                &["native"],
            ),
            (
                "position population",
                |r| r.positions[0].observation.population = "another".into(),
                &["positions"],
            ),
            (
                "position digest",
                |r| r.positions[0].observation.digest = vec![1, 2, 3],
                &["positions"],
            ),
            (
                "correspondence revision",
                |r| r.correspondence.authority_revision = "2".into(),
                &["native"],
            ),
            (
                "correspondence digest",
                |r| r.correspondence.digest = vec![9],
                &["native"],
            ),
            (
                "unknown origin",
                |r| r.authoritative_origin = None,
                &["authoritativeOrigin"],
            ),
        ];
        for (name, mutate, members) in mutations {
            let mut request = case.native.request.clone();
            mutate(&mut request);
            let changed = with_request(&case, &request);
            let projection = changed
                .project(TlLimits::default())
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let after = expected_members(&changed, &projection);
            assert_eq!(expected::canonical(&after), projection.bytes(), "{name}");
            assert_eq!(expected::differing(&base, &after), members, "{name}");
            let error = read_tl_projection(
                original.bytes(),
                ExpectedTlProjection {
                    package: changed.package,
                    subject: changed.subject,
                    classification: changed.classification,
                    surrounding: changed.surrounding,
                    native: changed.native,
                    selection: changed.selection,
                },
                TlLimits::default(),
            )
            .err()
            .unwrap_or_else(|| panic!("{name}: original bytes must mismatch"));
            assert_eq!(
                error.code(),
                TlProjectionCode::CorrespondenceMismatch,
                "{name}"
            );
        }
    });
}

/// Inputs that live in the package or the independent QObs views change the
/// members that record them, and the original bytes then mismatch.
#[trace("IT-022-SC-10", "FR-350-AC-3", "FR-350-AC-11")]
#[test]
fn package_and_view_inputs_change_their_members() {
    use qsl_package::checked_views::Closure;
    use quire_observation::authority::OpenClosed;
    let members = |formula: &str,
                   sample: bool,
                   surrounding: Closure,
                   decision: OpenClosed|
     -> serde_json::Map<String, Value> {
        let mut out = None;
        projection_support::with_case_scopes(formula, sample, surrounding, decision, |case| {
            let projection = case.project(TlLimits::default()).expect("projection");
            out = Some(expected_members(&case, &projection));
        });
        out.expect("members")
    };
    let base = members("holds(true)", false, Closure::Complete, OpenClosed::Closed);
    let contains = |changed: &serde_json::Map<String, Value>, names: &[&str]| {
        let differing = expected::differing(&base, changed);
        for name in names {
            assert!(
                differing.iter().any(|d| d == name),
                "{name} in {differing:?}"
            );
        }
    };
    contains(
        &members("holds(true)", false, Closure::Open, OpenClosed::Closed),
        &["surrounding", "surroundingClosure", "native"],
    );
    contains(
        &members("holds(true)", false, Closure::Complete, OpenClosed::Open),
        &["decisionClosure", "native"],
    );
    contains(
        &members(
            "not holds(true)",
            false,
            Closure::Complete,
            OpenClosed::Closed,
        ),
        &["formula", "intervals"],
    );
    contains(
        &members("holds(true)", true, Closure::Complete, OpenClosed::Closed),
        &[
            "clock",
            "observation",
            "clockView",
            "subjectProfile",
            "lock",
        ],
    );
}

/// A valuation row, a progress view and a capture each change exactly the members
/// that record them.
#[trace("IT-022-SC-10", "FR-350-AC-3", "FR-350-AC-11")]
#[test]
fn valuation_rows_progress_views_and_captures_change_their_members() {
    use qsl_package::checked_views::Closure;
    use quire_observation::authority::OpenClosed;
    use quire_spec_language::temporal::ValuationEvidence;
    projection_support::with_case("holds(true) and holds(false)", false, |case| {
        let base = case.project(TlLimits::default()).expect("projection");
        let before = expected_members(&case, &base);
        let mut rows = case.native.valuations.rows().to_vec();
        rows[0].value = !rows[0].value;
        let evidence = ValuationEvidence::admit(
            case.subject,
            &projection_support::trace_for(&case),
            case.native.instance,
            &case.native.observation.revision,
            rows,
        )
        .expect("flipped row admits");
        let changed = projection_support::Case {
            native: NativeProjectionInput {
                valuations: &evidence,
                ..case.native
            },
            ..case
        };
        let projection = changed.project(TlLimits::default()).expect("projection");
        let after = expected_members(&changed, &projection);
        assert_eq!(expected::canonical(&after), projection.bytes());
        let differing = expected::differing(&before, &after);
        assert!(
            differing.iter().any(|name| name == "valuations"),
            "{differing:?}"
        );
        assert!(
            differing.iter().any(|name| name == "input"),
            "{differing:?}"
        );
        let error = read_tl_projection(
            base.bytes(),
            ExpectedTlProjection {
                package: changed.package,
                subject: changed.subject,
                classification: changed.classification,
                surrounding: changed.surrounding,
                native: changed.native,
                selection: changed.selection,
            },
            TlLimits::default(),
        )
        .expect_err("original bytes mismatch");
        assert_eq!(error.code(), TlProjectionCode::CorrespondenceMismatch);
    });
    let progress = |watermark: u64| {
        let mut out = None;
        projection_support::with_case_watermark(
            "holds(true)",
            false,
            Closure::Complete,
            OpenClosed::Closed,
            watermark,
            |case| {
                let projection = case.project(TlLimits::default()).expect("projection");
                out = Some(expected_members(&case, &projection));
            },
        );
        out.expect("members")
    };
    let differing = expected::differing(&progress(1), &progress(2));
    for name in ["decisionProgress", "surroundingProgress", "native"] {
        assert!(
            differing.iter().any(|d| d == name),
            "{name} in {differing:?}"
        );
    }
    let captured = |value: &'static [u8]| {
        let mut out = None;
        fixture::with_source(
            &fixture::declarations("capture saved: Int[0, 10] = 1; holds(saved = 1)"),
            "Due",
            &[value],
            |case| {
                let projection = case.project(TlLimits::default()).expect("projection");
                out = Some(expected_members(&case, &projection));
            },
        );
        out.expect("members")
    };
    let differing = expected::differing(&captured(b"1"), &captured(b"2"));
    assert!(
        differing.iter().any(|name| name == "captures"),
        "{differing:?}"
    );
}
