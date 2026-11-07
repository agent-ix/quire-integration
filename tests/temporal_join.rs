// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: join genuine native and mapped TL results without evaluation in the bridge.

#[path = "support/temporal_projection.rs"]
mod projection_support;
mod support;

use ix_trace_rs::trace;
use projection_support::{with_case, with_case_scopes};
use qsl_eval::temporal::evidence::{ObligationKey, Relation, Settlement, Truth};
use qsl_package::{checked_views::Closure, NativeJoinBudget};
use quire_exact::{Meter, ScalarLimits};
use quire_integration::temporal::{
    join_tl, project_tl, read_tl_join, ExpectedTlJoin, NativeProjectionInput, TlJoinCause,
    TlJoinDecision, TlJoinError, TlLimits, TlProjectionLimit, ValidatedTlProjection,
};
use quire_mltl::{
    contract_ir::{self, MappingSelection, ValidatedMappedResult},
    report::{self, AssessmentExecution, ResultRelationInput, ValidatedTemporalResult},
};
use quire_observation::authority::OpenClosed;
use quire_spec_language::temporal::{self as native, CheckedSubject, Report, ReportOutcome};
use serde_json::{json, Value};
use tl_mltl::wire::OwnerLimits;

fn native_budget() -> NativeJoinBudget {
    NativeJoinBudget::new(1 << 30, 1 << 30)
}

fn scalar_limits() -> ScalarLimits {
    ScalarLimits {
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
    }
}

/// Evaluation is explicit fixture preparation, never hidden inside join or read.
fn evaluate_native(projection: &ValidatedTlProjection<'_>) -> Report {
    let input = projection.native();
    let subject = CheckedSubject::new(
        projection.package(),
        input.occurrence,
        input.valuations,
        input.request,
    )
    .expect("actual native checked subject");
    native::evaluate_v2(
        &subject,
        input.observation,
        &mut Meter::new(scalar_limits()),
        &mut native_budget(),
    )
    .expect("explicit native fixture evaluation")
}

/// Both evaluator invocations remain outside the bridge under test.
fn evaluate_tl(
    projection: &ValidatedTlProjection<'_>,
    relation: ResultRelationInput<'_>,
    limits: OwnerLimits,
) -> ValidatedTemporalResult {
    let document = report::evaluate(projection.request(), relation, limits)
        .expect("explicit TL fixture evaluation");
    report::read(document.bytes(), projection.request(), relation, limits)
        .expect("actual TL source result reader")
}

fn map_source(source: &ValidatedTemporalResult, limits: OwnerLimits) -> ValidatedMappedResult {
    let selection = MappingSelection::for_result(source);
    let document = contract_ir::map(source, &selection, limits).expect("actual TL result mapper");
    contract_ir::read(document.bytes(), source, &selection, limits)
        .expect("actual non-evaluating TL mapped-result reader")
}

fn evaluate_mapped(projection: &ValidatedTlProjection<'_>) -> ValidatedMappedResult {
    let limits = projection.selection().owner_limits;
    map_source(
        &evaluate_tl(projection, ResultRelationInput::Original, limits),
        limits,
    )
}

#[trace("TC-461", "FR-350-AC-5", "FR-350-AC-6")]
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
            assert_eq!(wire["correspondence"], json!(projection.bytes()));
            assert_eq!(wire["native"], json!(original_native));
            assert_eq!(wire["mapped"], json!(original_mapped));
            assert_eq!(native.to_owner_bytes(), original_native);
            assert_eq!(mapped.bytes(), original_mapped);
            assert_eq!(mapped.source().bytes(), original_source);
        });
    }
}

#[trace("TC-461", "FR-350-AC-5")]
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

#[trace("TC-461", "FR-350-AC-5")]
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

#[trace("TC-461", "FR-350-AC-5")]
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

