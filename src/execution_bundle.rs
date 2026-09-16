// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Canonical composition of independently owned execution results.

use std::{
    cell::Cell,
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{
    de::{DeserializeSeed as _, IgnoredAny, Visitor},
    ser::SerializeSeq as _,
    Deserialize, Serialize,
};

use crate::{
    ecosystem_lock::{AdmittedCapability, CapabilitySelection, ValidatedLock},
    ArtifactId, BuildId, ContractId, DefinitionId, Disposition, Error as CommonError, ErrorCode,
    Meter, PackageId, ProviderId, RoleId, WorkflowInstanceId,
};

/// Canonical execution-bundle wire contract.
pub const CONTRACT: &str = "quire.execution-bundle/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../schemas/quire.execution-bundle-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "a4e1dc8069bd38fd99d3248cccb21c1d07422d3705e80054d120ce7c3adae433";

/// Stable boundary error code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundleErrorCode {
    /// A caller-lowered item or byte bound was exceeded.
    ResourceExhausted,
    /// The caller-lowered operational deadline expired.
    DurationExpired,
    /// A requested or completed item key occurred more than once.
    DuplicateResult,
    /// An operation named an item absent from the request.
    UnknownItem,
    /// An item used an invalid disposition for that operation.
    InvalidDisposition,
    /// Supplied artifact identity differed from the selected artifact.
    ArtifactMismatch,
    /// A selected result requires different shared prerequisite identities.
    PrerequisiteContradiction,
    /// Input did not conform to the closed wire contract.
    Malformed,
    /// Input was valid data but not the canonical representation.
    CanonicalMismatch,
}

/// Typed execution-bundle boundary refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleError {
    code: BundleErrorCode,
    item: Option<ItemKey>,
    detail: &'static str,
}

impl BundleError {
    fn new(code: BundleErrorCode, item: Option<ItemKey>, detail: &'static str) -> Self {
        Self { code, item, detail }
    }

    fn resource(detail: &'static str) -> Self {
        Self::new(BundleErrorCode::ResourceExhausted, None, detail)
    }

    fn duration(detail: &'static str) -> Self {
        Self::new(BundleErrorCode::DurationExpired, None, detail)
    }

    fn from_common(error: CommonError) -> Self {
        let code = if error.code() == ErrorCode::ResourceExhausted {
            BundleErrorCode::ResourceExhausted
        } else {
            BundleErrorCode::Malformed
        };
        Self::new(code, None, error.detail())
    }

    /// Stable refusal category.
    #[must_use]
    pub const fn code(&self) -> BundleErrorCode {
        self.code
    }

    /// Exact affected item, when item-local.
    #[must_use]
    pub const fn item(&self) -> Option<&ItemKey> {
        self.item.as_ref()
    }

    /// Bounded diagnostic detail.
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for BundleError {}

/// Execution-bundle result.
pub type Result<T> = std::result::Result<T, BundleError>;

/// Selected result domain. Declaration order is canonical wire order.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    /// State semantics.
    State,
    /// Temporal semantics.
    Temporal,
    /// Observation facts and queries.
    Observation,
    /// Protocol/choreography semantics.
    Protocol,
    /// Optional analyses.
    Analysis,
    /// Backend-owned execution evidence.
    Backend,
}

/// Canonical item identity: domain plus semantic definition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ItemKey {
    domain: Domain,
    identity: DefinitionId,
}

impl ItemKey {
    /// Constructs one already-bounded semantic key.
    #[must_use]
    pub const fn new(domain: Domain, identity: DefinitionId) -> Self {
        Self { domain, identity }
    }

    /// Selected domain.
    #[must_use]
    pub const fn domain(&self) -> Domain {
        self.domain
    }

    /// Domain-owned semantic identity.
    #[must_use]
    pub const fn identity(&self) -> &DefinitionId {
        &self.identity
    }
}

/// One exact requested item and producer selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemRequest {
    key: ItemKey,
    capability: CapabilitySelection,
    artifact: ArtifactId,
    prerequisites: SharedPrerequisites,
}

impl ItemRequest {
    /// Constructs an exact request from bounded identities.
    #[must_use]
    pub const fn new(
        key: ItemKey,
        capability: CapabilitySelection,
        artifact: ArtifactId,
        prerequisites: SharedPrerequisites,
    ) -> Self {
        Self {
            key,
            capability,
            artifact,
            prerequisites,
        }
    }

    /// Item key.
    #[must_use]
    pub const fn key(&self) -> &ItemKey {
        &self.key
    }

    /// Exact capability tuple.
    #[must_use]
    pub const fn capability(&self) -> &CapabilitySelection {
        &self.capability
    }

    /// Exact owner-artifact identity.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactId {
        &self.artifact
    }

    /// Exact shared prerequisites required by this selected result.
    #[must_use]
    pub const fn prerequisites(&self) -> &SharedPrerequisites {
        &self.prerequisites
    }

    /// Replaces the capability candidate before request validation.
    pub fn set_capability(&mut self, capability: CapabilitySelection) {
        self.capability = capability;
    }

    /// Replaces the required shared prerequisites before request validation.
    pub fn set_prerequisites(&mut self, prerequisites: SharedPrerequisites) {
        self.prerequisites = prerequisites;
    }
}

/// Validated, canonically ordered execution request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionRequest {
    package: PackageId,
    workflow: WorkflowInstanceId,
    items: Vec<ItemRequest>,
}

impl ExecutionRequest {
    /// Validates uniqueness and item count before retaining canonical order.
    pub fn new(
        package: PackageId,
        workflow: WorkflowInstanceId,
        mut items: Vec<ItemRequest>,
        max_items: u64,
    ) -> Result<Self> {
        check_count(
            items.len(),
            max_items,
            "request item count exceeds its ceiling",
        )?;
        if items.is_empty() {
            return Err(BundleError::new(
                BundleErrorCode::Malformed,
                None,
                "execution request must select at least one item",
            ));
        }
        items.sort_by(|left, right| left.key.cmp(&right.key));
        if let Some(pair) = items.windows(2).find(|pair| pair[0].key == pair[1].key) {
            return Err(BundleError::new(
                BundleErrorCode::DuplicateResult,
                Some(pair[0].key.clone()),
                "execution request contains a duplicate item key",
            ));
        }
        Ok(Self {
            package,
            workflow,
            items,
        })
    }

    /// Exact selected package.
    #[must_use]
    pub const fn package(&self) -> &PackageId {
        &self.package
    }

    /// Exact selected workflow instance.
    #[must_use]
    pub const fn workflow(&self) -> &WorkflowInstanceId {
        &self.workflow
    }

    /// Canonically ordered requested items.
    #[must_use]
    pub fn items(&self) -> &[ItemRequest] {
        &self.items
    }
}

/// Exact shared lock identities that apply to every item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedPrerequisites {
    model: ArtifactId,
    package: ArtifactId,
    clock: ArtifactId,
    population: ArtifactId,
}

impl SharedPrerequisites {
    /// Constructs exact shared prerequisite identities.
    #[must_use]
    pub const fn new(
        model: ArtifactId,
        package: ArtifactId,
        clock: ArtifactId,
        population: ArtifactId,
    ) -> Self {
        Self {
            model,
            package,
            clock,
            population,
        }
    }

    /// Exact model-lock identity.
    #[must_use]
    pub const fn model(&self) -> &ArtifactId {
        &self.model
    }

    /// Exact package-lock identity.
    #[must_use]
    pub const fn package(&self) -> &ArtifactId {
        &self.package
    }

    /// Exact clock-lock identity.
    #[must_use]
    pub const fn clock(&self) -> &ArtifactId {
        &self.clock
    }

    /// Exact population-lock identity.
    #[must_use]
    pub const fn population(&self) -> &ArtifactId {
        &self.population
    }
}

/// Caller-lowered execution and output bounds.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExecutionLimits {
    max_items: u64,
    max_artifact_bytes: u64,
    max_result_bytes: u64,
    max_output_bytes: u64,
}

impl ExecutionLimits {
    /// Constructs an explicit limits envelope without ambient defaults.
    #[must_use]
    pub const fn new(
        max_items: u64,
        max_artifact_bytes: u64,
        max_result_bytes: u64,
        max_output_bytes: u64,
    ) -> Self {
        Self {
            max_items,
            max_artifact_bytes,
            max_result_bytes,
            max_output_bytes,
        }
    }

