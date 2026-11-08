// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-022: atomic projection through the original native and TL owner APIs.

use qsl_eval::temporal::evidence::{AuthenticatedBinding, BindingMismatch, EvidenceRequest};
use qsl_package::checked_views::{Classification, ClauseOccurrence, Closure, TemporalSubjectView};
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2};
use quire_mltl::request::{
    self, ObservationInputs, RequestInput, TemporalInput, ValidatedTemporalRequest,
};
use quire_observation::authority::clock;
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
mod positions;
mod record;

pub(in crate::temporal) use record::{hex, hex_string};

/// Public bridge profile name supplied directly to the request owner.
pub const PROFILE: &str = "quire.native-temporal-correspondence/v1";

/// Original native owner values for one selected occurrence and instance.
#[derive(Clone, Copy)]
pub struct NativeProjectionInput<'a> {
    /// The actual checked-package native request, never an evaluator callback.
    pub request: &'a EvidenceRequest,
    /// Exact authored occurrence selected by the request.
    pub occurrence: &'a ClauseOccurrence,
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

/// The seven caller ceilings of the FR-350 bridge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlLimits {
    /// Canonical bytes per owner record.
    pub max_bytes: u64,
    /// Formula occurrences.
    pub max_nodes: u64,
    /// Selected positions.
    pub max_positions: u64,
    /// Native request valuation entries.
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
    /// A valuation row failed an owner admission check.
    Valuation,
    /// Native clock authentication refused.
    NativeClock,
    /// Formula construction or admission refused.
    Formula,
    /// A TL owner producer or strict reader refused.
    Owner,
    /// The canonical writer refused the correspondence record.
    Encoding,
    /// Supplied record bytes differ from the reconstructed record.
    CorrespondenceMismatch,
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
            Self::Encoding => "tl-projection-encoding",
            Self::CorrespondenceMismatch => "tl-projection-correspondence-mismatch",
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
            Self::Encoding,
            Self::CorrespondenceMismatch,
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
    /// A control outside the subject's requirements.
    Control,
    /// Exact authored occurrence name.
    SubjectName,
    /// The request's clock cover is not exact.
    ClockCover,
    /// The request's correspondence reference is not the bridge definition.
    CorrespondenceReference,
    /// Opaque anchor equality and its admitted numeric position.
    Anchor,
    /// Native activation contradiction or refused fault.
    Activation,
    /// A declared capture has another name, type or anchor.
    Capture,
    /// Actual observation revision differs from the bundle.
    ObservationRevision,
    /// Two bundle records share one observation identity.
    Observation,
    /// Position record, range or order differs from admitted owner facts.
    Position,
    /// Source clock and observation clock differ.
    Clock,
    /// Owner number domain cannot encode the selected exact clock.
    ClockRepresentation,
    /// Independent decision closure differs.
    DecisionClosure,
    /// Independent surrounding closure differs.
    SurroundingClosure,
    /// Native progress differs from the selected owner boundary.
    Progress,
    /// Completeness facts contradict the supplied owner state.
    Completeness,
    /// No unique source-map entry matches the selected occurrence.
    Source,
    /// Original interval cannot be represented in the selected formula domain.
    Interval,
}

