// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-113: source/native selected-model quantity subcells; generated remains Planned.
#[path = "support/tc187_p8_model_values.rs"]
mod model;

use ix_trace_rs::trace;

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-23")]
#[test]
fn probe_u38_original_equivalent_conversion_positive() {
    model::bind("U38", &["convert<Quantity<F::s>>(3 R::s)"], &["3 F::s"]);
    model::checked(&format!("{} function observe using v(): Quantity<F::s> pure {{ convert<Quantity<F::s>>(3 R::s) }}", model::RF), false);
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-26")]
#[test]
fn probe_u40_original_equivalent_conversion_positive() {
    model::bind(
        "U40",
        &["convert<Quantity<G::ms>>(3 R::s)"],
        &["3000 G::ms"],
    );
    model::checked(&format!("{} function observe using v(): Quantity<G::ms> pure {{ convert<Quantity<G::ms>>(3 R::s) }}", model::RG), true);
}
