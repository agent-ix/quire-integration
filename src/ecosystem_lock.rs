// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Exact, canonical, constructor-private ecosystem locks.
//!
//! Validation is deliberately separate from producer capabilities: callers
//! cannot obtain a [`ValidatedLock`] until every selected pin, mapping,
//! dependency contract, compatibility statement, and resource bound succeeds.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;

use serde::{ser::SerializeSeq as _, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{
    ArtifactId, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, PackageId,
    ProviderId, RepositoryId, RoleId, RunId, ToolchainId, WorkflowInstanceId,
};

/// Canonical wire contract.
pub const CONTRACT: &str = "quire.ecosystem-lock/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../schemas/quire.ecosystem-lock-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "f71de2810dba86c245eb2b892d3eaeb45d39ffee93da39b0826ab6cce99e0288";

/// Stable lock-validation refusal cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalCode {
    /// A required selection is absent.
    MissingSelection,
    /// The same identity occurs more than once in one domain.
    DuplicateSelection,
    /// A selected revision is stale.
    StaleSelection,
    /// A selected repository contains uncommitted state.
    DirtySelection,
    /// A selected revision is moving or ambient.
    MovingSelection,
    /// A selected revision or compatibility result is unknown.
    UnknownSelection,
    /// A mapping points outside its declared identity domain.
    CrossDomainReference,
    /// Reachability did not establish compatibility.
    UnknownCompatibility,
    /// An exact compatibility contract is known to be incompatible.
    IncompatibleContract,
    /// A change affects the selected contract but is not yet qualified.
    AffectedSelection,
    /// Input or output is malformed.
    Malformed,
    /// A caller-lowered resource ceiling was exceeded.
    ResourceExhausted,
    /// Supplied bytes do not match the independently selected lock.
    CanonicalMismatch,
    /// Checked integer arithmetic failed.
    IntegerOverflow,
    /// Dependency reachability is explicitly unresolved.
    UnresolvedDependency,
    /// Dependency topology contains or declares a cycle.
    CyclicDependency,
    /// The selected target is proven unreachable.
    UnreachableDependency,
}

impl RefusalCode {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingSelection => "QINT-LOCK-MISSING-SELECTION",
            Self::DuplicateSelection => "QINT-LOCK-DUPLICATE-SELECTION",
            Self::StaleSelection => "QINT-LOCK-STALE-SELECTION",
            Self::DirtySelection => "QINT-LOCK-DIRTY-SELECTION",
            Self::MovingSelection => "QINT-LOCK-MOVING-SELECTION",
            Self::UnknownSelection => "QINT-LOCK-UNKNOWN-SELECTION",
            Self::CrossDomainReference => "QINT-LOCK-CROSS-DOMAIN-REFERENCE",
            Self::UnknownCompatibility => "QINT-LOCK-UNKNOWN-COMPATIBILITY",
            Self::IncompatibleContract => "QINT-LOCK-INCOMPATIBLE-CONTRACT",
            Self::AffectedSelection => "QINT-LOCK-AFFECTED-SELECTION",
            Self::Malformed => "QINT-LOCK-MALFORMED",
            Self::ResourceExhausted => "QINT-LOCK-RESOURCE-EXHAUSTED",
            Self::CanonicalMismatch => "QINT-LOCK-CANONICAL-MISMATCH",
            Self::IntegerOverflow => "QINT-LOCK-INTEGER-OVERFLOW",
            Self::UnresolvedDependency => "QINT-LOCK-UNRESOLVED-DEPENDENCY",
            Self::CyclicDependency => "QINT-LOCK-CYCLIC-DEPENDENCY",
            Self::UnreachableDependency => "QINT-LOCK-UNREACHABLE-DEPENDENCY",
        }
    }
}

/// Identity domain associated with a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionDomain {
    /// Source repository.
    Repository,
    /// Rust or native package.
    Package,
    /// Semantic definition.
    Definition,
    /// Runtime provider.
    Provider,
    /// Compiler/toolchain.
    Toolchain,
    /// Immutable build.
    Build,
    /// Deployment selection.
    Deployment,
    /// Semantic component.
    Component,
    /// Runtime role.
    Role,
    /// Exact producer-capability binding.
    Capability,
    /// Workflow instance.
    WorkflowInstance,
    /// Explicit dependency edge.
    Dependency,
    /// Complete lock document.
    Lock,
}

/// Typed fail-closed ecosystem-lock refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    code: RefusalCode,
    domain: Option<SelectionDomain>,
    selection: Option<String>,
    detail: &'static str,
}

impl Refusal {
    fn new(code: RefusalCode, domain: Option<SelectionDomain>, detail: &'static str) -> Self {
        Self {
            code,
            domain,
            selection: None,
            detail,
        }
    }

    fn selection(
        code: RefusalCode,
        domain: SelectionDomain,
        selection: impl Into<String>,
        detail: &'static str,
    ) -> Self {
        Self {
            code,
            domain: Some(domain),
            selection: Some(selection.into()),
            detail,
        }
    }

    /// Stable refusal cause.
    #[must_use]
    pub const fn code(&self) -> RefusalCode {
        self.code
    }

    /// Exact identity domain, when one caused the refusal.
    #[must_use]
    pub const fn domain(&self) -> Option<SelectionDomain> {
        self.domain
    }

    /// Exact offending selection identity, when a single selection caused the refusal.
    #[must_use]
    pub fn selection_identity(&self) -> Option<&str> {
        self.selection.as_deref()
    }

    /// Bounded non-authoritative diagnostic.
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for Refusal {}

/// Ecosystem-lock result.
pub type Result<T> = std::result::Result<T, Refusal>;

/// Prevalidation state of one exact selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinState {
    /// One immutable revision was selected.
    Exact,
    /// A required selection is absent.
    Missing,
    /// The recorded revision is stale.
    Stale,
    /// The repository is dirty.
    Dirty,
    /// The selection names a branch, tag, default, or other moving target.
    Moving,
    /// Resolution could not establish the exact revision.
    Unknown,
}

/// One identity and immutable revision before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pin<T> {
    identity: T,
    revision: String,
    state: PinState,
}

mod pin_identity_sealed {
    pub trait Sealed {}
}

/// Sealed identity domain accepted by [`Pin`].
pub trait PinIdentity: pin_identity_sealed::Sealed {
    #[doc(hidden)]
    const DOMAIN: SelectionDomain;

    #[doc(hidden)]
    fn text(&self) -> &str;
}

macro_rules! identity_text {
    ($($identity:ty => $domain:ident),+ $(,)?) => {$ (
        impl pin_identity_sealed::Sealed for $identity {}

        impl PinIdentity for $identity {
            const DOMAIN: SelectionDomain = SelectionDomain::$domain;

            fn text(&self) -> &str { self.as_str() }
        }
    )+ };
}

identity_text!(
    RepositoryId => Repository,
    PackageId => Package,
    DefinitionId => Definition,
    ProviderId => Provider,
    ToolchainId => Toolchain,
    BuildId => Build,
    DeploymentId => Deployment,
    ComponentId => Component,
    RoleId => Role,
    WorkflowInstanceId => WorkflowInstance,
);

impl<T: PinIdentity> Pin<T> {
    /// Constructs a candidate pin after bounding its revision before retention.
    pub fn new(
        identity: T,
        revision: &str,
        state: PinState,
        max_revision_bytes: u64,
    ) -> Result<Self> {
        validate_revision_candidate(&identity, revision, state, max_revision_bytes)?;
        Ok(Self {
            identity,
            revision: revision.to_owned(),
            state,
        })
    }

    /// Mutates the candidate revision after bounding it before retention.
    pub fn set_revision(&mut self, revision: &str, max_revision_bytes: u64) -> Result<()> {
        validate_revision_candidate(&self.identity, revision, self.state, max_revision_bytes)?;
        self.revision.clear();
        self.revision.push_str(revision);
        Ok(())
    }

    /// Mutates the candidate state before validation.
    pub fn set_state(&mut self, state: PinState) -> Result<()> {
        validate_revision_candidate(
            &self.identity,
            &self.revision,
            state,
            count(self.revision.len())?,
        )?;
        self.state = state;
        Ok(())
    }
}

fn validate_revision_candidate<T: PinIdentity>(
    identity: &T,
    revision: &str,
    state: PinState,
    maximum: u64,
) -> Result<()> {
    if count(revision.len())? > maximum {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            T::DOMAIN,
            identity.text(),
            "revision exceeds its caller-lowered byte ceiling",
        ));
    }
    if state == PinState::Exact && !is_content_addressed_revision(revision) {
        return Err(Refusal::selection(
            RefusalCode::MovingSelection,
            T::DOMAIN,
            identity.text(),
            "exact pin is not a canonical content-addressed revision",
        ));
    }
    Ok(())
}