/// Named missing facts; none authorizes an implicit Boolean value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TlMissingPremise {
    /// Authoritative complete origin history is absent.
    History,
    /// A required position or fixed sample is absent.
    Position,
    /// A selected anchor is absent.
    Anchor,
    /// A selected semantic instance is absent or inactive.
    Instance,
    /// The retained activation carries a missing-input fault.
    Activation,
    /// A required capture is missing, null or stale.
    Capture,
    /// Input completeness is not established.
    CompleteInput,
    /// A selected observation is absent from the genuine owner bundle.
    Observation,
    /// No agreed valuation of the operand at the request position.
    Valuation {
        /// Request position index.
        position: usize,
        /// Original checked predicate node.
        operand: CheckedNodeId,
    },
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
    /// A request map names an atom outside the checked formula.
    #[error("projection valuation at position {position} names unknown atom {atom}")]
    Valuation {
        /// Original request position index.
        position: usize,
        /// Unknown native atom index.
        atom: u64,
    },
    /// Original native owner authentication failure.
    #[error("native clock authentication refused: {0}")]
    NativeClock(#[source] BindingMismatch),
    /// Original mechanical formula component failure.
    #[error("TL formula construction refused: {0}")]
    Formula(#[source] TlFormulaError),
    /// Original typed TL producer/reader failure.
    #[error("TL owner admission refused: {0}")]
    Owner(#[source] Box<OwnerReadError>),
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
    /// An IR value in the record holds a non-integer JSON number.
    #[error("correspondence encoding refused a non-integer number")]
    NonIntegerNumber,
    /// Original canonical writer refusal.
    #[error("correspondence encoding refused: {0}")]
    Canonical(#[source] quire_canonical::Error),
    /// Supplied record bytes differ from the reconstructed record.
    #[error("projection correspondence differs from the complete expected value")]
    Correspondence,
    /// First excess unit, rejected before the action it would pay for.
    #[error("projection {limit:?} limit reached: {value} > {setting}")]
    LimitReached {
        /// Caller counter.
        limit: TlProjectionLimit,
        /// First excess value, always `setting + 1`.
        value: u64,
        /// Effective caller setting.
        setting: u64,
        /// The owner refusal when an owner stopped at a lowered byte ceiling.
        #[source]
        owner: Option<Box<OwnerReadError>>,
    },
}

impl TlProjectionError {
    /// Stable machine discriminator; callers need not branch on error text.
    pub const fn code(&self) -> TlProjectionCode {
        match self {
            Self::Binding(_) => TlProjectionCode::Binding,
            Self::Unsupported(_) => TlProjectionCode::Unsupported,
            Self::Incomplete(_) => TlProjectionCode::Incomplete,
            Self::NativePremise(_) => TlProjectionCode::NativePremise,
            Self::Valuation { .. } => TlProjectionCode::Valuation,
            Self::NativeClock(_) => TlProjectionCode::NativeClock,
            Self::Formula(_) => TlProjectionCode::Formula,
            Self::Owner(_)
            | Self::PropositionMap(_)
            | Self::SyntaxRead(_)
            | Self::SyntaxEncoding(_)
            | Self::History(_)
            | Self::ExactNumber(_) => TlProjectionCode::Owner,
            Self::NonIntegerNumber | Self::Canonical(_) => TlProjectionCode::Encoding,
            Self::Correspondence => TlProjectionCode::CorrespondenceMismatch,
            Self::LimitReached { .. } => TlProjectionCode::LimitReached,
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

impl std::fmt::Debug for ValidatedTlProjection<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedTlProjection")
            .field("anchor", &self.anchor)
            .field("positions", &self.positions)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
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
    fn new(limits: TlLimits) -> Self {
        Self {
            limits,
            usage: TlProjectionUsage::default(),
        }
    }

    /// Record one checked byte value; the usage is the largest checked length.
    fn bytes(&mut self, length: usize) -> Result<(), TlProjectionError> {
        let value = u64::try_from(length).unwrap_or(u64::MAX);
        if value > self.limits.max_bytes {
            return Err(limit(TlProjectionLimit::Bytes, self.limits.max_bytes));
        }
        self.usage.bytes = self.usage.bytes.max(value);
        Ok(())
    }

    /// Charge `count` units to `counter` (when named) and to work, before the
    /// units are visited. The counter precedes work in FR-350's table order.
    fn charge(
        &mut self,
        count: usize,
        counter: Option<TlProjectionLimit>,
    ) -> Result<(), TlProjectionError> {
        let count = u64::try_from(count).unwrap_or(u64::MAX);
        if let Some(counter) = counter {
            let (used, setting) = match counter {
                TlProjectionLimit::Nodes => (&mut self.usage.nodes, self.limits.max_nodes),
                TlProjectionLimit::Positions => {
                    (&mut self.usage.positions, self.limits.max_positions)
                }
                TlProjectionLimit::Valuations => {
                    (&mut self.usage.valuations, self.limits.max_valuations)
                }
                _ => unreachable!("projection charges only nodes, positions and valuations"),
            };
            if used.saturating_add(count) > setting {
                return Err(limit(counter, setting));
            }
            *used += count;
        }
        if self.usage.work.saturating_add(count) > self.limits.max_work {
            return Err(limit(TlProjectionLimit::Work, self.limits.max_work));
        }
        self.usage.work += count;
        Ok(())
    }
}

fn limit(counter: TlProjectionLimit, setting: u64) -> TlProjectionError {
    TlProjectionError::LimitReached {
        limit: counter,
        value: setting.saturating_add(1),
        setting,
        owner: None,
    }
}

/// Reject any interval the TL formula domain cannot preserve.
fn check_intervals(
    subject: &TemporalSubjectView<'_>,
    budget: &mut Budget,
) -> Result<(), TlProjectionError> {
    use qsl_package::checked_views::FormulaOperation as Op;
    use quire_contract_ir::CheckedOperationInterval as Interval;
    budget.charge(subject.formula_graph().len(), None)?;
    for node in subject.formula_graph() {
        if !matches!(
            node.kind(),
            Some(
                Op::Eventually
                    | Op::Always
                    | Op::Once
                    | Op::Historically
                    | Op::Until
                    | Op::Release
                    | Op::Since
                    | Op::Triggered
            )
        ) {
            continue;
        }
        let representable = match &node.interval {
            Some(Interval::Integer {
                lower,
                upper: Some(upper),
            }) => u32::try_from(lower).is_ok() && u32::try_from(upper).is_ok(),
            Some(Interval::Integer { upper: None, .. } | Interval::Timed { .. }) | None => false,
        };
        if !representable {
            return Err(TlProjectionError::NativePremise(
                TlProjectionPremise::Interval,
            ));
        }
    }
    Ok(())
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
    project_with(
        package,
        subject,
        classification,
        surrounding,
        native,
        selection,
        Budget::new(limits),
    )
}

fn project_with<'a>(
    package: &'a CheckedPackageV2,
    subject: &TemporalSubjectView<'a>,
    classification: &Classification,
    surrounding: Closure,
    native: NativeProjectionInput<'a>,
    selection: TlProjectionSelection<'a>,
    mut budget: Budget,
) -> Result<ValidatedTlProjection<'a>, TlProjectionError> {
    let limits = budget.limits;
    let facts = native::check(
        package,
        subject,
        classification,
        surrounding,
        native,
        selection,
        &mut budget,
    )?;
    budget.charge(package.source_map().len(), None)?;
    let source = positions::exactly_one(
        package.source_map().iter().filter(|entry| {
            entry.node_id == native.occurrence.node
                && entry.role == native.occurrence.role
                && entry.ordinal == native.occurrence.ordinal
        }),
        TlProjectionError::NativePremise(TlProjectionPremise::Source),
    )?;
    check_intervals(subject, &mut budget)?;
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
    let owner_error = |error| owners::error(error, selection.owner_limits, limits.max_bytes);
    let document = request::derive(request_input, owner_limits).map_err(owner_error)?;
    budget.bytes(document.bytes().len())?;
    let request =
        request::read(document.bytes(), request_input, owner_limits).map_err(owner_error)?;
    let bytes = record::encode(
        package,
        subject,
        surrounding,
        native,
        selection,
        &facts,
        source,
        &built,
        &request,
        &mut budget,
    )?;
    Ok(ValidatedTlProjection {
        package,
        bytes,
        source,
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
    let mut budget = Budget::new(limits);
    budget.bytes(bytes.len())?;
    let projection = project_with(
        expected.package,
        expected.subject,
        expected.classification,
        expected.surrounding,
        expected.native,
        expected.selection,
        budget,
    )?;
    if bytes != projection.bytes() {
        return Err(TlProjectionError::Correspondence);
    }
    Ok(projection)
}

/// Owner limits with every byte ceiling lowered to the bridge's `max_bytes`.
pub(in crate::temporal) fn lowered_owner_limits(
    original: OwnerLimits,
    max_bytes: u64,
) -> OwnerLimits {
    owners::limits(original, max_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_refusal_maps_to_its_fr_350_code() {
        let cases = [
            (
                TlProjectionError::NativePremise(TlProjectionPremise::SubjectName),
                TlProjectionCode::NativePremise,
            ),
            (
                TlProjectionError::NativePremise(TlProjectionPremise::Source),
                TlProjectionCode::NativePremise,
            ),
            (
                TlProjectionError::Incomplete(TlMissingPremise::History),
                TlProjectionCode::Incomplete,
            ),
            (
                TlProjectionError::NonIntegerNumber,
                TlProjectionCode::Encoding,
            ),
            (
                TlProjectionError::Correspondence,
                TlProjectionCode::CorrespondenceMismatch,
            ),
        ];
        for (error, code) in cases {
            assert_eq!(error.code(), code, "{error:?}");
        }
        assert_eq!(TlProjectionCode::all().len(), 11);
    }
}
