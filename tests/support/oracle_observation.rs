// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mechanical observation shared by the host and generated native test.

use quire_exact::{LimitKind, Meter, Outcome};

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
