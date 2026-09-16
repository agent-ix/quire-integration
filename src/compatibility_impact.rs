// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-005 compatibility-impact, stale-evidence and regression-vector coordination.
//!
//! This module owns only feature-local projections and overlays. Exact owner readers
//! remain independently selected capabilities, and no graph or presentation value can
//! manufacture compatibility, evidence, qualification or release authority.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sha2::{Digest as _, Sha256};

use crate::ecosystem_lock::{AdmittedCapability, CapabilitySelection, LockIdentity, ValidatedLock};
use crate::{ArtifactId, ProviderId};

/// Feature-local comparison projection contract.
pub const COMPARISON_CONTRACT: &str = "quire.impact-comparison/v1";
/// Feature-local evidence-graph projection contract.
pub const GRAPH_CONTRACT: &str = "quire.impact-evidence-graph/v1";
/// FR-336 conformance-corpus contract consumed by this feature.
pub const CORPUS_CONTRACT: &str = "quire.conformance-corpus/v1";
/// FR-337 evidence-intake contract consumed by this feature.
pub const EVIDENCE_CONTRACT: &str = "quire.evidence-intake/v1";
/// Feature-local impact-policy contract.
pub const POLICY_CONTRACT: &str = "quire.impact-policy/v1";
/// Feature-local regression-vector manifest contract.
pub const MANIFEST_CONTRACT: &str = "quire.regression-vector-manifest/v1";

/// Stable failure category for the compatibility-impact boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// An identity is empty or exceeds its caller-lowered ceiling.
    InvalidIdentity,
    /// A named finite resource ceiling was exceeded.
    ResourceExhausted,
    /// Checked arithmetic or a platform conversion failed.
    IntegerOverflow,
    /// Input is malformed, duplicated, dangling or non-canonical.
    Malformed,
    /// Input belongs to another lock, authority or contract.
    ForeignInput,
    /// An exact selected reader is not admitted by its selected lock.
    CapabilityMismatch,
    /// An admitted reader failed before producing a validated projection.
    ReaderFailure,
    /// An admitted owner tuple contradicts the closed tuple table.
    OwnerContradiction,
    /// Operational duration expired; no envelope exists.
    DurationExpired,
    /// Canonical bytes or a declared digest do not match.
    CanonicalMismatch,
}

/// Exact finite dimension associated with a refusal or incomplete traversal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Resource {
    /// Bytes in one input document.
    DocumentBytes,
    /// Bytes across all input documents.
    TotalInputBytes,
    /// Bytes in one identity.
    IdentityBytes,
    /// Bytes in one bounded reason.
    ReasonBytes,
    /// Comparison correspondences.
    Correspondences,
    /// Owner assessments.
    Assessments,
    /// Changed roots.
    ChangedRoots,
    /// Graph nodes.
    GraphNodes,
    /// Graph edges.
    GraphEdges,
    /// Evidence records.
    EvidenceRecords,
    /// Evidence rights.
    EvidenceRights,
    /// Evidence limitations.
    EvidenceLimitations,
    /// Strict-reader invocations.
    ReaderInvocations,
    /// Policy seed rules.
    PolicySeedRules,
    /// Policy propagation rules.
    PolicyPropagationRules,
    /// Policy vector-selection rules.
    PolicyVectorRules,
    /// Vector definitions.
    VectorDefinitions,
    /// Provider-manifest entries.
    ManifestEntries,
    /// Provider selections in the complete manifest inventory.
    ManifestProviders,
    /// Unresolved overlay entries.
    UnresolvedEntries,
    /// Bytes across all bounded reasons.
    TotalReasonBytes,
    /// Atomic topology items.
    TopologyItems,
    /// Traversal node expansions.
    TraversalSteps,
    /// Traversed effect-carrying edges.
    TraversedEdges,
    /// Retained root/subject overlays.
    SubjectRecords,
    /// Retained path edges.
    PathEdges,
    /// Retained witnesses.
    Witnesses,
    /// Encoded witness bytes.
    WitnessBytes,
    /// Retained traversal frontier entries.
    FrontierEntries,
    /// Canonical body bytes.
    OutputBytes,
    /// Complete accounting-envelope bytes.
    AccountingBytes,
    /// Operational monotonic duration.
    Duration,
}

/// Typed fail-closed compatibility-impact error.
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

    /// Stable machine-readable category.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Exact exhausted resource, when applicable.
    #[must_use]
    pub const fn resource(&self) -> Option<Resource> {
        self.resource
    }

    /// Bounded non-authoritative diagnostic.
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for Error {}

/// Compatibility-impact result.
pub type Result<T> = std::result::Result<T, Error>;

/// Closed caller-lowered finite ceilings for one impact request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Bytes in each canonical input document.
    pub max_document_bytes: u64,
    /// Bytes across all canonical input documents.
    pub max_total_input_bytes: u64,
    /// Bytes in each identity.
    pub max_identity_bytes: u64,
    /// Bytes in each reason.
    pub max_reason_bytes: u64,
    /// Comparison correspondences.
    pub max_correspondences: u64,
    /// Owner assessments.
    pub max_assessments: u64,
    /// Changed roots.
    pub max_changed_roots: u64,
    /// Graph nodes.
    pub max_graph_nodes: u64,
    /// Graph edges.
    pub max_graph_edges: u64,
    /// Evidence records.
    pub max_evidence_records: u64,
    /// Evidence rights.
    pub max_evidence_rights: u64,
    /// Evidence limitations.
    pub max_evidence_limitations: u64,
    /// Strict-reader calls.
    pub max_reader_invocations: u64,
    /// Policy seed rules.
    pub max_policy_seed_rules: u64,
    /// Policy propagation rules.
    pub max_policy_propagation_rules: u64,
    /// Policy vector-selection rules.
    pub max_policy_vector_rules: u64,
    /// Vector definitions.
    pub max_vector_definitions: u64,
    /// Manifest entries.
    pub max_manifest_entries: u64,
    /// Provider selections in the complete manifest inventory.
    pub max_manifest_providers: u64,
    /// Unresolved entries.
    pub max_unresolved_entries: u64,
    /// Bytes across all reasons.
    pub max_total_reason_bytes: u64,
    /// Atomic graph-topology items.
    pub max_topology_items: u64,
    /// Traversal expansions.
    pub max_traversal_steps: u64,
    /// Traversed edges.
    pub max_traversed_edges: u64,
    /// Retained subject overlays.
    pub max_subject_records: u64,
    /// Retained path edges.
    pub max_path_edges: u64,
    /// Retained witness records.
    pub max_witnesses: u64,
    /// Encoded witness bytes.
    pub max_witness_bytes: u64,
    /// Retained frontier entries.
    pub max_frontier_entries: u64,
    /// Canonical body bytes.
    pub max_output_bytes: u64,
    /// Complete envelope bytes.
    pub max_accounting_bytes: u64,
    /// Operational duration in monotonic milliseconds.
    pub max_duration_ms: u64,
}

impl Limits {
    fn validate(self) -> Result<()> {
        let values = [
            (self.max_document_bytes, Resource::DocumentBytes),
            (self.max_total_input_bytes, Resource::TotalInputBytes),
            (self.max_identity_bytes, Resource::IdentityBytes),
            (self.max_reason_bytes, Resource::ReasonBytes),
            (self.max_correspondences, Resource::Correspondences),
            (self.max_assessments, Resource::Assessments),
            (self.max_changed_roots, Resource::ChangedRoots),
            (self.max_graph_nodes, Resource::GraphNodes),
            (self.max_graph_edges, Resource::GraphEdges),
            (self.max_evidence_records, Resource::EvidenceRecords),
            (self.max_evidence_rights, Resource::EvidenceRights),
            (self.max_evidence_limitations, Resource::EvidenceLimitations),
            (self.max_reader_invocations, Resource::ReaderInvocations),
            (self.max_policy_seed_rules, Resource::PolicySeedRules),
            (
                self.max_policy_propagation_rules,
                Resource::PolicyPropagationRules,
            ),
            (self.max_policy_vector_rules, Resource::PolicyVectorRules),
            (self.max_vector_definitions, Resource::VectorDefinitions),
            (self.max_manifest_entries, Resource::ManifestEntries),
            (self.max_manifest_providers, Resource::ManifestProviders),
            (self.max_unresolved_entries, Resource::UnresolvedEntries),
            (self.max_total_reason_bytes, Resource::TotalReasonBytes),
            (self.max_topology_items, Resource::TopologyItems),
            (self.max_traversal_steps, Resource::TraversalSteps),
            (self.max_traversed_edges, Resource::TraversedEdges),
            (self.max_subject_records, Resource::SubjectRecords),
            (self.max_path_edges, Resource::PathEdges),
            (self.max_witnesses, Resource::Witnesses),
            (self.max_witness_bytes, Resource::WitnessBytes),
            (self.max_frontier_entries, Resource::FrontierEntries),
            (self.max_output_bytes, Resource::OutputBytes),
            (self.max_accounting_bytes, Resource::AccountingBytes),
            (self.max_duration_ms, Resource::Duration),
        ];
        if let Some((_, resource)) = values.into_iter().find(|(value, _)| *value == 0) {
            return Err(Error::exhausted(resource, "impact limit must be nonzero"));
        }
        Ok(())
    }
}

macro_rules! local_identity {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Constructs one bounded nonempty exact identity.
            pub fn new(value: impl AsRef<str>, max_bytes: u64) -> Result<Self> {
                let value = value.as_ref();
                let length = count(value.len())?;
                if length > max_bytes || value.trim().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidIdentity,
                        "invalid bounded impact identity",
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

local_identity!(AuthorityId, "Exact semantic authority identity.");
local_identity!(RevisionId, "Exact immutable owner revision identity.");
local_identity!(SubjectId, "Owner-local semantic subject identity.");
local_identity!(CorrespondenceId, "Directional correspondence identity.");
local_identity!(RuleId, "Impact-policy or comparison-rule identity.");
local_identity!(ChangeSetId, "Exact owner change-set identity.");
local_identity!(RelationId, "Exact owner relation identity.");
local_identity!(ScopeId, "Exact owner scope identity.");
local_identity!(ProvenanceId, "Exact owner provenance identity.");
local_identity!(EvidenceId, "Exact FR-337 evidence identity.");
local_identity!(VectorId, "Exact FR-336 vector-definition identity.");
local_identity!(MethodId, "Exact verification-method identity.");
local_identity!(EnvironmentId, "Exact execution-environment identity.");
local_identity!(ProviderRoleId, "Exact requested provider-role identity.");
local_identity!(PolicyProfileId, "Exact immutable policy-profile identity.");

/// Closed semantic subject kind used by the feature-local graph.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubjectKind {
    /// Requirement subject.
    Requirement,
    /// Assumption subject.
    Assumption,
    /// Semantic model subject.
    Model,
    /// Implementation subject.
    Implementation,
    /// Evidence subject.
    Evidence,
    /// Provider subject.
    Provider,
}

/// Complete authority-qualified owner subject identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QualifiedSubjectId {
    /// Closed owner subject kind.
    pub kind: SubjectKind,
    /// Exact owner authority.
    pub authority: AuthorityId,
    /// Exact immutable owner revision.
    pub revision: RevisionId,
    /// Owner-local identity.
    pub local: SubjectId,
}

/// Independent structural-assessment axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StructuralAssessment {
    /// No structural change was reported.
    Unchanged,
    /// Additive structurally compatible change.
    AdditiveCompatible,
    /// Payload-breaking structural change.
    PayloadBreaking,
    /// Baseline subject was removed.
    Removed,
    /// Owner could not establish structural state.
    Unknown,
}

/// Independent behavioral-assessment axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BehavioralAssessment {
    /// No behavioral assessment was requested.
    NotAssessed,
    /// Comparable results conform.
    Conforming,
    /// Comparable results regress.
    Regressed,
    /// Closed comparable inventory lacks the candidate result.
    Absent,
    /// Selected method does not support the behavior.
    Unsupported,
    /// Required provider is unavailable.
    Unavailable,
    /// Owner could not establish behavioral state.
    Unknown,
}

/// Independent directional subject-presence axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubjectPresence {
    /// Subject exists under both locks.
    BothPresent,
    /// Subject exists only under the baseline lock.
    BaselineOnly,
    /// Subject exists only under the candidate lock.
    CandidateOnly,
    /// Presence could not be established.
    Unknown,
}

/// Closed comparable-result evidence state used to validate behavioral claims.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComparableResults {
    /// Comparable baseline and candidate results both exist.
    Both,
    /// Baseline result exists and the closed inventory lacks the candidate result.
    CandidateMissing,
    /// No comparable-result claim is attached.
    NotApplicable,
}

/// Exact closed key grounding one comparable behavioral-result inventory entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ComparableResultKey {
    /// Exact directional subject being compared.
    pub subject: QualifiedSubjectId,
    /// Exact regression-vector definition.
    pub vector: VectorId,
    /// Exact comparison method.
    pub method: MethodId,
    /// Exact result provider.
    pub provider: ProviderId,
    /// Exact execution environment.
    pub environment: EnvironmentId,
    /// Exact immutable assumptions artifact.
    pub assumptions: ArtifactId,
    /// Exact qualified comparison domain.
    pub domain: ScopeId,
    /// Exact baseline lock.
    pub baseline_lock: LockIdentity,
    /// Exact candidate lock.
    pub candidate_lock: LockIdentity,
}

/// One directional owner correspondence and independent assessment tuple.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Correspondence {
    /// Stable directional correspondence identity.
    pub id: CorrespondenceId,
    /// Optional exact baseline subject.
    pub baseline: Option<QualifiedSubjectId>,
    /// Optional exact candidate subject.
    pub candidate: Option<QualifiedSubjectId>,
    /// Structural owner assessment.
    pub structural: StructuralAssessment,
    /// Behavioral owner assessment.
    pub behavioral: BehavioralAssessment,
    /// Comparable-result existence proof.
    pub comparable_results: ComparableResults,
    /// Exact closed comparable-result key, absent only when no comparison is asserted.
    pub comparable_key: Option<ComparableResultKey>,
    /// Directional presence assessment.
    pub presence: SubjectPresence,
}

/// Strict-reader comparison projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonProjection {
    /// Exact baseline lock identity declared by the owner projection.
    pub baseline_lock: LockIdentity,
    /// Exact candidate lock identity declared by the owner projection.
    pub candidate_lock: LockIdentity,
    /// Exact comparison-rule identity.
    pub comparison_rule: RuleId,
    /// Exact comparison-rule revision.
    pub rule_revision: RevisionId,
    /// Declared comparison-rule digest.
    pub rule_digest: [u8; 32],
    /// Exact change-set identity.
    pub change_set: ChangeSetId,
    /// Declared change-set digest.
    pub change_set_digest: [u8; 32],
    /// Directional owner correspondences.
    pub correspondences: Vec<Correspondence>,
}

/// One exact owner graph node.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GraphNode {
    /// Complete owner subject identity.
    pub id: QualifiedSubjectId,
    /// Referenced FR-337 evidence identity, when this is evidence.
    pub evidence: Option<EvidenceId>,
}

/// One exact typed owner graph edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GraphEdge {
    /// Exact source subject.
    pub source: QualifiedSubjectId,
    /// Exact owner relation.
    pub relation: RelationId,
    /// Exact target subject.
    pub target: QualifiedSubjectId,
    /// Exact owner scope.
    pub scope: ScopeId,
    /// Exact owner provenance.
    pub provenance: ProvenanceId,
}

/// Separate valid unresolved-dependency overlay entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnresolvedDependency {
    /// Exact changed root.
    pub root: QualifiedSubjectId,
    /// Exact predecessor known to the graph.
    pub predecessor: QualifiedSubjectId,
    /// Complete absent target identity.
    pub target: QualifiedSubjectId,
    /// Exact owner relation.
    pub relation: RelationId,
    /// Exact owner scope.
    pub scope: ScopeId,
    /// Exact owner provenance.
    pub provenance: ProvenanceId,
    /// Bounded reason.
    pub reason: String,
    /// Digest over every preceding field.
    pub digest: [u8; 32],
}

/// Strict-reader feature-local graph projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphProjection {
    /// Closed admitted semantic-authority inventory.
    pub authorities: Vec<AuthorityId>,
    /// Closed admitted owner-relation inventory.
    pub relations: Vec<RelationId>,
    /// Closed admitted qualified-scope inventory.
    pub scopes: Vec<ScopeId>,
    /// Closed admitted provenance inventory.
    pub provenances: Vec<ProvenanceId>,
    /// Exact owner graph nodes.
    pub nodes: Vec<GraphNode>,
    /// Exact owner graph edges.
    pub edges: Vec<GraphEdge>,
    /// Separate unresolved overlays.
    pub unresolved: Vec<UnresolvedDependency>,
}

/// One immutable FR-336 vector definition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VectorDefinition {
    /// Exact vector identity.
    pub id: VectorId,
    /// Unchanged canonical owner bytes.
    pub owner_bytes: Vec<u8>,
    /// Raw SHA-256 of `owner_bytes`.
    pub digest: [u8; 32],
}

/// Strict-reader FR-336 corpus projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusProjection {
    /// Complete immutable vector catalog.
    pub vectors: Vec<VectorDefinition>,
}

/// Exact owner evidence relation, independent of freshness.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EvidenceRelation {
    /// Owner evidence supports its subject.
    Supports,
    /// Owner evidence contradicts its subject.
    Contradicts,
}

/// Input evidence freshness before this comparison.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Freshness {
    /// Evidence is current for the exact baseline revision.
    Current,
    /// Evidence was already stale.
    Stale,
    /// Freshness was not established.
    Unknown,
}

/// One immutable FR-337 evidence-intake record.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvidenceRecord {
    /// Exact evidence identity.
    pub id: EvidenceId,
    /// Exact baseline subject revision the record concerns.
    pub subject: QualifiedSubjectId,
    /// Unchanged owner relation.
    pub relation: EvidenceRelation,
    /// Unchanged canonical owner bytes.
    pub owner_bytes: Vec<u8>,
    /// Exact owner scope.
    pub scope: ScopeId,
    /// Exact owner provenance.
    pub provenance: ProvenanceId,
    /// Immutable rights entries.
    pub rights: Vec<String>,
    /// Immutable limitation entries.
    pub limitations: Vec<String>,
    /// Prior freshness overlay.
    pub freshness: Freshness,
}

/// Strict-reader FR-337 evidence projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceProjection {
    /// Complete retained evidence inventory.
    pub records: Vec<EvidenceRecord>,
}

/// Independent propagation effect.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Effect {
    /// Potential impact reachability.
    Affectedness,
    /// Evidence-staleness reachability.
    Staleness,
}

/// Closed owner tuple selected by one seed rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssessmentTuple {
    /// Structural axis.
    pub structural: StructuralAssessment,
    /// Behavioral axis.
    pub behavioral: BehavioralAssessment,
    /// Presence axis.
    pub presence: SubjectPresence,
}

/// One exact seed-effect policy rule.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SeedRule {
    /// Stable rule identity.
    pub id: RuleId,
    /// Exact admitted assessment tuple.
    pub assessment: AssessmentTuple,
    /// Explicit initial effect subset.
    pub effects: BTreeSet<Effect>,
}

/// Closed feature-local edge orientation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Direction {
    /// Preserve owner source-to-target orientation.
    Forward,
    /// Traverse exact owner target-to-source orientation.
    Reverse,
}

/// One exact-only effect propagation rule.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PropagationRule {
    /// Stable rule identity.
    pub id: RuleId,
    /// Exact owner relation.
    pub relation: RelationId,
    /// Exact source kind after orientation.
    pub source_kind: SubjectKind,
    /// Exact target kind after orientation.
    pub target_kind: SubjectKind,
    /// Explicit direction.
    pub direction: Direction,
    /// Exact-only qualified scope matcher.
    pub scope: ScopeId,
    /// Independent transferred effect subset.
    pub effects: BTreeSet<Effect>,
}

/// Requirement state selected independently of availability.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VectorRequirement {
    /// Policy requires execution later in qualification.
    Required,
    /// Policy retains the vector but does not require execution.
    NotApplicable,
}

/// Complete vector record key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VectorKey {
    /// FR-336 definition identity.
    pub vector: VectorId,
    /// Exact subject.
    pub subject: QualifiedSubjectId,
    /// Exact scope.
    pub scope: ScopeId,
    /// Exact method.
    pub method: MethodId,
    /// Exact environment.
    pub environment: EnvironmentId,
    /// Requested provider role.
    pub provider_role: ProviderRoleId,
    /// Exact policy profile.
    pub profile: PolicyProfileId,
}

/// One policy contribution to a vector key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VectorRule {
    /// Stable contributing rule identity.
    pub id: RuleId,
    /// Complete vector record key.
    pub key: VectorKey,
    /// Independent requirement state.
    pub requirement: VectorRequirement,
}

