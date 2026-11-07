// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350 limits: each count is computed from the inputs, never from the bridge's usage.

#[path = "support/temporal_join_owners.rs"]
mod owners;
#[path = "support/temporal_projection.rs"]
mod projection_support;
mod support;

use ix_trace_rs::trace;
use owners::{evaluate_mapped, evaluate_native, native_budget};
use projection_support::{with_case, Case};
use quire_integration::temporal::{
    join_tl, read_tl_bridge_definition, read_tl_join, read_tl_projection, ExpectedTlJoin,
    ExpectedTlProjection, TlJoinError, TlLimits, TlProjectionError, TlProjectionLimit,
    ValidatedTlProjection, TL_BRIDGE_DEFINITION,
};

const HIGH: u64 = 1 << 40;

fn high() -> TlLimits {
    TlLimits {
        max_bytes: HIGH,
        max_nodes: HIGH,
        max_positions: HIGH,
        max_valuations: HIGH,
        max_support: HIGH,
        max_causes: HIGH,
        max_work: HIGH,
    }
}

fn set(limit: TlProjectionLimit, value: u64) -> TlLimits {
    let mut limits = high();
    match limit {
        TlProjectionLimit::Bytes => limits.max_bytes = value,
        TlProjectionLimit::Nodes => limits.max_nodes = value,
        TlProjectionLimit::Positions => limits.max_positions = value,
        TlProjectionLimit::Valuations => limits.max_valuations = value,
        TlProjectionLimit::Support => limits.max_support = value,
        TlProjectionLimit::Causes => limits.max_causes = value,
        TlProjectionLimit::Work => limits.max_work = value,
    }
    limits
}

fn len(count: usize) -> u64 {
    u64::try_from(count).expect("small count")
}

/// FR-350's `max_bytes` count for a projection: the largest checked byte value.
fn projection_bytes(case: &Case<'_>, projection: &ValidatedTlProjection<'_>) -> u64 {
    let views = case.selection.observations;
    let propositions = projection
        .propositions()
        .canonical_json_bytes()
        .expect("owner proposition map bytes");
    [
        case.selection.bridge_definition.bytes().len(),
        case.selection.observation.bundle().bytes().len(),
        case.selection.clock.bytes().len(),
        views.decision_scope_progress.bytes().len(),
        views.decision_scope_closure.bytes().len(),
        views.surrounding_execution_progress.bytes().len(),
        views.surrounding_execution_closure.bytes().len(),
        views.completeness.bytes().len(),
        views.availability.bytes().len(),
        projection.formula().bytes().len(),
        propositions.len(),
        projection.input().bytes().len(),
        projection.request().bytes().len(),
        projection.bytes().len(),
    ]
    .into_iter()
    .map(len)
    .max()
    .expect("non-empty")
}

fn lengths(case: &Case<'_>, projection: &ValidatedTlProjection<'_>) -> Vec<usize> {
    let views = case.selection.observations;
    vec![
        case.selection.bridge_definition.bytes().len(),
        case.selection.observation.bundle().bytes().len(),
        case.selection.clock.bytes().len(),
        views.decision_scope_progress.bytes().len(),
        views.completeness.bytes().len(),
        views.availability.bytes().len(),
        projection.formula().bytes().len(),
        projection.input().bytes().len(),
        projection.request().bytes().len(),
        projection.bytes().len(),
    ]
}

/// FR-350's `max_work` count for a projection, summed item by item from the inputs.
fn projection_work(case: &Case<'_>, projection: &ValidatedTlProjection<'_>) -> u64 {
    let request = case.native.request;
    let ledger = request.ledger.as_ref();
    let leaves = case.subject.predicate_leaves().len();
    let tl_positions = request.positions.len();
    len(case.package.source_map().len()
        + case
            .selection
            .observation
            .bundle()
            .payload()
            .records()
            .len()
        + request.positions.len()
        + case.native.valuations.rows().len()
        + tl_positions * leaves
        + projection.formula().formula().nodes().len()
        + projection.propositions().propositions().len()
        + case.subject.formula_graph().len()
        + case.package.selected_model_packages().count()
        + ledger.map_or(0, |ledger| ledger.instance_count() + ledger.result_count())
        + request.deliveries.len()
        + request
            .deliveries
            .iter()
            .map(|delivery| delivery.captures.len())
            .sum::<usize>()
        + request.origins.len()
        + request
            .origins
            .iter()
            .map(|origin| origin.captures.len())
            .sum::<usize>()
        + request.evicted.len())
}

