// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-022: atomic projection through the original native and TL owner APIs.

use qsl_eval::temporal::evidence::{AuthenticatedBinding, BindingMismatch, EvidenceRequest};
use qsl_package::checked_views::{Classification, ClauseOccurrence, Closure, TemporalSubjectView};
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2};
use quire_mltl::request::{
    self, ObservationInputs, RequestInput, TemporalInput, ValidatedTemporalRequest,
};
use quire_observation::authority::clock;
use quire_spec_language::temporal::{
    ObservationRef, ValuationCause, ValuationEvidence, ValuationInstance,
};
use tl_mltl::{
    past::history::ValidatedPositionHistory,
    wire::{OwnerLimits, OwnerReadError, ValidatedTrace},
};
use tl_syntax::PropositionMapDocument;

use super::{
    AdmittedObservationRevision, MappingBindingRefusal, TlFormulaComponent, TlFormulaError,
};

mod native;
mod owners;
mod record;

/// Public bridge profile name supplied directly to the request owner.
pub const PROFILE: &str = "quire.native-temporal-correspondence/v1";

/// Original native owner values for one selected occurrence and instance.
#[derive(Clone, Copy)]
pub struct NativeProjectionInput<'a> {
    /// The actual checked-package native request, never an evaluator callback.
    pub request: &'a EvidenceRequest,
    /// Ordered rows admitted by the native valuation owner.
    pub valuations: &'a ValuationEvidence,
    /// Exact authored occurrence selected by the request.
    pub occurrence: &'a ClauseOccurrence,
    /// The native row admission's explicit anchor and immutable capture instance.
    pub instance: ValuationInstance<'a>,
    /// Native reference to the actual supplied observation revision.
    pub observation: &'a ObservationRef,
}

/// Independently selected genuine observation and TL owner inputs.
#[derive(Clone, Copy)]
pub struct TlProjectionSelection<'a> {
    /// Actual strict admission to the original integration-owned bridge definition.
    pub bridge_definition: &'a super::ValidatedTlBridgeDefinition,
    /// The actual admitted observation revision, including the complete bundle.
    pub observation: &'a AdmittedObservationRevision<'a>,
    /// Independently admitted clock binding.
    pub clock: &'a clock::View,
    /// Independent progress, closure, completeness and availability views.
    pub observations: ObservationInputs<'a>,
    /// Finite original limits passed to the TL producers and strict readers.
    pub owner_limits: OwnerLimits,
}

/// Caller ceilings of the FR-350 bridge. Join-only counters remain unused here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlLimits {
    /// Canonical bytes per owner record.
    pub max_bytes: u64,
    /// Formula occurrences.
    pub max_nodes: u64,
    /// Selected positions.
    pub max_positions: u64,
    /// Projected Boolean cells.
    pub max_valuations: u64,
    /// Compared decision-support entries (the later join).
    pub max_support: u64,
    /// Returned ordered join causes (the later join).
    pub max_causes: u64,
    /// Bridge visits, excluding work internal to the original owners.
    pub max_work: u64,
}

impl Default for TlLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16_777_216,
            max_nodes: 65_536,
            max_positions: 1_048_576,
            max_valuations: 16_777_216,
            max_support: 1_048_576,
            max_causes: 64,
            max_work: 4_294_967_296,
        }
    }
}

/// Closed caller counter vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlProjectionLimit {
    /// Bytes of one generated or read owner record.
    Bytes,
    /// Formula occurrences.
    Nodes,
    /// Positions.
    Positions,
    /// Boolean cells.
    Valuations,
    /// Decision-support entries.
    Support,
    /// Ordered causes.
    Causes,
    /// Bridge visits.
    Work,
}

impl TlProjectionLimit {
    /// Exact public caller-setting name.
    pub const fn setting(self) -> &'static str {
        match self {
            Self::Bytes => "temporal.correspondence.max_bytes",
            Self::Nodes => "temporal.correspondence.max_nodes",
            Self::Positions => "temporal.correspondence.max_positions",
            Self::Valuations => "temporal.correspondence.max_valuations",
            Self::Support => "temporal.correspondence.max_support",
            Self::Causes => "temporal.correspondence.max_causes",
            Self::Work => "temporal.correspondence.max_work",
        }
    }
}