/// Strict-reader pure immutable impact policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImpactPolicy {
    /// Exact policy identity.
    pub identity: PolicyProfileId,
    /// Exact immutable revision.
    pub revision: RevisionId,
    /// Declared digest.
    pub digest: [u8; 32],
    /// Closed seed rules.
    pub seed_rules: Vec<SeedRule>,
    /// Closed propagation rules.
    pub propagation_rules: Vec<PropagationRule>,
    /// Closed vector rules.
    pub vector_rules: Vec<VectorRule>,
}

/// Provider-manifest status for one vector.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ManifestStatus {
    /// Provider supports the vector.
    Supported,
    /// Provider explicitly excludes the vector.
    Excluded,
    /// Provider reports a bounded indeterminate reason.
    Unknown(String),
}

/// One exact provider-manifest entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ManifestEntry {
    /// Exact candidate-lock provider capability owning this entry.
    pub provider: CapabilitySelection,
    /// Exact vector definition.
    pub vector: VectorId,
    /// Provider-declared status.
    pub status: ManifestStatus,
}

/// Strict-reader regression-vector manifest projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestProjection {
    /// Complete candidate-lock provider capability inventory described by the manifest.
    pub providers: Vec<CapabilitySelection>,
    /// Complete manifest inventory.
    pub entries: Vec<ManifestEntry>,
}

/// Closed strict-reader input contract.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InputContract {
    /// Feature-local comparison projection.
    Comparison,
    /// Feature-local evidence graph.
    Graph,
    /// FR-336 vector corpus.
    Corpus,
    /// FR-337 evidence intake.
    Evidence,
    /// Feature-local impact policy.
    Policy,
    /// Feature-local provider manifest.
    Manifest,
}

impl InputContract {
    /// Exact wire-contract label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Comparison => COMPARISON_CONTRACT,
            Self::Graph => GRAPH_CONTRACT,
            Self::Corpus => CORPUS_CONTRACT,
            Self::Evidence => EVIDENCE_CONTRACT,
            Self::Policy => POLICY_CONTRACT,
            Self::Manifest => MANIFEST_CONTRACT,
        }
    }

    const fn ordinal(self) -> u8 {
        match self {
            Self::Comparison => 0,
            Self::Graph => 1,
            Self::Corpus => 2,
            Self::Evidence => 3,
            Self::Policy => 4,
            Self::Manifest => 5,
        }
    }
}

/// Lock under which one strict reader must be admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockSide {
    /// Exact baseline lock.
    Baseline,
    /// Exact candidate lock.
    Candidate,
}

/// Canonical untrusted bytes supplied to one selected strict reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerDocument {
    /// Declared exact contract.
    pub contract: InputContract,
    /// Canonical owner/facade bytes.
    pub canonical_bytes: Vec<u8>,
    /// Raw SHA-256 of `canonical_bytes`.
    pub digest: [u8; 32],
}

impl OwnerDocument {
    /// Constructs a document with its exact raw-byte digest.
    #[must_use]
    pub fn new(contract: InputContract, canonical_bytes: Vec<u8>) -> Self {
        let digest = Sha256::digest(&canonical_bytes).into();
        Self {
            contract,
            canonical_bytes,
            digest,
        }
    }
}

/// One exact reader selection and its untrusted document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderInput {
    /// Required input contract.
    pub contract: InputContract,
    /// Exact selected pure reader capability.
    pub selection: CapabilitySelection,
    /// Lock that must admit the selection.
    pub lock: LockSide,
    /// Canonical input document.
    pub document: OwnerDocument,
}

/// Projection emitted only through a bounded coordinator-owned sink.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Projection {
    /// Comparison projection.
    Comparison(ComparisonProjection),
    /// Evidence graph projection.
    Graph(GraphProjection),
    /// FR-336 corpus projection.
    Corpus(CorpusProjection),
    /// FR-337 evidence projection.
    Evidence(EvidenceProjection),
    /// Impact policy projection.
    Policy(ImpactPolicy),
    /// Provider manifest projection.
    Manifest(ManifestProjection),
}

/// Fixed metadata that starts one incrementally retained strict-reader projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionHeader {
    /// Comparison metadata preceding directional correspondences.
    Comparison {
        /// Exact baseline lock.
        baseline_lock: LockIdentity,
        /// Exact candidate lock.
        candidate_lock: LockIdentity,
        /// Comparison-rule identity.
        comparison_rule: RuleId,
        /// Comparison-rule revision.
        rule_revision: RevisionId,
        /// Comparison-rule digest.
        rule_digest: [u8; 32],
        /// Change-set identity.
        change_set: ChangeSetId,
        /// Change-set digest.
        change_set_digest: [u8; 32],
    },
    /// Evidence-graph projection with incrementally emitted owner vocabulary.
    Graph,
    /// FR-336 corpus projection with no fixed metadata.
    Corpus,
    /// FR-337 evidence projection with no fixed metadata.
    Evidence,
    /// Impact-policy metadata preceding its closed rules.
    Policy {
        /// Policy profile identity.
        identity: PolicyProfileId,
        /// Policy revision.
        revision: RevisionId,
        /// Policy digest.
        digest: [u8; 32],
    },
    /// Provider-manifest metadata preceding its vector entries.
    Manifest,
}

impl ProjectionHeader {
    const fn contract(&self) -> InputContract {
        match self {
            Self::Comparison { .. } => InputContract::Comparison,
            Self::Graph => InputContract::Graph,
            Self::Corpus => InputContract::Corpus,
            Self::Evidence => InputContract::Evidence,
            Self::Policy { .. } => InputContract::Policy,
            Self::Manifest => InputContract::Manifest,
        }
    }
}

/// One incrementally retained item emitted by a selected strict reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionItem {
    /// Directional comparison correspondence.
    Correspondence(Correspondence),
    /// Admitted graph authority vocabulary member.
    GraphAuthority(AuthorityId),
    /// Admitted graph relation vocabulary member.
    GraphRelation(RelationId),
    /// Admitted graph scope vocabulary member.
    GraphScope(ScopeId),
    /// Admitted graph provenance vocabulary member.
    GraphProvenance(ProvenanceId),
    /// Evidence-graph node.
    GraphNode(GraphNode),
    /// Evidence-graph edge.
    GraphEdge(GraphEdge),
    /// Separate unresolved-dependency overlay.
    Unresolved(UnresolvedDependency),
    /// Immutable FR-336 vector definition.
    VectorDefinition(VectorDefinition),
    /// Immutable FR-337 evidence record.
    EvidenceRecord(EvidenceRecord),
    /// Policy seed rule.
    SeedRule(SeedRule),
    /// Policy propagation rule.
    PropagationRule(PropagationRule),
    /// Policy vector rule.
    VectorRule(VectorRule),
    /// Provider-manifest entry.
    ManifestEntry(ManifestEntry),
    /// One exact provider capability in the complete manifest inventory.
    ManifestProvider(CapabilitySelection),
}

impl Projection {
    /// Exact contract represented by this projection.
    #[must_use]
    pub const fn contract(&self) -> InputContract {
        match self {
            Self::Comparison(_) => InputContract::Comparison,
            Self::Graph(_) => InputContract::Graph,
            Self::Corpus(_) => InputContract::Corpus,
            Self::Evidence(_) => InputContract::Evidence,
            Self::Policy(_) => InputContract::Policy,
            Self::Manifest(_) => InputContract::Manifest,
        }
    }
}

/// Lowered immutable budget supplied to a selected strict reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderBudget {
    limits: Limits,
    remaining_duration_ms: u64,
}

impl ReaderBudget {
    /// Exact caller-lowered limits.
    #[must_use]
    pub const fn limits(self) -> Limits {
        self.limits
    }

    /// Remaining total operational duration at the call boundary.
    #[must_use]
    pub const fn remaining_duration_ms(self) -> u64 {
        self.remaining_duration_ms
    }
}

/// Stable admitted strict-reader failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderError {
    /// Owner bytes are invalid.
    Invalid,
    /// Reader exhausted one supplied bound.
    Exhausted(Resource),
    /// Reader failed after exact admission.
    Failed,
}

/// Coordinator-owned sink that admits exactly one contract-matching projection.
pub struct ProjectionSink {
    expected: InputContract,
    limits: Limits,
    builder: Option<ProjectionBuilder>,
    retained_bytes: u64,
    total_reason_bytes: u64,
    reason_budget: u64,
    evidence_rights: u64,
    evidence_limitations: u64,
    failure: Option<ReaderError>,
}

enum ProjectionBuilder {
    Comparison(ComparisonProjection),
    Graph(GraphProjection),
    Corpus(CorpusProjection),
    Evidence(EvidenceProjection),
    Policy(ImpactPolicy),
    Manifest(ManifestProjection),
}

impl ProjectionSink {
    fn new(expected: InputContract, limits: Limits, reason_budget: u64) -> Self {
        Self {
            expected,
            limits,
            builder: None,
            retained_bytes: 0,
            total_reason_bytes: 0,
            reason_budget,
            evidence_rights: 0,
            evidence_limitations: 0,
            failure: None,
        }
    }

    /// Starts exactly one projection after checking its fixed metadata byte demand.
    pub fn begin(&mut self, header: ProjectionHeader) -> std::result::Result<(), ReaderError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.begin_inner(header);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }

    fn begin_inner(&mut self, header: ProjectionHeader) -> std::result::Result<(), ReaderError> {
        if header.contract() != self.expected || self.builder.is_some() {
            return Err(ReaderError::Invalid);
        }
        validate_projection_header(&header, self.limits)?;
        self.charge_retained_bytes(encoded_projection_header_len(&header)?)?;
        self.builder = Some(match header {
            ProjectionHeader::Comparison {
                baseline_lock,
                candidate_lock,
                comparison_rule,
                rule_revision,
                rule_digest,
                change_set,
                change_set_digest,
            } => ProjectionBuilder::Comparison(ComparisonProjection {
                baseline_lock,
                candidate_lock,
                comparison_rule,
                rule_revision,
                rule_digest,
                change_set,
                change_set_digest,
                correspondences: Vec::new(),
            }),
            ProjectionHeader::Graph => ProjectionBuilder::Graph(GraphProjection {
                authorities: Vec::new(),
                relations: Vec::new(),
                scopes: Vec::new(),
                provenances: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                unresolved: Vec::new(),
            }),
            ProjectionHeader::Corpus => ProjectionBuilder::Corpus(CorpusProjection {
                vectors: Vec::new(),
            }),
            ProjectionHeader::Evidence => ProjectionBuilder::Evidence(EvidenceProjection {
                records: Vec::new(),
            }),
            ProjectionHeader::Policy {
                identity,
                revision,
                digest,
            } => ProjectionBuilder::Policy(ImpactPolicy {
                identity,
                revision,
                digest,
                seed_rules: Vec::new(),
                propagation_rules: Vec::new(),
                vector_rules: Vec::new(),
            }),
            ProjectionHeader::Manifest => ProjectionBuilder::Manifest(ManifestProjection {
                providers: Vec::new(),
                entries: Vec::new(),
            }),
        });
        Ok(())
    }

    /// Retains one item only after its byte and applicable collection ceilings succeed.
    pub fn push(&mut self, item: ProjectionItem) -> std::result::Result<(), ReaderError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.push_inner(item);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }

    fn push_inner(&mut self, item: ProjectionItem) -> std::result::Result<(), ReaderError> {
        let item_bytes = encoded_projection_item_len(&item)?;
        self.preflight_item(&item, item_bytes)?;
        match (self.builder.as_mut(), item) {
            (Some(ProjectionBuilder::Comparison(value)), ProjectionItem::Correspondence(item)) => {
                value.correspondences.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphAuthority(item)) => {
                value.authorities.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphRelation(item)) => {
                value.relations.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphScope(item)) => {
                value.scopes.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphProvenance(item)) => {
                value.provenances.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphNode(item)) => {
                value.nodes.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphEdge(item)) => {
                value.edges.push(item);
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::Unresolved(item)) => {
                self.total_reason_bytes = checked_reader_add(
                    self.total_reason_bytes,
                    item.reason.len(),
                    Resource::TotalReasonBytes,
                )?;
                value.unresolved.push(item);
            }
            (Some(ProjectionBuilder::Corpus(value)), ProjectionItem::VectorDefinition(item)) => {
                value.vectors.push(item);
            }
            (Some(ProjectionBuilder::Evidence(value)), ProjectionItem::EvidenceRecord(item)) => {
                self.evidence_rights = checked_reader_add(
                    self.evidence_rights,
                    item.rights.len(),
                    Resource::EvidenceRights,
                )?;
                self.evidence_limitations = checked_reader_add(
                    self.evidence_limitations,
                    item.limitations.len(),
                    Resource::EvidenceLimitations,
                )?;
                value.records.push(item);
            }
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::SeedRule(item)) => {
                value.seed_rules.push(item);
            }
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::PropagationRule(item)) => {
                value.propagation_rules.push(item)
            }
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::VectorRule(item)) => {
                value.vector_rules.push(item);
            }
            (Some(ProjectionBuilder::Manifest(value)), ProjectionItem::ManifestEntry(item)) => {
                if let ManifestStatus::Unknown(reason) = &item.status {
                    self.total_reason_bytes = checked_reader_add(
                        self.total_reason_bytes,
                        reason.len(),
                        Resource::TotalReasonBytes,
                    )?;
                }
                value.entries.push(item);
            }
            (Some(ProjectionBuilder::Manifest(value)), ProjectionItem::ManifestProvider(item)) => {
                value.providers.push(item);
            }
            _ => return Err(ReaderError::Invalid),
        }
        self.retained_bytes = self
            .retained_bytes
            .checked_add(item_bytes)
            .ok_or_else(|| reader_exhausted(Resource::DocumentBytes))?;
        Ok(())
    }

    fn charge_retained_bytes(&mut self, bytes: u64) -> std::result::Result<(), ReaderError> {
        let next = self
            .retained_bytes
            .checked_add(bytes)
            .ok_or_else(|| reader_exhausted(Resource::DocumentBytes))?;
        if next > self.limits.max_document_bytes {
            return Err(reader_exhausted(Resource::DocumentBytes));
        }
        self.retained_bytes = next;
        Ok(())
    }

    fn preflight_item(
        &self,
        item: &ProjectionItem,
        item_bytes: u64,
    ) -> std::result::Result<(), ReaderError> {
        validate_projection_item(item, self.limits)?;
        let next_bytes = self
            .retained_bytes
            .checked_add(item_bytes)
            .ok_or_else(|| reader_exhausted(Resource::DocumentBytes))?;
        if next_bytes > self.limits.max_document_bytes {
            return Err(reader_exhausted(Resource::DocumentBytes));
        }
        if let Some(ProjectionBuilder::Graph(graph)) = self.builder.as_ref() {
            let vocabulary = match item {
                ProjectionItem::GraphAuthority(value) => Some((
                    graph.authorities.contains(value),
                    graph.authorities.len(),
                    self.limits
                        .max_graph_nodes
                        .saturating_add(self.limits.max_unresolved_entries),
                    Resource::GraphNodes,
                )),
                ProjectionItem::GraphRelation(value) => Some((
                    graph.relations.contains(value),
                    graph.relations.len(),
                    self.limits
                        .max_graph_edges
                        .saturating_add(self.limits.max_unresolved_entries),
                    Resource::GraphEdges,
                )),
                ProjectionItem::GraphScope(value) => Some((
                    graph.scopes.contains(value),
                    graph.scopes.len(),
                    self.limits
                        .max_graph_edges
                        .saturating_add(self.limits.max_unresolved_entries),
                    Resource::GraphEdges,
                )),
                ProjectionItem::GraphProvenance(value) => Some((
                    graph.provenances.contains(value),
                    graph.provenances.len(),
                    self.limits
                        .max_graph_edges
                        .saturating_add(self.limits.max_unresolved_entries),
                    Resource::GraphEdges,
                )),
                _ => None,
            };
            if let Some((duplicate, current, limit, resource)) = vocabulary {
                if duplicate {
                    return Err(ReaderError::Invalid);
                }
                let next = u64::try_from(current)
                    .map_err(|_| reader_exhausted(resource))?
                    .checked_add(1)
                    .ok_or_else(|| reader_exhausted(resource))?;
                return (next <= limit)
                    .then_some(())
                    .ok_or_else(|| reader_exhausted(resource));
            }
        }
        let (current, limit, resource) = match (self.builder.as_ref(), item) {
            (Some(ProjectionBuilder::Comparison(value)), ProjectionItem::Correspondence(_)) => {
                let next = u64::try_from(value.correspondences.len())
                    .map_err(|_| reader_exhausted(Resource::Correspondences))?
                    .checked_add(1)
                    .ok_or_else(|| reader_exhausted(Resource::Correspondences))?;
                for (limit, resource) in [
                    (self.limits.max_correspondences, Resource::Correspondences),
                    (self.limits.max_assessments, Resource::Assessments),
                    (self.limits.max_changed_roots, Resource::ChangedRoots),
                ] {
                    if next > limit {
                        return Err(reader_exhausted(resource));
                    }
                }
                (
                    value.correspondences.len(),
                    self.limits.max_correspondences,
                    Resource::Correspondences,
                )
            }
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphNode(_)) => (
                value.nodes.len(),
                self.limits.max_graph_nodes,
                Resource::GraphNodes,
            ),
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::GraphEdge(_)) => (
                value.edges.len(),
                self.limits.max_graph_edges,
                Resource::GraphEdges,
            ),
            (Some(ProjectionBuilder::Graph(value)), ProjectionItem::Unresolved(item)) => {
                check_reader_len(
                    item.reason.len(),
                    self.limits.max_reason_bytes,
                    Resource::ReasonBytes,
                )?;
                let next_reason = checked_reader_add(
                    self.total_reason_bytes,
                    item.reason.len(),
                    Resource::TotalReasonBytes,
                )?;
                if next_reason > self.reason_budget {
                    return Err(reader_exhausted(Resource::TotalReasonBytes));
                }
                (
                    value.unresolved.len(),
                    self.limits.max_unresolved_entries,
                    Resource::UnresolvedEntries,
                )
            }
            (Some(ProjectionBuilder::Corpus(value)), ProjectionItem::VectorDefinition(_)) => (
                value.vectors.len(),
                self.limits.max_vector_definitions,
                Resource::VectorDefinitions,
            ),
            (Some(ProjectionBuilder::Evidence(value)), ProjectionItem::EvidenceRecord(item)) => {
                let rights = checked_reader_add(
                    self.evidence_rights,
                    item.rights.len(),
                    Resource::EvidenceRights,
                )?;
                let limitations = checked_reader_add(
                    self.evidence_limitations,
                    item.limitations.len(),
                    Resource::EvidenceLimitations,
                )?;
                if rights > self.limits.max_evidence_rights {
                    return Err(reader_exhausted(Resource::EvidenceRights));
                }
                if limitations > self.limits.max_evidence_limitations {
                    return Err(reader_exhausted(Resource::EvidenceLimitations));
                }
                (
                    value.records.len(),
                    self.limits.max_evidence_records,
                    Resource::EvidenceRecords,
                )
            }
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::SeedRule(_)) => (
                value.seed_rules.len(),
                self.limits.max_policy_seed_rules,
                Resource::PolicySeedRules,
            ),
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::PropagationRule(_)) => (
                value.propagation_rules.len(),
                self.limits.max_policy_propagation_rules,
                Resource::PolicyPropagationRules,
            ),
            (Some(ProjectionBuilder::Policy(value)), ProjectionItem::VectorRule(_)) => (
                value.vector_rules.len(),
                self.limits.max_policy_vector_rules,
                Resource::PolicyVectorRules,
            ),
            (Some(ProjectionBuilder::Manifest(value)), ProjectionItem::ManifestEntry(item)) => {
                if let ManifestStatus::Unknown(reason) = &item.status {
                    check_reader_len(
                        reason.len(),
                        self.limits.max_reason_bytes,
                        Resource::ReasonBytes,
                    )?;
                    let next_reason = checked_reader_add(
                        self.total_reason_bytes,
                        reason.len(),
                        Resource::TotalReasonBytes,
                    )?;
                    if next_reason > self.reason_budget {
                        return Err(reader_exhausted(Resource::TotalReasonBytes));
                    }
                }
                (
                    value.entries.len(),
                    self.limits.max_manifest_entries,
                    Resource::ManifestEntries,
                )
            }
            (Some(ProjectionBuilder::Manifest(value)), ProjectionItem::ManifestProvider(_)) => (
                value.providers.len(),
                self.limits.max_manifest_providers,
                Resource::ManifestProviders,
            ),
            _ => return Err(ReaderError::Invalid),
        };
        let next = u64::try_from(current)
            .map_err(|_| reader_exhausted(resource))?
            .checked_add(1)
            .ok_or_else(|| reader_exhausted(resource))?;
        if next > limit {
            return Err(reader_exhausted(resource));
        }
        Ok(())
    }

    fn finish(self) -> std::result::Result<Projection, ReaderError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let projection = match self.builder.ok_or(ReaderError::Invalid)? {
            ProjectionBuilder::Comparison(value) => Ok(Projection::Comparison(value)),
            ProjectionBuilder::Graph(value) => Ok(Projection::Graph(value)),
            ProjectionBuilder::Corpus(value) => Ok(Projection::Corpus(value)),
            ProjectionBuilder::Evidence(value) => Ok(Projection::Evidence(value)),
            ProjectionBuilder::Policy(value) => Ok(Projection::Policy(value)),
            ProjectionBuilder::Manifest(value) => Ok(Projection::Manifest(value)),
        }?;
        validate_projection_bounds(&projection, self.limits)?;
        Ok(projection)
    }

    fn failure(&self) -> Option<ReaderError> {
        self.failure
    }

    fn reason_bytes(&self) -> u64 {
        self.total_reason_bytes
    }
}

