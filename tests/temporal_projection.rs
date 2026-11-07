// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: genuine native occurrence, evidence and TL-owner projection.

mod support;

use ix_trace_rs::trace;
use qsl_eval::temporal::ProgressClosure;
use qsl_package::checked_views::{mapping_support, temporal_subject, Closure};
use quire_integration::temporal::{
    project_tl, MappingBindingRefusal, NativeProjectionInput, TlInput, TlLimits, TlMissingPremise,
    TlProjectionError, TlProjectionPremise, TL_CORRESPONDENCE_PROFILE,
};
use quire_mltl::request::TemporalLane;
use quire_observation::authority::OpenClosed;
use quire_spec_language::temporal::{self as native, ValuationEvidence, ValuationInstance};
use tl_mltl::wire::OwnerReadErrorCode;
use tl_syntax::PropositionId;

#[path = "support/temporal_projection.rs"]
mod projection_support;
use projection_support::*;

#[trace("IT-022-SC-04", "FR-350-AC-1", "FR-350-AC-9")]
#[test]
fn opaque_observation_anchor_projects_actual_future_values_and_owner_request() {
    with_case("holds(true) and holds(false)", false, |case| {
        let projected = case
            .project(TlLimits::default())
            .expect("future projection");
        assert_eq!(projected.anchor(), 0);
        assert_eq!(projected.source_positions(), &[0]);
        assert_eq!(projected.request().anchor(), 0);
        assert_eq!(projected.request().subject_identity(), "Due");
        assert_eq!(
            projected.request().correspondence_identity(),
            TL_CORRESPONDENCE_PROFILE
        );
        assert_eq!(projected.request().lane(), TemporalLane::Future);
        let TlInput::Future(trace) = projected.input() else {
            panic!("actual future trace")
        };
        assert_eq!(trace.document().instants, vec![vec![PropositionId(0)]]);
        assert!(trace.document().closed);
        assert_eq!(projected.propositions().propositions().len(), 2);
        assert_eq!(projected.usage().valuations, 2);
        assert_eq!(
            projected.native().request.anchor,
            case.native.request.anchor
        );
    });
}

#[trace("IT-022-SC-04", "IT-022-SC-06", "FR-350-AC-1", "FR-350-AC-4")]
#[test]
fn genuine_origin_complete_past_history_retains_values_and_clock() {
    for sample in [false, true] {
        with_case("once[0,0] holds(true)", sample, |case| {
            let projected = case.project(TlLimits::default()).expect("past projection");
            assert_eq!(projected.request().lane(), TemporalLane::Past);
            let TlInput::Past(history) = projected.input() else {
                panic!("actual past history")
            };
            assert_eq!(history.document().through_position(), 0);
            assert_eq!(history.document().observations().len(), 1);
            assert_eq!(history.document().observations()[0].position, 0);
            assert_eq!(
                history.document().observations()[0].true_propositions,
                vec![PropositionId(0)]
            );
            assert_eq!(projected.anchor(), 0);
            assert_eq!(projected.usage().positions, 1);
            assert_eq!(projected.usage().valuations, 1);
            assert_eq!(
                history.document().clock(),
                Some(&if sample {
                    tl_mltl::ClockBinding::FixedSample {
                        epoch: tl_mltl::ExactNumber::new(0, 1).unwrap(),
                        period: tl_mltl::ExactNumber::new(1, 1).unwrap(),
                        unit: "nanoseconds".into(),
                    }
                } else {
                    tl_mltl::ClockBinding::EventPosition
                })
            );
        });
    }
}

#[trace("IT-022-SC-16", "FR-350-AC-2")]
#[test]
fn local_subject_cannot_use_an_independently_admitted_equal_classification() {
    with_case("holds(true)", false, |case| {
        let (_, foreign, clause) = packages("holds(true)", EVENT);
        let foreign_subject = temporal_subject(&foreign, &clause).expect("foreign checked subject");
        let classification =
            mapping_support(&foreign_subject, Closure::Complete).expect("foreign classification");
        assert_eq!(case.classification.retained, classification.retained);
        let error = project_tl(
            case.package,
            case.subject,
            &classification,
            Closure::Complete,
            case.native,
            case.selection,
            TlLimits::default(),
        )
        .expect_err("foreign admission refusal");
        assert!(matches!(
            error,
            TlProjectionError::Binding(MappingBindingRefusal::ForeignClassification)
        ));
    });
}

