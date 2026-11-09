// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: non-evaluating admission and comparison of original owner results.

use std::collections::{BTreeMap, BTreeSet};

use qsl_eval::temporal::evidence::PackageMismatch;
use qsl_eval::temporal::{
    evidence::{
        matches_result_identity, ActivationState, EvaluatedEvidence, EvidencePackage,
        EvidenceResult, ExecutionDisposition, NonValueKind as NativeNonValueKind, Relation,
        Settlement, Truth,
    },
    ProgressClosure,
};
use qsl_package::BarePackage;
use qsl_package::NativeJoinBudget;
use quire_eval::{CallAdmissionFailure, CallAdmissionLocus, CallFailure};
use quire_mltl::{
    contract_ir::{MappedOutcome, MappingSelection, NonValueKind, ValidatedMappedResult},
    report::{AssessmentExecution, ResultRelationKind, SettlementBasis, TemporalTruth},
};
use quire_observation::authority::{availability, completeness, OpenClosed};
use serde::Serialize;
use serde_json::{json, Value};

use super::{
    projection::{hex, hex_string, lowered_owner_limits},
    TlLimits, TlProjectionLimit, TlProjectionUsage, ValidatedTlProjection,
};

/// The public version of an admitted native/TL join.
pub const VERSION: &str = "quire.native-temporal-join/v1";

/// The first matching row of the FR-350 decision table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlJoinDecision {
    /// An owner read or correspondence check refused.
    Refused,
    /// An assessment failed.
    Failed,
    /// A required result or producer is unavailable.
    Unavailable,
    /// An assessment is unsupported.
    Unsupported,
    /// Required evidence or an explicit resource is incomplete.
    Incomplete,
    /// Complete values, support or independent premises conflict.
    Conflict,
    /// Equal final Boolean values and all independent premises agree.
    EqualFinal,
    /// Both results are equally pending under an open decision scope.
    EqualPending,
}

impl TlJoinDecision {
    /// Stable public machine spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Incomplete => "incomplete",
            Self::Conflict => "conflict",
            Self::EqualFinal | Self::EqualPending => "agreement",
        }
    }

    /// Independent comparison disposition.
    pub const fn comparison(self) -> &'static str {
        match self {
            Self::Refused
            | Self::Failed
            | Self::Unavailable
            | Self::Unsupported
            | Self::Incomplete => "not-compared",
            Self::Conflict => "mismatch",
            Self::EqualFinal => "equal-final",
            Self::EqualPending => "equal-pending",
        }
    }
}

/// Every matching non-final row, emitted once in table order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlJoinCause {
    /// An owner read or complete correspondence check refused.
    ResultRefused,
    /// Either assessment failed.
    ResultFailed,
    /// A required result or producer was unavailable.
    ResultUnavailable,
    /// Either assessment was unsupported.
    ResultUnsupported,
    /// Either assessment was incomplete.
    ResultIncomplete,
    /// Values, complete premises, support or predecessor relations conflict.
    ResultMismatch,
    /// Both assessments are equally pending under an open decision scope.
    ResultPending,
}

impl TlJoinCause {
    /// Exact public cause spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResultRefused => "result-refused",
            Self::ResultFailed => "result-failed",
            Self::ResultUnavailable => "result-unavailable",
            Self::ResultUnsupported => "result-unsupported",
            Self::ResultIncomplete => "result-incomplete",
            Self::ResultMismatch => "result-mismatch",
            Self::ResultPending => "result-pending",
        }
    }
}

/// Stable refusal discriminator of a whole join operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlJoinCode {
    /// Supplied bytes differ from the rebuilt join.
    ExpectedMismatch,
    /// A bridge limit, or a native owner resource denial, stopped the operation.
    LimitReached,
    /// The canonical writer refused the join record.
    Encoding,
    /// Original native evidence package or identity-binding refusal.
    NativeEvidence,
}

impl TlJoinCode {
    /// Exact public machine spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExpectedMismatch => "tl-join-expected-mismatch",
            Self::LimitReached => "tl-join-limit-reached",
            Self::Encoding => "tl-join-encoding",
            Self::NativeEvidence => "tl-join-native-report",
        }
    }
    /// The complete closed code catalog.
    pub const fn all() -> &'static [Self] {
        &[
            Self::ExpectedMismatch,
            Self::LimitReached,
            Self::Encoding,
            Self::NativeEvidence,
        ]
    }
    /// Parse one exact public spelling, without accepting aliases.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|item| item.as_str() == code)
    }
}

