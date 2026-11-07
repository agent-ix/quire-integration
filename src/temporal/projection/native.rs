// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native correspondence checks over original immutable owner values.
//!
//! Steps run in the order FR-350 assigns and the first failure is returned.

use std::collections::BTreeMap;

use qsl_eval::temporal::{
    evidence::{
        ActivationInput, AuthenticatedBinding, CaptureFact, ClauseSelection, Evicted, FaultKind,
        ObligationKey, TriggerEvidence, TriggerRecord, TriggerScope,
    },
    ClockCover, ProgressClosure, ProgressCompleteness,
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
    Budget, NativeProjectionInput, TlMissingPremise as Missing, TlProjectionError as Error,
    TlProjectionLimit, TlProjectionPremise as Premise, TlProjectionSelection,
};

/// Values established by the binding steps and consumed by construction.
pub(super) struct Facts<'subject, 'input> {
    pub name: &'subject str,
    pub target: Target,
    pub binding: AuthenticatedBinding,
    /// Request index of each TL position, in ascending stamp order.
    pub positions: Vec<usize>,
    /// The numeric TL anchor: the anchor position's stamp minus the range start.
    pub anchor: u64,
    pub captures: Vec<&'input [u8]>,
    pub trigger_payload: &'input [u8],
    /// Agreed Boolean per (request index, operand).
    pub values: BTreeMap<(usize, &'subject CheckedNodeId), bool>,
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
    // Step 1: admission and support.
    super::super::validate_mapping_binding(package, subject, classification, surrounding)
        .map_err(Error::Binding)?;
    let target = match classification.support {
        Support::Supported { target, .. } => target,
        Support::Unsupported { .. } => {
            return Err(Error::Unsupported(Box::new(classification.clone())))
        }
    };
    let observations = selection.observations;
    let bundle = selection.observation.bundle();
    for bytes in [
        selection.bridge_definition.bytes(),
        bundle.bytes(),
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

    // Step 2: occurrence, control and subject name.
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
        !subject.requirements().contains(&control)
    }) {
        return Err(Error::NativePremise(Premise::Control));
    }
    let name = subject
        .name_of(native.occurrence)
        .ok_or(Error::NativePremise(Premise::SubjectName))?;

    // Step 3: binding and cover.
    let binding = native
        .request
        .binding
        .authenticate(&ClauseSelection {
            declaration: subject.declaration(),
            profile: subject.profile(),
            clock: subject.clock_of(native.occurrence),
        })
        .map_err(Error::NativeClock)?;
    if native.request.cover != ClockCover::Exact {
        return Err(Error::NativePremise(Premise::ClockCover));
    }

    // Step 4: correspondence reference.
    if native.request.correspondence.identity != selection.bridge_definition.identity()
        || native.request.correspondence.authority_identity
            != selection.bridge_definition.authority()
    {
        return Err(Error::NativePremise(Premise::CorrespondenceReference));
    }

    // Step 5: anchor, instance and captures.
    let opaque_anchor = native
        .request
        .anchor
        .as_deref()
        .ok_or(Error::Incomplete(Missing::Anchor))?;
    if opaque_anchor != native.instance.anchor || native.instance.capture_instance.is_empty() {
        return Err(Error::NativePremise(Premise::Anchor));
    }
    let (captures, trigger_payload) = check_captures(subject, native, budget)?;

    // Step 6: observation.
    if native.observation.reference != bundle.identity().as_str()
        || native.observation.revision != bundle.revision().to_string()
    {
        return Err(Error::NativePremise(Premise::ObservationRevision));
    }
    let through = match native
        .request
        .completeness
        .as_ref()
        .map(|record| record.state)
    {
        Some(ProgressCompleteness::Complete { through }) => Some(through),
        Some(ProgressCompleteness::Incomplete) | None => None,
    };
    budget.charge(bundle.payload().records().len(), None)?;
    let mut records = BTreeMap::new();
    for fact in bundle.payload().records() {
        if let EmbeddedPayloadRef::Observation(record) = fact.payload() {
            if records
                .insert(record.observation_identity(), record)
                .is_some()
            {
                return Err(Error::NativePremise(Premise::Observation));
            }
        }
    }
    budget.charge(
        native.request.positions.len(),
        Some(TlProjectionLimit::Positions),
    )?;
    let mut anchor_stamp = None;
    let mut anchor_matches = 0_usize;
    for position in &native.request.positions {
        let record = records
            .get(position.observation.identity.as_str())
            .ok_or(Error::Incomplete(Missing::Observation))?;
        if record.clock().identity != selection.clock.payload().clock_identity()
            || record.clock().revision != selection.clock.payload().clock_revision()
            || through.is_some_and(|through| position.position > through)
        {
            return Err(Error::NativePremise(Premise::Position));
        }
        check_owner_anchor(
            &binding,
            record.anchor(),
            position.position,
            target,
            selection,
        )?;
        if position.observation.identity == opaque_anchor {
            anchor_matches += 1;
            anchor_stamp = Some(position.position);
        }
    }
    let anchor_stamp = match (anchor_matches, anchor_stamp) {
        (0, _) | (_, None) => return Err(Error::Incomplete(Missing::Anchor)),
        (1, Some(stamp)) => stamp,
        _ => return Err(Error::NativePremise(Premise::Anchor)),
    };

