// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Bounded deterministic semantic views and non-authoritative consumer accounting.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sha2::{Digest as _, Sha256};

use crate::ecosystem_lock::{AdmittedCapability, CapabilitySelection, LockIdentity, ValidatedLock};
use crate::{ArtifactId, DefinitionId, PackageId};

/// Exact checked-package reader contract admitted by the ecosystem lock.
pub const CHECKED_READER_CONTRACT: &str = "quire.checked-package-reader/v1";
/// Exact pure semantic-consumer contract admitted by the ecosystem lock.
pub const CONSUMER_CONTRACT: &str = "quire.semantic-consumer/v1";

/// Stable failure category for the consumer-view boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// An identity is empty or outside its byte ceiling.
    InvalidIdentity,
    /// A caller-lowered finite ceiling was exceeded.
    ResourceExhausted,
    /// Checked integer arithmetic or conversion failed.
    IntegerOverflow,
    /// Input is malformed, duplicated, dangling, or non-canonical.
    Malformed,
    /// Package, query, profile, or capability belongs to another selection.
    ForeignInput,
    /// The exact reader or consumer is not admitted by the lock.
    CapabilityMismatch,
    /// An owner reader failed after admission.
    ReaderFailure,
}

/// Exact exhausted resource dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resource {
    /// Input bytes.
    InputBytes,
    /// Semantic nodes.
    Nodes,
    /// Reader-declared source artifacts.
    SourceArtifacts,
    /// Semantic edges.
    Edges,
    /// Traversal expansions.
    TraversalSteps,
    /// Retained view results.
    Results,
    /// Retained omitted-frontier entries.
    Frontier,
    /// Requested consumer obligations.
    Obligations,
    /// Output mappings.
    Mappings,
    /// Canonical/output bytes.
    OutputBytes,
    /// Canonical view/accounting envelope bytes.
    AccountingBytes,
    /// Total invocation duration.
    Duration,
    /// Selected strict-reader invocations.
    ReaderInvocations,
}

/// Typed failure before an accountable run can be retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    code: ErrorCode,
    resource: Option<Resource>,
    detail: &'static str,
}

impl Error {
    fn new(code: ErrorCode, detail: &'static str) -> Self {
        Self {
            code,
            resource: None,
            detail,
        }
    }

    fn exhausted(resource: Resource, detail: &'static str) -> Self {
        Self {
            code: ErrorCode::ResourceExhausted,
            resource: Some(resource),
            detail,
        }
    }

    /// Stable failure category.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Exact exhausted dimension, when applicable.
    #[must_use]
    pub const fn resource(&self) -> Option<Resource> {
        self.resource
    }

    /// Bounded diagnostic text.
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for Error {}

/// Consumer-view result.
pub type Result<T> = std::result::Result<T, Error>;

/// Caller-lowered finite ceilings for Task 005.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum bytes in any feature-local identity.
    pub max_identity_bytes: u64,
    /// Maximum retained owner/input bytes.
    pub max_input_bytes: u64,
    /// Maximum semantic nodes.
    pub max_nodes: u64,
    /// Maximum source artifacts declared by the selected reader.
    pub max_source_artifacts: u64,
    /// Maximum edges retained from the checked-reader projection.
    pub max_projection_edges: u64,
    /// Maximum selected strict-reader invocations per query (currently zero or one).
    pub max_reader_invocations: u64,
    /// Maximum traversed selected-relation edges.
    pub max_edges: u64,
    /// Maximum canonical node expansions.
    pub max_traversal_steps: u64,
    /// Maximum retained displayed/omitted results.
    pub max_results: u64,
    /// Maximum exact immediate-frontier entries.
    pub max_frontier: u64,
    /// Maximum requested consumer obligations.
    pub max_obligations: u64,
    /// Maximum selected-consumer invocations per request (currently zero or one).
    pub max_consumer_invocations: u64,
    /// Maximum returned output mappings.
    pub max_mappings: u64,
    /// Maximum retained canonical/output bytes.
    pub max_output_bytes: u64,
    /// Maximum canonical view/accounting envelope bytes.
    pub max_accounting_bytes: u64,
    /// Maximum total duration for one public invocation.
    pub max_duration_ms: u64,
}

macro_rules! local_identity {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Constructs a bounded nonempty exact identity.
            pub fn new(value: impl AsRef<str>, max_bytes: u64) -> Result<Self> {
                let value = value.as_ref();
                let length = u64::try_from(value.len()).map_err(|_| {
                    Error::new(ErrorCode::IntegerOverflow, "identity length overflow")
                })?;
                if length > max_bytes || value.trim().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidIdentity,
                        "invalid bounded identity",
                    ));
                }
                Ok(Self(value.to_owned()))
            }

            /// Exact identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

local_identity!(SemanticId, "Qualified owner semantic identity.");
local_identity!(SourceRegionId, "Exact owner source-region identity.");
local_identity!(
    ObligationId,
    "Exact requested consumer-obligation identity."
);
local_identity!(ProfileId, "Exact versioned consumer-profile identity.");

/// Closed output-region encoding rule selected by the feature-local profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionEncoding {
    /// Output bytes and region endpoints must be valid UTF-8 boundaries.
    Utf8,
    /// Output bytes are binary; only half-open byte-range validity is required.
    Binary,
}

/// Exact source attribution for one semantic fact or relation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceRef {
    /// Immutable owner artifact.
    pub artifact: ArtifactId,
    /// Owner-defined region identity.
    pub region: SourceRegionId,
}

/// Qualified semantic identity admitted by an exact lock.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QualifiedSemanticId {
    /// Lock-selected semantic definition.
    pub definition: DefinitionId,
    /// Owner identity within that definition.
    pub semantic: SemanticId,
}

/// Closed typed relation vocabulary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationKind {
    /// Semantic containment.
    Contains,
    /// Explicit semantic reference.
    References,
    /// Declared dependency.
    DependsOn,
    /// Explicit refinement.
    Refines,
}

impl RelationKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::References => "references",
            Self::DependsOn => "depends-on",
            Self::Refines => "refines",
        }
    }
}

/// Exact versioned relation selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationSelection {
    /// Must equal `quire.semantic-relation/v1`.
    pub contract_version: String,
    /// Selected relation kind.
    pub kind: RelationKind,
}

/// One immutable semantic fact from an owner projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticNode {
    /// Qualified semantic identity.
    pub id: QualifiedSemanticId,
    /// Exact source attribution.
    pub source: SourceRef,
}

/// One immutable typed semantic edge from an owner projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticEdge {
    /// Edge source.
    pub source: QualifiedSemanticId,
    /// Typed relation.
    pub relation: RelationKind,
    /// Edge target.
    pub target: QualifiedSemanticId,
    /// Exact source attribution for the relation.
    pub provenance: SourceRef,
}

/// Exact immutable owner package presented to a selected strict reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerPackage {
    /// Exact owner wire contract.
    pub contract_version: String,
    /// Lock-selected package.
    pub package: PackageId,
    /// Immutable owner artifact.
    pub artifact: ArtifactId,
    /// Owner canonical bytes.
    pub canonical_bytes: Vec<u8>,
    /// Raw SHA-256 of `canonical_bytes`.
    pub digest: [u8; 32],
}

/// Owner projection returned only through a selected checked-package reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderProjection {
    /// Exact owner package identity.
    pub package: PackageId,
    /// Exact owner artifact identity.
    pub artifact: ArtifactId,
    /// Owner raw-byte digest.
    pub digest: [u8; 32],
    /// Typed semantic nodes in any producer order.
    pub nodes: Vec<SemanticNode>,
    /// Typed semantic edges in any producer order.
    pub edges: Vec<SemanticEdge>,
}

/// Cooperative remaining budget passed to owner/consumer ports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallBudget {
    remaining_duration_ms: u64,
    max_output_bytes: u64,
    max_nodes: u64,
    max_edges: u64,
}

impl CallBudget {
    /// Remaining total duration at this call boundary.
    #[must_use]
    pub const fn remaining_duration_ms(self) -> u64 {
        self.remaining_duration_ms
    }

    /// Remaining caller-lowered output ceiling.
    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }

    /// Maximum nodes the coordinator will retain from the projection.
    #[must_use]
    pub const fn max_nodes(self) -> u64 {
        self.max_nodes
    }

    /// Maximum edges the coordinator will retain from the projection.
    #[must_use]
    pub const fn max_edges(self) -> u64 {
        self.max_edges
    }
}

/// Selected strict reader for exact owner semantics.
pub trait CheckedPackageReader {
    /// Exact capability selection implemented by this reader.
    fn selection(&self) -> &CapabilitySelection;

    /// Immutable source-artifact inventory understood by this selected reader.
    fn source_artifacts(&self) -> &BTreeSet<ArtifactId>;

    /// Reads and validates owner bytes under constructor-private lock admission.
    fn read(
        &self,
        admission: &AdmittedCapability<'_>,
        owner: &OwnerPackage,
        budget: CallBudget,
    ) -> std::result::Result<ReaderProjection, ReaderError>;
}

/// Selected reader failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderError {
    /// Owner input is invalid under its selected contract.
    Invalid,
    /// Reader could not complete within one supplied bound.
    Exhausted(ReaderResource),
    /// Reader failed after admission.
    Failed,
}

/// Closed resource dimensions enforced cooperatively by the selected reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderResource {
    /// Owner input bytes.
    InputBytes,
    /// Projected nodes.
    Nodes,
    /// Projected edges.
    Edges,
    /// Total call duration.
    Duration,
}

/// Closed pure query filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewFilter {
    /// Retain every discovered fact.
    All,
    /// Retain facts attributed to one exact owner artifact.
    SourceArtifact(ArtifactId),
}

/// Closed pure query projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewProjection {
    /// Retain semantic and exact source identity.
    IdentityAndSource,
    /// Retain identity while still accounting for source attribution separately.
    IdentityOnly,
}