/// Pure selected strict-reader seam for Task 006 inputs.
pub trait StrictImpactReader {
    /// Exact capability selection implemented by this reader.
    fn selection(&self) -> &CapabilitySelection;

    /// Strict-reads one canonical document into the bounded coordinator-owned sink.
    fn read(
        &mut self,
        admission: &AdmittedCapability<'_>,
        document: &OwnerDocument,
        budget: ReaderBudget,
        sink: &mut ProjectionSink,
    ) -> std::result::Result<(), ReaderError>;
}

/// Monotonic operational clock injectable for deterministic deadline tests.
pub trait MonotonicClock: Send + Sync {
    /// Elapsed monotonic duration from an arbitrary stable origin.
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
            return Err(Error::new(
                ErrorCode::DurationExpired,
                "zero operational duration",
            ));
        }
        Ok(Self {
            clock,
            started: clock.now(),
            limit: Duration::from_millis(limit_ms),
        })
    }

    fn remaining_ms(self) -> Result<u64> {
        let elapsed = self.clock.now().checked_sub(self.started).ok_or_else(|| {
            Error::new(
                ErrorCode::DurationExpired,
                "monotonic clock moved backwards",
            )
        })?;
        let remaining = self.limit.checked_sub(elapsed).ok_or_else(|| {
            Error::new(ErrorCode::DurationExpired, "operational duration expired")
        })?;
        let millis = u64::try_from(remaining.as_millis())
            .map_err(|_| Error::new(ErrorCode::IntegerOverflow, "duration overflow"))?;
        if millis == 0 {
            return Err(Error::new(
                ErrorCode::DurationExpired,
                "operational duration expired",
            ));
        }
        Ok(millis)
    }

    fn check(self) -> Result<()> {
        self.remaining_ms().map(|_| ())
    }
}

fn count(value: usize) -> Result<u64> {
    u64::try_from(value)
        .map_err(|_| Error::new(ErrorCode::IntegerOverflow, "collection length overflow"))
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "resource usage overflow"))
}

fn reader_exhausted(resource: Resource) -> ReaderError {
    ReaderError::Exhausted(resource)
}

fn check_reader_len(
    value: usize,
    limit: u64,
    resource: Resource,
) -> std::result::Result<(), ReaderError> {
    let value = u64::try_from(value).map_err(|_| reader_exhausted(resource))?;
    if value > limit {
        return Err(reader_exhausted(resource));
    }
    Ok(())
}

fn checked_reader_add(
    total: u64,
    length: usize,
    resource: Resource,
) -> std::result::Result<u64, ReaderError> {
    let length = u64::try_from(length).map_err(|_| reader_exhausted(resource))?;
    total
        .checked_add(length)
        .ok_or_else(|| reader_exhausted(resource))
}

fn reader_validate_id(value: &str, limit: u64) -> std::result::Result<(), ReaderError> {
    if value.trim().is_empty() {
        return Err(ReaderError::Invalid);
    }
    check_reader_len(value.len(), limit, Resource::IdentityBytes)
}

fn reader_validate_subject(
    value: &QualifiedSubjectId,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    reader_validate_id(value.authority.as_str(), limits.max_identity_bytes)?;
    reader_validate_id(value.revision.as_str(), limits.max_identity_bytes)?;
    reader_validate_id(value.local.as_str(), limits.max_identity_bytes)
}

fn reader_validate_selection(
    value: &CapabilitySelection,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    for identity in [
        value.role().as_str(),
        value.provider().as_str(),
        value.build().as_str(),
        value.contract().as_str(),
        value.options().as_str(),
    ] {
        reader_validate_id(identity, limits.max_identity_bytes)?;
    }
    Ok(())
}

fn validate_projection_header(
    header: &ProjectionHeader,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    match header {
        ProjectionHeader::Comparison {
            comparison_rule,
            rule_revision,
            change_set,
            ..
        } => {
            reader_validate_id(comparison_rule.as_str(), limits.max_identity_bytes)?;
            reader_validate_id(rule_revision.as_str(), limits.max_identity_bytes)?;
            reader_validate_id(change_set.as_str(), limits.max_identity_bytes)
        }
        ProjectionHeader::Policy {
            identity, revision, ..
        } => {
            reader_validate_id(identity.as_str(), limits.max_identity_bytes)?;
            reader_validate_id(revision.as_str(), limits.max_identity_bytes)
        }
        ProjectionHeader::Graph
        | ProjectionHeader::Corpus
        | ProjectionHeader::Evidence
        | ProjectionHeader::Manifest => Ok(()),
    }
}

fn validate_projection_item(
    item: &ProjectionItem,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    let validate = |value: &str| reader_validate_id(value, limits.max_identity_bytes);
    match item {
        ProjectionItem::Correspondence(value) => {
            validate(value.id.as_str())?;
            if let Some(subject) = &value.baseline {
                reader_validate_subject(subject, limits)?;
            }
            if let Some(subject) = &value.candidate {
                reader_validate_subject(subject, limits)?;
            }
            if let Some(key) = &value.comparable_key {
                reader_validate_subject(&key.subject, limits)?;
                for identity in [
                    key.vector.as_str(),
                    key.method.as_str(),
                    key.provider.as_str(),
                    key.environment.as_str(),
                    key.assumptions.as_str(),
                    key.domain.as_str(),
                ] {
                    validate(identity)?;
                }
            }
            Ok(())
        }
        ProjectionItem::GraphAuthority(value) => validate(value.as_str()),
        ProjectionItem::GraphRelation(value) => validate(value.as_str()),
        ProjectionItem::GraphScope(value) => validate(value.as_str()),
        ProjectionItem::GraphProvenance(value) => validate(value.as_str()),
        ProjectionItem::GraphNode(value) => {
            reader_validate_subject(&value.id, limits)?;
            if let Some(evidence) = &value.evidence {
                validate(evidence.as_str())?;
            }
            Ok(())
        }
        ProjectionItem::GraphEdge(value) => {
            reader_validate_subject(&value.source, limits)?;
            reader_validate_subject(&value.target, limits)?;
            validate(value.relation.as_str())?;
            validate(value.scope.as_str())?;
            validate(value.provenance.as_str())
        }
        ProjectionItem::Unresolved(value) => {
            reader_validate_subject(&value.root, limits)?;
            reader_validate_subject(&value.predecessor, limits)?;
            reader_validate_subject(&value.target, limits)?;
            validate(value.relation.as_str())?;
            validate(value.scope.as_str())?;
            validate(value.provenance.as_str())
        }
        ProjectionItem::VectorDefinition(value) => validate(value.id.as_str()),
        ProjectionItem::EvidenceRecord(value) => {
            validate(value.id.as_str())?;
            reader_validate_subject(&value.subject, limits)?;
            validate(value.scope.as_str())?;
            validate(value.provenance.as_str())?;
            for identity in value.rights.iter().chain(&value.limitations) {
                validate(identity)?;
            }
            Ok(())
        }
        ProjectionItem::SeedRule(value) => validate(value.id.as_str()),
        ProjectionItem::PropagationRule(value) => {
            validate(value.id.as_str())?;
            validate(value.relation.as_str())?;
            validate(value.scope.as_str())
        }
        ProjectionItem::VectorRule(value) => {
            validate(value.id.as_str())?;
            validate_vector_key_reader(&value.key, limits)
        }
        ProjectionItem::ManifestEntry(value) => {
            reader_validate_selection(&value.provider, limits)?;
            validate(value.vector.as_str())
        }
        ProjectionItem::ManifestProvider(value) => reader_validate_selection(value, limits),
    }
}

fn validate_vector_key_reader(
    key: &VectorKey,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    reader_validate_subject(&key.subject, limits)?;
    for identity in [
        key.vector.as_str(),
        key.scope.as_str(),
        key.method.as_str(),
        key.environment.as_str(),
        key.provider_role.as_str(),
        key.profile.as_str(),
    ] {
        reader_validate_id(identity, limits.max_identity_bytes)?;
    }
    Ok(())
}

fn reader_encoded_len(
    encode: impl FnOnce(&mut CanonicalWriter) -> Result<()>,
) -> std::result::Result<u64, ReaderError> {
    let mut writer = CanonicalWriter::counting();
    encode(&mut writer).map_err(|_| ReaderError::Invalid)?;
    Ok(writer.written_len())
}

fn encoded_projection_header_len(
    header: &ProjectionHeader,
) -> std::result::Result<u64, ReaderError> {
    reader_encoded_len(|writer| match header {
        ProjectionHeader::Comparison {
            baseline_lock,
            candidate_lock,
            comparison_rule,
            rule_revision,
            rule_digest,
            change_set,
            change_set_digest,
        } => {
            writer.u8(InputContract::Comparison.ordinal())?;
            writer.string(&baseline_lock.to_hex())?;
            writer.string(&candidate_lock.to_hex())?;
            writer.string(comparison_rule.as_str())?;
            writer.string(rule_revision.as_str())?;
            writer.digest(rule_digest)?;
            writer.string(change_set.as_str())?;
            writer.digest(change_set_digest)
        }
        ProjectionHeader::Graph => writer.u8(InputContract::Graph.ordinal()),
        ProjectionHeader::Corpus => writer.u8(InputContract::Corpus.ordinal()),
        ProjectionHeader::Evidence => writer.u8(InputContract::Evidence.ordinal()),
        ProjectionHeader::Policy {
            identity,
            revision,
            digest,
        } => {
            writer.u8(InputContract::Policy.ordinal())?;
            writer.string(identity.as_str())?;
            writer.string(revision.as_str())?;
            writer.digest(digest)
        }
        ProjectionHeader::Manifest => writer.u8(InputContract::Manifest.ordinal()),
    })
}

fn encoded_projection_item_len(item: &ProjectionItem) -> std::result::Result<u64, ReaderError> {
    reader_encoded_len(|writer| match item {
        ProjectionItem::Correspondence(value) => encode_correspondence(writer, value),
        ProjectionItem::GraphAuthority(value) => writer.string(value.as_str()),
        ProjectionItem::GraphRelation(value) => writer.string(value.as_str()),
        ProjectionItem::GraphScope(value) => writer.string(value.as_str()),
        ProjectionItem::GraphProvenance(value) => writer.string(value.as_str()),
        ProjectionItem::GraphNode(value) => {
            encode_subject(writer, &value.id)?;
            encode_optional_string(writer, value.evidence.as_ref().map(EvidenceId::as_str))
        }
        ProjectionItem::GraphEdge(value) => {
            encode_subject(writer, &value.source)?;
            writer.string(value.relation.as_str())?;
            encode_subject(writer, &value.target)?;
            writer.string(value.scope.as_str())?;
            writer.string(value.provenance.as_str())
        }
        ProjectionItem::Unresolved(value) => encode_unresolved(writer, value),
        ProjectionItem::VectorDefinition(value) => {
            writer.string(value.id.as_str())?;
            writer.bytes(&value.owner_bytes)?;
            writer.digest(&value.digest)
        }
        ProjectionItem::EvidenceRecord(value) => {
            writer.string(value.id.as_str())?;
            encode_subject(writer, &value.subject)?;
            writer.u8(evidence_relation_code(value.relation))?;
            writer.bytes(&value.owner_bytes)?;
            writer.string(value.scope.as_str())?;
            writer.string(value.provenance.as_str())?;
            writer.len(value.rights.len())?;
            for right in &value.rights {
                writer.string(right)?;
            }
            writer.len(value.limitations.len())?;
            for limitation in &value.limitations {
                writer.string(limitation)?;
            }
            writer.u8(freshness_code(value.freshness))
        }
        ProjectionItem::SeedRule(value) => encode_seed_rule(writer, value),
        ProjectionItem::PropagationRule(value) => encode_propagation_rule(writer, value),
        ProjectionItem::VectorRule(value) => encode_vector_rule(writer, value),
        ProjectionItem::ManifestEntry(value) => {
            encode_selection(writer, &value.provider)?;
            writer.string(value.vector.as_str())?;
            match &value.status {
                ManifestStatus::Supported => writer.u8(0),
                ManifestStatus::Excluded => writer.u8(1),
                ManifestStatus::Unknown(reason) => {
                    writer.u8(2)?;
                    writer.string(reason)
                }
            }
        }
        ProjectionItem::ManifestProvider(value) => encode_selection(writer, value),
    })
}

fn validate_projection_bounds(
    projection: &Projection,
    limits: Limits,
) -> std::result::Result<(), ReaderError> {
    match projection {
        Projection::Comparison(value) => {
            check_reader_len(
                value.correspondences.len(),
                limits.max_correspondences,
                Resource::Correspondences,
            )?;
            check_reader_len(
                value.correspondences.len(),
                limits.max_assessments,
                Resource::Assessments,
            )?;
            check_reader_len(
                value.correspondences.len(),
                limits.max_changed_roots,
                Resource::ChangedRoots,
            )?;
        }
        Projection::Graph(value) => {
            check_reader_len(
                value.nodes.len(),
                limits.max_graph_nodes,
                Resource::GraphNodes,
            )?;
            check_reader_len(
                value.edges.len(),
                limits.max_graph_edges,
                Resource::GraphEdges,
            )?;
            check_reader_len(
                value.unresolved.len(),
                limits.max_unresolved_entries,
                Resource::UnresolvedEntries,
            )?;
            let reason_bytes = value.unresolved.iter().try_fold(0_u64, |total, entry| {
                let length = u64::try_from(entry.reason.len())
                    .map_err(|_| reader_exhausted(Resource::ReasonBytes))?;
                if length > limits.max_reason_bytes {
                    return Err(reader_exhausted(Resource::ReasonBytes));
                }
                total
                    .checked_add(length)
                    .ok_or_else(|| reader_exhausted(Resource::TotalReasonBytes))
            })?;
            if reason_bytes > limits.max_total_reason_bytes {
                return Err(reader_exhausted(Resource::TotalReasonBytes));
            }
        }
        Projection::Corpus(value) => check_reader_len(
            value.vectors.len(),
            limits.max_vector_definitions,
            Resource::VectorDefinitions,
        )?,
        Projection::Evidence(value) => {
            check_reader_len(
                value.records.len(),
                limits.max_evidence_records,
                Resource::EvidenceRecords,
            )?;
            let rights = value.records.iter().try_fold(0_u64, |total, record| {
                let current = u64::try_from(record.rights.len())
                    .map_err(|_| reader_exhausted(Resource::EvidenceRights))?;
                total
                    .checked_add(current)
                    .ok_or_else(|| reader_exhausted(Resource::EvidenceRights))
            })?;
            if rights > limits.max_evidence_rights {
                return Err(reader_exhausted(Resource::EvidenceRights));
            }
            let limitations = value.records.iter().try_fold(0_u64, |total, record| {
                let current = u64::try_from(record.limitations.len())
                    .map_err(|_| reader_exhausted(Resource::EvidenceLimitations))?;
                total
                    .checked_add(current)
                    .ok_or_else(|| reader_exhausted(Resource::EvidenceLimitations))
            })?;
            if limitations > limits.max_evidence_limitations {
                return Err(reader_exhausted(Resource::EvidenceLimitations));
            }
        }
        Projection::Policy(value) => {
            check_reader_len(
                value.seed_rules.len(),
                limits.max_policy_seed_rules,
                Resource::PolicySeedRules,
            )?;
            check_reader_len(
                value.propagation_rules.len(),
                limits.max_policy_propagation_rules,
                Resource::PolicyPropagationRules,
            )?;
            check_reader_len(
                value.vector_rules.len(),
                limits.max_policy_vector_rules,
                Resource::PolicyVectorRules,
            )?;
        }
        Projection::Manifest(value) => {
            check_reader_len(
                value.providers.len(),
                limits.max_manifest_providers,
                Resource::ManifestProviders,
            )?;
            check_reader_len(
                value.entries.len(),
                limits.max_manifest_entries,
                Resource::ManifestEntries,
            )?;
            let reason_bytes = value.entries.iter().try_fold(0_u64, |total, entry| {
                let ManifestStatus::Unknown(reason) = &entry.status else {
                    return Ok(total);
                };
                let length = u64::try_from(reason.len())
                    .map_err(|_| reader_exhausted(Resource::ReasonBytes))?;
                if length > limits.max_reason_bytes {
                    return Err(reader_exhausted(Resource::ReasonBytes));
                }
                total
                    .checked_add(length)
                    .ok_or_else(|| reader_exhausted(Resource::TotalReasonBytes))
            })?;
            if reason_bytes > limits.max_total_reason_bytes {
                return Err(reader_exhausted(Resource::TotalReasonBytes));
            }
        }
    }
    let retained_bytes = projection_retained_bytes(projection)?;
    if retained_bytes > limits.max_document_bytes {
        return Err(reader_exhausted(Resource::DocumentBytes));
    }
    Ok(())
}

fn projection_retained_bytes(projection: &Projection) -> std::result::Result<u64, ReaderError> {
    fn add(total: u64, length: usize) -> std::result::Result<u64, ReaderError> {
        let length =
            u64::try_from(length).map_err(|_| reader_exhausted(Resource::DocumentBytes))?;
        total
            .checked_add(length)
            .ok_or_else(|| reader_exhausted(Resource::DocumentBytes))
    }

    match projection {
        Projection::Comparison(value) => value
            .correspondences
            .iter()
            .try_fold(0, |total, item| add(total, item.id.as_str().len())),
        Projection::Graph(value) => {
            let total = value
                .nodes
                .iter()
                .try_fold(0, |total, node| add(total, node.id.local.as_str().len()))?;
            let total = value.edges.iter().try_fold(total, |total, edge| {
                add(total, edge.provenance.as_str().len())
            })?;
            value
                .unresolved
                .iter()
                .try_fold(total, |total, item| add(total, item.reason.len()))
        }
        Projection::Corpus(value) => value
            .vectors
            .iter()
            .try_fold(0, |total, item| add(total, item.owner_bytes.len())),
        Projection::Evidence(value) => value.records.iter().try_fold(0, |total, item| {
            let total = add(total, item.owner_bytes.len())?;
            let total = item
                .rights
                .iter()
                .try_fold(total, |total, right| add(total, right.len()))?;
            item.limitations
                .iter()
                .try_fold(total, |total, limitation| add(total, limitation.len()))
        }),
        Projection::Policy(value) => {
            let total = value
                .seed_rules
                .iter()
                .try_fold(0, |total, item| add(total, item.id.as_str().len()))?;
            let total = value
                .propagation_rules
                .iter()
                .try_fold(total, |total, item| add(total, item.id.as_str().len()))?;
            value
                .vector_rules
                .iter()
                .try_fold(total, |total, item| add(total, item.id.as_str().len()))
        }
        Projection::Manifest(value) => value.entries.iter().try_fold(0, |total, item| {
            let total = add(total, item.vector.as_str().len())?;
            match &item.status {
                ManifestStatus::Unknown(reason) => add(total, reason.len()),
                ManifestStatus::Supported | ManifestStatus::Excluded => Ok(total),
            }
        }),
    }
}

/// Closed potential-impact reachability overlay.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Reachability {
    /// An affectedness path reached the subject.
    Affected,
    /// Complete traversal did not reach the subject.
    Unaffected,
    /// Incomplete or unresolved traversal cannot establish reachability.
    Unknown,
}

/// One exact effect-oriented edge retained in a witness or frontier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OrientedEdge {
    /// Independent effect carried by this edge.
    pub effect: Effect,
    /// Oriented source identity.
    pub source: QualifiedSubjectId,
    /// Exact owner relation.
    pub relation: RelationId,
    /// Oriented target identity.
    pub target: QualifiedSubjectId,
    /// Exact owner scope.
    pub scope: ScopeId,
    /// Exact owner provenance.
    pub provenance: ProvenanceId,
    /// Feature-local orientation applied to the owner edge.
    pub direction: Direction,
}