/// Actual bridge work of one atomic projection, not an owner-internal receipt.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TlProjectionUsage {
    /// Largest owner record admitted or produced.
    pub bytes: u64,
    /// Formula occurrences constructed.
    pub nodes: u64,
    /// Distinct selected positions.
    pub positions: u64,
    /// Explicit generated Boolean cells, false included.
    pub valuations: u64,
    /// Compared support entries; zero before result joining.
    pub support: u64,
    /// Emitted ordered join causes; zero before result joining.
    pub causes: u64,
    /// Component visits actually charged before their actions.
    pub work: u64,
}

/// Stable projection discriminator; original owner errors remain typed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlProjectionCode {
    /// Package, subject or classification admission differs.
    Binding,
    /// QSL classifies the selected subject as unsupported.
    Unsupported,
    /// A required input fact is missing.
    Incomplete,
    /// Native input facts disagree.
    NativePremise,
    /// An explicit Boolean cell is duplicate or foreign.
    Valuation,
    /// Native clock authentication refused.
    NativeClock,
    /// Formula construction or admission refused.
    Formula,
    /// A TL owner producer or strict reader refused.
    Owner,
    /// A caller ceiling refused the next action.
    LimitReached,
}

impl TlProjectionCode {
    /// Stable machine spelling, independent of human error text.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Binding => "tl-projection-binding",
            Self::Unsupported => "tl-projection-unsupported",
            Self::Incomplete => "tl-projection-incomplete",
            Self::NativePremise => "tl-projection-native-premise",
            Self::Valuation => "tl-projection-valuation",
            Self::NativeClock => "tl-projection-native-clock",
            Self::Formula => "tl-projection-formula",
            Self::Owner => "tl-projection-owner",
            Self::LimitReached => "tl-projection-limit-reached",
        }
    }

    /// Complete closed catalog.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Binding,
            Self::Unsupported,
            Self::Incomplete,
            Self::NativePremise,
            Self::Valuation,
            Self::NativeClock,
            Self::Formula,
            Self::Owner,
            Self::LimitReached,
        ]
    }

    /// Parse one exact public spelling; aliases are not accepted.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// Closed native correspondence dimensions checked by this bridge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlProjectionPremise {
    /// Exact declared occurrence.
    Occurrence,
    /// Exact authored occurrence name.
    SubjectName,
    /// Opaque anchor equality and its admitted numeric position.
    Anchor,
    /// Selected native semantic instance.
    Instance,
    /// Actual observation revision and original owner data.
    ObservationRevision,
    /// Position reference, range or order differs from admitted owner facts.
    Position,
    /// Actual source clock and selected observation clock differ.
    Clock,
    /// Original interval cannot be represented in the selected formula domain.
    Interval,
    /// Owner number domain cannot encode the selected exact clock.
    ClockRepresentation,
    /// Independent decision closure differs.
    DecisionClosure,
    /// Independent surrounding closure differs.
    SurroundingClosure,
    /// A declared capture has another type/anchor/instance.
    Capture,
    /// Equal semantic deliveries disagree on their immutable facts.
    CaptureConflict,
    /// Native progress differs from the selected owner boundary.
    Progress,
    /// Completeness facts contradict the supplied owner state.
    Completeness,
}

/// Named missing facts; none authorizes an implicit Boolean value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlMissingPremise {
    /// Authoritative complete origin history is absent.
    History,
    /// A required position or fixed sample is absent.
    Position,
    /// A selected anchor is absent.
    Anchor,
    /// A selected semantic instance is absent or inactive.
    Instance,
    /// A required capture is missing, null or stale.
    Capture,
    /// Input completeness is not established.
    CompleteInput,
    /// A selected observation is absent from the genuine owner bundle.
    Observation,
}