/// One exact view request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewQuery {
    /// Exact checked-package reader selection.
    pub reader: CapabilitySelection,
    /// Exact owner package.
    pub package: PackageId,
    /// Exact immutable owner artifact.
    pub owner_artifact: ArtifactId,
    /// Exact owner raw-byte digest.
    pub package_digest: [u8; 32],
    /// Included traversal root.
    pub root: QualifiedSemanticId,
    /// Selected typed relation.
    pub relation: RelationSelection,
    /// Pure filter applied after discovery.
    pub filter: ViewFilter,
    /// Pure projection.
    pub projection: ViewProjection,
    /// Exact profile artifact governing view presentation.
    pub profile: ArtifactId,
}

/// Display accounting for one visited fact.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ViewItemState {
    /// Fact is present in the selected projection.
    Displayed,
    /// Fact was discovered and intentionally filtered from presentation.
    Omitted,
}

/// One exactly attributed visited fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewItem {
    /// Semantic identity.
    pub semantic: QualifiedSemanticId,
    /// Exact source attribution retained independently of presentation projection.
    pub attribution: SourceRef,
    /// Source field exposed by the selected presentation projection.
    pub projected_source: Option<SourceRef>,
    /// Presentation accounting state.
    pub state: ViewItemState,
}

/// Immediate canonical unexpanded queue entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FrontierEntry {
    /// Breadth-first depth.
    pub depth: u64,
    /// Pending semantic target.
    pub target: QualifiedSemanticId,
    /// Prior fact and edge source; absent for the root.
    pub predecessor: Option<QualifiedSemanticId>,
    /// Selected relation.
    pub relation: RelationKind,
    /// Edge/root source attribution.
    pub provenance: SourceRef,
}

/// Exact work retained for a view.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Retained owner input bytes.
    pub input_bytes: u64,
    /// Selected strict-reader calls performed.
    pub reader_invocations: u64,
    /// Reader-declared source artifacts retained for admission.
    pub source_artifacts: u64,
    /// Validated projection nodes retained.
    pub projected_nodes: u64,
    /// Validated projection edges retained.
    pub projected_edges: u64,
    /// Expanded canonical nodes.
    pub traversal_steps: u64,
    /// Selected outgoing edges admitted atomically.
    pub traversed_edges: u64,
    /// Retained displayed/omitted items.
    pub results: u64,
    /// Immediate frontier entries retained in an incomplete result.
    pub frontier_entries: u64,
}

/// Bound that stopped traversal before excess.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundExceeded {
    /// Traversal-step bound.
    TraversalSteps,
    /// Selected-edge work bound.
    Edges,
    /// Result-retention bound.
    Results,
    /// Total traversal duration.
    Duration,
}

/// Complete or truthfully incomplete traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraversalOutcome {
    /// Entire selected closure was accounted.
    Complete,
    /// Traversal stopped before excess with an exact immediate frontier.
    Incomplete {
        /// Bound that stopped progress.
        cause: BoundExceeded,
        /// Canonical immediate pending boundary.
        frontier: Vec<FrontierEntry>,
    },
}

/// Canonical semantic view. It exposes no constructor for checked source semantics.
///
/// TC-004 / FR-004-AC-6: consumer-facing views cannot be promoted to owner packages.
///
/// A downstream consumer cannot forge a checked view:
///
/// ```compile_fail
/// use quire_integration::consumer_views::SemanticView;
///
/// let _forged = SemanticView {};
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticView {
    package: PackageId,
    owner_artifact: ArtifactId,
    package_digest: [u8; 32],
    lock_identity: LockIdentity,
    query: ViewQuery,
    limits: Limits,
    items: Vec<ViewItem>,
    outcome: TraversalOutcome,
    usage: Usage,
    duration_exhausted: bool,
    canonical_bytes: Vec<u8>,
    identity: [u8; 32],
}

impl SemanticView {
    /// Exact source package.
    #[must_use]
    pub const fn package(&self) -> &PackageId {
        &self.package
    }

    /// Exact immutable owner artifact.
    #[must_use]
    pub const fn owner_artifact(&self) -> &ArtifactId {
        &self.owner_artifact
    }

    /// Exact source digest.
    #[must_use]
    pub const fn package_digest(&self) -> [u8; 32] {
        self.package_digest
    }

    /// Canonically ordered accounting items.
    #[must_use]
    pub fn items(&self) -> &[ViewItem] {
        &self.items
    }

    /// Complete/incomplete traversal state.
    #[must_use]
    pub const fn outcome(&self) -> &TraversalOutcome {
        &self.outcome
    }

    /// Exact retained usage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Whether bounded finalization crossed the total duration ceiling.
    #[must_use]
    pub const fn duration_exhausted(&self) -> bool {
        self.duration_exhausted
    }

    /// Canonical bytes over every meaning-affecting input and result.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Domain-separated SHA-256 view identity.
    #[must_use]
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }
}

/// Trusted monotonic source for deterministic duration testing.
pub trait MonotonicClock: Send + Sync {
    /// Elapsed monotonic time from a stable arbitrary origin.
    fn now(&self) -> Duration;
}

#[derive(Debug)]
struct SystemClock(Instant);

impl SystemClock {
    fn new() -> Self {
        Self(Instant::now())
    }
}

impl MonotonicClock for SystemClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}

#[derive(Clone, Copy)]
struct Timer<'a> {
    clock: &'a dyn MonotonicClock,
    started: Duration,
    limit: Duration,
}

impl<'a> Timer<'a> {
    fn start(clock: &'a dyn MonotonicClock, limit_ms: u64) -> Result<Self> {
        if limit_ms == 0 {
            return Err(Error::exhausted(Resource::Duration, "zero duration budget"));
        }
        Ok(Self {
            clock,
            started: clock.now(),
            limit: Duration::from_millis(limit_ms),
        })
    }

    fn remaining_ms(self) -> Result<u64> {
        let elapsed = self
            .clock
            .now()
            .checked_sub(self.started)
            .ok_or_else(|| Error::exhausted(Resource::Duration, "clock moved backwards"))?;
        let remaining = self
            .limit
            .checked_sub(elapsed)
            .ok_or_else(|| Error::exhausted(Resource::Duration, "duration exhausted"))?;
        let millis = u64::try_from(remaining.as_millis())
            .map_err(|_| Error::new(ErrorCode::IntegerOverflow, "duration overflow"))?;
        if millis == 0 {
            return Err(Error::exhausted(Resource::Duration, "duration exhausted"));
        }
        Ok(millis)
    }

    fn check(self) -> Result<()> {
        self.remaining_ms().map(|_| ())
    }
}

/// Pure bounded view coordinator.
#[derive(Clone)]
pub struct ViewCoordinator {
    limits: Limits,
    clock: Arc<dyn MonotonicClock>,
}

