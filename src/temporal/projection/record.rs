// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bridge-owned correspondence; owner documents remain complete byte values.

use serde::Serialize;

use super::{
    native::Facts, Budget, NativeProjectionInput, TlProjectionError, TlProjectionSelection,
};
use crate::temporal::{Closure, TemporalSubjectView};

#[derive(Serialize)]
struct Record<'a, S, D, L, B, I, V, A, M> {
    profile: &'static str,
    version: u32,
    bridge_definition: &'a [u8],
    package: &'a str,
    source: S,
    declaration: D,
    selections: L,
    bindings: B,
    models: M,
    intervals: I,
    anchor: &'a str,
    capture_instance: &'a str,
    captures: &'a [&'a [u8]],
    trigger: Option<&'a [u8]>,
    trigger_payload: &'a [u8],
    clock: &'a quire_contract_ir::ClockConfiguration,
    observation: &'a [u8],
    clock_record: &'a [u8],
    decision_progress: &'a [u8],
    decision_closure: &'a [u8],
    surrounding_progress: &'a [u8],
    surrounding_closure: &'a [u8],
    completeness: &'a [u8],
    availability: &'a [u8],
    surrounding_complete: bool,
    authoritative_origin: Option<bool>,
    source_positions: &'a [usize],
    formula: &'a [u8],
    proposition_map: &'a [u8],
    input: &'a [u8],
    request: &'a [u8],
    native_valuations: &'a [u8],
    valuations: V,
    native_axes: A,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode(
    package: &quire_contract_ir::CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    surrounding: Closure,
    native: NativeProjectionInput<'_>,
    selection: TlProjectionSelection<'_>,
    facts: &Facts<'_, '_>,
    built: &super::owners::Built,
    request: &quire_mltl::request::ValidatedTemporalRequest,
    budget: &mut Budget,
) -> Result<Vec<u8>, TlProjectionError> {
    let mut models = Vec::new();
    for (node, kind) in package.graph().nodes.iter().zip(package.node_kinds()) {
        budget.work()?;
        if *kind
            == quire_contract_ir::CheckedNodeKind::Model(quire_contract_ir::ModelForm::ModelRoot)
        {
            models.push(node);
        }
    }
    models.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    let mut bindings = Vec::with_capacity(built.formula.leaves().len());
    for leaf in built.formula.leaves() {
        budget.work()?;
        bindings.push((
            &leaf.predicate,
            package.term(&leaf.predicate),
            &leaf.proposition,
        ));
    }
    let mut intervals = Vec::with_capacity(subject.formula_graph().len());
    for node in subject.formula_graph() {
        budget.work()?;
        intervals.push((node.node, node.operation, &node.interval));
    }
    let propositions = built
        .propositions
        .canonical_json_bytes()
        .map_err(TlProjectionError::SyntaxEncoding)?;
    let mut valuations = Vec::with_capacity(native.valuations.rows().len());
    for row in native.valuations.rows() {
        budget.work()?;
        valuations.push((
            &row.subject.node,
            row.subject.role,
            row.subject.ordinal,
            row.position,
            &row.anchor,
            &row.capture_instance,
            &row.revision,
            &row.operand,
            row.value,
        ));
    }
    let completeness =
        native
            .request
            .completeness
            .as_ref()
            .ok_or(TlProjectionError::Incomplete(
                super::TlMissingPremise::CompleteInput,
            ))?;
    let reference = &completeness.reference;
    let evidence_ref = (
        &reference.identity,
        &reference.authority_identity,
        &reference.authority_revision,
        &reference.scope,
        &reference.population,
        &reference.contract,
        &reference.schema_digest,
        &reference.digest,
        &reference.authority_digest,
    );
    let execution = match native.request.execution {
        qsl_eval::temporal::evidence::ExecutionDisposition::Completed => "completed",
        qsl_eval::temporal::evidence::ExecutionDisposition::Failed => "failed",
    };
    let decision_closed = native
        .request
        .decision_closure
        .map(|value| value == qsl_eval::temporal::ledger::ProgressClosure::Closed);
    let surrounding_closed = native
        .request
        .surrounding_closure
        .map(|value| value == qsl_eval::temporal::ledger::ProgressClosure::Closed);
    let complete_through = match completeness.state {
        qsl_eval::temporal::ledger::ProgressCompleteness::Complete { through } => Some(through),
        qsl_eval::temporal::ledger::ProgressCompleteness::Incomplete => None,
    };
    let observations = selection.observations;
    let record = Record {
        profile: super::PROFILE,
        version: 1,
        bridge_definition: selection.bridge_definition.bytes(),
        package: package.name(),
        source: subject.source_span(),
        declaration: (
            &native.occurrence.node,
            native.occurrence.role,
            native.occurrence.ordinal,
            facts.name,
        ),
        selections: (package.lock(), subject.profile()),
        bindings,
        models,
        intervals,
        anchor: native.instance.anchor,
        capture_instance: native.instance.capture_instance,
        captures: &facts.captures,
        trigger: native
            .request
            .trigger
            .as_ref()
            .map(|value| value.as_bytes()),
        trigger_payload: facts.trigger_payload,
        clock: &facts.binding.clock,
        observation: selection.observation.bundle().bytes(),
        clock_record: selection.clock.bytes(),
        decision_progress: observations.decision_scope_progress.bytes(),
        decision_closure: observations.decision_scope_closure.bytes(),
        surrounding_progress: observations.surrounding_execution_progress.bytes(),
        surrounding_closure: observations.surrounding_execution_closure.bytes(),
        completeness: observations.completeness.bytes(),
        availability: observations.availability.bytes(),
        surrounding_complete: surrounding == Closure::Complete,
        authoritative_origin: native.request.authoritative_origin,
        source_positions: &facts.positions,
        formula: built.formula.bytes(),
        proposition_map: &propositions,
        input: built.input.bytes(),
        request: request.bytes(),
        native_valuations: &native.request.native_valuations,
        valuations,
        native_axes: (
            native.request.decision_progress,
            decision_closed,
            native.request.surrounding_progress,
            surrounding_closed,
            execution,
            evidence_ref,
            complete_through,
            &completeness.facts,
        ),
    };
    super::super::formula::check_serialized_size(&record, budget.limits.max_bytes)
        .map_err(super::owners::formula_error)?;
    let value = serde_json::to_value(record).map_err(TlProjectionError::SyntaxEncoding)?;
    let bytes = quire_canonical::to_vec(
        &value,
        quire_canonical::Limits::new(budget.limits.max_bytes),
    )
    .map_err(|error| match error {
        quire_canonical::Error::Limit(_) => {
            super::limit(super::TlProjectionLimit::Bytes, budget.limits.max_bytes)
        }
        error => TlProjectionError::Canonical(error),
    })?;
    budget.bytes(bytes.len())?;
    Ok(bytes)
}