    /// Maximum retained item count.
    #[must_use]
    pub const fn max_items(self) -> u64 {
        self.max_items
    }

    /// Maximum bytes passed to one owner strict reader.
    #[must_use]
    pub const fn max_artifact_bytes(self) -> u64 {
        self.max_artifact_bytes
    }

    /// Maximum retained owner-result bytes per item.
    #[must_use]
    pub const fn max_result_bytes(self) -> u64 {
        self.max_result_bytes
    }

    /// Maximum canonical bundle bytes.
    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }
}

/// Independent owner-defined semantic axes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultAxes {
    truth: String,
    execution: String,
    adequacy: String,
    completeness: String,
    provenance: ArtifactId,
}

impl ResultAxes {
    /// Bounds and retains four independent axes plus exact provenance.
    pub fn new(
        truth: &str,
        execution: &str,
        adequacy: &str,
        completeness: &str,
        provenance: ArtifactId,
        max_axis_bytes: u64,
    ) -> Result<Self> {
        Ok(Self {
            truth: bounded_text(truth, max_axis_bytes, "truth axis is invalid")?,
            execution: bounded_text(execution, max_axis_bytes, "execution axis is invalid")?,
            adequacy: bounded_text(adequacy, max_axis_bytes, "adequacy axis is invalid")?,
            completeness: bounded_text(
                completeness,
                max_axis_bytes,
                "completeness axis is invalid",
            )?,
            provenance,
        })
    }

    /// Owner truth axis.
    #[must_use]
    pub fn truth(&self) -> &str {
        &self.truth
    }

    /// Owner execution axis.
    #[must_use]
    pub fn execution(&self) -> &str {
        &self.execution
    }

    /// Owner adequacy axis.
    #[must_use]
    pub fn adequacy(&self) -> &str {
        &self.adequacy
    }

    /// Owner completeness axis.
    #[must_use]
    pub fn completeness(&self) -> &str {
        &self.completeness
    }

    /// Exact owner provenance identity.
    #[must_use]
    pub const fn provenance(&self) -> &ArtifactId {
        &self.provenance
    }
}

/// Opaque owner result preserved without semantic reinterpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerResult {
    disposition: Disposition,
    bytes: Vec<u8>,
    axes: ResultAxes,
}

impl OwnerResult {
    /// Bounds bytes before retention and preserves the owner disposition and axes.
    pub fn new(
        disposition: Disposition,
        bytes: &[u8],
        axes: ResultAxes,
        max_result_bytes: u64,
    ) -> Result<Self> {
        check_count(
            bytes.len(),
            max_result_bytes,
            "owner result exceeds its byte ceiling",
        )?;
        Ok(Self {
            disposition,
            bytes: bytes.to_vec(),
            axes,
        })
    }

    fn validate(&self, limits: ExecutionLimits) -> Result<()> {
        check_count(
            self.bytes.len(),
            limits.max_result_bytes,
            "owner result exceeds its byte ceiling",
        )?;
        for value in [
            self.axes.truth.as_str(),
            self.axes.execution.as_str(),
            self.axes.adequacy.as_str(),
            self.axes.completeness.as_str(),
            self.axes.provenance.as_str(),
        ] {
            check_count(
                value.len(),
                limits.max_artifact_bytes,
                "owner result axis exceeds its byte ceiling",
            )?;
        }
        Ok(())
    }
}

/// Explicit reason why no owner result bytes exist.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoResultCause {
    /// A required owner input is absent.
    MissingInput,
    /// Exact selections cannot be composed compatibly.
    IncompatibleSelection,
    /// Exact selected capability is unsupported by this lock.
    UnsupportedCapability,
    /// Previously valid evidence does not match the selected revision.
    StaleEvidence,
    /// Required owner evidence or work remains incomplete.
    IncompleteEvidence,
    /// Supplied artifact identity differs from the selected identity.
    ArtifactMismatch,
    /// Owner strict reader refused the artifact.
    ReaderRefused,
    /// A selected result contradicts the accepted shared prerequisites.
    PrerequisiteContradiction,
    /// A producer boundary failed without an owner result.
    ProducerFailed,
    /// An invoked producer exhausted its declared resource envelope.
    ProducerResourceExhausted,
    /// Work could not begin within its caller-lowered resource envelope.
    ResourceExhausted,
    /// Work could not begin or continue before the operational deadline.
    DeadlineExpired,
    /// An invoked producer returned after the operational deadline.
    ProducerDeadlineExpired,
    /// No invocation was attempted before finalization.
    NotInvoked,
}

impl NoResultCause {
    const fn disposition(self) -> Disposition {
        match self {
            Self::MissingInput => Disposition::Missing,
            Self::IncompatibleSelection => Disposition::Incompatible,
            Self::UnsupportedCapability => Disposition::Unsupported,
            Self::StaleEvidence => Disposition::Stale,
            Self::IncompleteEvidence => Disposition::Incomplete,
            Self::ArtifactMismatch | Self::ReaderRefused | Self::PrerequisiteContradiction => {
                Disposition::Refused
            }
            Self::ProducerFailed
            | Self::ProducerResourceExhausted
            | Self::ProducerDeadlineExpired => Disposition::Failed,
            Self::ResourceExhausted | Self::DeadlineExpired | Self::NotInvoked => {
                Disposition::Unexecuted
            }
        }
    }
}

/// One canonical item result, including explicit no-result states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemResult {
    request: ItemRequest,
    disposition: Disposition,
    owner: Option<OwnerResult>,
    cause: Option<NoResultCause>,
}

impl ItemResult {
    fn owner(request: ItemRequest, owner: OwnerResult) -> Self {
        Self {
            disposition: owner.disposition,
            request,
            owner: Some(owner),
            cause: None,
        }
    }

    fn unavailable(request: ItemRequest, disposition: Disposition, cause: NoResultCause) -> Self {
        Self {
            request,
            disposition,
            owner: None,
            cause: Some(cause),
        }
    }

    /// Item key.
    #[must_use]
    pub const fn key(&self) -> &ItemKey {
        self.request.key()
    }

    /// Original item-local disposition.
    #[must_use]
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Opaque owner bytes, absent only for an explicit no-result state.
    #[must_use]
    pub fn owner_bytes(&self) -> Option<&[u8]> {
        self.owner.as_ref().map(|owner| owner.bytes.as_slice())
    }

    /// Independent owner axes.
    #[must_use]
    pub fn axes(&self) -> Option<&ResultAxes> {
        self.owner.as_ref().map(|owner| &owner.axes)
    }

    /// Explicit no-result cause.
    #[must_use]
    pub const fn cause(&self) -> Option<NoResultCause> {
        self.cause
    }
}

/// Aggregate disposition without sibling relabeling.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BundleDisposition {
    /// Every requested sibling completed.
    Completed,
    /// At least one sibling did not complete and none refused or failed.
    Incomplete,
    /// At least one sibling refused and none failed.
    Refused,
    /// At least one sibling failed.
    Failed,
}

/// Owner strict-reader refusal.
#[derive(Debug)]
pub struct ReaderRefusal {
    resource: bool,
    detail: &'static str,
}

/// Exact lock-created context supplied to the selected owner strict reader.
#[derive(Clone, Copy, Debug)]
pub struct StrictReadContext<'a> {
    artifact: &'a ArtifactId,
    admission: AdmittedCapability<'a>,
    remaining_duration_ms: u64,
}

impl StrictReadContext<'_> {
    /// Exact selected artifact identity whose bytes must be validated.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactId {
        self.artifact
    }

    /// Exact admitted capability that owns the artifact reader.
    #[must_use]
    pub const fn admission(&self) -> AdmittedCapability<'_> {
        self.admission
    }

    /// Caller-lowered remaining operational duration.
    #[must_use]
    pub const fn remaining_duration_ms(&self) -> u64 {
        self.remaining_duration_ms
    }
}

impl ReaderRefusal {
    /// Constructs a malformed owner-artifact refusal.
    #[must_use]
    pub const fn malformed(detail: &'static str) -> Self {
        Self {
            resource: false,
            detail,
        }
    }