/// Canonical shortest witness from one exact changed root.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Witness {
    /// Exact changed root.
    pub root: QualifiedSubjectId,
    /// Independent propagated effect.
    pub effect: Effect,
    /// Exact reached subject.
    pub subject: QualifiedSubjectId,
    /// Canonical shortest full-edge path.
    pub path: Vec<OrientedEdge>,
}

/// One per-root subject overlay with independent effect witnesses.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SubjectRecord {
    /// Exact changed root.
    pub root: QualifiedSubjectId,
    /// Exact graph subject.
    pub subject: QualifiedSubjectId,
    /// Potential-impact reachability.
    pub reachability: Reachability,
    /// Canonical affectedness witness, when reached.
    pub affectedness_witness: Option<Witness>,
    /// Canonical staleness witness, when reached.
    pub staleness_witness: Option<Witness>,
}

/// Freshness propagation state used by the complete transition table.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FreshnessPropagation {
    /// A staleness path reached the evidence subject.
    Reached,
    /// Complete traversal did not reach the evidence subject.
    NotReached,
    /// Unresolved topology prevents a conclusion.
    Unknown,
    /// Deterministic traversal stopped before a conclusion.
    Incomplete,
}

/// One non-authoritative freshness overlay preserving the owner relation and bytes.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FreshnessRecord {
    /// Exact changed root.
    pub root: QualifiedSubjectId,
    /// Exact FR-337 evidence identity.
    pub evidence: EvidenceId,
    /// Exact evidence subject.
    pub subject: QualifiedSubjectId,
    /// Unchanged owner relation.
    pub relation: EvidenceRelation,
    /// Unchanged canonical owner bytes.
    pub owner_bytes: Vec<u8>,
    /// Exact owner scope.
    pub scope: ScopeId,
    /// Exact owner provenance.
    pub provenance: ProvenanceId,
    /// Prior freshness.
    pub prior: Freshness,
    /// Staleness-path state for this run.
    pub propagation: FreshnessPropagation,
    /// Result of the complete freshness table.
    pub freshness: Freshness,
    /// Witness only when this run changed current evidence to stale.
    pub witness: Option<Witness>,
}

/// One canonical finite cycle record.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CycleRecord {
    /// Exact changed root that can reach this SCC.
    pub root: QualifiedSubjectId,
    /// Independent effect-specific topology.
    pub effect: Effect,
    /// Canonical ordered SCC membership.
    pub component: Vec<QualifiedSubjectId>,
    /// Canonical shortest simple directed cycle.
    pub cycle: Vec<OrientedEdge>,
    /// Canonical root-to-cycle entry point and depth.
    pub entry: FrontierEntry,
}

/// One exact immediate unexpanded traversal entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FrontierEntry {
    /// Exact changed root.
    pub root: QualifiedSubjectId,
    /// Independent effect.
    pub effect: Effect,
    /// Exact predecessor, absent only for a seed entry.
    pub predecessor: Option<QualifiedSubjectId>,
    /// Exact target.
    pub target: QualifiedSubjectId,
    /// Exact owner relation, absent only for a seed entry.
    pub relation: Option<RelationId>,
    /// Exact owner scope, absent only for a seed entry.
    pub scope: Option<ScopeId>,
    /// Exact owner provenance, absent only for a seed entry.
    pub provenance: Option<ProvenanceId>,
    /// Exact canonical depth.
    pub depth: u64,
}

/// Independent provider availability axis.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VectorAvailability {
    /// Exact selected provider supports the vector.
    Available(ProviderId),
    /// Exact selected provider explicitly excludes the vector.
    Unsupported(ProviderId),
    /// No admitted matching provider entry exists.
    Unavailable,
    /// Admitted provider reported an indeterminate bounded reason.
    Unknown {
        /// Exact selected provider.
        provider: ProviderId,
        /// Bounded provider reason.
        reason: String,
    },
}

/// One exact-once catalog vector record.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VectorRecord {
    /// Complete vector key.
    pub key: VectorKey,
    /// Unchanged FR-336 owner definition bytes.
    pub definition_bytes: Vec<u8>,
    /// Independent requirement state.
    pub requirement: VectorRequirement,
    /// Independent provider availability state.
    pub availability: VectorAvailability,
    /// Every canonical contributing policy-rule identity.
    pub contributing_rules: Vec<RuleId>,
}

/// Complete or truthfully incomplete deterministic traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraversalOutcome {
    /// Every deterministic traversal unit completed.
    Complete,
    /// Traversal stopped before exceeding one deterministic retention/work ceiling.
    Incomplete {
        /// Exact exhausted deterministic dimension.
        cause: Resource,
        /// Canonical immediate unexpanded traversal frontier.
        frontier: Vec<FrontierEntry>,
    },
}

impl TraversalOutcome {
    /// Exact exhausted dimension, when incomplete.
    #[must_use]
    pub const fn cause(&self) -> Option<Resource> {
        match self {
            Self::Complete => None,
            Self::Incomplete { cause, .. } => Some(*cause),
        }
    }

    /// Canonical immediate frontier, empty for complete traversal.
    #[must_use]
    pub fn frontier(&self) -> &[FrontierEntry] {
        match self {
            Self::Complete => &[],
            Self::Incomplete { frontier, .. } => frontier,
        }
    }
}

/// Exact deterministic resource usage retained in the envelope.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Bytes across all input documents.
    pub input_bytes: u64,
    /// Strict-reader calls.
    pub reader_invocations: u64,
    /// Atomic topology items.
    pub topology_items: u64,
    /// Traversal node expansions.
    pub traversal_steps: u64,
    /// Traversed effect-carrying edges.
    pub traversed_edges: u64,
    /// Retained subject overlays.
    pub subject_records: u64,
    /// Retained path edges.
    pub path_edges: u64,
    /// Retained witnesses.
    pub witnesses: u64,
    /// Encoded witness bytes.
    pub witness_bytes: u64,
    /// Retained frontier entries.
    pub frontier_entries: u64,
}

/// One exact canonical input retained without relabeling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedInput {
    /// Exact contract.
    pub contract: InputContract,
    /// Exact admitted lock side.
    pub lock: LockSide,
    /// Exact selected reader capability.
    pub selection: CapabilitySelection,
    /// Unchanged canonical bytes.
    pub canonical_bytes: Vec<u8>,
    /// Raw byte digest.
    pub digest: [u8; 32],
}

/// Canonical FR-005 result envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImpactEnvelope {
    baseline_lock: LockIdentity,
    candidate_lock: LockIdentity,
    limits: Limits,
    inputs: Vec<RetainedInput>,
    assessments: Vec<Correspondence>,
    subjects: Vec<SubjectRecord>,
    freshness: Vec<FreshnessRecord>,
    cycles: Vec<CycleRecord>,
    unresolved: Vec<UnresolvedDependency>,
    vectors: Vec<VectorRecord>,
    outcome: TraversalOutcome,
    usage: Usage,
    projection_fingerprint: [u8; 32],
    body_bytes: Vec<u8>,
    canonical_bytes: Vec<u8>,
    identity: [u8; 32],
}

impl ImpactEnvelope {
    /// Exact directional baseline lock identity.
    #[must_use]
    pub const fn baseline_lock(&self) -> LockIdentity {
        self.baseline_lock
    }

    /// Exact directional candidate lock identity.
    #[must_use]
    pub const fn candidate_lock(&self) -> LockIdentity {
        self.candidate_lock
    }

    /// Exact deterministic limits bound into the body.
    #[must_use]
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// Exact retained canonical owner/facade inputs.
    #[must_use]
    pub fn inputs(&self) -> &[RetainedInput] {
        &self.inputs
    }

    /// Independent owner assessment tuples.
    #[must_use]
    pub fn assessments(&self) -> &[Correspondence] {
        &self.assessments
    }

    /// Per-root subject overlays.
    #[must_use]
    pub fn subjects(&self) -> &[SubjectRecord] {
        &self.subjects
    }

    /// Per-root FR-337 freshness overlays.
    #[must_use]
    pub fn freshness(&self) -> &[FreshnessRecord] {
        &self.freshness
    }

    /// Canonical per-root/effect/SCC cycle records.
    #[must_use]
    pub fn cycles(&self) -> &[CycleRecord] {
        &self.cycles
    }

    /// Valid unresolved dependency overlays.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedDependency] {
        &self.unresolved
    }

    /// Exact-once catalog vector records.
    #[must_use]
    pub fn vectors(&self) -> &[VectorRecord] {
        &self.vectors
    }

    /// Complete or truthfully incomplete traversal outcome.
    #[must_use]
    pub const fn outcome(&self) -> &TraversalOutcome {
        &self.outcome
    }

    /// Exact deterministic resource usage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Canonical body bytes excluding the identity field.
    #[must_use]
    pub fn body_bytes(&self) -> &[u8] {
        &self.body_bytes
    }

    /// Canonical wire bytes carrying the recomputable identity.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Domain-separated digest of `body_bytes`.
    #[must_use]
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }
}

/// Pure FR-005 coordinator.
pub struct ImpactCoordinator {
    limits: Limits,
    clock: Arc<dyn MonotonicClock>,
}

impl ImpactCoordinator {
    /// Constructs a coordinator using the system monotonic clock.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            clock: Arc::new(SystemClock::new()),
        }
    }

    /// Constructs a coordinator with an injected monotonic clock.
    #[must_use]
    pub fn with_monotonic_clock(limits: Limits, clock: Arc<dyn MonotonicClock>) -> Self {
        Self { limits, clock }
    }

    /// Strict-reads, validates and classifies one directional impact request.
    pub fn classify(
        &self,
        baseline: &ValidatedLock,
        candidate: &ValidatedLock,
        inputs: &[ReaderInput],
        readers: &mut [&mut dyn StrictImpactReader],
    ) -> Result<ImpactEnvelope> {
        self.limits.validate()?;
        let timer = Timer::start(self.clock.as_ref(), self.limits.max_duration_ms)?;
        timer.check()?;
        let ordered_inputs = validate_reader_inputs(inputs, self.limits)?;
        if readers.len() != ordered_inputs.len() {
            return Err(Error::new(
                ErrorCode::Malformed,
                "reader and input inventories differ",
            ));
        }
        if count(ordered_inputs.len())? > self.limits.max_reader_invocations {
            return Err(Error::exhausted(
                Resource::ReaderInvocations,
                "strict-reader invocation ceiling exceeded",
            ));
        }
        let mut selected_readers = BTreeSet::new();
        for input in &ordered_inputs {
            timer.check()?;
            if !selected_readers.insert(input.selection.clone()) {
                return Err(Error::new(
                    ErrorCode::CapabilityMismatch,
                    "one strict reader was selected more than once",
                ));
            }
            if !readers
                .iter()
                .any(|reader| reader.selection() == &input.selection)
            {
                return Err(Error::new(
                    ErrorCode::CapabilityMismatch,
                    "requested strict reader is absent",
                ));
            }
            let lock = match input.lock {
                LockSide::Baseline => baseline,
                LockSide::Candidate => candidate,
            };
            let admission = lock.admit_capability(&input.selection).map_err(|_| {
                Error::new(
                    ErrorCode::CapabilityMismatch,
                    "strict reader is not admitted by selected lock",
                )
            })?;
            if admission.contract() != input.contract.label() {
                return Err(Error::new(
                    ErrorCode::CapabilityMismatch,
                    "admitted reader contract differs from requested input",
                ));
            }
        }
        let mut projections = BTreeMap::new();
        let mut retained_inputs = Vec::with_capacity(ordered_inputs.len());
        let mut usage = Usage::default();
        let mut retained_reason_bytes = 0_u64;
        for input in &ordered_inputs {
            timer.check()?;
            let reader = readers
                .iter_mut()
                .find(|reader| reader.selection() == &input.selection)
                .ok_or_else(|| {
                    Error::new(
                        ErrorCode::CapabilityMismatch,
                        "requested strict reader is absent",
                    )
                })?;
            let lock = match input.lock {
                LockSide::Baseline => baseline,
                LockSide::Candidate => candidate,
            };
            let admission = lock.admit_capability(&input.selection).map_err(|_| {
                Error::new(
                    ErrorCode::CapabilityMismatch,
                    "strict reader is not admitted by selected lock",
                )
            })?;
            if admission.contract() != input.contract.label() {
                return Err(Error::new(
                    ErrorCode::CapabilityMismatch,
                    "admitted reader contract differs from requested input",
                ));
            }
            let reason_budget = self
                .limits
                .max_total_reason_bytes
                .checked_sub(retained_reason_bytes)
                .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "reason budget underflow"))?;
            let mut sink = ProjectionSink::new(input.contract, self.limits, reason_budget);
            let reader_result = reader.read(
                &admission,
                &input.document,
                ReaderBudget {
                    limits: self.limits,
                    remaining_duration_ms: timer.remaining_ms()?,
                },
                &mut sink,
            );
            timer.check()?;
            if let Some(error) = sink.failure() {
                return Err(map_reader_error(error));
            }
            reader_result.map_err(map_reader_error)?;
            retained_reason_bytes = checked_add(retained_reason_bytes, sink.reason_bytes())?;
            let projection = match sink.finish() {
                Ok(projection) => projection,
                Err(error) => return Err(map_reader_error(error)),
            };
            if projections.insert(input.contract, projection).is_some() {
                return Err(Error::new(
                    ErrorCode::Malformed,
                    "duplicate strict-reader projection",
                ));
            }
            usage.reader_invocations = checked_add(usage.reader_invocations, 1)?;
            usage.input_bytes = checked_add(
                usage.input_bytes,
                count(input.document.canonical_bytes.len())?,
            )?;
            retained_inputs.push(RetainedInput {
                contract: input.contract,
                lock: input.lock,
                selection: input.selection.clone(),
                canonical_bytes: input.document.canonical_bytes.clone(),
                digest: input.document.digest,
            });
        }
        let comparison = take_comparison(&mut projections)?;
        let graph = take_graph(&mut projections)?;
        let corpus = take_corpus(&mut projections)?;
        let evidence = take_evidence(&mut projections)?;
        let policy = take_policy(&mut projections)?;
        let manifest = take_manifest(&mut projections)?;
        if !projections.is_empty() {
            return Err(Error::new(
                ErrorCode::Malformed,
                "unexpected strict-reader projection",
            ));
        }
        self.classify_bundle(
            baseline,
            candidate,
            ProjectionBundle {
                retained_inputs,
                comparison,
                graph,
                corpus,
                evidence,
                policy,
                manifest,
            },
            usage,
            timer,
        )
    }
}

fn map_reader_error(error: ReaderError) -> Error {
    match error {
        ReaderError::Invalid => {
            Error::new(ErrorCode::ReaderFailure, "strict reader rejected input")
        }
        ReaderError::Exhausted(Resource::Duration) => Error::new(
            ErrorCode::DurationExpired,
            "strict reader exhausted operational duration",
        ),
        ReaderError::Exhausted(resource) => {
            Error::exhausted(resource, "strict reader exhausted supplied bound")
        }
        ReaderError::Failed => Error::new(ErrorCode::ReaderFailure, "strict reader failed"),
    }
}

fn validate_reader_inputs(inputs: &[ReaderInput], limits: Limits) -> Result<Vec<ReaderInput>> {
    if inputs.len() != 6 {
        return Err(Error::new(
            ErrorCode::Malformed,
            "exactly six strict-reader inputs are required",
        ));
    }
    let mut ordered = inputs.to_vec();
    ordered.sort_by_key(|input| input.contract);
    for (expected, input) in [
        InputContract::Comparison,
        InputContract::Graph,
        InputContract::Corpus,
        InputContract::Evidence,
        InputContract::Policy,
        InputContract::Manifest,
    ]
    .into_iter()
    .zip(&ordered)
    {
        if input.contract != expected || input.document.contract != expected {
            return Err(Error::new(
                ErrorCode::Malformed,
                "missing, duplicate or mismatched strict-reader input",
            ));
        }
        if input.selection.contract().as_str() != expected.label() {
            return Err(Error::new(
                ErrorCode::CapabilityMismatch,
                "reader selection contract does not match document contract",
            ));
        }
        validate_capability_selection(&input.selection, limits)?;
        let length = count(input.document.canonical_bytes.len())?;
        if length > limits.max_document_bytes {
            return Err(Error::exhausted(
                Resource::DocumentBytes,
                "input document exceeds byte ceiling",
            ));
        }
        if Sha256::digest(&input.document.canonical_bytes).as_slice() != input.document.digest {
            return Err(Error::new(
                ErrorCode::CanonicalMismatch,
                "input document digest mismatch",
            ));
        }
    }
    let total = ordered.iter().try_fold(0_u64, |sum, input| {
        checked_add(sum, count(input.document.canonical_bytes.len())?)
    })?;
    if total > limits.max_total_input_bytes {
        return Err(Error::exhausted(
            Resource::TotalInputBytes,
            "aggregate input bytes exceed ceiling",
        ));
    }
    Ok(ordered)
}

macro_rules! take_projection {
    ($name:ident, $variant:ident, $kind:expr, $type:ty) => {
        fn $name(projections: &mut BTreeMap<InputContract, Projection>) -> Result<$type> {
            match projections.remove(&$kind) {
                Some(Projection::$variant(value)) => Ok(value),
                _ => Err(Error::new(
                    ErrorCode::Malformed,
                    "strict reader produced wrong projection kind",
                )),
            }
        }
    };
}

take_projection!(
    take_comparison,
    Comparison,
    InputContract::Comparison,
    ComparisonProjection
);
take_projection!(take_graph, Graph, InputContract::Graph, GraphProjection);
take_projection!(take_corpus, Corpus, InputContract::Corpus, CorpusProjection);
take_projection!(
    take_evidence,
    Evidence,
    InputContract::Evidence,
    EvidenceProjection
);
take_projection!(take_policy, Policy, InputContract::Policy, ImpactPolicy);
take_projection!(
    take_manifest,
    Manifest,
    InputContract::Manifest,
    ManifestProjection
);