#[trace("IT-022-SC-03", "FR-350-AC-6", "FR-350-AC-9")]
#[test]
fn agreeing_duplicate_cells_project_and_disagreeing_cells_are_incomplete() {
    with_case("holds(true)", false, |case| {
        let mut rows = case.native.valuations.rows().to_vec();
        rows.push(rows[0].clone());
        let trace = trace_for(&case);
        let evidence = ValuationEvidence::admit(
            case.subject,
            &trace,
            case.native.instance,
            &case.native.observation.revision,
            rows,
        )
        .expect("owner retains agreeing duplicates");
        assert_eq!(evidence.rows().len(), 2);
        assert_eq!(
            evidence.value_of(case.native.occurrence, 0, &evidence.rows()[0].operand),
            Ok(true)
        );
        let projected = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                valuations: &evidence,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect("agreeing duplicates project their value");
        let TlInput::Future(trace) = projected.input() else {
            panic!("actual future trace")
        };
        assert_eq!(trace.document().instants, vec![vec![PropositionId(0)]]);
        // Both rows stay in the record: one value, two row objects.
        let record: serde_json::Value =
            serde_json::from_slice(projected.bytes()).expect("canonical record");
        assert_eq!(record["valuations"].as_array().map(Vec::len), Some(2));
        // Opposite values for one cell are incomplete, never false.
        let mut rows = case.native.valuations.rows().to_vec();
        let mut opposite = rows[0].clone();
        opposite.value = !opposite.value;
        rows.push(opposite);
        let conflicting = ValuationEvidence::admit(
            case.subject,
            &trace_for(&case),
            case.native.instance,
            &case.native.observation.revision,
            rows,
        )
        .expect("owner retains disagreeing rows");
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                valuations: &conflicting,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("disagreeing rows are incomplete");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::Valuation { position: 0, .. })
        ));
    });
}

#[trace("IT-022-SC-03", "FR-350-AC-6", "FR-350-AC-9")]
#[test]
fn absent_cell_establishes_no_false_value_or_partial_projection() {
    with_case("holds(true)", false, |case| {
        let evidence = ValuationEvidence::admit(
            case.subject,
            &native_trace(EVENT, case.native.instance.anchor),
            case.native.instance,
            &case.native.observation.revision,
            Vec::new(),
        )
        .expect("empty native relation is admitted");
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                valuations: &evidence,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("missing value refuses projection");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::Valuation { position: 0, .. })
        ));
    });
}

#[trace("IT-022-SC-07", "FR-350-AC-9")]
#[test]
fn independent_native_premises_refuse_without_owner_outputs() {
    with_case("holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.anchor = Some("opaque:unknown-anchor".into());
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("anchor mismatch");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::Anchor)
        ));
        request = case.native.request.clone();
        request.decision_closure = Some(ProgressClosure::Open);
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("independent decision closure mismatch");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::DecisionClosure)
        ));
        request = case.native.request.clone();
        request.surrounding_closure = Some(ProgressClosure::Open);
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("independent surrounding closure mismatch");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::SurroundingClosure)
        ));
        let mut observation = case.native.observation.clone();
        observation.revision = "2".into();
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                observation: &observation,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("revision mismatch");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::ObservationRevision)
        ));
    });
}

#[trace("IT-022-SC-04", "FR-350-AC-1")]
#[test]
fn past_projection_requires_authoritative_origin_and_complete_input() {
    with_case("once[0,0] holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.authoritative_origin = Some(false);
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("no origin history");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::History)
        ));
        request = case.native.request.clone();
        request.completeness = None;
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("no complete input");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::CompleteInput)
        ));
    });
}

#[trace("IT-022-SC-02", "FR-350-AC-9")]
#[test]
fn supplied_occurrence_and_native_request_must_select_the_same_authored_occurrence() {
    with_case("holds(true)", false, |case| {
        let mut foreign = case.native.occurrence.clone();
        foreign.ordinal += 1;
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                occurrence: &foreign,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("foreign occurrence refuses");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::Occurrence)
        ));
        let mut request = case.native.request.clone();
        request
            .occurrence
            .as_mut()
            .expect("explicit occurrence")
            .ordinal += 1;
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("request occurrence differs");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::Occurrence)
        ));
    });
}