    /// Converts a bounded-work refusal.
    #[must_use]
    pub fn resource(error: CommonError) -> Self {
        Self {
            resource: error.code() == ErrorCode::ResourceExhausted,
            detail: error.detail(),
        }
    }
}

/// Producer failure that did not yield owner result bytes.
#[derive(Debug)]
pub struct ProducerFailure {
    resource: bool,
    detail: &'static str,
}

impl ProducerFailure {
    /// Converts a bounded-work refusal.
    #[must_use]
    pub fn resource(error: CommonError) -> Self {
        Self {
            resource: error.code() == ErrorCode::ResourceExhausted,
            detail: error.detail(),
        }
    }

    /// Converts a result-boundary refusal.
    #[must_use]
    pub fn boundary(error: BundleError) -> Self {
        Self {
            resource: error.code == BundleErrorCode::ResourceExhausted,
            detail: error.detail,
        }
    }
}

/// Owner-specific strict reader. Its checked type statically selects the producer input.
pub trait StrictReader {
    /// Owner-checked artifact type.
    type Checked;

    /// Exact lock selection implemented by this reader.
    fn selection(&self) -> &CapabilitySelection;

    /// Validates the exact artifact before producer invocation.
    fn read(
        &self,
        context: StrictReadContext<'_>,
        artifact: &[u8],
        meter: &mut Meter,
    ) -> std::result::Result<Self::Checked, ReaderRefusal>;
}

/// Exact Rust producer selected for one owner-checked artifact type.
pub trait ExecutionProducer {
    /// Owner-checked artifact accepted by this producer.
    type Checked;

    /// Exact lock selection implemented by this producer.
    fn selection(&self) -> &CapabilitySelection;

    /// Executes only with a constructor-private lock admission proof.
    fn evaluate(
        &self,
        permit: &ExecutionPermit<'_>,
        checked: &Self::Checked,
        meter: &mut Meter,
        limits: ExecutionLimits,
    ) -> std::result::Result<OwnerResult, ProducerFailure>;
}

/// Opaque proof that one exact execution producer was selected and admitted.
#[derive(Clone, Copy, Debug)]
pub struct ExecutionPermit<'a> {
    admission: AdmittedCapability<'a>,
    remaining_duration_ms: u64,
}

impl ExecutionPermit<'_> {
    /// Exact admitted capability proof.
    #[must_use]
    pub const fn admission(&self) -> AdmittedCapability<'_> {
        self.admission
    }

    /// Caller-lowered remaining operational duration.
    #[must_use]
    pub const fn remaining_duration_ms(&self) -> u64 {
        self.remaining_duration_ms
    }
}

/// Trusted monotonic source for deterministic execution deadline verification.
pub trait MonotonicClock: Send + Sync {
    /// Elapsed duration from an arbitrary stable origin.
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

fn remaining_duration_ms(
    clock: &dyn MonotonicClock,
    started: Duration,
    limit: Duration,
) -> Result<u64> {
    let elapsed = clock
        .now()
        .checked_sub(started)
        .ok_or_else(|| BundleError::duration("execution clock moved backwards"))?;
    let remaining = limit
        .checked_sub(elapsed)
        .ok_or_else(|| BundleError::duration("execution duration expired"))?;
    let millis = u64::try_from(remaining.as_millis())
        .map_err(|_| BundleError::resource("execution duration is not representable"))?;
    if millis == 0 {
        return Err(BundleError::duration("execution duration expired"));
    }
    Ok(millis)
}

#[derive(Clone, Copy)]
struct ExecutionDeadline<'a> {
    clock: &'a dyn MonotonicClock,
    started: Duration,
    limit: Duration,
}

impl ExecutionDeadline<'_> {
    fn check(self) -> Result<()> {
        remaining_duration_ms(self.clock, self.started, self.limit).map(|_| ())
    }
}

/// Staged coordinator that prevents ambient or substitute dispatch.
pub struct Coordinator<'a> {
    lock: &'a ValidatedLock,
    request: ExecutionRequest,
    prerequisites: SharedPrerequisites,
    limits: ExecutionLimits,
    results: BTreeMap<ItemKey, ItemResult>,
    clock: Arc<dyn MonotonicClock>,
    started: Duration,
    duration_limit: Duration,
}

impl<'a> Coordinator<'a> {
    /// Starts only under a lock that contains the selected package and workflow.
    pub fn new(
        lock: &'a ValidatedLock,
        request: ExecutionRequest,
        prerequisites: SharedPrerequisites,
        limits: ExecutionLimits,
        max_duration_ms: u64,
    ) -> Result<Self> {
        Self::with_monotonic_clock(
            lock,
            request,
            prerequisites,
            limits,
            max_duration_ms,
            Arc::new(SystemClock::new()),
        )
    }

    /// Starts under an exact lock with an injected trusted monotonic clock.
    pub fn with_monotonic_clock(
        lock: &'a ValidatedLock,
        request: ExecutionRequest,
        prerequisites: SharedPrerequisites,
        limits: ExecutionLimits,
        max_duration_ms: u64,
        clock: Arc<dyn MonotonicClock>,
    ) -> Result<Self> {
        if max_duration_ms == 0 {
            return Err(BundleError::duration("zero execution duration"));
        }
        let started = clock.now();
        let duration_limit = Duration::from_millis(max_duration_ms);
        check_count(
            request.items.len(),
            limits.max_items,
            "request item count exceeds execution limit",
        )?;
        remaining_duration_ms(clock.as_ref(), started, duration_limit)?;
        validate_request_limits(&request, &prerequisites, limits)?;
        remaining_duration_ms(clock.as_ref(), started, duration_limit)?;
        if !lock.contains_package(&request.package) || !lock.contains_workflow(&request.workflow) {
            return Err(BundleError::new(
                BundleErrorCode::UnknownItem,
                None,
                "request package or workflow is absent from the exact lock",
            ));
        }
        remaining_duration_ms(clock.as_ref(), started, duration_limit)?;
        for item in &request.items {
            if !lock.contains_definition(&item.key.identity) {
                return Err(BundleError::new(
                    BundleErrorCode::UnknownItem,
                    None,
                    "requested item definition is absent from the exact lock",
                ));
            }
            remaining_duration_ms(clock.as_ref(), started, duration_limit)?;
        }
        Ok(Self {
            lock,
            request,
            prerequisites,
            limits,
            results: BTreeMap::new(),
            clock,
            started,
            duration_limit,
        })
    }

    fn remaining_duration_ms(&self) -> Result<u64> {
        remaining_duration_ms(self.clock.as_ref(), self.started, self.duration_limit)
    }

