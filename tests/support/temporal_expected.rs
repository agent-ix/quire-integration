// SPDX-License-Identifier: AGPL-3.0-or-later
//! The expected correspondence record, built independently from the owner inputs
//! by FR-350's member table and encodings. It reads no bridge output except the
//! owner-admitted values the projection retains (formula, map, trace, request).

#![allow(dead_code)] // Each target uses a different subset of these helpers.

use qsl_eval::temporal::{
    evidence::{EvidenceRef, ExecutionDisposition},
    ProgressClosure, ProgressCompleteness,
};
use qsl_package::checked_views::Closure;
use quire_integration::temporal::{TlInput, ValidatedTlProjection};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::projection_support::Case;

pub(crate) fn hex(bytes: &[u8]) -> Value {
    Value::String(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn dec(value: impl std::fmt::Display) -> Value {
    Value::String(value.to_string())
}

fn opt(value: Option<impl std::fmt::Display>) -> Value {
    value.map_or(Value::Null, dec)
}

/// An owner type's serde JSON value with every number replaced by its decimal string.
pub(crate) fn ir(value: &impl Serialize) -> Value {
    fn walk(value: Value) -> Value {
        match value {
            Value::Number(number) => {
                assert!(!number.is_f64(), "no non-integer number in an IR value");
                Value::String(number.to_string())
            }
            Value::Array(items) => Value::Array(items.into_iter().map(walk).collect()),
            Value::Object(members) => {
                Value::Object(members.into_iter().map(|(k, v)| (k, walk(v))).collect())
            }
            other => other,
        }
    }
    walk(serde_json::to_value(value).expect("owner JSON"))
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

fn closed(state: ProgressClosure) -> bool {
    state == ProgressClosure::Closed
}

/// The record as an object, member by member from the case's inputs.
pub(crate) fn expected_members(
    case: &Case<'_>,
    projection: &ValidatedTlProjection<'_>,
    captures: &[&[u8]],
    trigger_payload: &[u8],
    source_entry: &impl Serialize,
) -> Map<String, Value> {
    let request = case.native.request;
    let views = case.selection.observations;
    let completeness = request.completeness.as_ref().expect("complete request");
    let through = match completeness.state {
        ProgressCompleteness::Complete { through } => Some(through),
        ProgressCompleteness::Incomplete => None,
    };
    let (kind, input) = match projection.input() {
        TlInput::Future(trace) => ("future", trace.bytes()),
        TlInput::Past(history) => ("past", history.bytes()),
    };
    let occurrence = case.native.occurrence;
    // TL order is ascending stamp order; each entry keeps its request index.
    let mut order: Vec<usize> = (0..request.positions.len()).collect();
    order.sort_by_key(|&index| request.positions[index].position);
    let positions: Vec<Value> = order
        .into_iter()
        .map(|index| {
            let position = &request.positions[index];
            json!({
                "index": dec(index),
                "stamp": dec(position.position),
                "observation": evidence_ref(&position.observation),
                "orderKey": position.order_key.as_ref().map_or(Value::Null, |key| {
                    json!({"authority": key.authority, "key": dec(key.key)})
                }),
            })
        })
        .collect();
    let operands = qsl_eval::temporal::evidence::native_atom_operands(case.subject);
    let bound = qsl_eval::temporal::evidence::bound_atom_indices(
        case.subject.formula_graph(),
        &qsl_package::checked_views::instance_bound_atoms(case.subject).unwrap(),
    )
    .unwrap();
    let mut cells = std::collections::BTreeMap::new();
    for (position, supplied) in request.positions.iter().enumerate() {
        for (index, operand) in operands.iter().enumerate() {
            let Some(operand) = operand.as_ref() else {
                continue;
            };
            let atom = u64::try_from(index).unwrap();
            let value = if bound.contains(&atom) {
                supplied
                    .instance_valuations
                    .get(request.trigger.as_ref().unwrap())
                    .unwrap()
                    .get(&atom)
            } else {
                supplied.valuations.get(&atom)
            };
            let value = match value {
                Some(qsl_eval::temporal::Valuation::True) => true,
                Some(qsl_eval::temporal::Valuation::False) => false,
                other => panic!("expected admitted Boolean cell: {other:?}"),
            };
            cells.insert((position, operand), value);
        }
    }
    let valuations = cells
        .iter()
        .map(|((position, operand), value)| {
            json!({
                "position": dec(position), "operand": ir(operand), "value": value,
            })
        })
        .collect::<Vec<_>>();
    let value = json!({
        "profile": "quire.native-temporal-correspondence/v1",
        "bridgeDefinition": hex(case.selection.bridge_definition.bytes()),
        "package": case.package.name(),
        "lock": ir(case.package.lock()),
        "source": ir(source_entry),
        "declaration": {
            "node": ir(&occurrence.node),
            "role": ir(&occurrence.role),
            "ordinal": dec(occurrence.ordinal),
            "name": case.subject.name_of(occurrence),
        },
        "subjectProfile": ir(case.subject.profile()),
        "models": case.package.selected_model_packages().collect::<Vec<_>>(),
        "bindings": projection.formula().leaves().iter().map(|leaf| json!({
            "predicate": ir(&leaf.predicate),
            "term": ir(&case.package.term(&leaf.predicate)),
            "proposition": dec(leaf.proposition.0),
        })).collect::<Vec<_>>(),
        "intervals": case.subject.formula_graph().iter().map(|node| {
            let (lower, upper) = match &node.interval {
                Some(quire_contract_ir::CheckedOperationInterval::Integer { lower, upper }) =>
                    (dec(lower), upper.as_ref().map_or(Value::Null, dec)),
                _ => (Value::Null, Value::Null),
            };
            json!({"node": ir(&node.node), "operation": ir(&node.operation), "lower": lower, "upper": upper})
        }).collect::<Vec<_>>(),
        "anchor": request.anchor,
        "captures": captures.iter().map(|bytes| hex(bytes)).collect::<Vec<_>>(),
        "trigger": hex(request.trigger.as_ref().expect("trigger").as_bytes()),
        "triggerPayload": hex(trigger_payload),
        "clock": ir(&projection.binding().clock),
        "observation": hex(case.selection.observation.bundle().bytes()),
        "clockView": hex(case.selection.clock.bytes()),
        "decisionProgress": hex(views.decision_scope_progress.bytes()),
        "decisionClosure": hex(views.decision_scope_closure.bytes()),
        "surroundingProgress": hex(views.surrounding_execution_progress.bytes()),
        "surroundingClosure": hex(views.surrounding_execution_closure.bytes()),
        "completenessView": hex(views.completeness.bytes()),
        "availabilityView": hex(views.availability.bytes()),
        "surrounding": if case.surrounding == Closure::Complete { "complete" } else { "open" },
        "authoritativeOrigin": request.authoritative_origin,
        "positions": positions,
        "formula": hex(projection.formula().bytes()),
        "propositionMap": hex(&projection.propositions().canonical_json_bytes().expect("map bytes")),
        "request": hex(projection.request().bytes()),
        "input": {"kind": kind, "bytes": hex(input)},
        "valuations": valuations,
        "native": {
            "correspondence": evidence_ref(&request.correspondence),
            "triggerScope": if closed(request.trigger_scope) { "closed" } else { "open" },
            "decisionProgress": opt(request.decision_progress),
            "surroundingProgress": opt(request.surrounding_progress),
            "decisionClosed": request.decision_closure.map(closed),
            "surroundingClosed": request.surrounding_closure.map(closed),
            "execution": match request.execution {
                ExecutionDisposition::Completed => "completed",
                ExecutionDisposition::Failed => "failed",
            },
            "completeThrough": opt(through),
            "completenessFacts": completeness.facts,
            "completenessReference": evidence_ref(&completeness.reference),
        },
    });
    let Value::Object(members) = value else {
        unreachable!("object literal")
    };
    members
}

/// RFC 8785 bytes of the expected members.
pub(crate) fn canonical(members: &Map<String, Value>) -> Vec<u8> {
    quire_canonical::to_vec(
        &Value::Object(members.clone()),
        quire_canonical::Limits::new(1 << 30),
    )
    .expect("canonical expected record")
}

/// Every member whose value differs between two records.
pub(crate) fn differing(left: &Map<String, Value>, right: &Map<String, Value>) -> Vec<String> {
    assert_eq!(left.len(), right.len(), "same member set");
    left.iter()
        .filter(|(name, value)| right.get(*name) != Some(*value))
        .map(|(name, _)| name.clone())
        .collect()
}