    // Step 7: positions.
    let (start, end) = range(selection.clock)?;
    let mut ordered = BTreeMap::new();
    for (index, position) in native.request.positions.iter().enumerate() {
        if position.position < start
            || position.position >= end
            || ordered.insert(position.position, index).is_some()
        {
            return Err(Error::NativePremise(Premise::Position));
        }
    }
    let length = u64::try_from(native.request.positions.len()).unwrap_or(u64::MAX);
    if end.checked_sub(start) != Some(length) {
        return Err(Error::Incomplete(Missing::Position));
    }
    let mut positions = Vec::with_capacity(ordered.len());
    for (offset, (stamp, index)) in ordered.into_iter().enumerate() {
        if stamp.checked_sub(start) != u64::try_from(offset).ok() {
            return Err(Error::Incomplete(Missing::Position));
        }
        positions.push(index);
    }
    if target == Target::OriginCompleteHistory
        && (start != 0 || native.request.authoritative_origin != Some(true))
    {
        return Err(Error::Incomplete(Missing::History));
    }
    let anchor = anchor_stamp - start;

    // Step 8: progress, closure and completeness.
    let closed = |state| match state {
        ProgressClosure::Closed => OpenClosed::Closed,
        ProgressClosure::Open => OpenClosed::Open,
    };
    let decision = native
        .request
        .decision_closure
        .ok_or(Error::Incomplete(Missing::CompleteInput))?;
    if closed(decision) != observations.decision_scope_closure.payload().state() {
        return Err(Error::NativePremise(Premise::DecisionClosure));
    }
    let supplied_surrounding = match surrounding {
        Closure::Complete => ProgressClosure::Closed,
        Closure::Open => ProgressClosure::Open,
    };
    if native.request.surrounding_closure != Some(supplied_surrounding)
        || closed(supplied_surrounding)
            != observations.surrounding_execution_closure.payload().state()
    {
        return Err(Error::NativePremise(Premise::SurroundingClosure));
    }
    let decision_mark = watermark(observations.decision_scope_progress.payload().boundary())?;
    let surrounding_mark = watermark(
        observations
            .surrounding_execution_progress
            .payload()
            .boundary(),
    )?;
    if native.request.decision_progress != Some(decision_mark)
        || native.request.surrounding_progress != Some(surrounding_mark)
    {
        return Err(Error::NativePremise(Premise::Progress));
    }
    if through.is_none() {
        return Err(Error::Incomplete(Missing::CompleteInput));
    }
    match observations.completeness.payload().state() {
        completeness::State::Complete => {}
        completeness::State::Incomplete => return Err(Error::Incomplete(Missing::CompleteInput)),
        completeness::State::Contradicted => {
            return Err(Error::NativePremise(Premise::Completeness))
        }
    }