#[trace("IT-022-SC-14", "FR-350-AC-8")]
#[test]
fn defaults_are_the_seven_fr_350_settings() {
    assert_eq!(
        TlLimits::default(),
        TlLimits {
            max_bytes: 16_777_216,
            max_nodes: 65_536,
            max_positions: 1_048_576,
            max_valuations: 16_777_216,
            max_support: 1_048_576,
            max_causes: 64,
            max_work: 4_294_967_296,
        }
    );
    for (limit, name) in [
        (
            TlProjectionLimit::Bytes,
            "temporal.correspondence.max_bytes",
        ),
        (
            TlProjectionLimit::Nodes,
            "temporal.correspondence.max_nodes",
        ),
        (
            TlProjectionLimit::Positions,
            "temporal.correspondence.max_positions",
        ),
        (
            TlProjectionLimit::Valuations,
            "temporal.correspondence.max_valuations",
        ),
        (
            TlProjectionLimit::Support,
            "temporal.correspondence.max_support",
        ),
        (
            TlProjectionLimit::Causes,
            "temporal.correspondence.max_causes",
        ),
        (TlProjectionLimit::Work, "temporal.correspondence.max_work"),
    ] {
        assert_eq!(limit.setting(), name);
    }
}

fn assert_projection_limit(error: TlProjectionError, limit: TlProjectionLimit, n: u64) {
    match error {
        TlProjectionError::LimitReached {
            limit: reached,
            value,
            setting,
            ..
        } => {
            assert_eq!(reached, limit);
            assert_eq!(value, n);
            assert_eq!(setting, n - 1);
        }
        other => panic!("limit-reached required, got {other:?}"),
    }
}

#[trace("IT-022-SC-14", "FR-350-AC-8")]
#[test]
fn projection_limits_admit_at_the_computed_count_and_refuse_one_below() {
    with_case("holds(true) and holds(false)", false, |case| {
        let projection = case.project(high()).expect("unconstrained projection");
        let counts = [
            (
                TlProjectionLimit::Bytes,
                projection_bytes(&case, &projection),
            ),
            (
                TlProjectionLimit::Nodes,
                len(projection.formula().formula().nodes().len()),
            ),
            (
                TlProjectionLimit::Positions,
                len(case.native.request.positions.len()),
            ),
            (
                TlProjectionLimit::Valuations,
                len(case.native.valuations.rows().len()),
            ),
            (TlProjectionLimit::Work, projection_work(&case, &projection)),
        ];
        for (limit, n) in counts {
            let admitted = case.project(set(limit, n)).expect("n admits");
            let usage = admitted.usage();
            let reported = match limit {
                TlProjectionLimit::Bytes => usage.bytes,
                TlProjectionLimit::Nodes => usage.nodes,
                TlProjectionLimit::Positions => usage.positions,
                TlProjectionLimit::Valuations => usage.valuations,
                TlProjectionLimit::Work => usage.work,
                _ => unreachable!(),
            };
            assert_eq!(
                reported,
                n,
                "{limit:?} usage; lengths {:?}",
                lengths(&case, &projection)
            );
            assert_projection_limit(
                case.project(set(limit, n - 1))
                    .err()
                    .expect("n - 1 refuses"),
                limit,
                n,
            );
            // The strict reader charges the same counts.
            let expected = ExpectedTlProjection {
                package: case.package,
                subject: case.subject,
                classification: case.classification,
                surrounding: case.surrounding,
                native: case.native,
                selection: case.selection,
            };
            let read = read_tl_projection(admitted.bytes(), expected, set(limit, n))
                .expect("reader admits at n");
            assert_eq!(read.bytes(), admitted.bytes());
            let expected = ExpectedTlProjection {
                package: case.package,
                subject: case.subject,
                classification: case.classification,
                surrounding: case.surrounding,
                native: case.native,
                selection: case.selection,
            };
            assert_projection_limit(
                read_tl_projection(admitted.bytes(), expected, set(limit, n - 1))
                    .err()
                    .expect("reader refuses at n - 1"),
                limit,
                n,
            );
        }
    });
}

