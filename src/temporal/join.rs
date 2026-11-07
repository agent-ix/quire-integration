// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: non-evaluating admission and comparison of original owner results.

use qsl_eval::temporal::evidence::{Settlement, Truth};
use qsl_package::NativeJoinBudget;
use quire_mltl::{contract_ir::ValidatedMappedResult, report::ValidatedTemporalResult};
use quire_spec_language::temporal::{Report, ReportPredecessorUsage, ReportRefusal};

use super::{TlLimits, TlProjectionLimit, TlProjectionUsage, ValidatedTlProjection};

mod compare;
mod wire;

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
    /// Canonical bytes do not equal the supplied exact expected owner values.
    ExpectedMismatch,
    /// No decision-table row matches the owner pair.
    ResultMismatch,
    /// The next named bridge action exceeds its caller ceiling.
    LimitReached,
    /// Canonical encoding refuses the whole record.
    Encoding,
    /// The native report owner denied a bounded operation.
    NativeReport,
    /// The selected owner cannot attest an exact correction predecessor.
    PredecessorAuthority,
}

impl TlJoinCode {
    /// Exact public machine spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExpectedMismatch => "tl-join-expected-mismatch",
            Self::ResultMismatch => "tl-join-result-mismatch",
            Self::LimitReached => "tl-join-limit-reached",
            Self::Encoding => "tl-join-encoding",
            Self::NativeReport => "tl-join-native-report",
            Self::PredecessorAuthority => "tl-join-predecessor-authority",
        }
    }
    /// The complete closed code catalog.
    pub const fn all() -> &'static [Self] {
        &[
            Self::ExpectedMismatch,
            Self::ResultMismatch,
            Self::LimitReached,
            Self::Encoding,
            Self::NativeReport,
            Self::PredecessorAuthority,
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

/// An atomic join or strict-read refusal, exposing no partial join.
#[derive(Debug, thiserror::Error)]
pub enum TlJoinError {
    /// The complete canonical expected join differs.
    #[error("the supplied join is not the complete canonical expected join")]
    ExpectedMismatch,
    /// None of the independently evaluated table rows matched.
    #[error("the original result pair matches no join decision row")]
    ResultMismatch,
    /// The owner has no non-evaluating exact predecessor-value validation entry.
    #[error("the TL owner cannot authenticate the exact correction predecessor value")]
    PredecessorAuthority {
        /// Actual bounded native writer receipt before the TL-side stop.
        native_encoding_usage: Option<ReportPredecessorUsage>,
        /// Actual successful native full-predecessor validation, if that side admitted.
        native_predecessor_usage: Option<ReportPredecessorUsage>,
    },
    /// Original bounded native report-owner denial; no partial join is returned.
    #[error("native report owner refused: {0:?}")]
    NativeReport(ReportRefusal),
    /// First excess unit, refused before the named action.
    #[error("join {limit:?} limit reached: {value} > {setting}")]
    LimitReached {
        /// The configured caller counter.
        limit: TlProjectionLimit,
        /// The first excess count.
        value: u64,
        /// The original caller ceiling.
        setting: u64,
    },
    /// Original canonical encoder refusal.
    #[error("join canonical encoding refused: {0}")]
    Encoding(#[source] quire_canonical::Error),
}

impl TlJoinError {
    /// Stable discriminator independent of the displayed message.
    pub const fn code(&self) -> TlJoinCode {
        match self {
            Self::ExpectedMismatch => TlJoinCode::ExpectedMismatch,
            Self::ResultMismatch => TlJoinCode::ResultMismatch,
            Self::PredecessorAuthority { .. } => TlJoinCode::PredecessorAuthority,
            Self::NativeReport(ReportRefusal::PredecessorLimit { .. }) => TlJoinCode::LimitReached,
            Self::NativeReport(_) => TlJoinCode::NativeReport,
            Self::LimitReached { .. } => TlJoinCode::LimitReached,
            Self::Encoding(_) => TlJoinCode::Encoding,
        }
    }
}

/// Actual result-owner read failures, retained without string conversion.
#[derive(Debug)]
pub struct TlJoinOwnerRefusals {
    native: Option<ReportRefusal>,
    native_predecessor: Option<ReportRefusal>,
    mapped: Option<tl_mltl::wire::OwnerReadError>,
}

impl TlJoinOwnerRefusals {
    /// Original native owner refusal, if that read refused.
    pub fn native(&self) -> Option<&ReportRefusal> {
        self.native.as_ref()
    }
    /// Original full-predecessor mismatch, separate from a strict report-reader refusal.
    pub fn native_predecessor(&self) -> Option<&ReportRefusal> {
        self.native_predecessor.as_ref()
    }
    /// Original non-evaluating TL mapped-result owner refusal.
    pub fn mapped(&self) -> Option<&tl_mltl::wire::OwnerReadError> {
        self.mapped.as_ref()
    }
}

/// Constructor-private result retaining the complete projection and actual sources.
pub struct ValidatedTlJoin<'a> {
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<Report>,
    native_admitted: bool,
    mapped: Option<ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    refusals: TlJoinOwnerRefusals,
    decision: TlJoinDecision,
    causes: Vec<TlJoinCause>,
    value: Option<bool>,
    bytes: Vec<u8>,
    usage: TlProjectionUsage,
    native_encoding_usage: Option<ReportPredecessorUsage>,
    native_predecessor_usage: Option<ReportPredecessorUsage>,
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
    pub fn native(&self) -> Option<&Report> {
        self.native.as_ref()
    }
    /// Whether strict native admission succeeded in this exact join context.
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
    /// The first matching decision-table row.
    pub fn decision(&self) -> TlJoinDecision {
        self.decision
    }
    /// Every independently matching non-final cause, in table order.
    pub fn causes(&self) -> &[TlJoinCause] {
        &self.causes
    }
    /// Present only for an equal final pair with no earlier non-final row.
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
    /// Actual bounded native writer receipt, separate from bridge and IR accounting.
    pub fn native_encoding_usage(&self) -> Option<ReportPredecessorUsage> {
        self.native_encoding_usage
    }
    /// Actual successful full native predecessor validator receipt.
    pub fn native_predecessor_usage(&self) -> Option<ReportPredecessorUsage> {
        self.native_predecessor_usage
    }
    /// Fresh actual bridge counters for this join or strict-read call.
    pub fn usage(&self) -> TlProjectionUsage {
        self.usage
    }
}

/// Actual expected owner inputs of the strict join reader.
#[derive(Clone, Copy)]
pub struct ExpectedTlJoin<'a> {
    /// Complete admitted projection and correspondence.
    pub projection: &'a ValidatedTlProjection<'a>,
    /// Actual native result or genuine absence.
    pub native: Option<&'a Report>,
    /// Actual TL mapped result or genuine absence.
    pub mapped: Option<&'a ValidatedMappedResult>,
    /// Exact previous admitted join or genuine absence.
    pub previous: Option<&'a ValidatedTlJoin<'a>>,
}

