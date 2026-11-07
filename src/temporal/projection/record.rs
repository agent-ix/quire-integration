// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bridge-owned correspondence record; owner documents remain complete byte values.
//!
//! The record is RFC 8785 canonical JSON. Owner bytes are lowercase hexadecimal
//! strings and every integer is a decimal string, so no JSON number appears.

use qsl_eval::temporal::{
    evidence::{EvidenceRef, ExecutionDisposition},
    ProgressClosure, ProgressCompleteness,
};
use qsl_package::checked_views::Closure;
use quire_contract_ir::{CheckedOperationInterval, CheckedPackageV2, CheckedSourceMapEntry};
use serde::Serialize;
use serde_json::{json, Map, Value};

use super::{
    limit, native::Facts, Budget, NativeProjectionInput, TlInput, TlProjectionError as Error,
    TlProjectionLimit, TlProjectionSelection,
};
use crate::temporal::TemporalSubjectView;

/// Lowercase hexadecimal spelling of exact owner bytes.
pub(crate) fn hex(bytes: &[u8]) -> Value {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    Value::String(text)
}

fn dec(value: impl std::fmt::Display) -> Value {
    Value::String(value.to_string())
}

fn optional(value: Option<impl std::fmt::Display>) -> Value {
    value.map_or(Value::Null, dec)
}

/// The serde JSON value of an owner type, every number replaced by its decimal string.
fn ir(value: &impl Serialize) -> Result<Value, Error> {
    fn walk(value: Value) -> Result<Value, Error> {
        Ok(match value {
            Value::Number(number) if number.is_f64() => return Err(Error::NonIntegerNumber),
            Value::Number(number) => Value::String(number.to_string()),
            Value::Array(items) => {
                Value::Array(items.into_iter().map(walk).collect::<Result<_, _>>()?)
            }
            Value::Object(members) => Value::Object(
                members
                    .into_iter()
                    .map(|(name, member)| Ok((name, walk(member)?)))
                    .collect::<Result<Map<_, _>, Error>>()?,
            ),
            other => other,
        })
    }
    walk(serde_json::to_value(value).map_err(Error::SyntaxEncoding)?)
}

fn evidence_ref(reference: &EvidenceRef) -> Value {
    json!({
        "identity": reference.identity,
        "authorityIdentity": reference.authority_identity,
        "authorityRevision": reference.authority_revision,
        "scope": reference.scope,
        "population": reference.population,
        "contract": reference.contract,
        "schemaDigest": hex(&reference.schema_digest),
        "digest": hex(&reference.digest),
        "authorityDigest": hex(&reference.authority_digest),
    })
}