#[derive(Clone)]
struct ProjectionBundle {
    retained_inputs: Vec<RetainedInput>,
    comparison: ComparisonProjection,
    graph: GraphProjection,
    corpus: CorpusProjection,
    evidence: EvidenceProjection,
    policy: ImpactPolicy,
    manifest: ManifestProjection,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ChangedRoot {
    subject: QualifiedSubjectId,
    assessment: AssessmentTuple,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WorkItem {
    node: QualifiedSubjectId,
    path: Vec<OrientedEdge>,
    incoming: Option<OrientedEdge>,
    depth: u64,
}

#[derive(Default)]
struct TraversalState {
    witnesses: BTreeMap<(QualifiedSubjectId, QualifiedSubjectId, Effect), Witness>,
    incomplete: Option<Resource>,
    incomplete_units: BTreeSet<(QualifiedSubjectId, Effect)>,
    frontier: Vec<FrontierEntry>,
    usage: Usage,
}

#[derive(Default)]
struct CycleAnalysis {
    records: Vec<CycleRecord>,
    circular_unknowns: BTreeSet<(QualifiedSubjectId, QualifiedSubjectId, Effect)>,
    usage: Usage,
    exhausted: Option<Resource>,
}

impl ImpactCoordinator {
    fn classify_bundle(
        &self,
        baseline: &ValidatedLock,
        candidate: &ValidatedLock,
        mut bundle: ProjectionBundle,
        mut usage: Usage,
        timer: Timer<'_>,
    ) -> Result<ImpactEnvelope> {
        timer.check()?;
        canonicalize_and_validate_bundle(baseline, candidate, &mut bundle, self.limits)?;
        let roots = changed_roots(&bundle.comparison)?;
        let oriented = oriented_graph(&bundle.graph, &bundle.policy, self.limits, timer)?;
        let oriented_count = oriented.values().try_fold(0_u64, |total, edges| {
            checked_add(total, count(edges.len())?)
        })?;
        usage.topology_items = checked_add(count(bundle.graph.nodes.len())?, oriented_count)?;
        if usage.topology_items > self.limits.max_topology_items {
            return Err(Error::exhausted(
                Resource::TopologyItems,
                "effect-oriented topology exceeds atomic ceiling",
            ));
        }
        validate_unresolved_grounding(
            &roots,
            &bundle.graph.unresolved,
            &bundle.policy,
            &oriented,
            timer,
        )?;
        timer.check()?;
        let projection_fingerprint = projection_fingerprint(&bundle)?;
        let subject_demand = count(subject_record_keys(&roots, &bundle.graph).len())?;
        let subject_exhausted = subject_demand > self.limits.max_subject_records;
        let cycle_analysis = if subject_exhausted {
            CycleAnalysis::default()
        } else {
            classify_cycles(
                &roots,
                &bundle.graph,
                &oriented,
                &bundle.policy,
                self.limits,
                timer,
            )?
        };
        let mut traversal = if subject_exhausted {
            incomplete_before_traversal(
                &roots,
                &bundle.policy,
                Resource::SubjectRecords,
                cycle_analysis.usage,
                self.limits,
            )?
        } else if let Some(cause) = cycle_analysis.exhausted {
            incomplete_before_traversal(
                &roots,
                &bundle.policy,
                cause,
                cycle_analysis.usage,
                self.limits,
            )?
        } else {
            traverse_effects(
                &roots,
                &bundle.graph,
                &bundle.policy,
                &oriented,
                cycle_analysis.usage,
                self.limits,
                timer,
            )?
        };
        traversal.usage.input_bytes = usage.input_bytes;
        traversal.usage.reader_invocations = usage.reader_invocations;
        traversal.usage.topology_items = usage.topology_items;
        let subjects = compose_subject_records(
            &roots,
            &bundle.graph,
            &bundle.policy,
            &cycle_analysis.circular_unknowns,
            &traversal,
            self.limits,
        )?;
        traversal.usage.subject_records = count(subjects.len())?;
        let freshness = compose_freshness(
            &roots,
            &bundle.graph,
            &bundle.policy,
            &cycle_analysis.circular_unknowns,
            &bundle.evidence,
            &traversal,
        )?;
        let vectors = compose_vectors(&bundle.corpus, &bundle.policy, &bundle.manifest, candidate)?;
        let outcome = if let Some(cause) = traversal.incomplete {
            TraversalOutcome::Incomplete {
                cause,
                frontier: traversal.frontier.clone(),
            }
        } else {
            TraversalOutcome::Complete
        };
        timer.check()?;
        let mut envelope = ImpactEnvelope {
            baseline_lock: baseline.identity(),
            candidate_lock: candidate.identity(),
            limits: self.limits,
            inputs: bundle.retained_inputs,
            assessments: bundle.comparison.correspondences,
            subjects,
            freshness,
            cycles: cycle_analysis.records,
            unresolved: bundle.graph.unresolved,
            vectors,
            outcome,
            usage: traversal.usage,
            projection_fingerprint,
            body_bytes: Vec::new(),
            canonical_bytes: Vec::new(),
            identity: [0; 32],
        };
        envelope.body_bytes = encode_body(&envelope, self.limits.max_output_bytes)?;
        let mut hasher = Sha256::new();
        hasher.update(b"quire.compatibility-impact/v1\0");
        hasher.update(&envelope.body_bytes);
        envelope.identity = hasher.finalize().into();
        envelope.canonical_bytes = encode_envelope(&envelope, self.limits.max_accounting_bytes)?;
        timer.check()?;
        Ok(envelope)
    }
}

fn canonicalize_and_validate_bundle(
    baseline: &ValidatedLock,
    candidate: &ValidatedLock,
    bundle: &mut ProjectionBundle,
    limits: Limits,
) -> Result<()> {
    bundle.retained_inputs.sort_by_key(|input| input.contract);
    bundle.comparison.correspondences.sort();
    bundle.graph.authorities.sort();
    bundle.graph.relations.sort();
    bundle.graph.scopes.sort();
    bundle.graph.provenances.sort();
    bundle.graph.nodes.sort();
    bundle.graph.edges.sort();
    bundle.graph.unresolved.sort();
    bundle.corpus.vectors.sort();
    bundle.evidence.records.sort();
    bundle.policy.seed_rules.sort();
    bundle.policy.propagation_rules.sort();
    bundle.policy.vector_rules.sort();
    bundle.manifest.providers.sort();
    bundle.manifest.entries.sort();

    if bundle.comparison.baseline_lock != baseline.identity()
        || bundle.comparison.candidate_lock != candidate.identity()
    {
        return Err(Error::new(
            ErrorCode::ForeignInput,
            "comparison projection lock identities differ from request",
        ));
    }
    if baseline.identity() == candidate.identity() && !bundle.comparison.correspondences.is_empty()
    {
        return Err(Error::new(
            ErrorCode::OwnerContradiction,
            "equal locks cannot declare a nonempty change set",
        ));
    }
    reject_adjacent_duplicates(
        &bundle.comparison.correspondences,
        |value| &value.id,
        "duplicate correspondence identity",
    )?;
    for correspondence in &bundle.comparison.correspondences {
        validate_id(correspondence.id.as_str(), limits)?;
        validate_correspondence(correspondence)?;
        validate_subject_option(correspondence.baseline.as_ref(), limits)?;
        validate_subject_option(correspondence.candidate.as_ref(), limits)?;
        if let Some(key) = &correspondence.comparable_key {
            validate_subject(&key.subject, limits)?;
            validate_id(key.vector.as_str(), limits)?;
            validate_id(key.method.as_str(), limits)?;
            validate_id(key.provider.as_str(), limits)?;
            validate_id(key.environment.as_str(), limits)?;
            validate_id(key.assumptions.as_str(), limits)?;
            validate_id(key.domain.as_str(), limits)?;
            if key.baseline_lock != bundle.comparison.baseline_lock
                || key.candidate_lock != bundle.comparison.candidate_lock
                || correspondence.candidate.as_ref() != Some(&key.subject)
            {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "comparable-result key is foreign to its correspondence",
                ));
            }
        }
    }
    validate_id(bundle.comparison.comparison_rule.as_str(), limits)?;
    validate_id(bundle.comparison.rule_revision.as_str(), limits)?;
    validate_id(bundle.comparison.change_set.as_str(), limits)?;

    reject_exact_duplicates(&bundle.graph.authorities, "duplicate graph authority")?;
    reject_exact_duplicates(&bundle.graph.relations, "duplicate graph relation")?;
    reject_exact_duplicates(&bundle.graph.scopes, "duplicate graph scope")?;
    reject_exact_duplicates(&bundle.graph.provenances, "duplicate graph provenance")?;
    for authority in &bundle.graph.authorities {
        validate_id(authority.as_str(), limits)?;
    }
    for relation in &bundle.graph.relations {
        validate_id(relation.as_str(), limits)?;
    }
    for scope in &bundle.graph.scopes {
        validate_id(scope.as_str(), limits)?;
    }
    for provenance in &bundle.graph.provenances {
        validate_id(provenance.as_str(), limits)?;
    }
    let authorities: BTreeSet<_> = bundle.graph.authorities.iter().collect();
    let relations: BTreeSet<_> = bundle.graph.relations.iter().collect();
    let scopes: BTreeSet<_> = bundle.graph.scopes.iter().collect();
    let provenances: BTreeSet<_> = bundle.graph.provenances.iter().collect();
    reject_adjacent_duplicates(
        &bundle.graph.nodes,
        |value| &value.id,
        "duplicate graph node identity",
    )?;
    reject_exact_duplicates(&bundle.graph.edges, "duplicate graph edge")?;
    let mut unresolved_keys = BTreeSet::new();
    for value in &bundle.graph.unresolved {
        let key = (
            value.root.clone(),
            value.predecessor.clone(),
            value.target.clone(),
            value.relation.clone(),
            value.scope.clone(),
            value.provenance.clone(),
        );
        if !unresolved_keys.insert(key) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "duplicate or rebound unresolved dependency",
            ));
        }
    }
    let node_ids: BTreeSet<_> = bundle
        .graph
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect();
    for node in &bundle.graph.nodes {
        validate_subject(&node.id, limits)?;
        if !authorities.contains(&node.id.authority) {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "graph node authority is outside the closed owner vocabulary",
            ));
        }
        match (node.id.kind, node.evidence.as_ref()) {
            (SubjectKind::Evidence, None) => {
                return Err(Error::new(
                    ErrorCode::Malformed,
                    "evidence graph node lacks FR-337 identity",
                ));
            }
            (SubjectKind::Evidence, Some(_)) | (_, None) => {}
            (_, Some(_)) => {
                return Err(Error::new(
                    ErrorCode::Malformed,
                    "non-evidence graph node carries an FR-337 identity",
                ));
            }
        }
    }
    for edge in &bundle.graph.edges {
        if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "graph edge contains a missing endpoint",
            ));
        }
        validate_id(edge.relation.as_str(), limits)?;
        validate_id(edge.scope.as_str(), limits)?;
        validate_id(edge.provenance.as_str(), limits)?;
        if !relations.contains(&edge.relation)
            || !scopes.contains(&edge.scope)
            || !provenances.contains(&edge.provenance)
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "graph edge value is outside the closed owner vocabulary",
            ));
        }
    }
    for unresolved in &bundle.graph.unresolved {
        validate_unresolved(unresolved, &node_ids, limits)?;
        if !authorities.contains(&unresolved.target.authority)
            || !relations.contains(&unresolved.relation)
            || !scopes.contains(&unresolved.scope)
            || !provenances.contains(&unresolved.provenance)
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "unresolved dependency is outside the closed owner vocabulary",
            ));
        }
        if !bundle
            .policy
            .propagation_rules
            .iter()
            .any(|rule| unresolved_rule_matches(unresolved, rule))
        {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "unresolved dependency has no exact applicable policy rule",
            ));
        }
    }
    let used_authorities: BTreeSet<_> = bundle
        .graph
        .nodes
        .iter()
        .map(|node| &node.id.authority)
        .chain(
            bundle
                .graph
                .unresolved
                .iter()
                .map(|value| &value.target.authority),
        )
        .collect();
    let used_relations: BTreeSet<_> = bundle
        .graph
        .edges
        .iter()
        .map(|edge| &edge.relation)
        .chain(bundle.graph.unresolved.iter().map(|value| &value.relation))
        .collect();
    let used_scopes: BTreeSet<_> = bundle
        .graph
        .edges
        .iter()
        .map(|edge| &edge.scope)
        .chain(bundle.graph.unresolved.iter().map(|value| &value.scope))
        .collect();
    let used_provenances: BTreeSet<_> = bundle
        .graph
        .edges
        .iter()
        .map(|edge| &edge.provenance)
        .chain(
            bundle
                .graph
                .unresolved
                .iter()
                .map(|value| &value.provenance),
        )
        .collect();
    if authorities != used_authorities
        || relations != used_relations
        || scopes != used_scopes
        || provenances != used_provenances
    {
        return Err(Error::new(
            ErrorCode::Malformed,
            "graph owner vocabulary is incomplete or contains unused members",
        ));
    }

    reject_adjacent_duplicates(
        &bundle.corpus.vectors,
        |value| &value.id,
        "duplicate vector definition",
    )?;
    for vector in &bundle.corpus.vectors {
        validate_id(vector.id.as_str(), limits)?;
        if Sha256::digest(&vector.owner_bytes).as_slice() != vector.digest {
            return Err(Error::new(
                ErrorCode::CanonicalMismatch,
                "vector definition digest mismatch",
            ));
        }
    }

    reject_adjacent_duplicates(
        &bundle.evidence.records,
        |value| &value.id,
        "duplicate evidence identity",
    )?;
    for record in &bundle.evidence.records {
        validate_id(record.id.as_str(), limits)?;
        validate_subject(&record.subject, limits)?;
        validate_id(record.scope.as_str(), limits)?;
        validate_id(record.provenance.as_str(), limits)?;
        validate_strings(&record.rights, limits.max_identity_bytes)?;
        validate_strings(&record.limitations, limits.max_identity_bytes)?;
    }
    for node in &bundle.graph.nodes {
        if let Some(evidence) = &node.evidence {
            let Some(record) = bundle
                .evidence
                .records
                .iter()
                .find(|record| &record.id == evidence)
            else {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "graph references absent FR-337 evidence",
                ));
            };
            if record.subject != node.id {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "graph evidence identity is rebound to another subject",
                ));
            }
        }
    }

    reject_adjacent_duplicates(
        &bundle.policy.seed_rules,
        |value| &value.id,
        "duplicate seed-rule identity",
    )?;
    reject_adjacent_duplicates(
        &bundle.policy.propagation_rules,
        |value| &value.id,
        "duplicate propagation-rule identity",
    )?;
    reject_adjacent_duplicates(
        &bundle.policy.vector_rules,
        |value| &value.id,
        "duplicate vector-rule identity",
    )?;
    let mut seed_tuples = BTreeSet::new();
    for rule in &bundle.policy.seed_rules {
        validate_id(rule.id.as_str(), limits)?;
        if !seed_tuples.insert(rule.assessment) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "duplicate policy seed tuple",
            ));
        }
    }
    if seed_tuples != admitted_seed_tuples() {
        return Err(Error::new(
            ErrorCode::Malformed,
            "policy seed table does not cover the closed admitted tuple set",
        ));
    }
    let mut propagation_keys = BTreeSet::new();
    for rule in &bundle.policy.propagation_rules {
        validate_id(rule.id.as_str(), limits)?;
        validate_id(rule.relation.as_str(), limits)?;
        validate_id(rule.scope.as_str(), limits)?;
        let key = (
            rule.relation.clone(),
            rule.source_kind,
            rule.target_kind,
            rule.direction,
            rule.scope.clone(),
        );
        if !propagation_keys.insert(key) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "duplicate exact propagation matcher",
            ));
        }
    }
    let vector_ids: BTreeSet<_> = bundle
        .corpus
        .vectors
        .iter()
        .map(|vector| vector.id.clone())
        .collect();
    for rule in &bundle.policy.vector_rules {
        validate_id(rule.id.as_str(), limits)?;
        validate_vector_key(&rule.key, limits)?;
        if rule.key.profile != bundle.policy.identity {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "vector rule policy profile differs from owning policy",
            ));
        }
        if !vector_ids.contains(&rule.key.vector) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "policy references missing vector definition",
            ));
        }
    }
    for vector in &bundle.corpus.vectors {
        if !bundle
            .policy
            .vector_rules
            .iter()
            .any(|rule| rule.key.vector == vector.id)
        {
            return Err(Error::new(
                ErrorCode::Malformed,
                "catalog vector lacks a policy record",
            ));
        }
    }
    validate_id(bundle.policy.identity.as_str(), limits)?;
    validate_id(bundle.policy.revision.as_str(), limits)?;
    if bundle.policy.recompute_digest()? != bundle.policy.digest {
        return Err(Error::new(
            ErrorCode::CanonicalMismatch,
            "impact policy digest mismatch",
        ));
    }
    validate_manifest(&bundle.manifest, candidate, limits)?;
    for entry in &bundle.manifest.entries {
        if !vector_ids.contains(&entry.vector) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "provider manifest references an absent vector definition",
            ));
        }
    }
    validate_vector_composition(&bundle.policy, &bundle.manifest, candidate)?;
    let reason_bytes = bundle
        .graph
        .unresolved
        .iter()
        .try_fold(0_u64, |total, value| {
            checked_add(total, count(value.reason.len())?)
        })?;
    let reason_bytes = bundle
        .manifest
        .entries
        .iter()
        .try_fold(reason_bytes, |total, value| match &value.status {
            ManifestStatus::Unknown(reason) => checked_add(total, count(reason.len())?),
            ManifestStatus::Supported | ManifestStatus::Excluded => Ok(total),
        })?;
    if reason_bytes > limits.max_total_reason_bytes {
        return Err(Error::exhausted(
            Resource::TotalReasonBytes,
            "aggregate reasons exceed request ceiling",
        ));
    }
    Ok(())
}

fn reject_adjacent_duplicates<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> &K,
    detail: &'static str,
) -> Result<()> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        return Err(Error::new(ErrorCode::Malformed, detail));
    }
    Ok(())
}

fn reject_exact_duplicates<T: Eq>(values: &[T], detail: &'static str) -> Result<()> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::new(ErrorCode::Malformed, detail));
    }
    Ok(())
}

fn validate_id(value: &str, limits: Limits) -> Result<()> {
    let length = count(value.len())?;
    if value.trim().is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidIdentity,
            "projection identity is empty",
        ));
    }
    if length > limits.max_identity_bytes {
        return Err(Error::exhausted(
            Resource::IdentityBytes,
            "projection identity exceeds request ceiling",
        ));
    }
    Ok(())
}

fn validate_capability_selection(value: &CapabilitySelection, limits: Limits) -> Result<()> {
    validate_id(value.role().as_str(), limits)?;
    validate_id(value.provider().as_str(), limits)?;
    validate_id(value.build().as_str(), limits)?;
    validate_id(value.contract().as_str(), limits)?;
    validate_id(value.options().as_str(), limits)
}

fn validate_subject(subject: &QualifiedSubjectId, limits: Limits) -> Result<()> {
    validate_id(subject.authority.as_str(), limits)?;
    validate_id(subject.revision.as_str(), limits)?;
    validate_id(subject.local.as_str(), limits)
}

fn validate_subject_option(subject: Option<&QualifiedSubjectId>, limits: Limits) -> Result<()> {
    if let Some(subject) = subject {
        validate_subject(subject, limits)?;
    }
    Ok(())
}

fn validate_strings(values: &[String], limit: u64) -> Result<()> {
    for value in values {
        if value.trim().is_empty() || count(value.len())? > limit {
            return Err(Error::new(
                ErrorCode::InvalidIdentity,
                "bounded owner string is empty or over limit",
            ));
        }
    }
    Ok(())
}

fn validate_correspondence(value: &Correspondence) -> Result<()> {
    let structural_ok = match value.structural {
        StructuralAssessment::Unchanged | StructuralAssessment::PayloadBreaking => {
            value.presence == SubjectPresence::BothPresent
        }
        StructuralAssessment::Removed => value.presence == SubjectPresence::BaselineOnly,
        StructuralAssessment::AdditiveCompatible => matches!(
            value.presence,
            SubjectPresence::BothPresent | SubjectPresence::CandidateOnly
        ),
        StructuralAssessment::Unknown => true,
    };
    let identity_ok = match value.presence {
        SubjectPresence::BothPresent => value.baseline.is_some() && value.candidate.is_some(),
        SubjectPresence::BaselineOnly => value.baseline.is_some() && value.candidate.is_none(),
        SubjectPresence::CandidateOnly => value.baseline.is_none() && value.candidate.is_some(),
        SubjectPresence::Unknown => true,
    };
    let behavior_ok = match value.behavioral {
        BehavioralAssessment::Conforming | BehavioralAssessment::Regressed => {
            value.presence == SubjectPresence::BothPresent
                && value.comparable_results == ComparableResults::Both
        }
        BehavioralAssessment::Absent => {
            value.presence == SubjectPresence::BothPresent
                && value.comparable_results == ComparableResults::CandidateMissing
        }
        BehavioralAssessment::NotAssessed
        | BehavioralAssessment::Unsupported
        | BehavioralAssessment::Unavailable
        | BehavioralAssessment::Unknown => true,
    };
    let comparable_key_ok = match value.comparable_results {
        ComparableResults::Both | ComparableResults::CandidateMissing => {
            value.comparable_key.is_some()
        }
        ComparableResults::NotApplicable => value.comparable_key.is_none(),
    };
    if !structural_ok || !identity_ok || !behavior_ok || !comparable_key_ok {
        return Err(Error::new(
            ErrorCode::OwnerContradiction,
            "owner assessment tuple is outside the admitted table",
        ));
    }
    Ok(())
}

fn admitted_seed_tuples() -> BTreeSet<AssessmentTuple> {
    let structural = [
        StructuralAssessment::Unchanged,
        StructuralAssessment::AdditiveCompatible,
        StructuralAssessment::PayloadBreaking,
        StructuralAssessment::Removed,
        StructuralAssessment::Unknown,
    ];
    let behavioral = [
        BehavioralAssessment::NotAssessed,
        BehavioralAssessment::Conforming,
        BehavioralAssessment::Regressed,
        BehavioralAssessment::Absent,
        BehavioralAssessment::Unsupported,
        BehavioralAssessment::Unavailable,
        BehavioralAssessment::Unknown,
    ];
    let presence = [
        SubjectPresence::BothPresent,
        SubjectPresence::BaselineOnly,
        SubjectPresence::CandidateOnly,
        SubjectPresence::Unknown,
    ];
    let mut result = BTreeSet::new();
    for structural in structural {
        for behavioral in behavioral {
            for presence in presence {
                let structural_ok = match structural {
                    StructuralAssessment::Unchanged | StructuralAssessment::PayloadBreaking => {
                        presence == SubjectPresence::BothPresent
                    }
                    StructuralAssessment::Removed => presence == SubjectPresence::BaselineOnly,
                    StructuralAssessment::AdditiveCompatible => matches!(
                        presence,
                        SubjectPresence::BothPresent | SubjectPresence::CandidateOnly
                    ),
                    StructuralAssessment::Unknown => true,
                };
                let behavior_ok = !matches!(
                    behavioral,
                    BehavioralAssessment::Conforming
                        | BehavioralAssessment::Regressed
                        | BehavioralAssessment::Absent
                ) || presence == SubjectPresence::BothPresent;
                if structural_ok && behavior_ok {
                    result.insert(AssessmentTuple {
                        structural,
                        behavioral,
                        presence,
                    });
                }
            }
        }
    }
    result
}