struct Budget {
    limits: TlLimits,
    usage: TlProjectionUsage,
    native_encoding_usage: Option<ReportPredecessorUsage>,
    native_predecessor_usage: Option<ReportPredecessorUsage>,
}

impl Budget {
    fn new(limits: TlLimits) -> Self {
        Self {
            limits,
            usage: TlProjectionUsage::default(),
            native_encoding_usage: None,
            native_predecessor_usage: None,
        }
    }
    fn charge(&mut self, counter: TlProjectionLimit) -> Result<(), TlJoinError> {
        let (used, setting) = match counter {
            TlProjectionLimit::Support => (&mut self.usage.support, self.limits.max_support),
            TlProjectionLimit::Causes => (&mut self.usage.causes, self.limits.max_causes),
            TlProjectionLimit::Work => (&mut self.usage.work, self.limits.max_work),
            TlProjectionLimit::Bytes
            | TlProjectionLimit::Nodes
            | TlProjectionLimit::Positions
            | TlProjectionLimit::Valuations => unreachable!("join only visits support and causes"),
        };
        if *used >= setting {
            return Err(limit(counter, setting));
        }
        *used += 1;
        Ok(())
    }
    fn support(&mut self) -> Result<(), TlJoinError> {
        self.charge(TlProjectionLimit::Support)?;
        self.charge(TlProjectionLimit::Work)
    }
    fn cause(&mut self) -> Result<(), TlJoinError> {
        self.charge(TlProjectionLimit::Causes)
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

/// Owner-read both supplied results and evaluate every decision row independently.
/// Neither this operation nor either reader it invokes executes a temporal evaluator.
pub fn join_tl<'a>(
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<&Report>,
    mapped: Option<&ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    limits: TlLimits,
    native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin<'a>, TlJoinError> {
    let mut budget = Budget::new(limits);
    build(
        projection,
        native,
        mapped,
        previous,
        &mut budget,
        native_budget,
    )
}

/// Strict-read one exact expected join, including every source and predecessor value.
/// Comparing complete expected canonical bytes rejects every alternate field spelling,
/// unknown/duplicate/missing/order violation, trailing input or cross-wired source.
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
        &mut budget,
        native_budget,
    )?;
    if result.bytes != bytes {
        return Err(TlJoinError::ExpectedMismatch);
    }
    Ok(result)
}