    /// Strict-reads and dispatches one exact requested item.
    pub fn execute<R, P>(
        &mut self,
        key: &ItemKey,
        artifact: &ArtifactId,
        bytes: &[u8],
        reader: &R,
        producer: &P,
        meter: &mut Meter,
    ) -> Result<()>
    where
        R: StrictReader,
        P: ExecutionProducer<Checked = R::Checked>,
    {
        let selected = self.selected(key)?.clone();
        self.ensure_pending(key)?;
        if self.remaining_duration_ms().is_err() {
            self.insert_unavailable(
                selected,
                Disposition::Unexecuted,
                NoResultCause::DeadlineExpired,
            );
            return Ok(());
        }
        if selected.prerequisites != self.prerequisites {
            self.insert_unavailable(
                selected,
                Disposition::Refused,
                NoResultCause::PrerequisiteContradiction,
            );
            return Ok(());
        }
        if selected.artifact != *artifact {
            self.insert_unavailable(
                selected,
                Disposition::Refused,
                NoResultCause::ArtifactMismatch,
            );
            return Ok(());
        }
        if check_count(
            bytes.len(),
            self.limits.max_artifact_bytes,
            "owner artifact exceeds its strict-reader byte ceiling",
        )
        .is_err()
        {
            self.insert_unavailable(
                selected,
                Disposition::Unexecuted,
                NoResultCause::ResourceExhausted,
            );
            return Ok(());
        }
        let admission = match self.lock.admit_capability(&selected.capability) {
            Ok(admission) => admission,
            Err(_) => {
                self.insert_unavailable(
                    selected,
                    Disposition::Unsupported,
                    NoResultCause::UnsupportedCapability,
                );
                return Ok(());
            }
        };
        if admission.package() != self.request.package.as_str() {
            self.insert_unavailable(
                selected,
                Disposition::Incompatible,
                NoResultCause::IncompatibleSelection,
            );
            return Ok(());
        }
        if reader.selection() != &selected.capability
            || producer.selection() != &selected.capability
        {
            self.insert_unavailable(
                selected,
                Disposition::Unsupported,
                NoResultCause::UnsupportedCapability,
            );
            return Ok(());
        }
        if meter
            .consume_input_bytes(count(bytes.len())?)
            .map_err(BundleError::from_common)
            .is_err()
        {
            self.insert_unavailable(
                selected,
                Disposition::Unexecuted,
                NoResultCause::ResourceExhausted,
            );
            return Ok(());
        }
        let remaining_duration_ms = match self.remaining_duration_ms() {
            Ok(remaining) => remaining,
            Err(_) => {
                self.insert_unavailable(
                    selected,
                    Disposition::Unexecuted,
                    NoResultCause::DeadlineExpired,
                );
                return Ok(());
            }
        };
        let read_result = reader.read(
            StrictReadContext {
                artifact: &selected.artifact,
                admission,
                remaining_duration_ms,
            },
            bytes,
            meter,
        );
        let remaining_duration_ms = match self.remaining_duration_ms() {
            Ok(remaining) => remaining,
            Err(_) => {
                self.insert_unavailable(
                    selected,
                    Disposition::Unexecuted,
                    NoResultCause::DeadlineExpired,
                );
                return Ok(());
            }
        };
        let checked = match read_result {
            Ok(checked) => checked,
            Err(refusal) => {
                let disposition = if refusal.resource {
                    Disposition::Unexecuted
                } else {
                    Disposition::Refused
                };
                let cause = if refusal.resource {
                    NoResultCause::ResourceExhausted
                } else {
                    NoResultCause::ReaderRefused
                };
                let _detail = refusal.detail;
                self.insert_unavailable(selected, disposition, cause);
                return Ok(());
            }
        };
        if meter.invoke_producers(1).is_err() {
            self.insert_unavailable(
                selected,
                Disposition::Unexecuted,
                NoResultCause::ResourceExhausted,
            );
            return Ok(());
        }
        let permit = ExecutionPermit {
            admission,
            remaining_duration_ms,
        };
        let producer_result = producer.evaluate(&permit, &checked, meter, self.limits);
        if self.remaining_duration_ms().is_err() {
            self.insert_unavailable(
                selected,
                Disposition::Failed,
                NoResultCause::ProducerDeadlineExpired,
            );
            return Ok(());
        }
        match producer_result {
            Ok(owner) => {
                if owner.validate(self.limits).is_err() {
                    self.insert_unavailable(
                        selected,
                        Disposition::Failed,
                        NoResultCause::ProducerResourceExhausted,
                    );
                    return Ok(());
                }
                self.results
                    .insert(key.clone(), ItemResult::owner(selected, owner));
            }
            Err(failure) => {
                let disposition = Disposition::Failed;
                let cause = if failure.resource {
                    NoResultCause::ProducerResourceExhausted
                } else {
                    NoResultCause::ProducerFailed
                };
                let _detail = failure.detail;
                self.insert_unavailable(selected, disposition, cause);
            }
        }
        Ok(())
    }

    /// Records an explicit item-local no-result state without invoking a producer.
    pub fn mark_unavailable(
        &mut self,
        key: &ItemKey,
        disposition: Disposition,
        cause: NoResultCause,
    ) -> Result<()> {
        if disposition == Disposition::Completed || disposition != cause.disposition() {
            return Err(BundleError::new(
                BundleErrorCode::InvalidDisposition,
                Some(key.clone()),
                "no-result cause does not own the supplied disposition",
            ));
        }
        let selected = self.selected(key)?.clone();
        self.ensure_pending(key)?;
        if self.remaining_duration_ms().is_err() {
            self.insert_unavailable(
                selected,
                Disposition::Unexecuted,
                NoResultCause::DeadlineExpired,
            );
        } else {
            self.insert_unavailable(selected, disposition, cause);
        }
        Ok(())
    }

    /// Finalizes to either canonical bundle bytes or an attributable composition refusal.
    pub fn finish(mut self) -> Result<CompositionOutcome> {
        if self
            .request
            .items
            .iter()
            .any(|item| item.prerequisites != self.prerequisites)
        {
            return self
                .refuse_shared_prerequisite()
                .map(CompositionOutcome::Refused);
        }
        if self.remaining_duration_ms().is_err() {
            self.fill_unexecuted_with(NoResultCause::DeadlineExpired);
        } else {
            self.fill_unexecuted();
        }
        let deadline_clock = Arc::clone(&self.clock);
        let serialization_deadline = self
            .remaining_duration_ms()
            .ok()
            .map(|_| ExecutionDeadline {
                clock: deadline_clock.as_ref(),
                started: self.started,
                limit: self.duration_limit,
            });
        ExecutionBundle::build(
            self.lock.identity().to_hex(),
            self.request,
            self.prerequisites,
            self.results.into_values().collect(),
            self.limits,
            serialization_deadline,
        )
        .map(|bundle| CompositionOutcome::Bundle(Box::new(bundle)))
    }

    /// Refuses a shared prerequisite contradiction while preserving unaffected evidence.
    pub fn refuse_shared_prerequisite(mut self) -> Result<CompositionRefusal> {
        let affected = self
            .request
            .items
            .iter()
            .filter(|item| item.prerequisites != self.prerequisites)
            .map(|item| item.key.clone())
            .collect::<Vec<_>>();
        check_count(
            affected.len(),
            self.limits.max_items,
            "affected item count exceeds execution limit",
        )?;
        if affected.is_empty() {
            return Err(BundleError::new(
                BundleErrorCode::Malformed,
                None,
                "shared-prerequisite refusal must identify an affected item",
            ));
        }
        for key in &affected {
            let selected = self.selected(key)?.clone();
            self.insert_unavailable(
                selected,
                Disposition::Refused,
                NoResultCause::PrerequisiteContradiction,
            );
        }
        if self.remaining_duration_ms().is_err() {
            self.fill_unexecuted_with(NoResultCause::DeadlineExpired);
        } else {
            self.fill_unexecuted();
        }
        Ok(CompositionRefusal {
            affected,
            items: self.results.into_values().collect(),
        })
    }

    fn selected(&self, key: &ItemKey) -> Result<&ItemRequest> {
        self.request
            .items
            .binary_search_by(|candidate| candidate.key.cmp(key))
            .ok()
            .map(|index| &self.request.items[index])
            .ok_or_else(|| {
                BundleError::new(
                    BundleErrorCode::UnknownItem,
                    Some(key.clone()),
                    "item is absent from the validated request",
                )
            })
    }

    fn ensure_pending(&self, key: &ItemKey) -> Result<()> {
        if self.results.contains_key(key) {
            return Err(BundleError::new(
                BundleErrorCode::DuplicateResult,
                Some(key.clone()),
                "item already has a terminal result",
            ));
        }
        Ok(())
    }

    fn insert_unavailable(
        &mut self,
        request: ItemRequest,
        disposition: Disposition,
        cause: NoResultCause,
    ) {
        self.results.insert(
            request.key.clone(),
            ItemResult::unavailable(request, disposition, cause),
        );
    }

    fn fill_unexecuted(&mut self) {
        self.fill_unexecuted_with(NoResultCause::NotInvoked);
    }

    fn fill_unexecuted_with(&mut self, cause: NoResultCause) {
        for request in self.request.items.clone() {
            if self.results.contains_key(&request.key) {
                continue;
            }
            let cause = if self.remaining_duration_ms().is_err() {
                NoResultCause::DeadlineExpired
            } else {
                cause
            };
            self.results.insert(
                request.key.clone(),
                ItemResult::unavailable(request, Disposition::Unexecuted, cause),
            );
        }
    }
}

/// Shared-prerequisite refusal with exact scope and retained sibling evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRefusal {
    affected: Vec<ItemKey>,
    items: Vec<ItemResult>,
}

