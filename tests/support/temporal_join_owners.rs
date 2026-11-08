// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit owner fixture preparation shared by the join and limit targets.
//!
//! Evaluation happens here, outside the bridge under test.

#![allow(dead_code)] // Each target uses a different subset of these helpers.

use qsl_package::NativeJoinBudget;
use quire_exact::{Meter, ScalarLimits};
use quire_integration::temporal::ValidatedTlProjection;
use quire_mltl::{
    contract_ir::{self, MappingSelection, ValidatedMappedResult},
    report::{self, ResultRelationInput, ValidatedTemporalResult},
};
use quire_spec_language::temporal::{self as native, CheckedSubject, Report};
use tl_mltl::wire::OwnerLimits;

/// Lowercase hexadecimal spelling of owner bytes, as the records write them.
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn native_budget() -> NativeJoinBudget {
    NativeJoinBudget::new(1 << 30, 1 << 30)
}

pub(crate) fn scalar_limits() -> ScalarLimits {
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
pub(crate) fn evaluate_native(projection: &ValidatedTlProjection<'_>) -> Report {
    evaluate_native_with_prior(projection, None)
}

pub(crate) fn evaluate_native_with_prior(
    projection: &ValidatedTlProjection<'_>,
    prior: Option<&Report>,
) -> Report {
    let input = projection.native();
    let subject = CheckedSubject::new(
        projection.package(),
        input.occurrence,
        input.valuations,
        input.request,
    )
    .expect("actual native checked subject");
    let subject = if let Some(prior) = prior {
        subject.with_predecessor(prior, projection.selection().native_report_limits)
    } else {
        subject
    };
    native::evaluate_v2(
        &subject,
        input.observation,
        &mut Meter::new(scalar_limits()),
        &mut native_budget(),
    )
    .expect("explicit native fixture evaluation")
}

/// Both evaluator invocations remain outside the bridge under test.
pub(crate) fn evaluate_tl(
    projection: &ValidatedTlProjection<'_>,
    relation: ResultRelationInput<'_>,
    limits: OwnerLimits,
) -> ValidatedTemporalResult {
    let document = report::evaluate(projection.request(), relation, limits)
        .expect("explicit TL fixture evaluation");
    report::read(document.bytes(), projection.request(), relation, limits)
        .expect("actual TL source result reader")
}

pub(crate) fn map_source(
    source: &ValidatedTemporalResult,
    limits: OwnerLimits,
) -> ValidatedMappedResult {
    let selection = MappingSelection::for_result(source);
    let document = contract_ir::map(source, &selection, limits).expect("actual TL result mapper");
    contract_ir::read(document.bytes(), source, &selection, limits)
        .expect("actual non-evaluating TL mapped-result reader")
}

pub(crate) fn evaluate_mapped(projection: &ValidatedTlProjection<'_>) -> ValidatedMappedResult {
    let limits = projection.selection().owner_limits;
    map_source(
        &evaluate_tl(projection, ResultRelationInput::Original, limits),
        limits,
    )
}
