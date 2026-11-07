// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mechanical construction and strict admission through original TL owners.

use qsl_package::checked_views::{Classification, Closure, Target, TemporalSubjectView};
use quire_contract_ir::{CheckedPackageV2, ClockConfiguration, ClockExactNumber};
use tl_mltl::{
    fixed_sample_instant,
    past::history,
    wire::{
        self, OwnerLimits, OwnerReadError, OwnerReadErrorCode, TraceDocument, TraceSchemaVersion,
    },
    ClockBinding, ClockSample, ExactNumber, PositionHistoryDocument, PositionObservation,
};
use tl_syntax::{PropositionEntry, PropositionMapDocument, SyntaxArtifactLimits};

use super::super::{
    construct_tl_formula, formula::check_serialized_size, TlFormulaComponent, TlFormulaError,
    TlFormulaLimit, TlFormulaLimits,
};
use super::{
    native::Facts, Budget, TlInput, TlProjectionError as Error, TlProjectionLimit as Limit,
    TlProjectionPremise as Premise, TlProjectionSelection,
};

pub(super) struct Built {
    pub formula: TlFormulaComponent,
    pub propositions: PropositionMapDocument,
    pub input: TlInput,
}

pub(super) fn limits(original: OwnerLimits, max_bytes: u64) -> OwnerLimits {
    let cap = usize::try_from(max_bytes).unwrap_or(usize::MAX);
    OwnerLimits {
        max_input_bytes: original.max_input_bytes.min(cap),
        max_output_bytes: original.max_output_bytes.min(cap),
        ..original
    }
}

pub(super) fn error(error: OwnerReadError, original: OwnerLimits, max_bytes: u64) -> Error {
    let original = original.effective();
    let cap = match error.field() {
        "outputBytes" => original.max_output_bytes,
        "inputBytes" => original.max_input_bytes,
        _ => return Error::Owner(error),
    };
    if error.code() == OwnerReadErrorCode::ResourceIncomplete
        && u64::try_from(cap).is_ok_and(|cap| max_bytes <= cap)
    {
        return Error::LimitReached {
            limit: Limit::Bytes,
            value: max_bytes.saturating_add(1),
            setting: max_bytes,
            owner: Some(error),
        };
    }
    Error::Owner(error)
}

pub(super) fn formula_error(error: TlFormulaError) -> Error {
    match error {
        TlFormulaError::LimitReached {
            limit: counter,
            value,
            setting,
        } => Error::LimitReached {
            limit: match counter {
                TlFormulaLimit::Bytes => Limit::Bytes,
                TlFormulaLimit::Nodes => Limit::Nodes,
                TlFormulaLimit::Work => Limit::Work,
            },
            value,
            setting,
            owner: None,
        },
        other => Error::Formula(other),
    }
}