/// Total composition finalization outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompositionOutcome {
    /// Canonical executable bundle.
    Bundle(Box<ExecutionBundle>),
    /// Attributable shared-prerequisite refusal with retained sibling evidence.
    Refused(CompositionRefusal),
}

impl CompositionOutcome {
    /// Returns canonical bundle output when composition succeeded.
    #[must_use]
    pub fn into_bundle(self) -> Option<ExecutionBundle> {
        match self {
            Self::Bundle(bundle) => Some(*bundle),
            Self::Refused(_) => None,
        }
    }

    /// Returns the attributable refusal when shared prerequisites contradicted selections.
    #[must_use]
    pub fn into_refusal(self) -> Option<CompositionRefusal> {
        match self {
            Self::Bundle(_) => None,
            Self::Refused(refusal) => Some(refusal),
        }
    }
}

impl CompositionRefusal {
    /// Canonically ordered affected keys.
    #[must_use]
    pub fn affected(&self) -> &[ItemKey] {
        &self.affected
    }

    /// Canonically ordered item evidence, including unaffected completed siblings.
    #[must_use]
    pub fn items(&self) -> &[ItemResult] {
        &self.items
    }
}

/// Immutable canonical execution bundle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionBundle {
    lock_identity: String,
    request: ExecutionRequest,
    prerequisites: SharedPrerequisites,
    items: Vec<ItemResult>,
    disposition: BundleDisposition,
    limits: ExecutionLimits,
    bytes: Vec<u8>,
}

impl ExecutionBundle {
    fn build(
        lock_identity: String,
        request: ExecutionRequest,
        prerequisites: SharedPrerequisites,
        items: Vec<ItemResult>,
        limits: ExecutionLimits,
        deadline: Option<ExecutionDeadline<'_>>,
    ) -> Result<Self> {
        check_count(
            items.len(),
            limits.max_items,
            "bundle item count exceeds limit",
        )?;
        let disposition = aggregate(&items);
        let mut bundle = Self {
            lock_identity,
            request,
            prerequisites,
            items,
            disposition,
            limits,
            bytes: Vec::new(),
        };
        bundle.bytes = serialize_bundle(&bundle, limits.max_output_bytes, deadline)?;
        Ok(bundle)
    }

    /// Canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Lowercase SHA-256 identity of the exact ecosystem lock.
    #[must_use]
    pub fn lock_identity(&self) -> &str {
        &self.lock_identity
    }

    /// Canonical request retained in the bundle.
    #[must_use]
    pub const fn request(&self) -> &ExecutionRequest {
        &self.request
    }

    /// Exact shared prerequisites retained in the bundle.
    #[must_use]
    pub const fn shared_prerequisites(&self) -> &SharedPrerequisites {
        &self.prerequisites
    }

    /// Canonically ordered item results.
    #[must_use]
    pub fn items(&self) -> &[ItemResult] {
        &self.items
    }

    /// Aggregate disposition.
    #[must_use]
    pub const fn disposition(&self) -> BundleDisposition {
        self.disposition
    }

    /// Exact execution limits retained in the request bytes.
    #[must_use]
    pub const fn limits(&self) -> ExecutionLimits {
        self.limits
    }
}

fn aggregate(items: &[ItemResult]) -> BundleDisposition {
    if items
        .iter()
        .any(|item| item.disposition == Disposition::Failed)
    {
        BundleDisposition::Failed
    } else if items
        .iter()
        .any(|item| item.disposition == Disposition::Refused)
    {
        BundleDisposition::Refused
    } else if items
        .iter()
        .all(|item| item.disposition == Disposition::Completed)
    {
        BundleDisposition::Completed
    } else {
        BundleDisposition::Incomplete
    }
}

fn bounded_text(value: &str, maximum: u64, detail: &'static str) -> Result<String> {
    if value.trim().is_empty() {
        return Err(BundleError::new(BundleErrorCode::Malformed, None, detail));
    }
    check_count(value.len(), maximum, detail)?;
    Ok(value.to_owned())
}

fn validate_request_limits(
    request: &ExecutionRequest,
    prerequisites: &SharedPrerequisites,
    limits: ExecutionLimits,
) -> Result<()> {
    let maximum = limits.max_artifact_bytes;
    check_count(
        request.package.as_str().len(),
        maximum,
        "package identity exceeds limit",
    )?;
    check_count(
        request.workflow.as_str().len(),
        maximum,
        "workflow identity exceeds limit",
    )?;
    for item in &request.items {
        for value in [
            item.key.identity.as_str(),
            item.capability.role().as_str(),
            item.capability.provider().as_str(),
            item.capability.build().as_str(),
            item.capability.contract().as_str(),
            item.capability.options().as_str(),
            item.artifact.as_str(),
            item.prerequisites.model.as_str(),
            item.prerequisites.package.as_str(),
            item.prerequisites.clock.as_str(),
            item.prerequisites.population.as_str(),
        ] {
            check_count(value.len(), maximum, "request identity exceeds limit")?;
        }
    }
    for value in [
        prerequisites.model.as_str(),
        prerequisites.package.as_str(),
        prerequisites.clock.as_str(),
        prerequisites.population.as_str(),
    ] {
        check_count(value.len(), maximum, "prerequisite identity exceeds limit")?;
    }
    Ok(())
}

fn count(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| BundleError::resource("length is not representable"))
}

fn check_count(value: usize, maximum: u64, detail: &'static str) -> Result<()> {
    if count(value)? > maximum {
        return Err(BundleError::resource(detail));
    }
    Ok(())
}

const PREFLIGHT_BOUND: &str = "execution bundle preflight bound exceeded";

struct PreflightState {
    exceeded: Cell<bool>,
}

fn preflight_bound<E: serde::de::Error>(state: &PreflightState) -> E {
    state.exceeded.set(true);
    E::custom(PREFLIGHT_BOUND)
}

struct ValuePreflight<'a> {
    maximum: u64,
    state: &'a PreflightState,
}

impl<'de> serde::de::DeserializeSeed<'de> for ValuePreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for ValuePreflight<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON whose strings fit the caller-lowered identity ceiling")
    }

    fn visit_bool<E: serde::de::Error>(self, _value: bool) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_i64<E: serde::de::Error>(self, _value: i64) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_u64<E: serde::de::Error>(self, _value: u64) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_f64<E: serde::de::Error>(self, _value: f64) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<(), E> {
        if u64::try_from(value.len()).map_or(true, |length| length > self.maximum) {
            Err(preflight_bound(self.state))
        } else {
            Ok(())
        }
    }

    fn visit_none<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_some<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<(), A::Error> {
        while sequence
            .next_element_seed(ValuePreflight {
                maximum: self.maximum,
                state: self.state,
            })?
            .is_some()
        {}
        Ok(())
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while map
            .next_key_seed(KeyPreflight {
                maximum: self.maximum,
                state: self.state,
            })?
            .is_some()
        {
            map.next_value_seed(ValuePreflight {
                maximum: self.maximum,
                state: self.state,
            })?;
        }
        Ok(())
    }
}

struct KeyPreflight<'a> {
    maximum: u64,
    state: &'a PreflightState,
}

impl<'de> serde::de::DeserializeSeed<'de> for KeyPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_identifier(self)
    }
}

impl Visitor<'_> for KeyPreflight<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an object field name")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<(), E> {
        if u64::try_from(value.len()).map_or(true, |length| length > self.maximum) {
            Err(preflight_bound(self.state))
        } else {
            Ok(())
        }
    }
}

struct DocumentPreflight<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for DocumentPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(DocumentPreflightVisitor(self.0, self.1))
    }
}

struct DocumentPreflightVisitor<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> Visitor<'de> for DocumentPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an execution-bundle object")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut result_count = 0_u64;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "request" => {
                    map.next_value_seed(RequestPreflight(self.0, self.1))?;
                }
                "sharedPrerequisites" => {
                    map.next_value_seed(PrerequisitesPreflight(self.0.max_artifact_bytes, self.1))?;
                }
                "stateResults" | "temporalResults" | "observationResults" | "protocolResults"
                | "analysisResults" | "backendResults" => {
                    let count = map.next_value_seed(ResultItemsPreflight(self.0, self.1))?;
                    result_count = result_count
                        .checked_add(count)
                        .ok_or_else(|| preflight_bound(self.1))?;
                    if result_count > self.0.max_items {
                        return Err(preflight_bound(self.1));
                    }
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct BoundedTextPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for BoundedTextPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_str(self)
    }
}

