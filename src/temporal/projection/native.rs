// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native correspondence checks over original immutable owner values.

use std::collections::BTreeMap;

use qsl_eval::temporal::{
    evidence::{AuthenticatedBinding, CaptureFact, ClauseSelection, Evicted, TriggerEvidence},
    ledger::{ProgressClosure, ProgressCompleteness},
};
use qsl_package::checked_views::{
    Activation, Classification, Closure, Support, Target, TemporalSubjectView,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageV2, ClockConfiguration, ClockExactNumber,
};
use quire_observation::authority::{
    bundle::EmbeddedPayloadRef, clock, completeness, observation::AnchorRef, BoundaryRef,
    OpenClosed,
};
use quire_spec_language::temporal::ValuationCause;

use super::{
    limit, Budget, NativeProjectionInput, TlMissingPremise as Missing, TlProjectionError as Error,
    TlProjectionLimit, TlProjectionPremise as Premise, TlProjectionSelection,
};

pub(super) struct Facts<'subject, 'input> {
    pub name: &'subject str,
    pub binding: AuthenticatedBinding,
    pub positions: Vec<usize>,
    pub anchor: u64,
    pub captures: Vec<&'input [u8]>,
    pub values: BTreeMap<(usize, &'input CheckedNodeId), bool>,
}

#[allow(clippy::too_many_arguments)] // All inputs are independent original owner values at this boundary.
pub(super) fn check<'subject, 'input>(
    package: &CheckedPackageV2,
    subject: &'subject TemporalSubjectView<'_>,
    classification: &Classification,
    surrounding: Closure,
    native: NativeProjectionInput<'input>,
    selection: TlProjectionSelection<'_>,
    budget: &mut Budget,
) -> Result<Facts<'subject, 'input>, Error> {
    super::super::validate_mapping_binding(package, subject, classification, surrounding)
        .map_err(Error::Binding)?;
    let observations = selection.observations;
    for bytes in [
        selection.observation.bundle().bytes(),
        selection.clock.bytes(),
        observations.decision_scope_progress.bytes(),
        observations.decision_scope_closure.bytes(),
        observations.surrounding_execution_progress.bytes(),
        observations.surrounding_execution_closure.bytes(),
        observations.completeness.bytes(),
        observations.availability.bytes(),
    ] {
        budget.bytes(bytes.len())?;
    }
    let target = match classification.support {
        Support::Supported { target, .. } => target,
        Support::Unsupported { .. } => {
            return Err(Error::Unsupported(Box::new(classification.clone())))
        }
    };
    subject
        .arena_of(native.occurrence)
        .map_err(|_| Error::NativePremise(Premise::Occurrence))?;
    let selected = CheckedOccurrence {
        role: native.occurrence.role.clone(),
        ordinal: native.occurrence.ordinal,
    };
    if native
        .request
        .occurrence
        .as_ref()
        .is_some_and(|actual| actual != &selected)
        || (native.request.occurrence.is_none() && subject.occurrences().len() != 1)
    {
        return Err(Error::NativePremise(Premise::Occurrence));
    }
    if native.request.control.as_ref().is_some_and(|control| {
        !subject
            .requirements()
            .iter()
            .any(|required| *required == control)
    }) {
        return Err(Error::NativePremise(Premise::Occurrence));
    }
    let name = subject
        .name_of(native.occurrence)
        .ok_or(Error::NativePremise(Premise::SubjectName))?;
    let binding = native
        .request
        .binding
        .authenticate(&ClauseSelection {
            declaration: subject.declaration(),
            profile: subject.profile(),
            clock: subject.clock_of(native.occurrence),
        })
        .map_err(Error::NativeClock)?;
    let anchor = native
        .request
        .anchor
        .as_deref()
        .ok_or(Error::Incomplete(Missing::Anchor))?;
    if anchor != native.instance.anchor || native.instance.capture_instance.is_empty() {
        return Err(Error::NativePremise(Premise::Anchor));
    }
    let bundle = selection.observation.bundle();
    budget.bytes(bundle.bytes().len())?;
    if native.observation.reference != bundle.identity().as_str()
        || native.observation.revision != bundle.revision().to_string()
    {
        return Err(Error::NativePremise(Premise::ObservationRevision));
    }
    let closed = |state| match state {
        ProgressClosure::Closed => OpenClosed::Closed,
        ProgressClosure::Open => OpenClosed::Open,
    };
    let decision = native
        .request
        .decision_closure
        .ok_or(Error::Incomplete(Missing::CompleteInput))?;
    if closed(decision)
        != selection
            .observations
            .decision_scope_closure
            .payload()
            .state()
    {
        return Err(Error::NativePremise(Premise::DecisionClosure));
    }
    let expected_surrounding = match surrounding {
        Closure::Complete => ProgressClosure::Closed,
        Closure::Open => ProgressClosure::Open,
    };
    if native.request.surrounding_closure != Some(expected_surrounding)
        || closed(expected_surrounding)
            != selection
                .observations
                .surrounding_execution_closure
                .payload()
                .state()
    {
        return Err(Error::NativePremise(Premise::SurroundingClosure));
    }
    if native.request.decision_progress
        != Some(watermark(
            selection
                .observations
                .decision_scope_progress
                .payload()
                .boundary(),
        )?)
        || native.request.surrounding_progress
            != Some(watermark(
                selection
                    .observations
                    .surrounding_execution_progress
                    .payload()
                    .boundary(),
            )?)
    {
        return Err(Error::NativePremise(Premise::Progress));
    }
    let through = match native
        .request
        .completeness
        .as_ref()
        .map(|record| record.state)
    {
        Some(ProgressCompleteness::Complete { through }) => through,
        Some(ProgressCompleteness::Incomplete) | None => {
            return Err(Error::Incomplete(Missing::CompleteInput))
        }
    };
    match selection.observations.completeness.payload().state() {
        completeness::State::Complete => {}
        completeness::State::Incomplete => return Err(Error::Incomplete(Missing::CompleteInput)),
        completeness::State::Contradicted => {
            return Err(Error::NativePremise(Premise::Completeness))
        }
    }
    let captures = check_captures(subject, native, budget)?;
    let (start, end) = range(selection.clock)?;
    if target == Target::OriginCompleteHistory
        && (start != 0 || native.request.authoritative_origin != Some(true))
    {
        return Err(Error::Incomplete(Missing::History));
    }
    let length = u64::try_from(native.request.positions.len()).unwrap_or(u64::MAX);
    if length > budget.limits.max_positions {
        return Err(limit(
            TlProjectionLimit::Positions,
            budget.limits.max_positions,
        ));
    }
    if end.checked_sub(start) != Some(length) {
        return Err(Error::Incomplete(Missing::Position));
    }
    let mut records = BTreeMap::new();
    for fact in bundle.payload().records() {
        budget.work()?;
        if let EmbeddedPayloadRef::Observation(record) = fact.payload() {
            if records
                .insert(record.observation_identity(), record)
                .is_some()
            {
                return Err(Error::NativePremise(Premise::ObservationRevision));
            }
        }
    }
    let mut ordered = BTreeMap::new();
    let mut selected_anchor = None;
    for (index, position) in native.request.positions.iter().enumerate() {
        budget.position()?;
        let record = records
            .get(position.observation.identity.as_str())
            .ok_or(Error::Incomplete(Missing::Observation))?;
        if record.clock().identity != selection.clock.payload().clock_identity()
            || record.clock().revision != selection.clock.payload().clock_revision()
            || position.position > through
        {
            return Err(Error::NativePremise(Premise::Position));
        }
        match (&binding.clock, record.anchor()) {
            (
                ClockConfiguration::EventPosition { sequence_authority },
                AnchorRef::EventPosition {
                    position: owner_position,
                },
            ) if sequence_authority.as_ref() == selection.clock.payload().clock_identity()
                && owner_position.parse::<u64>().ok() == Some(position.position) => {}
            (
                ClockConfiguration::FixedSample {
                    epoch,
                    period,
                    unit,
                },
                AnchorRef::FixedSample {
                    index: owner_position,
                    epoch_nanos,
                    period_nanos,
                },
            ) if owner_position.parse::<u64>().ok() == Some(position.position)
                && (target == Target::OriginCompleteHistory || unit.as_ref() == "nanoseconds")
                && integer(epoch)
                    .is_some_and(|value| epoch_nanos.parse::<i64>().ok() == Some(value))
                && integer(period)
                    .is_some_and(|value| period_nanos.parse::<i64>().ok() == Some(value)) => {}
            _ => return Err(Error::NativePremise(Premise::Clock)),
        }
        if position.position < start
            || position.position >= end
            || ordered.insert(position.position, index).is_some()
        {
            return Err(Error::NativePremise(Premise::Position));
        }
        if position.observation.identity == anchor {
            if selected_anchor.replace(position.position - start).is_some() {
                return Err(Error::NativePremise(Premise::Anchor));
            }
        }
    }
    let mut positions = Vec::with_capacity(native.request.positions.len());
    for (offset, (coordinate, index)) in ordered.into_iter().enumerate() {
        budget.work()?;
        if coordinate.checked_sub(start) != u64::try_from(offset).ok() {
            return Err(Error::Incomplete(Missing::Position));
        }
        positions.push(index);
    }
    let anchor = selected_anchor.ok_or(Error::Incomplete(Missing::Anchor))?;
    let row_count = u64::try_from(native.valuations.rows().len()).unwrap_or(u64::MAX);
    if row_count > budget.limits.max_valuations {
        return Err(limit(
            TlProjectionLimit::Valuations,
            budget.limits.max_valuations,
        ));
    }
    let mut cells = BTreeMap::new();
    for (index, row) in native.valuations.rows().iter().enumerate() {
        budget.work()?;
        let cause = if row.subject != *native.occurrence {
            Some(ValuationCause::OccurrenceNotDeclared)
        } else if row.position >= native.request.positions.len() {
            Some(ValuationCause::PositionNotAdmitted)
        } else if row.anchor != native.instance.anchor {
            Some(ValuationCause::AnchorMismatch)
        } else if row.capture_instance != native.instance.capture_instance {
            Some(ValuationCause::CaptureInstanceMismatch)
        } else if row.revision != native.observation.revision {
            Some(ValuationCause::RevisionMismatch)
        } else if !subject.predicate_leaves().contains(&row.operand) {
            Some(ValuationCause::OperandNotInFormula)
        } else {
            None
        };
        if let Some(cause) = cause {
            return Err(Error::Valuation { row: index, cause });
        }
        if let Some((first, _)) = cells.insert((row.position, &row.operand), (index, row.value)) {
            return Err(Error::DuplicateValuation {
                first,
                duplicate: index,
            });
        }
    }
    let values: BTreeMap<_, _> = cells
        .into_iter()
        .map(|(key, (_, value))| (key, value))
        .collect();
    for &position in &positions {
        for operand in subject.predicate_leaves() {
            budget.cell()?;
            if !values.contains_key(&(position, operand)) {
                return Err(Error::MissingValuation {
                    position,
                    operand: operand.clone(),
                });
            }
        }
    }
    Ok(Facts {
        name,
        captures,
        binding,
        positions,
        anchor,
        values,
    })
}

fn range(selection: &clock::View) -> Result<(u64, u64), Error> {
    match selection.payload().selection() {
        clock::RangeRef::EventPosition {
            start,
            end_exclusive,
        }
        | clock::RangeRef::FixedSample {
            start,
            end_exclusive,
            ..
        } => {
            let start = start
                .parse()
                .map_err(|_| Error::NativePremise(Premise::ClockRepresentation))?;
            let end = end_exclusive
                .parse()
                .map_err(|_| Error::NativePremise(Premise::ClockRepresentation))?;
            Ok((start, end))
        }
        clock::RangeRef::TimestampedEvent { .. } => Err(Error::NativePremise(Premise::Clock)),
    }
}

fn integer(value: &ClockExactNumber) -> Option<i64> {
    match value {
        ClockExactNumber::Integer { decimal } => decimal.parse().ok(),
        ClockExactNumber::Rational {
            numerator,
            denominator,
        } if denominator.as_ref() == "1" => numerator.parse().ok(),
        ClockExactNumber::Rational { .. } => None,
    }
}

fn check_captures<'a>(
    subject: &TemporalSubjectView<'_>,
    native: NativeProjectionInput<'a>,
    budget: &mut Budget,
) -> Result<Vec<&'a [u8]>, Error> {
    if native.request.trigger_evidence != TriggerEvidence::Admitted {
        return Err(Error::Incomplete(Missing::Instance));
    }
    let selected = native
        .request
        .trigger
        .as_ref()
        .ok_or(Error::Incomplete(Missing::Instance))?;
    let mut captures = None;
    match subject.activation() {
        Activation::Origin => {
            let mut payload = None;
            for origin in &native.request.origins {
                budget.work()?;
                if &origin.identity != selected {
                    continue;
                }
                if payload.is_some_and(|prior| prior != &origin.payload)
                    || captures.is_some_and(|prior| prior != origin.captures.as_slice())
                {
                    return Err(Error::NativePremise(Premise::CaptureConflict));
                }
                payload = Some(&origin.payload);
                captures = Some(origin.captures.as_slice());
            }
        }
        Activation::Each { guard, .. } => {
            let mut payload = None;
            for delivery in &native.request.deliveries {
                budget.work()?;
                if &delivery.identity != selected {
                    continue;
                }
                if delivery.anchor != native.instance.anchor {
                    return Err(Error::NativePremise(Premise::Anchor));
                }
                if guard.is_some() && delivery.guard != Some(true) {
                    return Err(Error::Incomplete(Missing::Instance));
                }
                if payload.is_some_and(|prior| prior != &delivery.payload)
                    || captures.is_some_and(|prior| prior != delivery.captures.as_slice())
                {
                    return Err(Error::NativePremise(Premise::CaptureConflict));
                }
                payload = Some(&delivery.payload);
                captures = Some(delivery.captures.as_slice());
            }
        }
    }
    let captures = captures.ok_or(Error::Incomplete(Missing::Instance))?;
    if captures.len() != subject.captures().len() {
        return Err(Error::Incomplete(Missing::Capture));
    }
    for capture in captures {
        budget.work()?;
        match capture {
            CaptureFact::Value { anchor, .. } if anchor == native.instance.anchor => {}
            CaptureFact::Value { .. } | CaptureFact::WrongType => {
                return Err(Error::NativePremise(Premise::Capture))
            }
            CaptureFact::Missing | CaptureFact::Null | CaptureFact::Stale => {
                return Err(Error::Incomplete(Missing::Capture))
            }
        }
    }
    for evicted in &native.request.evicted {
        budget.work()?;
        match evicted {
            Evicted::Capture { instance, capture }
                if instance == selected && subject.capture_names().contains(capture) =>
            {
                return Err(Error::Incomplete(Missing::Capture))
            }
            Evicted::Valuation { instance, .. }
                if instance
                    .as_ref()
                    .is_none_or(|instance| instance == selected) =>
            {
                return Err(Error::Incomplete(Missing::CompleteInput))
            }
            Evicted::Capture { .. } | Evicted::Valuation { .. } => {}
        }
    }
    Ok(captures
        .iter()
        .filter_map(|capture| match capture {
            CaptureFact::Value { value, .. } => Some(value.as_bytes()),
            _ => None,
        })
        .collect())
}

fn watermark(boundary: BoundaryRef<'_>) -> Result<u64, Error> {
    match boundary {
        BoundaryRef::EventPosition { watermark, .. }
        | BoundaryRef::FixedSample { watermark, .. } => watermark
            .parse()
            .map_err(|_| Error::NativePremise(Premise::Progress)),
        BoundaryRef::TimestampedEvent { .. } => Err(Error::NativePremise(Premise::Clock)),
    }
}