impl ViewCoordinator {
    /// Constructs a coordinator using the process monotonic clock.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            clock: Arc::new(SystemClock::new()),
        }
    }

    /// Constructs a coordinator with a trusted test/composition-root clock.
    #[must_use]
    pub fn with_monotonic_clock(limits: Limits, clock: Arc<dyn MonotonicClock>) -> Self {
        Self { limits, clock }
    }

    /// Reads one exact owner package and composes its selected bounded view.
    pub fn query<R: CheckedPackageReader>(
        &self,
        lock: &ValidatedLock,
        owner: &OwnerPackage,
        query: &ViewQuery,
        reader: &R,
    ) -> Result<SemanticView> {
        let timer = Timer::start(self.clock.as_ref(), self.limits.max_duration_ms)?;
        self.validate_owner(lock, owner, query, reader)?;
        if self.limits.max_reader_invocations == 0 {
            return Err(Error::exhausted(
                Resource::ReaderInvocations,
                "reader invocation ceiling is zero",
            ));
        }
        let admission = lock
            .admit_capability(reader.selection())
            .map_err(|_| Error::new(ErrorCode::CapabilityMismatch, "reader not admitted"))?;
        if admission.contract() != CHECKED_READER_CONTRACT {
            return Err(Error::new(
                ErrorCode::CapabilityMismatch,
                "wrong reader contract",
            ));
        }
        let projection = reader
            .read(
                &admission,
                owner,
                CallBudget {
                    remaining_duration_ms: timer.remaining_ms()?,
                    max_output_bytes: self.limits.max_output_bytes,
                    max_nodes: self.limits.max_nodes,
                    max_edges: self.limits.max_projection_edges,
                },
            )
            .map_err(|error| match error {
                ReaderError::Exhausted(resource) => Error::exhausted(
                    match resource {
                        ReaderResource::InputBytes => Resource::InputBytes,
                        ReaderResource::Nodes => Resource::Nodes,
                        ReaderResource::Edges => Resource::Edges,
                        ReaderResource::Duration => Resource::Duration,
                    },
                    "reader exhausted supplied budget",
                ),
                ReaderError::Invalid | ReaderError::Failed => Error::new(
                    ErrorCode::ReaderFailure,
                    "selected reader refused owner bytes",
                ),
            })?;
        timer.check()?;
        let package =
            self.validate_projection(lock, owner, reader.source_artifacts(), projection)?;
        self.traverse(lock, query, package, timer)
    }

    fn validate_owner<R: CheckedPackageReader>(
        &self,
        lock: &ValidatedLock,
        owner: &OwnerPackage,
        query: &ViewQuery,
        reader: &R,
    ) -> Result<()> {
        if owner.contract_version != "quire.checked-package/v1"
            || owner.package != query.package
            || owner.artifact != query.owner_artifact
            || owner.digest != query.package_digest
            || reader.selection() != &query.reader
            || !lock.contains_package(&owner.package)
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "foreign owner or reader input",
            ));
        }
        validate_qualified(lock, &query.root, self.limits.max_identity_bytes)?;
        validate_local_identity(query.profile.as_str(), self.limits.max_identity_bytes)?;
        if query.profile != *query.reader.options() {
            return Err(Error::new(ErrorCode::ForeignInput, "foreign view profile"));
        }
        check_len(
            reader.source_artifacts().len(),
            self.limits.max_source_artifacts,
            Resource::SourceArtifacts,
        )?;
        if !reader.source_artifacts().contains(&owner.artifact) {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "reader does not own package artifact",
            ));
        }
        for artifact in reader.source_artifacts() {
            validate_local_identity(artifact.as_str(), self.limits.max_identity_bytes)?;
        }
        if let ViewFilter::SourceArtifact(artifact) = &query.filter {
            validate_local_identity(artifact.as_str(), self.limits.max_identity_bytes)?;
            if !reader.source_artifacts().contains(artifact) {
                return Err(Error::new(ErrorCode::ForeignInput, "foreign view filter"));
            }
        }
        check_len(
            owner.canonical_bytes.len(),
            self.limits.max_input_bytes,
            Resource::InputBytes,
        )?;
        if owner.digest != Sha256::digest(&owner.canonical_bytes).as_slice() {
            return Err(Error::new(ErrorCode::Malformed, "owner digest mismatch"));
        }
        if query.relation.contract_version != "quire.semantic-relation/v1" {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "foreign relation contract",
            ));
        }
        Ok(())
    }

    fn validate_projection(
        &self,
        lock: &ValidatedLock,
        owner: &OwnerPackage,
        source_artifacts: &BTreeSet<ArtifactId>,
        mut projection: ReaderProjection,
    ) -> Result<CheckedProjection> {
        if projection.package != owner.package
            || projection.artifact != owner.artifact
            || projection.digest != owner.digest
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "reader projection cross-wired",
            ));
        }
        check_len(
            projection.nodes.len(),
            self.limits.max_nodes,
            Resource::Nodes,
        )?;
        check_len(
            projection.edges.len(),
            self.limits.max_projection_edges,
            Resource::Edges,
        )?;
        projection
            .nodes
            .sort_by(|left, right| left.id.cmp(&right.id));
        if projection
            .nodes
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
        {
            return Err(Error::new(ErrorCode::Malformed, "duplicate semantic node"));
        }
        let mut node_ids = BTreeSet::new();
        for node in &projection.nodes {
            validate_qualified(lock, &node.id, self.limits.max_identity_bytes)?;
            validate_source(&node.source, self.limits.max_identity_bytes)?;
            if !source_artifacts.contains(&node.source.artifact) {
                return Err(Error::new(ErrorCode::ForeignInput, "foreign node source"));
            }
            node_ids.insert(node.id.clone());
        }
        projection.edges.sort_by(|left, right| {
            (&left.source, left.relation, &left.target, &left.provenance).cmp(&(
                &right.source,
                right.relation,
                &right.target,
                &right.provenance,
            ))
        });
        if projection.edges.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::new(ErrorCode::Malformed, "duplicate semantic edge"));
        }
        for edge in &projection.edges {
            if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                return Err(Error::new(ErrorCode::Malformed, "dangling semantic edge"));
            }
            validate_source(&edge.provenance, self.limits.max_identity_bytes)?;
            if !source_artifacts.contains(&edge.provenance.artifact) {
                return Err(Error::new(ErrorCode::ForeignInput, "foreign edge source"));
            }
        }
        Ok(CheckedProjection {
            input_bytes: count(owner.canonical_bytes.len())?,
            source_artifacts: count(source_artifacts.len())?,
            projected_nodes: count(projection.nodes.len())?,
            projected_edges: count(projection.edges.len())?,
            nodes: projection.nodes,
            edges: projection.edges,
        })
    }

    fn traverse(
        &self,
        lock: &ValidatedLock,
        query: &ViewQuery,
        package: CheckedProjection,
        timer: Timer<'_>,
    ) -> Result<SemanticView> {
        let nodes: BTreeMap<_, _> = package
            .nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();
        let root = nodes
            .get(&query.root)
            .ok_or_else(|| Error::new(ErrorCode::ForeignInput, "unknown view root"))?;
        let mut adjacency: BTreeMap<QualifiedSemanticId, Vec<SemanticEdge>> = BTreeMap::new();
        for edge in package
            .edges
            .into_iter()
            .filter(|edge| edge.relation == query.relation.kind)
        {
            adjacency.entry(edge.source.clone()).or_default().push(edge);
        }
        let root_entry = FrontierEntry {
            depth: 0,
            predecessor: None,
            target: query.root.clone(),
            relation: query.relation.kind,
            provenance: root.source.clone(),
        };
        let mut pending = BTreeSet::from([root_entry]);
        let mut visited = BTreeSet::new();
        let mut items = Vec::new();
        let mut usage = Usage {
            input_bytes: package.input_bytes,
            reader_invocations: 1,
            source_artifacts: package.source_artifacts,
            projected_nodes: package.projected_nodes,
            projected_edges: package.projected_edges,
            ..Usage::default()
        };
        let mut stopped = None;
        while let Some(next) = pending.first().cloned() {
            if visited.contains(&next.target) {
                pending.pop_first();
                continue;
            }
            if timer.check().is_err() {
                stopped = Some(BoundExceeded::Duration);
                break;
            }
            let outgoing = adjacency.get(&next.target).map_or(&[][..], Vec::as_slice);
            let outgoing_count = count(outgoing.len())?;
            if usage.traversal_steps >= self.limits.max_traversal_steps {
                stopped = Some(BoundExceeded::TraversalSteps);
                break;
            }
            if usage.results >= self.limits.max_results {
                stopped = Some(BoundExceeded::Results);
                break;
            }
            if usage
                .traversed_edges
                .checked_add(outgoing_count)
                .is_none_or(|total| total > self.limits.max_edges)
            {
                stopped = Some(BoundExceeded::Edges);
                break;
            }
            pending.pop_first();
            visited.insert(next.target.clone());
            let node = nodes
                .get(&next.target)
                .ok_or_else(|| Error::new(ErrorCode::Malformed, "pending node missing"))?;
            let displayed = match &query.filter {
                ViewFilter::All => true,
                ViewFilter::SourceArtifact(artifact) => &node.source.artifact == artifact,
            };
            items.push(ViewItem {
                semantic: node.id.clone(),
                attribution: node.source.clone(),
                projected_source: match query.projection {
                    ViewProjection::IdentityAndSource => Some(node.source.clone()),
                    ViewProjection::IdentityOnly => None,
                },
                state: if displayed {
                    ViewItemState::Displayed
                } else {
                    ViewItemState::Omitted
                },
            });
            usage.traversal_steps = checked_add(usage.traversal_steps, 1)?;
            usage.results = checked_add(usage.results, 1)?;
            usage.traversed_edges = checked_add(usage.traversed_edges, outgoing_count)?;
            let depth = checked_add(next.depth, 1)?;
            for edge in outgoing {
                if !visited.contains(&edge.target) {
                    let entry = FrontierEntry {
                        depth,
                        predecessor: Some(edge.source.clone()),
                        target: edge.target.clone(),
                        relation: edge.relation,
                        provenance: edge.provenance.clone(),
                    };
                    pending.insert(entry);
                }
            }
        }
        let outcome = if let Some(cause) = stopped {
            check_len(pending.len(), self.limits.max_frontier, Resource::Frontier)?;
            usage.frontier_entries = count(pending.len())?;
            TraversalOutcome::Incomplete {
                cause,
                frontier: pending.into_iter().collect(),
            }
        } else {
            TraversalOutcome::Complete
        };
        items.sort_by(|left, right| left.semantic.cmp(&right.semantic));
        let mut view = SemanticView {
            package: query.package.clone(),
            owner_artifact: query.owner_artifact.clone(),
            package_digest: query.package_digest,
            lock_identity: lock.identity(),
            query: query.clone(),
            limits: self.limits,
            items,
            outcome,
            usage,
            duration_exhausted: false,
            canonical_bytes: Vec::new(),
            identity: [0; 32],
        };
        view.canonical_bytes = encode_view(&view, self.limits.max_accounting_bytes)?;
        let mut hasher = Sha256::new();
        hasher.update(b"quire.semantic-view/v1\0");
        hasher.update(&view.canonical_bytes);
        view.identity = hasher.finalize().into();
        if timer.check().is_err() {
            view.duration_exhausted = true;
            if matches!(view.outcome, TraversalOutcome::Complete) {
                view.outcome = TraversalOutcome::Incomplete {
                    cause: BoundExceeded::Duration,
                    frontier: Vec::new(),
                };
            }
            view.canonical_bytes = encode_view(&view, self.limits.max_accounting_bytes)?;
            let mut hasher = Sha256::new();
            hasher.update(b"quire.semantic-view/v1\0");
            hasher.update(&view.canonical_bytes);
            view.identity = hasher.finalize().into();
        }
        Ok(view)
    }
}

struct CheckedProjection {
    input_bytes: u64,
    source_artifacts: u64,
    projected_nodes: u64,
    projected_edges: u64,
    nodes: Vec<SemanticNode>,
    edges: Vec<SemanticEdge>,
}

/// Selected pure consumer kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ConsumerKind {
    /// Renderer of an already checked view.
    Renderer,
    /// Non-authoritative analyzer.
    Analyzer,
    /// Source-attributed code generator.
    CodeGenerator,
}

impl ConsumerKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Renderer => "renderer",
            Self::Analyzer => "analyzer",
            Self::CodeGenerator => "code-generator",
        }
    }
}

/// Feature-local pure-consumer admission descriptor.
///
/// This is not the QSpec FR-334 owner manifest; real adapters retain and validate
/// that complete canonical owner value in Task 010 before constructing this tuple.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerAdmission {
    /// Exact lock-admitted consumer capability.
    pub selection: CapabilitySelection,
    /// Exact versioned profile.
    pub profile: ProfileId,
    /// Immutable profile artifact/options owner value.
    pub profile_artifact: ArtifactId,
    /// Exact output-region encoding rule.
    pub region_encoding: RegionEncoding,
    /// Pure operation kind.
    pub kind: ConsumerKind,
    /// Canonical set of supported obligations.
    pub supported_obligations: BTreeSet<ObligationId>,
}

/// One exact requested consumer obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestedObligation {
    /// Exact request identity.
    pub id: ObligationId,
    /// Exact checked semantic subject.
    pub semantic: QualifiedSemanticId,
    /// Exact source attribution.
    pub source: SourceRef,
    /// Required finite item-output bound; absence means `requires-bound`.
    pub output_bound: Option<u64>,
}

/// Exact admitted batch request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerRequest {
    /// Exact consumer selection expected from the composition root.
    pub selection: CapabilitySelection,
    /// Exact profile identity.
    pub profile: ProfileId,
    /// Exact profile artifact.
    pub profile_artifact: ArtifactId,
    /// Exact output-region encoding rule.
    pub region_encoding: RegionEncoding,
    /// Exact pure operation kind.
    pub kind: ConsumerKind,
    /// Explicit immutable obligation inventory.
    pub obligations: Vec<RequestedObligation>,
}