fn is_content_addressed_revision(revision: &str) -> bool {
    revision
        .strip_prefix("git:")
        .is_some_and(|digest| digest.len() == 40 && digest.bytes().all(is_lower_hex))
        || revision
            .strip_prefix("sha256:")
            .is_some_and(|digest| digest.len() == 64 && digest.bytes().all(is_lower_hex))
}

const fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

/// Repository selection plus its exact remote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryBinding {
    pin: Pin<RepositoryId>,
    remote: String,
}

impl RepositoryBinding {
    /// Constructs a candidate repository binding after bounding its remote before retention.
    pub fn new(pin: Pin<RepositoryId>, remote: &str, max_field_bytes: u64) -> Result<Self> {
        let remote = retain_bounded_field(
            remote,
            max_field_bytes,
            SelectionDomain::Repository,
            pin.identity.as_str(),
        )?;
        Ok(Self { pin, remote })
    }

    /// Mutable candidate pin, available only before validation.
    pub fn pin_mut(&mut self) -> &mut Pin<RepositoryId> {
        &mut self.pin
    }

    /// Changes the exact repository remote candidate.
    pub fn set_remote(&mut self, remote: &str, max_field_bytes: u64) -> Result<()> {
        self.remote = retain_bounded_field(
            remote,
            max_field_bytes,
            SelectionDomain::Repository,
            self.pin.identity.as_str(),
        )?;
        Ok(())
    }
}

/// Package selection and its exact repository/features.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageBinding {
    pin: Pin<PackageId>,
    version: String,
    repository: RepositoryId,
    repository_revision: String,
    features: Vec<String>,
}

impl PackageBinding {
    /// Constructs a candidate package binding.
    pub fn new(
        pin: Pin<PackageId>,
        version: &str,
        repository: RepositoryId,
        repository_revision: &str,
        features: &[&str],
        max_features: u64,
        max_field_bytes: u64,
    ) -> Result<Self> {
        let selection = pin.identity.as_str();
        let version = retain_bounded_field(
            version,
            max_field_bytes,
            SelectionDomain::Package,
            selection,
        )?;
        if semver::Version::parse(&version).is_err() {
            return Err(Refusal::selection(
                RefusalCode::MovingSelection,
                SelectionDomain::Package,
                selection,
                "package version is not one exact semantic version",
            ));
        }
        let repository_revision = retain_bounded_field(
            repository_revision,
            max_field_bytes,
            SelectionDomain::Package,
            selection,
        )?;
        let features = retain_feature_set(features, max_features, max_field_bytes, selection)?;
        Ok(Self {
            pin,
            version,
            repository,
            repository_revision,
            features,
        })
    }

    /// Mutable candidate pin, available only before validation.
    pub fn pin_mut(&mut self) -> &mut Pin<PackageId> {
        &mut self.pin
    }

    /// Changes the exact repository revision claimed by this package candidate.
    pub fn set_repository_revision(&mut self, revision: &str, max_field_bytes: u64) -> Result<()> {
        self.repository_revision = retain_bounded_field(
            revision,
            max_field_bytes,
            SelectionDomain::Package,
            self.pin.identity.as_str(),
        )?;
        Ok(())
    }

    /// Changes the exact semantic package version candidate.
    pub fn set_version(&mut self, version: &str, max_field_bytes: u64) -> Result<()> {
        let version = retain_bounded_field(
            version,
            max_field_bytes,
            SelectionDomain::Package,
            self.pin.identity.as_str(),
        )?;
        if semver::Version::parse(&version).is_err() {
            return Err(Refusal::selection(
                RefusalCode::MovingSelection,
                SelectionDomain::Package,
                self.pin.identity.as_str(),
                "package version is not one exact semantic version",
            ));
        }
        self.version = version;
        Ok(())
    }

    /// Changes the bounded canonical feature set candidate.
    pub fn set_features(
        &mut self,
        features: &[&str],
        max_features: u64,
        max_field_bytes: u64,
    ) -> Result<()> {
        self.features = retain_feature_set(
            features,
            max_features,
            max_field_bytes,
            self.pin.identity.as_str(),
        )?;
        Ok(())
    }
}

/// Component-to-repository/package mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentBinding {
    pin: Pin<ComponentId>,
    repository: RepositoryId,
    package: PackageId,
}

impl ComponentBinding {
    /// Constructs a candidate component binding.
    #[must_use]
    pub fn new(pin: Pin<ComponentId>, repository: RepositoryId, package: PackageId) -> Self {
        Self {
            pin,
            repository,
            package,
        }
    }

    /// Mutable candidate pin, available only before validation.
    pub fn pin_mut(&mut self) -> &mut Pin<ComponentId> {
        &mut self.pin
    }
}

/// Role-to-component mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleBinding {
    pin: Pin<RoleId>,
    component: ComponentId,
}

impl RoleBinding {
    /// Constructs a candidate role binding.
    #[must_use]
    pub fn new(pin: Pin<RoleId>, component: ComponentId) -> Self {
        Self { pin, component }
    }

    /// Mutable component reference, available only before validation.
    pub fn component_mut(&mut self) -> &mut ComponentId {
        &mut self.component
    }

    /// Mutable candidate pin, available only before validation.
    pub fn pin_mut(&mut self) -> &mut Pin<RoleId> {
        &mut self.pin
    }
}

/// Exact capability selection used to request admission under a validated lock.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilitySelection {
    role: RoleId,
    provider: ProviderId,
    build: BuildId,
    contract: ContractId,
    options: ArtifactId,
}

impl CapabilitySelection {
    /// Constructs an exact selection from already bounded, domain-safe identities.
    #[must_use]
    pub fn new(
        role: RoleId,
        provider: ProviderId,
        build: BuildId,
        contract: ContractId,
        options: ArtifactId,
    ) -> Self {
        Self {
            role,
            provider,
            build,
            contract,
            options,
        }
    }

    /// Selected role.
    #[must_use]
    pub const fn role(&self) -> &RoleId {
        &self.role
    }

    /// Selected provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderId {
        &self.provider
    }

    /// Selected immutable build.
    #[must_use]
    pub const fn build(&self) -> &BuildId {
        &self.build
    }

    /// Selected capability contract.
    #[must_use]
    pub const fn contract(&self) -> &ContractId {
        &self.contract
    }

    /// Selected exact options artifact.
    #[must_use]
    pub const fn options(&self) -> &ArtifactId {
        &self.options
    }
}

/// One lock-owned role/provider/build/contract/options binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityBinding {
    selection: CapabilitySelection,
}

impl CapabilityBinding {
    /// Constructs a candidate binding from already bounded, domain-safe identities.
    #[must_use]
    pub fn new(selection: CapabilitySelection) -> Self {
        Self { selection }
    }

    /// Exact candidate selection.
    #[must_use]
    pub const fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    /// Mutable candidate selection, available only before lock validation.
    pub fn selection_mut(&mut self) -> &mut CapabilitySelection {
        &mut self.selection
    }
}

/// Workflow-instance-to-definition mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowBinding {
    pin: Pin<WorkflowInstanceId>,
    definition: DefinitionId,
}

impl WorkflowBinding {
    /// Constructs a candidate workflow binding.
    #[must_use]
    pub fn new(pin: Pin<WorkflowInstanceId>, definition: DefinitionId) -> Self {
        Self { pin, definition }
    }

    /// Mutable candidate pin, available only before validation.
    pub fn pin_mut(&mut self) -> &mut Pin<WorkflowInstanceId> {
        &mut self.pin
    }
}

/// Graph reachability observation, which is not a compatibility decision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reachability {
    /// Target is reachable through the declared dependency graph.
    Reachable,
    /// Target is not reachable through the declared graph.
    Unreachable,
    /// Graph resolution could not determine reachability.
    Unresolved,
    /// Resolution encountered a dependency cycle.
    Cyclic,
}

/// Independent compatibility classification for one exact contract.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Compatibility {
    /// The exact vector demonstrated compatibility.
    Compatible,
    /// The exact vector demonstrated incompatibility.
    Incompatible,
    /// No decisive vector exists.
    Unknown,
    /// The selected change affects the contract and needs qualification.
    Affected,
    /// The retained compatibility evidence is stale.
    Stale,
}

/// One exact dependency edge and compatibility contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    source: PackageId,
    target: PackageId,
    contract: String,
    reachability: Reachability,
    compatibility: Compatibility,
}