    // Step 9: valuations come only from the admitted rows.
    let rows = native.valuations.rows();
    budget.charge(rows.len(), Some(TlProjectionLimit::Valuations))?;
    for (index, row) in rows.iter().enumerate() {
        let cause = if row.subject != *native.occurrence {
            Some(ValuationCause::OccurrenceNotDeclared)
        } else if !subject.predicate_leaves().contains(&row.operand) {
            Some(ValuationCause::OperandNotInFormula)
        } else if row.position >= native.request.positions.len() {
            Some(ValuationCause::PositionNotAdmitted)
        } else if row.anchor != native.instance.anchor {
            Some(ValuationCause::AnchorMismatch)
        } else if row.capture_instance != native.instance.capture_instance {
            Some(ValuationCause::CaptureInstanceMismatch)
        } else if row.revision != native.observation.revision {
            Some(ValuationCause::RevisionMismatch)
        } else {
            None
        };
        if let Some(cause) = cause {
            return Err(Error::Valuation { row: index, cause });
        }
    }
    let leaves = subject.predicate_leaves();
    budget.charge(positions.len().saturating_mul(leaves.len()), None)?;
    let mut values = BTreeMap::new();
    for &position in &positions {
        for operand in leaves {
            let value = native
                .valuations
                .value_of(native.occurrence, position, operand)
                .map_err(|_| {
                    Error::Incomplete(Missing::Valuation {
                        position,
                        operand: operand.clone(),
                    })
                })?;
            values.insert((position, operand), value);
        }
    }
    Ok(Facts {
        name,
        target,
        captures,
        trigger_payload,
        binding,
        positions,
        anchor,
        values,
    })
}

/// Compare the owner anchor of one position with the authenticated clock form.
fn check_owner_anchor(
    binding: &AuthenticatedBinding,
    owner: AnchorRef<'_>,
    stamp: u64,
    target: Target,
    selection: TlProjectionSelection<'_>,
) -> Result<(), Error> {
    let clock_error = || Error::NativePremise(Premise::Clock);
    match (&binding.clock, owner) {
        (
            ClockConfiguration::EventPosition { sequence_authority },
            AnchorRef::EventPosition { position },
        ) => {
            if sequence_authority.as_ref() == selection.clock.payload().clock_identity()
                && position.parse::<u64>().ok() == Some(stamp)
            {
                Ok(())
            } else {
                Err(clock_error())
            }
        }
        (
            ClockConfiguration::FixedSample {
                epoch,
                period,
                unit,
            },
            AnchorRef::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            },
        ) => {
            if index.parse::<u64>().ok() != Some(stamp) {
                return Err(clock_error());
            }
            if unit.as_ref() != "nanoseconds" {
                // The unit is never converted. A future target cannot carry it;
                // a past target hands the unchanged clock to the TL request owner.
                return if target == Target::OriginCompleteHistory {
                    Ok(())
                } else {
                    Err(Error::NativePremise(Premise::ClockRepresentation))
                };
            }
            let epoch = integer(epoch)?;
            let period = integer(period)?;
            if epoch_nanos.parse::<i64>().ok() == Some(epoch)
                && period_nanos.parse::<i64>().ok() == Some(period)
            {
                Ok(())
            } else {
                Err(clock_error())
            }
        }
        _ => Err(clock_error()),
    }
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

/// An integer clock parameter; a rational with a denominator other than 1 has no
/// integer nanosecond form to compare.
fn integer(value: &ClockExactNumber) -> Result<i64, Error> {
    let unrepresentable = || Error::NativePremise(Premise::ClockRepresentation);
    match value {
        ClockExactNumber::Integer { decimal } => decimal.parse().map_err(|_| unrepresentable()),
        ClockExactNumber::Rational {
            numerator,
            denominator,
        } if denominator.as_ref() == "1" => numerator.parse().map_err(|_| unrepresentable()),
        ClockExactNumber::Rational { .. } => Err(unrepresentable()),
    }
}

type Selected<'a> = (Vec<&'a [u8]>, &'a [u8]);