/// Owner mapping-loss value retained without translation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MappingLoss {
    /// Exact meaning represented.
    Preserved,
    /// Representation is conditional.
    Conditional,
    /// Meaning is intentionally unrepresented.
    Unrepresented,
    /// Mapping was refused.
    Refused,
}

impl MappingLoss {
    const fn label(self) -> &'static str {
        match self {
            Self::Preserved => "preserved",
            Self::Conditional => "conditional",
            Self::Unrepresented => "unrepresented",
            Self::Refused => "refused",
        }
    }
}

/// Half-open output byte region.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OutputRegion {
    /// Inclusive start byte.
    pub start: u64,
    /// Exclusive end byte.
    pub end: u64,
}

/// Consumer-produced result for one invoked obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerCompletion {
    /// Requested obligation.
    pub obligation: ObligationId,
    /// Consumer-echoed semantic identity, validated by the coordinator.
    pub semantic: QualifiedSemanticId,
    /// Consumer-echoed source identity, validated by the coordinator.
    pub source: SourceRef,
    /// Pure attributed payload.
    pub output: ConsumerOutput,
}

struct ValidatedOutput {
    outcome: AccountingOutcome,
    mapping_loss: Option<MappingLoss>,
}

/// Consumer-returned non-authoritative facade payload.
///
/// This type is not an FR-298 record, FR-299 package, or FR-333 method result.
/// Task 010's strict adapters own those complete canonical owner contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsumerOutput {
    /// Display regions in a renderer artifact.
    Displayed {
        /// Immutable output bytes.
        bytes: Vec<u8>,
        /// Exact mapped regions.
        regions: Vec<OutputRegion>,
        /// Exact upstream mapping-loss axis.
        mapping_loss: MappingLoss,
    },
    /// Fact intentionally omitted with owner mapping classification.
    Omitted {
        /// Bounded non-authoritative reason code.
        reason: String,
        /// Exact upstream mapping-loss axis.
        mapping_loss: MappingLoss,
    },
    /// Generated output bytes and mappings.
    Generated {
        /// Immutable output bytes.
        bytes: Vec<u8>,
        /// Exact mapped regions.
        regions: Vec<OutputRegion>,
        /// Exact upstream mapping-loss axis.
        mapping_loss: MappingLoss,
    },
    /// Consumer-local failure after admission.
    Failed {
        /// Bounded stable failure code.
        code: String,
    },
}

/// Borrowed consumer completion submitted to the coordinator-owned bounded sink.
pub struct ConsumerCompletionRef<'a> {
    /// Requested obligation.
    pub obligation: &'a ObligationId,
    /// Consumer-echoed semantic identity.
    pub semantic: &'a QualifiedSemanticId,
    /// Consumer-echoed source identity.
    pub source: &'a SourceRef,
    /// Borrowed output payload.
    pub output: ConsumerOutputRef<'a>,
}

/// Borrowed payload accepted only after the sink preflights every retained dimension.
pub enum ConsumerOutputRef<'a> {
    /// Displayed bytes and regions.
    Displayed {
        /// Output bytes.
        bytes: &'a [u8],
        /// Output regions.
        regions: &'a [OutputRegion],
        /// Independent mapping-loss value.
        mapping_loss: MappingLoss,
    },
    /// Omitted item.
    Omitted {
        /// Bounded reason code.
        reason: &'a str,
        /// Independent mapping-loss value.
        mapping_loss: MappingLoss,
    },
    /// Generated bytes and regions.
    Generated {
        /// Output bytes.
        bytes: &'a [u8],
        /// Output regions.
        regions: &'a [OutputRegion],
        /// Independent mapping-loss value.
        mapping_loss: MappingLoss,
    },
    /// Consumer-local failure.
    Failed {
        /// Bounded stable code.
        code: &'a str,
    },
}

impl ConsumerCompletion {
    /// Borrows this fixture/provider value for bounded sink submission.
    #[must_use]
    pub fn borrowed(&self) -> ConsumerCompletionRef<'_> {
        let output = match &self.output {
            ConsumerOutput::Displayed {
                bytes,
                regions,
                mapping_loss,
            } => ConsumerOutputRef::Displayed {
                bytes,
                regions,
                mapping_loss: *mapping_loss,
            },
            ConsumerOutput::Omitted {
                reason,
                mapping_loss,
            } => ConsumerOutputRef::Omitted {
                reason,
                mapping_loss: *mapping_loss,
            },
            ConsumerOutput::Generated {
                bytes,
                regions,
                mapping_loss,
            } => ConsumerOutputRef::Generated {
                bytes,
                regions,
                mapping_loss: *mapping_loss,
            },
            ConsumerOutput::Failed { code } => ConsumerOutputRef::Failed { code },
        };
        ConsumerCompletionRef {
            obligation: &self.obligation,
            semantic: &self.semantic,
            source: &self.source,
            output,
        }
    }
}

/// Bounded-sink refusal visible to a cooperative selected consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumerSinkError {
    /// A finite retention ceiling would be exceeded.
    Exhausted,
    /// The submitted identity or bounded code is malformed.
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SinkFailure {
    Exhausted(RequiredBound, u64),
    Invalid,
}

/// Coordinator-owned output sink that checks bounds before retaining provider bytes.
pub struct ConsumerOutputSink {
    completions: Vec<ConsumerCompletion>,
    output_bounds: BTreeMap<ObligationId, u64>,
    limits: Limits,
    usage: ConsumerUsage,
    failure: Option<SinkFailure>,
}

impl ConsumerOutputSink {
    fn new(obligations: &[RequestedObligation], limits: Limits) -> Self {
        Self {
            completions: Vec::new(),
            output_bounds: obligations
                .iter()
                .filter_map(|item| item.output_bound.map(|bound| (item.id.clone(), bound)))
                .collect(),
            limits,
            usage: ConsumerUsage::default(),
            failure: None,
        }
    }

    /// Checks all retention dimensions, then copies one borrowed completion.
    pub fn submit(
        &mut self,
        completion: ConsumerCompletionRef<'_>,
    ) -> std::result::Result<(), ConsumerSinkError> {
        if let Some(failure) = self.failure {
            return Err(match failure {
                SinkFailure::Exhausted(..) => ConsumerSinkError::Exhausted,
                SinkFailure::Invalid => ConsumerSinkError::Invalid,
            });
        }
        if validate_local_identity(
            completion.obligation.as_str(),
            self.limits.max_identity_bytes,
        )
        .is_err()
            || validate_local_identity(
                completion.semantic.definition.as_str(),
                self.limits.max_identity_bytes,
            )
            .is_err()
            || validate_local_identity(
                completion.semantic.semantic.as_str(),
                self.limits.max_identity_bytes,
            )
            .is_err()
            || validate_source(completion.source, self.limits.max_identity_bytes).is_err()
        {
            return Err(self.fail_invalid());
        }
        let Some(item_bound) = self.output_bounds.get(completion.obligation).copied() else {
            return Err(self.fail_invalid());
        };
        let Ok(result_count) = count(self.completions.len()) else {
            return Err(self.fail_invalid());
        };
        let Ok(next_results) = checked_add(result_count, 1) else {
            return Err(self.fail_invalid());
        };
        if next_results > self.limits.max_obligations {
            return Err(self.fail_exhausted(RequiredBound::ConsumerResults, next_results));
        }
        let Ok((bytes, mappings)) = borrowed_output_usage(&completion.output) else {
            return Err(self.fail_invalid());
        };
        let Ok(next_bytes) = checked_add(self.usage.output_bytes, bytes) else {
            return Err(self.fail_invalid());
        };
        let Ok(next_mappings) = checked_add(self.usage.mappings, mappings) else {
            return Err(self.fail_invalid());
        };
        self.usage.output_bytes = next_bytes;
        self.usage.mappings = next_mappings;
        if bytes > item_bound {
            return Err(self.fail_exhausted(RequiredBound::OutputBytes, bytes));
        }
        if next_bytes > self.limits.max_output_bytes {
            return Err(self.fail_exhausted(RequiredBound::OutputBytes, next_bytes));
        }
        if next_mappings > self.limits.max_mappings {
            return Err(self.fail_exhausted(RequiredBound::Mappings, next_mappings));
        }
        let Ok(output) = clone_borrowed_output(&completion.output, self.limits.max_identity_bytes)
        else {
            return Err(self.fail_invalid());
        };
        self.completions.push(ConsumerCompletion {
            obligation: completion.obligation.clone(),
            semantic: completion.semantic.clone(),
            source: completion.source.clone(),
            output,
        });
        Ok(())
    }

    fn fail_invalid(&mut self) -> ConsumerSinkError {
        self.failure = Some(SinkFailure::Invalid);
        ConsumerSinkError::Invalid
    }

    fn fail_exhausted(&mut self, resource: RequiredBound, minimum: u64) -> ConsumerSinkError {
        self.failure = Some(SinkFailure::Exhausted(resource, minimum));
        ConsumerSinkError::Exhausted
    }

    fn finish(self) -> (Vec<ConsumerCompletion>, Option<SinkFailure>, ConsumerUsage) {
        (self.completions, self.failure, self.usage)
    }
}

/// Selected consumer call failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumerError {
    /// Consumer could not complete within the supplied budget.
    Exhausted,
    /// Consumer failed after admission.
    Failed,
}

/// Opaque proof that one exact view/request batch was admitted.
///
/// TC-004 / FR-004-AC-6: only this coordinator can grant consumer invocation.
///
/// Only the coordinator can construct an invocation permit:
///
/// ```compile_fail
/// use quire_integration::consumer_views::ConsumerPermit;
///
/// let _forged = ConsumerPermit {};
/// ```
#[derive(Debug)]
pub struct ConsumerPermit {
    view_identity: [u8; 32],
    lock_identity: LockIdentity,
    remaining_duration_ms: u64,
    max_output_bytes: u64,
    remaining_mappings: u64,
}

impl ConsumerPermit {
    /// Exact admitted view identity.
    #[must_use]
    pub const fn view_identity(&self) -> [u8; 32] {
        self.view_identity
    }