/// Atomic non-value: no variant exposes a partial admitted projection.
#[derive(Debug, thiserror::Error)]
pub enum TlProjectionError {
    /// Original mapping admission refusal.
    #[error("projection mapping refused: {0:?}")]
    Binding(MappingBindingRefusal),
    /// Every original unsupported dimension is retained.
    #[error("the selected native subject has no supported TL mapping")]
    Unsupported(Box<Classification>),
    /// Named missing premise; a missing value is never false.
    #[error("projection input is incomplete: {0:?}")]
    Incomplete(TlMissingPremise),
    /// Named cross-wired native premise.
    #[error("projection native premise differs: {0:?}")]
    NativePremise(TlProjectionPremise),
    /// A row does not belong to this exact requested cell population.
    #[error("projection valuation row {row} differs in {cause:?}")]
    Valuation {
        /// Original admitted row ordinal.
        row: usize,
        /// Exact failing binding dimension.
        cause: ValuationCause,
    },
    /// Duplicate native rows remain distinguishable, even when they agree.
    #[error("projection valuation row {duplicate} duplicates row {first}")]
    DuplicateValuation {
        /// Original cell's admitted row ordinal.
        first: usize,
        /// Repeated cell's admitted row ordinal.
        duplicate: usize,
    },
    /// Original native owner authentication failure.
    #[error("native clock authentication refused: {0}")]
    NativeClock(#[source] BindingMismatch),
    /// Original mechanical formula component failure.
    #[error("TL formula construction refused: {0}")]
    Formula(#[source] TlFormulaError),
    /// Original typed TL producer/reader failure.
    #[error("TL owner admission refused: {0}")]
    Owner(#[source] OwnerReadError),
    /// Actual owner refusal at a byte ceiling lowered by this caller.
    #[error("projection {limit:?} limit reached: {value} > {setting}: {error}")]
    OwnerLimitReached {
        /// Original owner refusal, including its complete usage.
        #[source]
        error: OwnerReadError,
        /// Effective bridge counter.
        limit: TlProjectionLimit,
        /// First excess unit.
        value: u64,
        /// Original caller setting.
        setting: u64,
    },
    /// Preserve original syntax-owner refusals without a copied wire record.
    #[error("TL proposition map refused: {0}")]
    PropositionMap(tl_syntax::PropositionMapError),
    /// Original strict syntax reader failure.
    #[error("TL syntax reader refused: {0}")]
    SyntaxRead(tl_syntax::StrictDocumentReadError),
    /// Original owner canonical encoding failure.
    #[error("TL syntax encoding refused: {0}")]
    SyntaxEncoding(#[source] serde_json::Error),
    /// Original position-history constructor failure.
    #[error("TL history constructor refused: {0}")]
    History(#[source] tl_mltl::HistoryError),
    /// Original exact-number constructor failure.
    #[error("TL clock number refused: {0}")]
    ExactNumber(#[source] tl_mltl::ExactNumberError),
    /// An explicit cell is absent, never an implicit false.
    #[error("no admitted valuation at position {position} for {operand:?}")]
    MissingValuation {
        /// Native position ordinal.
        position: usize,
        /// Original checked predicate node.
        operand: CheckedNodeId,
    },
    /// Original canonical writer refusal.
    #[error("correspondence encoding refused: {0}")]
    Canonical(#[source] quire_canonical::Error),
    /// Complete expected correspondence differs, including order or canonical spelling.
    #[error("projection correspondence differs from the complete expected value")]
    Correspondence,
    /// First excess unit, rejected before the action it would pay for.
    #[error("projection {limit:?} limit reached: {value} > {setting}")]
    LimitReached {
        /// Caller counter.
        limit: TlProjectionLimit,
        /// First excess value.
        value: u64,
        /// Effective caller setting.
        setting: u64,
    },
}

impl TlProjectionError {
    /// Stable machine discriminator; callers need not branch on error text.
    pub const fn code(&self) -> TlProjectionCode {
        match self {
            Self::Binding(_) => TlProjectionCode::Binding,
            Self::Unsupported(_) => TlProjectionCode::Unsupported,
            Self::Incomplete(_) | Self::MissingValuation { .. } => TlProjectionCode::Incomplete,
            Self::NativePremise(_) | Self::Correspondence => TlProjectionCode::NativePremise,
            Self::Valuation { .. } | Self::DuplicateValuation { .. } => TlProjectionCode::Valuation,
            Self::NativeClock(_) => TlProjectionCode::NativeClock,
            Self::Formula(_) => TlProjectionCode::Formula,
            Self::Owner(_)
            | Self::Canonical(_)
            | Self::PropositionMap(_)
            | Self::SyntaxRead(_)
            | Self::SyntaxEncoding(_)
            | Self::History(_)
            | Self::ExactNumber(_) => TlProjectionCode::Owner,
            Self::LimitReached { .. } => TlProjectionCode::LimitReached,
            Self::OwnerLimitReached { .. } => TlProjectionCode::LimitReached,
        }
    }
}

/// Actual TL-owned input, never a bridge copy of its schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TlInput {
    /// Strictly admitted finite trace.
    Future(ValidatedTrace),
    /// Strictly admitted complete position history.
    Past(ValidatedPositionHistory),
}

impl TlInput {
    /// Exact immutable bytes returned by the original input reader.
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Future(trace) => trace.bytes(),
            Self::Past(history) => history.bytes(),
        }
    }

    fn request_input(&self) -> TemporalInput<'_> {
        match self {
            Self::Future(trace) => TemporalInput::Future(trace),
            Self::Past(history) => TemporalInput::Past(history),
        }
    }
}

/// Constructor-private projection retaining the original owner values and facts.
pub struct ValidatedTlProjection<'a> {
    package: &'a CheckedPackageV2,
    bytes: Vec<u8>,
    source: &'a quire_contract_ir::CheckedSourceMapEntry,
    formula: TlFormulaComponent,
    propositions: PropositionMapDocument,
    input: TlInput,
    request: ValidatedTemporalRequest,
    native: NativeProjectionInput<'a>,
    selection: TlProjectionSelection<'a>,
    binding: AuthenticatedBinding,
    positions: Vec<usize>,
    anchor: u64,
    usage: TlProjectionUsage,
}

impl<'a> ValidatedTlProjection<'a> {
    /// Original admitted package; its retained admission is checked before facts.
    pub fn package(&self) -> &'a CheckedPackageV2 {
        self.package
    }
    /// Complete canonical bridge correspondence, retaining original owner bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Original admitted source occurrence and all exact half-open source spans.
    pub fn source(&self) -> &'a quire_contract_ir::CheckedSourceMapEntry {
        self.source
    }
    /// Actual owner-admitted formula and original predicate relation.
    pub fn formula(&self) -> &TlFormulaComponent {
        &self.formula
    }
    /// Actual strict-read proposition map.
    pub fn propositions(&self) -> &PropositionMapDocument {
        &self.propositions
    }
    /// Actual strict-read trace or history.
    pub fn input(&self) -> &TlInput {
        &self.input
    }
    /// Exact request returned by the real owner reader.
    pub fn request(&self) -> &ValidatedTemporalRequest {
        &self.request
    }
    /// Original native owner inputs, including their independent axes.
    pub fn native(&self) -> NativeProjectionInput<'a> {
        self.native
    }
    /// Original independently supplied genuine owner selections.
    pub fn selection(&self) -> TlProjectionSelection<'a> {
        self.selection
    }
    /// Native owner's authenticated profile/clock binding.
    pub fn binding(&self) -> &AuthenticatedBinding {
        &self.binding
    }
    /// Original native row index at each ordered TL position.
    pub fn source_positions(&self) -> &[usize] {
        &self.positions
    }
    /// Numeric TL anchor established from the actual native observation reference.
    pub fn anchor(&self) -> u64 {
        self.anchor
    }
    /// This operation's actual bridge counters.
    pub fn usage(&self) -> TlProjectionUsage {
        self.usage
    }
}