impl DependencyEdge {
    /// Constructs a candidate dependency edge after bounding its contract before retention.
    pub fn new(
        source: PackageId,
        target: PackageId,
        contract: &str,
        reachability: Reachability,
        compatibility: Compatibility,
        max_field_bytes: u64,
    ) -> Result<Self> {
        let contract = retain_bounded_field(
            contract,
            max_field_bytes,
            SelectionDomain::Dependency,
            source.as_str(),
        )?;
        Ok(Self {
            source,
            target,
            contract,
            reachability,
            compatibility,
        })
    }

    /// Changes only the graph-reachability observation.
    pub fn set_reachability(&mut self, reachability: Reachability) {
        self.reachability = reachability;
    }

    /// Changes only the independent compatibility classification.
    pub fn set_compatibility(&mut self, compatibility: Compatibility) {
        self.compatibility = compatibility;
    }

    /// Changes the exact compatibility-contract candidate.
    pub fn set_contract(&mut self, contract: &str, max_field_bytes: u64) -> Result<()> {
        self.contract = retain_bounded_field(
            contract,
            max_field_bytes,
            SelectionDomain::Dependency,
            self.source.as_str(),
        )?;
        Ok(())
    }
}

fn retain_bounded_field(
    value: &str,
    maximum: u64,
    domain: SelectionDomain,
    selection: &str,
) -> Result<String> {
    if count(value.len())? > maximum {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            domain,
            selection,
            "lock field exceeds its caller-lowered byte ceiling before retention",
        ));
    }
    if value.trim().is_empty() {
        return Err(Refusal::selection(
            RefusalCode::Malformed,
            domain,
            selection,
            "lock field is empty",
        ));
    }
    Ok(value.to_owned())
}

fn retain_feature_set(
    features: &[&str],
    maximum: u64,
    max_field_bytes: u64,
    selection: &str,
) -> Result<Vec<String>> {
    if count(features.len())? > maximum {
        let index = usize::try_from(maximum).expect("over-limit feature index is representable");
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            SelectionDomain::Package,
            features[index],
            "package feature set exceeds its caller-lowered item ceiling",
        ));
    }
    for feature in features {
        if count(feature.len())? > max_field_bytes {
            return Err(Refusal::selection(
                RefusalCode::ResourceExhausted,
                SelectionDomain::Package,
                selection,
                "package feature exceeds its caller-lowered byte ceiling before retention",
            ));
        }
        if feature.trim().is_empty() {
            return Err(Refusal::selection(
                RefusalCode::Malformed,
                SelectionDomain::Package,
                selection,
                "package feature is empty",
            ));
        }
    }
    let mut retained = features
        .iter()
        .map(|feature| (*feature).to_owned())
        .collect::<Vec<_>>();
    retained.sort();
    if let Some(duplicate) = retained.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(Refusal::selection(
            RefusalCode::DuplicateSelection,
            SelectionDomain::Package,
            &duplicate[0],
            "package feature set contains a duplicate",
        ));
    }
    Ok(retained)
}

/// Complete caller-supplied lock selection before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockSelection {
    repositories: Vec<RepositoryBinding>,
    packages: Vec<PackageBinding>,
    definitions: Vec<Pin<DefinitionId>>,
    providers: Vec<Pin<ProviderId>>,
    toolchains: Vec<Pin<ToolchainId>>,
    builds: Vec<Pin<BuildId>>,
    deployments: Vec<Pin<DeploymentId>>,
    components: Vec<ComponentBinding>,
    roles: Vec<RoleBinding>,
    capabilities: Vec<CapabilityBinding>,
    workflows: Vec<WorkflowBinding>,
    dependencies: Vec<DependencyEdge>,
}

impl LockSelection {
    /// Constructs an unvalidated complete selection without ambient discovery.
    #[expect(
        clippy::too_many_arguments,
        reason = "the constructor keeps all independently typed lock domains explicit"
    )]
    #[must_use]
    pub fn new(
        repositories: Vec<RepositoryBinding>,
        packages: Vec<PackageBinding>,
        definitions: Vec<Pin<DefinitionId>>,
        providers: Vec<Pin<ProviderId>>,
        toolchains: Vec<Pin<ToolchainId>>,
        builds: Vec<Pin<BuildId>>,
        deployments: Vec<Pin<DeploymentId>>,
        components: Vec<ComponentBinding>,
        roles: Vec<RoleBinding>,
        capabilities: Vec<CapabilityBinding>,
        workflows: Vec<WorkflowBinding>,
        dependencies: Vec<DependencyEdge>,
    ) -> Self {
        Self {
            repositories,
            packages,
            definitions,
            providers,
            toolchains,
            builds,
            deployments,
            components,
            roles,
            capabilities,
            workflows,
            dependencies,
        }
    }

    /// Mutable repository candidates.
    pub fn repositories_mut(&mut self) -> &mut [RepositoryBinding] {
        &mut self.repositories
    }

    /// Mutable package candidates.
    pub fn packages_mut(&mut self) -> &mut [PackageBinding] {
        &mut self.packages
    }

    /// Mutable definition candidates.
    pub fn definitions_mut(&mut self) -> &mut [Pin<DefinitionId>] {
        &mut self.definitions
    }

    /// Mutable provider candidates.
    pub fn providers_mut(&mut self) -> &mut [Pin<ProviderId>] {
        &mut self.providers
    }

    /// Mutable toolchain candidates.
    pub fn toolchains_mut(&mut self) -> &mut [Pin<ToolchainId>] {
        &mut self.toolchains
    }

    /// Mutable build candidates.
    pub fn builds_mut(&mut self) -> &mut [Pin<BuildId>] {
        &mut self.builds
    }

    /// Mutable deployment candidates.
    pub fn deployments_mut(&mut self) -> &mut [Pin<DeploymentId>] {
        &mut self.deployments
    }

    /// Mutable component candidates.
    pub fn components_mut(&mut self) -> &mut [ComponentBinding] {
        &mut self.components
    }

    /// Mutable workflow-instance candidates.
    pub fn workflows_mut(&mut self) -> &mut [WorkflowBinding] {
        &mut self.workflows
    }

    /// Role candidates.
    #[must_use]
    pub fn roles(&self) -> &[RoleBinding] {
        &self.roles
    }

    /// Mutable role candidates.
    pub fn roles_mut(&mut self) -> &mut Vec<RoleBinding> {
        &mut self.roles
    }

    /// Capability-binding candidates.
    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityBinding] {
        &self.capabilities
    }

    /// Mutable capability-binding candidates.
    pub fn capabilities_mut(&mut self) -> &mut Vec<CapabilityBinding> {
        &mut self.capabilities
    }

    /// Mutable dependency candidates.
    pub fn dependencies_mut(&mut self) -> &mut Vec<DependencyEdge> {
        &mut self.dependencies
    }
}

/// Caller-lowered validation and encoding ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Repository selections.
    pub max_repositories: u64,
    /// Package selections.
    pub max_packages: u64,
    /// Definition selections.
    pub max_definitions: u64,
    /// Provider selections.
    pub max_providers: u64,
    /// Toolchain selections.
    pub max_toolchains: u64,
    /// Build selections.
    pub max_builds: u64,
    /// Deployment selections.
    pub max_deployments: u64,
    /// Component mappings.
    pub max_components: u64,
    /// Role mappings.
    pub max_roles: u64,
    /// Exact producer-capability bindings.
    pub max_capabilities: u64,
    /// Workflow-instance mappings.
    pub max_workflows: u64,
    /// Dependency edges.
    pub max_dependencies: u64,
    /// Dependency-validation and topology traversal steps.
    pub max_traversal_steps: u64,
    /// Total input string bytes visited.
    pub max_input_bytes: u64,
    /// Maximum strict-reader document bytes.
    pub max_document_bytes: u64,
    /// Per-string identity/revision/contract bytes.
    pub max_identity_bytes: u64,
    /// Canonical output bytes.
    pub max_output_bytes: u64,
}

impl Limits {
    /// Derives exact collection/input bounds for one selection and explicit output ceiling.
    pub fn for_selection(
        selection: &LockSelection,
        max_output_bytes: u64,
        max_identity_bytes: u64,
    ) -> Result<Self> {
        Ok(Self {
            max_repositories: count(selection.repositories.len())?,
            max_packages: count(selection.packages.len())?,
            max_definitions: count(selection.definitions.len())?,
            max_providers: count(selection.providers.len())?,
            max_toolchains: count(selection.toolchains.len())?,
            max_builds: count(selection.builds.len())?,
            max_deployments: count(selection.deployments.len())?,
            max_components: count(selection.components.len())?,
            max_roles: count(selection.roles.len())?,
            max_capabilities: count(selection.capabilities.len())?,
            max_workflows: count(selection.workflows.len())?,
            max_dependencies: count(selection.dependencies.len())?,
            max_traversal_steps: dependency_steps(selection)?,
            max_input_bytes: selection_input_bytes(selection)?,
            max_document_bytes: max_output_bytes,
            max_identity_bytes,
            max_output_bytes,
        })
    }
}