    /// Exact admitting lock identity.
    #[must_use]
    pub const fn lock_identity(&self) -> LockIdentity {
        self.lock_identity
    }

    /// Cooperative remaining total duration.
    #[must_use]
    pub const fn remaining_duration_ms(&self) -> u64 {
        self.remaining_duration_ms
    }

    /// Caller-lowered aggregate output ceiling.
    #[must_use]
    pub const fn max_output_bytes(&self) -> u64 {
        self.max_output_bytes
    }

    /// Remaining aggregate mapping-region ceiling for this batch.
    #[must_use]
    pub const fn remaining_mappings(&self) -> u64 {
        self.remaining_mappings
    }
}

/// Exact pure consumer implementation selected by the composition root.
pub trait SemanticConsumer {
    /// Exact immutable feature-local admission descriptor.
    fn admission(&self) -> &ConsumerAdmission;

    /// Consumes one canonical batch and submits non-authoritative attributed outputs.
    ///
    /// Implementations must stop when the coordinator-owned sink refuses a submission;
    /// the sink checks every retention bound before copying borrowed provider bytes.
    fn consume(
        &mut self,
        permit: &ConsumerPermit,
        view: &SemanticView,
        obligations: &[RequestedObligation],
        output: &mut ConsumerOutputSink,
    ) -> std::result::Result<(), ConsumerError>;
}

/// Stable facade-invalidity category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InvalidCode {
    /// Request refers to a fact absent from the admitted view.
    ForeignSemantic,
    /// Consumer batch omitted, duplicated, or added an obligation.
    InventoryMismatch,
    /// Consumer echoed a foreign source/semantic identity.
    AttributionMismatch,
    /// Output mapping is out of bounds, malformed, or semantically inconsistent.
    MalformedMapping,
}

/// Stable admitted consumer failure category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FailureCode {
    /// Consumer returned an explicit failure.
    ConsumerFailed,
    /// Consumer exceeded its cooperative duration.
    ConsumerExhausted,
}

/// Payload-bearing facade item state; illegal artifact/state combinations are unrepresentable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountingOutcome {
    /// Rendered output and exact regions.
    Displayed {
        /// Immutable non-authoritative bytes.
        bytes: Vec<u8>,
        /// Exact mapped regions.
        regions: Vec<OutputRegion>,
    },
    /// Intentionally omitted presentation item.
    Omitted {
        /// Bounded reason.
        reason: String,
    },
    /// Generated attributable non-authoritative bytes.
    Generated {
        /// Immutable bytes.
        bytes: Vec<u8>,
        /// Raw byte digest.
        digest: [u8; 32],
        /// Exact mapped regions.
        regions: Vec<OutputRegion>,
    },
    /// Manifest does not support this exact obligation.
    Unsupported {
        /// Bounded reason.
        reason: &'static str,
    },
    /// Required finite item bound was absent or insufficient.
    RequiresBound {
        /// Exact missing/exhausted bound dimension.
        resource: RequiredBound,
        /// Minimum known required ceiling in the resource's unit.
        minimum: u64,
    },
    /// Structurally attributable but invalid request/consumer item.
    Invalid {
        /// Stable invalidity code.
        code: InvalidCode,
    },
    /// Admitted consumer failed.
    Failed {
        /// Stable failure code.
        code: FailureCode,
        /// Exact bounded selected-consumer code, when supplied by the consumer.
        consumer_code: Option<String>,
    },
}

/// Bound dimension required to retain a consumer result.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RequiredBound {
    /// Consumer result records.
    ConsumerResults,
    /// Output bytes.
    OutputBytes,
    /// Aggregate output mapping regions.
    Mappings,
    /// Selected-consumer invocations.
    Invocations,
    /// Total invocation duration.
    Duration,
}

/// Exact deterministic consumer work retained in an accounting run.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConsumerUsage {
    /// Selected consumer calls performed.
    pub invocations: u64,
    /// Bytes returned by the selected consumer before facade retention decisions.
    pub output_bytes: u64,
    /// Regions returned by the selected consumer before facade retention decisions.
    pub mappings: u64,
}

/// One exact requested obligation and its sole feature-local accounting outcome.
///
/// It is deliberately not a replacement for the QSpec FR-298 owner record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingRecord {
    /// Requested identity.
    pub obligation: ObligationId,
    /// Exact semantic identity.
    pub semantic: QualifiedSemanticId,
    /// Exact source attribution.
    pub source: SourceRef,
    /// Exact caller-supplied item-output bound.
    pub requested_output_bound: Option<u64>,
    /// Independent exact owner mapping-loss value, when the consumer supplied one.
    pub mapping_loss: Option<MappingLoss>,
    /// Sole payload-bearing outcome.
    pub outcome: AccountingOutcome,
}

/// Aggregate coordination state for one admitted consumer run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinationState {
    /// Every invoked supported item completed with a valid response.
    Complete,
    /// A finite view, invocation, mapping, output, or duration bound prevented completion.
    Incomplete,
    /// Request was structurally accountable but not invocable.
    Refused,
    /// Admitted consumer failed or violated its contract.
    Failed,
}

/// Complete canonical accounting envelope for one structurally admitted request.
///
/// TC-004 / FR-004-AC-6: accounting output is not checked source or qualification
/// authority, and the type system rejects either conversion.
///
/// ```compile_fail
/// use quire_integration::consumer_views::{ConsumerRun, OwnerPackage};
///
/// fn promote(run: ConsumerRun) {
///     let _source: OwnerPackage = run;
/// }
/// ```
///
/// ```compile_fail
/// use quire_integration::consumer_views::{AccountingRecord, ReaderProjection};
///
/// fn reenter(record: AccountingRecord) {
///     let _projection: ReaderProjection = record;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerRun {
    state: CoordinationState,
    view_identity: [u8; 32],
    view_outcome: TraversalOutcome,
    view_usage: Usage,
    usage: ConsumerUsage,
    duration_exhausted: bool,
    lock_identity: LockIdentity,
    selection: CapabilitySelection,
    profile: ProfileId,
    profile_artifact: ArtifactId,
    region_encoding: RegionEncoding,
    kind: ConsumerKind,
    limits: Limits,
    records: Vec<AccountingRecord>,
    canonical_bytes: Vec<u8>,
    identity: [u8; 32],
}

impl ConsumerRun {
    /// Aggregate coordination state.
    #[must_use]
    pub const fn state(&self) -> CoordinationState {
        self.state
    }

    /// Exact source-view completeness state.
    #[must_use]
    pub const fn view_outcome(&self) -> &TraversalOutcome {
        &self.view_outcome
    }

    /// Exact source-view retained usage.
    #[must_use]
    pub const fn view_usage(&self) -> Usage {
        self.view_usage
    }

    /// Exact deterministic consumer work performed.
    #[must_use]
    pub const fn usage(&self) -> ConsumerUsage {
        self.usage
    }

    /// Whether bounded finalization crossed the total duration ceiling.
    #[must_use]
    pub const fn duration_exhausted(&self) -> bool {
        self.duration_exhausted
    }

    /// Exactly one canonical record per request obligation.
    #[must_use]
    pub fn records(&self) -> &[AccountingRecord] {
        &self.records
    }

    /// Canonical complete accounting bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Domain-separated accounting identity.
    #[must_use]
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }
}

/// Exact consumer/accounting coordinator.
#[derive(Clone)]
pub struct ConsumerCoordinator {
    limits: Limits,
    clock: Arc<dyn MonotonicClock>,
}