#[trace("IT-022-SC-03", "FR-350-AC-6", "FR-350-AC-9")]
#[test]
fn a_relation_admitted_for_another_capture_instance_or_revision_is_not_reused() {
    with_case("holds(true)", false, |case| {
        let trace = trace_for(&case);
        for revision_mismatch in [false, true] {
            let instance = ValuationInstance {
                anchor: case.native.instance.anchor,
                capture_instance: if revision_mismatch {
                    CAPTURE_INSTANCE
                } else {
                    "capture-environment:foreign"
                },
            };
            let revision = if revision_mismatch {
                "foreign-revision"
            } else {
                &case.native.observation.revision
            };
            let mut rows = case.native.valuations.rows().to_vec();
            for row in &mut rows {
                row.capture_instance = instance.capture_instance.into();
                row.revision = revision.into();
            }
            let evidence = ValuationEvidence::admit(case.subject, &trace, instance, revision, rows)
                .expect("relation admits against its actual supplied native facts");
            let error = project_tl(
                case.package,
                case.subject,
                case.classification,
                Closure::Complete,
                NativeProjectionInput {
                    valuations: &evidence,
                    ..case.native
                },
                case.selection,
                TlLimits::default(),
            )
            .expect_err("foreign relation fact refuses");
            let TlProjectionError::Valuation { row, cause } = error else {
                panic!("typed valuation refusal: {error:?}")
            };
            assert_eq!(row, 0);
            assert_eq!(
                cause,
                if revision_mismatch {
                    native::ValuationCause::RevisionMismatch
                } else {
                    native::ValuationCause::CaptureInstanceMismatch
                }
            );
        }
    });
}

#[trace("IT-022-SC-02", "FR-350-AC-9")]
#[test]
fn missing_semantic_instance_and_position_are_incomplete_not_successful_values() {
    with_case("holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.origins.clear();
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("no origin instance");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::Instance)
        ));
        request = case.native.request.clone();
        request.positions.clear();
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            Closure::Complete,
            NativeProjectionInput {
                request: &request,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("no selected position");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::Anchor)
        ));
    });
}

#[trace("IT-022-SC-11", "FR-350-AC-6")]
#[test]
fn projection_reader_requires_complete_canonical_expected_value() {
    with_case("holds(true)", false, |case| {
        let projection = case
            .project(TlLimits::default())
            .expect("actual projection");
        let read = |bytes: &[u8]| {
            quire_integration::temporal::read_tl_projection(
                bytes,
                quire_integration::temporal::ExpectedTlProjection {
                    package: case.package,
                    subject: case.subject,
                    classification: case.classification,
                    surrounding: case.surrounding,
                    native: case.native,
                    selection: case.selection,
                },
                TlLimits::default(),
            )
        };
        let admitted = read(projection.bytes()).expect("strict expected projection");
        assert_eq!(admitted.bytes(), projection.bytes());
        let mut trailing = projection.bytes().to_vec();
        trailing.push(b' ');
        assert!(matches!(
            read(&trailing),
            Err(TlProjectionError::Correspondence)
        ));
        let mut value: serde_json::Value = serde_json::from_slice(projection.bytes()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_owned(), true.into());
        let unknown = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            read(&unknown),
            Err(TlProjectionError::Correspondence)
        ));
        let duplicate = [b"{\"profile\":null,".as_slice(), &projection.bytes()[1..]].concat();
        assert!(matches!(
            read(&duplicate),
            Err(TlProjectionError::Correspondence)
        ));
        let mut missing: serde_json::Value = serde_json::from_slice(projection.bytes()).unwrap();
        missing.as_object_mut().unwrap().remove("captures");
        assert!(matches!(
            read(&serde_json::to_vec(&missing).unwrap()),
            Err(TlProjectionError::Correspondence)
        ));
    });
}