pub(super) fn construct(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    classification: &Classification,
    surrounding: Closure,
    selection: TlProjectionSelection<'_>,
    facts: &Facts<'_, '_>,
    budget: &mut Budget,
) -> Result<Built, Error> {
    let formula = construct_tl_formula(
        package,
        subject,
        classification,
        surrounding,
        TlFormulaLimits {
            max_bytes: budget.limits.max_bytes,
            max_nodes: budget.limits.max_nodes,
            // The bridge counts formula nodes as work itself, below.
            max_work: u64::MAX,
        },
    )
    .map_err(formula_error)?;
    budget.charge(formula.formula().nodes().len(), Some(Limit::Nodes))?;
    budget.bytes(formula.bytes().len())?;
    budget.charge(formula.leaves().len(), None)?;
    let mut entries = Vec::new();
    for leaf in formula.leaves() {
        entries.push(PropositionEntry {
            id: leaf.proposition,
            name: format!("holds/{}", leaf.proposition.0),
        });
    }
    let map = PropositionMapDocument::new(entries).map_err(Error::PropositionMap)?;
    check_serialized_size(&map, budget.limits.max_bytes).map_err(formula_error)?;
    let map_bytes = map.canonical_json_bytes().map_err(Error::SyntaxEncoding)?;
    budget.bytes(map_bytes.len())?;
    let propositions = PropositionMapDocument::from_json_bytes(
        &map_bytes,
        SyntaxArtifactLimits {
            document_bytes: usize::try_from(budget.limits.max_bytes).unwrap_or(usize::MAX),
            ..SyntaxArtifactLimits::default()
        },
    )
    .map_err(Error::SyntaxRead)?;
    let mut instants = Vec::with_capacity(facts.positions.len());
    for &position in &facts.positions {
        let mut values = Vec::new();
        for leaf in formula.leaves() {
            let value = facts
                .values
                .get(&(position, &leaf.predicate))
                .ok_or_else(|| {
                    Error::Incomplete(super::TlMissingPremise::Valuation {
                        position,
                        operand: leaf.predicate.clone(),
                    })
                })?;
            if *value {
                values.push(leaf.proposition);
            }
        }
        instants.push(values);
    }
    let owner_limits = limits(selection.owner_limits, budget.limits.max_bytes);
    let max_bytes = budget.limits.max_bytes;
    let owner_error = |failure| error(failure, selection.owner_limits, max_bytes);
    let target = facts.target;
    let name = selection.observation.bundle().identity().as_str();
    let input = match target {
        Target::ClosedTrace | Target::OnlinePrefix => {
            let trace = TraceDocument {
                schema_version: TraceSchemaVersion::V1,
                trace_id: name.to_owned(),
                closed: target == Target::ClosedTrace,
                instants,
            };
            let document = wire::trace::derive(&trace, owner_limits).map_err(owner_error)?;
            budget.bytes(document.bytes().len())?;
            TlInput::Future(
                wire::trace::read(document.bytes(), &trace, owner_limits).map_err(owner_error)?,
            )
        }
        Target::OriginCompleteHistory => {
            let clock = match &facts.binding.clock {
                ClockConfiguration::EventPosition { .. } => ClockBinding::EventPosition,
                ClockConfiguration::FixedSample {
                    epoch,
                    period,
                    unit,
                } => ClockBinding::FixedSample {
                    epoch: exact(epoch)?,
                    period: exact(period)?,
                    unit: unit.to_string(),
                },
                ClockConfiguration::TimestampedEvent { .. } => {
                    return Err(Error::NativePremise(Premise::Clock))
                }
            };
            let mut observations = Vec::with_capacity(instants.len());
            for (index, values) in instants.into_iter().enumerate() {
                let position =
                    u64::try_from(index).map_err(|_| Error::NativePremise(Premise::Position))?;
                let sample = match &clock {
                    ClockBinding::EventPosition => None,
                    ClockBinding::FixedSample {
                        epoch,
                        period,
                        unit,
                    } => Some(ClockSample {
                        instant: fixed_sample_instant(*epoch, *period, position)
                            .map_err(Error::ExactNumber)?,
                        unit: unit.clone(),
                    }),
                    ClockBinding::Unsupported { .. } => {
                        return Err(Error::NativePremise(Premise::Clock))
                    }
                };
                observations.push(PositionObservation::new(position, values, sample));
            }
            let through = u64::try_from(observations.len())
                .ok()
                .and_then(|length| length.checked_sub(1))
                .ok_or(Error::Incomplete(super::TlMissingPremise::History))?;
            let history = PositionHistoryDocument::new(
                name,
                selection.observation.bundle().revision(),
                0,
                through,
                Some(clock),
                observations,
            )
            .map_err(Error::History)?;
            check_serialized_size(&history, budget.limits.max_bytes).map_err(formula_error)?;
            let document = history::derive(&history, owner_limits).map_err(owner_error)?;
            budget.bytes(document.bytes().len())?;
            TlInput::Past(
                history::read(document.bytes(), &history, owner_limits).map_err(owner_error)?,
            )
        }
    };
    budget.bytes(input.bytes().len())?;
    Ok(Built {
        formula,
        propositions,
        input,
    })
}

fn exact(number: &ClockExactNumber) -> Result<ExactNumber, Error> {
    let (numerator, denominator) = match number {
        ClockExactNumber::Integer { decimal } => (
            decimal
                .parse::<i64>()
                .map_err(|_| Error::NativePremise(Premise::ClockRepresentation))?,
            1,
        ),
        ClockExactNumber::Rational {
            numerator,
            denominator,
        } => (
            numerator
                .parse::<i64>()
                .map_err(|_| Error::NativePremise(Premise::ClockRepresentation))?,
            denominator
                .parse::<u64>()
                .map_err(|_| Error::NativePremise(Premise::ClockRepresentation))?,
        ),
    };
    ExactNumber::new(numerator, denominator).map_err(Error::ExactNumber)
}