fn validate_unresolved(
    value: &UnresolvedDependency,
    node_ids: &BTreeSet<QualifiedSubjectId>,
    limits: Limits,
) -> Result<()> {
    if !node_ids.contains(&value.root)
        || !node_ids.contains(&value.predecessor)
        || node_ids.contains(&value.target)
    {
        return Err(Error::new(
            ErrorCode::Malformed,
            "unresolved overlay endpoint classification is invalid",
        ));
    }
    validate_subject(&value.target, limits)?;
    validate_id(value.relation.as_str(), limits)?;
    validate_id(value.scope.as_str(), limits)?;
    validate_id(value.provenance.as_str(), limits)?;
    if value.reason.trim().is_empty() || count(value.reason.len())? > limits.max_reason_bytes {
        return Err(Error::new(
            ErrorCode::InvalidIdentity,
            "unresolved reason is empty or over limit",
        ));
    }
    if value.recompute_digest()? != value.digest {
        return Err(Error::new(
            ErrorCode::CanonicalMismatch,
            "unresolved overlay digest mismatch",
        ));
    }
    Ok(())
}

fn validate_vector_key(key: &VectorKey, limits: Limits) -> Result<()> {
    validate_id(key.vector.as_str(), limits)?;
    validate_subject(&key.subject, limits)?;
    validate_id(key.scope.as_str(), limits)?;
    validate_id(key.method.as_str(), limits)?;
    validate_id(key.environment.as_str(), limits)?;
    validate_id(key.provider_role.as_str(), limits)?;
    validate_id(key.profile.as_str(), limits)
}

fn validate_manifest(
    manifest: &ManifestProjection,
    candidate: &ValidatedLock,
    limits: Limits,
) -> Result<()> {
    reject_exact_duplicates(&manifest.providers, "duplicate manifest provider selection")?;
    reject_exact_duplicates(&manifest.entries, "duplicate provider-manifest entry")?;
    let provider_set: BTreeSet<_> = manifest.providers.iter().collect();
    for provider in &manifest.providers {
        validate_capability_selection(provider, limits)?;
        candidate.admit_capability(provider).map_err(|_| {
            Error::new(
                ErrorCode::CapabilityMismatch,
                "manifest provider is not admitted by candidate lock",
            )
        })?;
    }
    let mut entry_keys = BTreeSet::new();
    for entry in &manifest.entries {
        validate_capability_selection(&entry.provider, limits)?;
        if !provider_set.contains(&entry.provider) {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "manifest entry references a provider outside its complete inventory",
            ));
        }
        if !entry_keys.insert((entry.provider.clone(), entry.vector.clone())) {
            return Err(Error::new(
                ErrorCode::Malformed,
                "duplicate provider and vector manifest key",
            ));
        }
        validate_id(entry.vector.as_str(), limits)?;
        if let ManifestStatus::Unknown(reason) = &entry.status {
            if reason.trim().is_empty() || count(reason.len())? > limits.max_reason_bytes {
                return Err(Error::new(
                    ErrorCode::InvalidIdentity,
                    "manifest reason is empty or over limit",
                ));
            }
        }
    }
    Ok(())
}

fn validate_vector_composition(
    policy: &ImpactPolicy,
    manifest: &ManifestProjection,
    candidate: &ValidatedLock,
) -> Result<()> {
    let mut requirements = BTreeMap::new();
    for rule in &policy.vector_rules {
        if requirements
            .insert(rule.key.clone(), rule.requirement)
            .is_some_and(|requirement| requirement != rule.requirement)
        {
            return Err(Error::new(
                ErrorCode::Malformed,
                "one complete vector key has conflicting requirements",
            ));
        }
    }
    for key in requirements.keys() {
        let matches = manifest
            .providers
            .iter()
            .filter(|provider| provider.role().as_str() == key.provider_role.as_str())
            .count();
        if matches != candidate.capability_role_count(key.provider_role.as_str()) {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "manifest provider inventory is incomplete for requested role",
            ));
        }
        if matches > 1 {
            return Err(Error::new(
                ErrorCode::CapabilityMismatch,
                "requested provider role has multiple admitted candidates",
            ));
        }
    }
    Ok(())
}

fn changed_roots(comparison: &ComparisonProjection) -> Result<Vec<ChangedRoot>> {
    let mut roots = Vec::with_capacity(comparison.correspondences.len());
    let mut assessments = BTreeMap::new();
    for correspondence in &comparison.correspondences {
        let subject = match correspondence.presence {
            SubjectPresence::BothPresent | SubjectPresence::CandidateOnly => {
                correspondence.candidate.clone()
            }
            SubjectPresence::BaselineOnly => correspondence.baseline.clone(),
            SubjectPresence::Unknown => correspondence
                .candidate
                .clone()
                .or_else(|| correspondence.baseline.clone()),
        }
        .ok_or_else(|| {
            Error::new(
                ErrorCode::OwnerContradiction,
                "changed correspondence has no directional subject",
            )
        })?;
        let assessment = AssessmentTuple {
            structural: correspondence.structural,
            behavioral: correspondence.behavioral,
            presence: correspondence.presence,
        };
        if assessments.insert(subject.clone(), assessment).is_some() {
            return Err(Error::new(
                ErrorCode::OwnerContradiction,
                "directional subject has duplicate or conflicting assessments",
            ));
        }
        roots.push(ChangedRoot {
            subject,
            assessment,
        });
    }
    roots.sort();
    reject_exact_duplicates(&roots, "duplicate changed root and assessment")?;
    Ok(roots)
}

fn oriented_graph(
    graph: &GraphProjection,
    policy: &ImpactPolicy,
    limits: Limits,
    timer: Timer<'_>,
) -> Result<BTreeMap<Effect, Vec<OrientedEdge>>> {
    let node_count = count(graph.nodes.len())?;
    if node_count > limits.max_topology_items {
        return Err(Error::exhausted(
            Resource::TopologyItems,
            "graph nodes exceed atomic topology ceiling",
        ));
    }
    let edge_capacity = limits
        .max_topology_items
        .checked_sub(node_count)
        .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "topology capacity underflow"))?;
    let kinds: BTreeMap<_, _> = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node.id.kind))
        .collect();
    let mut result: BTreeMap<Effect, BTreeSet<OrientedEdge>> = BTreeMap::new();
    let mut retained_edges = 0_u64;
    for edge in &graph.edges {
        for rule in &policy.propagation_rules {
            timer.check()?;
            if rule.relation != edge.relation || rule.scope != edge.scope {
                continue;
            }
            let (source, target) = match rule.direction {
                Direction::Forward => (&edge.source, &edge.target),
                Direction::Reverse => (&edge.target, &edge.source),
            };
            if kinds.get(source) != Some(&rule.source_kind)
                || kinds.get(target) != Some(&rule.target_kind)
            {
                continue;
            }
            for effect in &rule.effects {
                let oriented = OrientedEdge {
                    effect: *effect,
                    source: source.clone(),
                    relation: edge.relation.clone(),
                    target: target.clone(),
                    scope: edge.scope.clone(),
                    provenance: edge.provenance.clone(),
                    direction: rule.direction,
                };
                let effect_edges = result.entry(*effect).or_default();
                if effect_edges.contains(&oriented) {
                    continue;
                }
                if retained_edges >= edge_capacity {
                    return Err(Error::exhausted(
                        Resource::TopologyItems,
                        "effect-oriented topology exceeds atomic ceiling",
                    ));
                }
                effect_edges.insert(oriented);
                retained_edges = checked_add(retained_edges, 1)?;
            }
        }
    }
    Ok(result
        .into_iter()
        .map(|(effect, edges)| (effect, edges.into_iter().collect()))
        .collect())
}

fn unresolved_rule_matches(value: &UnresolvedDependency, rule: &PropagationRule) -> bool {
    rule.direction == Direction::Forward
        && rule.relation == value.relation
        && rule.scope == value.scope
        && rule.source_kind == value.predecessor.kind
        && rule.target_kind == value.target.kind
}

fn unresolved_carries(value: &UnresolvedDependency, policy: &ImpactPolicy, effect: Effect) -> bool {
    policy
        .propagation_rules
        .iter()
        .any(|rule| unresolved_rule_matches(value, rule) && rule.effects.contains(&effect))
}

fn validate_unresolved_grounding(
    roots: &[ChangedRoot],
    unresolved: &[UnresolvedDependency],
    policy: &ImpactPolicy,
    oriented: &BTreeMap<Effect, Vec<OrientedEdge>>,
    timer: Timer<'_>,
) -> Result<()> {
    let root_map: BTreeMap<_, _> = roots
        .iter()
        .map(|root| (root.subject.clone(), root))
        .collect();
    for entry in unresolved {
        let root = root_map.get(&entry.root).ok_or_else(|| {
            Error::new(
                ErrorCode::ForeignInput,
                "unresolved dependency root is not an actual changed root",
            )
        })?;
        let seeded = seed_effects(root, policy)?;
        for effect in [Effect::Affectedness, Effect::Staleness] {
            if !unresolved_carries(entry, policy, effect) {
                continue;
            }
            if !seeded.contains(&effect) {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "unresolved dependency effect is not seeded by its changed root",
                ));
            }
            let mut adjacency: BTreeMap<_, Vec<_>> = BTreeMap::new();
            for edge in oriented.get(&effect).map_or(&[][..], Vec::as_slice) {
                adjacency
                    .entry(edge.source.clone())
                    .or_default()
                    .push(edge.clone());
            }
            for edges in adjacency.values_mut() {
                edges.sort_by(canonical_edge_cmp);
            }
            if !reachable_nodes(&entry.root, &adjacency, timer)?.contains(&entry.predecessor) {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "unresolved dependency predecessor is not effect-reachable from its root",
                ));
            }
        }
    }
    Ok(())
}

fn seed_effects<'a>(root: &ChangedRoot, policy: &'a ImpactPolicy) -> Result<&'a BTreeSet<Effect>> {
    let mut matching = policy
        .seed_rules
        .iter()
        .filter(|rule| rule.assessment == root.assessment);
    let first = matching.next().ok_or_else(|| {
        Error::new(
            ErrorCode::Malformed,
            "changed root has no exact seed-effect rule",
        )
    })?;
    if matching.next().is_some() {
        return Err(Error::new(
            ErrorCode::Malformed,
            "changed root has multiple seed-effect rules",
        ));
    }
    Ok(&first.effects)
}

fn traversal_units(
    roots: &[ChangedRoot],
    policy: &ImpactPolicy,
) -> Result<Vec<(QualifiedSubjectId, Effect)>> {
    let mut units = Vec::new();
    for root in roots {
        for effect in seed_effects(root, policy)? {
            units.push((root.subject.clone(), *effect));
        }
    }
    units.sort();
    Ok(units)
}

fn incomplete_before_traversal(
    roots: &[ChangedRoot],
    policy: &ImpactPolicy,
    cause: Resource,
    usage: Usage,
    limits: Limits,
) -> Result<TraversalState> {
    let units = traversal_units(roots, policy)?;
    let frontier: Vec<_> = units
        .iter()
        .map(|(root, effect)| FrontierEntry {
            root: root.clone(),
            effect: *effect,
            predecessor: None,
            target: root.clone(),
            relation: None,
            scope: None,
            provenance: None,
            depth: 0,
        })
        .collect();
    if count(frontier.len())? > limits.max_frontier_entries {
        return Err(Error::exhausted(
            Resource::FrontierEntries,
            "pre-traversal frontier exceeds retention ceiling",
        ));
    }
    let mut usage = usage;
    usage.frontier_entries = count(frontier.len())?;
    Ok(TraversalState {
        witnesses: BTreeMap::new(),
        incomplete: Some(cause),
        incomplete_units: units.into_iter().collect(),
        frontier,
        usage,
    })
}

fn traverse_effects(
    roots: &[ChangedRoot],
    graph: &GraphProjection,
    policy: &ImpactPolicy,
    oriented: &BTreeMap<Effect, Vec<OrientedEdge>>,
    initial_usage: Usage,
    limits: Limits,
    timer: Timer<'_>,
) -> Result<TraversalState> {
    let node_ids: BTreeSet<_> = graph.nodes.iter().map(|node| node.id.clone()).collect();
    let mut adjacency: BTreeMap<(Effect, QualifiedSubjectId), Vec<OrientedEdge>> = BTreeMap::new();
    for (effect, edges) in oriented {
        for edge in edges {
            adjacency
                .entry((*effect, edge.source.clone()))
                .or_default()
                .push(edge.clone());
        }
    }
    for edges in adjacency.values_mut() {
        edges.sort_by(canonical_edge_cmp);
    }
    let mut state = TraversalState {
        usage: initial_usage,
        ..TraversalState::default()
    };
    for root in roots {
        if !node_ids.contains(&root.subject) {
            return Err(Error::new(
                ErrorCode::ForeignInput,
                "changed root is absent from evidence graph",
            ));
        }
    }
    let units = traversal_units(roots, policy)?;
    for (unit_index, (root, effect)) in units.iter().enumerate() {
        let mut queue = VecDeque::from([WorkItem {
            node: root.clone(),
            path: Vec::new(),
            incoming: None,
            depth: 0,
        }]);
        let mut visited = BTreeSet::new();
        let mut discovered = BTreeSet::from([root.clone()]);
        while let Some(work) = queue.pop_front() {
            if visited.contains(&work.node) {
                continue;
            }
            if state.usage.traversal_steps >= limits.max_traversal_steps {
                queue.push_front(work);
                stop_traversal(
                    &mut state,
                    Resource::TraversalSteps,
                    queue_frontier(root, *effect, &queue),
                    &units[unit_index..],
                    limits,
                )?;
                return Ok(state);
            }
            let outgoing = adjacency
                .get(&(*effect, work.node.clone()))
                .map_or(&[][..], Vec::as_slice);
            if !outgoing.is_empty() && state.usage.traversed_edges >= limits.max_traversed_edges {
                let mut frontier = queue_frontier(root, *effect, &queue);
                for pending in outgoing {
                    frontier.push(FrontierEntry {
                        root: root.clone(),
                        effect: *effect,
                        predecessor: Some(pending.source.clone()),
                        target: pending.target.clone(),
                        relation: Some(pending.relation.clone()),
                        scope: Some(pending.scope.clone()),
                        provenance: Some(pending.provenance.clone()),
                        depth: checked_add(work.depth, 1)?,
                    });
                }
                stop_traversal(
                    &mut state,
                    Resource::TraversedEdges,
                    frontier,
                    &units[unit_index..],
                    limits,
                )?;
                return Ok(state);
            }
            let next_path_edges = checked_add(state.usage.path_edges, count(work.path.len())?)?;
            if next_path_edges > limits.max_path_edges {
                queue.push_front(work);
                stop_traversal(
                    &mut state,
                    Resource::PathEdges,
                    queue_frontier(root, *effect, &queue),
                    &units[unit_index..],
                    limits,
                )?;
                return Ok(state);
            }
            let next_witness_count = checked_add(state.usage.witnesses, 1)?;
            if next_witness_count > limits.max_witnesses {
                queue.push_front(work);
                stop_traversal(
                    &mut state,
                    Resource::Witnesses,
                    queue_frontier(root, *effect, &queue),
                    &units[unit_index..],
                    limits,
                )?;
                return Ok(state);
            }
            let witness_bytes = encoded_witness_len(root, *effect, &work)?;
            let next_witness_bytes = checked_add(state.usage.witness_bytes, witness_bytes)?;
            if next_witness_bytes > limits.max_witness_bytes {
                queue.push_front(work);
                stop_traversal(
                    &mut state,
                    Resource::WitnessBytes,
                    queue_frontier(root, *effect, &queue),
                    &units[unit_index..],
                    limits,
                )?;
                return Ok(state);
            }
            timer.check()?;
            visited.insert(work.node.clone());
            state.usage.traversal_steps = checked_add(state.usage.traversal_steps, 1)?;
            state.usage.witnesses = next_witness_count;
            state.usage.path_edges = next_path_edges;
            state.usage.witness_bytes = next_witness_bytes;
            state.witnesses.insert(
                (root.clone(), work.node.clone(), *effect),
                Witness {
                    root: root.clone(),
                    effect: *effect,
                    subject: work.node.clone(),
                    path: work.path.clone(),
                },
            );
            for (edge_index, edge) in outgoing.iter().enumerate() {
                if state.usage.traversed_edges >= limits.max_traversed_edges {
                    let mut frontier = queue_frontier(root, *effect, &queue);
                    for pending in &outgoing[edge_index..] {
                        frontier.push(FrontierEntry {
                            root: root.clone(),
                            effect: *effect,
                            predecessor: Some(pending.source.clone()),
                            target: pending.target.clone(),
                            relation: Some(pending.relation.clone()),
                            scope: Some(pending.scope.clone()),
                            provenance: Some(pending.provenance.clone()),
                            depth: checked_add(work.depth, 1)?,
                        });
                    }
                    stop_traversal(
                        &mut state,
                        Resource::TraversedEdges,
                        frontier,
                        &units[unit_index..],
                        limits,
                    )?;
                    return Ok(state);
                }
                timer.check()?;
                state.usage.traversed_edges = checked_add(state.usage.traversed_edges, 1)?;
                if discovered.insert(edge.target.clone()) {
                    queue.push_back(WorkItem {
                        node: edge.target.clone(),
                        path: extended_path(&work.path, edge),
                        incoming: Some(edge.clone()),
                        depth: checked_add(work.depth, 1)?,
                    });
                }
            }
        }
    }
    Ok(state)
}

fn extended_path(path: &[OrientedEdge], edge: &OrientedEdge) -> Vec<OrientedEdge> {
    let mut result = Vec::with_capacity(path.len().saturating_add(1));
    result.extend_from_slice(path);
    result.push(edge.clone());
    result
}

fn canonical_edge_cmp(left: &OrientedEdge, right: &OrientedEdge) -> Ordering {
    (
        &left.source,
        &left.target,
        left.effect,
        &left.relation,
        &left.scope,
        &left.provenance,
        left.direction,
    )
        .cmp(&(
            &right.source,
            &right.target,
            right.effect,
            &right.relation,
            &right.scope,
            &right.provenance,
            right.direction,
        ))
}

fn encoded_witness_len(root: &QualifiedSubjectId, effect: Effect, work: &WorkItem) -> Result<u64> {
    let mut writer = CanonicalWriter::counting();
    encode_subject(&mut writer, root)?;
    writer.u8(effect_code(effect))?;
    encode_subject(&mut writer, &work.node)?;
    writer.len(work.path.len())?;
    for edge in &work.path {
        encode_oriented_edge(&mut writer, edge)?;
    }
    Ok(writer.written_len())
}

fn stop_traversal(
    state: &mut TraversalState,
    cause: Resource,
    mut frontier: Vec<FrontierEntry>,
    incomplete_units: &[(QualifiedSubjectId, Effect)],
    limits: Limits,
) -> Result<()> {
    for (root, effect) in incomplete_units.iter().skip(1) {
        frontier.push(FrontierEntry {
            root: root.clone(),
            effect: *effect,
            predecessor: None,
            target: root.clone(),
            relation: None,
            scope: None,
            provenance: None,
            depth: 0,
        });
    }
    frontier.sort();
    frontier.dedup();
    if count(frontier.len())? > limits.max_frontier_entries {
        return Err(Error::exhausted(
            Resource::FrontierEntries,
            "exact traversal frontier exceeds retention ceiling",
        ));
    }
    state.usage.frontier_entries = count(frontier.len())?;
    state.incomplete = Some(cause);
    state.incomplete_units = incomplete_units.iter().cloned().collect();
    state.frontier = frontier;
    Ok(())
}

fn queue_frontier(
    root: &QualifiedSubjectId,
    effect: Effect,
    queue: &VecDeque<WorkItem>,
) -> Vec<FrontierEntry> {
    queue
        .iter()
        .map(|work| frontier_entry(root, effect, work))
        .collect()
}

fn frontier_entry(root: &QualifiedSubjectId, effect: Effect, work: &WorkItem) -> FrontierEntry {
    FrontierEntry {
        root: root.clone(),
        effect,
        predecessor: work.incoming.as_ref().map(|edge| edge.source.clone()),
        target: work.node.clone(),
        relation: work.incoming.as_ref().map(|edge| edge.relation.clone()),
        scope: work.incoming.as_ref().map(|edge| edge.scope.clone()),
        provenance: work.incoming.as_ref().map(|edge| edge.provenance.clone()),
        depth: work.depth,
    }
}