#[trace("IT-022-SC-05", "FR-350-AC-2")]
#[test]
fn actual_unsupported_classes_retain_every_owner_dimension_without_projection() {
    with_case("holds(true)", false, |case| {
        for (formula, profile) in [
            ("eventually holds(true)", "quire.temporal.infinite-trace/v1"),
            ("eventually[0,2] (once[0,1] holds(true))", EVENT),
            (
                "eventually[0,2] (once[0,1] holds(true))",
                "quire.temporal.timestamped-event.finite-window/v1",
            ),
        ] {
            let (package, _, clause) = packages(formula, profile);
            let subject = temporal_subject(&package, &clause).unwrap();
            let classification = mapping_support(&subject, Closure::Complete).unwrap();
            let result = project_tl(
                &package,
                &subject,
                &classification,
                Closure::Complete,
                case.native,
                case.selection,
                TlLimits::default(),
            );
            let Err(TlProjectionError::Unsupported(retained)) = result else {
                panic!("original owner unsupported classification must refuse atomically");
            };
            assert_eq!(*retained, classification);
        }
    });
}

#[trace("IT-022-SC-04", "FR-350-AC-1")]
#[test]
fn each_supported_past_operator_is_owner_admitted_under_both_profiles() {
    for sample in [false, true] {
        for formula in [
            "once[0,1] holds(true)",
            "historically[0,1] holds(true)",
            "holds(true) since[0,1] holds(false)",
            "holds(true) triggered[0,1] holds(false)",
            "once[1,1] holds(true)",
        ] {
            with_case(formula, sample, |case| {
                let projection = case
                    .project(TlLimits::default())
                    .expect("actual supported past owner readers");
                assert_eq!(projection.request().lane(), TemporalLane::Past);
                assert!(matches!(projection.input(), TlInput::Past(_)));
                assert_eq!(
                    projection.formula().leaves().len(),
                    case.subject.predicate_leaves().len()
                );
                assert_eq!(
                    projection.usage().valuations,
                    u64::try_from(case.subject.predicate_leaves().len()).unwrap()
                );
            });
        }
    }
}

#[trace("IT-022-SC-04", "FR-350-AC-1")]
#[test]
fn each_bounded_future_operator_preserves_independent_surrounding_target() {
    for sample in [false, true] {
        for surrounding in [Closure::Complete, Closure::Open] {
            for formula in [
                "eventually[0,1] holds(true)",
                "always[0,1] holds(true)",
                "holds(true) until[0,1] holds(false)",
                "holds(true) release[0,1] holds(false)",
            ] {
                with_case_scopes(formula, sample, surrounding, OpenClosed::Closed, |case| {
                    let projection = case
                        .project(TlLimits::default())
                        .expect("actual supported future owner readers");
                    assert_eq!(projection.request().lane(), TemporalLane::Future);
                    let TlInput::Future(trace) = projection.input() else {
                        panic!("future input");
                    };
                    assert_eq!(trace.document().closed, surrounding == Closure::Complete);
                    assert_eq!(
                        projection
                            .selection()
                            .observations
                            .decision_scope_closure
                            .payload()
                            .state(),
                        OpenClosed::Closed
                    );
                    assert_eq!(
                        projection.formula().formula().semantic_profile(),
                        if surrounding == Closure::Complete {
                            tl_syntax::SemanticProfile::ClosedTraceV1
                        } else {
                            tl_syntax::SemanticProfile::OnlinePrefixV1
                        }
                    );
                });
            }
        }
    }
}

type Mutation = fn(&mut qsl_eval::temporal::evidence::EvidenceRequest);

fn premise(p: TlProjectionPremise) -> impl Fn(&TlProjectionError) -> bool {
    move |error| matches!(error, TlProjectionError::NativePremise(actual) if *actual == p)
}

fn missing(m: TlMissingPremise) -> impl Fn(&TlProjectionError) -> bool {
    move |error| matches!(error, TlProjectionError::Incomplete(actual) if *actual == m)
}