impl ConsumerCoordinator {
    /// Constructs a coordinator using process monotonic time.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            clock: Arc::new(SystemClock::new()),
        }
    }

    /// Constructs a coordinator with a trusted monotonic clock.
    #[must_use]
    pub fn with_monotonic_clock(limits: Limits, clock: Arc<dyn MonotonicClock>) -> Self {
        Self { limits, clock }
    }

    /// Accounts for one exact canonical consumer batch.
    pub fn consume<C: SemanticConsumer>(
        &self,
        lock: &ValidatedLock,
        view: &SemanticView,
        request: &ConsumerRequest,
        consumer: &mut C,
    ) -> Result<ConsumerRun> {
        let timer = Timer::start(self.clock.as_ref(), self.limits.max_duration_ms)?;
        let manifest = consumer.admission();
        if manifest.selection != request.selection
            || manifest.profile != request.profile
            || manifest.profile_artifact != request.profile_artifact
            || manifest.region_encoding != request.region_encoding
            || manifest.kind != request.kind
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "consumer manifest mismatch",
            ));
        }
        validate_local_identity(request.profile.as_str(), self.limits.max_identity_bytes)?;
        validate_local_identity(
            request.profile_artifact.as_str(),
            self.limits.max_identity_bytes,
        )?;
        if request.profile_artifact != *request.selection.options() {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "foreign consumer profile artifact",
            ));
        }
        check_len(
            manifest.supported_obligations.len(),
            self.limits.max_obligations,
            Resource::Obligations,
        )?;
        for supported in &manifest.supported_obligations {
            validate_local_identity(supported.as_str(), self.limits.max_identity_bytes)?;
        }
        let admission = lock
            .admit_capability(&request.selection)
            .map_err(|_| Error::new(ErrorCode::CapabilityMismatch, "consumer not admitted"))?;
        if admission.contract() != CONSUMER_CONTRACT || lock.identity() != view.lock_identity {
            return Err(Error::new(
                ErrorCode::CapabilityMismatch,
                "wrong consumer contract or lock",
            ));
        }
        check_len(
            request.obligations.len(),
            self.limits.max_obligations,
            Resource::Obligations,
        )?;
        if request.obligations.is_empty() {
            return Err(Error::new(
                ErrorCode::Malformed,
                "empty obligation inventory",
            ));
        }
        for obligation in &request.obligations {
            validate_local_identity(&obligation.id.0, self.limits.max_identity_bytes)?;
            validate_local_identity(
                obligation.semantic.definition.as_str(),
                self.limits.max_identity_bytes,
            )?;
            validate_local_identity(
                obligation.semantic.semantic.as_str(),
                self.limits.max_identity_bytes,
            )?;
            validate_source(&obligation.source, self.limits.max_identity_bytes)?;
        }
        let mut obligations = request.obligations.clone();
        obligations.sort_by(|left, right| left.id.cmp(&right.id));
        if obligations.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "duplicate obligation identity",
            ));
        }
        let view_items: BTreeMap<_, _> = view
            .items
            .iter()
            .map(|item| (item.semantic.clone(), item.attribution.clone()))
            .collect();
        let mut records = Vec::with_capacity(obligations.len());
        let mut invoked = Vec::new();
        let mut preflight_invalid = false;
        let mut invocation_exhausted = false;
        for obligation in obligations {
            if !lock.contains_definition(&obligation.semantic.definition) {
                preflight_invalid = true;
                records.push(AccountingRecord {
                    obligation: obligation.id,
                    semantic: obligation.semantic,
                    source: obligation.source,
                    requested_output_bound: obligation.output_bound,
                    mapping_loss: None,
                    outcome: AccountingOutcome::Invalid {
                        code: InvalidCode::ForeignSemantic,
                    },
                });
                continue;
            }
            let Some(source) = view_items.get(&obligation.semantic) else {
                preflight_invalid = true;
                records.push(AccountingRecord {
                    obligation: obligation.id,
                    semantic: obligation.semantic,
                    source: obligation.source,
                    requested_output_bound: obligation.output_bound,
                    mapping_loss: None,
                    outcome: AccountingOutcome::Invalid {
                        code: InvalidCode::ForeignSemantic,
                    },
                });
                continue;
            };
            if source != &obligation.source {
                preflight_invalid = true;
                records.push(AccountingRecord {
                    obligation: obligation.id,
                    semantic: obligation.semantic,
                    source: obligation.source,
                    requested_output_bound: obligation.output_bound,
                    mapping_loss: None,
                    outcome: AccountingOutcome::Invalid {
                        code: InvalidCode::AttributionMismatch,
                    },
                });
                continue;
            }
            if !manifest.supported_obligations.contains(&obligation.id) {
                records.push(AccountingRecord {
                    obligation: obligation.id,
                    semantic: obligation.semantic,
                    source: obligation.source,
                    requested_output_bound: obligation.output_bound,
                    mapping_loss: None,
                    outcome: AccountingOutcome::Unsupported {
                        reason: "selected profile does not support this obligation",
                    },
                });
            } else if obligation.output_bound.is_none() {
                records.push(AccountingRecord {
                    obligation: obligation.id,
                    semantic: obligation.semantic,
                    source: obligation.source,
                    requested_output_bound: obligation.output_bound,
                    mapping_loss: None,
                    outcome: AccountingOutcome::RequiresBound {
                        resource: RequiredBound::OutputBytes,
                        minimum: 1,
                    },
                });
            } else {
                invoked.push(obligation);
            }
        }
        let (completions, sink_failure, consumer_usage) = if invoked.is_empty() {
            (Vec::new(), None, ConsumerUsage::default())
        } else if self.limits.max_consumer_invocations == 0 {
            invocation_exhausted = true;
            records.extend(invoked.drain(..).map(|obligation| AccountingRecord {
                obligation: obligation.id,
                semantic: obligation.semantic,
                source: obligation.source,
                requested_output_bound: obligation.output_bound,
                mapping_loss: None,
                outcome: AccountingOutcome::RequiresBound {
                    resource: RequiredBound::Invocations,
                    minimum: 1,
                },
            }));
            (Vec::new(), None, ConsumerUsage::default())
        } else {
            let permit = ConsumerPermit {
                view_identity: view.identity,
                lock_identity: lock.identity(),
                remaining_duration_ms: timer.remaining_ms()?,
                max_output_bytes: self.limits.max_output_bytes,
                remaining_mappings: self.limits.max_mappings,
            };
            let mut sink = ConsumerOutputSink::new(&invoked, self.limits);
            let result = consumer.consume(&permit, view, &invoked, &mut sink);
            let (completions, sink_failure, mut usage) = sink.finish();
            usage.invocations = 1;
            if sink_failure.is_some() {
                (completions, sink_failure, usage)
            } else {
                match result {
                    Ok(()) => (completions, sink_failure, usage),
                    Err(error) => {
                        let code = match error {
                            ConsumerError::Exhausted => FailureCode::ConsumerExhausted,
                            ConsumerError::Failed => FailureCode::ConsumerFailed,
                        };
                        records.extend(invoked.into_iter().map(|obligation| AccountingRecord {
                            obligation: obligation.id,
                            semantic: obligation.semantic,
                            source: obligation.source,
                            requested_output_bound: obligation.output_bound,
                            mapping_loss: None,
                            outcome: AccountingOutcome::Failed {
                                code,
                                consumer_code: None,
                            },
                        }));
                        return self.finish_run(
                            view,
                            lock.identity(),
                            request,
                            CoordinationState::Failed,
                            records,
                            usage,
                            timer,
                        );
                    }
                }
            }
        };
        if let Some(failure) = sink_failure {
            let (state, outcome) = match failure {
                SinkFailure::Exhausted(resource, minimum) => (
                    if preflight_invalid {
                        CoordinationState::Failed
                    } else {
                        CoordinationState::Incomplete
                    },
                    AccountingOutcome::RequiresBound { resource, minimum },
                ),
                SinkFailure::Invalid => (
                    CoordinationState::Failed,
                    AccountingOutcome::Invalid {
                        code: InvalidCode::InventoryMismatch,
                    },
                ),
            };
            records.extend(invoked.into_iter().map(|obligation| AccountingRecord {
                obligation: obligation.id,
                semantic: obligation.semantic,
                source: obligation.source,
                requested_output_bound: obligation.output_bound,
                mapping_loss: None,
                outcome: outcome.clone(),
            }));
            return self.finish_run(
                view,
                lock.identity(),
                request,
                state,
                records,
                consumer_usage,
                timer,
            );
        }
        if timer.check().is_err() {
            records.extend(invoked.into_iter().map(|obligation| AccountingRecord {
                obligation: obligation.id,
                semantic: obligation.semantic,
                source: obligation.source,
                requested_output_bound: obligation.output_bound,
                mapping_loss: None,
                outcome: AccountingOutcome::Failed {
                    code: FailureCode::ConsumerExhausted,
                    consumer_code: None,
                },
            }));
            return self.finish_run(
                view,
                lock.identity(),
                request,
                CoordinationState::Failed,
                records,
                consumer_usage,
                timer,
            );
        }
        let (mut validated, output_bytes, mapping_count) =
            self.validate_completions(&invoked, completions, request.region_encoding)?;
        let response_invalid = validated
            .iter()
            .any(|record| matches!(record.outcome, AccountingOutcome::Invalid { .. }));
        let item_output_exhausted = validated
            .iter()
            .any(|record| matches!(record.outcome, AccountingOutcome::RequiresBound { .. }));
        records.append(&mut validated);
        if mapping_count > self.limits.max_mappings {
            for record in &mut records {
                if matches!(
                    record.outcome,
                    AccountingOutcome::Displayed { .. } | AccountingOutcome::Generated { .. }
                ) {
                    record.outcome = AccountingOutcome::RequiresBound {
                        resource: RequiredBound::Mappings,
                        minimum: mapping_count,
                    };
                }
            }
        }
        let output_exhausted = output_bytes > self.limits.max_output_bytes;
        if output_exhausted {
            for record in &mut records {
                if matches!(
                    record.outcome,
                    AccountingOutcome::Displayed { .. } | AccountingOutcome::Generated { .. }
                ) {
                    record.outcome = AccountingOutcome::RequiresBound {
                        resource: RequiredBound::OutputBytes,
                        minimum: output_bytes,
                    };
                }
            }
        }
        let state = if response_invalid
            || records
                .iter()
                .any(|record| matches!(record.outcome, AccountingOutcome::Failed { .. }))
        {
            CoordinationState::Failed
        } else if output_exhausted
            || item_output_exhausted
            || mapping_count > self.limits.max_mappings
            || invocation_exhausted
            || matches!(view.outcome, TraversalOutcome::Incomplete { .. })
        {
            CoordinationState::Incomplete
        } else if invoked.is_empty() {
            CoordinationState::Refused
        } else if preflight_invalid {
            CoordinationState::Failed
        } else {
            CoordinationState::Complete
        };
        let run = self.finish_run(
            view,
            lock.identity(),
            request,
            state,
            records,
            consumer_usage,
            timer,
        )?;
        Ok(run)
    }

    fn validate_completions(
        &self,
        invoked: &[RequestedObligation],
        mut completions: Vec<ConsumerCompletion>,
        region_encoding: RegionEncoding,
    ) -> Result<(Vec<AccountingRecord>, u64, u64)> {
        let (output_bytes, mapping_count) = completion_usage(&completions)?;
        if count(completions.len())? > self.limits.max_obligations {
            return Ok((
                invoked
                    .iter()
                    .map(|request| AccountingRecord {
                        obligation: request.id.clone(),
                        semantic: request.semantic.clone(),
                        source: request.source.clone(),
                        requested_output_bound: request.output_bound,
                        mapping_loss: None,
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::InventoryMismatch,
                        },
                    })
                    .collect(),
                output_bytes,
                mapping_count,
            ));
        }
        completions.sort_by(|left, right| left.obligation.cmp(&right.obligation));
        let inventory_matches = completions.len() == invoked.len()
            && completions
                .iter()
                .zip(invoked)
                .all(|(completion, request)| completion.obligation == request.id);
        if !inventory_matches
            || completions
                .windows(2)
                .any(|pair| pair[0].obligation == pair[1].obligation)
        {
            return Ok((
                invoked
                    .iter()
                    .map(|request| AccountingRecord {
                        obligation: request.id.clone(),
                        semantic: request.semantic.clone(),
                        source: request.source.clone(),
                        requested_output_bound: request.output_bound,
                        mapping_loss: None,
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::InventoryMismatch,
                        },
                    })
                    .collect(),
                output_bytes,
                mapping_count,
            ));
        }
        let mut records = Vec::with_capacity(invoked.len());
        for (completion, request) in completions.into_iter().zip(invoked) {
            let validated =
                if completion.semantic != request.semantic || completion.source != request.source {
                    ValidatedOutput {
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::AttributionMismatch,
                        },
                        mapping_loss: None,
                    }
                } else {
                    self.validate_consumer_output(completion.output, request, region_encoding)?
                };
            records.push(AccountingRecord {
                obligation: request.id.clone(),
                semantic: request.semantic.clone(),
                source: request.source.clone(),
                requested_output_bound: request.output_bound,
                mapping_loss: validated.mapping_loss,
                outcome: validated.outcome,
            });
        }
        Ok((records, output_bytes, mapping_count))
    }

    fn validate_consumer_output(
        &self,
        output: ConsumerOutput,
        request: &RequestedObligation,
        region_encoding: RegionEncoding,
    ) -> Result<ValidatedOutput> {
        match output {
            ConsumerOutput::Displayed {
                bytes,
                mut regions,
                mapping_loss,
            } => {
                let length = count(bytes.len())?;
                if length > request.output_bound.unwrap_or(0)
                    || length > self.limits.max_output_bytes
                {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::RequiresBound {
                            resource: RequiredBound::OutputBytes,
                            minimum: length,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                let mapping_count = count(regions.len())?;
                if mapping_count > self.limits.max_mappings {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::RequiresBound {
                            resource: RequiredBound::Mappings,
                            minimum: mapping_count,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                if !valid_regions(&bytes, &mut regions, region_encoding)? {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::MalformedMapping,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                Ok(ValidatedOutput {
                    outcome: AccountingOutcome::Displayed { bytes, regions },
                    mapping_loss: Some(mapping_loss),
                })
            }
            ConsumerOutput::Omitted {
                reason,
                mapping_loss,
            } => {
                if reason.is_empty() || count(reason.len())? > self.limits.max_identity_bytes {
                    Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::MalformedMapping,
                        },
                        mapping_loss: Some(mapping_loss),
                    })
                } else {
                    Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Omitted { reason },
                        mapping_loss: Some(mapping_loss),
                    })
                }
            }
            ConsumerOutput::Generated {
                bytes,
                mut regions,
                mapping_loss,
            } => {
                let length = count(bytes.len())?;
                if length > request.output_bound.unwrap_or(0)
                    || length > self.limits.max_output_bytes
                {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::RequiresBound {
                            resource: RequiredBound::OutputBytes,
                            minimum: length,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                let mapping_count = count(regions.len())?;
                if mapping_count > self.limits.max_mappings {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::RequiresBound {
                            resource: RequiredBound::Mappings,
                            minimum: mapping_count,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                if !valid_regions(&bytes, &mut regions, region_encoding)? {
                    return Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::MalformedMapping,
                        },
                        mapping_loss: Some(mapping_loss),
                    });
                }
                let digest = Sha256::digest(&bytes).into();
                Ok(ValidatedOutput {
                    outcome: AccountingOutcome::Generated {
                        bytes,
                        digest,
                        regions,
                    },
                    mapping_loss: Some(mapping_loss),
                })
            }
            ConsumerOutput::Failed { code } => {
                if code.is_empty() || count(code.len())? > self.limits.max_identity_bytes {
                    Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Invalid {
                            code: InvalidCode::MalformedMapping,
                        },
                        mapping_loss: None,
                    })
                } else {
                    Ok(ValidatedOutput {
                        outcome: AccountingOutcome::Failed {
                            code: FailureCode::ConsumerFailed,
                            consumer_code: Some(code),
                        },
                        mapping_loss: None,
                    })
                }
            }
        }
    }

    fn finish_run(
        &self,
        view: &SemanticView,
        lock_identity: LockIdentity,
        request: &ConsumerRequest,
        state: CoordinationState,
        mut records: Vec<AccountingRecord>,
        usage: ConsumerUsage,
        timer: Timer<'_>,
    ) -> Result<ConsumerRun> {
        records.sort_by(|left, right| left.obligation.cmp(&right.obligation));
        let mut run = ConsumerRun {
            state,
            view_identity: view.identity,
            view_outcome: view.outcome.clone(),
            view_usage: view.usage,
            usage,
            duration_exhausted: false,
            lock_identity,
            selection: request.selection.clone(),
            profile: request.profile.clone(),
            profile_artifact: request.profile_artifact.clone(),
            region_encoding: request.region_encoding,
            kind: request.kind,
            limits: self.limits,
            records,
            canonical_bytes: Vec::new(),
            identity: [0; 32],
        };
        run.canonical_bytes = encode_run(&run, self.limits.max_accounting_bytes)?;
        let mut hasher = Sha256::new();
        hasher.update(b"quire.consumer-accounting/v1\0");
        hasher.update(&run.canonical_bytes);
        run.identity = hasher.finalize().into();
        if timer.check().is_err() {
            run.duration_exhausted = true;
            let minimum = self.limits.max_duration_ms.saturating_add(1);
            for record in &mut run.records {
                if matches!(
                    record.outcome,
                    AccountingOutcome::Displayed { .. } | AccountingOutcome::Generated { .. }
                ) {
                    record.outcome = AccountingOutcome::RequiresBound {
                        resource: RequiredBound::Duration,
                        minimum,
                    };
                }
            }
            if run.state != CoordinationState::Failed {
                run.state = CoordinationState::Incomplete;
            }
            run.canonical_bytes = encode_run(&run, self.limits.max_accounting_bytes)?;
            let mut hasher = Sha256::new();
            hasher.update(b"quire.consumer-accounting/v1\0");
            hasher.update(&run.canonical_bytes);
            run.identity = hasher.finalize().into();
        }
        Ok(run)
    }
}

