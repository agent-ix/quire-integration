// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical expected-value encoding of the bridge-owned join envelope.

use quire_canonical::FixedShape;
use quire_mltl::contract_ir::ValidatedMappedResult;
use serde::Serialize;

use super::{Budget, TlJoinCause, TlJoinDecision, TlJoinError, ValidatedTlJoin, VERSION};
use crate::temporal::ValidatedTlProjection;

#[derive(Serialize, FixedShape)]
#[serde(rename_all = "camelCase")]
struct JoinWire<'a> {
    version: &'static str,
    correspondence: &'a [u8],
    native: Option<&'a [u8]>,
    native_admitted: bool,
    mapped: Option<&'a [u8]>,
    previous: Option<&'a [u8]>,
    decision: &'static str,
    comparison: &'static str,
    causes: Vec<&'static str>,
    value: Option<bool>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode(
    projection: &ValidatedTlProjection<'_>,
    native: Option<&[u8]>,
    native_admitted: bool,
    mapped: Option<&ValidatedMappedResult>,
    previous: Option<&ValidatedTlJoin<'_>>,
    decision: TlJoinDecision,
    causes: &[TlJoinCause],
    value: Option<bool>,
    budget: &mut Budget,
) -> Result<Vec<u8>, TlJoinError> {
    let wire = JoinWire {
        version: VERSION,
        correspondence: projection.bytes(),
        native,
        native_admitted,
        mapped: mapped.map(ValidatedMappedResult::bytes),
        previous: previous.map(ValidatedTlJoin::bytes),
        decision: decision.as_str(),
        comparison: decision.comparison(),
        causes: causes.iter().map(|cause| cause.as_str()).collect(),
        value,
    };
    let bytes =
        quire_canonical::to_vec(&wire, quire_canonical::Limits::new(budget.limits.max_bytes))
            .map_err(|error| match error {
                quire_canonical::Error::Limit(_) => super::limit(
                    crate::temporal::TlProjectionLimit::Bytes,
                    budget.limits.max_bytes,
                ),
                error => TlJoinError::Encoding(error),
            })?;
    budget.bytes(bytes.len())?;
    Ok(bytes)
}