/// Each request-level failure returns exactly the code and premise its binding step assigns.
#[trace(
    "IT-022-SC-02",
    "IT-022-SC-07",
    "IT-022-SC-16",
    "FR-350-AC-6",
    "FR-350-AC-9"
)]
#[test]
fn each_request_failure_returns_its_assigned_code_and_premise() {
    use qsl_eval::temporal::{evidence::TriggerEvidence, ClockCover};
    type Check = Box<dyn Fn(&TlProjectionError) -> bool>;
    with_case("holds(true)", false, |case| {
        let cases: Vec<(&str, Mutation, Check)> = vec![
            (
                "occurrence of another ordinal",
                |r| r.occurrence.as_mut().unwrap().ordinal += 1,
                Box::new(premise(TlProjectionPremise::Occurrence)),
            ),
            (
                "wrong binding",
                |r| r.binding.clock_name = "another-clock".into(),
                Box::new(|e| matches!(e, TlProjectionError::NativeClock(_))),
            ),
            (
                "counter cover",
                |r| r.cover = ClockCover::Counter { uncertainty: 1 },
                Box::new(premise(TlProjectionPremise::ClockCover)),
            ),
            (
                "correspondence identity",
                |r| r.correspondence.identity = "another".into(),
                Box::new(premise(TlProjectionPremise::CorrespondenceReference)),
            ),
            (
                "correspondence authority",
                |r| r.correspondence.authority_identity = "another".into(),
                Box::new(premise(TlProjectionPremise::CorrespondenceReference)),
            ),
            (
                "absent anchor",
                |r| r.anchor = None,
                Box::new(missing(TlMissingPremise::Anchor)),
            ),
            (
                "anchor unequal to the instance",
                |r| r.anchor = Some("another-anchor".into()),
                Box::new(premise(TlProjectionPremise::Anchor)),
            ),
            (
                "unadmitted trigger",
                |r| r.trigger_evidence = TriggerEvidence::Missing,
                Box::new(missing(TlMissingPremise::Instance)),
            ),
            (
                "absent trigger",
                |r| r.trigger = None,
                Box::new(missing(TlMissingPremise::Instance)),
            ),
            (
                "no origin for the trigger",
                |r| r.origins.clear(),
                Box::new(missing(TlMissingPremise::Instance)),
            ),
            (
                "position of an unknown observation",
                |r| r.positions[0].observation.identity = "unknown-observation".into(),
                Box::new(missing(TlMissingPremise::Observation)),
            ),
            (
                "absent decision closure",
                |r| r.decision_closure = None,
                Box::new(missing(TlMissingPremise::CompleteInput)),
            ),
            (
                "differing decision closure",
                |r| r.decision_closure = Some(ProgressClosure::Open),
                Box::new(premise(TlProjectionPremise::DecisionClosure)),
            ),
            (
                "differing surrounding closure",
                |r| r.surrounding_closure = Some(ProgressClosure::Open),
                Box::new(premise(TlProjectionPremise::SurroundingClosure)),
            ),
            (
                "absent surrounding closure",
                |r| r.surrounding_closure = None,
                Box::new(premise(TlProjectionPremise::SurroundingClosure)),
            ),
            (
                "absent progress",
                |r| r.decision_progress = None,
                Box::new(premise(TlProjectionPremise::Progress)),
            ),
            (
                "differing surrounding progress",
                |r| r.surrounding_progress = Some(99),
                Box::new(premise(TlProjectionPremise::Progress)),
            ),
            (
                "absent completeness",
                |r| r.completeness = None,
                Box::new(missing(TlMissingPremise::CompleteInput)),
            ),
            (
                "incomplete completeness",
                |r| {
                    r.completeness.as_mut().unwrap().state =
                        qsl_eval::temporal::ProgressCompleteness::Incomplete
                },
                Box::new(missing(TlMissingPremise::CompleteInput)),
            ),
        ];
        for (name, mutate, check) in cases {
            let mut request = case.native.request.clone();
            mutate(&mut request);
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
            .err()
            .unwrap_or_else(|| panic!("{name}: must refuse"));
            assert!(check(&error), "{name}: {error:?}");
        }
        // The instance fields are separate inputs.
        let empty_instance = ValuationInstance {
            capture_instance: "",
            ..case.native.instance
        };
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                instance: empty_instance,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("empty capture instance");
        assert!(premise(TlProjectionPremise::Anchor)(&error), "{error:?}");
        // A control outside the subject's requirements.
        let mut request = case.native.request.clone();
        request.control = Some(case.native.occurrence.node.clone());
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
        .expect_err("foreign control");
        assert!(premise(TlProjectionPremise::Control)(&error), "{error:?}");
        // An observation reference of another bundle.
        let mut observation = case.native.observation.clone();
        observation.reference = "another-bundle".into();
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                observation: &observation,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("foreign observation");
        assert!(
            premise(TlProjectionPremise::ObservationRevision)(&error),
            "{error:?}"
        );
        // An anchor equal to no position: both the request and the instance name it.
        let mut request = case.native.request.clone();
        request.anchor = Some("no-such-position".into());
        let instance = ValuationInstance {
            anchor: "no-such-position",
            ..case.native.instance
        };
        let error = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                request: &request,
                instance,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect_err("anchor without position");
        assert!(missing(TlMissingPremise::Anchor)(&error), "{error:?}");
    });
}

