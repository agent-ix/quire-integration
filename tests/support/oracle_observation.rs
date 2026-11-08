// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mechanical observation shared by the host and generated native test.

use quire_exact::{ChargeAdmission, ChargeEvent, ChargePoint, LimitKind, Meter, Outcome};

/// Preserve the entire typed result, loss descriptors, ordered admitted charges
/// and all counters. This formats observations; it evaluates no operation.
pub fn snapshot<T: std::fmt::Debug, L: std::fmt::Debug>(
    outcome: &Outcome<T>,
    losses: &L,
    meter: &Meter,
) -> String {
    format!(
        "{:?}",
        (
            outcome,
            losses,
            meter.admitted_charges(),
            LimitKind::ALL.map(|kind| (kind, meter.consumed(kind))),
        )
    )
}

/// Preserve the typed result, the ordered admitted per-charge requests, the
/// points of denied requests and all counters. This formats owner observations;
/// it evaluates no operation.
#[allow(dead_code)]
pub fn ledger_snapshot<T: std::fmt::Debug>(
    outcome: &Outcome<T>,
    admitted: &[&ChargeEvent],
    denied: &[ChargePoint],
    meter: &Meter,
) -> String {
    format!(
        "{:?}",
        (
            outcome,
            admitted,
            denied,
            LimitKind::ALL.map(|kind| (kind, meter.consumed(kind))),
        )
    )
}

/// Split a captured interval into its admitted requests and denied points.
#[allow(dead_code)]
pub fn split(events: &[ChargeEvent]) -> (Vec<&ChargeEvent>, Vec<ChargePoint>) {
    let admitted = events
        .iter()
        .filter(|event| event.admission == ChargeAdmission::Admitted)
        .collect();
    let denied = events
        .iter()
        .filter(|event| event.admission == ChargeAdmission::Denied)
        .map(|event| event.point)
        .collect();
    (admitted, denied)
}