/// Content-derived SHA-256 lock identity.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LockIdentity([u8; 32]);

impl LockIdentity {
    /// Lowercase hexadecimal digest.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

/// Content-derived identity for one requested run under one validated lock.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RunIdentity([u8; 32]);

impl RunIdentity {
    /// Lowercase hexadecimal digest.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl fmt::Debug for RunIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RunIdentity")
            .field(&self.to_hex())
            .finish()
    }
}

impl fmt::Debug for LockIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("LockIdentity")
            .field(&self.to_hex())
            .finish()
    }
}

/// Constructor-private validated lock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLock {
    identity: LockIdentity,
    bytes: Vec<u8>,
    wire: LockWire,
    limits: Limits,
}

/// Constructor-private proof that one exact capability selection is admitted by a lock.
#[derive(Clone, Copy, Debug)]
pub struct AdmittedCapability<'a> {
    lock_identity: LockIdentity,
    binding: &'a CapabilityWire,
    component: &'a str,
    package: &'a str,
    features: &'a [String],
}

impl AdmittedCapability<'_> {
    /// Lock identity under which this capability is admitted.
    #[must_use]
    pub const fn lock_identity(&self) -> LockIdentity {
        self.lock_identity
    }

    /// Admitted role identity.
    #[must_use]
    pub fn role(&self) -> &str {
        &self.binding.role
    }

    /// Admitted provider identity.
    #[must_use]
    pub fn provider(&self) -> &str {
        &self.binding.provider
    }

    /// Admitted immutable build identity.
    #[must_use]
    pub fn build(&self) -> &str {
        &self.binding.build
    }

    /// Admitted contract identity.
    #[must_use]
    pub fn contract(&self) -> &str {
        &self.binding.contract
    }

    /// Admitted options-artifact identity.
    #[must_use]
    pub fn options(&self) -> &str {
        &self.binding.options
    }

    /// Component resolved through the admitted role binding.
    #[must_use]
    pub const fn component(&self) -> &str {
        self.component
    }

    /// Owning package resolved through the admitted component binding.
    #[must_use]
    pub const fn package(&self) -> &str {
        self.package
    }

    /// Exact canonical package features admitted by the lock.
    #[must_use]
    pub const fn features(&self) -> &[String] {
        self.features
    }
}

impl ValidatedLock {
    /// Content-derived identity.
    #[must_use]
    pub const fn identity(&self) -> LockIdentity {
        self.identity
    }

    /// Canonical complete JSON bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Wire contract label.
    #[must_use]
    pub const fn contract(&self) -> &'static str {
        CONTRACT
    }

    /// Pinned schema digest.
    #[must_use]
    pub const fn schema_sha256(&self) -> &'static str {
        SCHEMA_SHA256
    }

    /// Exact limits bound into this lock's canonical bytes and identity.
    #[must_use]
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// Derives one run identity from this exact lock and a domain-separated run request.
    pub fn derive_run_identity(&self, run: &RunId) -> Result<RunIdentity> {
        let length = u64::try_from(run.as_str().len()).map_err(|_| {
            Refusal::selection(
                RefusalCode::IntegerOverflow,
                SelectionDomain::Lock,
                run.as_str(),
                "run identity length is not representable",
            )
        })?;
        let mut hasher = Sha256::new();
        hasher.update(b"quire.composed-run/v1\0");
        hasher.update(self.identity.0);
        hasher.update(length.to_be_bytes());
        hasher.update(run.as_str().as_bytes());
        Ok(RunIdentity(hasher.finalize().into()))
    }

    /// Canonical repository selections.
    pub fn repositories(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.wire
            .repositories
            .iter()
            .map(|pin| (pin.identity.as_str(), pin.revision.as_str()))
    }

    /// Canonical component selections.
    pub fn components(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.wire
            .components
            .iter()
            .map(|binding| (binding.identity.as_str(), binding.revision.as_str()))
    }

    /// Canonical role selections.
    pub fn roles(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.wire
            .roles
            .iter()
            .map(|binding| (binding.identity.as_str(), binding.revision.as_str()))
    }

    /// Admits only an exact role/provider/build/contract/options selection present in this lock.
    pub fn admit_capability(
        &self,
        selection: &CapabilitySelection,
    ) -> Result<AdmittedCapability<'_>> {
        let binding = self
            .wire
            .capabilities
            .iter()
            .find(|binding| {
                binding.role == selection.role.as_str()
                    && binding.provider == selection.provider.as_str()
                    && binding.build == selection.build.as_str()
                    && binding.contract == selection.contract.as_str()
                    && binding.options == selection.options.as_str()
            })
            .ok_or_else(|| {
                Refusal::selection(
                    RefusalCode::UnknownSelection,
                    SelectionDomain::Capability,
                    capability_label(selection),
                    "exact capability selection is not admitted by this lock",
                )
            })?;
        let role = self
            .wire
            .roles
            .iter()
            .find(|role| role.identity == binding.role)
            .ok_or_else(|| internal_mapping_refusal(&binding.role))?;
        let component = self
            .wire
            .components
            .iter()
            .find(|component| component.identity == role.component)
            .ok_or_else(|| internal_mapping_refusal(&role.component))?;
        let package = self
            .wire
            .packages
            .iter()
            .find(|package| package.identity == component.package)
            .ok_or_else(|| internal_mapping_refusal(&component.package))?;
        Ok(AdmittedCapability {
            lock_identity: self.identity,
            binding,
            component: &role.component,
            package: &component.package,
            features: &package.features,
        })
    }
}

fn internal_mapping_refusal(selection: &str) -> Refusal {
    Refusal::selection(
        RefusalCode::CanonicalMismatch,
        SelectionDomain::Lock,
        selection,
        "validated capability mapping is internally inconsistent",
    )
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct LockWire {
    contract: String,
    limits: LimitsWire,
    repositories: Vec<RepositoryWire>,
    packages: Vec<PackageWire>,
    definitions: Vec<PinWire>,
    providers: Vec<PinWire>,
    toolchains: Vec<PinWire>,
    builds: Vec<PinWire>,
    deployments: Vec<PinWire>,
    components: Vec<ComponentWire>,
    roles: Vec<RoleWire>,
    capabilities: Vec<CapabilityWire>,
    workflow_instances: Vec<WorkflowWire>,
    dependencies: Vec<DependencyWire>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LimitsWire {
    max_repositories: u64,
    max_packages: u64,
    max_definitions: u64,
    max_providers: u64,
    max_toolchains: u64,
    max_builds: u64,
    max_deployments: u64,
    max_components: u64,
    max_roles: u64,
    max_capabilities: u64,
    max_workflows: u64,
    max_dependencies: u64,
    max_traversal_steps: u64,
    max_input_bytes: u64,
    max_document_bytes: u64,
    max_identity_bytes: u64,
    max_output_bytes: u64,
}

impl From<Limits> for LimitsWire {
    fn from(limits: Limits) -> Self {
        Self {
            max_repositories: limits.max_repositories,
            max_packages: limits.max_packages,
            max_definitions: limits.max_definitions,
            max_providers: limits.max_providers,
            max_toolchains: limits.max_toolchains,
            max_builds: limits.max_builds,
            max_deployments: limits.max_deployments,
            max_components: limits.max_components,
            max_roles: limits.max_roles,
            max_capabilities: limits.max_capabilities,
            max_workflows: limits.max_workflows,
            max_dependencies: limits.max_dependencies,
            max_traversal_steps: limits.max_traversal_steps,
            max_input_bytes: limits.max_input_bytes,
            max_document_bytes: limits.max_document_bytes,
            max_identity_bytes: limits.max_identity_bytes,
            max_output_bytes: limits.max_output_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct PinWire {
    identity: String,
    revision: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RepositoryWire {
    identity: String,
    revision: String,
    remote: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PackageWire {
    identity: String,
    revision: String,
    version: String,
    repository: String,
    repository_revision: String,
    features: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct ComponentWire {
    identity: String,
    revision: String,
    repository: String,
    package: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct RoleWire {
    identity: String,
    revision: String,
    component: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityWire {
    role: String,
    provider: String,
    build: String,
    contract: String,
    options: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowWire {
    identity: String,
    revision: String,
    definition: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct DependencyWire {
    source: String,
    target: String,
    contract: String,
    reachability: Reachability,
    compatibility: Compatibility,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LockWireRef<'a> {
    contract: &'static str,
    limits: LimitsWire,
    repositories: RepositoryRefs<'a>,
    packages: PackageRefs<'a>,
    definitions: PinRefs<'a, DefinitionId>,
    providers: PinRefs<'a, ProviderId>,
    toolchains: PinRefs<'a, ToolchainId>,
    builds: PinRefs<'a, BuildId>,
    deployments: PinRefs<'a, DeploymentId>,
    components: ComponentRefs<'a>,
    roles: RoleRefs<'a>,
    capabilities: CapabilityRefs<'a>,
    workflow_instances: WorkflowRefs<'a>,
    dependencies: DependencyRefs<'a>,
}

struct PinRefs<'a, T>(&'a [Pin<T>]);
struct RepositoryRefs<'a>(&'a [RepositoryBinding]);
struct PackageRefs<'a>(&'a [PackageBinding]);
struct ComponentRefs<'a>(&'a [ComponentBinding]);
struct RoleRefs<'a>(&'a [RoleBinding]);
struct CapabilityRefs<'a>(&'a [CapabilityBinding]);
struct WorkflowRefs<'a>(&'a [WorkflowBinding]);
struct DependencyRefs<'a>(&'a [DependencyEdge]);

#[derive(Serialize)]
struct PinWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
}

#[derive(Serialize)]
struct RepositoryWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
    remote: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
    version: &'a str,
    repository: &'a str,
    repository_revision: &'a str,
    features: &'a [String],
}

#[derive(Serialize)]
struct ComponentWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
    repository: &'a str,
    package: &'a str,
}

#[derive(Serialize)]
struct RoleWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
    component: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityWireRef<'a> {
    role: &'a str,
    provider: &'a str,
    build: &'a str,
    contract: &'a str,
    options: &'a str,
}