/// A whole-operation refusal; no partial join is observable.
#[derive(Debug, thiserror::Error)]
pub enum TlJoinError {
    /// The supplied join bytes differ from the rebuilt join.
    #[error("the supplied join is not the complete canonical expected join")]
    ExpectedMismatch,
    /// First excess unit, refused before the named action.
    #[error("join {limit:?} limit reached: {value} > {setting}")]
    LimitReached {
        /// The configured caller counter.
        limit: TlProjectionLimit,
        /// The first excess count, `setting + 1`.
        value: u64,
        /// The caller ceiling.
        setting: u64,
    },
    /// The original native subject or identity-binding refusal.
    #[error("native evidence owner refused: {0:?}")]
    NativeEvidence(CallFailure),
    /// The original native package-row or capture-projection refusal.
    #[error("native evidence package refused: {0:?}")]
    NativePackage(PackageMismatch),
    /// Original canonical encoder refusal.
    #[error("join canonical encoding refused: {0}")]
    Encoding(#[source] quire_canonical::Error),
}

impl TlJoinError {
    /// Stable discriminator independent of the displayed message.
    pub const fn code(&self) -> TlJoinCode {
        match self {
            Self::ExpectedMismatch => TlJoinCode::ExpectedMismatch,
            Self::LimitReached { .. } | Self::NativePackage(PackageMismatch::Exhausted(_)) => {
                TlJoinCode::LimitReached
            }
            Self::NativeEvidence(_) | Self::NativePackage(_) => TlJoinCode::NativeEvidence,
            Self::Encoding(_) => TlJoinCode::Encoding,
        }
    }
}

/// A genuine native result belongs to another independently selected input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlNativeMismatch {
    /// The result's checked package, clause, occurrence or source map differs.
    Subject,
    /// The original request differs from the evaluator-issued result identity.
    Input,
    /// The current correction does not name the supplied prior issued result.
    Predecessor,
}

/// Actual result-owner read failures, retained without string conversion.
#[derive(Debug)]
pub struct TlJoinOwnerRefusals {
    native: Option<TlNativeMismatch>,
    mapped: Option<tl_mltl::wire::OwnerReadError>,
}

impl TlJoinOwnerRefusals {
    /// Native source, request or predecessor mismatch of a genuine issued result.
    pub fn native(&self) -> Option<&TlNativeMismatch> {
        self.native.as_ref()
    }
    /// Original non-evaluating TL mapped-result owner refusal.
    pub fn mapped(&self) -> Option<&tl_mltl::wire::OwnerReadError> {
        self.mapped.as_ref()
    }
}

/// Constructor-private result retaining the complete projection and actual sources.
pub struct ValidatedTlJoin<'a> {
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<&'a Result<EvaluatedEvidence, CallFailure>>,
    native_admitted: bool,
    mapped: Option<ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    refusals: TlJoinOwnerRefusals,
    decision: TlJoinDecision,
    causes: Vec<TlJoinCause>,
    value: Option<bool>,
    bytes: Vec<u8>,
    usage: TlProjectionUsage,
}

impl std::fmt::Debug for ValidatedTlJoin<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedTlJoin")
            .field("decision", &self.decision)
            .field("causes", &self.causes)
            .field("value", &self.value)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