impl Visitor<'_> for BoundedTextPreflight<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a caller-bounded string")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<(), E> {
        if u64::try_from(value.len()).map_or(true, |length| length > self.0) {
            Err(preflight_bound(self.1))
        } else {
            Ok(())
        }
    }
}

struct PrerequisitesPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for PrerequisitesPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(PrerequisitesPreflightVisitor(self.0, self.1))
    }
}

struct PrerequisitesPreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for PrerequisitesPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded shared prerequisites")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "model" | "package" | "clock" | "population" => {
                    map.next_value_seed(BoundedTextPreflight(self.0, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct ItemRequestsPreflight<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for ItemRequestsPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(ItemRequestsPreflightVisitor(self.0, self.1))
    }
}

struct ItemRequestsPreflightVisitor<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> Visitor<'de> for ItemRequestsPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded item requests")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<(), A::Error> {
        let mut count = 0_u64;
        while sequence
            .next_element_seed(ItemRequestPreflight(self.0.max_artifact_bytes, self.1))?
            .is_some()
        {
            count = count
                .checked_add(1)
                .ok_or_else(|| preflight_bound(self.1))?;
            if count > self.0.max_items {
                return Err(preflight_bound(self.1));
            }
        }
        Ok(())
    }
}

struct ItemRequestPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for ItemRequestPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(ItemRequestPreflightVisitor(self.0, self.1))
    }
}

struct ItemRequestPreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for ItemRequestPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded item request")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "key" => map.next_value_seed(ItemKeyPreflight(self.0, self.1))?,
                "capability" => map.next_value_seed(CapabilityPreflight(self.0, self.1))?,
                "artifact" => map.next_value_seed(BoundedTextPreflight(self.0, self.1))?,
                "prerequisites" => {
                    map.next_value_seed(PrerequisitesPreflight(self.0, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct ItemKeyPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for ItemKeyPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(ItemKeyPreflightVisitor(self.0, self.1))
    }
}

struct ItemKeyPreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for ItemKeyPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded item key")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            if key == "identity" {
                map.next_value_seed(BoundedTextPreflight(self.0, self.1))?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(())
    }
}

struct CapabilityPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for CapabilityPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(CapabilityPreflightVisitor(self.0, self.1))
    }
}

struct CapabilityPreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for CapabilityPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded capability selection")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "role" | "provider" | "build" | "contract" | "options" => {
                    map.next_value_seed(BoundedTextPreflight(self.0, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct RequestPreflight<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for RequestPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(RequestPreflightVisitor(self.0, self.1))
    }
}

struct RequestPreflightVisitor<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> Visitor<'de> for RequestPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an execution request")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "package" | "workflow" => {
                    map.next_value_seed(BoundedTextPreflight(self.0.max_artifact_bytes, self.1))?;
                }
                "items" => {
                    map.next_value_seed(ItemRequestsPreflight(self.0, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct IgnoredSequencePreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for IgnoredSequencePreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded sequence")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut count = 0_u64;
        while sequence.next_element::<IgnoredAny>()?.is_some() {
            count = count
                .checked_add(1)
                .ok_or_else(|| preflight_bound(self.1))?;
            if count > self.0 {
                return Err(preflight_bound(self.1));
            }
        }
        Ok(())
    }
}

struct ResultItemsPreflight<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for ResultItemsPreflight<'_> {
    type Value = u64;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(ResultItemsPreflightVisitor(self.0, self.1))
    }
}

struct ResultItemsPreflightVisitor<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> Visitor<'de> for ResultItemsPreflightVisitor<'_> {
    type Value = u64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded execution results")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut count = 0_u64;
        while sequence
            .next_element_seed(ResultPreflight(self.0, self.1))?
            .is_some()
        {
            count = count
                .checked_add(1)
                .ok_or_else(|| preflight_bound(self.1))?;
            if count > self.0.max_items {
                return Err(preflight_bound(self.1));
            }
        }
        Ok(count)
    }
}

struct ResultPreflight<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for ResultPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_map(ResultPreflightVisitor(self.0, self.1))
    }
}

struct ResultPreflightVisitor<'a>(ExecutionLimits, &'a PreflightState);

impl<'de> Visitor<'de> for ResultPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an execution result")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "request" => {
                    map.next_value_seed(ItemRequestPreflight(self.0.max_artifact_bytes, self.1))?;
                }
                "ownerBytes" => {
                    map.next_value_seed(OptionalByteSequencePreflight(
                        self.0.max_result_bytes,
                        self.1,
                    ))?;
                }
                "axes" => {
                    map.next_value_seed(OptionalAxesPreflight(self.0.max_artifact_bytes, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct OptionalAxesPreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for OptionalAxesPreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_any(OptionalAxesPreflightVisitor(self.0, self.1))
    }
}

struct OptionalAxesPreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for OptionalAxesPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("null or bounded owner axes")
    }

    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_none<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "truth" | "execution" | "adequacy" | "completeness" | "provenance" => {
                    map.next_value_seed(BoundedTextPreflight(self.0, self.1))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct OptionalByteSequencePreflight<'a>(u64, &'a PreflightState);

impl<'de> serde::de::DeserializeSeed<'de> for OptionalByteSequencePreflight<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_any(OptionalByteSequencePreflightVisitor(self.0, self.1))
    }
}

struct OptionalByteSequencePreflightVisitor<'a>(u64, &'a PreflightState);

impl<'de> Visitor<'de> for OptionalByteSequencePreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("null or a bounded byte sequence")
    }

    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(())
    }

    fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(())
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        IgnoredSequencePreflightVisitor(self.0, self.1).visit_seq(&mut sequence)
    }
}

fn preflight_document(bytes: &[u8], limits: ExecutionLimits) -> Result<()> {
    let state = PreflightState {
        exceeded: Cell::new(false),
    };
    let mut string_deserializer = serde_json::Deserializer::from_slice(bytes);
    ValuePreflight {
        maximum: limits.max_artifact_bytes.max(64),
        state: &state,
    }
    .deserialize(&mut string_deserializer)
    .and_then(|()| string_deserializer.end())
    .map_err(|_| {
        if state.exceeded.get() {
            BundleError::resource("bundle string exceeds its caller-lowered ceiling")
        } else {
            BundleError::new(
                BundleErrorCode::Malformed,
                None,
                "bundle preflight could not traverse the closed document",
            )
        }
    })?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    DocumentPreflight(limits, &state)
        .deserialize(&mut deserializer)
        .and_then(|()| deserializer.end())
        .map_err(|_| {
            if state.exceeded.get() {
                BundleError::resource("bundle collection or owner bytes exceed their ceiling")
            } else {
                BundleError::new(
                    BundleErrorCode::Malformed,
                    None,
                    "bundle preflight could not traverse the closed document",
                )
            }
        })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleWireRef<'a> {
    contract: &'static str,
    lock_identity: &'a str,
    request: RequestWireRef<'a>,
    shared_prerequisites: PrerequisiteWireRef<'a>,
    state_results: DomainResultsRef<'a>,
    temporal_results: DomainResultsRef<'a>,
    observation_results: DomainResultsRef<'a>,
    protocol_results: DomainResultsRef<'a>,
    analysis_results: DomainResultsRef<'a>,
    backend_results: DomainResultsRef<'a>,
    bundle_disposition: BundleDisposition,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestWireRef<'a> {
    package: &'a str,
    workflow: &'a str,
    items: ItemRequestsRef<'a>,
    limits: ExecutionLimits,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrerequisiteWireRef<'a> {
    model: &'a str,
    package: &'a str,
    clock: &'a str,
    population: &'a str,
}