#[derive(Serialize)]
struct WorkflowWireRef<'a> {
    identity: &'a str,
    revision: &'a str,
    definition: &'a str,
}

#[derive(Serialize)]
struct DependencyWireRef<'a> {
    source: &'a str,
    target: &'a str,
    contract: &'a str,
    reachability: Reachability,
    compatibility: Compatibility,
}

impl<T: PinIdentity> Serialize for PinRefs<'_, T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for pin in self.0 {
            sequence.serialize_element(&PinWireRef {
                identity: pin.identity.text(),
                revision: &pin.revision,
            })?;
        }
        sequence.end()
    }
}

macro_rules! serialize_refs {
    ($wrapper:ty, $item:ident, $wire:expr) => {
        impl Serialize for $wrapper {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
                for $item in self.0 {
                    sequence.serialize_element(&$wire)?;
                }
                sequence.end()
            }
        }
    };
}

serialize_refs!(RepositoryRefs<'_>, binding, {
    RepositoryWireRef {
        identity: binding.pin.identity.as_str(),
        revision: &binding.pin.revision,
        remote: &binding.remote,
    }
});
serialize_refs!(PackageRefs<'_>, binding, {
    PackageWireRef {
        identity: binding.pin.identity.as_str(),
        revision: &binding.pin.revision,
        version: &binding.version,
        repository: binding.repository.as_str(),
        repository_revision: &binding.repository_revision,
        features: &binding.features,
    }
});
serialize_refs!(ComponentRefs<'_>, binding, {
    ComponentWireRef {
        identity: binding.pin.identity.as_str(),
        revision: &binding.pin.revision,
        repository: binding.repository.as_str(),
        package: binding.package.as_str(),
    }
});
serialize_refs!(RoleRefs<'_>, binding, {
    RoleWireRef {
        identity: binding.pin.identity.as_str(),
        revision: &binding.pin.revision,
        component: binding.component.as_str(),
    }
});
serialize_refs!(CapabilityRefs<'_>, binding, {
    CapabilityWireRef {
        role: binding.selection.role.as_str(),
        provider: binding.selection.provider.as_str(),
        build: binding.selection.build.as_str(),
        contract: binding.selection.contract.as_str(),
        options: binding.selection.options.as_str(),
    }
});
serialize_refs!(WorkflowRefs<'_>, binding, {
    WorkflowWireRef {
        identity: binding.pin.identity.as_str(),
        revision: &binding.pin.revision,
        definition: binding.definition.as_str(),
    }
});
serialize_refs!(DependencyRefs<'_>, edge, {
    DependencyWireRef {
        source: edge.source.as_str(),
        target: edge.target.as_str(),
        contract: &edge.contract,
        reachability: edge.reachability,
        compatibility: edge.compatibility,
    }
});

/// Validates and canonicalizes one complete lock before any producer invocation.
pub fn validate(selection: &LockSelection, limits: Limits) -> Result<ValidatedLock> {
    preflight(selection, limits)?;
    let output_bytes = preflight_output(selection, limits)?;

    let repositories = validate_repositories(&selection.repositories, limits)?;
    let repository_ids = repositories.keys().cloned().collect::<BTreeSet<_>>();
    let package_ids = validate_packages(&selection.packages, &repositories, limits)?;
    let definition_ids =
        validate_pins(&selection.definitions, SelectionDomain::Definition, limits)?;
    let provider_ids = validate_pins(&selection.providers, SelectionDomain::Provider, limits)?;
    validate_pins(&selection.toolchains, SelectionDomain::Toolchain, limits)?;
    let build_ids = validate_pins(&selection.builds, SelectionDomain::Build, limits)?;
    validate_pins(&selection.deployments, SelectionDomain::Deployment, limits)?;
    let component_ids =
        validate_components(&selection.components, &repository_ids, &package_ids, limits)?;
    let role_ids = validate_roles(&selection.roles, &component_ids, limits)?;
    validate_capabilities(
        &selection.capabilities,
        &role_ids,
        &provider_ids,
        &build_ids,
        limits,
    )?;
    validate_workflows(&selection.workflows, &definition_ids, limits)?;
    validate_dependencies(&selection.dependencies, &package_ids, limits)?;

    let wire = canonical_wire(selection, limits);
    let bytes = serialize_wire(&wire, output_bytes)?;
    let identity = LockIdentity(Sha256::digest(&bytes).into());
    Ok(ValidatedLock {
        identity,
        bytes,
        wire,
        limits,
    })
}

/// Validates the complete lock before entering one producer or consumer invocation seam.
pub fn validate_then<T>(
    selection: &LockSelection,
    limits: Limits,
    invoke: impl FnOnce(&ValidatedLock) -> T,
) -> Result<T> {
    let lock = validate(selection, limits)?;
    Ok(invoke(&lock))
}

/// Strict-reads canonical bytes against an independently supplied selection.
pub fn read(bytes: &[u8], selection: &LockSelection, limits: Limits) -> Result<ValidatedLock> {
    if count(bytes.len())? > limits.max_document_bytes {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            SelectionDomain::Lock,
            CONTRACT,
            "lock document exceeds its caller-lowered byte ceiling",
        ));
    }
    let expected = validate(selection, limits)?;
    if bytes != expected.bytes {
        return Err(Refusal::selection(
            RefusalCode::CanonicalMismatch,
            SelectionDomain::Lock,
            CONTRACT,
            "ecosystem lock bytes do not match the independent selection",
        ));
    }
    Ok(expected)
}

fn preflight(selection: &LockSelection, limits: Limits) -> Result<()> {
    check_collection(
        &selection.repositories,
        limits.max_repositories,
        SelectionDomain::Repository,
        true,
        |binding| binding.pin.identity.as_str(),
    )?;
    check_collection(
        &selection.packages,
        limits.max_packages,
        SelectionDomain::Package,
        true,
        |binding| binding.pin.identity.as_str(),
    )?;
    check_collection(
        &selection.definitions,
        limits.max_definitions,
        SelectionDomain::Definition,
        true,
        |pin| pin.identity.as_str(),
    )?;
    check_collection(
        &selection.providers,
        limits.max_providers,
        SelectionDomain::Provider,
        true,
        |pin| pin.identity.as_str(),
    )?;
    check_collection(
        &selection.toolchains,
        limits.max_toolchains,
        SelectionDomain::Toolchain,
        true,
        |pin| pin.identity.as_str(),
    )?;
    check_collection(
        &selection.builds,
        limits.max_builds,
        SelectionDomain::Build,
        true,
        |pin| pin.identity.as_str(),
    )?;
    check_collection(
        &selection.deployments,
        limits.max_deployments,
        SelectionDomain::Deployment,
        true,
        |pin| pin.identity.as_str(),
    )?;
    check_collection(
        &selection.components,
        limits.max_components,
        SelectionDomain::Component,
        true,
        |binding| binding.pin.identity.as_str(),
    )?;
    check_collection(
        &selection.roles,
        limits.max_roles,
        SelectionDomain::Role,
        true,
        |binding| binding.pin.identity.as_str(),
    )?;
    check_collection(
        &selection.capabilities,
        limits.max_capabilities,
        SelectionDomain::Capability,
        true,
        |binding| binding.selection.role.as_str(),
    )?;
    check_collection(
        &selection.workflows,
        limits.max_workflows,
        SelectionDomain::WorkflowInstance,
        true,
        |binding| binding.pin.identity.as_str(),
    )?;
    check_collection(
        &selection.dependencies,
        limits.max_dependencies,
        SelectionDomain::Dependency,
        false,
        |edge| edge.source.as_str(),
    )?;
    let input_bytes = selection_input_bytes(selection)?;
    if input_bytes > limits.max_input_bytes {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            SelectionDomain::Lock,
            CONTRACT,
            "lock input exceeds its caller-lowered byte ceiling",
        ));
    }
    if dependency_steps(selection)? > limits.max_traversal_steps {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            SelectionDomain::Dependency,
            CONTRACT,
            "dependency validation exceeds its caller-lowered traversal ceiling",
        ));
    }
    Ok(())
}