struct Budget {
    limits: TlLimits,
    usage: TlProjectionUsage,
}

impl Budget {
    fn work(&mut self) -> Result<(), TlProjectionError> {
        Self::charge(
            &mut self.usage.work,
            self.limits.max_work,
            TlProjectionLimit::Work,
        )
    }
    fn position(&mut self) -> Result<(), TlProjectionError> {
        Self::charge(
            &mut self.usage.positions,
            self.limits.max_positions,
            TlProjectionLimit::Positions,
        )?;
        self.work()
    }
    fn cell(&mut self) -> Result<(), TlProjectionError> {
        Self::charge(
            &mut self.usage.valuations,
            self.limits.max_valuations,
            TlProjectionLimit::Valuations,
        )?;
        self.work()
    }
    fn bytes(&mut self, length: usize) -> Result<(), TlProjectionError> {
        let value = u64::try_from(length).unwrap_or(u64::MAX);
        if value > self.limits.max_bytes {
            return Err(limit(TlProjectionLimit::Bytes, self.limits.max_bytes));
        }
        self.usage.bytes = self.usage.bytes.max(value);
        Ok(())
    }
    fn charge(
        used: &mut u64,
        setting: u64,
        counter: TlProjectionLimit,
    ) -> Result<(), TlProjectionError> {
        if *used >= setting {
            return Err(limit(counter, setting));
        }
        *used += 1;
        Ok(())
    }
}

