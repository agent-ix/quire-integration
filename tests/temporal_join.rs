// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: join genuine native and mapped TL results without evaluation in the bridge.

#[path = "support/temporal_join_owners.rs"]
mod owners;
#[path = "support/temporal_projection.rs"]
mod projection_support;
mod support;

use ix_trace_rs::trace;
use owners::*;
use projection_support::{with_case, with_case_scopes};
use qsl_eval::temporal::evidence::{ObligationKey, Relation, Settlement, Truth};
use qsl_package::{checked_views::Closure, NativeJoinBudget};
use quire_exact::Meter;
use quire_integration::temporal::{
    join_tl, project_tl, read_tl_join, ExpectedTlJoin, NativeProjectionInput, TlJoinCause,
    TlJoinDecision, TlJoinError, TlLimits, TlProjectionLimit, ValidatedTlProjection,
};
use quire_mltl::{
    contract_ir,
    report::{self, AssessmentExecution, ResultRelationInput},
};
use quire_observation::authority::OpenClosed;
use quire_spec_language::temporal::{self as native, CheckedSubject, Report, ReportOutcome};
use serde_json::{json, Value};
use tl_mltl::wire::OwnerLimits;

#[trace("IT-022-SC-12", "IT-022-SC-15", "FR-350-AC-5", "FR-350-AC-6")]
#[test]
fn genuine_equal_final_pairs_retain_actual_sources_and_their_boolean() {
    for (formula, expected_value) in [("holds(true)", true), ("not holds(true)", false)] {
        with_case(formula, false, |case| {
            let projection = case.project(TlLimits::default()).expect("projection");
            let native = evaluate_native(&projection);
            let mapped = evaluate_mapped(&projection);
            assert_eq!(native.outcome(), ReportOutcome::Evaluated);
            assert_eq!(
                native.truth(),
                Some(if expected_value {
                    Truth::True
                } else {
                    Truth::False
                })
            );
            let original_native = native.to_owner_bytes();
            let original_mapped = mapped.bytes().to_vec();
            let original_source = mapped.source().bytes().to_vec();
            let joined = join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("equal final join");
            assert_eq!(joined.decision(), TlJoinDecision::EqualFinal);
            assert_eq!(joined.value(), Some(expected_value));
            assert!(joined.causes().is_empty());
            assert!(joined.owner_refusals().native().is_none());
            assert!(joined.owner_refusals().mapped().is_none());
            assert_eq!(
                joined.native().expect("retained native").to_owner_bytes(),
                original_native
            );
            assert_eq!(
                joined.mapped().expect("retained TL").bytes(),
                original_mapped
            );
            assert_eq!(
                joined.mapped().expect("retained TL").source().bytes(),
                original_source
            );
            assert!(std::ptr::eq(joined.projection(), &projection));
            let wire: Value = serde_json::from_slice(joined.bytes()).expect("canonical join JSON");
            assert_eq!(wire["value"], expected_value);
            assert_eq!(wire["decision"], "agreement");
            assert_eq!(wire["comparison"], "equal-final");
            assert_eq!(wire["causes"], json!([]));
            assert_eq!(wire["correspondence"], json!(hex(projection.bytes())));
            assert_eq!(wire["native"], json!(hex(&original_native)));
            assert_eq!(wire["mapped"], json!(hex(&original_mapped)));
            assert_eq!(native.to_owner_bytes(), original_native);
            assert_eq!(mapped.bytes(), original_mapped);
            assert_eq!(mapped.source().bytes(), original_source);
        });
    }
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn equal_pending_requires_actual_open_decision_and_surrounding_scopes() {
    with_case_scopes(
        "eventually[1,2] holds(true)",
        false,
        Closure::Open,
        OpenClosed::Open,
        |case| {
            let projection = case
                .project(TlLimits::default())
                .expect("actual online prefix");
            let native = evaluate_native(&projection);
            let mapped = evaluate_mapped(&projection);
            assert_eq!(native.truth(), Some(Truth::Pending));
            assert_eq!(native.settlement(), Some(Settlement::Unsettled));
            assert_eq!(mapped.source().truth(), report::TemporalTruth::Pending);
            assert_eq!(
                mapped.source().settlement(),
                report::SettlementBasis::Unsettled
            );
            let joined = join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("equal pending join");
            assert_eq!(joined.decision(), TlJoinDecision::EqualPending);
            assert_eq!(joined.value(), None);
            assert_eq!(joined.causes(), &[TlJoinCause::ResultPending]);
            assert_eq!(
                joined
                    .mapped()
                    .unwrap()
                    .source()
                    .decision_scope_closure()
                    .state(),
                OpenClosed::Open
            );
            assert_eq!(
                joined
                    .mapped()
                    .unwrap()
                    .source()
                    .surrounding_execution_closure()
                    .state(),
                OpenClosed::Open
            );
        },
    );
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn surrounding_closed_does_not_replace_open_decision_scope_or_hide_conflict() {
    with_case_scopes(
        "eventually[1,2] holds(true)",
        false,
        Closure::Complete,
        OpenClosed::Open,
        |case| {
            let projection = case
                .project(TlLimits::default())
                .expect("separate closure axes");
            let native = evaluate_native(&projection);
            let mapped = evaluate_mapped(&projection);
            assert_eq!(native.truth(), Some(Truth::Pending));
            assert_eq!(mapped.source().truth(), report::TemporalTruth::Violated);
            let joined = join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("genuine unequal result dimensions");
            assert_eq!(joined.decision(), TlJoinDecision::Conflict);
            assert_eq!(joined.value(), None);
            assert_eq!(joined.causes(), &[TlJoinCause::ResultMismatch]);
            let retained = joined.mapped().unwrap().source();
            assert_eq!(retained.decision_scope_closure().state(), OpenClosed::Open);
            assert_eq!(
                retained.surrounding_execution_closure().state(),
                OpenClosed::Closed
            );
        },
    );
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn absence_is_unavailable_and_never_an_equal_false_result() {
    with_case("not holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        for (left, right) in [(None, None), (Some(&native), None), (None, Some(&mapped))] {
            let joined = join_tl(
                &projection,
                left,
                right,
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("explicit unavailable join");
            assert_eq!(joined.decision(), TlJoinDecision::Unavailable);
            assert_eq!(joined.causes(), &[TlJoinCause::ResultUnavailable]);
            assert_eq!(joined.value(), None);
            assert_eq!(joined.native().is_some(), left.is_some());
            assert_eq!(joined.mapped().is_some(), right.is_some());
        }
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn genuine_tl_resource_incomplete_stays_non_value_and_collects_later_conflict_once() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let limits = OwnerLimits {
            max_evaluation_steps: 0,
            ..projection.selection().owner_limits
        };
        let source = evaluate_tl(&projection, ResultRelationInput::Original, limits);
        assert_eq!(source.execution(), AssessmentExecution::ResourceIncomplete);
        let mapped = map_source(&source, projection.selection().owner_limits);
        let joined = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("incomplete join");
        assert_eq!(joined.decision(), TlJoinDecision::Incomplete);
        assert_eq!(joined.value(), None);
        assert_eq!(joined.causes(), &[TlJoinCause::ResultIncomplete]);
        assert_eq!(
            joined.mapped().unwrap().source().execution(),
            AssessmentExecution::ResourceIncomplete
        );
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn unavailable_precedes_native_incomplete_and_preserves_both_causes_in_order() {
    with_case("holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.limits.valuations = 0;
        let projection = project_tl(
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
        .expect("projection does not evaluate the native request");
        let native = evaluate_native(&projection);
        assert_eq!(native.outcome(), ReportOutcome::Limit);
        let joined = join_tl(
            &projection,
            Some(&native),
            None,
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("unavailable and incomplete");
        assert_eq!(joined.decision(), TlJoinDecision::Unavailable);
        assert_eq!(
            joined.causes(),
            &[
                TlJoinCause::ResultUnavailable,
                TlJoinCause::ResultIncomplete
            ]
        );
        assert_eq!(joined.value(), None);
        assert_eq!(joined.native().unwrap().outcome(), ReportOutcome::Limit);
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5", "FR-350-AC-10")]
#[test]
fn canonical_join_roundtrip_requires_each_exact_expected_source_and_prior_value() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let joined = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("join");
        let expected = ExpectedTlJoin {
            projection: &projection,
            native: Some(&native),
            mapped: Some(&mapped),
            previous: None,
        };
        let read = read_tl_join(
            joined.bytes(),
            expected,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("strict expected-value read");
        assert_eq!(read.bytes(), joined.bytes());
        assert_eq!(read.decision(), joined.decision());
        assert_eq!(read.causes(), joined.causes());
        assert_eq!(read.value(), joined.value());
        assert_eq!(read.usage(), joined.usage());
        for alternate in [
            ExpectedTlJoin {
                native: None,
                ..expected
            },
            ExpectedTlJoin {
                mapped: None,
                ..expected
            },
            ExpectedTlJoin {
                previous: Some(&joined),
                ..expected
            },
        ] {
            assert!(matches!(
                read_tl_join(
                    joined.bytes(),
                    alternate,
                    TlLimits::default(),
                    &mut native_budget()
                ),
                Err(TlJoinError::ExpectedMismatch)
            ));
        }
        with_case("not holds(true)", false, |other| {
            let foreign_projection = other
                .project(TlLimits::default())
                .expect("other genuine projection");
            let alternate = ExpectedTlJoin {
                projection: &foreign_projection,
                ..expected
            };
            assert!(matches!(
                read_tl_join(
                    joined.bytes(),
                    alternate,
                    TlLimits::default(),
                    &mut native_budget()
                ),
                Err(TlJoinError::ExpectedMismatch)
            ));
        });
    });
}

#[trace("IT-022-SC-11", "FR-350-AC-6")]
#[test]
fn strict_join_reader_rejects_each_noncanonical_or_cross_wired_encoding() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let joined = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("join");
        let expected = ExpectedTlJoin {
            projection: &projection,
            native: Some(&native),
            mapped: Some(&mapped),
            previous: None,
        };
        let original: Value = serde_json::from_slice(joined.bytes()).expect("join JSON");
        assert_eq!(serde_json::to_vec(&original).unwrap(), joined.bytes());
        let mut mutations = Vec::new();
        let mut unknown = original.clone();
        unknown["unknown"] = json!(true);
        mutations.push(serde_json::to_vec(&unknown).unwrap());
        let mut missing = original.clone();
        missing.as_object_mut().unwrap().remove("native");
        mutations.push(serde_json::to_vec(&missing).unwrap());
        let mut version = original.clone();
        version["version"] = json!("quire.native-temporal-join/v2");
        mutations.push(serde_json::to_vec(&version).unwrap());
        let mut decision = original.clone();
        decision["value"] = json!(false);
        mutations.push(serde_json::to_vec(&decision).unwrap());
        let mut cause = original.clone();
        cause["causes"] = json!(["result-refused"]);
        mutations.push(serde_json::to_vec(&cause).unwrap());
        let mut native_bytes = original.clone();
        native_bytes["native"] = json!("ff");
        mutations.push(serde_json::to_vec(&native_bytes).unwrap());
        let mut duplicate = b"{\"value\":true,".to_vec();
        duplicate.extend_from_slice(&joined.bytes()[1..]);
        mutations.push(duplicate);
        let mut out_of_order = b"{\"version\":\"quire.native-temporal-join/v1\",".to_vec();
        let mut without_version = original.clone();
        without_version.as_object_mut().unwrap().remove("version");
        out_of_order.extend_from_slice(&serde_json::to_vec(&without_version).unwrap()[1..]);
        mutations.push(out_of_order);
        let mut trailing = joined.bytes().to_vec();
        trailing.extend_from_slice(b" ");
        mutations.push(trailing);
        let mut truncated = joined.bytes().to_vec();
        truncated.pop();
        mutations.push(truncated);
        for changed in mutations {
            assert_ne!(changed, joined.bytes());
            assert!(
                matches!(
                    read_tl_join(
                        &changed,
                        expected,
                        TlLimits::default(),
                        &mut native_budget()
                    ),
                    Err(TlJoinError::ExpectedMismatch)
                ),
                "altered complete join must refuse"
            );
        }
        assert_eq!(
            read_tl_join(
                joined.bytes(),
                expected,
                TlLimits::default(),
                &mut native_budget()
            )
            .expect("failed reads did not modify expected owners")
            .bytes(),
            joined.bytes()
        );
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn support_work_and_byte_counters_accept_exact_and_refuse_one_short_with_fresh_usage() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let first = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("measure join");
        let usage = first.usage();
        assert!(usage.bytes > 0 && usage.support > 0 && usage.work > 0);
        assert_eq!(usage.causes, 0);
        let exact = TlLimits {
            max_bytes: usage.bytes,
            max_support: usage.support,
            max_work: usage.work,
            ..TlLimits::default()
        };
        let second = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            exact,
            &mut native_budget(),
        )
        .expect("exact join ceilings");
        assert_eq!(second.bytes(), first.bytes());
        assert_eq!(second.usage(), usage);
        let expected = ExpectedTlJoin {
            projection: &projection,
            native: Some(&native),
            mapped: Some(&mapped),
            previous: None,
        };
        let read = read_tl_join(first.bytes(), expected, exact, &mut native_budget())
            .expect("exact strict-read ceilings");
        assert_eq!(read.usage(), usage);
        for counter in [
            TlProjectionLimit::Bytes,
            TlProjectionLimit::Support,
            TlProjectionLimit::Work,
        ] {
            let mut short = exact;
            let setting = match counter {
                TlProjectionLimit::Bytes => &mut short.max_bytes,
                TlProjectionLimit::Support => &mut short.max_support,
                TlProjectionLimit::Work => &mut short.max_work,
                _ => unreachable!(),
            };
            *setting -= 1;
            let configured = *setting;
            let error = join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                short,
                &mut native_budget(),
            )
            .err()
            .expect("one-short join refuses");
            assert_limit(error, counter, configured);
            let error = read_tl_join(first.bytes(), expected, short, &mut native_budget())
                .err()
                .expect("one-short read refuses");
            assert_limit(error, counter, configured);
        }
        let both_short = TlLimits {
            max_support: 0,
            max_work: 0,
            ..exact
        };
        assert_limit(
            join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                both_short,
                &mut native_budget(),
            )
            .err()
            .expect("shared action crosses both ceilings"),
            TlProjectionLimit::Support,
            0,
        );
        assert_eq!(
            join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                exact,
                &mut native_budget()
            )
            .expect("counters reset after refusal")
            .usage(),
            usage
        );
    });
}