fn validate_local_identity(value: &str, maximum: u64) -> Result<()> {
    let length = count(value.len())?;
    if value.trim().is_empty() || length > maximum {
        return Err(Error::new(
            ErrorCode::InvalidIdentity,
            "invalid bounded identity",
        ));
    }
    Ok(())
}

fn validate_qualified(
    lock: &ValidatedLock,
    identity: &QualifiedSemanticId,
    maximum: u64,
) -> Result<()> {
    validate_local_identity(identity.definition.as_str(), maximum)?;
    validate_local_identity(identity.semantic.as_str(), maximum)?;
    if !lock.contains_definition(&identity.definition) {
        return Err(Error::new(
            ErrorCode::ForeignInput,
            "semantic definition not in lock",
        ));
    }
    Ok(())
}

fn validate_source(source: &SourceRef, maximum: u64) -> Result<()> {
    validate_local_identity(source.artifact.as_str(), maximum)?;
    validate_local_identity(source.region.as_str(), maximum)
}

fn consumer_output_usage(output: &ConsumerOutput) -> Result<(u64, u64)> {
    match output {
        ConsumerOutput::Displayed { bytes, regions, .. }
        | ConsumerOutput::Generated { bytes, regions, .. } => {
            Ok((count(bytes.len())?, count(regions.len())?))
        }
        ConsumerOutput::Omitted { .. } | ConsumerOutput::Failed { .. } => Ok((0, 0)),
    }
}

fn borrowed_output_usage(output: &ConsumerOutputRef<'_>) -> Result<(u64, u64)> {
    match output {
        ConsumerOutputRef::Displayed { bytes, regions, .. }
        | ConsumerOutputRef::Generated { bytes, regions, .. } => {
            Ok((count(bytes.len())?, count(regions.len())?))
        }
        ConsumerOutputRef::Omitted { .. } | ConsumerOutputRef::Failed { .. } => Ok((0, 0)),
    }
}

fn clone_borrowed_output(output: &ConsumerOutputRef<'_>, maximum: u64) -> Result<ConsumerOutput> {
    match output {
        ConsumerOutputRef::Displayed {
            bytes,
            regions,
            mapping_loss,
        } => Ok(ConsumerOutput::Displayed {
            bytes: bytes.to_vec(),
            regions: regions.to_vec(),
            mapping_loss: *mapping_loss,
        }),
        ConsumerOutputRef::Omitted {
            reason,
            mapping_loss,
        } => {
            validate_local_identity(reason, maximum)?;
            Ok(ConsumerOutput::Omitted {
                reason: (*reason).to_owned(),
                mapping_loss: *mapping_loss,
            })
        }
        ConsumerOutputRef::Generated {
            bytes,
            regions,
            mapping_loss,
        } => Ok(ConsumerOutput::Generated {
            bytes: bytes.to_vec(),
            regions: regions.to_vec(),
            mapping_loss: *mapping_loss,
        }),
        ConsumerOutputRef::Failed { code } => {
            validate_local_identity(code, maximum)?;
            Ok(ConsumerOutput::Failed {
                code: (*code).to_owned(),
            })
        }
    }
}

fn completion_usage(completions: &[ConsumerCompletion]) -> Result<(u64, u64)> {
    completions.iter().try_fold(
        (0_u64, 0_u64),
        |(bytes_total, mapping_total), completion| {
            let (bytes, mappings) = consumer_output_usage(&completion.output)?;
            Ok((
                checked_add(bytes_total, bytes)?,
                checked_add(mapping_total, mappings)?,
            ))
        },
    )
}

fn check_len(value: usize, maximum: u64, resource: Resource) -> Result<()> {
    if count(value)? > maximum {
        return Err(Error::exhausted(
            resource,
            "collection exceeds finite ceiling",
        ));
    }
    Ok(())
}

fn count(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| Error::new(ErrorCode::IntegerOverflow, "count overflow"))
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "counter overflow"))
}