fn final_truth(truth: Truth, settlement: Option<Settlement>) -> Option<bool> {
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
        Truth::True => Some(true),
        Truth::False => Some(false),
        Truth::Pending => None,
    }
}

fn source_result(mapped: Option<&ValidatedMappedResult>) -> Option<&ValidatedTemporalResult> {
    mapped.map(ValidatedMappedResult::source)
}

fn build<'a>(
    projection: &'a ValidatedTlProjection<'a>,
    native: Option<&Report>,
    mapped: Option<&ValidatedMappedResult>,
    previous: Option<&'a ValidatedTlJoin<'a>>,
    budget: &mut Budget,
    native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin<'a>, TlJoinError> {
    let native_limits = projection.selection().native_report_limits;
    let native_bytes = native
        .map(|report| {
            report
                .to_owner_bytes_bounded(native_limits)
                .map(|(bytes, usage)| {
                    budget.native_encoding_usage = Some(usage);
                    bytes
                })
                .map_err(TlJoinError::NativeReport)
        })
        .transpose()?;
    if let Some(bytes) = &native_bytes {
        budget.bytes(bytes.len())?;
    }
    if let Some(result) = mapped {
        budget.bytes(result.bytes().len())?;
        budget.bytes(result.source().bytes().len())?;
    }
    let input = projection.native();
    let mut refusals = TlJoinOwnerRefusals {
        native: None,
        native_predecessor: None,
        mapped: None,
    };
    let mut predecessor_conflict = false;
    let prior_native = previous
        .filter(|join| join.native_admitted())
        .and_then(ValidatedTlJoin::native);
    if let Some(current) = native.filter(|report| report.predecessor().is_some()) {
        if let Some(prior) = prior_native {
            match current.validate_predecessor(prior, native_limits) {
                Ok(usage) => budget.native_predecessor_usage = Some(usage),
                Err(error @ ReportRefusal::InvalidPredecessor) => {
                    predecessor_conflict = true;
                    refusals.native_predecessor = Some(error);
                }
                Err(error) => return Err(TlJoinError::NativeReport(error)),
            }
        } else {
            predecessor_conflict = true;
        }
    }
    let read_native = match native_bytes.as_deref() {
        None => None,
        Some(_) if predecessor_conflict => None,
        Some(bytes) => {
            let read = quire_spec_language::temporal::CheckedSubject::new(
                projection.package(),
                input.occurrence,
                input.valuations,
                input.request,
            )
            .and_then(|subject| {
                let subject = match (native, prior_native) {
                    (Some(current), Some(prior)) if current.predecessor().is_some() => {
                        subject.with_predecessor(prior, native_limits)
                    }
                    _ => subject,
                };
                Report::from_owner_bytes(bytes, &subject, input.observation, native_budget)
            });
            match read {
                Ok(result) => Some(result),
                Err(error) => {
                    refusals.native = Some(error);
                    None
                }
            }
        }
    };
    let read_mapped = match mapped {
        None => None,
        Some(result) => {
            let selection = quire_mltl::contract_ir::MappingSelection::for_result(result.source());
            match quire_mltl::contract_ir::read(
                result.bytes(),
                result.source(),
                &selection,
                projection.selection().owner_limits,
            ) {
                Ok(result) => Some(result),
                Err(error) => {
                    refusals.mapped = Some(error);
                    None
                }
            }
        }
    };
    let compared = compare::rows(
        projection,
        read_native.as_ref(),
        read_mapped.as_ref(),
        native.is_none() || mapped.is_none(),
        &refusals,
        predecessor_conflict,
        previous,
        budget,
    )?;
    let bytes = wire::encode(
        projection,
        native_bytes.as_deref(),
        read_native.is_some(),
        mapped,
        previous,
        compared.decision,
        &compared.causes,
        compared.value,
        budget,
    )?;
    Ok(ValidatedTlJoin {
        projection,
        native: native.cloned(),
        native_admitted: read_native.is_some(),
        mapped: mapped.cloned(),
        previous,
        refusals,
        decision: compared.decision,
        causes: compared.causes,
        value: compared.value,
        bytes,
        usage: budget.usage,
        native_encoding_usage: budget.native_encoding_usage,
        native_predecessor_usage: budget.native_predecessor_usage,
    })
}
