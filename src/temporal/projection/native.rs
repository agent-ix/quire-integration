// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native correspondence checks over original immutable owner values.
//!
//! Steps run in the order FR-350 assigns and the first failure is returned.

use std::collections::BTreeMap;

use qsl_eval::temporal::{
    evidence::{bound_atom_indices, native_atom_operands},
    Valuation,
};
use qsl_eval::temporal::{
    evidence::{
        ActivationInput, AuthenticatedBinding, CaptureFact, ClauseSelection, Evicted, FaultKind,
        ObligationKey, TriggerEvidence, TriggerRecord, TriggerScope,
    },
    ClockCover, ProgressClosure, ProgressCompleteness,
};
use qsl_package::checked_views::instance_bound_atoms;
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

use super::{
    positions, Budget, NativeProjectionInput, TlMissingPremise as Missing,
    TlProjectionError as Error, TlProjectionLimit, TlProjectionPremise as Premise,
    TlProjectionSelection,
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
    if native
        .request
        .control
        .as_ref()
        .is_some_and(|control| !subject.requirements().contains(&control))
    {
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
    let (captures, trigger_payload) = check_captures(subject, native, budget)?;

    // Step 6: observation.
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
    let records = positions::unique_identities(bundle.payload().records().filter_map(|fact| {
        match fact.payload() {
            EmbeddedPayloadRef::Observation(record) => {
                Some((record.observation_identity(), record))
            }
            _ => None,
        }
    }))?;
    budget.charge(
        native.request.positions.len(),
        Some(TlProjectionLimit::Positions),
    )?;
    for position in &native.request.positions {
        if position.observation.authority_identity != bundle.identity().as_str()
            || position.observation.authority_revision != bundle.revision().to_string()
        {
            return Err(Error::NativePremise(Premise::ObservationRevision));
        }
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
    }
    let identified: Vec<(&str, u64)> = native
        .request
        .positions
        .iter()
        .map(|position| (position.observation.identity.as_str(), position.position))
        .collect();
    let anchor_stamp = positions::anchor_stamp(&identified, opaque_anchor)?;

    // Step 7: positions.
    let (start, end) = range(selection.clock)?;
    let stamps: Vec<u64> = identified.iter().map(|(_, stamp)| *stamp).collect();
    let positions = positions::order_by_stamp(&stamps, start, end)?;
    if target == Target::OriginCompleteHistory
        && (start != 0 || native.request.authoritative_origin != Some(true))
    {
        return Err(Error::Incomplete(Missing::History));
    }
    let anchor = positions::tl_anchor(anchor_stamp, start);

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

    // Step 9: native atom indices and instance scope come from the checked package.
    let operands = native_atom_operands(subject);
    let reads = instance_bound_atoms(subject).map_err(|_| Error::NativePremise(Premise::Source))?;
    let bound = bound_atom_indices(subject.formula_graph(), &reads)
        .ok_or(Error::NativePremise(Premise::Source))?;
    for (position, supplied) in native.request.positions.iter().enumerate() {
        let count = supplied.valuations.len()
            + supplied
                .instance_valuations
                .values()
                .map(BTreeMap::len)
                .sum::<usize>();
        budget.charge(count, Some(TlProjectionLimit::Valuations))?;
        for atom in supplied.valuations.keys().chain(
            supplied
                .instance_valuations
                .values()
                .flat_map(|values| values.keys()),
        ) {
            if usize::try_from(*atom)
                .ok()
                .is_none_or(|index| index >= operands.len())
            {
                return Err(Error::Valuation {
                    position,
                    atom: *atom,
                });
            }
        }
    }
    budget.charge(positions.len().saturating_mul(operands.len()), None)?;
    let mut values = BTreeMap::new();
    for &position in &positions {
        let supplied = &native.request.positions[position];
        for (index, operand) in operands.iter().enumerate() {
            let Some(checked_operand) = operand.as_ref() else {
                continue;
            };
            let operand = subject
                .predicate_leaves()
                .iter()
                .find(|operand| *operand == checked_operand)
                .ok_or(Error::NativePremise(Premise::Source))?;
            let atom = u64::try_from(index).map_err(|_| Error::NativePremise(Premise::Source))?;
            let valuation = if bound.contains(&atom) {
                native
                    .request
                    .trigger
                    .as_ref()
                    .and_then(|trigger| supplied.instance_valuations.get(trigger))
                    .and_then(|entries| entries.get(&atom))
            } else {
                supplied.valuations.get(&atom)
            };
            let value = match valuation {
                Some(Valuation::True) => true,
                Some(Valuation::False) => false,
                Some(Valuation::Missing | Valuation::Undefined(_)) | None => {
                    return Err(Error::Incomplete(Missing::Valuation {
                        position,
                        operand: operand.clone(),
                    }));
                }
            };
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
            // A rational with a denominator other than 1 has no integer form under
            // any unit, so it is refused before the unit is considered.
            let epoch = integer(epoch)?;
            let period = integer(period)?;
            if unit.as_ref() != "nanoseconds" {
                // The unit is never converted. A future target cannot carry it;
                // a past target hands the unchanged clock to the TL request owner.
                return if target == Target::OriginCompleteHistory {
                    Ok(())
                } else {
                    Err(Error::NativePremise(Premise::ClockRepresentation))
                };
            }
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
    let anchor = native
        .request
        .anchor
        .as_deref()
        .ok_or(Error::Incomplete(Missing::Anchor))?;
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
            anchor: anchor.to_owned(),
            evicted: native.request.evicted.clone(),
            ledger: native.request.ledger.clone(),
            execution: native.request.execution,
        };
    if activation.contradiction(&obligation).is_some() {
        return Err(Error::NativePremise(Premise::Activation));
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
            if capture.name() != name || capture.anchor() != anchor {
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
                    if delivery.anchor != anchor {
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
                CaptureFact::Value {
                    anchor: capture_anchor,
                    value,
                } if capture_anchor == anchor => values.push(value.as_bytes()),
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