impl<'a> ValidatedTlJoin<'a> {
    /// Complete admitted correspondence and all original owner input values.
    pub fn projection(&self) -> &'a ValidatedTlProjection<'a> {
        self.projection
    }
    /// Original native owner result; no bridge mirror replaces it.
    pub fn native(&self) -> Option<&'a EvaluatedEvidence> {
        self.native.and_then(|outcome| outcome.as_ref().ok())
    }
    /// Original whole owner call outcome, preserving failure separately from absence.
    pub fn native_outcome(&self) -> Option<&'a Result<EvaluatedEvidence, CallFailure>> {
        self.native
    }
    /// Original owner call failure, never promoted to validated result authority.
    pub fn native_call_failure(&self) -> Option<&'a CallFailure> {
        self.native.and_then(|outcome| outcome.as_ref().err())
    }
    /// Whether the genuine issued result matched this input and supplied predecessor.
    pub fn native_admitted(&self) -> bool {
        self.native_admitted
    }
    /// Actual owner-admitted mapped TL result and complete source result.
    pub fn mapped(&self) -> Option<&ValidatedMappedResult> {
        self.mapped.as_ref()
    }
    /// Exact immutable previously admitted join, when supplied.
    pub fn previous(&self) -> Option<&'a ValidatedTlJoin<'a>> {
        self.previous
    }
    /// The decision.
    pub fn decision(&self) -> TlJoinDecision {
        self.decision
    }
    /// Every holding R1 to R6 cause in row order, then `result-pending` for R8.
    pub fn causes(&self) -> &[TlJoinCause] {
        &self.causes
    }
    /// Present only for an R7 decision.
    pub fn value(&self) -> Option<bool> {
        self.value
    }
    /// Original owner refusals, which never become successful Boolean values.
    pub fn owner_refusals(&self) -> &TlJoinOwnerRefusals {
        &self.refusals
    }
    /// Exact immutable RFC 8785 join bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Fresh bridge counters for this join or strict-read call.
    pub fn usage(&self) -> TlProjectionUsage {
        self.usage
    }
}

/// Actual expected owner inputs of the strict join reader.
#[derive(Clone, Copy)]
pub struct ExpectedTlJoin<'a> {
    /// Complete admitted projection and correspondence.
    pub projection: &'a ValidatedTlProjection<'a>,
    /// Original whole native call outcome, or genuine absence.
    pub native: Option<&'a Result<EvaluatedEvidence, CallFailure>>,
    /// Actual TL mapped result or genuine absence.
    pub mapped: Option<&'a ValidatedMappedResult>,
    /// Exact previous admitted join or genuine absence.
    pub previous: Option<&'a ValidatedTlJoin<'a>>,
}

struct Budget {
    limits: TlLimits,
    usage: TlProjectionUsage,
}

impl Budget {
    fn new(limits: TlLimits) -> Self {
        Self {
            limits,
            usage: TlProjectionUsage::default(),
        }
    }
    /// Charge `count` units to a counter and then to work, before they are visited.
    fn charge(&mut self, counter: TlProjectionLimit, count: usize) -> Result<(), TlJoinError> {
        let count = u64::try_from(count).unwrap_or(u64::MAX);
        let (used, setting) = match counter {
            TlProjectionLimit::Support => (&mut self.usage.support, self.limits.max_support),
            TlProjectionLimit::Causes => (&mut self.usage.causes, self.limits.max_causes),
            _ => unreachable!("join charges only support and causes"),
        };
        if used.saturating_add(count) > setting {
            return Err(limit(counter, setting));
        }
        *used += count;
        if counter == TlProjectionLimit::Support {
            if self.usage.work.saturating_add(count) > self.limits.max_work {
                return Err(limit(TlProjectionLimit::Work, self.limits.max_work));
            }
            self.usage.work += count;
        }
        Ok(())
    }
    /// Visit original native diagnostic members before copying them into the record.
    fn diagnostic_work(&mut self, count: usize) -> Result<(), TlJoinError> {
        let count = u64::try_from(count).unwrap_or(u64::MAX);
        if self.usage.work.saturating_add(count) > self.limits.max_work {
            return Err(limit(TlProjectionLimit::Work, self.limits.max_work));
        }
        self.usage.work += count;
        Ok(())
    }
    fn bytes(&mut self, length: usize) -> Result<(), TlJoinError> {
        let value = u64::try_from(length).unwrap_or(u64::MAX);
        if value > self.limits.max_bytes {
            return Err(limit(TlProjectionLimit::Bytes, self.limits.max_bytes));
        }
        self.usage.bytes = self.usage.bytes.max(value);
        Ok(())
    }
}

fn limit(counter: TlProjectionLimit, setting: u64) -> TlJoinError {
    TlJoinError::LimitReached {
        limit: counter,
        value: setting.saturating_add(1),
        setting,
    }
}

/// Bind the original native call outcome, owner-read the TL result and apply the join table.
/// This operation never executes a temporal evaluator.
pub fn join_tl<'a>(
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<&'a Result<EvaluatedEvidence, CallFailure>>,
    mapped: Option<&ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    limits: TlLimits,
    native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin<'a>, TlJoinError> {
    build(
        projection,
        native,
        mapped,
        previous,
        Budget::new(limits),
        native_budget,
    )
}