#[trace("TC-461", "FR-350-AC-5")]
#[test]
fn actual_owner_read_refusal_is_retained_and_never_compared_as_a_value() {
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
            &mut NativeJoinBudget::new(0, 0),
        )
        .expect("owner refusal becomes row state");
        assert_eq!(joined.decision(), TlJoinDecision::Refused);
        assert_eq!(joined.causes(), &[TlJoinCause::ResultRefused]);
        assert_eq!(joined.value(), None);
        assert!(joined.owner_refusals().native().is_some());
        assert!(joined.owner_refusals().mapped().is_none());
        assert_eq!(
            joined.native().unwrap().to_owner_bytes(),
            native.to_owner_bytes()
        );
        assert_eq!(joined.mapped().unwrap().bytes(), mapped.bytes());
    });
}

#[trace("TC-461", "FR-350-AC-5")]
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
        assert_eq!(
            joined.causes(),
            &[TlJoinCause::ResultIncomplete, TlJoinCause::ResultMismatch]
        );
        assert_eq!(
            joined.mapped().unwrap().source().execution(),
            AssessmentExecution::ResourceIncomplete
        );
    });
}

#[trace("TC-461", "FR-350-AC-5")]
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

#[trace("TC-461", "FR-350-AC-5", "FR-350-AC-6")]
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

#[trace("TC-461", "FR-350-AC-6")]
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
        native_bytes["native"][0] = json!(255);
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

#[trace("TC-461", "FR-350-AC-6")]
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

#[trace("TC-461", "FR-350-AC-6")]
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

#[trace("TC-461", "FR-350-AC-5", "FR-350-AC-6")]
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
        assert_eq!(
            refused.causes(),
            &[TlJoinCause::ResultRefused, TlJoinCause::ResultMismatch]
        );
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

#[trace("TC-461", "FR-350-AC-5")]
#[test]
fn genuine_correction_pairs_preserve_prior_bytes_and_refuse_missing_predecessor_authority() {
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
        let ledger = original_native
            .ledger()
            .expect("actual evaluator next-call ledger");
        let occurrence = case.native.occurrence;
        let checked_occurrence = quire_contract_ir::CheckedOccurrence {
            role: occurrence.role.clone(),
            ordinal: occurrence.ordinal,
        };
        let obligation = ObligationKey::new(
            occurrence.node.package.clone(),
            serde_json::to_vec(&(&occurrence.node, checked_occurrence))
                .expect("public native obligation key"),
            occurrence.ordinal,
        );
        let predecessor = ledger
            .result(&obligation, case.native.request.trigger.as_ref())
            .expect("actual original ledger result")
            .result_identity()
            .clone();
        assert_eq!(predecessor.as_bytes(), original_native.result_identity());
        let mut request = case.native.request.clone();
        request.ledger = Some(ledger.clone());
        request.relation = Relation::Superseding(predecessor);
        let corrected_projection = project_tl(
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
        .expect("correction retains actual native owner inputs");
        let corrected_native = evaluate_native(&corrected_projection);
        assert_eq!(
            corrected_native.predecessor(),
            Some(original_native.result_identity())
        );
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
        let mixed = join_tl(
            &corrected_projection,
            Some(&corrected_native),
            Some(&original_mapped),
            None,
            TlLimits::default(),
            &mut native_budget(),
        )
        .expect("mixed original/corrected pair conflicts");
        assert_eq!(mixed.decision(), TlJoinDecision::Conflict);
        assert_eq!(mixed.value(), None);
        assert!(matches!(
            join_tl(
                &corrected_projection,
                Some(&corrected_native),
                Some(&corrected_mapped),
                Some(&prior),
                TlLimits::default(),
                &mut native_budget()
            ),
            Err(TlJoinError::PredecessorAuthority)
        ));
        assert_eq!(prior.bytes(), saved_prior);
        assert_eq!(original_native.to_owner_bytes(), saved_native);
        assert_eq!(original_mapped.bytes(), saved_mapped);
    });
}
