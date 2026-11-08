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
use qsl_eval::temporal::evidence::{EvaluatedEvidence, EvidencePackage};
use qsl_eval::temporal::evidence::{ObligationKey, Relation, Settlement, Truth};
use qsl_package::checked_views::Closure;
use qsl_package::BarePackage;
use quire_eval::CallFailure;
use quire_exact::Meter;
use quire_integration::temporal::{
    join_tl, project_tl, read_tl_join, ExpectedTlJoin, NativeProjectionInput, TlJoinCause,
    TlJoinDecision, TlJoinError, TlLimits, TlProjectionLimit, ValidatedTlProjection,
};
use quire_mltl::report::{self, AssessmentExecution, ResultRelationInput};
use quire_observation::authority::OpenClosed;
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
            assert!(native
                .as_ref()
                .expect("successful owner evaluation")
                .result()
                .incomplete
                .is_none());
            assert_eq!(
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .truth,
                Some(if expected_value {
                    Truth::True
                } else {
                    Truth::False
                })
            );
            let original_native = format!(
                "{:?}",
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
            );
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
                format!("{:?}", joined.native().expect("retained native").result()),
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
            assert_eq!(
                wire["native"],
                json!(hex(native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .result_identity
                    .as_bytes()))
            );
            assert_eq!(wire["mapped"], json!(hex(&original_mapped)));
            assert_eq!(
                format!(
                    "{:?}",
                    native
                        .as_ref()
                        .expect("successful owner evaluation")
                        .result()
                ),
                original_native
            );
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
            assert_eq!(
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .truth,
                Some(Truth::Pending)
            );
            assert_eq!(
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .settlement,
                Some(Settlement::Unsettled)
            );
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
            assert_eq!(
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .truth,
                Some(Truth::Pending)
            );
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
        assert!(native
            .as_ref()
            .expect("successful owner evaluation")
            .result()
            .incomplete
            .is_some());
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
        assert!(joined.native().unwrap().result().incomplete.is_some());
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
            .expect_err("one-short join refuses");
            assert_limit(error, counter, configured);
            let error = read_tl_join(first.bytes(), expected, short, &mut native_budget())
                .expect_err("one-short read refuses");
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
            .expect_err("shared action crosses both ceilings"),
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
        assert!(native
            .as_ref()
            .expect("successful owner evaluation")
            .result()
            .incomplete
            .is_some());
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
            .expect_err("second cause exceeds one"),
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
fn genuine_correction_pairs_preserve_original_carriers_and_reach_equal_final() {
    with_case("holds(true)", false, |case| {
        let original = case.project(TlLimits::default()).unwrap();
        let native = evaluate_native(&original);
        let mapped = evaluate_mapped(&original);
        let prior = join_tl(
            &original,
            Some(&native),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .unwrap();
        let saved = format!(
            "{:?}",
            native
                .as_ref()
                .expect("successful owner evaluation")
                .result()
        );
        let prior_bytes = prior.bytes().to_vec();
        with_progress_correction(&original, &native, |projection, issued| {
            let source = evaluate_tl(
                projection,
                ResultRelationInput::Superseding(mapped.source()),
                projection.selection().owner_limits,
            );
            let corrected_mapped = map_source(&source, projection.selection().owner_limits);
            let joined = join_tl(
                projection,
                Some(&issued),
                Some(&corrected_mapped),
                Some(&prior),
                TlLimits::default(),
                &mut native_budget(),
            )
            .unwrap();
            assert_eq!(joined.decision(), TlJoinDecision::EqualFinal);
            assert_eq!(joined.value(), Some(true));
            assert!(std::ptr::eq(
                joined.native().unwrap(),
                issued.as_ref().unwrap()
            ));
            let read = read_tl_join(
                joined.bytes(),
                ExpectedTlJoin {
                    projection,
                    native: Some(&issued),
                    mapped: Some(&corrected_mapped),
                    previous: Some(&prior),
                },
                TlLimits::default(),
                &mut native_budget(),
            )
            .unwrap();
            assert_eq!(read.bytes(), joined.bytes());
            let missing = join_tl(
                projection,
                Some(&issued),
                Some(&corrected_mapped),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .unwrap();
            assert_eq!(missing.decision(), TlJoinDecision::Conflict);
            assert_eq!(missing.value(), None);
            assert!(!missing.native_admitted());
            let original_mapped = evaluate_mapped(projection);
            let mixed = join_tl(
                projection,
                Some(&issued),
                Some(&original_mapped),
                Some(&prior),
                TlLimits::default(),
                &mut native_budget(),
            )
            .unwrap();
            assert_eq!(mixed.decision(), TlJoinDecision::Conflict);
            assert_eq!(mixed.value(), None);
        });
        assert_eq!(
            format!(
                "{:?}",
                native
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
            ),
            saved
        );
        assert_eq!(prior.bytes(), prior_bytes);
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
        assert!(native
            .as_ref()
            .expect("successful owner evaluation")
            .result()
            .incomplete
            .is_none());
        assert_eq!(
            native
                .as_ref()
                .expect("successful owner evaluation")
                .result()
                .truth,
            Some(Truth::True)
        );
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
fn native_lineage_limit_is_the_owner_counter_and_preserves_prior_authority() {
    with_case("holds(true)", false, |case| {
        let original = case.project(TlLimits::default()).unwrap();
        let prior = evaluate_native(&original);
        let saved = format!(
            "{:?}",
            prior
                .as_ref()
                .expect("successful owner evaluation")
                .result()
        );
        projection_support::with_case_watermark(
            "holds(true)",
            false,
            Closure::Complete,
            OpenClosed::Closed,
            2,
            |advanced| {
                let mut request = actual_correction_request(&original, &prior, advanced);
                let bare = BarePackage::new(original.package());
                let package = EvidencePackage::from_package(&bare, &mut native_budget()).unwrap();
                for ceiling in [0, 1] {
                    request.limits.lineage = ceiling;
                    let issued = package
                        .evaluate_temporal_evidence(
                            &original.native().occurrence.node,
                            &request,
                            &mut Meter::new(scalar_limits()),
                        )
                        .unwrap();
                    assert_eq!(issued.result().lineage, 1);
                    assert_eq!(
                        issued.result().relation,
                        Relation::Superseding(
                            prior
                                .as_ref()
                                .expect("successful predecessor call")
                                .predecessor()
                                .clone()
                        )
                    );
                    if ceiling == 0 {
                        assert!(issued.result().incomplete.is_some());
                        assert_eq!(issued.result().truth, None);
                        assert_eq!(issued.result().usage.get(qsl_eval::temporal::evidence::EvidenceCounter::Instances), 0);
                        assert_eq!(issued.result().usage.get(qsl_eval::temporal::evidence::EvidenceCounter::Positions), 0);
                        assert_eq!(
                            issued.result().ledger,
                            prior
                                .as_ref()
                                .expect("successful owner evaluation")
                                .result()
                                .ledger
                        );
                    } else {
                        assert!(issued.result().incomplete.is_none());
                        assert_eq!(issued.result().truth, Some(Truth::True));
                    }
                }
            },
        );
        assert_eq!(
            format!(
                "{:?}",
                prior
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
            ),
            saved
        );
    });
}

fn actual_correction_request(
    projection: &ValidatedTlProjection<'_>,
    prior: &Result<EvaluatedEvidence, CallFailure>,
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
    let ledger = &prior
        .as_ref()
        .expect("successful owner evaluation")
        .result()
        .ledger;
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
                        corrected,
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
                                projection,
                                ResultRelationInput::Superseding(mapped.source()),
                                other.selection.owner_limits,
                            ),
                            other.selection.owner_limits,
                        );
                        assert!(!matches!(
                            native
                                .as_ref()
                                .expect("successful owner evaluation")
                                .result()
                                .relation,
                            Relation::Original
                        ));
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
                        corrected,
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
                        Some(
                            quire_integration::temporal::TlNativeMismatch::Subject
                                | quire_integration::temporal::TlNativeMismatch::Input
                        )
                    ));
                    assert!(!refused_native.native_admitted());
                    let reread = read_tl_join(
                        refused_native.bytes(),
                        ExpectedTlJoin {
                            projection: corrected,
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
                corrected,
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
                            projection: corrected,
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
    prior: &Result<EvaluatedEvidence, CallFailure>,
    test: impl FnOnce(&ValidatedTlProjection<'_>, Result<EvaluatedEvidence, CallFailure>),
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
            let bare = BarePackage::new(original.package());
            let package = EvidencePackage::from_package(&bare, &mut native_budget()).unwrap();
            let mut meter = Meter::new(scalar_limits());
            let refusal = package
                .evaluate_temporal_evidence(&input.occurrence.node, &unchanged, &mut meter)
                .expect_err("ledger/relation alone must refuse unchanged input");
            assert!(matches!(refusal, quire_eval::CallFailure::Admission(_)));
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
            .expect("actual advanced progress projects coherently");
            assert_eq!(corrected.native().request.decision_progress, Some(2));
            assert_ne!(corrected.request(), original.request());
            let issued = evaluate_native_with_prior(&corrected, Some(prior));
            assert_ne!(
                issued
                    .as_ref()
                    .expect("successful correction call")
                    .result()
                    .input_identity,
                prior
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .input_identity
            );
            assert_eq!(
                issued
                    .as_ref()
                    .expect("successful correction call")
                    .result()
                    .relation,
                Relation::Superseding(
                    prior
                        .as_ref()
                        .expect("successful predecessor call")
                        .predecessor()
                        .clone()
                )
            );
            assert_eq!(
                issued
                    .as_ref()
                    .expect("successful correction call")
                    .result()
                    .lineage,
                prior
                    .as_ref()
                    .expect("successful owner evaluation")
                    .result()
                    .lineage
                    + 1
            );
            assert!(issued
                .as_ref()
                .expect("successful correction call")
                .result()
                .incomplete
                .is_none());
            test(&corrected, issued);
        },
    );
}

/// Each lineage condition of FR-350's `L` is its own case and reaches R6 with no Boolean.
#[trace("IT-022-SC-13", "FR-350-AC-5", "FR-350-AC-10")]
#[test]
fn each_lineage_condition_reaches_conflict_in_its_own_case() {
    with_case("holds(true)", false, |case| {
        let original = case.project(TlLimits::default()).expect("projection");
        let native0 = evaluate_native(&original);
        let mapped0 = evaluate_mapped(&original);
        let prior = join_tl(
            &original,
            Some(&native0),
            Some(&mapped0),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("original join");
        let conflict =
            |joined: Result<quire_integration::temporal::ValidatedTlJoin<'_>, TlJoinError>,
             name: &str| {
                let joined = joined.unwrap_or_else(|error| panic!("{name}: {error:?}"));
                assert_eq!(joined.decision(), TlJoinDecision::Conflict, "{name}");
                assert_eq!(joined.causes(), &[TlJoinCause::ResultMismatch], "{name}");
                assert_eq!(joined.value(), None, "{name}");
            };
        // Both originals, yet a previous join is supplied.
        conflict(
            join_tl(
                &original,
                Some(&native0),
                Some(&mapped0),
                Some(&prior),
                TlLimits::default(),
                &mut native_budget(),
            ),
            "original with previous",
        );
        let native_only = join_tl(
            &original,
            Some(&native0),
            None,
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("native-only join");
        with_progress_correction(&original, &native0, |corrected, native1| {
            let limits = corrected.selection().owner_limits;
            let source1 = evaluate_tl(
                corrected,
                ResultRelationInput::Superseding(mapped0.source()),
                limits,
            );
            let mapped1 = map_source(&source1, limits);
            conflict(
                join_tl(
                    corrected,
                    Some(&native1),
                    Some(&mapped1),
                    None,
                    TlLimits::default(),
                    &mut native_budget(),
                ),
                "correction without previous",
            );
            // The previous join retained no TL result: missing-predecessor.
            conflict(
                join_tl(
                    corrected,
                    Some(&native1),
                    Some(&mapped1),
                    Some(&native_only),
                    TlLimits::default(),
                    &mut native_budget(),
                ),
                "TL missing-predecessor",
            );
            // The previous join retained a different complete TL result of equal
            // identity: predecessor-mismatch.
            let different = evaluate_mapped(corrected);
            let wrong_tl = join_tl(
                &original,
                Some(&native0),
                Some(&different),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("join over a foreign TL result");
            conflict(
                join_tl(
                    corrected,
                    Some(&native1),
                    Some(&mapped1),
                    Some(&wrong_tl),
                    TlLimits::default(),
                    &mut native_budget(),
                ),
                "TL predecessor-mismatch",
            );
        });
    });
}

/// Every pair of FR-350 rows R1 to R6 that genuine owner results can satisfy
/// together returns the first decision and both causes once, in row order.
#[trace("IT-022-SC-12", "FR-350-AC-5")]
#[test]
fn reachable_row_pairs_return_the_first_decision_and_both_causes() {
    use qsl_eval::temporal::evidence::ExecutionDisposition;
    with_case("holds(true)", false, |case| {
        let mut foreign = None;
        with_case("not holds(true)", false, |other| {
            let projection = other
                .project(TlLimits::default())
                .expect("other projection");
            foreign = Some(evaluate_mapped(&projection));
        });
        let foreign = foreign.expect("genuine result for another request");
        let pair = |projection: &ValidatedTlProjection<'_>,
                    native: Option<&Result<EvaluatedEvidence, CallFailure>>,
                    mapped: Option<&quire_mltl::contract_ir::ValidatedMappedResult>,
                    decision: TlJoinDecision,
                    causes: &[TlJoinCause],
                    name: &str| {
            let joined = join_tl(
                projection,
                native,
                mapped,
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(joined.decision(), decision, "{name}");
            assert_eq!(joined.causes(), causes, "{name}");
            assert_eq!(joined.value(), None, "{name}");
        };
        use TlJoinCause::{
            ResultFailed as R2, ResultIncomplete as R5, ResultRefused as R1,
            ResultUnavailable as R3,
        };
        // R1 + R3: a foreign TL result and no native report.
        let base = case.project(TlLimits::default()).expect("projection");
        pair(
            &base,
            None,
            Some(&foreign),
            TlJoinDecision::Refused,
            &[R1, R3],
            "R1+R3",
        );
        // R1 + R5: a foreign TL result and an incomplete native report.
        let mut limited = case.native.request.clone();
        limited.limits.valuations = 0;
        let limited_projection = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                request: &limited,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect("limited projection");
        let incomplete = evaluate_native(&limited_projection);
        assert!(incomplete
            .as_ref()
            .expect("successful owner evaluation")
            .result()
            .incomplete
            .is_some());
        pair(
            &limited_projection,
            Some(&incomplete),
            Some(&foreign),
            TlJoinDecision::Refused,
            &[R1, R5],
            "R1+R5",
        );
        // R3 + R5: no TL result and an incomplete native report.
        pair(
            &limited_projection,
            Some(&incomplete),
            None,
            TlJoinDecision::Unavailable,
            &[R3, R5],
            "R3+R5",
        );
        // R2 + R1, R2 + R3 and R2 + R5 over a failed assessment execution.
        let mut failed = case.native.request.clone();
        failed.execution = ExecutionDisposition::Failed;
        let failed_projection = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                request: &failed,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect("failed projection");
        let failed_native = evaluate_native(&failed_projection);
        let failed_mapped = evaluate_mapped(&failed_projection);
        pair(
            &failed_projection,
            Some(&failed_native),
            Some(&foreign),
            TlJoinDecision::Refused,
            &[R1, R2],
            "R1+R2",
        );
        pair(
            &failed_projection,
            None,
            Some(&failed_mapped),
            TlJoinDecision::Failed,
            &[R2, R3],
            "R2+R3",
        );
        let mut failed_limited = failed.clone();
        failed_limited.limits.valuations = 0;
        let failed_limited_projection = project_tl(
            case.package,
            case.subject,
            case.classification,
            case.surrounding,
            NativeProjectionInput {
                request: &failed_limited,
                ..case.native
            },
            case.selection,
            TlLimits::default(),
        )
        .expect("failed limited projection");
        let failed_incomplete = evaluate_native(&failed_limited_projection);
        pair(
            &failed_limited_projection,
            Some(&failed_incomplete),
            None,
            TlJoinDecision::Failed,
            &[R2, R3, R5],
            "R2+R3+R5",
        );
    });
}

/// A native correction against a foreign original TL result: the TL side is refused
/// (R1) and exactly one side is a correction (R6).
#[trace("IT-022-SC-12", "IT-022-SC-13", "FR-350-AC-5")]
#[test]
fn a_native_correction_against_a_foreign_original_tl_result_is_refused_and_conflicting() {
    with_case("holds(true)", false, |case| {
        let original = case.project(TlLimits::default()).expect("projection");
        let native0 = evaluate_native(&original);
        let mapped0 = evaluate_mapped(&original);
        let prior = join_tl(
            &original,
            Some(&native0),
            Some(&mapped0),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("original join");
        let mut foreign = None;
        with_case("not holds(true)", false, |other| {
            let projection = other
                .project(TlLimits::default())
                .expect("other projection");
            foreign = Some(evaluate_mapped(&projection));
        });
        let foreign = foreign.expect("genuine result for another request");
        with_progress_correction(&original, &native0, |corrected, native1| {
            let joined = join_tl(
                corrected,
                Some(&native1),
                Some(&foreign),
                Some(&prior),
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("join");
            assert_eq!(joined.decision(), TlJoinDecision::Refused);
            assert_eq!(
                joined.causes(),
                &[TlJoinCause::ResultRefused, TlJoinCause::ResultMismatch]
            );
            assert_eq!(joined.value(), None);
            // The native correction read and validated against its prior; the TL side
            // is refused because its request is not the projection's, which is not an
            // owner read refusal.
            assert!(joined.native_admitted());
            assert!(joined.owner_refusals().native().is_none());
            assert!(joined.owner_refusals().mapped().is_none());
        });
    });
}

#[trace("IT-022-SC-12", "IT-022-SC-15", "FR-350-AC-5", "FR-350-AC-6")]
#[test]
fn actual_native_call_refusal_is_retained_distinct_from_absence_without_evaluation() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).unwrap();
        let mapped = evaluate_mapped(&projection);
        let bare = BarePackage::new(case.package);
        let owner = EvidencePackage::from_package(&bare, &mut native_budget()).unwrap();
        let mut request = case.native.request.clone();
        request.binding.clock_name = "another-clock".into();
        let mut meter = Meter::new(scalar_limits());
        let outcome =
            owner.evaluate_temporal_evidence(&case.native.occurrence.node, &request, &mut meter);
        let original = outcome
            .as_ref()
            .expect_err("the actual owner refuses a different clock binding");
        assert!(matches!(original, CallFailure::Admission(_)));
        assert_eq!(meter.consumed(quire_exact::LimitKind::WorkUnits), 0);
        let joined = join_tl(
            &projection,
            Some(&outcome),
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .unwrap();
        assert_eq!(joined.decision(), TlJoinDecision::Refused);
        assert_eq!(joined.value(), None);
        assert!(!joined.native_admitted());
        assert!(joined.native().is_none());
        assert!(std::ptr::eq(joined.native_outcome().unwrap(), &outcome));
        assert!(std::ptr::eq(
            joined.native_call_failure().unwrap(),
            original
        ));
        assert_eq!(joined.causes().first(), Some(&TlJoinCause::ResultRefused));
        let record: Value = serde_json::from_slice(joined.bytes()).unwrap();
        assert_eq!(record["native"]["failure"], "admission");
        let quire_eval::CallFailure::Admission(admission) = original else {
            unreachable!()
        };
        let diagnostic = match admission.as_ref() {
            quire_eval::CallAdmissionFailure::Refused(record)
            | quire_eval::CallAdmissionFailure::Incomplete(record) => record,
        };
        let work = u64::try_from(diagnostic.record.fields.len() + diagnostic.record.path.len() + 1)
            .unwrap();
        assert_eq!(joined.usage().work, work);
        let mut limits = TlLimits {
            max_work: work,
            ..TlLimits::default()
        };
        let exact = join_tl(
            &projection,
            Some(&outcome),
            Some(&mapped),
            None,
            limits,
            &mut native_budget(),
        )
        .unwrap();
        assert_eq!(exact.bytes(), joined.bytes());
        limits.max_work = work - 1;
        assert!(
            matches!(join_tl(&projection, Some(&outcome), Some(&mapped), None, limits, &mut native_budget()),
            Err(TlJoinError::LimitReached { limit: TlProjectionLimit::Work, value, setting }) if value == work && setting == work - 1)
        );
        let bytes = u64::try_from(joined.bytes().len()).unwrap();
        limits = TlLimits {
            max_bytes: bytes,
            ..TlLimits::default()
        };
        assert_eq!(
            join_tl(
                &projection,
                Some(&outcome),
                Some(&mapped),
                None,
                limits,
                &mut native_budget()
            )
            .unwrap()
            .bytes(),
            joined.bytes()
        );
        limits.max_bytes -= 1;
        assert!(
            matches!(join_tl(&projection, Some(&outcome), Some(&mapped), None, limits, &mut native_budget()),
            Err(TlJoinError::LimitReached { limit: TlProjectionLimit::Bytes, value, setting }) if value == bytes && setting == bytes - 1)
        );

        let reread = read_tl_join(
            joined.bytes(),
            ExpectedTlJoin {
                projection: &projection,
                native: Some(&outcome),
                mapped: Some(&mapped),
                previous: None,
            },
            TlLimits::default(),
            &mut native_budget(),
        )
        .unwrap();
        assert_eq!(reread.bytes(), joined.bytes());
        assert_eq!(reread.native_call_failure(), Some(original));
        assert_eq!(meter.consumed(quire_exact::LimitKind::WorkUnits), 0);
        let absent = join_tl(
            &projection,
            None,
            Some(&mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .unwrap();
        assert_eq!(absent.decision(), TlJoinDecision::Unavailable);
        assert_ne!(absent.bytes(), joined.bytes());
        request.binding.clock_name = "third-clock".into();
        let different =
            owner.evaluate_temporal_evidence(&case.native.occurrence.node, &request, &mut meter);
        assert!(different.is_err());
        assert!(matches!(
            read_tl_join(
                joined.bytes(),
                ExpectedTlJoin {
                    projection: &projection,
                    native: Some(&different),
                    mapped: Some(&mapped),
                    previous: None,
                },
                TlLimits::default(),
                &mut native_budget()
            ),
            Err(TlJoinError::ExpectedMismatch)
        ));
    });
}