/// Rebuild the expected join from its independent inputs and require byte equality.
/// This rejects every alternate spelling, unknown, duplicate, missing or reordered
/// member, trailing input and cross-wired source or predecessor.
pub fn read_tl_join<'a>(
    bytes: &[u8],
    expected: ExpectedTlJoin<'a>,
    limits: TlLimits,
    native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin<'a>, TlJoinError> {
    let mut budget = Budget::new(limits);
    budget.bytes(bytes.len())?;
    let result = build(
        expected.projection,
        expected.native,
        expected.mapped,
        expected.previous,
        budget,
        native_budget,
    )?;
    if result.bytes != bytes {
        return Err(TlJoinError::ExpectedMismatch);
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeState {
    Refused,
    Failed,
    Lineage,
    Unavailable,
    Incomplete,
    Final(bool),
    Pending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TlState {
    Refused,
    Failed,
    Unavailable,
    Unsupported,
    Incomplete,
    Contradicted,
    Final(bool),
    Pending,
}

fn native_final(truth: Option<Truth>, settlement: Option<Settlement>) -> Option<bool> {
    if !matches!(
        settlement,
        Some(
            Settlement::ClosedScope
                | Settlement::DecisiveWitness
                | Settlement::DecisiveCounterexample
        )
    ) {
        return None;
    }
    match truth {
        Some(Truth::True) => Some(true),
        Some(Truth::False) => Some(false),
        Some(Truth::Pending) | None => None,
    }
}

fn native_state(result: &EvidenceResult) -> NativeState {
    if result.incomplete.is_some() || result.indeterminate.is_some() {
        return NativeState::Incomplete;
    }
    if let Some(non_value) = &result.non_value {
        return match non_value.kind {
            NativeNonValueKind::Refused => NativeState::Refused,
            NativeNonValueKind::Missing => NativeState::Incomplete,
            NativeNonValueKind::Inactive
            | NativeNonValueKind::ActivationUnknown
            | NativeNonValueKind::Unactivated => NativeState::Unavailable,
        };
    }
    match result.activation {
        ActivationState::Inactive | ActivationState::Unknown | ActivationState::Unavailable => {
            NativeState::Unavailable
        }
        ActivationState::Active => native_final(result.truth, result.settlement)
            .map_or(NativeState::Pending, NativeState::Final),
    }
}

fn tl_state(mapped: &ValidatedMappedResult) -> TlState {
    let source = mapped.source();
    if source.execution() == AssessmentExecution::Refused {
        TlState::Refused
    } else if source.execution() == AssessmentExecution::Failed {
        TlState::Failed
    } else if matches!(
        mapped.outcome(),
        MappedOutcome::NonValue {
            reason: NonValueKind::Unavailable
        }
    ) || source.availability().state() != availability::State::Available
    {
        TlState::Unavailable
    } else if source.execution() == AssessmentExecution::Unsupported {
        TlState::Unsupported
    } else if source.execution() == AssessmentExecution::ResourceIncomplete
        || source.completeness().state() == completeness::State::Incomplete
    {
        TlState::Incomplete
    } else if source.completeness().state() == completeness::State::Contradicted {
        TlState::Contradicted
    } else {
        match mapped.outcome() {
            MappedOutcome::Value { value } if source.is_final() => TlState::Final(value),
            _ => TlState::Pending,
        }
    }
}

fn settlement_equal(native: Option<Settlement>, tl: SettlementBasis) -> bool {
    matches!(
        (native, tl),
        (Some(Settlement::ClosedScope), SettlementBasis::ClosedScope)
            | (
                Some(Settlement::DecisiveWitness),
                SettlementBasis::DecisiveWitness
            )
            | (
                Some(Settlement::DecisiveCounterexample),
                SettlementBasis::DecisiveCounterexample
            )
            | (Some(Settlement::Unsettled), SettlementBasis::Unsettled)
            | (
                Some(Settlement::Unavailable) | None,
                SettlementBasis::Unavailable
            )
    )
}

/// Premise equality `P` over two admitted results.
fn premises_equal(
    projection: &ValidatedTlProjection<'_>,
    native: &EvidenceResult,
    mapped: &ValidatedMappedResult,
) -> bool {
    let source = mapped.source();
    let expected = projection.request();
    let execution = match projection.native().request.execution {
        ExecutionDisposition::Completed => AssessmentExecution::Completed,
        ExecutionDisposition::Failed => AssessmentExecution::Failed,
    };
    let native_support: BTreeSet<&str> = native
        .support
        .iter()
        .map(|support| support.observation.identity.as_str())
        .collect();
    let tl_support: BTreeSet<&str> = source
        .decision_support()
        .iter()
        .map(String::as_str)
        .collect();
    source.request() == expected
        && source.execution() == execution
        && source.decision_scope_progress() == expected.decision_scope_progress()
        && source.decision_scope_closure() == expected.decision_scope_closure()
        && source.surrounding_execution_progress() == expected.surrounding_execution_progress()
        && source.surrounding_execution_closure() == expected.surrounding_execution_closure()
        && source.completeness() == expected.completeness()
        && source.availability() == expected.availability()
        && settlement_equal(native.settlement, source.settlement())
        && native_support == tl_support
}

#[derive(Serialize, quire_canonical::FixedShape)]
#[serde(untagged)]
enum NativeOutcomeRecord<'a> {
    Identity(String),
    Failure(NativeFailureRecord<'a>),
}

#[derive(Serialize, quire_canonical::FixedShape)]
#[serde(tag = "failure", rename_all = "snake_case")]
enum NativeFailureRecord<'a> {
    Admission(NativeAdmissionRecord<'a>),
    Fault(NativeFaultRecord<'a>),
}

#[derive(Serialize, quire_canonical::FixedShape)]
#[serde(rename_all = "snake_case")]
enum NativeAdmissionKind {
    Refused,
    Incomplete,
}

#[derive(Serialize, quire_canonical::FixedShape)]
struct NativeAdmissionRecord<'a> {
    kind: NativeAdmissionKind,
    code: &'a str,
    cause: &'a str,
    prefix: Option<&'a str>,
    fields: &'a BTreeMap<&'static str, String>,
    member: &'a str,
    index: Option<String>,
    path: Vec<NativeAdmissionPath<'a>>,
}

#[derive(Serialize, quire_canonical::FixedShape)]
#[serde(untagged)]
enum NativeAdmissionPath<'a> {
    Field { field: &'a str },
    Option { option: bool },
    Index { index: String },
}

#[derive(Serialize, quire_canonical::FixedShape)]
struct NativeFaultRecord<'a> {
    stage: &'a str,
    invariant: &'a str,
    named: &'a [(&'static str, String)],
}