struct ItemRequestsRef<'a>(&'a [ItemRequest]);
struct DomainResultsRef<'a> {
    items: &'a [ItemResult],
    domain: Domain,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyWireRef<'a> {
    domain: Domain,
    identity: &'a str,
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
#[serde(rename_all = "camelCase")]
struct ItemRequestWireRef<'a> {
    key: KeyWireRef<'a>,
    capability: CapabilityWireRef<'a>,
    artifact: &'a str,
    prerequisites: PrerequisiteWireRef<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AxesWireRef<'a> {
    truth: &'a str,
    execution: &'a str,
    adequacy: &'a str,
    completeness: &'a str,
    provenance: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemResultWireRef<'a> {
    request: ItemRequestWireRef<'a>,
    disposition: &'static str,
    owner_bytes: Option<&'a [u8]>,
    axes: Option<AxesWireRef<'a>>,
    no_result_cause: Option<NoResultCause>,
}

fn key_wire(key: &ItemKey) -> KeyWireRef<'_> {
    KeyWireRef {
        domain: key.domain,
        identity: key.identity.as_str(),
    }
}

fn capability_wire(capability: &CapabilitySelection) -> CapabilityWireRef<'_> {
    CapabilityWireRef {
        role: capability.role().as_str(),
        provider: capability.provider().as_str(),
        build: capability.build().as_str(),
        contract: capability.contract().as_str(),
        options: capability.options().as_str(),
    }
}

fn request_wire(item: &ItemRequest) -> ItemRequestWireRef<'_> {
    ItemRequestWireRef {
        key: key_wire(&item.key),
        capability: capability_wire(&item.capability),
        artifact: item.artifact.as_str(),
        prerequisites: PrerequisiteWireRef {
            model: item.prerequisites.model.as_str(),
            package: item.prerequisites.package.as_str(),
            clock: item.prerequisites.clock.as_str(),
            population: item.prerequisites.population.as_str(),
        },
    }
}

impl Serialize for ItemRequestsRef<'_> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            sequence.serialize_element(&request_wire(item))?;
        }
        sequence.end()
    }
}

impl Serialize for DomainResultsRef<'_> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let count = self
            .items
            .iter()
            .filter(|item| item.key().domain() == self.domain)
            .count();
        let mut sequence = serializer.serialize_seq(Some(count))?;
        for item in self
            .items
            .iter()
            .filter(|item| item.key().domain() == self.domain)
        {
            let axes = item.owner.as_ref().map(|owner| AxesWireRef {
                truth: &owner.axes.truth,
                execution: &owner.axes.execution,
                adequacy: &owner.axes.adequacy,
                completeness: &owner.axes.completeness,
                provenance: owner.axes.provenance.as_str(),
            });
            sequence.serialize_element(&ItemResultWireRef {
                request: request_wire(&item.request),
                disposition: item.disposition.as_str(),
                owner_bytes: item.owner.as_ref().map(|owner| owner.bytes.as_slice()),
                axes,
                no_result_cause: item.cause,
            })?;
        }
        sequence.end()
    }
}

fn wire_ref(bundle: &ExecutionBundle) -> BundleWireRef<'_> {
    BundleWireRef {
        contract: CONTRACT,
        lock_identity: &bundle.lock_identity,
        request: RequestWireRef {
            package: bundle.request.package.as_str(),
            workflow: bundle.request.workflow.as_str(),
            items: ItemRequestsRef(&bundle.request.items),
            limits: bundle.limits,
        },
        shared_prerequisites: PrerequisiteWireRef {
            model: bundle.prerequisites.model.as_str(),
            package: bundle.prerequisites.package.as_str(),
            clock: bundle.prerequisites.clock.as_str(),
            population: bundle.prerequisites.population.as_str(),
        },
        state_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::State,
        },
        temporal_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::Temporal,
        },
        observation_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::Observation,
        },
        protocol_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::Protocol,
        },
        analysis_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::Analysis,
        },
        backend_results: DomainResultsRef {
            items: &bundle.items,
            domain: Domain::Backend,
        },
        bundle_disposition: bundle.disposition,
    }
}

struct CountingWriter<'a> {
    consumed: u64,
    maximum: u64,
    deadline: Option<ExecutionDeadline<'a>>,
    deadline_expired: bool,
}

impl std::io::Write for CountingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self
            .deadline
            .is_some_and(|deadline| deadline.check().is_err())
        {
            self.deadline_expired = true;
            return Err(std::io::Error::other("execution duration expired"));
        }
        let incoming = u64::try_from(buffer.len())
            .map_err(|_| std::io::Error::other("bundle length is not representable"))?;
        let next = self
            .consumed
            .checked_add(incoming)
            .ok_or_else(|| std::io::Error::other("bundle length overflowed"))?;
        if next > self.maximum {
            return Err(std::io::Error::other("bundle exceeds output ceiling"));
        }
        self.consumed = next;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct BoundedWriter<'a> {
    bytes: Vec<u8>,
    maximum: u64,
    deadline: Option<ExecutionDeadline<'a>>,
    deadline_expired: bool,
}