fn check_collection<T>(
    values: &[T],
    maximum: u64,
    domain: SelectionDomain,
    required: bool,
    identity: impl Fn(&T) -> &str,
) -> Result<()> {
    if count(values.len())? > maximum {
        let first_over = usize::try_from(maximum)
            .ok()
            .and_then(|index| values.get(index))
            .expect("an over-limit collection contains its first excess item");
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            domain,
            identity(first_over),
            "lock collection exceeds its caller-lowered item ceiling",
        ));
    }
    if required && values.is_empty() {
        return Err(Refusal::new(
            RefusalCode::MissingSelection,
            Some(domain),
            "required lock domain is empty",
        ));
    }
    Ok(())
}

fn count(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Refusal::selection(
            RefusalCode::IntegerOverflow,
            SelectionDomain::Lock,
            CONTRACT,
            "lock count is not representable",
        )
    })
}

fn dependency_steps(selection: &LockSelection) -> Result<u64> {
    let packages = count(selection.packages.len())?;
    let dependencies = count(selection.dependencies.len())?;
    packages
        .checked_mul(4)
        .and_then(|package_steps| {
            dependencies
                .checked_mul(2)
                .and_then(|dependency_steps| package_steps.checked_add(dependency_steps))
        })
        .ok_or_else(|| {
            Refusal::selection(
                RefusalCode::IntegerOverflow,
                SelectionDomain::Dependency,
                CONTRACT,
                "dependency traversal preflight overflowed",
            )
        })
}

fn validate_state<T: PinIdentity>(
    pin: &Pin<T>,
    domain: SelectionDomain,
    limits: Limits,
) -> Result<()> {
    validate_text(pin.identity.text(), domain, pin.identity.text(), limits)?;
    let code = match pin.state {
        PinState::Exact => {
            validate_text(&pin.revision, domain, pin.identity.text(), limits)?;
            if !is_content_addressed_revision(&pin.revision) {
                return Err(Refusal::selection(
                    RefusalCode::MovingSelection,
                    domain,
                    pin.identity.text(),
                    "exact pin is not a canonical content-addressed revision",
                ));
            }
            return Ok(());
        }
        PinState::Missing => RefusalCode::MissingSelection,
        PinState::Stale => RefusalCode::StaleSelection,
        PinState::Dirty => RefusalCode::DirtySelection,
        PinState::Moving => RefusalCode::MovingSelection,
        PinState::Unknown => RefusalCode::UnknownSelection,
    };
    Err(Refusal::selection(
        code,
        domain,
        pin.identity.text(),
        "lock selection is not one clean immutable revision",
    ))
}

fn validate_text(
    value: &str,
    domain: SelectionDomain,
    selection: &str,
    limits: Limits,
) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Refusal::selection(
            RefusalCode::Malformed,
            domain,
            selection,
            "lock identity, revision, remote, feature set, or contract is empty",
        ));
    }
    if count(value.len())? > limits.max_identity_bytes {
        return Err(Refusal::selection(
            RefusalCode::ResourceExhausted,
            domain,
            selection,
            "lock field exceeds its caller-lowered byte ceiling",
        ));
    }
    Ok(())
}

fn validate_pins<T: PinIdentity>(
    pins: &[Pin<T>],
    domain: SelectionDomain,
    limits: Limits,
) -> Result<BTreeSet<String>> {
    let mut identities = BTreeSet::new();
    for pin in pins {
        validate_state(pin, domain, limits)?;
        if !identities.insert(pin.identity.text().to_owned()) {
            return Err(Refusal::selection(
                RefusalCode::DuplicateSelection,
                domain,
                pin.identity.text(),
                "lock domain contains a duplicate identity",
            ));
        }
    }
    Ok(identities)
}

fn validate_repositories(
    repositories: &[RepositoryBinding],
    limits: Limits,
) -> Result<BTreeMap<String, String>> {
    let mut identities = BTreeMap::new();
    for binding in repositories {
        validate_state(&binding.pin, SelectionDomain::Repository, limits)?;
        validate_text(
            &binding.remote,
            SelectionDomain::Repository,
            binding.pin.identity.as_str(),
            limits,
        )?;
        if identities
            .insert(
                binding.pin.identity.as_str().to_owned(),
                binding.pin.revision.clone(),
            )
            .is_some()
        {
            return Err(Refusal::selection(
                RefusalCode::DuplicateSelection,
                SelectionDomain::Repository,
                binding.pin.identity.as_str(),
                "repository identity is duplicated",
            ));
        }
    }
    Ok(identities)
}

fn validate_packages(
    packages: &[PackageBinding],
    repositories: &BTreeMap<String, String>,
    limits: Limits,
) -> Result<BTreeSet<String>> {
    let mut identities = BTreeSet::new();
    for binding in packages {
        validate_state(&binding.pin, SelectionDomain::Package, limits)?;
        validate_text(
            &binding.version,
            SelectionDomain::Package,
            binding.pin.identity.as_str(),
            limits,
        )?;
        validate_text(
            &binding.repository_revision,
            SelectionDomain::Package,
            binding.pin.identity.as_str(),
            limits,
        )?;
        for feature in &binding.features {
            validate_text(
                feature,
                SelectionDomain::Package,
                binding.pin.identity.as_str(),
                limits,
            )?;
        }
        let Some(repository_revision) = repositories.get(binding.repository.as_str()) else {
            return Err(cross_domain(
                SelectionDomain::Repository,
                binding.repository.as_str(),
            ));
        };
        if repository_revision != &binding.repository_revision {
            return Err(Refusal::selection(
                RefusalCode::CrossDomainReference,
                SelectionDomain::Repository,
                binding.repository.as_str(),
                "package repository revision does not match the selected repository pin",
            ));
        }
        if !identities.insert(binding.pin.identity.as_str().to_owned()) {
            return Err(duplicate(
                SelectionDomain::Package,
                binding.pin.identity.as_str(),
            ));
        }
    }
    Ok(identities)
}

fn validate_components(
    components: &[ComponentBinding],
    repositories: &BTreeSet<String>,
    packages: &BTreeSet<String>,
    limits: Limits,
) -> Result<BTreeSet<String>> {
    let mut identities = BTreeSet::new();
    for binding in components {
        validate_state(&binding.pin, SelectionDomain::Component, limits)?;
        if !repositories.contains(binding.repository.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Repository,
                binding.repository.as_str(),
            ));
        }
        if !packages.contains(binding.package.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Package,
                binding.package.as_str(),
            ));
        }
        if !identities.insert(binding.pin.identity.as_str().to_owned()) {
            return Err(duplicate(
                SelectionDomain::Component,
                binding.pin.identity.as_str(),
            ));
        }
    }
    Ok(identities)
}

fn validate_roles(
    roles: &[RoleBinding],
    components: &BTreeSet<String>,
    limits: Limits,
) -> Result<BTreeSet<String>> {
    let mut identities = BTreeSet::new();
    for binding in roles {
        validate_state(&binding.pin, SelectionDomain::Role, limits)?;
        if !components.contains(binding.component.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Component,
                binding.component.as_str(),
            ));
        }
        if !identities.insert(binding.pin.identity.as_str()) {
            return Err(duplicate(
                SelectionDomain::Role,
                binding.pin.identity.as_str(),
            ));
        }
    }
    Ok(identities.into_iter().map(str::to_owned).collect())
}