#[trace("IT-022-SC-14", "FR-350-AC-8")]
#[test]
fn one_action_crossing_two_limits_returns_the_first_in_table_order() {
    with_case("holds(true) and holds(false)", false, |case| {
        let rows = len(case.native.valuations.rows().len());
        let request = case.native.request;
        // Work already charged when the valuation rows are charged together
        // with the valuation counter: the instance visits, the bundle records
        // and the request positions.
        let before = len(request.origins.len()
            + case
                .selection
                .observation
                .bundle()
                .payload()
                .records()
                .len()
            + request.positions.len());
        let mut limits = high();
        limits.max_valuations = rows - 1;
        limits.max_work = before;
        assert_projection_limit(
            case.project(limits).err().expect("both limits crossed"),
            TlProjectionLimit::Valuations,
            rows,
        );
    });
}

#[trace("IT-022-SC-14", "FR-350-AC-8")]
#[test]
fn bridge_definition_reader_charges_only_its_record_length() {
    let n = len(TL_BRIDGE_DEFINITION.len());
    let admitted =
        read_tl_bridge_definition(TL_BRIDGE_DEFINITION, set(TlProjectionLimit::Bytes, n))
            .expect("n admits");
    assert_eq!(admitted.usage().bytes, n);
    assert!(
        read_tl_bridge_definition(TL_BRIDGE_DEFINITION, set(TlProjectionLimit::Bytes, n - 1))
            .is_err()
    );
}

fn assert_join_limit(error: TlJoinError, limit: TlProjectionLimit, n: u64) {
    match error {
        TlJoinError::LimitReached {
            limit: reached,
            value,
            setting,
        } => {
            assert_eq!(reached, limit);
            assert_eq!(value, n);
            assert_eq!(setting, n - 1);
        }
        other => panic!("limit-reached required, got {other:?}"),
    }
}

#[trace("IT-022-SC-14", "FR-350-AC-8")]
#[test]
fn join_limits_admit_at_the_computed_count_and_refuse_one_below() {
    with_case("holds(true)", false, |case| {
        let projection = case.project(TlLimits::default()).expect("projection");
        let native = evaluate_native(&projection);
        let mapped = evaluate_mapped(&projection);
        let reference = join_tl(
            &projection,
            Some(&native),
            Some(&mapped),
            None,
            high(),
            &mut native_budget(),
        )
        .expect("unconstrained join");
        let support = len(native.support().len() + mapped.source().decision_support().len());
        let bytes = [
            native.to_owner_bytes().len(),
            mapped.bytes().len(),
            mapped.source().bytes().len(),
            reference.bytes().len(),
        ]
        .into_iter()
        .map(len)
        .max()
        .expect("non-empty");
        let causes = len(reference.causes().len());
        assert!(support > 0 && causes == 0);
        for (limit, n) in [
            (TlProjectionLimit::Bytes, bytes),
            (TlProjectionLimit::Support, support),
            (TlProjectionLimit::Work, support),
        ] {
            let admitted = join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                set(limit, n),
                &mut native_budget(),
            )
            .expect("n admits");
            let reported = match limit {
                TlProjectionLimit::Bytes => admitted.usage().bytes,
                TlProjectionLimit::Support => admitted.usage().support,
                _ => admitted.usage().work,
            };
            assert_eq!(reported, n, "{limit:?} usage");
            assert_join_limit(
                join_tl(
                    &projection,
                    Some(&native),
                    Some(&mapped),
                    None,
                    set(limit, n - 1),
                    &mut native_budget(),
                )
                .err()
                .expect("n - 1 refuses"),
                limit,
                n,
            );
            let expected = ExpectedTlJoin {
                projection: &projection,
                native: Some(&native),
                mapped: Some(&mapped),
                previous: None,
            };
            read_tl_join(
                admitted.bytes(),
                expected,
                set(limit, n),
                &mut native_budget(),
            )
            .expect("reader admits at n");
            assert_join_limit(
                read_tl_join(
                    admitted.bytes(),
                    expected,
                    set(limit, n - 1),
                    &mut native_budget(),
                )
                .err()
                .expect("reader refuses at n - 1"),
                limit,
                n,
            );
        }
        // Support and work are crossed by one action: support is first in table order.
        let mut both = high();
        both.max_support = support - 1;
        both.max_work = support - 1;
        assert_join_limit(
            join_tl(
                &projection,
                Some(&native),
                Some(&mapped),
                None,
                both,
                &mut native_budget(),
            )
            .err()
            .expect("both crossed"),
            TlProjectionLimit::Support,
            support,
        );
    });
}