fn limit(counter: TlProjectionLimit, setting: u64) -> TlProjectionError {
    TlProjectionError::LimitReached {
        limit: counter,
        value: setting.saturating_add(1),
        setting,
    }
}

/// Project one actual native instance atomically through the selected owner readers.
/// No evaluator, callback, mirror record or content-derived identity is used.
pub fn project_tl<'a>(
    package: &'a CheckedPackageV2,
    subject: &TemporalSubjectView<'a>,
    classification: &Classification,
    surrounding: Closure,
    native: NativeProjectionInput<'a>,
    selection: TlProjectionSelection<'a>,
    limits: TlLimits,
) -> Result<ValidatedTlProjection<'a>, TlProjectionError> {
    let mut budget = Budget {
        limits,
        usage: TlProjectionUsage::default(),
    };
    let facts = native::check(
        package,
        subject,
        classification,
        surrounding,
        native,
        selection,
        &mut budget,
    )?;
    let built = owners::construct(
        package,
        subject,
        classification,
        surrounding,
        selection,
        &facts,
        &mut budget,
    )?;
    let request_input = RequestInput {
        formula: built.formula.formula(),
        proposition_map: &built.propositions,
        input: built.input.request_input(),
        clock: selection.clock,
        subject_identity: facts.name,
        correspondence_identity: PROFILE,
        anchor: facts.anchor,
        observations: selection.observations,
    };
    let owner_limits = owners::limits(selection.owner_limits, limits.max_bytes);
    let document = request::derive(request_input, owner_limits)
        .map_err(|error| owners::error(error, selection.owner_limits, limits.max_bytes))?;
    budget.bytes(document.bytes().len())?;
    let request = request::read(document.bytes(), request_input, owner_limits)
        .map_err(|error| owners::error(error, selection.owner_limits, limits.max_bytes))?;
    let bytes = record::encode(
        package,
        subject,
        surrounding,
        native,
        selection,
        &facts,
        &built,
        &request,
        &mut budget,
    )?;
    Ok(ValidatedTlProjection {
        package,
        bytes,
        source: subject.source_span(),
        formula: built.formula,
        propositions: built.propositions,
        input: built.input,
        request,
        native,
        selection,
        binding: facts.binding,
        positions: facts.positions,
        anchor: facts.anchor,
        usage: budget.usage,
    })
}

/// Independent original owner inputs used to reconstruct the complete expected value.
pub struct ExpectedTlProjection<'a, 's> {
    /// Original checked admission.
    pub package: &'a CheckedPackageV2,
    /// View borrowed from that admission.
    pub subject: &'s TemporalSubjectView<'a>,
    /// Original authenticated classification.
    pub classification: &'s Classification,
    /// Independent surrounding-execution closure.
    pub surrounding: Closure,
    /// Original native owner facts.
    pub native: NativeProjectionInput<'a>,
    /// Independently selected owner inputs.
    pub selection: TlProjectionSelection<'a>,
}

/// Strictly reconstruct and compare complete canonical correspondence without evaluation.
/// Exact expected bytes reject unknown, duplicate, missing, reordered and trailing members.
pub fn read_tl_projection<'a>(
    bytes: &[u8],
    expected: ExpectedTlProjection<'a, '_>,
    limits: TlLimits,
) -> Result<ValidatedTlProjection<'a>, TlProjectionError> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limits.max_bytes {
        return Err(limit(TlProjectionLimit::Bytes, limits.max_bytes));
    }
    let projection = project_tl(
        expected.package,
        expected.subject,
        expected.classification,
        expected.surrounding,
        expected.native,
        expected.selection,
        limits,
    )?;
    if bytes != projection.bytes() {
        return Err(TlProjectionError::Correspondence);
    }
    Ok(projection)
}