fn valid_regions(
    bytes: &[u8],
    regions: &mut Vec<OutputRegion>,
    encoding: RegionEncoding,
) -> Result<bool> {
    if regions.is_empty() {
        return Ok(false);
    }
    regions.sort_unstable();
    if regions.windows(2).any(|pair| pair[0].end > pair[1].start) {
        return Ok(false);
    }
    let length = count(bytes.len())?;
    let utf8 = match encoding {
        RegionEncoding::Utf8 => {
            let Ok(text) = std::str::from_utf8(bytes) else {
                return Ok(false);
            };
            Some(text)
        }
        RegionEncoding::Binary => None,
    };
    for region in regions {
        if region.start >= region.end || region.end > length {
            return Ok(false);
        }
        let start = usize::try_from(region.start)
            .map_err(|_| Error::new(ErrorCode::IntegerOverflow, "region start overflow"))?;
        let end = usize::try_from(region.end)
            .map_err(|_| Error::new(ErrorCode::IntegerOverflow, "region end overflow"))?;
        if let Some(text) = utf8 {
            if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

struct CanonicalWriter {
    bytes: Vec<u8>,
    maximum: u64,
}

impl CanonicalWriter {
    fn new(maximum: u64) -> Self {
        Self {
            bytes: Vec::new(),
            maximum,
        }
    }

    fn raw(&mut self, value: &[u8]) -> Result<()> {
        let next = count(self.bytes.len())?
            .checked_add(count(value.len())?)
            .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "output length overflow"))?;
        if next > self.maximum {
            return Err(Error::exhausted(
                Resource::AccountingBytes,
                "canonical accounting output exhausted",
            ));
        }
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn field(&mut self, value: &[u8]) -> Result<()> {
        self.raw(&count(value.len())?.to_be_bytes())?;
        self.raw(value)
    }

    fn text(&mut self, value: &str) -> Result<()> {
        self.field(value.as_bytes())
    }

    fn number(&mut self, value: u64) -> Result<()> {
        self.raw(&value.to_be_bytes())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn encode_qualified(writer: &mut CanonicalWriter, identity: &QualifiedSemanticId) -> Result<()> {
    writer.text(identity.definition.as_str())?;
    writer.text(identity.semantic.as_str())
}

fn encode_source(writer: &mut CanonicalWriter, source: &SourceRef) -> Result<()> {
    writer.text(source.artifact.as_str())?;
    writer.text(source.region.as_str())
}

fn encode_selection(writer: &mut CanonicalWriter, selection: &CapabilitySelection) -> Result<()> {
    writer.text(selection.role().as_str())?;
    writer.text(selection.provider().as_str())?;
    writer.text(selection.build().as_str())?;
    writer.text(selection.contract().as_str())?;
    writer.text(selection.options().as_str())
}

fn encode_view(view: &SemanticView, maximum: u64) -> Result<Vec<u8>> {
    let mut writer = CanonicalWriter::new(maximum);
    writer.text("quire.semantic-view/v1")?;
    writer.text(view.package.as_str())?;
    writer.text(view.owner_artifact.as_str())?;
    writer.raw(&view.package_digest)?;
    writer.text(&view.lock_identity.to_hex())?;
    encode_selection(&mut writer, &view.query.reader)?;
    encode_qualified(&mut writer, &view.query.root)?;
    writer.text(&view.query.relation.contract_version)?;
    writer.text(view.query.relation.kind.label())?;
    match &view.query.filter {
        ViewFilter::All => writer.text("all")?,
        ViewFilter::SourceArtifact(artifact) => {
            writer.text("source-artifact")?;
            writer.text(artifact.as_str())?;
        }
    }
    writer.text(match view.query.projection {
        ViewProjection::IdentityAndSource => "identity-and-source",
        ViewProjection::IdentityOnly => "identity-only",
    })?;
    writer.text(view.query.profile.as_str())?;
    encode_limits(&mut writer, view.limits)?;
    writer.text(if view.duration_exhausted {
        "duration-exhausted"
    } else {
        "duration-within-bound"
    })?;
    writer.number(count(view.items.len())?)?;
    for item in &view.items {
        encode_qualified(&mut writer, &item.semantic)?;
        encode_source(&mut writer, &item.attribution)?;
        match &item.projected_source {
            Some(source) => {
                writer.text("some")?;
                encode_source(&mut writer, source)?;
            }
            None => writer.text("none")?,
        }
        writer.text(match item.state {
            ViewItemState::Displayed => "displayed",
            ViewItemState::Omitted => "omitted",
        })?;
    }
    encode_traversal_outcome(&mut writer, &view.outcome)?;
    encode_usage(&mut writer, view.usage)?;
    Ok(writer.finish())
}

fn encode_traversal_outcome(
    writer: &mut CanonicalWriter,
    outcome: &TraversalOutcome,
) -> Result<()> {
    match outcome {
        TraversalOutcome::Complete => writer.text("complete")?,
        TraversalOutcome::Incomplete { cause, frontier } => {
            writer.text("incomplete")?;
            writer.text(match cause {
                BoundExceeded::TraversalSteps => "traversal-steps",
                BoundExceeded::Edges => "edges",
                BoundExceeded::Results => "results",
                BoundExceeded::Duration => "duration",
            })?;
            writer.number(count(frontier.len())?)?;
            for entry in frontier {
                writer.number(entry.depth)?;
                match &entry.predecessor {
                    Some(predecessor) => {
                        writer.text("some")?;
                        encode_qualified(writer, predecessor)?;
                    }
                    None => writer.text("none")?,
                }
                encode_qualified(writer, &entry.target)?;
                writer.text(entry.relation.label())?;
                encode_source(writer, &entry.provenance)?;
            }
        }
    }
    Ok(())
}

fn encode_usage(writer: &mut CanonicalWriter, usage: Usage) -> Result<()> {
    writer.number(usage.input_bytes)?;
    writer.number(usage.reader_invocations)?;
    writer.number(usage.source_artifacts)?;
    writer.number(usage.projected_nodes)?;
    writer.number(usage.projected_edges)?;
    writer.number(usage.traversal_steps)?;
    writer.number(usage.traversed_edges)?;
    writer.number(usage.results)?;
    writer.number(usage.frontier_entries)
}

fn encode_run(run: &ConsumerRun, maximum: u64) -> Result<Vec<u8>> {
    let mut writer = CanonicalWriter::new(maximum);
    writer.text("quire.consumer-accounting/v1")?;
    writer.raw(&run.view_identity)?;
    encode_traversal_outcome(&mut writer, &run.view_outcome)?;
    encode_usage(&mut writer, run.view_usage)?;
    writer.number(run.usage.invocations)?;
    writer.number(run.usage.output_bytes)?;
    writer.number(run.usage.mappings)?;
    writer.text(if run.duration_exhausted {
        "duration-exhausted"
    } else {
        "duration-within-bound"
    })?;
    writer.text(&run.lock_identity.to_hex())?;
    encode_selection(&mut writer, &run.selection)?;
    writer.text(run.profile.as_str())?;
    writer.text(run.profile_artifact.as_str())?;
    writer.text(match run.region_encoding {
        RegionEncoding::Utf8 => "utf8",
        RegionEncoding::Binary => "binary",
    })?;
    writer.text(run.kind.label())?;
    encode_limits(&mut writer, run.limits)?;
    writer.text(match run.state {
        CoordinationState::Complete => "complete",
        CoordinationState::Incomplete => "incomplete",
        CoordinationState::Refused => "refused",
        CoordinationState::Failed => "failed",
    })?;
    writer.number(count(run.records.len())?)?;
    for record in &run.records {
        writer.text(record.obligation.as_str())?;
        encode_qualified(&mut writer, &record.semantic)?;
        encode_source(&mut writer, &record.source)?;
        match record.requested_output_bound {
            Some(bound) => {
                writer.text("some")?;
                writer.number(bound)?;
            }
            None => writer.text("none")?,
        }
        match record.mapping_loss {
            Some(loss) => {
                writer.text("some")?;
                writer.text(loss.label())?;
            }
            None => writer.text("none")?,
        }
        match &record.outcome {
            AccountingOutcome::Displayed { bytes, regions } => {
                writer.text("displayed")?;
                writer.field(bytes)?;
                encode_regions(&mut writer, regions)?;
            }
            AccountingOutcome::Omitted { reason } => {
                writer.text("omitted")?;
                writer.text(reason)?;
            }
            AccountingOutcome::Generated {
                bytes,
                digest,
                regions,
            } => {
                writer.text("generated")?;
                writer.field(bytes)?;
                writer.raw(digest)?;
                encode_regions(&mut writer, regions)?;
            }
            AccountingOutcome::Unsupported { reason } => {
                writer.text("unsupported")?;
                writer.text(reason)?;
            }
            AccountingOutcome::RequiresBound { resource, minimum } => {
                writer.text("requires-bound")?;
                writer.text(match resource {
                    RequiredBound::ConsumerResults => "consumer-results",
                    RequiredBound::OutputBytes => "output-bytes",
                    RequiredBound::Mappings => "mappings",
                    RequiredBound::Invocations => "invocations",
                    RequiredBound::Duration => "duration",
                })?;
                writer.number(*minimum)?;
            }
            AccountingOutcome::Invalid { code } => {
                writer.text("invalid")?;
                writer.text(match code {
                    InvalidCode::ForeignSemantic => "foreign-semantic",
                    InvalidCode::InventoryMismatch => "inventory-mismatch",
                    InvalidCode::AttributionMismatch => "attribution-mismatch",
                    InvalidCode::MalformedMapping => "malformed-mapping",
                })?;
            }
            AccountingOutcome::Failed {
                code,
                consumer_code,
            } => {
                writer.text("failed")?;
                writer.text(match code {
                    FailureCode::ConsumerFailed => "consumer-failed",
                    FailureCode::ConsumerExhausted => "consumer-exhausted",
                })?;
                match consumer_code {
                    Some(code) => {
                        writer.text("some")?;
                        writer.text(code)?;
                    }
                    None => writer.text("none")?,
                }
            }
        }
    }
    Ok(writer.finish())
}

fn encode_regions(writer: &mut CanonicalWriter, regions: &[OutputRegion]) -> Result<()> {
    writer.number(count(regions.len())?)?;
    for region in regions {
        writer.number(region.start)?;
        writer.number(region.end)?;
    }
    Ok(())
}

fn encode_limits(writer: &mut CanonicalWriter, limits: Limits) -> Result<()> {
    writer.number(limits.max_identity_bytes)?;
    writer.number(limits.max_input_bytes)?;
    writer.number(limits.max_nodes)?;
    writer.number(limits.max_source_artifacts)?;
    writer.number(limits.max_projection_edges)?;
    writer.number(limits.max_reader_invocations)?;
    writer.number(limits.max_edges)?;
    writer.number(limits.max_traversal_steps)?;
    writer.number(limits.max_results)?;
    writer.number(limits.max_frontier)?;
    writer.number(limits.max_obligations)?;
    writer.number(limits.max_consumer_invocations)?;
    writer.number(limits.max_mappings)?;
    writer.number(limits.max_output_bytes)?;
    writer.number(limits.max_accounting_bytes)?;
    writer.number(limits.max_duration_ms)
}