fn compose_subject_records(
    roots: &[ChangedRoot],
    graph: &GraphProjection,
    policy: &ImpactPolicy,
    circular_unknowns: &BTreeSet<(QualifiedSubjectId, QualifiedSubjectId, Effect)>,
    traversal: &TraversalState,
    limits: Limits,
) -> Result<Vec<SubjectRecord>> {
    let mut records = Vec::new();
    for root in roots {
        let mut subjects: BTreeSet<_> = graph.nodes.iter().map(|node| node.id.clone()).collect();
        subjects.extend(
            graph
                .unresolved
                .iter()
                .filter(|entry| entry.root == root.subject)
                .map(|entry| entry.target.clone()),
        );
        for subject in &subjects {
            if count(records.len())? >= limits.max_subject_records {
                records.sort();
                return Ok(records);
            }
            let affectedness = traversal
                .witnesses
                .get(&(root.subject.clone(), subject.clone(), Effect::Affectedness))
                .cloned();
            let staleness = traversal
                .witnesses
                .get(&(root.subject.clone(), subject.clone(), Effect::Staleness))
                .cloned();
            let unresolved = graph.unresolved.iter().any(|entry| {
                entry.root == root.subject
                    && entry.target == *subject
                    && unresolved_carries(entry, policy, Effect::Affectedness)
            });
            let incomplete_affectedness = traversal
                .incomplete_units
                .contains(&(root.subject.clone(), Effect::Affectedness));
            let circular_unknown = circular_unknowns.contains(&(
                root.subject.clone(),
                subject.clone(),
                Effect::Affectedness,
            ));
            let reachability = if circular_unknown {
                Reachability::Unknown
            } else if affectedness.is_some() {
                Reachability::Affected
            } else if unresolved || incomplete_affectedness {
                Reachability::Unknown
            } else {
                Reachability::Unaffected
            };
            records.push(SubjectRecord {
                root: root.subject.clone(),
                subject: subject.clone(),
                reachability,
                affectedness_witness: affectedness,
                staleness_witness: staleness,
            });
        }
    }
    records.sort();
    Ok(records)
}

fn subject_record_keys(
    roots: &[ChangedRoot],
    graph: &GraphProjection,
) -> BTreeSet<(QualifiedSubjectId, QualifiedSubjectId)> {
    let mut keys = BTreeSet::new();
    for root in roots {
        for node in &graph.nodes {
            keys.insert((root.subject.clone(), node.id.clone()));
        }
        for unresolved in graph
            .unresolved
            .iter()
            .filter(|entry| entry.root == root.subject)
        {
            keys.insert((root.subject.clone(), unresolved.target.clone()));
        }
    }
    keys
}

fn compose_freshness(
    roots: &[ChangedRoot],
    graph: &GraphProjection,
    policy: &ImpactPolicy,
    circular_unknowns: &BTreeSet<(QualifiedSubjectId, QualifiedSubjectId, Effect)>,
    evidence: &EvidenceProjection,
    traversal: &TraversalState,
) -> Result<Vec<FreshnessRecord>> {
    let mut records = Vec::new();
    for root in roots {
        for owner in &evidence.records {
            let witness = traversal.witnesses.get(&(
                root.subject.clone(),
                owner.subject.clone(),
                Effect::Staleness,
            ));
            let unresolved = graph.unresolved.iter().any(|entry| {
                entry.root == root.subject
                    && entry.target == owner.subject
                    && unresolved_carries(entry, policy, Effect::Staleness)
            });
            let circular_unknown = circular_unknowns.contains(&(
                root.subject.clone(),
                owner.subject.clone(),
                Effect::Staleness,
            ));
            let propagation = if circular_unknown {
                FreshnessPropagation::Unknown
            } else if witness.is_some() {
                FreshnessPropagation::Reached
            } else if unresolved {
                FreshnessPropagation::Unknown
            } else if traversal
                .incomplete_units
                .contains(&(root.subject.clone(), Effect::Staleness))
            {
                FreshnessPropagation::Incomplete
            } else {
                FreshnessPropagation::NotReached
            };
            let freshness = transition_freshness(owner.freshness, propagation);
            let transition_witness =
                if owner.freshness == Freshness::Current && freshness == Freshness::Stale {
                    witness.cloned()
                } else {
                    None
                };
            records.push(FreshnessRecord {
                root: root.subject.clone(),
                evidence: owner.id.clone(),
                subject: owner.subject.clone(),
                relation: owner.relation,
                owner_bytes: owner.owner_bytes.clone(),
                scope: owner.scope.clone(),
                provenance: owner.provenance.clone(),
                prior: owner.freshness,
                propagation,
                freshness,
                witness: transition_witness,
            });
        }
    }
    records.sort();
    Ok(records)
}

/// Applies the complete FR-005 freshness transition table.
#[must_use]
pub const fn transition_freshness(
    prior: Freshness,
    propagation: FreshnessPropagation,
) -> Freshness {
    match prior {
        Freshness::Current => match propagation {
            FreshnessPropagation::Reached => Freshness::Stale,
            FreshnessPropagation::NotReached => Freshness::Current,
            FreshnessPropagation::Unknown | FreshnessPropagation::Incomplete => Freshness::Unknown,
        },
        Freshness::Stale => Freshness::Stale,
        Freshness::Unknown => Freshness::Unknown,
    }
}

fn classify_cycles(
    roots: &[ChangedRoot],
    graph: &GraphProjection,
    oriented: &BTreeMap<Effect, Vec<OrientedEdge>>,
    policy: &ImpactPolicy,
    limits: Limits,
    timer: Timer<'_>,
) -> Result<CycleAnalysis> {
    let mut analysis = CycleAnalysis::default();
    let mut topologies = BTreeMap::new();
    for effect in [Effect::Affectedness, Effect::Staleness] {
        let edges = oriented.get(&effect).map_or(&[][..], Vec::as_slice);
        let mut nodes = BTreeSet::new();
        let mut adjacency: BTreeMap<QualifiedSubjectId, Vec<OrientedEdge>> = BTreeMap::new();
        for edge in edges {
            nodes.insert(edge.source.clone());
            nodes.insert(edge.target.clone());
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .push(edge.clone());
        }
        for outgoing in adjacency.values_mut() {
            outgoing.sort_by(canonical_edge_cmp);
        }
        let mut remaining = nodes.clone();
        let mut components = Vec::new();
        while let Some(seed) = remaining.first().cloned() {
            timer.check()?;
            let forward = reachable_nodes(&seed, &adjacency, timer)?;
            let mut component = BTreeSet::new();
            for candidate in &remaining {
                timer.check()?;
                if forward.contains(candidate)
                    && reachable_nodes(candidate, &adjacency, timer)?.contains(&seed)
                {
                    component.insert(candidate.clone());
                }
            }
            for member in &component {
                remaining.remove(member);
            }
            let self_loop = adjacency
                .get(&seed)
                .is_some_and(|outgoing| outgoing.iter().any(|edge| edge.target == seed));
            if component.len() > 1 || self_loop {
                components.push(component);
            }
        }
        components.sort();
        topologies.insert(effect, (nodes, adjacency, components));
    }
    for root in roots {
        for effect in [Effect::Affectedness, Effect::Staleness] {
            if !seed_effects(root, policy)?.contains(&effect) {
                continue;
            }
            let (nodes, adjacency, components) = topologies.get(&effect).ok_or_else(|| {
                Error::new(ErrorCode::Malformed, "effect cycle topology is absent")
            })?;
            let reachable = reachable_nodes(&root.subject, adjacency, timer)?;
            for component in components {
                if component.is_disjoint(&reachable) {
                    continue;
                }
                let cycle = shortest_cycle(component, adjacency, timer)?;
                let cycle_start = cycle
                    .first()
                    .ok_or_else(|| Error::new(ErrorCode::Malformed, "cycle witness is empty"))?;
                let approach = if cycle_start.source == root.subject {
                    Vec::new()
                } else {
                    shortest_path(&root.subject, &cycle_start.source, nodes, adjacency, timer)?
                        .ok_or_else(|| {
                            Error::new(
                                ErrorCode::Malformed,
                                "reachable cycle lacks a canonical approach path",
                            )
                        })?
                };
                let incoming = approach.last();
                let entry = FrontierEntry {
                    root: root.subject.clone(),
                    effect,
                    predecessor: incoming.map(|edge| edge.source.clone()),
                    target: cycle_start.source.clone(),
                    relation: incoming.map(|edge| edge.relation.clone()),
                    scope: incoming.map(|edge| edge.scope.clone()),
                    provenance: incoming.map(|edge| edge.provenance.clone()),
                    depth: count(approach.len())?,
                };
                let record = CycleRecord {
                    root: root.subject.clone(),
                    effect,
                    component: component.iter().cloned().collect(),
                    cycle,
                    entry,
                };
                let grounded = component.contains(&root.subject)
                    || graph
                        .nodes
                        .iter()
                        .any(|node| component.contains(&node.id) && node.evidence.is_some());
                let next_paths =
                    checked_add(analysis.usage.path_edges, count(record.cycle.len())?)?;
                if next_paths > limits.max_path_edges {
                    analysis.exhausted = Some(Resource::PathEdges);
                    return Ok(analysis);
                }
                let next_witnesses = checked_add(analysis.usage.witnesses, 1)?;
                if next_witnesses > limits.max_witnesses {
                    analysis.exhausted = Some(Resource::Witnesses);
                    return Ok(analysis);
                }
                let mut writer = CanonicalWriter::counting();
                encode_cycle(&mut writer, &record)?;
                let next_bytes = checked_add(analysis.usage.witness_bytes, writer.written_len())?;
                if next_bytes > limits.max_witness_bytes {
                    analysis.exhausted = Some(Resource::WitnessBytes);
                    return Ok(analysis);
                }
                analysis.usage.path_edges = next_paths;
                analysis.usage.witnesses = next_witnesses;
                analysis.usage.witness_bytes = next_bytes;
                if !grounded {
                    for subject in &record.component {
                        analysis.circular_unknowns.insert((
                            record.root.clone(),
                            subject.clone(),
                            record.effect,
                        ));
                    }
                }
                analysis.records.push(record);
            }
        }
    }
    Ok(analysis)
}

fn reachable_nodes(
    start: &QualifiedSubjectId,
    adjacency: &BTreeMap<QualifiedSubjectId, Vec<OrientedEdge>>,
    timer: Timer<'_>,
) -> Result<BTreeSet<QualifiedSubjectId>> {
    let mut result = BTreeSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(node) = queue.pop_front() {
        timer.check()?;
        if !result.insert(node.clone()) {
            continue;
        }
        if let Some(outgoing) = adjacency.get(&node) {
            for edge in outgoing {
                if !result.contains(&edge.target) {
                    queue.push_back(edge.target.clone());
                }
            }
        }
    }
    Ok(result)
}

fn shortest_cycle(
    component: &BTreeSet<QualifiedSubjectId>,
    adjacency: &BTreeMap<QualifiedSubjectId, Vec<OrientedEdge>>,
    timer: Timer<'_>,
) -> Result<Vec<OrientedEdge>> {
    let mut candidates = Vec::new();
    for start in component {
        timer.check()?;
        let Some(outgoing) = adjacency.get(start) else {
            continue;
        };
        for first in outgoing
            .iter()
            .filter(|edge| component.contains(&edge.target))
        {
            if first.target == *start {
                candidates.push(vec![first.clone()]);
                continue;
            }
            if let Some(mut rest) =
                shortest_path(&first.target, start, component, adjacency, timer)?
            {
                let mut candidate = Vec::with_capacity(rest.len().saturating_add(1));
                candidate.push(first.clone());
                candidate.append(&mut rest);
                candidates.push(candidate);
            }
        }
    }
    for candidate in &mut candidates {
        rotate_cycle(candidate);
    }
    candidates.sort_by(|left, right| {
        left.len()
            .cmp(&right.len())
            .then_with(|| compare_edge_sequences(left, right))
    });
    candidates.into_iter().next().ok_or_else(|| {
        Error::new(
            ErrorCode::Malformed,
            "cyclic component lacks a directed cycle",
        )
    })
}

fn shortest_path(
    start: &QualifiedSubjectId,
    target: &QualifiedSubjectId,
    component: &BTreeSet<QualifiedSubjectId>,
    adjacency: &BTreeMap<QualifiedSubjectId, Vec<OrientedEdge>>,
    timer: Timer<'_>,
) -> Result<Option<Vec<OrientedEdge>>> {
    let mut queue = VecDeque::from([(start.clone(), Vec::<OrientedEdge>::new())]);
    let mut best_depth: BTreeMap<QualifiedSubjectId, usize> = BTreeMap::new();
    while let Some((node, path)) = queue.pop_front() {
        timer.check()?;
        if &node == target {
            return Ok(Some(path));
        }
        if best_depth
            .get(&node)
            .is_some_and(|depth| *depth <= path.len())
        {
            continue;
        }
        best_depth.insert(node.clone(), path.len());
        if let Some(outgoing) = adjacency.get(&node) {
            for edge in outgoing
                .iter()
                .filter(|edge| component.contains(&edge.target))
            {
                if path.iter().any(|prior| prior.source == edge.target) {
                    continue;
                }
                queue.push_back((edge.target.clone(), extended_path(&path, edge)));
            }
        }
    }
    Ok(None)
}

fn rotate_cycle(cycle: &mut [OrientedEdge]) {
    let Some((index, _)) = cycle
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| canonical_edge_cmp(left, right))
    else {
        return;
    };
    cycle.rotate_left(index);
}