fn validate_capabilities(
    capabilities: &[CapabilityBinding],
    roles: &BTreeSet<String>,
    providers: &BTreeSet<String>,
    builds: &BTreeSet<String>,
    limits: Limits,
) -> Result<()> {
    let mut bound_roles = BTreeSet::new();
    for binding in capabilities {
        let selection = &binding.selection;
        validate_text(
            selection.role.as_str(),
            SelectionDomain::Capability,
            selection.role.as_str(),
            limits,
        )?;
        validate_text(
            selection.provider.as_str(),
            SelectionDomain::Capability,
            selection.role.as_str(),
            limits,
        )?;
        validate_text(
            selection.build.as_str(),
            SelectionDomain::Capability,
            selection.role.as_str(),
            limits,
        )?;
        validate_text(
            selection.contract.as_str(),
            SelectionDomain::Capability,
            selection.role.as_str(),
            limits,
        )?;
        validate_text(
            selection.options.as_str(),
            SelectionDomain::Capability,
            selection.role.as_str(),
            limits,
        )?;
        if !roles.contains(selection.role.as_str()) {
            return Err(cross_domain(SelectionDomain::Role, selection.role.as_str()));
        }
        if !providers.contains(selection.provider.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Provider,
                selection.provider.as_str(),
            ));
        }
        if !builds.contains(selection.build.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Build,
                selection.build.as_str(),
            ));
        }
        if !bound_roles.insert(selection.role.as_str()) {
            return Err(duplicate(
                SelectionDomain::Capability,
                selection.role.as_str(),
            ));
        }
    }
    if let Some(missing_role) = roles
        .iter()
        .find(|role| !bound_roles.contains(role.as_str()))
    {
        return Err(Refusal::selection(
            RefusalCode::MissingSelection,
            SelectionDomain::Capability,
            missing_role,
            "required role has no exact capability binding",
        ));
    }
    Ok(())
}

fn capability_label(selection: &CapabilitySelection) -> String {
    format!(
        "{} / {} / {} / {} / {}",
        selection.role.as_str(),
        selection.provider.as_str(),
        selection.build.as_str(),
        selection.contract.as_str(),
        selection.options.as_str()
    )
}

fn validate_workflows(
    workflows: &[WorkflowBinding],
    definitions: &BTreeSet<String>,
    limits: Limits,
) -> Result<()> {
    let mut identities = BTreeSet::new();
    for binding in workflows {
        validate_state(&binding.pin, SelectionDomain::WorkflowInstance, limits)?;
        if !definitions.contains(binding.definition.as_str()) {
            return Err(cross_domain(
                SelectionDomain::Definition,
                binding.definition.as_str(),
            ));
        }
        if !identities.insert(binding.pin.identity.as_str()) {
            return Err(duplicate(
                SelectionDomain::WorkflowInstance,
                binding.pin.identity.as_str(),
            ));
        }
    }
    Ok(())
}