/// Rows failing the bridge's owner-order checks name their cause and admitted index.
#[trace("IT-022-SC-03", "FR-350-AC-6", "FR-350-AC-9")]
#[test]
fn request_valuation_maps_do_not_change_the_projection() {
    use qsl_eval::temporal::Valuation;
    with_case("holds(true) and holds(false)", false, |case| {
        let before = case.project(TlLimits::default()).expect("projection");
        let mut request = case.native.request.clone();
        for (index, value) in [Valuation::Missing, Valuation::True]
            .into_iter()
            .enumerate()
        {
            request.positions[0].valuations.insert(index as u64, value);
        }
        let after = project_tl(
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
        .expect("request maps are not read");
        assert_eq!(after.input().bytes(), before.input().bytes());
        assert_eq!(after.request().bytes(), before.request().bytes());
    });
}

/// A fixed-sample clock must match the owner anchor exactly and is never converted.
#[trace("IT-022-SC-06", "FR-350-AC-4")]
#[test]
fn fixed_sample_clock_matches_the_owner_anchor_and_units_are_never_converted() {
    // Equal epoch and period project.
    with_case("once[0,0] holds(true)", true, |case| {
        case.project(TlLimits::default())
            .expect("matching sample clock");
    });
    // Only the period differs: the formula bytes are equal, the clock is not.
    projection_support::with_sample_clock("2", "nanoseconds", || {
        with_case("holds(true)", true, |case| {
            let error = case
                .project(TlLimits::default())
                .expect_err("period mismatch");
            assert!(premise(TlProjectionPremise::Clock)(&error), "{error:?}");
        });
    });
    // Samples reinterpreted as event counts.
    projection_support::with_observation_form(false, || {
        with_case("holds(true)", true, |case| {
            let error = case.project(TlLimits::default()).expect_err("event counts");
            assert!(premise(TlProjectionPremise::Clock)(&error), "{error:?}");
        });
    });
    // A millisecond unit: a future target cannot carry it; a past target hands it
    // unchanged to the TL request owner.
    projection_support::with_sample_clock("1", "milliseconds", || {
        with_case("holds(true)", true, |case| {
            let error = case.project(TlLimits::default()).expect_err("future unit");
            assert!(
                premise(TlProjectionPremise::ClockRepresentation)(&error),
                "{error:?}"
            );
        });
        with_case("once[0,0] holds(true)", true, |case| {
            let error = case.project(TlLimits::default()).expect_err("past unit");
            // The TL request owner's own refusal (quire-mltl FR-001-AC-9), exactly.
            let TlProjectionError::Owner(owner) = &error else {
                panic!("tl-projection-owner required: {error:?}")
            };
            assert_eq!(owner.code(), OwnerReadErrorCode::ExpectedMismatch);
            assert_eq!(owner.field(), "clockBinding");
        });
    });
}

/// Project with a mutated copy of the case's request and keep only the verdict.
fn run(
    case: &Case<'_>,
    mutate: impl FnOnce(&mut qsl_eval::temporal::evidence::EvidenceRequest),
) -> Result<(), TlProjectionError> {
    let mut request = case.native.request.clone();
    mutate(&mut request);
    project_tl(
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
    .map(|_| ())
}

/// The position-set defects a single genuine bundle record can exhibit. A QObs
/// bundle carries one capture environment at one anchor and every role exactly
/// once, so the owner admits no bundle with several distinct stamps.
#[trace("IT-022-SC-02", "IT-022-SC-16", "FR-350-AC-4", "FR-350-AC-9")]
#[test]
fn position_set_defects_return_their_assigned_premises() {
    with_case("holds(true)", false, |case| {
        let several = run(&case, |r| {
            r.positions.push(r.positions[0].clone());
        })
        .expect_err("anchor matches several positions");
        assert!(
            premise(TlProjectionPremise::Anchor)(&several),
            "{several:?}"
        );
    });
    projection_support::with_positions(1, 1, 0, || {
        with_case("holds(true)", false, |case| {
            let above = run(&case, |r| {
                r.completeness.as_mut().unwrap().state =
                    qsl_eval::temporal::ProgressCompleteness::Complete { through: 0 };
            })
            .expect_err("stamp above through");
            assert!(premise(TlProjectionPremise::Position)(&above), "{above:?}");
        });
    });
    // A clock range longer than the supplied samples: a sample is missing.
    projection_support::with_range_extra(1, || {
        with_case("holds(true)", false, |case| {
            let error = case
                .project(TlLimits::default())
                .expect_err("missing sample");
            assert!(missing(TlMissingPremise::Position)(&error), "{error:?}");
        });
    });
}

/// A past target needs an origin-complete history: range start 0 and an
/// authoritative origin that is exactly `Some(true)`.
#[trace("IT-022-SC-04", "FR-350-AC-1")]
#[test]
fn past_history_needs_range_start_zero_and_an_authoritative_origin() {
    projection_support::with_positions(1, 1, 0, || {
        with_case("once[0,0] holds(true)", false, |case| {
            let error = case.project(TlLimits::default()).expect_err("start 1");
            assert!(missing(TlMissingPremise::History)(&error), "{error:?}");
        });
    });
    with_case("once[0,0] holds(true)", false, |case| {
        for origin in [Some(false), None] {
            let error = run(&case, |r| r.authoritative_origin = origin)
                .expect_err("origin not authoritative");
            assert!(
                missing(TlMissingPremise::History)(&error),
                "{origin:?}: {error:?}"
            );
        }
    });
}

/// A rational epoch or period has no integer form under any unit.
#[trace("IT-022-SC-16", "FR-350-AC-4")]
#[test]
fn a_rational_clock_is_refused_as_a_clock_representation_under_every_unit() {
    for (unit, formula) in [
        ("nanoseconds", "holds(true)"),
        ("milliseconds", "holds(true)"),
        ("milliseconds", "once[0,0] holds(true)"),
    ] {
        projection_support::with_sample_clock("1/2", unit, || {
            with_case(formula, true, |case| {
                let error = case.project(TlLimits::default()).expect_err("rational");
                assert!(
                    premise(TlProjectionPremise::ClockRepresentation)(&error),
                    "{unit} {formula}: {error:?}"
                );
            });
        });
    }
}

/// A QObs completeness view with a contradicted fact is a native premise.
#[trace("IT-022-SC-07", "IT-022-SC-16", "FR-350-AC-9")]
#[test]
fn contradicted_completeness_is_a_native_premise_and_incomplete_is_incomplete_input() {
    use quire_observation::authority::completeness::FactStatus;
    projection_support::with_completeness(FactStatus::Contradicted, || {
        with_case("holds(true)", false, |case| {
            let error = case.project(TlLimits::default()).expect_err("contradicted");
            assert!(
                premise(TlProjectionPremise::Completeness)(&error),
                "{error:?}"
            );
        });
    });
    projection_support::with_completeness(FactStatus::Incomplete, || {
        with_case("holds(true)", false, |case| {
            let error = case.project(TlLimits::default()).expect_err("incomplete");
            assert!(
                missing(TlMissingPremise::CompleteInput)(&error),
                "{error:?}"
            );
        });
    });
}

/// An integer interval above `u32::MAX` cannot be preserved.
#[trace("IT-022-SC-05", "FR-350-AC-3")]
#[test]
fn an_interval_bound_above_u32_max_is_a_native_premise() {
    with_case("eventually[0,4294967296] holds(true)", false, |case| {
        let error = case.project(TlLimits::default()).expect_err("bound");
        assert!(premise(TlProjectionPremise::Interval)(&error), "{error:?}");
    });
}