/// Encode the current evidence call's original diagnostic members without parsing
/// display text. Other call families retain their typed failure as an operation error.
fn native_outcome_record<'a>(
    outcome: &'a Result<EvaluatedEvidence, CallFailure>,
    budget: &mut Budget,
) -> Result<NativeOutcomeRecord<'a>, TlJoinError> {
    match outcome {
        Ok(issued) => {
            let identity = issued.result().result_identity.as_bytes();
            budget.bytes(identity.len())?;
            Ok(NativeOutcomeRecord::Identity(hex_string(identity)))
        }
        Err(failure @ CallFailure::Admission(admission)) => {
            let (kind, diagnostic) = match admission.as_ref() {
                CallAdmissionFailure::Refused(record) => (NativeAdmissionKind::Refused, record),
                CallAdmissionFailure::Incomplete(record) => {
                    (NativeAdmissionKind::Incomplete, record)
                }
            };
            // Temporal evidence admission names a request member and has no kernel
            // construction cause. Never erase a different family's extra provenance.
            let CallAdmissionLocus::Member { name, index } = &diagnostic.locus else {
                return Err(TlJoinError::NativeEvidence(failure.clone()));
            };
            if diagnostic.record.kernel.is_some() {
                return Err(TlJoinError::NativeEvidence(failure.clone()));
            }
            budget.diagnostic_work(
                diagnostic
                    .record
                    .fields
                    .len()
                    .saturating_add(diagnostic.record.path.len())
                    .saturating_add(1),
            )?;
            let size = diagnostic
                .record
                .fields
                .iter()
                .fold(name.len(), |sum, (key, value)| {
                    sum.saturating_add(key.len()).saturating_add(value.len())
                });
            let size = diagnostic
                .record
                .path
                .iter()
                .fold(size, |sum, step| match step {
                    quire_semantic_value::admission::PathStep::Field(name) => {
                        sum.saturating_add(name.len())
                    }
                    quire_semantic_value::admission::PathStep::OptionValue
                    | quire_semantic_value::admission::PathStep::Index(_) => sum,
                });
            budget.bytes(size)?;
            let path = diagnostic
                .record
                .path
                .iter()
                .map(|step| match step {
                    quire_semantic_value::admission::PathStep::Field(name) => {
                        NativeAdmissionPath::Field { field: name }
                    }
                    quire_semantic_value::admission::PathStep::OptionValue => {
                        NativeAdmissionPath::Option { option: true }
                    }
                    quire_semantic_value::admission::PathStep::Index(index) => {
                        NativeAdmissionPath::Index {
                            index: index.to_string(),
                        }
                    }
                })
                .collect::<Vec<_>>();
            Ok(NativeOutcomeRecord::Failure(
                NativeFailureRecord::Admission(NativeAdmissionRecord {
                    kind,
                    code: diagnostic.record.catalog.code(),
                    cause: diagnostic.record.catalog.cause(),
                    prefix: diagnostic.record.catalog.prefix(),
                    fields: &diagnostic.record.fields,
                    member: name,
                    index: index.map(|value| value.to_string()),
                    path,
                }),
            ))
        }
        Err(failure @ CallFailure::Fault(fault)) => {
            if fault.kernel_cause().is_some() {
                return Err(TlJoinError::NativeEvidence(failure.clone()));
            }
            budget.diagnostic_work(fault.named().len().saturating_add(1))?;
            let size = fault.named().iter().fold(
                fault.stage().len().saturating_add(fault.invariant().len()),
                |sum, (name, value)| sum.saturating_add(name.len()).saturating_add(value.len()),
            );
            budget.bytes(size)?;
            Ok(NativeOutcomeRecord::Failure(NativeFailureRecord::Fault(
                NativeFaultRecord {
                    stage: fault.stage(),
                    invariant: fault.invariant(),
                    named: fault.named(),
                },
            )))
        }
        Err(
            failure @ (CallFailure::Input(_)
            | CallFailure::Body(_)
            | CallFailure::Cancelled(_)
            | CallFailure::Storage { .. }
            | CallFailure::Supplied(_)
            | CallFailure::UnsupportedHelper(_)
            | CallFailure::Unsupported(_)
            | CallFailure::Limit(_)),
        ) => Err(TlJoinError::NativeEvidence(failure.clone())),
    }
}