/// Step 5 after the anchor premises: trigger, activation, captures, evictions.
fn check_captures<'a>(
    subject: &TemporalSubjectView<'_>,
    native: NativeProjectionInput<'a>,
    budget: &mut Budget,
) -> Result<Selected<'a>, Error> {
    if native.request.trigger_evidence != TriggerEvidence::Admitted {
        return Err(Error::Incomplete(Missing::Instance));
    }
    let selected = native
        .request
        .trigger
        .as_ref()
        .ok_or(Error::Incomplete(Missing::Instance))?;
    let occurrence = CheckedOccurrence {
        role: native.occurrence.role.clone(),
        ordinal: native.occurrence.ordinal,
    };
    let obligation = ObligationKey::new(
        native.occurrence.node.package.clone(),
        serde_json::to_vec(&(&native.occurrence.node, occurrence))
            .map_err(Error::SyntaxEncoding)?,
        native.occurrence.ordinal,
    );
    let ledger = native.request.ledger.as_ref();
    let mut visits = ledger.map_or(0, |ledger| ledger.instance_count() + ledger.result_count());
    visits += native.request.deliveries.len()
        + native
            .request
            .deliveries
            .iter()
            .map(|delivery| delivery.captures.len())
            .sum::<usize>();
    visits += native.request.origins.len()
        + native
            .request
            .origins
            .iter()
            .map(|origin| origin.captures.len())
            .sum::<usize>();
    visits += native.request.evicted.len();
    budget.charge(visits, None)?;
    let activation =
        ActivationInput {
            scope: TriggerScope {
                closure: native.request.trigger_scope,
                evidence: native.request.trigger_evidence,
                record: if native.request.completeness.as_ref().is_some_and(|record| {
                    matches!(record.state, ProgressCompleteness::Complete { .. })
                }) {
                    TriggerRecord::Complete
                } else {
                    TriggerRecord::Incomplete
                },
            },
            deliveries: native.request.deliveries.clone(),
            origins: native.request.origins.clone(),
            anchor: native.instance.anchor.to_owned(),
            evicted: native.request.evicted.clone(),
            ledger: native.request.ledger.clone(),
            execution: native.request.execution,
        };
    if let Some(contradiction) = activation.contradiction(&obligation) {
        return Err(Error::NativeActivation(contradiction));
    }
    let retained = ledger.and_then(|ledger| ledger.instance(&obligation, selected));
    let (values, payload) = if let Some(retained) = retained {
        if let Some(fault) = retained.fault() {
            return Err(match fault.kind {
                FaultKind::Missing => Error::Incomplete(Missing::Activation),
                FaultKind::Refused => Error::NativePremise(Premise::Activation),
            });
        }
        let captures = retained.environment().entries();
        budget.charge(captures.len(), None)?;
        if captures.len() != subject.capture_names().len() {
            return Err(Error::Incomplete(Missing::Capture));
        }
        let mut values = Vec::with_capacity(captures.len());
        for (capture, name) in captures.iter().zip(subject.capture_names()) {
            if capture.name() != name || capture.anchor() != native.instance.anchor {
                return Err(Error::NativePremise(Premise::Capture));
            }
            values.push(capture.value().as_bytes());
        }
        // Redelivery never re-runs initializers or replaces the owner environment.
        (values, retained.payload().as_bytes())
    } else {
        let mut captures = None;
        let mut payload = None;
        match subject.activation() {
            Activation::Origin => {
                if let Some(origin) = native
                    .request
                    .origins
                    .iter()
                    .find(|origin| &origin.identity == selected)
                {
                    payload = Some(&origin.payload);
                    captures = Some(origin.captures.as_slice());
                }
            }
            Activation::Each { guard, .. } => {
                if let Some(delivery) = native.request.deliveries.iter().find(|delivery| {
                    &delivery.identity == selected
                        && (guard.is_none() || delivery.guard == Some(true))
                }) {
                    if delivery.anchor != native.instance.anchor {
                        return Err(Error::NativePremise(Premise::Anchor));
                    }
                    payload = Some(&delivery.payload);
                    captures = Some(delivery.captures.as_slice());
                }
            }
        }
        let captures = captures.ok_or(Error::Incomplete(Missing::Instance))?;
        if captures.len() != subject.capture_names().len() {
            return Err(Error::Incomplete(Missing::Capture));
        }
        let mut values = Vec::with_capacity(captures.len());
        for capture in captures {
            match capture {
                CaptureFact::Value { anchor, value } if anchor == native.instance.anchor => {
                    values.push(value.as_bytes())
                }
                CaptureFact::Value { .. } | CaptureFact::WrongType => {
                    return Err(Error::NativePremise(Premise::Capture))
                }
                CaptureFact::Missing | CaptureFact::Null | CaptureFact::Stale => {
                    return Err(Error::Incomplete(Missing::Capture))
                }
            }
        }
        (
            values,
            payload
                .ok_or(Error::Incomplete(Missing::Instance))?
                .as_bytes(),
        )
    };
    for evicted in &native.request.evicted {
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
    Ok((values, payload))
}

fn watermark(boundary: BoundaryRef<'_>) -> Result<u64, Error> {
    match boundary {
        BoundaryRef::EventPosition { watermark, .. }
        | BoundaryRef::FixedSample { watermark, .. } => watermark
            .parse()
            .map_err(|_| Error::NativePremise(Premise::Progress)),
        BoundaryRef::TimestampedEvent { .. } => Err(Error::NativePremise(Premise::Progress)),
    }
}
