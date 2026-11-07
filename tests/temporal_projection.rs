// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: genuine native occurrence, evidence and TL-owner projection.

mod support;

use ix_trace_rs::trace;
use qsl_eval::temporal::{
    evidence::{
        CompletenessRecord, EvidencePosition, EvidenceRef, EvidenceRequest, ExecutionDisposition,
        ExecutionOrigin, Payload, Relation, SemanticTriggerIdentity, TemporalEvidenceLimits,
        TraceBinding, TriggerEvidence,
    },
    finite_window::ClockCover,
    ledger::{ProgressClosure, ProgressCompleteness},
};
use qsl_package::checked_views::{
    mapping_support, temporal_subject, Classification, Closure, TemporalSubjectView,
};
use qsl_semantics::{
    check::{CheckRefusal, CheckedGraph, PackageDeclarations},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedNodeKind, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, ClockConfiguration,
    ClockExactNumber, TemporalForm,
};
use quire_integration::temporal::{
    project_tl, read_observation_revision, MappingBindingRefusal, NativeProjectionInput,
    ObservationAdmissionLimits, TlInput, TlLimits, TlMissingPremise, TlProjectionError,
    TlProjectionLimit, TlProjectionPremise, TlProjectionSelection, ValidatedTlProjection,
    TL_CORRESPONDENCE_PROFILE,
};
use quire_mltl::request::{ObservationInputs, TemporalLane};
use quire_observation::authority::{self as a, bundle, Limits, OpenClosed};
use quire_semantic_value::checking::CheckingLimits;
use quire_spec_language::temporal::{
    self as native, ObservationRef, ValuationEvidence, ValuationInstance, ValuationRow,
};
use std::collections::BTreeMap;
use tl_mltl::wire::OwnerLimits;
use tl_syntax::PropositionId;

#[path = "support/temporal_projection.rs"]
mod projection_support;
use projection_support::*;

#[trace("TC-461", "FR-350-AC-1", "FR-350-AC-3")]
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

#[trace("TC-461", "FR-350-AC-1", "FR-350-AC-4")]
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

#[trace("TC-461", "FR-350-AC-3", "FR-350-AC-6")]
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
        .err()
        .expect("foreign admission refusal");
        assert!(matches!(
            error,
            TlProjectionError::Binding(MappingBindingRefusal::ForeignClassification)
        ));
    });
}

#[trace("TC-461", "FR-350-AC-6")]
#[test]
fn agreeing_duplicate_cells_are_not_silently_collapsed() {
    with_case("holds(true)", false, |case| {
        let mut rows = case.native.valuations.rows().to_vec();
        rows.push(rows[0].clone());
        let trace = native_trace(EVENT, case.native.instance.anchor);
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
        .err()
        .expect("bridge refuses duplicate cells");
        assert!(matches!(
            error,
            TlProjectionError::DuplicateValuation {
                first: 0,
                duplicate: 1
            }
        ));
    });
}

#[trace("TC-461", "FR-350-AC-6")]
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
        .err()
        .expect("missing value refuses projection");
        assert!(matches!(
            error,
            TlProjectionError::MissingValuation { position: 0, .. }
        ));
    });
}

#[trace("TC-461", "FR-350-AC-3", "FR-350-AC-6")]
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
        .err()
        .expect("anchor mismatch");
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
        .err()
        .expect("independent decision closure mismatch");
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
        .err()
        .expect("independent surrounding closure mismatch");
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
        .err()
        .expect("revision mismatch");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::ObservationRevision)
        ));
    });
}

#[trace("TC-461", "FR-350-AC-1")]
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
        .err()
        .expect("no origin history");
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
        .err()
        .expect("no complete input");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::CompleteInput)
        ));
    });
}

#[trace("TC-461", "FR-350-AC-8")]
#[test]
fn projection_counters_accept_exact_and_refuse_one_short() {
    with_case("holds(true)", false, |case| {
        let first = case
            .project(TlLimits::default())
            .expect("measurement projection");
        let usage = first.usage();
        let exact = TlLimits {
            max_bytes: usage.bytes,
            max_nodes: usage.nodes,
            max_positions: usage.positions,
            max_valuations: usage.valuations,
            max_work: usage.work,
            ..TlLimits::default()
        };
        let second = case
            .project(exact)
            .expect("all counters exactly sufficient");
        assert_eq!(second.input().bytes(), first.input().bytes());
        assert_eq!(second.usage(), usage);
        for counter in [
            TlProjectionLimit::Bytes,
            TlProjectionLimit::Nodes,
            TlProjectionLimit::Positions,
            TlProjectionLimit::Valuations,
            TlProjectionLimit::Work,
        ] {
            let mut short = exact;
            let setting = match counter {
                TlProjectionLimit::Bytes => &mut short.max_bytes,
                TlProjectionLimit::Nodes => &mut short.max_nodes,
                TlProjectionLimit::Positions => &mut short.max_positions,
                TlProjectionLimit::Valuations => &mut short.max_valuations,
                TlProjectionLimit::Work => &mut short.max_work,
                TlProjectionLimit::Support | TlProjectionLimit::Causes => unreachable!(),
            };
            *setting = setting.checked_sub(1).expect("positive exercised counter");
            let expected_setting = *setting;
            let error = case
                .project(short)
                .err()
                .expect("one-short ceiling refuses atomically");
            match error {
                TlProjectionError::LimitReached {
                    limit,
                    value,
                    setting,
                }
                | TlProjectionError::OwnerLimitReached {
                    limit,
                    value,
                    setting,
                    ..
                } => {
                    assert_eq!(limit, counter);
                    assert_eq!(setting, expected_setting);
                    assert_eq!(value, expected_setting + 1);
                }
                other => panic!("typed caller-limit refusal required: {other:?}"),
            }
        }
    });
}

#[trace("TC-461", "FR-350-AC-3", "FR-350-AC-6")]
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
        .err()
        .expect("foreign occurrence refuses");
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
        .err()
        .expect("request occurrence differs");
        assert!(matches!(
            error,
            TlProjectionError::NativePremise(TlProjectionPremise::Occurrence)
        ));
    });
}

#[trace("TC-461", "FR-350-AC-3", "FR-350-AC-6")]
#[test]
fn a_relation_admitted_for_another_capture_instance_or_revision_is_not_reused() {
    with_case("holds(true)", false, |case| {
        let trace = native_trace(EVENT, case.native.instance.anchor);
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
            .err()
            .expect("foreign relation fact refuses");
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

#[trace("TC-461", "FR-350-AC-6")]
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
        .err()
        .expect("no origin instance");
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
        .err()
        .expect("no selected position");
        assert!(matches!(
            error,
            TlProjectionError::Incomplete(TlMissingPremise::Position)
        ));
    });
}

#[trace("TC-461", "FR-350-AC-6")]
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