impl std::io::Write for BoundedWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self
            .deadline
            .is_some_and(|deadline| deadline.check().is_err())
        {
            self.deadline_expired = true;
            return Err(std::io::Error::other("execution duration expired"));
        }
        let current = u64::try_from(self.bytes.len())
            .map_err(|_| std::io::Error::other("bundle length is not representable"))?;
        let incoming = u64::try_from(buffer.len())
            .map_err(|_| std::io::Error::other("bundle length is not representable"))?;
        let next = current
            .checked_add(incoming)
            .ok_or_else(|| std::io::Error::other("bundle length overflowed"))?;
        if next > self.maximum {
            return Err(std::io::Error::other("bundle exceeds output ceiling"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serialize_bundle(
    bundle: &ExecutionBundle,
    maximum: u64,
    deadline: Option<ExecutionDeadline<'_>>,
) -> Result<Vec<u8>> {
    let mut counter = CountingWriter {
        consumed: 0,
        maximum,
        deadline,
        deadline_expired: false,
    };
    let counted = serde_json::to_writer(&mut counter, &wire_ref(bundle));
    if counter.deadline_expired {
        return Err(BundleError::duration(
            "execution duration expired during output preflight",
        ));
    }
    counted.map_err(|_| BundleError::resource("canonical bundle output exceeds its ceiling"))?;
    let capacity = usize::try_from(counter.consumed)
        .map_err(|_| BundleError::resource("canonical bundle length is not representable"))?;
    let mut writer = BoundedWriter {
        bytes: Vec::with_capacity(capacity),
        maximum: counter.consumed,
        deadline,
        deadline_expired: false,
    };
    let serialized = serde_json::to_writer(&mut writer, &wire_ref(bundle));
    if writer.deadline_expired {
        return Err(BundleError::duration(
            "execution duration expired during output serialization",
        ));
    }
    serialized.map_err(|_| {
        BundleError::new(
            BundleErrorCode::CanonicalMismatch,
            None,
            "bundle serialization changed after preflight",
        )
    })?;
    if writer.bytes.len() != capacity {
        return Err(BundleError::new(
            BundleErrorCode::CanonicalMismatch,
            None,
            "bundle serialization length changed after preflight",
        ));
    }
    Ok(writer.bytes)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BundleWireOwned {
    contract: String,
    lock_identity: String,
    request: RequestWireOwned,
    shared_prerequisites: PrerequisiteWireOwned,
    state_results: Vec<ItemResultWireOwned>,
    temporal_results: Vec<ItemResultWireOwned>,
    observation_results: Vec<ItemResultWireOwned>,
    protocol_results: Vec<ItemResultWireOwned>,
    analysis_results: Vec<ItemResultWireOwned>,
    backend_results: Vec<ItemResultWireOwned>,
    bundle_disposition: BundleDisposition,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RequestWireOwned {
    package: String,
    workflow: String,
    items: Vec<ItemRequestWireOwned>,
    limits: ExecutionLimits,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PrerequisiteWireOwned {
    model: String,
    package: String,
    clock: String,
    population: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct KeyWireOwned {
    domain: Domain,
    identity: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CapabilityWireOwned {
    role: String,
    provider: String,
    build: String,
    contract: String,
    options: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ItemRequestWireOwned {
    key: KeyWireOwned,
    capability: CapabilityWireOwned,
    artifact: String,
    prerequisites: PrerequisiteWireOwned,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AxesWireOwned {
    truth: String,
    execution: String,
    adequacy: String,
    completeness: String,
    provenance: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ItemResultWireOwned {
    request: ItemRequestWireOwned,
    disposition: String,
    owner_bytes: Option<Vec<u8>>,
    axes: Option<AxesWireOwned>,
    no_result_cause: Option<NoResultCause>,
}

/// Strict-reads canonical bundle bytes without access to producers.
pub fn read(bytes: &[u8], limits: ExecutionLimits) -> Result<ExecutionBundle> {
    check_count(
        bytes.len(),
        limits.max_output_bytes,
        "bundle document exceeds its byte ceiling",
    )?;
    preflight_document(bytes, limits)?;
    let wire: BundleWireOwned = serde_json::from_slice(bytes).map_err(|_| {
        BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "bundle document does not match the closed contract",
        )
    })?;
    if wire.contract != CONTRACT || !is_digest(&wire.lock_identity) {
        return Err(BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "bundle contract or lock identity is invalid",
        ));
    }
    let maximum = limits.max_artifact_bytes;
    let request_items = wire
        .request
        .items
        .into_iter()
        .map(|item| parse_request(item, maximum))
        .collect::<Result<Vec<_>>>()?;
    if wire.request.limits != limits {
        return Err(BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "bundle limits differ from the strict reader envelope",
        ));
    }
    let request = ExecutionRequest::new(
        PackageId::new(&wire.request.package, maximum).map_err(BundleError::from_common)?,
        WorkflowInstanceId::new(&wire.request.workflow, maximum)
            .map_err(BundleError::from_common)?,
        request_items,
        limits.max_items,
    )?;
    let prerequisites = SharedPrerequisites::new(
        ArtifactId::new(&wire.shared_prerequisites.model, maximum)
            .map_err(BundleError::from_common)?,
        ArtifactId::new(&wire.shared_prerequisites.package, maximum)
            .map_err(BundleError::from_common)?,
        ArtifactId::new(&wire.shared_prerequisites.clock, maximum)
            .map_err(BundleError::from_common)?,
        ArtifactId::new(&wire.shared_prerequisites.population, maximum)
            .map_err(BundleError::from_common)?,
    );
    if request
        .items
        .iter()
        .any(|item| item.prerequisites != prerequisites)
    {
        return Err(BundleError::new(
            BundleErrorCode::PrerequisiteContradiction,
            None,
            "bundle result selection contradicts its shared prerequisites",
        ));
    }
    let groups = [
        (Domain::State, wire.state_results),
        (Domain::Temporal, wire.temporal_results),
        (Domain::Observation, wire.observation_results),
        (Domain::Protocol, wire.protocol_results),
        (Domain::Analysis, wire.analysis_results),
        (Domain::Backend, wire.backend_results),
    ];
    let total = groups.iter().try_fold(0_u64, |total, (_, group)| {
        total
            .checked_add(count(group.len())?)
            .ok_or_else(|| BundleError::resource("bundle item count overflowed"))
    })?;
    if total > limits.max_items {
        return Err(BundleError::resource("bundle item count exceeds limit"));
    }
    let capacity = usize::try_from(total)
        .map_err(|_| BundleError::resource("bundle item count is not representable"))?;
    let mut items = Vec::with_capacity(capacity);
    for (expected_domain, group) in groups {
        for item in group {
            let parsed = parse_result(item, limits)?;
            if parsed.key().domain() != expected_domain {
                return Err(BundleError::new(
                    BundleErrorCode::Malformed,
                    Some(parsed.key().clone()),
                    "result appears under the wrong domain collection",
                ));
            }
            items.push(parsed);
        }
    }
    if !items.windows(2).all(|pair| pair[0].key() < pair[1].key()) {
        return Err(BundleError::new(
            BundleErrorCode::CanonicalMismatch,
            None,
            "bundle items are not in canonical unique order",
        ));
    }
    let requested = request
        .items
        .iter()
        .map(|item| &item.key)
        .collect::<Vec<_>>();
    let retained = items.iter().map(ItemResult::key).collect::<Vec<_>>();
    if requested != retained {
        return Err(BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "bundle results do not exactly cover the request",
        ));
    }
    for (selected, retained) in request.items.iter().zip(&items) {
        if selected != &retained.request {
            return Err(BundleError::new(
                BundleErrorCode::Malformed,
                Some(selected.key.clone()),
                "bundle result selection differs from its request",
            ));
        }
    }
    let actual = aggregate(&items);
    if wire.bundle_disposition != actual {
        return Err(BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "bundle disposition disagrees with item dispositions",
        ));
    }
    let mut bundle = ExecutionBundle {
        lock_identity: wire.lock_identity,
        request,
        prerequisites,
        items,
        disposition: actual,
        limits,
        bytes: Vec::new(),
    };
    bundle.bytes = serialize_bundle(&bundle, limits.max_output_bytes, None)?;
    if bundle.bytes != bytes {
        return Err(BundleError::new(
            BundleErrorCode::CanonicalMismatch,
            None,
            "bundle bytes are not the canonical encoding",
        ));
    }
    Ok(bundle)
}

fn parse_request(item: ItemRequestWireOwned, maximum: u64) -> Result<ItemRequest> {
    Ok(ItemRequest::new(
        ItemKey::new(
            item.key.domain,
            DefinitionId::new(&item.key.identity, maximum).map_err(BundleError::from_common)?,
        ),
        CapabilitySelection::new(
            RoleId::new(&item.capability.role, maximum).map_err(BundleError::from_common)?,
            ProviderId::new(&item.capability.provider, maximum)
                .map_err(BundleError::from_common)?,
            BuildId::new(&item.capability.build, maximum).map_err(BundleError::from_common)?,
            ContractId::new(&item.capability.contract, maximum)
                .map_err(BundleError::from_common)?,
            ArtifactId::new(&item.capability.options, maximum).map_err(BundleError::from_common)?,
        ),
        ArtifactId::new(&item.artifact, maximum).map_err(BundleError::from_common)?,
        SharedPrerequisites::new(
            ArtifactId::new(&item.prerequisites.model, maximum)
                .map_err(BundleError::from_common)?,
            ArtifactId::new(&item.prerequisites.package, maximum)
                .map_err(BundleError::from_common)?,
            ArtifactId::new(&item.prerequisites.clock, maximum)
                .map_err(BundleError::from_common)?,
            ArtifactId::new(&item.prerequisites.population, maximum)
                .map_err(BundleError::from_common)?,
        ),
    ))
}

fn parse_result(item: ItemResultWireOwned, limits: ExecutionLimits) -> Result<ItemResult> {
    let request = parse_request(item.request, limits.max_artifact_bytes)?;
    let disposition = parse_disposition(&item.disposition)?;
    match (item.owner_bytes, item.axes, item.no_result_cause) {
        (Some(bytes), Some(axes), None) => {
            let axes = ResultAxes::new(
                &axes.truth,
                &axes.execution,
                &axes.adequacy,
                &axes.completeness,
                ArtifactId::new(&axes.provenance, limits.max_artifact_bytes)
                    .map_err(BundleError::from_common)?,
                limits.max_artifact_bytes,
            )?;
            Ok(ItemResult::owner(
                request,
                OwnerResult::new(disposition, &bytes, axes, limits.max_result_bytes)?,
            ))
        }
        (None, None, Some(cause)) if disposition == cause.disposition() => {
            Ok(ItemResult::unavailable(request, disposition, cause))
        }
        _ => Err(BundleError::new(
            BundleErrorCode::Malformed,
            Some(request.key.clone()),
            "owner bytes/axes and no-result cause are inconsistent",
        )),
    }
}

fn parse_disposition(value: &str) -> Result<Disposition> {
    match value {
        "completed" => Ok(Disposition::Completed),
        "missing" => Ok(Disposition::Missing),
        "incompatible" => Ok(Disposition::Incompatible),
        "unsupported" => Ok(Disposition::Unsupported),
        "stale" => Ok(Disposition::Stale),
        "incomplete" => Ok(Disposition::Incomplete),
        "refused" => Ok(Disposition::Refused),
        "failed" => Ok(Disposition::Failed),
        "unexecuted" => Ok(Disposition::Unexecuted),
        _ => Err(BundleError::new(
            BundleErrorCode::Malformed,
            None,
            "unknown item disposition",
        )),
    }
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