fn build<'a>(
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<&'a Result<EvaluatedEvidence, CallFailure>>,
    mapped: Option<&ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    mut budget: Budget,
    native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin<'a>, TlJoinError> {
    let native_record = native
        .map(|outcome| native_outcome_record(outcome, &mut budget))
        .transpose()?;
    if let Some(record) = &native_record {
        let bytes = quire_canonical::to_vec(
            record,
            quire_canonical::Limits::new(budget.limits.max_bytes),
        )
        .map_err(|error| match error {
            quire_canonical::Error::Limit(_) => {
                limit(TlProjectionLimit::Bytes, budget.limits.max_bytes)
            }
            error => TlJoinError::Encoding(error),
        })?;
        budget.bytes(bytes.len())?;
    }
    if let Some(result) = mapped {
        budget.bytes(result.bytes().len())?;
        budget.bytes(result.source().bytes().len())?;
    }
    let input = projection.native();
    let mut refusals = TlJoinOwnerRefusals {
        native: None,
        mapped: None,
    };

    // Native authority is the original opaque evaluator-issued carrier. Matching its
    // input identity binds it to this projection; no evaluator runs in the join.
    let prior_native = previous
        .filter(|join| join.native_admitted())
        .and_then(ValidatedTlJoin::native);
    let mut native_side = NativeState::Unavailable;
    let mut read_native = None;
    if let Some(Err(failure)) = native {
        native_side = match failure {
            CallFailure::Admission(admission) => match admission.as_ref() {
                CallAdmissionFailure::Refused(_) => NativeState::Refused,
                CallAdmissionFailure::Incomplete(_) => NativeState::Incomplete,
            },
            CallFailure::Fault(_) => NativeState::Failed,
            CallFailure::Input(_)
            | CallFailure::Body(_)
            | CallFailure::Cancelled(_)
            | CallFailure::Storage { .. }
            | CallFailure::Supplied(_)
            | CallFailure::UnsupportedHelper(_)
            | CallFailure::Unsupported(_)
            | CallFailure::Limit(_) => return Err(TlJoinError::NativeEvidence(failure.clone())),
        };
    }
    if let Some(Ok(issued)) = native {
        let result = issued.result();
        let source = projection.source();
        let selected = &result.subject;
        let subject_matches = selected.package_identity == projection.package().name()
            && selected.package_version == qsl_package::CHECKED_PACKAGE_V2
            && selected.clause == input.occurrence.node
            && selected.occurrence.role == input.occurrence.role
            && selected.occurrence.ordinal == input.occurrence.ordinal
            && &selected.source_map == source;
        if !subject_matches {
            native_side = NativeState::Refused;
            refusals.native = Some(TlNativeMismatch::Subject);
        } else {
            let bare = BarePackage::new(projection.package());
            let package = EvidencePackage::from_package(&bare, native_budget)
                .map_err(TlJoinError::NativePackage)?;
            let subject = package
                .subject(&input.occurrence.node, input.request.occurrence.as_ref())
                .map_err(TlJoinError::NativeEvidence)?;
            if !matches_result_identity(&subject, input.request, result.result_identity.as_bytes())
                .map_err(TlJoinError::NativeEvidence)?
            {
                native_side = NativeState::Refused;
                refusals.native = Some(TlNativeMismatch::Input);
            } else {
                let prior_matches = match &result.relation {
                    Relation::Original => previous.is_none(),
                    Relation::Superseding(identity) | Relation::Invalidating(identity) => {
                        prior_native.is_some_and(|prior| {
                            let prior = prior.result();
                            identity == &prior.result_identity
                                && prior.lineage.checked_add(1) == Some(result.lineage)
                        })
                    }
                };
                if !prior_matches {
                    native_side = NativeState::Lineage;
                    refusals.native = Some(TlNativeMismatch::Predecessor);
                } else {
                    native_side = native_state(result);
                    read_native = Some(result);
                }
            }
        }
    }

    // TL side: non-evaluating re-read through the owner's selection.
    let mut tl_side = TlState::Unavailable;
    let mut read_mapped = None;
    if let Some(result) = mapped {
        let selection = MappingSelection::for_result(result.source());
        let owner_limits =
            lowered_owner_limits(projection.selection().owner_limits, budget.limits.max_bytes);
        match quire_mltl::contract_ir::read(
            result.bytes(),
            result.source(),
            &selection,
            owner_limits,
        ) {
            Ok(read) => {
                let source = read.source();
                let request = projection.request();
                tl_side = if source.request() != request
                    || source.subject_identity() != request.subject_identity()
                    || source.correspondence_identity() != request.correspondence_identity()
                {
                    TlState::Refused
                } else {
                    tl_state(&read)
                };
                read_mapped = Some(read);
            }
            Err(error) => {
                tl_side = TlState::Refused;
                refusals.mapped = Some(error);
            }
        }
    }

    // Both strict reads admitted: charge the compared support.
    let admitted_pair = match (&read_native, &read_mapped) {
        (Some(report), Some(result)) => {
            budget.charge(
                TlProjectionLimit::Support,
                report.support.len() + result.source().decision_support().len(),
            )?;
            Some((report, result))
        }
        _ => None,
    };

    // Lineage conflict L.
    let lineage_conflict = match (native, mapped) {
        (Some(Ok(report)), Some(result)) => {
            let native_correction = !matches!(report.result().relation, Relation::Original);
            let tl_correction = result.source().relation_kind() != ResultRelationKind::Original;
            (native_correction != tl_correction)
                || (!native_correction && !tl_correction && previous.is_some())
                || (native_correction
                    && tl_correction
                    && previous.is_none_or(|join| !join.native_admitted()))
                || result
                    .source()
                    .validate_supplied_predecessor(
                        previous
                            .and_then(ValidatedTlJoin::mapped)
                            .map(|prior| prior.source().bytes()),
                    )
                    .is_err()
        }
        _ => false,
    };

    let both_settled = matches!(native_side, NativeState::Final(_) | NativeState::Pending)
        && matches!(tl_side, TlState::Final(_) | TlState::Pending);
    let premises = both_settled
        && admitted_pair.is_some_and(|(report, result)| premises_equal(projection, report, result));
    let r7 = match (native_side, tl_side) {
        (NativeState::Final(left), TlState::Final(right)) => {
            left == right && premises && !lineage_conflict
        }
        _ => false,
    };
    let r8 = both_settled
        && native_side == NativeState::Pending
        && tl_side == TlState::Pending
        && premises
        && !lineage_conflict
        && admitted_pair.is_some_and(|(report, result)| {
            report.truth == Some(Truth::Pending)
                && report.settlement == Some(Settlement::Unsettled)
                && result.source().truth() == TemporalTruth::Pending
                && result.source().settlement() == SettlementBasis::Unsettled
                && input.request.decision_closure == Some(ProgressClosure::Open)
                && result.source().decision_scope_closure().state() == OpenClosed::Open
        });
    let rows = [
        (
            native_side == NativeState::Refused || tl_side == TlState::Refused,
            TlJoinDecision::Refused,
            TlJoinCause::ResultRefused,
        ),
        (
            input.request.execution == ExecutionDisposition::Failed
                || native_side == NativeState::Failed
                || tl_side == TlState::Failed,
            TlJoinDecision::Failed,
            TlJoinCause::ResultFailed,
        ),
        (
            native_side == NativeState::Unavailable || tl_side == TlState::Unavailable,
            TlJoinDecision::Unavailable,
            TlJoinCause::ResultUnavailable,
        ),
        (
            tl_side == TlState::Unsupported,
            TlJoinDecision::Unsupported,
            TlJoinCause::ResultUnsupported,
        ),
        (
            native_side == NativeState::Incomplete || tl_side == TlState::Incomplete,
            TlJoinDecision::Incomplete,
            TlJoinCause::ResultIncomplete,
        ),
        (
            lineage_conflict
                || native_side == NativeState::Lineage
                || tl_side == TlState::Contradicted
                || (both_settled && !r7 && !r8),
            TlJoinDecision::Conflict,
            TlJoinCause::ResultMismatch,
        ),
    ];
    let mut decision = None;
    let mut causes = Vec::new();
    for (holds, row, cause) in rows {
        if holds {
            decision.get_or_insert(row);
            causes.push(cause);
        }
    }
    let decision = match decision {
        Some(row) => row,
        None if r7 => TlJoinDecision::EqualFinal,
        None => TlJoinDecision::EqualPending,
    };
    if decision == TlJoinDecision::EqualPending {
        causes.push(TlJoinCause::ResultPending);
    }
    let value = match (decision, native_side) {
        (TlJoinDecision::EqualFinal, NativeState::Final(value)) => Some(value),
        _ => None,
    };
    budget.charge(TlProjectionLimit::Causes, causes.len())?;

    let previous_native_record = previous
        .and_then(|join| join.native)
        .map(|outcome| native_outcome_record(outcome, &mut budget))
        .transpose()?;
    let record = json!({
        "version": VERSION,
        "correspondence": hex(projection.bytes()),
        "native": native_record,
        "nativeAdmitted": read_native.is_some(),
        "mapped": mapped.map(|result| hex(result.bytes())),
        "tlResult": mapped.map(|result| hex(result.source().bytes())),
        "previous": previous.map(|join| json!({
            "native": previous_native_record,
            "tlResult": join.mapped.as_ref().map(|result| hex(result.source().bytes())),
        })),
        "decision": decision.as_str(),
        "comparison": decision.comparison(),
        "causes": causes.iter().map(|cause| Value::from(cause.as_str())).collect::<Vec<_>>(),
        "value": value,
    });
    let bytes = quire_canonical::to_vec(
        &record,
        quire_canonical::Limits::new(budget.limits.max_bytes),
    )
    .map_err(|error| match error {
        quire_canonical::Error::Limit(_) => {
            limit(TlProjectionLimit::Bytes, budget.limits.max_bytes)
        }
        error => TlJoinError::Encoding(error),
    })?;
    budget.bytes(bytes.len())?;
    Ok(ValidatedTlJoin {
        projection,
        native,
        native_admitted: read_native.is_some(),
        mapped: mapped.cloned(),
        previous,
        refusals,
        decision,
        causes,
        value,
        bytes,
        usage: budget.usage,
    })
}
