// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-backed Boolean observations through the native and TL bridge owners.

#[path = "support/temporal_join_owners.rs"]
mod owners;
#[path = "support/temporal_projection.rs"]
mod projection_support;
mod support;

use ix_trace_rs::trace;
use owners::{evaluate_mapped, evaluate_native, native_budget};
use projection_support::with_case;
use qsl_eval::temporal::evidence::Truth;
use quire_integration::temporal::{
    join_tl, read_tl_join, read_tl_projection, ExpectedTlJoin, ExpectedTlProjection, TlInput,
    TlJoinCause, TlJoinDecision, TlLimits,
};
use quire_mltl::{contract_ir::MappedOutcome, report::TemporalTruth};
use quire_spec_language::temporal::ReportOutcome;
use tl_syntax::PropositionId;

#[trace("TC-115", "FR-350-AC-1")]
#[trace("TC-115", "FR-350-AC-5")]
#[trace("TC-115", "FR-350-AC-6")]
#[test]
fn two_distinct_valuation_rows_reach_both_owner_results_and_the_strict_join() {
    for (formula, expected) in [
        ("holds(true) or holds(false)", true),
        ("holds(true) and holds(false)", false),
    ] {
        with_case(formula, false, |case| {
            let rows = case.native.valuations.rows();
            assert_eq!(rows.len(), 2);
            assert_eq!((rows[0].value, rows[1].value), (true, false));
            assert_ne!(rows[0].operand, rows[1].operand);
            let projection = case.project(TlLimits::default()).expect("projection");
            let read = read_tl_projection(
                projection.bytes(),
                ExpectedTlProjection {
                    package: case.package,
                    subject: case.subject,
                    classification: case.classification,
                    surrounding: case.surrounding,
                    native: case.native,
                    selection: case.selection,
                },
                TlLimits::default(),
            )
            .expect("strict correspondence read");
            assert_eq!(read.bytes(), projection.bytes());
            assert_eq!(read.source_positions(), &[0]);
            let TlInput::Future(trace) = read.input() else {
                panic!("the future formula must retain a TL trace")
            };
            assert_eq!(trace.document().instants, vec![vec![PropositionId(0)]]);
            assert!(trace.document().closed);
            let native = evaluate_native(&read);
            let mapped = evaluate_mapped(&read);
            assert_eq!(native.outcome(), ReportOutcome::Evaluated);
            assert_eq!(
                native.truth(),
                Some(if expected { Truth::True } else { Truth::False })
            );
            assert_eq!(
                mapped.source().truth(),
                if expected {
                    TemporalTruth::True
                } else {
                    TemporalTruth::False
                }
            );
            assert_eq!(mapped.outcome(), MappedOutcome::Value { value: expected });
            let joined = join_tl(
                &read,
                Some(&native),
                Some(&mapped),
                None,
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("independent owner results join");
            let reread = read_tl_join(
                joined.bytes(),
                ExpectedTlJoin {
                    projection: &read,
                    native: Some(&native),
                    mapped: Some(&mapped),
                    previous: None,
                },
                TlLimits::default(),
                &mut native_budget(),
            )
            .expect("strict join read");
            assert_eq!(reread.decision(), TlJoinDecision::EqualFinal);
            assert_eq!(reread.value(), Some(expected));
            assert!(reread.causes().is_empty());
            assert_eq!(reread.bytes(), joined.bytes());
            assert_eq!(
                reread.native().expect("native retained").to_owner_bytes(),
                native.to_owner_bytes()
            );
            assert_eq!(
                reread.mapped().expect("mapped retained").bytes(),
                mapped.bytes()
            );
            assert_eq!(
                reread.mapped().expect("source retained").source().bytes(),
                mapped.source().bytes()
            );
        });
    }
}

#[trace("TC-115", "FR-350-AC-5")]
#[trace("TC-115", "FR-350-AC-6")]
#[test]
fn equal_true_results_from_another_formula_do_not_authorize_the_join() {
    with_case("holds(true) or holds(false)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        with_case("not (holds(true) and holds(false))", false, |other| {
            let foreign = other
                .project(TlLimits::default())
                .expect("foreign projection");
            let foreign_native = evaluate_native(&foreign);
            let foreign_mapped = evaluate_mapped(&foreign);
            assert_eq!(native.truth(), Some(Truth::True));
            assert_eq!(foreign_native.truth(), Some(Truth::True));
            assert_eq!(mapped.source().truth(), TemporalTruth::True);
            assert_eq!(foreign_mapped.source().truth(), TemporalTruth::True);
            assert_eq!(mapped.outcome(), MappedOutcome::Value { value: true });
            assert_eq!(
                foreign_mapped.outcome(),
                MappedOutcome::Value { value: true }
            );
            assert_ne!(foreign.request(), projection.request());
            assert_ne!(foreign.bytes(), projection.bytes());
            for (left, right) in [(&foreign_native, &mapped), (&native, &foreign_mapped)] {
                let joined = join_tl(
                    &projection,
                    Some(left),
                    Some(right),
                    None,
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .expect("foreign owner inputs yield a refused record");
                assert_eq!(joined.decision(), TlJoinDecision::Refused);
                assert_eq!(joined.causes(), &[TlJoinCause::ResultRefused]);
                assert_eq!(joined.value(), None);
                let read = read_tl_join(
                    joined.bytes(),
                    ExpectedTlJoin {
                        projection: &projection,
                        native: Some(left),
                        mapped: Some(right),
                        previous: None,
                    },
                    TlLimits::default(),
                    &mut native_budget(),
                )
                .expect("refusal itself roundtrips through the strict reader");
                assert_eq!(read.decision(), TlJoinDecision::Refused);
                assert_eq!(read.value(), None);
                assert_eq!(read.bytes(), joined.bytes());
            }
        });
    });
}