fn closure(state: ProgressClosure) -> bool {
    state == ProgressClosure::Closed
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    surrounding: Closure,
    native: NativeProjectionInput<'_>,
    selection: TlProjectionSelection<'_>,
    facts: &Facts<'_, '_>,
    source: &CheckedSourceMapEntry,
    built: &super::owners::Built,
    request: &quire_mltl::request::ValidatedTemporalRequest,
    budget: &mut Budget,
) -> Result<Vec<u8>, Error> {
    let models: Vec<&str> = package.selected_model_packages().collect();
    budget.charge(models.len(), None)?;
    let bindings = built
        .formula
        .leaves()
        .iter()
        .map(|leaf| {
            Ok(json!({
                "predicate": ir(&leaf.predicate)?,
                "term": ir(&package.term(&leaf.predicate))?,
                "proposition": dec(leaf.proposition.0),
            }))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let intervals = subject
        .formula_graph()
        .iter()
        .map(|node| {
            let (lower, upper) = match &node.interval {
                Some(CheckedOperationInterval::Integer { lower, upper }) => {
                    (dec(lower), upper.as_ref().map_or(Value::Null, dec))
                }
                Some(CheckedOperationInterval::Timed { .. }) => {
                    return Err(Error::NativePremise(super::TlProjectionPremise::Interval))
                }
                None => (Value::Null, Value::Null),
            };
            Ok(json!({
                "node": ir(&node.node)?,
                "operation": ir(&node.operation)?,
                "lower": lower,
                "upper": upper,
            }))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let positions = facts
        .positions
        .iter()
        .map(|&index| {
            let position = &native.request.positions[index];
            json!({
                "index": dec(index),
                "stamp": dec(position.position),
                "observation": evidence_ref(&position.observation),
                "orderKey": position.order_key.as_ref().map_or(Value::Null, |key| {
                    json!({"authority": key.authority, "key": dec(key.key)})
                }),
            })
        })
        .collect::<Vec<_>>();
    let valuations = native
        .valuations
        .rows()
        .iter()
        .map(|row| {
            Ok(json!({
                "node": ir(&row.subject.node)?,
                "role": ir(&row.subject.role)?,
                "ordinal": dec(row.subject.ordinal),
                "position": dec(row.position),
                "anchor": row.anchor,
                "captureInstance": row.capture_instance,
                "revision": row.revision,
                "operand": ir(&row.operand)?,
                "value": row.value,
            }))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let completeness = native
        .request
        .completeness
        .as_ref()
        .ok_or(Error::Incomplete(super::TlMissingPremise::CompleteInput))?;
    let through = match completeness.state {
        ProgressCompleteness::Complete { through } => Some(through),
        ProgressCompleteness::Incomplete => None,
    };
    let observations = selection.observations;
    let (kind, input) = match &built.input {
        TlInput::Future(trace) => ("future", trace.bytes()),
        TlInput::Past(history) => ("past", history.bytes()),
    };
    let record = json!({
        "profile": super::PROFILE,
        "bridgeDefinition": hex(selection.bridge_definition.bytes()),
        "package": package.name(),
        "lock": ir(package.lock())?,
        "source": ir(source)?,
        "declaration": {
            "node": ir(&native.occurrence.node)?,
            "role": ir(&native.occurrence.role)?,
            "ordinal": dec(native.occurrence.ordinal),
            "name": facts.name,
        },
        "subjectProfile": ir(subject.profile())?,
        "models": models,
        "bindings": bindings,
        "intervals": intervals,
        "anchor": native.instance.anchor,
        "captureInstance": native.instance.capture_instance,
        "captures": facts.captures.iter().map(|bytes| hex(bytes)).collect::<Vec<_>>(),
        "trigger": hex(native.request.trigger.as_ref().map_or(&[][..], |value| value.as_bytes())),
        "triggerPayload": hex(facts.trigger_payload),
        "clock": ir(&facts.binding.clock)?,
        "observation": hex(selection.observation.bundle().bytes()),
        "clockView": hex(selection.clock.bytes()),
        "decisionProgress": hex(observations.decision_scope_progress.bytes()),
        "decisionClosure": hex(observations.decision_scope_closure.bytes()),
        "surroundingProgress": hex(observations.surrounding_execution_progress.bytes()),
        "surroundingClosure": hex(observations.surrounding_execution_closure.bytes()),
        "completenessView": hex(observations.completeness.bytes()),
        "availabilityView": hex(observations.availability.bytes()),
        "surrounding": if surrounding == Closure::Complete { "complete" } else { "open" },
        "authoritativeOrigin": native.request.authoritative_origin,
        "positions": positions,
        "formula": hex(built.formula.bytes()),
        "propositionMap": hex(&built.propositions.canonical_json_bytes().map_err(Error::SyntaxEncoding)?),
        "request": hex(request.bytes()),
        "input": {"kind": kind, "bytes": hex(input)},
        "valuations": valuations,
        "native": {
            "correspondence": evidence_ref(&native.request.correspondence),
            "triggerScope": if closure(native.request.trigger_scope) { "closed" } else { "open" },
            "decisionProgress": optional(native.request.decision_progress),
            "surroundingProgress": optional(native.request.surrounding_progress),
            "decisionClosed": native.request.decision_closure.map(closure),
            "surroundingClosed": native.request.surrounding_closure.map(closure),
            "execution": match native.request.execution {
                ExecutionDisposition::Completed => "completed",
                ExecutionDisposition::Failed => "failed",
            },
            "completeThrough": optional(through),
            "completenessFacts": completeness.facts,
            "completenessReference": evidence_ref(&completeness.reference),
        },
    });
    let bytes = quire_canonical::to_vec(
        &record,
        quire_canonical::Limits::new(budget.limits.max_bytes),
    )
    .map_err(|error| match error {
        quire_canonical::Error::Limit(_) => {
            limit(TlProjectionLimit::Bytes, budget.limits.max_bytes)
        }
        error => Error::Canonical(error),
    })?;
    budget.bytes(bytes.len())?;
    Ok(bytes)
}