fn compare_edge_sequences(left: &[OrientedEdge], right: &[OrientedEdge]) -> Ordering {
    for (left, right) in left.iter().zip(right) {
        let ordering = canonical_edge_cmp(left, right);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

fn compose_vectors(
    corpus: &CorpusProjection,
    policy: &ImpactPolicy,
    manifest: &ManifestProjection,
    candidate: &ValidatedLock,
) -> Result<Vec<VectorRecord>> {
    let manifest_by_key: BTreeMap<_, _> = manifest
        .entries
        .iter()
        .map(|entry| ((entry.provider.clone(), entry.vector.clone()), entry))
        .collect();
    let mut records = Vec::with_capacity(corpus.vectors.len());
    for definition in &corpus.vectors {
        let rules: Vec<_> = policy
            .vector_rules
            .iter()
            .filter(|rule| rule.key.vector == definition.id)
            .collect();
        if rules.is_empty() {
            return Err(Error::new(
                ErrorCode::Malformed,
                "catalog vector lacks policy selection",
            ));
        }
        let mut by_key: BTreeMap<VectorKey, Vec<&VectorRule>> = BTreeMap::new();
        for rule in rules {
            by_key.entry(rule.key.clone()).or_default().push(rule);
        }
        for (key, key_rules) in by_key {
            let first = key_rules.first().ok_or_else(|| {
                Error::new(
                    ErrorCode::Malformed,
                    "catalog vector lacks policy selection",
                )
            })?;
            if key_rules
                .iter()
                .any(|rule| rule.requirement != first.requirement)
            {
                return Err(Error::new(
                    ErrorCode::Malformed,
                    "one complete vector key has conflicting requirements",
                ));
            }
            let matching: Vec<_> = manifest
                .providers
                .iter()
                .filter(|provider| provider.role().as_str() == key.provider_role.as_str())
                .collect();
            if matching.len() != candidate.capability_role_count(key.provider_role.as_str()) {
                return Err(Error::new(
                    ErrorCode::ForeignInput,
                    "manifest provider inventory is incomplete for requested role",
                ));
            }
            let availability = match matching.as_slice() {
                [] => VectorAvailability::Unavailable,
                [provider] => {
                    match manifest_by_key.get(&((*provider).clone(), definition.id.clone())) {
                        None => VectorAvailability::Unavailable,
                        Some(entry) => match &entry.status {
                            ManifestStatus::Supported => {
                                VectorAvailability::Available(provider.provider().clone())
                            }
                            ManifestStatus::Excluded => {
                                VectorAvailability::Unsupported(provider.provider().clone())
                            }
                            ManifestStatus::Unknown(reason) => VectorAvailability::Unknown {
                                provider: provider.provider().clone(),
                                reason: reason.clone(),
                            },
                        },
                    }
                }
                _ => {
                    return Err(Error::new(
                        ErrorCode::CapabilityMismatch,
                        "requested provider role has multiple admitted candidates",
                    ));
                }
            };
            let mut contributing_rules: Vec<_> =
                key_rules.iter().map(|rule| rule.id.clone()).collect();
            contributing_rules.sort();
            records.push(VectorRecord {
                key,
                definition_bytes: definition.owner_bytes.clone(),
                requirement: first.requirement,
                availability,
                contributing_rules,
            });
        }
    }
    records.sort();
    Ok(records)
}

impl ImpactPolicy {
    /// Recomputes the domain-separated digest over policy content excluding `digest`.
    pub fn recompute_digest(&self) -> Result<[u8; 32]> {
        let mut writer = CanonicalWriter::unbounded();
        writer.string(self.identity.as_str())?;
        writer.string(self.revision.as_str())?;
        let mut seed_rules = self.seed_rules.clone();
        seed_rules.sort();
        writer.len(seed_rules.len())?;
        for rule in &seed_rules {
            encode_seed_rule(&mut writer, rule)?;
        }
        let mut propagation_rules = self.propagation_rules.clone();
        propagation_rules.sort();
        writer.len(propagation_rules.len())?;
        for rule in &propagation_rules {
            encode_propagation_rule(&mut writer, rule)?;
        }
        let mut vector_rules = self.vector_rules.clone();
        vector_rules.sort();
        writer.len(vector_rules.len())?;
        for rule in &vector_rules {
            encode_vector_rule(&mut writer, rule)?;
        }
        let mut hasher = Sha256::new();
        hasher.update(b"quire.impact-policy/v1\0");
        hasher.update(writer.finish());
        Ok(hasher.finalize().into())
    }
}

impl UnresolvedDependency {
    /// Recomputes the domain-separated digest over every unresolved-entry field.
    pub fn recompute_digest(&self) -> Result<[u8; 32]> {
        let mut writer = CanonicalWriter::unbounded();
        encode_subject(&mut writer, &self.root)?;
        encode_subject(&mut writer, &self.predecessor)?;
        encode_subject(&mut writer, &self.target)?;
        writer.string(self.relation.as_str())?;
        writer.string(self.scope.as_str())?;
        writer.string(self.provenance.as_str())?;
        writer.string(&self.reason)?;
        let mut hasher = Sha256::new();
        hasher.update(b"quire.unresolved-impact-dependency/v1\0");
        hasher.update(writer.finish());
        Ok(hasher.finalize().into())
    }
}

fn projection_fingerprint(bundle: &ProjectionBundle) -> Result<[u8; 32]> {
    let mut writer = CanonicalWriter::unbounded();
    writer.string(&bundle.comparison.baseline_lock.to_hex())?;
    writer.string(&bundle.comparison.candidate_lock.to_hex())?;
    writer.string(bundle.comparison.comparison_rule.as_str())?;
    writer.string(bundle.comparison.rule_revision.as_str())?;
    writer.digest(&bundle.comparison.rule_digest)?;
    writer.string(bundle.comparison.change_set.as_str())?;
    writer.digest(&bundle.comparison.change_set_digest)?;
    writer.len(bundle.comparison.correspondences.len())?;
    for value in &bundle.comparison.correspondences {
        encode_correspondence(&mut writer, value)?;
    }
    writer.len(bundle.graph.authorities.len())?;
    for value in &bundle.graph.authorities {
        writer.string(value.as_str())?;
    }
    writer.len(bundle.graph.relations.len())?;
    for value in &bundle.graph.relations {
        writer.string(value.as_str())?;
    }
    writer.len(bundle.graph.scopes.len())?;
    for value in &bundle.graph.scopes {
        writer.string(value.as_str())?;
    }
    writer.len(bundle.graph.provenances.len())?;
    for value in &bundle.graph.provenances {
        writer.string(value.as_str())?;
    }
    writer.len(bundle.graph.nodes.len())?;
    for node in &bundle.graph.nodes {
        encode_subject(&mut writer, &node.id)?;
        encode_optional_string(&mut writer, node.evidence.as_ref().map(EvidenceId::as_str))?;
    }
    writer.len(bundle.graph.edges.len())?;
    for edge in &bundle.graph.edges {
        encode_subject(&mut writer, &edge.source)?;
        writer.string(edge.relation.as_str())?;
        encode_subject(&mut writer, &edge.target)?;
        writer.string(edge.scope.as_str())?;
        writer.string(edge.provenance.as_str())?;
    }
    writer.len(bundle.graph.unresolved.len())?;
    for value in &bundle.graph.unresolved {
        encode_unresolved(&mut writer, value)?;
    }
    writer.len(bundle.corpus.vectors.len())?;
    for vector in &bundle.corpus.vectors {
        writer.string(vector.id.as_str())?;
        writer.bytes(&vector.owner_bytes)?;
        writer.digest(&vector.digest)?;
    }
    writer.len(bundle.evidence.records.len())?;
    for evidence in &bundle.evidence.records {
        writer.string(evidence.id.as_str())?;
        encode_subject(&mut writer, &evidence.subject)?;
        writer.u8(evidence_relation_code(evidence.relation))?;
        writer.bytes(&evidence.owner_bytes)?;
        writer.string(evidence.scope.as_str())?;
        writer.string(evidence.provenance.as_str())?;
        writer.len(evidence.rights.len())?;
        for right in &evidence.rights {
            writer.string(right)?;
        }
        writer.len(evidence.limitations.len())?;
        for limitation in &evidence.limitations {
            writer.string(limitation)?;
        }
        writer.u8(freshness_code(evidence.freshness))?;
    }
    writer.string(bundle.policy.identity.as_str())?;
    writer.string(bundle.policy.revision.as_str())?;
    writer.digest(&bundle.policy.digest)?;
    writer.len(bundle.policy.seed_rules.len())?;
    for rule in &bundle.policy.seed_rules {
        encode_seed_rule(&mut writer, rule)?;
    }
    writer.len(bundle.policy.propagation_rules.len())?;
    for rule in &bundle.policy.propagation_rules {
        encode_propagation_rule(&mut writer, rule)?;
    }
    writer.len(bundle.policy.vector_rules.len())?;
    for rule in &bundle.policy.vector_rules {
        encode_vector_rule(&mut writer, rule)?;
    }
    writer.len(bundle.manifest.providers.len())?;
    for provider in &bundle.manifest.providers {
        encode_selection(&mut writer, provider)?;
    }
    writer.len(bundle.manifest.entries.len())?;
    for entry in &bundle.manifest.entries {
        encode_selection(&mut writer, &entry.provider)?;
        writer.string(entry.vector.as_str())?;
        match &entry.status {
            ManifestStatus::Supported => writer.u8(0)?,
            ManifestStatus::Excluded => writer.u8(1)?,
            ManifestStatus::Unknown(reason) => {
                writer.u8(2)?;
                writer.string(reason)?;
            }
        }
    }
    let mut hasher = Sha256::new();
    hasher.update(b"quire.impact-projections/v1\0");
    hasher.update(writer.finish());
    Ok(hasher.finalize().into())
}

struct CanonicalWriter {
    bytes: Vec<u8>,
    written: u64,
    limit: u64,
    resource: Resource,
    retain: bool,
}

impl CanonicalWriter {
    fn bounded(limit: u64, resource: Resource) -> Self {
        Self {
            bytes: Vec::new(),
            written: 0,
            limit,
            resource,
            retain: true,
        }
    }

    fn unbounded() -> Self {
        Self::bounded(u64::MAX, Resource::OutputBytes)
    }

    fn counting() -> Self {
        Self {
            bytes: Vec::new(),
            written: 0,
            limit: u64::MAX,
            resource: Resource::OutputBytes,
            retain: false,
        }
    }

    fn append(&mut self, bytes: &[u8]) -> Result<()> {
        let next = checked_add(self.written, count(bytes.len())?)?;
        if next > self.limit {
            return Err(Error::exhausted(
                self.resource,
                "canonical encoding exceeds byte ceiling",
            ));
        }
        if self.retain {
            self.bytes.extend_from_slice(bytes);
        }
        self.written = next;
        Ok(())
    }

    fn u8(&mut self, value: u8) -> Result<()> {
        self.append(&[value])
    }

    fn u64(&mut self, value: u64) -> Result<()> {
        self.append(&value.to_be_bytes())
    }

    fn len(&mut self, value: usize) -> Result<()> {
        self.u64(count(value)?)
    }

    fn bytes(&mut self, value: &[u8]) -> Result<()> {
        self.len(value.len())?;
        self.append(value)
    }

    fn string(&mut self, value: &str) -> Result<()> {
        self.bytes(value.as_bytes())
    }

    fn digest(&mut self, value: &[u8; 32]) -> Result<()> {
        self.append(value)
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    const fn written_len(&self) -> u64 {
        self.written
    }
}

fn encode_body(envelope: &ImpactEnvelope, limit: u64) -> Result<Vec<u8>> {
    let mut writer = CanonicalWriter::bounded(limit, Resource::OutputBytes);
    writer.string("quire.compatibility-impact-body/v1")?;
    writer.string(&envelope.baseline_lock.to_hex())?;
    writer.string(&envelope.candidate_lock.to_hex())?;
    encode_limits(&mut writer, envelope.limits)?;
    writer.digest(&envelope.projection_fingerprint)?;
    writer.len(envelope.inputs.len())?;
    for input in &envelope.inputs {
        writer.u8(input.contract.ordinal())?;
        writer.u8(lock_side_code(input.lock))?;
        encode_selection(&mut writer, &input.selection)?;
        writer.bytes(&input.canonical_bytes)?;
        writer.digest(&input.digest)?;
    }
    writer.len(envelope.assessments.len())?;
    for assessment in &envelope.assessments {
        encode_correspondence(&mut writer, assessment)?;
    }
    writer.len(envelope.subjects.len())?;
    for subject in &envelope.subjects {
        encode_subject_record(&mut writer, subject)?;
    }
    writer.len(envelope.freshness.len())?;
    for freshness in &envelope.freshness {
        encode_freshness_record(&mut writer, freshness)?;
    }
    writer.len(envelope.cycles.len())?;
    for cycle in &envelope.cycles {
        encode_cycle(&mut writer, cycle)?;
    }
    writer.len(envelope.unresolved.len())?;
    for unresolved in &envelope.unresolved {
        encode_unresolved(&mut writer, unresolved)?;
    }
    writer.len(envelope.vectors.len())?;
    for vector in &envelope.vectors {
        encode_vector_record(&mut writer, vector)?;
    }
    encode_outcome(&mut writer, &envelope.outcome)?;
    encode_usage(&mut writer, envelope.usage)?;
    Ok(writer.finish())
}

fn encode_envelope(envelope: &ImpactEnvelope, limit: u64) -> Result<Vec<u8>> {
    let mut writer = CanonicalWriter::bounded(limit, Resource::AccountingBytes);
    writer.string("quire.compatibility-impact/v1")?;
    writer.digest(&envelope.identity)?;
    writer.bytes(&envelope.body_bytes)?;
    Ok(writer.finish())
}

fn encode_limits(writer: &mut CanonicalWriter, value: Limits) -> Result<()> {
    for field in [
        value.max_document_bytes,
        value.max_total_input_bytes,
        value.max_identity_bytes,
        value.max_reason_bytes,
        value.max_correspondences,
        value.max_assessments,
        value.max_changed_roots,
        value.max_graph_nodes,
        value.max_graph_edges,
        value.max_evidence_records,
        value.max_evidence_rights,
        value.max_evidence_limitations,
        value.max_reader_invocations,
        value.max_policy_seed_rules,
        value.max_policy_propagation_rules,
        value.max_policy_vector_rules,
        value.max_vector_definitions,
        value.max_manifest_entries,
        value.max_manifest_providers,
        value.max_unresolved_entries,
        value.max_total_reason_bytes,
        value.max_topology_items,
        value.max_traversal_steps,
        value.max_traversed_edges,
        value.max_subject_records,
        value.max_path_edges,
        value.max_witnesses,
        value.max_witness_bytes,
        value.max_frontier_entries,
        value.max_output_bytes,
        value.max_accounting_bytes,
    ] {
        writer.u64(field)?;
    }
    Ok(())
}

fn encode_selection(writer: &mut CanonicalWriter, value: &CapabilitySelection) -> Result<()> {
    writer.string(value.role().as_str())?;
    writer.string(value.provider().as_str())?;
    writer.string(value.build().as_str())?;
    writer.string(value.contract().as_str())?;
    writer.string(value.options().as_str())
}

fn encode_subject(writer: &mut CanonicalWriter, value: &QualifiedSubjectId) -> Result<()> {
    writer.u8(subject_kind_code(value.kind))?;
    writer.string(value.authority.as_str())?;
    writer.string(value.revision.as_str())?;
    writer.string(value.local.as_str())
}

fn encode_optional_subject(
    writer: &mut CanonicalWriter,
    value: Option<&QualifiedSubjectId>,
) -> Result<()> {
    match value {
        Some(value) => {
            writer.u8(1)?;
            encode_subject(writer, value)
        }
        None => writer.u8(0),
    }
}

fn encode_correspondence(writer: &mut CanonicalWriter, value: &Correspondence) -> Result<()> {
    writer.string(value.id.as_str())?;
    encode_optional_subject(writer, value.baseline.as_ref())?;
    encode_optional_subject(writer, value.candidate.as_ref())?;
    writer.u8(structural_code(value.structural))?;
    writer.u8(behavioral_code(value.behavioral))?;
    writer.u8(comparable_code(value.comparable_results))?;
    match &value.comparable_key {
        Some(key) => {
            writer.u8(1)?;
            encode_subject(writer, &key.subject)?;
            writer.string(key.vector.as_str())?;
            writer.string(key.method.as_str())?;
            writer.string(key.provider.as_str())?;
            writer.string(key.environment.as_str())?;
            writer.string(key.assumptions.as_str())?;
            writer.string(key.domain.as_str())?;
            writer.string(&key.baseline_lock.to_hex())?;
            writer.string(&key.candidate_lock.to_hex())?;
        }
        None => writer.u8(0)?,
    }
    writer.u8(presence_code(value.presence))
}

fn encode_oriented_edge(writer: &mut CanonicalWriter, value: &OrientedEdge) -> Result<()> {
    writer.u8(effect_code(value.effect))?;
    encode_subject(writer, &value.source)?;
    writer.string(value.relation.as_str())?;
    encode_subject(writer, &value.target)?;
    writer.string(value.scope.as_str())?;
    writer.string(value.provenance.as_str())?;
    writer.u8(direction_code(value.direction))
}

fn encode_witness(writer: &mut CanonicalWriter, value: &Witness) -> Result<()> {
    encode_subject(writer, &value.root)?;
    writer.u8(effect_code(value.effect))?;
    encode_subject(writer, &value.subject)?;
    writer.len(value.path.len())?;
    for edge in &value.path {
        encode_oriented_edge(writer, edge)?;
    }
    Ok(())
}

fn encode_optional_witness(writer: &mut CanonicalWriter, value: Option<&Witness>) -> Result<()> {
    match value {
        Some(value) => {
            writer.u8(1)?;
            encode_witness(writer, value)
        }
        None => writer.u8(0),
    }
}

fn encode_subject_record(writer: &mut CanonicalWriter, value: &SubjectRecord) -> Result<()> {
    encode_subject(writer, &value.root)?;
    encode_subject(writer, &value.subject)?;
    writer.u8(reachability_code(value.reachability))?;
    encode_optional_witness(writer, value.affectedness_witness.as_ref())?;
    encode_optional_witness(writer, value.staleness_witness.as_ref())
}

fn encode_freshness_record(writer: &mut CanonicalWriter, value: &FreshnessRecord) -> Result<()> {
    encode_subject(writer, &value.root)?;
    writer.string(value.evidence.as_str())?;
    encode_subject(writer, &value.subject)?;
    writer.u8(evidence_relation_code(value.relation))?;
    writer.bytes(&value.owner_bytes)?;
    writer.string(value.scope.as_str())?;
    writer.string(value.provenance.as_str())?;
    writer.u8(freshness_code(value.prior))?;
    writer.u8(freshness_propagation_code(value.propagation))?;
    writer.u8(freshness_code(value.freshness))?;
    encode_optional_witness(writer, value.witness.as_ref())
}

fn encode_cycle(writer: &mut CanonicalWriter, value: &CycleRecord) -> Result<()> {
    encode_subject(writer, &value.root)?;
    writer.u8(effect_code(value.effect))?;
    writer.len(value.component.len())?;
    for subject in &value.component {
        encode_subject(writer, subject)?;
    }
    writer.len(value.cycle.len())?;
    for edge in &value.cycle {
        encode_oriented_edge(writer, edge)?;
    }
    encode_frontier_entry(writer, &value.entry)
}

fn encode_frontier_entry(writer: &mut CanonicalWriter, entry: &FrontierEntry) -> Result<()> {
    encode_subject(writer, &entry.root)?;
    writer.u8(effect_code(entry.effect))?;
    encode_optional_subject(writer, entry.predecessor.as_ref())?;
    encode_subject(writer, &entry.target)?;
    encode_optional_string(writer, entry.relation.as_ref().map(RelationId::as_str))?;
    encode_optional_string(writer, entry.scope.as_ref().map(ScopeId::as_str))?;
    encode_optional_string(writer, entry.provenance.as_ref().map(ProvenanceId::as_str))?;
    writer.u64(entry.depth)
}

fn encode_unresolved(writer: &mut CanonicalWriter, value: &UnresolvedDependency) -> Result<()> {
    encode_subject(writer, &value.root)?;
    encode_subject(writer, &value.predecessor)?;
    encode_subject(writer, &value.target)?;
    writer.string(value.relation.as_str())?;
    writer.string(value.scope.as_str())?;
    writer.string(value.provenance.as_str())?;
    writer.string(&value.reason)?;
    writer.digest(&value.digest)
}

fn encode_vector_key(writer: &mut CanonicalWriter, value: &VectorKey) -> Result<()> {
    writer.string(value.vector.as_str())?;
    encode_subject(writer, &value.subject)?;
    writer.string(value.scope.as_str())?;
    writer.string(value.method.as_str())?;
    writer.string(value.environment.as_str())?;
    writer.string(value.provider_role.as_str())?;
    writer.string(value.profile.as_str())
}

fn encode_vector_record(writer: &mut CanonicalWriter, value: &VectorRecord) -> Result<()> {
    encode_vector_key(writer, &value.key)?;
    writer.bytes(&value.definition_bytes)?;
    writer.u8(vector_requirement_code(value.requirement))?;
    match &value.availability {
        VectorAvailability::Available(provider) => {
            writer.u8(0)?;
            writer.string(provider.as_str())?;
        }
        VectorAvailability::Unsupported(provider) => {
            writer.u8(1)?;
            writer.string(provider.as_str())?;
        }
        VectorAvailability::Unavailable => writer.u8(2)?,
        VectorAvailability::Unknown { provider, reason } => {
            writer.u8(3)?;
            writer.string(provider.as_str())?;
            writer.string(reason)?;
        }
    }
    writer.len(value.contributing_rules.len())?;
    for rule in &value.contributing_rules {
        writer.string(rule.as_str())?;
    }
    Ok(())
}

fn encode_outcome(writer: &mut CanonicalWriter, value: &TraversalOutcome) -> Result<()> {
    match value {
        TraversalOutcome::Complete => writer.u8(0),
        TraversalOutcome::Incomplete { cause, frontier } => {
            writer.u8(1)?;
            writer.u8(resource_code(*cause))?;
            writer.len(frontier.len())?;
            for entry in frontier {
                encode_frontier_entry(writer, entry)?;
            }
            Ok(())
        }
    }
}

fn encode_optional_string(writer: &mut CanonicalWriter, value: Option<&str>) -> Result<()> {
    match value {
        Some(value) => {
            writer.u8(1)?;
            writer.string(value)
        }
        None => writer.u8(0),
    }
}

fn encode_usage(writer: &mut CanonicalWriter, value: Usage) -> Result<()> {
    for field in [
        value.input_bytes,
        value.reader_invocations,
        value.topology_items,
        value.traversal_steps,
        value.traversed_edges,
        value.subject_records,
        value.path_edges,
        value.witnesses,
        value.witness_bytes,
        value.frontier_entries,
    ] {
        writer.u64(field)?;
    }
    Ok(())
}

fn encode_seed_rule(writer: &mut CanonicalWriter, value: &SeedRule) -> Result<()> {
    writer.string(value.id.as_str())?;
    writer.u8(structural_code(value.assessment.structural))?;
    writer.u8(behavioral_code(value.assessment.behavioral))?;
    writer.u8(presence_code(value.assessment.presence))?;
    writer.len(value.effects.len())?;
    for effect in &value.effects {
        writer.u8(effect_code(*effect))?;
    }
    Ok(())
}

fn encode_propagation_rule(writer: &mut CanonicalWriter, value: &PropagationRule) -> Result<()> {
    writer.string(value.id.as_str())?;
    writer.string(value.relation.as_str())?;
    writer.u8(subject_kind_code(value.source_kind))?;
    writer.u8(subject_kind_code(value.target_kind))?;
    writer.u8(direction_code(value.direction))?;
    writer.string(value.scope.as_str())?;
    writer.len(value.effects.len())?;
    for effect in &value.effects {
        writer.u8(effect_code(*effect))?;
    }
    Ok(())
}

fn encode_vector_rule(writer: &mut CanonicalWriter, value: &VectorRule) -> Result<()> {
    writer.string(value.id.as_str())?;
    encode_vector_key(writer, &value.key)?;
    writer.u8(vector_requirement_code(value.requirement))
}

const fn subject_kind_code(value: SubjectKind) -> u8 {
    match value {
        SubjectKind::Requirement => 0,
        SubjectKind::Assumption => 1,
        SubjectKind::Model => 2,
        SubjectKind::Implementation => 3,
        SubjectKind::Evidence => 4,
        SubjectKind::Provider => 5,
    }
}

const fn structural_code(value: StructuralAssessment) -> u8 {
    match value {
        StructuralAssessment::Unchanged => 0,
        StructuralAssessment::AdditiveCompatible => 1,
        StructuralAssessment::PayloadBreaking => 2,
        StructuralAssessment::Removed => 3,
        StructuralAssessment::Unknown => 4,
    }
}

const fn behavioral_code(value: BehavioralAssessment) -> u8 {
    match value {
        BehavioralAssessment::NotAssessed => 0,
        BehavioralAssessment::Conforming => 1,
        BehavioralAssessment::Regressed => 2,
        BehavioralAssessment::Absent => 3,
        BehavioralAssessment::Unsupported => 4,
        BehavioralAssessment::Unavailable => 5,
        BehavioralAssessment::Unknown => 6,
    }
}

const fn comparable_code(value: ComparableResults) -> u8 {
    match value {
        ComparableResults::Both => 0,
        ComparableResults::CandidateMissing => 1,
        ComparableResults::NotApplicable => 2,
    }
}

const fn presence_code(value: SubjectPresence) -> u8 {
    match value {
        SubjectPresence::BothPresent => 0,
        SubjectPresence::BaselineOnly => 1,
        SubjectPresence::CandidateOnly => 2,
        SubjectPresence::Unknown => 3,
    }
}

const fn effect_code(value: Effect) -> u8 {
    match value {
        Effect::Affectedness => 0,
        Effect::Staleness => 1,
    }
}

const fn direction_code(value: Direction) -> u8 {
    match value {
        Direction::Forward => 0,
        Direction::Reverse => 1,
    }
}

const fn reachability_code(value: Reachability) -> u8 {
    match value {
        Reachability::Affected => 0,
        Reachability::Unaffected => 1,
        Reachability::Unknown => 2,
    }
}

const fn evidence_relation_code(value: EvidenceRelation) -> u8 {
    match value {
        EvidenceRelation::Supports => 0,
        EvidenceRelation::Contradicts => 1,
    }
}

const fn freshness_code(value: Freshness) -> u8 {
    match value {
        Freshness::Current => 0,
        Freshness::Stale => 1,
        Freshness::Unknown => 2,
    }
}

const fn freshness_propagation_code(value: FreshnessPropagation) -> u8 {
    match value {
        FreshnessPropagation::Reached => 0,
        FreshnessPropagation::NotReached => 1,
        FreshnessPropagation::Unknown => 2,
        FreshnessPropagation::Incomplete => 3,
    }
}

const fn vector_requirement_code(value: VectorRequirement) -> u8 {
    match value {
        VectorRequirement::Required => 0,
        VectorRequirement::NotApplicable => 1,
    }
}

const fn lock_side_code(value: LockSide) -> u8 {
    match value {
        LockSide::Baseline => 0,
        LockSide::Candidate => 1,
    }
}

const fn resource_code(value: Resource) -> u8 {
    match value {
        Resource::DocumentBytes => 0,
        Resource::TotalInputBytes => 1,
        Resource::IdentityBytes => 2,
        Resource::ReasonBytes => 3,
        Resource::Correspondences => 4,
        Resource::Assessments => 5,
        Resource::ChangedRoots => 6,
        Resource::GraphNodes => 7,
        Resource::GraphEdges => 8,
        Resource::EvidenceRecords => 9,
        Resource::EvidenceRights => 10,
        Resource::EvidenceLimitations => 11,
        Resource::ReaderInvocations => 12,
        Resource::PolicySeedRules => 13,
        Resource::PolicyPropagationRules => 14,
        Resource::VectorDefinitions => 15,
        Resource::ManifestEntries => 16,
        Resource::UnresolvedEntries => 17,
        Resource::TotalReasonBytes => 18,
        Resource::TopologyItems => 19,
        Resource::TraversalSteps => 20,
        Resource::TraversedEdges => 21,
        Resource::SubjectRecords => 22,
        Resource::PathEdges => 23,
        Resource::Witnesses => 24,
        Resource::WitnessBytes => 25,
        Resource::FrontierEntries => 26,
        Resource::OutputBytes => 27,
        Resource::AccountingBytes => 28,
        Resource::Duration => 29,
        Resource::PolicyVectorRules => 30,
        Resource::ManifestProviders => 31,
    }
}