fn validate_dependencies(
    dependencies: &[DependencyEdge],
    packages: &BTreeSet<String>,
    limits: Limits,
) -> Result<()> {
    let mut identities = BTreeSet::new();
    let mut adjacency = packages
        .iter()
        .map(|package| (package.clone(), Vec::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    let mut indegree = packages
        .iter()
        .map(|package| (package.clone(), 0_u64))
        .collect::<BTreeMap<_, _>>();
    for edge in dependencies {
        validate_text(
            &edge.contract,
            SelectionDomain::Dependency,
            edge.source.as_str(),
            limits,
        )?;
        if !packages.contains(edge.source.as_str()) || !packages.contains(edge.target.as_str()) {
            let foreign = if !packages.contains(edge.source.as_str()) {
                edge.source.as_str()
            } else {
                edge.target.as_str()
            };
            return Err(cross_domain(SelectionDomain::Package, foreign));
        }
        let key = (
            edge.source.as_str(),
            edge.target.as_str(),
            edge.contract.as_str(),
        );
        if !identities.insert(key) {
            return Err(duplicate(
                SelectionDomain::Dependency,
                format!(
                    "{} -> {} [{}]",
                    edge.source.as_str(),
                    edge.target.as_str(),
                    edge.contract
                ),
            ));
        }
        let edge_identity = format!(
            "{} -> {} [{}]",
            edge.source.as_str(),
            edge.target.as_str(),
            edge.contract
        );
        match edge.reachability {
            Reachability::Unreachable => {
                return Err(Refusal::selection(
                    RefusalCode::UnreachableDependency,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency target is explicitly unreachable",
                ));
            }
            Reachability::Unresolved => {
                return Err(Refusal::selection(
                    RefusalCode::UnresolvedDependency,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency reachability is unresolved",
                ));
            }
            Reachability::Cyclic => {
                return Err(Refusal::selection(
                    RefusalCode::CyclicDependency,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency reachability contains a declared cycle",
                ));
            }
            Reachability::Reachable => {}
        }
        match edge.compatibility {
            Compatibility::Compatible => {}
            Compatibility::Incompatible => {
                return Err(Refusal::selection(
                    RefusalCode::IncompatibleContract,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency contract is explicitly incompatible",
                ));
            }
            Compatibility::Affected => {
                return Err(Refusal::selection(
                    RefusalCode::AffectedSelection,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency contract is affected and not qualified",
                ));
            }
            Compatibility::Stale => {
                return Err(Refusal::selection(
                    RefusalCode::StaleSelection,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "dependency compatibility evidence is stale",
                ));
            }
            Compatibility::Unknown => {
                return Err(Refusal::selection(
                    RefusalCode::UnknownCompatibility,
                    SelectionDomain::Dependency,
                    edge_identity,
                    "reachability does not establish compatibility",
                ));
            }
        }
        adjacency
            .get_mut(edge.source.as_str())
            .expect("package membership was validated")
            .push(edge.target.as_str().to_owned());
        let target_indegree = indegree
            .get_mut(edge.target.as_str())
            .expect("package membership was validated");
        *target_indegree = target_indegree.checked_add(1).ok_or_else(|| {
            Refusal::selection(
                RefusalCode::IntegerOverflow,
                SelectionDomain::Dependency,
                edge.target.as_str(),
                "dependency indegree overflowed",
            )
        })?;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(package, degree)| (*degree == 0).then_some(package.clone()))
        .collect::<BTreeSet<_>>();
    let mut visited = 0_u64;
    while let Some(package) = ready.pop_first() {
        visited = visited.checked_add(1).ok_or_else(|| {
            Refusal::selection(
                RefusalCode::IntegerOverflow,
                SelectionDomain::Dependency,
                package.as_str(),
                "dependency traversal count overflowed",
            )
        })?;
        for target in adjacency
            .get(&package)
            .expect("every package owns an adjacency entry")
        {
            let degree = indegree
                .get_mut(target)
                .expect("every package owns an indegree entry");
            *degree = degree.checked_sub(1).ok_or_else(|| {
                Refusal::selection(
                    RefusalCode::IntegerOverflow,
                    SelectionDomain::Dependency,
                    target,
                    "dependency indegree underflowed",
                )
            })?;
            if *degree == 0 {
                ready.insert(target.clone());
            }
        }
    }
    if visited != count(packages.len())? {
        let cycle_member = indegree
            .iter()
            .find_map(|(package, degree)| (*degree > 0).then_some(package.as_str()))
            .unwrap_or("unknown dependency cycle");
        return Err(Refusal::selection(
            RefusalCode::CyclicDependency,
            SelectionDomain::Dependency,
            cycle_member,
            "dependency topology contains a cycle",
        ));
    }
    Ok(())
}

fn duplicate(domain: SelectionDomain, selection: impl Into<String>) -> Refusal {
    Refusal::selection(
        RefusalCode::DuplicateSelection,
        domain,
        selection,
        "lock domain contains a duplicate identity or edge",
    )
}

fn cross_domain(domain: SelectionDomain, selection: impl Into<String>) -> Refusal {
    Refusal::selection(
        RefusalCode::CrossDomainReference,
        domain,
        selection,
        "lock mapping references an identity absent from the required domain",
    )
}

fn borrowed_wire(selection: &LockSelection, limits: Limits) -> LockWireRef<'_> {
    LockWireRef {
        contract: CONTRACT,
        limits: limits.into(),
        repositories: RepositoryRefs(&selection.repositories),
        packages: PackageRefs(&selection.packages),
        definitions: PinRefs(&selection.definitions),
        providers: PinRefs(&selection.providers),
        toolchains: PinRefs(&selection.toolchains),
        builds: PinRefs(&selection.builds),
        deployments: PinRefs(&selection.deployments),
        components: ComponentRefs(&selection.components),
        roles: RoleRefs(&selection.roles),
        capabilities: CapabilityRefs(&selection.capabilities),
        workflow_instances: WorkflowRefs(&selection.workflows),
        dependencies: DependencyRefs(&selection.dependencies),
    }
}

fn canonical_wire(selection: &LockSelection, limits: Limits) -> LockWire {
    let mut repositories = selection
        .repositories
        .iter()
        .map(|binding| RepositoryWire {
            identity: binding.pin.identity.as_str().to_owned(),
            revision: binding.pin.revision.clone(),
            remote: binding.remote.clone(),
        })
        .collect::<Vec<_>>();
    repositories.sort();
    let mut packages = selection
        .packages
        .iter()
        .map(|binding| PackageWire {
            identity: binding.pin.identity.as_str().to_owned(),
            revision: binding.pin.revision.clone(),
            version: binding.version.clone(),
            repository: binding.repository.as_str().to_owned(),
            repository_revision: binding.repository_revision.clone(),
            features: binding.features.clone(),
        })
        .collect::<Vec<_>>();
    packages.sort();
    let mut components = selection
        .components
        .iter()
        .map(|binding| ComponentWire {
            identity: binding.pin.identity.as_str().to_owned(),
            revision: binding.pin.revision.clone(),
            repository: binding.repository.as_str().to_owned(),
            package: binding.package.as_str().to_owned(),
        })
        .collect::<Vec<_>>();
    components.sort();
    let mut roles = selection
        .roles
        .iter()
        .map(|binding| RoleWire {
            identity: binding.pin.identity.as_str().to_owned(),
            revision: binding.pin.revision.clone(),
            component: binding.component.as_str().to_owned(),
        })
        .collect::<Vec<_>>();
    roles.sort();
    let mut capabilities = selection
        .capabilities
        .iter()
        .map(|binding| CapabilityWire {
            role: binding.selection.role.as_str().to_owned(),
            provider: binding.selection.provider.as_str().to_owned(),
            build: binding.selection.build.as_str().to_owned(),
            contract: binding.selection.contract.as_str().to_owned(),
            options: binding.selection.options.as_str().to_owned(),
        })
        .collect::<Vec<_>>();
    capabilities.sort();
    let mut workflow_instances = selection
        .workflows
        .iter()
        .map(|binding| WorkflowWire {
            identity: binding.pin.identity.as_str().to_owned(),
            revision: binding.pin.revision.clone(),
            definition: binding.definition.as_str().to_owned(),
        })
        .collect::<Vec<_>>();
    workflow_instances.sort();
    let mut dependencies = selection
        .dependencies
        .iter()
        .map(|edge| DependencyWire {
            source: edge.source.as_str().to_owned(),
            target: edge.target.as_str().to_owned(),
            contract: edge.contract.clone(),
            reachability: edge.reachability,
            compatibility: edge.compatibility,
        })
        .collect::<Vec<_>>();
    dependencies.sort();
    LockWire {
        contract: CONTRACT.to_owned(),
        limits: limits.into(),
        repositories,
        packages,
        definitions: canonical_pins(&selection.definitions),
        providers: canonical_pins(&selection.providers),
        toolchains: canonical_pins(&selection.toolchains),
        builds: canonical_pins(&selection.builds),
        deployments: canonical_pins(&selection.deployments),
        components,
        roles,
        capabilities,
        workflow_instances,
        dependencies,
    }
}

fn canonical_pins<T: PinIdentity>(pins: &[Pin<T>]) -> Vec<PinWire> {
    let mut wire = pins
        .iter()
        .map(|pin| PinWire {
            identity: pin.identity.text().to_owned(),
            revision: pin.revision.clone(),
        })
        .collect::<Vec<_>>();
    wire.sort();
    wire
}

struct BoundedWriter {
    bytes: Vec<u8>,
    maximum: u64,
}

struct CountingWriter {
    consumed: u64,
    maximum: u64,
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let incoming = u64::try_from(buffer.len())
            .map_err(|_| std::io::Error::other("canonical write length is not representable"))?;
        let next = self
            .consumed
            .checked_add(incoming)
            .ok_or_else(|| std::io::Error::other("canonical length overflowed"))?;
        if next > self.maximum {
            return Err(std::io::Error::other(
                "canonical output exceeds its caller-lowered byte ceiling",
            ));
        }
        self.consumed = next;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn preflight_output(selection: &LockSelection, limits: Limits) -> Result<u64> {
    let borrowed = borrowed_wire(selection, limits);
    let mut writer = CountingWriter {
        consumed: 0,
        maximum: limits.max_output_bytes,
    };
    serde_json::to_writer(&mut writer, &borrowed).map_err(|error| {
        let code = if error.is_io() {
            RefusalCode::ResourceExhausted
        } else {
            RefusalCode::Malformed
        };
        Refusal::selection(
            code,
            SelectionDomain::Lock,
            CONTRACT,
            "canonical output preflight failed or exceeded its byte ceiling",
        )
    })?;
    Ok(writer.consumed)
}

impl Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let current = u64::try_from(self.bytes.len())
            .map_err(|_| std::io::Error::other("canonical length is not representable"))?;
        let incoming = u64::try_from(buffer.len())
            .map_err(|_| std::io::Error::other("canonical write length is not representable"))?;
        let next = current
            .checked_add(incoming)
            .ok_or_else(|| std::io::Error::other("canonical length overflowed"))?;
        if next > self.maximum {
            return Err(std::io::Error::other(
                "canonical output exceeds its caller-lowered byte ceiling",
            ));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serialize_wire(wire: &LockWire, exact_length: u64) -> Result<Vec<u8>> {
    let capacity = usize::try_from(exact_length).map_err(|_| {
        Refusal::selection(
            RefusalCode::IntegerOverflow,
            SelectionDomain::Lock,
            CONTRACT,
            "canonical output length is not representable",
        )
    })?;
    let mut writer = BoundedWriter {
        bytes: Vec::with_capacity(capacity),
        maximum: exact_length,
    };
    serde_json::to_writer(&mut writer, wire).map_err(|error| {
        let code = if error.is_io() {
            RefusalCode::ResourceExhausted
        } else {
            RefusalCode::Malformed
        };
        Refusal::selection(
            code,
            SelectionDomain::Lock,
            CONTRACT,
            "canonical lock serialization failed or exceeded its byte ceiling",
        )
    })?;
    if writer.bytes.len() != capacity {
        return Err(Refusal::selection(
            RefusalCode::CanonicalMismatch,
            SelectionDomain::Lock,
            CONTRACT,
            "canonical preflight length changed before final serialization",
        ));
    }
    Ok(writer.bytes)
}

fn selection_input_bytes(selection: &LockSelection) -> Result<u64> {
    let mut total = 0_u64;
    macro_rules! add {
        ($value:expr) => {{
            let length = u64::try_from($value.len()).map_err(|_| {
                Refusal::selection(
                    RefusalCode::IntegerOverflow,
                    SelectionDomain::Lock,
                    CONTRACT,
                    "lock string length is not representable",
                )
            })?;
            total = total.checked_add(length).ok_or_else(|| {
                Refusal::selection(
                    RefusalCode::IntegerOverflow,
                    SelectionDomain::Lock,
                    CONTRACT,
                    "lock input byte count overflowed",
                )
            })?;
        }};
    }
    for binding in &selection.repositories {
        add!(binding.pin.identity.as_str());
        add!(binding.pin.revision);
        add!(binding.remote);
    }
    for binding in &selection.packages {
        add!(binding.pin.identity.as_str());
        add!(binding.pin.revision);
        add!(binding.version);
        add!(binding.repository.as_str());
        add!(binding.repository_revision);
        for feature in &binding.features {
            add!(feature);
        }
    }
    macro_rules! add_pins {
        ($pins:expr) => {
            for pin in $pins {
                add!(pin.identity.as_str());
                add!(pin.revision);
            }
        };
    }
    add_pins!(&selection.definitions);
    add_pins!(&selection.providers);
    add_pins!(&selection.toolchains);
    add_pins!(&selection.builds);
    add_pins!(&selection.deployments);
    for binding in &selection.components {
        add!(binding.pin.identity.as_str());
        add!(binding.pin.revision);
        add!(binding.repository.as_str());
        add!(binding.package.as_str());
    }
    for binding in &selection.roles {
        add!(binding.pin.identity.as_str());
        add!(binding.pin.revision);
        add!(binding.component.as_str());
    }
    for binding in &selection.capabilities {
        add!(binding.selection.role.as_str());
        add!(binding.selection.provider.as_str());
        add!(binding.selection.build.as_str());
        add!(binding.selection.contract.as_str());
        add!(binding.selection.options.as_str());
    }
    for binding in &selection.workflows {
        add!(binding.pin.identity.as_str());
        add!(binding.pin.revision);
        add!(binding.definition.as_str());
    }
    for edge in &selection.dependencies {
        add!(edge.source.as_str());
        add!(edge.target.as_str());
        add!(edge.contract);
    }
    Ok(total)
}