fn assert_limit(error: TlJoinError, expected_limit: TlProjectionLimit, configured: u64) {
    let TlJoinError::LimitReached {
        limit,
        value,
        setting,
    } = error
    else {
        panic!("named limit refusal required: {error:?}")
    };
    assert_eq!(limit, expected_limit);
    assert_eq!(setting, configured);
    assert_eq!(value, configured + 1);
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn ordered_causes_have_their_own_exact_and_one_short_ceiling() {
    with_case("holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.limits.valuations = 0;
        let projection = project_tl(
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
        .expect("projection");
        let native = evaluate_native(&projection);
        assert_eq!(native.outcome(), ReportOutcome::Limit);
        let first = join_tl(
            &projection,
            Some(&native),
            None,
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("two independent matching rows");
        assert_eq!(
            first.causes(),
            &[
                TlJoinCause::ResultUnavailable,
                TlJoinCause::ResultIncomplete
            ]
        );
        assert_eq!(first.usage().causes, 2);
        let exact = TlLimits {
            max_causes: 2,
            ..TlLimits::default()
        };
        assert_eq!(
            join_tl(
                &projection,
                Some(&native),
                None,
                None,
                exact,
                &mut native_budget()
            )
            .expect("exact cause ceiling")
            .bytes(),
            first.bytes()
        );
        assert_limit(
            join_tl(
                &projection,
                Some(&native),
                None,
                None,
                TlLimits {
                    max_causes: 1,
                    ..exact
                },
                &mut native_budget(),
            )
            .err()
            .expect("second cause exceeds one"),
            TlProjectionLimit::Causes,
            1,
        );
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn wrong_genuine_native_and_mapped_results_do_not_authorize_the_expected_join() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let joined = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("original join");
        let mut foreign = None;
        with_case("not holds(true)", false, |other| {
            let other_projection = other
                .project(TlLimits::default())
                .expect("other projection");
            foreign = Some((
                evaluate_native(&other_projection),
                evaluate_mapped(&other_projection),
            ));
        });
        let (wrong_native, wrong_mapped) = foreign.expect("genuine independent owner results");
        let expected = ExpectedTlJoin {
            projection: &projection,
            native: Some(&native),
            mapped: Some(&mapped),
            previous: None,
        };
        for changed in [
            ExpectedTlJoin {
                native: Some(&wrong_native),
                ..expected
            },
            ExpectedTlJoin {
                mapped: Some(&wrong_mapped),
                ..expected
            },
        ] {
            assert!(matches!(
                read_tl_join(
                    joined.bytes(),
                    changed,
                    TlLimits::default(),
                    &mut native_budget()
                ),
                Err(TlJoinError::ExpectedMismatch)
            ));
        }
        let refused = join_tl(
            &projection,
            Some(&native),
            Some(&wrong_mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("foreign genuine TL request is a refused state");
        assert_eq!(refused.decision(), TlJoinDecision::Refused);
        assert_eq!(refused.causes(), &[TlJoinCause::ResultRefused]);
        assert_eq!(refused.value(), None);
        assert_eq!(
            refused.mapped().unwrap().source().request(),
            wrong_mapped.source().request()
        );
        assert_ne!(wrong_mapped.source().request(), projection.request());
        let refused = join_tl(
            &projection,
            Some(&wrong_native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("native owner rejects foreign checked subject");
        assert_eq!(refused.decision(), TlJoinDecision::Refused);
        assert!(refused.owner_refusals().native().is_some());
        assert_eq!(refused.value(), None);
    });
}

#[trace("IT-022-SC-13", "FR-350-AC-5", "FR-350-AC-10")]
#[test]
fn genuine_correction_pairs_preserve_prior_bytes_and_reach_equal_final() {
    with_case("holds(true)", false, |case| {
        let original_projection = case
            .project(TlLimits::default())
            .expect("original projection");
        let original_native = evaluate_native(&original_projection);
        let original_mapped = evaluate_mapped(&original_projection);
        let prior = join_tl(
            &original_projection,
            Some(&original_native),
            Some(&original_mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("original join");
        let saved_prior = prior.bytes().to_vec();
        let saved_native = original_native.to_owner_bytes();
        let saved_mapped = original_mapped.bytes().to_vec();
        with_progress_correction(
            &original_projection,
            &original_native,
            |corrected_projection, corrected_native| {
                let native_limits = corrected_projection.selection().native_report_limits;
                let (_, encoded_usage) = original_native
                    .to_owner_bytes_bounded(native_limits)
                    .unwrap();
                assert!(encoded_usage.work > 0 && encoded_usage.storage > 0);
                let validation_usage = corrected_native
                    .validate_predecessor(&original_native, native_limits)
                    .unwrap();
                assert!(validation_usage.work > 0 && validation_usage.storage > 0);
                assert_eq!(
                    corrected_native.predecessor_owner_bytes(),
                    Some(saved_native.as_slice())
                );
                let subject = CheckedSubject::new(
                    corrected_projection.package(),
                    corrected_projection.native().occurrence,
                    corrected_projection.native().valuations,
                    corrected_projection.native().request,
                )
                .unwrap()
                .with_predecessor(&original_native, native_limits);
                let checked = Report::from_owner_bytes(
                    &corrected_native.to_owner_bytes(),
                    &subject,
                    corrected_projection.native().observation,
                    &mut native_budget(),
                )
                .unwrap();
                assert_eq!(
                    checked.predecessor_owner_bytes(),
                    Some(saved_native.as_slice())
                );
                assert_eq!(
                    corrected_native.predecessor(),
                    Some(original_native.result_identity())
                );
                let mut exhausted = Meter::new(scalar_limits());
                let ceiling = exhausted.limits().work_units;
                exhausted
                    .charge(
                        quire_exact::Charge::new(quire_exact::ChargePoint::TemporalVisit)
                            .work(quire_exact::Integer::from(ceiling)),
                    )
                    .unwrap();
                let original_subject = CheckedSubject::new(
                    original_projection.package(),
                    original_projection.native().occurrence,
                    original_projection.native().valuations,
                    original_projection.native().request,
                )
                .unwrap();
                let different_prior = native::evaluate_v2(
                    &original_subject,
                    original_projection.native().observation,
                    &mut exhausted,
                    &mut native_budget(),
                )
                .unwrap();
                assert_eq!(different_prior.outcome(), ReportOutcome::Meter);
                assert_eq!(
                    different_prior.result_identity(),
                    original_native.result_identity()
                );
                assert_ne!(different_prior.to_owner_bytes(), saved_native);
                assert!(matches!(
                    corrected_native.validate_predecessor(&different_prior, native_limits),
                    Err(native::ReportRefusal::InvalidPredecessor)
                ));
                let wrong_subject = CheckedSubject::new(
                    corrected_projection.package(),
                    corrected_projection.native().occurrence,
                    corrected_projection.native().valuations,
                    corrected_projection.native().request,
                )
                .unwrap()
                .with_predecessor(&different_prior, native_limits);
                assert!(matches!(
                    Report::from_owner_bytes(
                        &corrected_native.to_owner_bytes(),
                        &wrong_subject,
                        corrected_projection.native().observation,
                        &mut native_budget()
                    ),
                    Err(native::ReportRefusal::InvalidPredecessor)
                ));
                let wrong_prior_join = join_tl(
                    &original_projection,
                    Some(&different_prior),
                    Some(&original_mapped),
                    None,
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .unwrap();
                let limits = corrected_projection.selection().owner_limits;
                let corrected_source = evaluate_tl(
                    &corrected_projection,
                    ResultRelationInput::Superseding(original_mapped.source()),
                    limits,
                );
                assert_eq!(
                    corrected_source.relation_kind(),
                    report::ResultRelationKind::Superseding
                );
                let corrected_mapped = map_source(&corrected_source, limits);
                let missing = join_tl(
                    &corrected_projection,
                    Some(&corrected_native),
                    Some(&corrected_mapped),
                    None,
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .expect("missing prior is conflict");
                assert_eq!(missing.decision(), TlJoinDecision::Conflict);
                assert_eq!(missing.value(), None);
                let current_original_mapped = evaluate_mapped(corrected_projection);
                assert_eq!(
                    current_original_mapped.source().request(),
                    corrected_projection.request()
                );
                assert_eq!(
                    current_original_mapped.source().relation_kind(),
                    report::ResultRelationKind::Original
                );
                let mixed = join_tl(
                    &corrected_projection,
                    Some(&corrected_native),
                    Some(&current_original_mapped),
                    Some(&prior),
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .expect("mixed original/corrected pair conflicts");
                assert!(
                    mixed.native_admitted(),
                    "native correction admits the actual complete prior before relation comparison"
                );
                assert_eq!(mixed.decision(), TlJoinDecision::Conflict);
                assert_eq!(mixed.value(), None);
                let corrected = join_tl(
                    &corrected_projection,
                    Some(&corrected_native),
                    Some(&corrected_mapped),
                    Some(&prior),
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .expect("a correction over both direct predecessors joins");
                assert_eq!(corrected.decision(), TlJoinDecision::EqualFinal);
                assert!(corrected.causes().is_empty());
                assert!(corrected
                    .native_predecessor_usage()
                    .is_some_and(|usage| usage.work > 0 && usage.storage > 0));
                let wrong = join_tl(
                    &corrected_projection,
                    Some(&corrected_native),
                    Some(&corrected_mapped),
                    Some(&wrong_prior_join),
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .unwrap();
                assert_eq!(wrong.decision(), TlJoinDecision::Conflict);
                assert_eq!(wrong.value(), None);
                assert!(!wrong.native_admitted());
                assert!(matches!(
                    wrong.owner_refusals().native(),
                    Some(native::ReportRefusal::InvalidPredecessor)
                ));
                let wire: Value = serde_json::from_slice(wrong.bytes()).unwrap();
                assert_eq!(wire["nativeAdmitted"], false);
                let reread_wrong = read_tl_join(
                    wrong.bytes(),
                    ExpectedTlJoin {
                        projection: &corrected_projection,
                        native: Some(&corrected_native),
                        mapped: Some(&corrected_mapped),
                        previous: Some(&wrong_prior_join),
                    },
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .unwrap();
                assert_eq!(reread_wrong.bytes(), wrong.bytes());
                assert!(matches!(
                    read_tl_join(
                        wrong.bytes(),
                        ExpectedTlJoin {
                            projection: &corrected_projection,
                            native: Some(&corrected_native),
                            mapped: Some(&corrected_mapped),
                            previous: Some(&prior)
                        },
                        TlLimits::default(),
                        &mut native_budget()
                    ),
                    Err(TlJoinError::ExpectedMismatch)
                ));
                assert_eq!(prior.bytes(), saved_prior);
                assert_eq!(original_native.to_owner_bytes(), saved_native);
                assert_eq!(original_mapped.bytes(), saved_mapped);
            },
        );
    });
}

#[trace("IT-022-SC-12", "FR-350-AC-5", "FR-350-AC-6")]
#[test]
fn native_failed_execution_precedes_equal_boolean_and_retains_all_causes() {
    with_case("holds(true)", false, |case| {
        let mut request = case.native.request.clone();
        request.execution = qsl_eval::temporal::evidence::ExecutionDisposition::Failed;
        let projection = project_tl(
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
        .expect("failed assessment execution remains independent of projection premises");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        assert_eq!(native.outcome(), ReportOutcome::Evaluated);
        assert_eq!(native.truth(), Some(Truth::True));
        assert_eq!(mapped.source().execution(), AssessmentExecution::Completed);
        let joined = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("actual failed-execution join");
        assert_eq!(joined.decision(), TlJoinDecision::Failed);
        assert_eq!(joined.value(), None);
        assert_eq!(
            joined.causes(),
            &[TlJoinCause::ResultFailed, TlJoinCause::ResultMismatch]
        );
        assert_eq!(
            joined.projection().native().request.execution,
            qsl_eval::temporal::evidence::ExecutionDisposition::Failed
        );
        let read = read_tl_join(
            joined.bytes(),
            ExpectedTlJoin {
                projection: &projection,
                native: Some(&native),
                mapped: Some(&mapped),
                previous: None,
            },
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("strict failed join read");
        assert_eq!(read.bytes(), joined.bytes());
        assert_eq!(read.decision(), TlJoinDecision::Failed);
    });
}

#[trace("IT-022-SC-13", "FR-350-AC-5")]
#[test]
fn native_report_owner_limits_are_explicit_and_preserve_the_existing_ir_budget() {
    use native::{ReportPredecessorLimits, ReportPredecessorResource, ReportRefusal};
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).unwrap();
        let report = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let (bytes, usage) = report
            .to_owner_bytes_bounded(case.selection.native_report_limits)
            .unwrap();
        assert_eq!(bytes, report.to_owner_bytes());
        assert!(usage.work > 0 && usage.storage > 0);
        for (limits, refused_resource) in [
            (
                ReportPredecessorLimits::new(usage.work, usage.storage),
                None,
            ),
            (
                ReportPredecessorLimits::new(usage.work - 1, usage.storage),
                Some(ReportPredecessorResource::Work),
            ),
            (
                ReportPredecessorLimits::new(usage.work, usage.storage - 1),
                Some(ReportPredecessorResource::Storage),
            ),
        ] {
            let mut selection = case.selection;
            selection.native_report_limits = limits;
            let bounded = project_tl(
                case.package,
                case.subject,
                case.classification,
                case.surrounding,
                case.native,
                selection,
                TlLimits::default(),
            )
            .unwrap();
            let mut shared = native_budget();
            case.package
                .project_exact_type(
                    &case.native.valuations.rows()[0].operand,
                    &quire_contract_ir::SelectedExactIdentities::empty(),
                    &mut shared,
                )
                .unwrap();
            let before = shared.consumed();
            assert!(before.work > 0);
            let result = join_tl(
                &bounded,
                Some(&report),
                Some(&mapped),
                None,
                TlLimits::default(),
                &mut shared,
            );
            assert_eq!(shared.consumed(), before);
            if let Some(expected) = refused_resource {
                let error = result.unwrap_err();
                assert_eq!(
                    error.code(),
                    quire_integration::temporal::TlJoinCode::LimitReached
                );
                let TlJoinError::NativeReport(ReportRefusal::PredecessorLimit {
                    resource,
                    limit,
                    actual,
                }) = error
                else {
                    panic!("original bounded native report-owner denial");
                };
                assert_eq!(resource, expected);
                assert_eq!(
                    limit,
                    match resource {
                        ReportPredecessorResource::Work => limits.work,
                        ReportPredecessorResource::Storage => limits.storage,
                    }
                );
                assert!(actual > u128::from(limit));
            } else {
                let joined = result.unwrap();
                assert_eq!(joined.native_encoding_usage(), Some(usage));
                assert_eq!(joined.native_predecessor_usage(), None);
                assert!(joined.native_admitted());
                assert_eq!(joined.decision(), TlJoinDecision::EqualFinal);
                let reread = read_tl_join(
                    joined.bytes(),
                    ExpectedTlJoin {
                        projection: &bounded,
                        native: Some(&report),
                        mapped: Some(&mapped),
                        previous: None,
                    },
                    TlLimits::default(),
                    &mut shared,
                )
                .unwrap();
                assert_eq!(reread.native_encoding_usage(), Some(usage));
                assert_eq!(reread.bytes(), joined.bytes());
                assert_eq!(shared.consumed(), before);
            }
        }
    });
}

fn actual_correction_request(
    projection: &ValidatedTlProjection<'_>,
    prior: &Report,
    advanced: projection_support::Case<'_>,
) -> qsl_eval::temporal::evidence::EvidenceRequest {
    let input = projection.native();
    let selected = quire_contract_ir::CheckedOccurrence {
        role: input.occurrence.role.clone(),
        ordinal: input.occurrence.ordinal,
    };
    let key = ObligationKey::new(
        input.occurrence.node.package.clone(),
        serde_json::to_vec(&(&input.occurrence.node, selected)).unwrap(),
        input.occurrence.ordinal,
    );
    let ledger = prior.ledger().expect("actual native next-call ledger");
    let predecessor = ledger
        .result(&key, input.request.trigger.as_ref())
        .unwrap()
        .result_identity()
        .clone();
    let mut request = input.request.clone();
    request.ledger = Some(ledger.clone());
    request.relation = Relation::Superseding(predecessor);
    request.decision_progress = advanced.native.request.decision_progress;
    request.surrounding_progress = advanced.native.request.surrounding_progress;
    assert_ne!(request.decision_progress, input.request.decision_progress);
    assert_eq!(request.execution, input.request.execution);
    request
}

#[trace("IT-022-SC-13", "FR-350-AC-5", "FR-350-AC-10")]
#[test]
fn foreign_corrected_native_and_tl_reads_remain_refused_before_correction_authority() {
    with_case("holds(true)", false, |case| {
        let original = case.project(TlLimits::default()).unwrap();
        let original_native = evaluate_native(&original);
        let original_mapped = evaluate_mapped(&original);
        let prior = join_tl(
            &original,
            Some(&original_native),
            Some(&original_mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .unwrap();
        with_progress_correction(
            &original,
            &original_native,
            |corrected, corrected_native| {
                let corrected_mapped = map_source(
                    &evaluate_tl(
                        &corrected,
                        ResultRelationInput::Superseding(original_mapped.source()),
                        case.selection.owner_limits,
                    ),
                    case.selection.owner_limits,
                );
                let mut foreign = None;
                with_case("not holds(true)", false, |other| {
                    let original = other.project(TlLimits::default()).unwrap();
                    let native = evaluate_native(&original);
                    let mapped = evaluate_mapped(&original);
                    with_progress_correction(&original, &native, |projection, native| {
                        let mapped = map_source(
                            &evaluate_tl(
                                &projection,
                                ResultRelationInput::Superseding(mapped.source()),
                                other.selection.owner_limits,
                            ),
                            other.selection.owner_limits,
                        );
                        assert!(native.predecessor().is_some());
                        assert_eq!(
                            mapped.source().relation_kind(),
                            report::ResultRelationKind::Superseding
                        );
                        foreign = Some((native, mapped));
                    });
                });
                let (foreign_native, foreign_mapped) = foreign.unwrap();
                for previous in [None, Some(&prior)] {
                    let refused_native = join_tl(
                        &corrected,
                        Some(&foreign_native),
                        Some(&corrected_mapped),
                        previous,
                        TlLimits::default(),
                        &mut native_budget(),
                    )
                    .expect("foreign native binding precedes missing/wrong predecessor");
                    assert_eq!(refused_native.decision(), TlJoinDecision::Refused);
                    assert_eq!(refused_native.value(), None);
                    assert_eq!(
                        refused_native.causes().first(),
                        Some(&TlJoinCause::ResultRefused)
                    );
                    assert!(matches!(
                        refused_native.owner_refusals().native(),
                        Some(native::ReportRefusal::BindingMismatch(_))
                    ));
                    assert!(!refused_native.native_admitted());
                    let reread = read_tl_join(
                        refused_native.bytes(),
                        ExpectedTlJoin {
                            projection: &corrected,
                            native: Some(&foreign_native),
                            mapped: Some(&corrected_mapped),
                            previous,
                        },
                        TlLimits::default(),
                        &mut native_budget(),
                    )
                    .unwrap();
                    assert_eq!(reread.decision(), TlJoinDecision::Refused);
                    assert_eq!(reread.bytes(), refused_native.bytes());
                    let refused_tl = join_tl(
                &corrected,
                Some(&corrected_native),
                Some(&foreign_mapped),
                previous,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("foreign corrected TL refusal precedes unavailable TL predecessor authority");
                    assert_eq!(refused_tl.decision(), TlJoinDecision::Refused);
                    assert_eq!(refused_tl.value(), None);
                    assert_eq!(
                        refused_tl.causes().first(),
                        Some(&TlJoinCause::ResultRefused)
                    );
                    assert_eq!(refused_tl.native_admitted(), previous.is_some());
                    let reread = read_tl_join(
                        refused_tl.bytes(),
                        ExpectedTlJoin {
                            projection: &corrected,
                            native: Some(&corrected_native),
                            mapped: Some(&foreign_mapped),
                            previous,
                        },
                        TlLimits::default(),
                        &mut native_budget(),
                    )
                    .unwrap();
                    assert_eq!(reread.decision(), TlJoinDecision::Refused);
                    assert_eq!(reread.bytes(), refused_tl.bytes());
                }
            },
        );
    });
}

fn with_progress_correction(
    original: &ValidatedTlProjection<'_>,
    prior: &Report,
    test: impl FnOnce(&ValidatedTlProjection<'_>, Report),
) {
    projection_support::with_case_watermark(
        "holds(true)",
        false,
        Closure::Complete,
        OpenClosed::Closed,
        2,
        |advanced| {
            let request = actual_correction_request(original, prior, advanced);
            let input = original.native();
            let mut unchanged = request.clone();
            unchanged.decision_progress = input.request.decision_progress;
            unchanged.surrounding_progress = input.request.surrounding_progress;
            let unchanged_subject = CheckedSubject::new(
                original.package(),
                input.occurrence,
                input.valuations,
                &unchanged,
            )
            .unwrap()
            .with_predecessor(prior, original.selection().native_report_limits);
            assert!(
                matches!(
                    native::evaluate_v2(
                        &unchanged_subject,
                        input.observation,
                        &mut Meter::new(scalar_limits()),
                        &mut native_budget()
                    ),
                    Err(native::NativeReportError::Evaluation(_))
                ),
                "ledger/relation alone is an actual unchanged-input owner refusal"
            );
            let subject = qsl_package::checked_views::temporal_subject(
                original.package(),
                &input.occurrence.node,
            )
            .unwrap();
            let corrected = project_tl(
                original.package(),
                &subject,
                original.formula().classification(),
                Closure::Complete,
                NativeProjectionInput {
                    request: &request,
                    ..input
                },
                advanced.selection,
                TlLimits::default(),
            )
            .expect("actual advanced owner progress projects coherently");
            assert_eq!(
                corrected
                    .selection()
                    .observations
                    .decision_scope_progress
                    .payload()
                    .boundary(),
                advanced
                    .selection
                    .observations
                    .decision_scope_progress
                    .payload()
                    .boundary()
            );
            assert_eq!(corrected.native().request.decision_progress, Some(2));
            assert_eq!(
                corrected.native().request.execution,
                qsl_eval::temporal::evidence::ExecutionDisposition::Completed
            );
            assert_ne!(
                corrected.request(),
                original.request(),
                "actual admitted QObs progress advances the TL request too"
            );
            let report = evaluate_native_with_prior(&corrected, Some(prior));
            assert_eq!(report.outcome(), ReportOutcome::Evaluated);
            let original_input =
                qsl_eval::temporal::evidence::ResultIdentity::input_identity_from_bytes(
                    prior.result_identity(),
                )
                .unwrap();
            let corrected_input =
                qsl_eval::temporal::evidence::ResultIdentity::input_identity_from_bytes(
                    report.result_identity(),
                )
                .unwrap();
            assert_ne!(
                original_input, corrected_input,
                "real admitted progress changes owner input identity"
            );
            assert_eq!(report.predecessor(), Some(prior.result_identity()));
            report
                .validate_predecessor(prior, corrected.selection().native_report_limits)
                .expect("genuine changed-input native correction precedes outcome assertions");
            let checked = CheckedSubject::new(
                corrected.package(),
                input.occurrence,
                input.valuations,
                &request,
            )
            .unwrap()
            .with_predecessor(prior, corrected.selection().native_report_limits);
            let read = Report::from_owner_bytes(
                &report.to_owner_bytes(),
                &checked,
                input.observation,
                &mut native_budget(),
            )
            .expect("changed-input correction passes original strict native reader");
            assert_eq!(read.to_owner_bytes(), report.to_owner_bytes());
            test(&corrected, report);
        },
    );
}
