// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![allow(
    dead_code,
    reason = "the subprocess and parent scenarios use different helpers"
)]

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use fs2::FileExt as _;
use quire_integration::ecosystem_lock::{
    validate, CapabilityBinding, CapabilitySelection, ComponentBinding, Limits as LockLimits,
    LockIdentity, LockSelection, PackageBinding, Pin, PinState, RepositoryBinding, RoleBinding,
    WorkflowBinding,
};
use quire_integration::effect_reconciliation::{
    ActionId, ActorId, AdapterError, AdapterManifest, AttemptId, AuthorityGrant, AuthorityGrantId,
    AuthorityScope, AuthorityStatus, AuthorityValidationError, AuthorizedEffectAdapter, CallBudget,
    Decision, DecisionAuthority, DecisionId, DeliveryUnknownCause, DispatchPermit, DispatchResult,
    DuplicateGuarantee, EffectRequest, EffectStore, Event, ExternalEffectId, IdempotencyKey,
    Limits, Observation, ObservationAuthority, ObservationError, ObservationId, OperationId,
    OwnerArtifact, PersistedEffectRequest, Receipt, ReceiptId, Recommendation, RecommendationId,
    ReconciliationDisposition, ReconciliationPermit, ReconciliationResult, ReconciliationSupport,
    Record, Reservation, ScopeId, State, StoreError, SubjectId, Submission,
};
use quire_integration::{
    ArtifactId, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, PackageId,
    ProviderId, RepositoryId, RoleId, ToolchainId, WorkflowInstanceId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub const ID_LIMIT: u64 = 256;
pub const CRASH_AFTER_RESERVATION: i32 = 81;
pub const CRASH_AFTER_ATTEMPT: i32 = 82;
pub const CRASH_AFTER_ACCEPTANCE: i32 = 83;
pub const ADAPTER_CONTRACT: &[u8] = include_bytes!("../fixtures/it002-effect-adapter-v1.json");
pub const UNSUPPORTED_ADAPTER_CONTRACT: &[u8] =
    include_bytes!("../fixtures/it002-effect-adapter-unsupported-v1.json");

trait FixtureIdentity: Sized {
    fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self>;
}

macro_rules! fixture_identities {
    ($($identity:ty),+ $(,)?) => {$ (
        impl FixtureIdentity for $identity {
            fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self> {
                Self::new(value, maximum)
            }
        }
    )+ };
}

fixture_identities!(
    ArtifactId,
    BuildId,
    ComponentId,
    ContractId,
    DefinitionId,
    DeploymentId,
    PackageId,
    ProviderId,
    RepositoryId,
    RoleId,
    ToolchainId,
    WorkflowInstanceId,
);

fn id<T: FixtureIdentity>(value: &str) -> T {
    T::fixture_new(value, ID_LIMIT).expect("bounded IT-002 fixture identity")
}

macro_rules! effect_id {
    ($identity:ty, $value:expr) => {
        <$identity>::new($value, ID_LIMIT).expect("bounded IT-002 effect identity")
    };
}

fn stored_id<T: FixtureIdentity>(value: &str) -> std::result::Result<T, StoreError> {
    T::fixture_new(value, ID_LIMIT).map_err(|_| StoreError::Unavailable)
}

macro_rules! stored_effect_id {
    ($identity:ty, $value:expr) => {
        <$identity>::new($value, ID_LIMIT).map_err(|_| StoreError::Unavailable)?
    };
}

fn revision(seed: char) -> String {
    format!("git:{}", seed.to_string().repeat(40))
}

fn exact<T: quire_integration::ecosystem_lock::PinIdentity>(identity: T, seed: char) -> Pin<T> {
    Pin::new(identity, &revision(seed), PinState::Exact, ID_LIMIT).expect("exact IT-002 pin")
}

pub fn adapter_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("effect-adapter"),
        id::<ProviderId>("loopback-http"),
        id::<BuildId>("loopback-http-build"),
        id::<ContractId>("quire.effect-adapter/v1"),
        id::<ArtifactId>("sha256:it002-effect-adapter-v1"),
    )
}

pub fn observation_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("observation-authority"),
        id::<ProviderId>("qobs-local-authority"),
        id::<BuildId>("qobs-local-build"),
        id::<ContractId>("quire.observation-authority/v1"),
        id::<ArtifactId>("qobs-owner-options"),
    )
}

pub fn decision_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("decision-authority"),
        id::<ProviderId>("local-decision-authority"),
        id::<BuildId>("local-decision-build"),
        id::<ContractId>("quire.human-decision/v1"),
        id::<ArtifactId>("decision-owner-options"),
    )
}

pub fn grant_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("grant-authority"),
        id::<ProviderId>("local-grant-authority"),
        id::<BuildId>("local-grant-build"),
        id::<ContractId>("quire.effect-decision/v1"),
        id::<ArtifactId>("grant-owner-options"),
    )
}

pub fn validated_lock() -> quire_integration::ecosystem_lock::ValidatedLock {
    let repository = id::<RepositoryId>("agent-ix/quire-integration-it002");
    let package = id::<PackageId>("quire-integration-it002");
    let component = id::<ComponentId>("durable-effect-fixture");
    let roles = [
        ("effect-adapter", '7'),
        ("observation-authority", '8'),
        ("decision-authority", '9'),
        ("grant-authority", '0'),
    ];
    let selection = LockSelection::new(
        vec![RepositoryBinding::new(
            exact(repository.clone(), 'a'),
            "https://github.com/agent-ix/quire-integration",
            ID_LIMIT,
        )
        .expect("repository binding")],
        vec![PackageBinding::new(
            exact(package.clone(), 'b'),
            "0.1.0",
            repository.clone(),
            &revision('a'),
            &["it002-loopback"],
            1,
            ID_LIMIT,
        )
        .expect("package binding")],
        vec![exact(id::<DefinitionId>("timed-refund"), 'c')],
        vec![
            exact(id::<ProviderId>("loopback-http"), 'd'),
            exact(id::<ProviderId>("qobs-local-authority"), 'e'),
            exact(id::<ProviderId>("local-decision-authority"), 'f'),
            exact(id::<ProviderId>("local-grant-authority"), '1'),
        ],
        vec![exact(id::<ToolchainId>("rust-1.98-or-later"), '2')],
        vec![
            exact(id::<BuildId>("loopback-http-build"), '3'),
            exact(id::<BuildId>("qobs-local-build"), '4'),
            exact(id::<BuildId>("local-decision-build"), '5'),
            exact(id::<BuildId>("local-grant-build"), '6'),
        ],
        vec![exact(id::<DeploymentId>("local-loopback"), '7')],
        vec![ComponentBinding::new(
            exact(component.clone(), '8'),
            repository,
            package,
        )],
        roles
            .into_iter()
            .map(|(role, seed)| {
                RoleBinding::new(exact(id::<RoleId>(role), seed), component.clone())
            })
            .collect(),
        vec![
            CapabilityBinding::new(adapter_selection()),
            CapabilityBinding::new(observation_selection()),
            CapabilityBinding::new(decision_selection()),
            CapabilityBinding::new(grant_selection()),
        ],
        vec![WorkflowBinding::new(
            exact(id::<WorkflowInstanceId>("refund:it002"), '9'),
            id::<DefinitionId>("timed-refund"),
        )],
        vec![],
    );
    let limits =
        LockLimits::for_selection(&selection, 64 * 1024, ID_LIMIT, 10_000).expect("lock limits");
    validate(&selection, limits).expect("validated IT-002 lock")
}

pub fn limits() -> Limits {
    Limits {
        max_identity_bytes: ID_LIMIT,
        max_argument_bytes: 1024,
        max_request_bytes: 8192,
        max_history_events: 32,
        max_attempts: 4,
        max_receipt_bytes: 2048,
        max_observation_bytes: 2048,
        max_owner_artifact_bytes: 2048,
        max_execution_duration_ms: 10_000,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestMode {
    Safe,
    Unsupported,
}

pub fn manifest(mode: ManifestMode) -> AdapterManifest {
    AdapterManifest {
        selection: adapter_selection(),
        idempotency_field: "Idempotency-Key".to_owned(),
        duplicate_guarantee: match mode {
            ManifestMode::Safe => DuplicateGuarantee::SameKeyAndBytesSameEffect,
            ManifestMode::Unsupported => DuplicateGuarantee::None,
        },
        reconciliation: match mode {
            ManifestMode::Safe => {
                ReconciliationSupport::Operation(effect_id!(OperationId, "refund.lookup"))
            }
            ManifestMode::Unsupported => ReconciliationSupport::None,
        },
    }
}

pub fn submission(mode: ManifestMode, key: &str, amount: u64) -> Submission {
    let canonical_arguments = format!("{{\"amount\":{amount},\"currency\":\"USD\"}}").into_bytes();
    let recommendation = Recommendation {
        id: effect_id!(RecommendationId, format!("recommendation:refund:{amount}")),
        digest: digest(format!("recommendation:refund:{amount}").as_bytes()),
        action: effect_id!(ActionId, "refund"),
        subject: effect_id!(SubjectId, "order:42"),
        scope: effect_id!(ScopeId, "workflow:refund:42"),
    };
    let grant_bytes = b"quire.effect-decision/v1:grant:refund:42:exact".to_vec();
    let authority = AuthorityGrant {
        id: effect_id!(AuthorityGrantId, digest(&grant_bytes)),
        artifact: OwnerArtifact {
            contract_version: "quire.effect-decision/v1".to_owned(),
            canonical_bytes: grant_bytes,
            attestation: b"it002-grant-attestation".to_vec(),
        },
        issuer: grant_selection(),
        actor: effect_id!(ActorId, "operator:alice"),
        action: recommendation.action.clone(),
        subject: recommendation.subject.clone(),
        scope: AuthorityScope::Exact(recommendation.scope.clone()),
        not_before_ms: 10,
        expires_at_ms: 100,
    };
    let operation = effect_id!(OperationId, "refund.create");
    let decision_bytes = format!("quire.human-decision/v1:accepted:refund:{amount}").into_bytes();
    let decision = Decision {
        id: effect_id!(DecisionId, digest(&decision_bytes)),
        artifact: OwnerArtifact {
            contract_version: "quire.human-decision/v1".to_owned(),
            canonical_bytes: decision_bytes,
            attestation: b"it002-decision-attestation".to_vec(),
        },
        recommendation_id: recommendation.id.clone(),
        recommendation_digest: recommendation.digest.clone(),
        authority_grant_id: authority.id.clone(),
        authority: decision_selection(),
        actor: authority.actor.clone(),
        action: recommendation.action.clone(),
        subject: recommendation.subject.clone(),
        scope: recommendation.scope.clone(),
        adapter: adapter_selection(),
        operation: operation.clone(),
        argument_digest: digest(&canonical_arguments),
        supersedes: None,
    };
    Submission {
        recommendation,
        decision,
        authority: Some(authority),
        manifest: manifest(mode),
        key: effect_id!(IdempotencyKey, key),
        operation,
        canonical_arguments,
        supersedes_request: None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrashPoint {
    None,
    AfterReservation,
    AfterAttempt,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum DurableEvent {
    RequestPersisted,
    AttemptPrepared {
        attempt: String,
    },
    DispatchNotAccepted {
        attempt: String,
    },
    Reconciliation {
        attempt: String,
        disposition: DurableReconciliation,
    },
    Receipt {
        receipt: DurableReceipt,
    },
    DeliveryUnknown {
        attempt: String,
        cause: DurableUnknownCause,
    },
    Observation {
        observation: DurableObservation,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DurableReconciliation {
    NotDelivered,
    Inconclusive,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DurableUnknownCause {
    IndeterminateTransport,
    AdapterFailure,
    InvalidReceipt,
    InconclusiveReconciliation,
    ReconciliationUnavailable,
    ReconciliationContractViolation,
    PostBoundaryPersistenceFailure,
    ReceiptPersistenceFailure,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableReceipt {
    id: String,
    request_id: String,
    external_effect_id: String,
    payload: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableObservation {
    id: String,
    request_id: String,
    external_effect_id: Option<String>,
    authority: DurableCapability,
    payload: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DurableRecord {
    pub key: String,
    pub canonical_digest: String,
    pub revision: u64,
    request: DurableRequest,
    events: Vec<DurableEvent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableRequest {
    lock_identity: String,
    id: String,
    recommendation: DurableRecommendation,
    decision: DurableDecision,
    authority: DurableGrant,
    key: String,
    selection: DurableCapability,
    authority_grant_id: String,
    operation: String,
    canonical_arguments: Vec<u8>,
    supersedes: Option<String>,
    canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableRecommendation {
    id: String,
    digest: String,
    action: String,
    subject: String,
    scope: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableOwnerArtifact {
    contract_version: String,
    canonical_bytes: Vec<u8>,
    attestation: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "identity",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum DurableScope {
    Exact(String),
    All,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableGrant {
    id: String,
    artifact: DurableOwnerArtifact,
    issuer: DurableCapability,
    actor: String,
    action: String,
    subject: String,
    scope: DurableScope,
    not_before_ms: u64,
    expires_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableDecision {
    id: String,
    artifact: DurableOwnerArtifact,
    recommendation_id: String,
    recommendation_digest: String,
    authority_grant_id: String,
    authority: DurableCapability,
    actor: String,
    action: String,
    subject: String,
    scope: String,
    adapter: DurableCapability,
    operation: String,
    argument_digest: String,
    supersedes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableCapability {
    role: String,
    provider: String,
    build: String,
    contract: String,
    options: String,
}

impl DurableRecord {
    pub fn event_kinds(&self) -> Vec<&'static str> {
        self.events
            .iter()
            .map(|event| match event {
                DurableEvent::RequestPersisted => "request_persisted",
                DurableEvent::AttemptPrepared { .. } => "attempt_prepared",
                DurableEvent::DispatchNotAccepted { .. } => "dispatch_not_accepted",
                DurableEvent::Reconciliation { .. } => "reconciliation",
                DurableEvent::Receipt { .. } => "receipt",
                DurableEvent::DeliveryUnknown { .. } => "delivery_unknown",
                DurableEvent::Observation { .. } => "observation",
            })
            .collect()
    }

    pub fn receipt_external_effect_id(&self) -> Option<&str> {
        self.events.iter().rev().find_map(|event| match event {
            DurableEvent::Receipt { receipt } => Some(receipt.external_effect_id.as_str()),
            _ => None,
        })
    }
}

fn reconciliation_to_durable(value: ReconciliationDisposition) -> DurableReconciliation {
    match value {
        ReconciliationDisposition::NotDelivered => DurableReconciliation::NotDelivered,
        ReconciliationDisposition::Inconclusive => DurableReconciliation::Inconclusive,
        ReconciliationDisposition::Unavailable => DurableReconciliation::Unavailable,
    }
}

fn reconciliation_from_durable(value: DurableReconciliation) -> ReconciliationDisposition {
    match value {
        DurableReconciliation::NotDelivered => ReconciliationDisposition::NotDelivered,
        DurableReconciliation::Inconclusive => ReconciliationDisposition::Inconclusive,
        DurableReconciliation::Unavailable => ReconciliationDisposition::Unavailable,
    }
}

fn cause_to_durable(value: DeliveryUnknownCause) -> DurableUnknownCause {
    match value {
        DeliveryUnknownCause::IndeterminateTransport => DurableUnknownCause::IndeterminateTransport,
        DeliveryUnknownCause::AdapterFailure => DurableUnknownCause::AdapterFailure,
        DeliveryUnknownCause::InvalidReceipt => DurableUnknownCause::InvalidReceipt,
        DeliveryUnknownCause::InconclusiveReconciliation => {
            DurableUnknownCause::InconclusiveReconciliation
        }
        DeliveryUnknownCause::ReconciliationUnavailable => {
            DurableUnknownCause::ReconciliationUnavailable
        }
        DeliveryUnknownCause::ReconciliationContractViolation => {
            DurableUnknownCause::ReconciliationContractViolation
        }
        DeliveryUnknownCause::PostBoundaryPersistenceFailure => {
            DurableUnknownCause::PostBoundaryPersistenceFailure
        }
        DeliveryUnknownCause::ReceiptPersistenceFailure => {
            DurableUnknownCause::ReceiptPersistenceFailure
        }
    }
}

fn cause_from_durable(value: DurableUnknownCause) -> DeliveryUnknownCause {
    match value {
        DurableUnknownCause::IndeterminateTransport => DeliveryUnknownCause::IndeterminateTransport,
        DurableUnknownCause::AdapterFailure => DeliveryUnknownCause::AdapterFailure,
        DurableUnknownCause::InvalidReceipt => DeliveryUnknownCause::InvalidReceipt,
        DurableUnknownCause::InconclusiveReconciliation => {
            DeliveryUnknownCause::InconclusiveReconciliation
        }
        DurableUnknownCause::ReconciliationUnavailable => {
            DeliveryUnknownCause::ReconciliationUnavailable
        }
        DurableUnknownCause::ReconciliationContractViolation => {
            DeliveryUnknownCause::ReconciliationContractViolation
        }
        DurableUnknownCause::PostBoundaryPersistenceFailure => {
            DeliveryUnknownCause::PostBoundaryPersistenceFailure
        }
        DurableUnknownCause::ReceiptPersistenceFailure => {
            DeliveryUnknownCause::ReceiptPersistenceFailure
        }
    }
}

fn durable_receipt(receipt: &Receipt) -> DurableReceipt {
    DurableReceipt {
        id: receipt.id.as_str().to_owned(),
        request_id: receipt.request_id.as_str().to_owned(),
        external_effect_id: receipt.external_effect_id.as_str().to_owned(),
        payload: receipt.payload.clone(),
    }
}

fn receipt_from_durable(
    value: &DurableReceipt,
    selection: CapabilitySelection,
) -> std::result::Result<Receipt, StoreError> {
    Ok(Receipt {
        id: stored_effect_id!(ReceiptId, &value.id),
        request_id: stored_effect_id!(
            quire_integration::effect_reconciliation::EffectRequestId,
            &value.request_id
        ),
        selection,
        external_effect_id: stored_effect_id!(ExternalEffectId, &value.external_effect_id),
        payload: value.payload.clone(),
    })
}

fn durable_observation(observation: &Observation) -> DurableObservation {
    DurableObservation {
        id: observation.id.as_str().to_owned(),
        request_id: observation.request_id.as_str().to_owned(),
        external_effect_id: observation
            .external_effect_id
            .as_ref()
            .map(|identity| identity.as_str().to_owned()),
        authority: durable_capability(&observation.authority),
        payload: observation.payload.clone(),
    }
}

fn observation_from_durable(
    value: &DurableObservation,
) -> std::result::Result<Observation, StoreError> {
    Ok(Observation {
        id: stored_effect_id!(ObservationId, &value.id),
        request_id: stored_effect_id!(
            quire_integration::effect_reconciliation::EffectRequestId,
            &value.request_id
        ),
        external_effect_id: value
            .external_effect_id
            .as_deref()
            .map(|identity| ExternalEffectId::new(identity, ID_LIMIT))
            .transpose()
            .map_err(|_| StoreError::Unavailable)?,
        authority: capability_from_durable(&value.authority)?,
        payload: value.payload.clone(),
    })
}

fn durable_capability(value: &CapabilitySelection) -> DurableCapability {
    DurableCapability {
        role: value.role().as_str().to_owned(),
        provider: value.provider().as_str().to_owned(),
        build: value.build().as_str().to_owned(),
        contract: value.contract().as_str().to_owned(),
        options: value.options().as_str().to_owned(),
    }
}

fn capability_from_durable(
    value: &DurableCapability,
) -> std::result::Result<CapabilitySelection, StoreError> {
    Ok(CapabilitySelection::new(
        stored_id::<RoleId>(&value.role)?,
        stored_id::<ProviderId>(&value.provider)?,
        stored_id::<BuildId>(&value.build)?,
        stored_id::<ContractId>(&value.contract)?,
        stored_id::<ArtifactId>(&value.options)?,
    ))
}

fn durable_artifact(value: &OwnerArtifact) -> DurableOwnerArtifact {
    DurableOwnerArtifact {
        contract_version: value.contract_version.clone(),
        canonical_bytes: value.canonical_bytes.clone(),
        attestation: value.attestation.clone(),
    }
}

fn artifact_from_durable(value: &DurableOwnerArtifact) -> OwnerArtifact {
    OwnerArtifact {
        contract_version: value.contract_version.clone(),
        canonical_bytes: value.canonical_bytes.clone(),
        attestation: value.attestation.clone(),
    }
}

fn durable_request(value: &EffectRequest) -> DurableRequest {
    let recommendation = value.recommendation();
    let decision = value.decision();
    let authority = value.authority();
    DurableRequest {
        lock_identity: value.lock_identity().to_hex(),
        id: value.id().as_str().to_owned(),
        recommendation: DurableRecommendation {
            id: recommendation.id.as_str().to_owned(),
            digest: recommendation.digest.clone(),
            action: recommendation.action.as_str().to_owned(),
            subject: recommendation.subject.as_str().to_owned(),
            scope: recommendation.scope.as_str().to_owned(),
        },
        decision: DurableDecision {
            id: decision.id.as_str().to_owned(),
            artifact: durable_artifact(&decision.artifact),
            recommendation_id: decision.recommendation_id.as_str().to_owned(),
            recommendation_digest: decision.recommendation_digest.clone(),
            authority_grant_id: decision.authority_grant_id.as_str().to_owned(),
            authority: durable_capability(&decision.authority),
            actor: decision.actor.as_str().to_owned(),
            action: decision.action.as_str().to_owned(),
            subject: decision.subject.as_str().to_owned(),
            scope: decision.scope.as_str().to_owned(),
            adapter: durable_capability(&decision.adapter),
            operation: decision.operation.as_str().to_owned(),
            argument_digest: decision.argument_digest.clone(),
            supersedes: decision
                .supersedes
                .as_ref()
                .map(|identity| identity.as_str().to_owned()),
        },
        authority: DurableGrant {
            id: authority.id.as_str().to_owned(),
            artifact: durable_artifact(&authority.artifact),
            issuer: durable_capability(&authority.issuer),
            actor: authority.actor.as_str().to_owned(),
            action: authority.action.as_str().to_owned(),
            subject: authority.subject.as_str().to_owned(),
            scope: match &authority.scope {
                AuthorityScope::Exact(scope) => DurableScope::Exact(scope.as_str().to_owned()),
                AuthorityScope::All => DurableScope::All,
            },
            not_before_ms: authority.not_before_ms,
            expires_at_ms: authority.expires_at_ms,
        },
        key: value.key().as_str().to_owned(),
        selection: durable_capability(value.selection()),
        authority_grant_id: value.authority_grant_id().as_str().to_owned(),
        operation: value.operation().as_str().to_owned(),
        canonical_arguments: value.canonical_arguments().to_vec(),
        supersedes: value
            .supersedes()
            .map(|identity| identity.as_str().to_owned()),
        canonical_bytes: value.canonical_bytes().to_vec(),
    }
}

fn request_from_durable(
    value: &DurableRequest,
    lock_identity: LockIdentity,
) -> std::result::Result<EffectRequest, StoreError> {
    if value.lock_identity != lock_identity.to_hex() {
        return Err(StoreError::Conflict);
    }
    let recommendation = Recommendation {
        id: stored_effect_id!(RecommendationId, &value.recommendation.id),
        digest: value.recommendation.digest.clone(),
        action: stored_effect_id!(ActionId, &value.recommendation.action),
        subject: stored_effect_id!(SubjectId, &value.recommendation.subject),
        scope: stored_effect_id!(ScopeId, &value.recommendation.scope),
    };
    let authority = AuthorityGrant {
        id: stored_effect_id!(AuthorityGrantId, &value.authority.id),
        artifact: artifact_from_durable(&value.authority.artifact),
        issuer: capability_from_durable(&value.authority.issuer)?,
        actor: stored_effect_id!(ActorId, &value.authority.actor),
        action: stored_effect_id!(ActionId, &value.authority.action),
        subject: stored_effect_id!(SubjectId, &value.authority.subject),
        scope: match &value.authority.scope {
            DurableScope::Exact(scope) => AuthorityScope::Exact(stored_effect_id!(ScopeId, scope)),
            DurableScope::All => AuthorityScope::All,
        },
        not_before_ms: value.authority.not_before_ms,
        expires_at_ms: value.authority.expires_at_ms,
    };
    let decision = Decision {
        id: stored_effect_id!(DecisionId, &value.decision.id),
        artifact: artifact_from_durable(&value.decision.artifact),
        recommendation_id: stored_effect_id!(RecommendationId, &value.decision.recommendation_id),
        recommendation_digest: value.decision.recommendation_digest.clone(),
        authority_grant_id: stored_effect_id!(AuthorityGrantId, &value.decision.authority_grant_id),
        authority: capability_from_durable(&value.decision.authority)?,
        actor: stored_effect_id!(ActorId, &value.decision.actor),
        action: stored_effect_id!(ActionId, &value.decision.action),
        subject: stored_effect_id!(SubjectId, &value.decision.subject),
        scope: stored_effect_id!(ScopeId, &value.decision.scope),
        adapter: capability_from_durable(&value.decision.adapter)?,
        operation: stored_effect_id!(OperationId, &value.decision.operation),
        argument_digest: value.decision.argument_digest.clone(),
        supersedes: value
            .decision
            .supersedes
            .as_deref()
            .map(|identity| DecisionId::new(identity, ID_LIMIT))
            .transpose()
            .map_err(|_| StoreError::Unavailable)?,
    };
    EffectRequest::from_persisted(
        PersistedEffectRequest {
            id: stored_effect_id!(
                quire_integration::effect_reconciliation::EffectRequestId,
                &value.id
            ),
            lock_identity,
            recommendation,
            decision,
            authority,
            key: stored_effect_id!(IdempotencyKey, &value.key),
            selection: capability_from_durable(&value.selection)?,
            authority_grant_id: stored_effect_id!(AuthorityGrantId, &value.authority_grant_id),
            operation: stored_effect_id!(OperationId, &value.operation),
            canonical_arguments: value.canonical_arguments.clone(),
            supersedes: value
                .supersedes
                .as_deref()
                .map(|identity| {
                    quire_integration::effect_reconciliation::EffectRequestId::new(
                        identity, ID_LIMIT,
                    )
                })
                .transpose()
                .map_err(|_| StoreError::Unavailable)?,
            canonical_bytes: value.canonical_bytes.clone(),
        },
        limits(),
    )
    .map_err(|_| StoreError::Unavailable)
}

pub struct FileStore {
    root: PathBuf,
    lock_identity: LockIdentity,
    record: Option<Record>,
    crash: CrashPoint,
    operation_lock: Option<StoreLock>,
    reservation_barrier: Option<Arc<std::sync::Barrier>>,
}

struct StoreLock(File);

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.0);
    }
}

static STORE_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

impl FileStore {
    pub fn new(root: impl Into<PathBuf>, crash: CrashPoint, lock_identity: LockIdentity) -> Self {
        Self {
            root: root.into(),
            lock_identity,
            record: None,
            crash,
            operation_lock: None,
            reservation_barrier: None,
        }
    }

    pub fn with_reservation_barrier(mut self, barrier: Arc<std::sync::Barrier>) -> Self {
        self.reservation_barrier = Some(barrier);
        self
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("store.lock")
    }

    fn digest(request: &EffectRequest) -> String {
        digest(request.canonical_bytes())
    }

    fn journal_byte_limit(limits: Limits) -> std::result::Result<u64, StoreError> {
        let event_bytes = limits
            .max_identity_bytes
            .checked_mul(6)
            .and_then(|value| value.checked_add(limits.max_receipt_bytes))
            .and_then(|value| value.checked_add(limits.max_observation_bytes))
            .and_then(|value| value.checked_add(1024))
            .ok_or(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::RequestBytes,
            ))?;
        limits
            .max_history_events
            .checked_mul(event_bytes)
            .and_then(|value| value.checked_add(limits.max_request_bytes))
            .and_then(|value| value.checked_add(4096))
            .ok_or(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::RequestBytes,
            ))
    }

    fn acquire_lock(&self, limits: Limits) -> std::result::Result<StoreLock, StoreError> {
        if limits.max_execution_duration_ms == 0 {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::ExecutionDuration,
            ));
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())
            .map_err(|_| StoreError::Unavailable)?;
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(limits.max_execution_duration_ms))
            .ok_or(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::ExecutionDuration,
            ))?;
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(StoreLock(file)),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(StoreError::Exhausted(
                            quire_integration::effect_reconciliation::Resource::ExecutionDuration,
                        ));
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Err(_) => return Err(StoreError::Unavailable),
            }
        }
    }

    fn read_durable(&self, limits: Limits) -> std::result::Result<DurableRecord, StoreError> {
        let metadata = fs::metadata(self.state_path()).map_err(|_| StoreError::Unavailable)?;
        if metadata.len() > Self::journal_byte_limit(limits)? {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::RequestBytes,
            ));
        }
        let bytes = fs::read(self.state_path()).map_err(|_| StoreError::Unavailable)?;
        serde_json::from_slice(&bytes).map_err(|_| StoreError::Unavailable)
    }

    fn save_durable(
        &self,
        durable: &DurableRecord,
        limits: Limits,
    ) -> std::result::Result<(), StoreError> {
        let bytes = serde_json::to_vec(durable).map_err(|_| StoreError::Unavailable)?;
        if u64::try_from(bytes.len()).map_err(|_| StoreError::Unavailable)?
            > Self::journal_byte_limit(limits)?
        {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::RequestBytes,
            ));
        }
        let sequence = STORE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = self
            .root
            .join(format!("state.{}.{sequence}.tmp", process::id()));
        {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temporary)
                .map_err(|_| StoreError::Unavailable)?;
            file.write_all(&bytes)
                .map_err(|_| StoreError::Unavailable)?;
            file.sync_all().map_err(|_| StoreError::Unavailable)?;
        }
        fs::rename(&temporary, self.state_path()).map_err(|_| StoreError::Unavailable)?;
        File::open(&self.root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| StoreError::Unavailable)
    }

    fn decode(
        &self,
        request: &EffectRequest,
        durable: &DurableRecord,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        if durable.key != request.key().as_str()
            || durable.canonical_digest != Self::digest(request)
        {
            return Err(StoreError::Conflict);
        }
        if u64::try_from(durable.events.len()).map_err(|_| StoreError::Unavailable)?
            > limits.max_history_events
        {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::HistoryEvents,
            ));
        }
        let mut history = Vec::with_capacity(durable.events.len());
        let mut state = State::Pending;
        for event in &durable.events {
            match event {
                DurableEvent::RequestPersisted => history.push(Event::RequestPersisted),
                DurableEvent::AttemptPrepared { attempt } => {
                    let attempt = stored_effect_id!(AttemptId, attempt);
                    state = State::Open(attempt.clone());
                    history.push(Event::AttemptPrepared(attempt));
                }
                DurableEvent::DispatchNotAccepted { attempt } => {
                    let attempt = stored_effect_id!(AttemptId, attempt);
                    state = State::NotDelivered(attempt.clone());
                    history.push(Event::DispatchNotAccepted(attempt));
                }
                DurableEvent::Reconciliation {
                    attempt,
                    disposition,
                } => {
                    let attempt = stored_effect_id!(AttemptId, attempt);
                    let disposition = reconciliation_from_durable(*disposition);
                    state = match disposition {
                        ReconciliationDisposition::NotDelivered => {
                            State::NotDelivered(attempt.clone())
                        }
                        ReconciliationDisposition::Inconclusive
                        | ReconciliationDisposition::Unavailable => {
                            State::DeliveryUnknown(attempt.clone())
                        }
                    };
                    history.push(Event::ReconciliationRecorded {
                        attempt,
                        disposition,
                    });
                }
                DurableEvent::Receipt { receipt } => {
                    if u64::try_from(receipt.payload.len()).map_err(|_| StoreError::Unavailable)?
                        > limits.max_receipt_bytes
                    {
                        return Err(StoreError::Exhausted(
                            quire_integration::effect_reconciliation::Resource::ReceiptBytes,
                        ));
                    }
                    let receipt = receipt_from_durable(receipt, request.selection().clone())?;
                    state = State::Delivered(receipt.clone());
                    history.push(Event::ReceiptPersisted(Box::new(receipt)));
                }
                DurableEvent::DeliveryUnknown { attempt, cause } => {
                    let attempt = stored_effect_id!(AttemptId, attempt);
                    let cause = cause_from_durable(*cause);
                    state = State::DeliveryUnknown(attempt.clone());
                    history.push(Event::DeliveryUnknownRecorded { attempt, cause });
                }
                DurableEvent::Observation { observation } => {
                    if u64::try_from(observation.payload.len())
                        .map_err(|_| StoreError::Unavailable)?
                        > limits.max_observation_bytes
                    {
                        return Err(StoreError::Exhausted(
                            quire_integration::effect_reconciliation::Resource::ObservationBytes,
                        ));
                    }
                    let observation = observation_from_durable(observation)?;
                    let receipt = match &state {
                        State::Delivered(receipt) => Some(receipt.clone()),
                        _ => None,
                    };
                    state = State::EffectObserved {
                        receipt,
                        observation: Box::new(observation.clone()),
                    };
                    history.push(Event::ObservationLinked(Box::new(observation)));
                }
            }
        }
        if history.is_empty()
            || durable.revision
                != u64::try_from(history.len()).map_err(|_| StoreError::Unavailable)?
        {
            return Err(StoreError::Unavailable);
        }
        Ok(Record {
            request: request.clone(),
            revision: durable.revision,
            state,
            history,
        })
    }

    fn durable_from_record(record: &Record) -> DurableRecord {
        let events = record
            .history
            .iter()
            .map(|event| match event {
                Event::RequestPersisted => DurableEvent::RequestPersisted,
                Event::AttemptPrepared(attempt) => DurableEvent::AttemptPrepared {
                    attempt: attempt.as_str().to_owned(),
                },
                Event::DispatchNotAccepted(attempt) => DurableEvent::DispatchNotAccepted {
                    attempt: attempt.as_str().to_owned(),
                },
                Event::ReconciliationRecorded {
                    attempt,
                    disposition,
                } => DurableEvent::Reconciliation {
                    attempt: attempt.as_str().to_owned(),
                    disposition: reconciliation_to_durable(*disposition),
                },
                Event::ReceiptPersisted(receipt) => DurableEvent::Receipt {
                    receipt: durable_receipt(receipt),
                },
                Event::DeliveryUnknownRecorded { attempt, cause } => {
                    DurableEvent::DeliveryUnknown {
                        attempt: attempt.as_str().to_owned(),
                        cause: cause_to_durable(*cause),
                    }
                }
                Event::ObservationLinked(observation) => DurableEvent::Observation {
                    observation: durable_observation(observation),
                },
            })
            .collect();
        DurableRecord {
            key: record.request.key().as_str().to_owned(),
            canonical_digest: Self::digest(&record.request),
            revision: record.revision,
            request: durable_request(&record.request),
            events,
        }
    }

    fn persist(
        &mut self,
        record: Record,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.save_durable(&Self::durable_from_record(&record), limits)?;
        self.record = Some(record.clone());
        Ok(record)
    }

    fn append(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        event: Event,
        state: State,
        required_attempt: Option<&AttemptId>,
        retain_lock: bool,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        let acquired = if self.operation_lock.is_none() {
            Some(self.acquire_lock(limits)?)
        } else {
            None
        };
        let durable = self.read_durable(limits)?;
        if durable.key != key.as_str() {
            return Err(StoreError::Conflict);
        }
        let request = request_from_durable(&durable.request, self.lock_identity)?;
        let mut record = self.decode(&request, &durable, limits)?;
        if record.revision != expected_revision {
            return Err(StoreError::Conflict);
        }
        if required_attempt.is_some_and(|attempt| !Self::attempt_matches(&record, attempt)) {
            return Err(StoreError::Conflict);
        }
        let next = record
            .history
            .len()
            .checked_add(1)
            .ok_or(StoreError::Unavailable)?;
        if u64::try_from(next).map_err(|_| StoreError::Unavailable)? > limits.max_history_events {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::HistoryEvents,
            ));
        }
        record.history.push(event);
        record.revision = record
            .revision
            .checked_add(1)
            .ok_or(StoreError::Unavailable)?;
        record.state = state;
        let record = self.persist(record, limits)?;
        if retain_lock {
            if self.operation_lock.is_none() {
                self.operation_lock = acquired;
            }
        } else {
            self.operation_lock.take();
        }
        Ok(record)
    }

    fn attempt_matches(record: &Record, attempt: &AttemptId) -> bool {
        matches!(&record.state, State::Open(current) | State::DeliveryUnknown(current) if current == attempt)
    }
}

impl EffectStore for FileStore {
    fn load(
        &mut self,
        key: &IdempotencyKey,
        limits: Limits,
    ) -> std::result::Result<Option<Record>, StoreError> {
        let lock = self.acquire_lock(limits)?;
        if !self.state_path().is_file() {
            return Ok(None);
        }
        let durable = self.read_durable(limits)?;
        if durable.key != key.as_str() {
            return Ok(None);
        }
        let request = request_from_durable(&durable.request, self.lock_identity)?;
        let record = self.decode(&request, &durable, limits)?;
        self.record = Some(record.clone());
        if !matches!(
            record.state,
            State::Delivered(_) | State::EffectObserved { .. }
        ) {
            self.operation_lock = Some(lock);
        }
        Ok(Some(record))
    }

    fn reserve_or_load(
        &mut self,
        request: &EffectRequest,
        limits: Limits,
    ) -> std::result::Result<Reservation, StoreError> {
        if limits.max_history_events < 1 {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::HistoryEvents,
            ));
        }
        if let Some(barrier) = &self.reservation_barrier {
            barrier.wait();
        }
        let lock = self.acquire_lock(limits)?;
        if self.state_path().is_file() {
            let durable = self.read_durable(limits)?;
            if durable.key != request.key().as_str()
                || durable.canonical_digest != Self::digest(request)
            {
                return Ok(Reservation::Conflict);
            }
            let record = self.decode(request, &durable, limits)?;
            self.record = Some(record.clone());
            if !matches!(
                record.state,
                State::Delivered(_) | State::EffectObserved { .. }
            ) {
                self.operation_lock = Some(lock);
            }
            return Ok(Reservation::Existing(record));
        }
        let record = Record {
            request: request.clone(),
            revision: 1,
            state: State::Pending,
            history: vec![Event::RequestPersisted],
        };
        let record = self.persist(record, limits)?;
        self.operation_lock = Some(lock);
        if self.crash == CrashPoint::AfterReservation {
            process::exit(CRASH_AFTER_RESERVATION);
        }
        Ok(Reservation::Inserted(record))
    }

    fn prepare_attempt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        let record = self.append(
            key,
            expected_revision,
            Event::AttemptPrepared(attempt.clone()),
            State::Open(attempt.clone()),
            None,
            true,
            limits,
        )?;
        if self.crash == CrashPoint::AfterAttempt {
            process::exit(CRASH_AFTER_ATTEMPT);
        }
        Ok(record)
    }

    fn record_reconciliation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        disposition: ReconciliationDisposition,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        let state = match disposition {
            ReconciliationDisposition::NotDelivered => State::NotDelivered(attempt.clone()),
            ReconciliationDisposition::Inconclusive | ReconciliationDisposition::Unavailable => {
                State::DeliveryUnknown(attempt.clone())
            }
        };
        self.append(
            key,
            expected_revision,
            Event::ReconciliationRecorded {
                attempt: attempt.clone(),
                disposition,
            },
            state,
            Some(attempt),
            false,
            limits,
        )
    }

    fn record_dispatch_not_accepted(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.append(
            key,
            expected_revision,
            Event::DispatchNotAccepted(attempt.clone()),
            State::NotDelivered(attempt.clone()),
            Some(attempt),
            false,
            limits,
        )
    }

    fn record_receipt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        receipt: &Receipt,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.append(
            key,
            expected_revision,
            Event::ReceiptPersisted(Box::new(receipt.clone())),
            State::Delivered(receipt.clone()),
            Some(attempt),
            false,
            limits,
        )
    }

    fn record_delivery_unknown(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        cause: DeliveryUnknownCause,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.append(
            key,
            expected_revision,
            Event::DeliveryUnknownRecorded {
                attempt: attempt.clone(),
                cause,
            },
            State::DeliveryUnknown(attempt.clone()),
            Some(attempt),
            false,
            limits,
        )
    }

    fn link_observation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        observation: &Observation,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        let acquired = if self.operation_lock.is_none() {
            Some(self.acquire_lock(limits)?)
        } else {
            None
        };
        let durable = self.read_durable(limits)?;
        if durable.key != key.as_str() {
            return Err(StoreError::Conflict);
        }
        let request = request_from_durable(&durable.request, self.lock_identity)?;
        let mut current = self.decode(&request, &durable, limits)?;
        if current.revision != expected_revision {
            return Err(StoreError::Conflict);
        }
        let receipt = match &current.state {
            State::Delivered(receipt) => Some(receipt.clone()),
            State::Pending
            | State::Open(_)
            | State::NotDelivered(_)
            | State::DeliveryUnknown(_) => None,
            State::EffectObserved { .. } => return Err(StoreError::Conflict),
        };
        let next = current
            .history
            .len()
            .checked_add(1)
            .ok_or(StoreError::Unavailable)?;
        if u64::try_from(next).map_err(|_| StoreError::Unavailable)? > limits.max_history_events {
            return Err(StoreError::Exhausted(
                quire_integration::effect_reconciliation::Resource::HistoryEvents,
            ));
        }
        current
            .history
            .push(Event::ObservationLinked(Box::new(observation.clone())));
        current.revision = current
            .revision
            .checked_add(1)
            .ok_or(StoreError::Unavailable)?;
        current.state = State::EffectObserved {
            receipt,
            observation: Box::new(observation.clone()),
        };
        let record = self.persist(current, limits)?;
        self.operation_lock.take();
        drop(acquired);
        Ok(record)
    }
}

pub fn read_durable(root: &Path) -> DurableRecord {
    let bytes = fs::read(root.join("state.json")).expect("durable state bytes");
    serde_json::from_slice(&bytes).expect("durable state contract")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterCrash {
    None,
    AfterAcceptance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AdapterContract {
    contract_version: String,
    dispatch_path: String,
    duplicate_guarantee: String,
    idempotency_field: String,
    reconciliation_path: Option<String>,
    request_content_type: String,
    response_content_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ServiceReceipt {
    external_effect_id: String,
}

pub struct HttpAdapter {
    address: SocketAddr,
    manifest: AdapterManifest,
    contract: AdapterContract,
    crash: AdapterCrash,
}

impl HttpAdapter {
    const MAX_HTTP_HEADER_BYTES: u64 = 16 * 1024;

    pub fn new(address: SocketAddr, mode: ManifestMode, crash: AdapterCrash) -> Self {
        let contract_bytes = match mode {
            ManifestMode::Safe => ADAPTER_CONTRACT,
            ManifestMode::Unsupported => UNSUPPORTED_ADAPTER_CONTRACT,
        };
        let contract: AdapterContract =
            serde_json::from_slice(contract_bytes).expect("selected adapter contract fixture");
        assert_eq!(contract.contract_version, "quire.effect-adapter/v1");
        assert_eq!(contract.idempotency_field, "Idempotency-Key");
        assert_eq!(contract.request_content_type, "application/json");
        assert_eq!(contract.response_content_type, "application/json");
        match mode {
            ManifestMode::Safe => {
                assert_eq!(
                    contract.duplicate_guarantee,
                    "same-key-and-bytes-same-effect"
                );
                assert_eq!(
                    contract.reconciliation_path.as_deref(),
                    Some("/effects/{key}")
                );
            }
            ManifestMode::Unsupported => {
                assert_eq!(contract.duplicate_guarantee, "none");
                assert_eq!(contract.reconciliation_path, None);
            }
        }
        Self {
            address,
            manifest: manifest(mode),
            contract,
            crash,
        }
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        key: &str,
        body: &[u8],
        timeout_ms: u64,
        max_response_bytes: u64,
    ) -> Result<(u16, Vec<u8>), AdapterError> {
        if timeout_ms == 0 {
            return Err(AdapterError::Unavailable);
        }
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(timeout_ms))
            .ok_or(AdapterError::Unavailable)?;
        let mut stream = TcpStream::connect_timeout(&self.address, remaining(deadline)?)
            .map_err(|_| AdapterError::Unavailable)?;
        remaining(deadline)?;
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\n{}: {key}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.address,
            self.contract.idempotency_field,
            body.len()
        );
        write_before(&mut stream, head.as_bytes(), deadline)?;
        write_before(&mut stream, body, deadline)?;
        let maximum = Self::MAX_HTTP_HEADER_BYTES
            .checked_add(max_response_bytes)
            .ok_or(AdapterError::ContractViolation)?;
        let mut response = Vec::with_capacity(
            usize::try_from(maximum.min(8192)).map_err(|_| AdapterError::ContractViolation)?,
        );
        let mut buffer = [0_u8; 4096];
        loop {
            stream
                .set_read_timeout(Some(remaining(deadline)?))
                .map_err(|_| AdapterError::Unavailable)?;
            let remaining_capacity = maximum
                .saturating_add(1)
                .saturating_sub(
                    u64::try_from(response.len()).map_err(|_| AdapterError::ContractViolation)?,
                )
                .min(u64::try_from(buffer.len()).expect("fixed response buffer"));
            if remaining_capacity == 0 {
                break;
            }
            let read = stream
                .read(
                    &mut buffer[..usize::try_from(remaining_capacity)
                        .map_err(|_| AdapterError::ContractViolation)?],
                )
                .map_err(|_| AdapterError::Unavailable)?;
            remaining(deadline)?;
            if read == 0 {
                break;
            }
            response.extend_from_slice(&buffer[..read]);
        }
        if u64::try_from(response.len()).map_err(|_| AdapterError::ContractViolation)? > maximum {
            return Err(AdapterError::ContractViolation);
        }
        parse_http_response(&response, max_response_bytes)
    }

    fn receipt(&self, request: &EffectRequest, body: Vec<u8>) -> Result<Receipt, AdapterError> {
        let decoded: ServiceReceipt =
            serde_json::from_slice(&body).map_err(|_| AdapterError::ContractViolation)?;
        Ok(Receipt {
            id: effect_id!(ReceiptId, format!("receipt:{}", decoded.external_effect_id)),
            request_id: request.id().clone(),
            selection: self.manifest.selection.clone(),
            external_effect_id: effect_id!(ExternalEffectId, decoded.external_effect_id),
            payload: body,
        })
    }
}

fn remaining(deadline: Instant) -> Result<Duration, AdapterError> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or(AdapterError::Unavailable)
}

fn write_before(
    stream: &mut TcpStream,
    mut bytes: &[u8],
    deadline: Instant,
) -> Result<(), AdapterError> {
    while !bytes.is_empty() {
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(|_| AdapterError::Unavailable)?;
        let written = stream.write(bytes).map_err(|_| AdapterError::Unavailable)?;
        remaining(deadline)?;
        if written == 0 {
            return Err(AdapterError::Unavailable);
        }
        bytes = &bytes[written..];
    }
    Ok(())
}

impl AuthorizedEffectAdapter for HttpAdapter {
    fn manifest(&self) -> &AdapterManifest {
        &self.manifest
    }

    fn dispatch(
        &mut self,
        permit: &DispatchPermit,
        request: &EffectRequest,
    ) -> std::result::Result<DispatchResult, AdapterError> {
        if permit.request_id() != request.id() || permit.remaining_duration_ms() == 0 {
            return Err(AdapterError::ContractViolation);
        }
        let dispatch_path = self.contract.dispatch_path.clone();
        let (status, body) = self.request(
            "POST",
            &dispatch_path,
            request.key().as_str(),
            request.canonical_arguments(),
            permit.remaining_duration_ms(),
            permit.max_receipt_bytes(),
        )?;
        if self.crash == AdapterCrash::AfterAcceptance && status == 200 {
            process::exit(CRASH_AFTER_ACCEPTANCE);
        }
        match status {
            200 => self
                .receipt(request, body)
                .map(|receipt| DispatchResult::Accepted(Box::new(receipt))),
            409 => Ok(DispatchResult::NotAccepted),
            _ => Ok(DispatchResult::Indeterminate),
        }
    }

    fn reconcile(
        &mut self,
        permit: &ReconciliationPermit,
        request: &EffectRequest,
    ) -> std::result::Result<ReconciliationResult, AdapterError> {
        if permit.request_id() != request.id() || permit.remaining_duration_ms() == 0 {
            return Err(AdapterError::ContractViolation);
        }
        let path = self
            .contract
            .reconciliation_path
            .as_deref()
            .ok_or(AdapterError::ContractViolation)?
            .replace("{key}", request.key().as_str());
        let (status, body) = self.request(
            "GET",
            &path,
            request.key().as_str(),
            &[],
            permit.remaining_duration_ms(),
            permit.max_receipt_bytes(),
        )?;
        match status {
            200 => self
                .receipt(request, body)
                .map(|receipt| ReconciliationResult::Accepted(Box::new(receipt))),
            404 => Ok(ReconciliationResult::NotDelivered),
            _ => Ok(ReconciliationResult::Inconclusive),
        }
    }
}

pub fn parse_http_response(
    bytes: &[u8],
    max_body_bytes: u64,
) -> Result<(u16, Vec<u8>), AdapterError> {
    let split = find_bytes(bytes, b"\r\n\r\n").ok_or(AdapterError::ContractViolation)?;
    if u64::try_from(split).map_err(|_| AdapterError::ContractViolation)?
        > HttpAdapter::MAX_HTTP_HEADER_BYTES
    {
        return Err(AdapterError::ContractViolation);
    }
    let head = std::str::from_utf8(&bytes[..split]).map_err(|_| AdapterError::ContractViolation)?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(AdapterError::ContractViolation)?;
    let content_lengths = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("Content-Length"))
        .map(|(_, value)| value.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AdapterError::ContractViolation)?;
    let [content_length] = content_lengths.as_slice() else {
        return Err(AdapterError::ContractViolation);
    };
    let content_types = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("Content-Type"))
        .map(|(_, value)| value.trim())
        .collect::<Vec<_>>();
    let [content_type] = content_types.as_slice() else {
        return Err(AdapterError::ContractViolation);
    };
    if *content_type != "application/json" {
        return Err(AdapterError::ContractViolation);
    }
    if *content_length > max_body_bytes {
        return Err(AdapterError::ContractViolation);
    }
    let body = &bytes[split + 4..];
    if u64::try_from(body.len()).map_err(|_| AdapterError::ContractViolation)? != *content_length {
        return Err(AdapterError::ContractViolation);
    }
    Ok((status, body.to_vec()))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[derive(Default)]
struct ServiceState {
    effects: BTreeMap<String, (Vec<u8>, String)>,
    calls: Vec<String>,
}

pub struct HttpService {
    address: SocketAddr,
    state: Arc<Mutex<ServiceState>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl HttpService {
    pub fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback service");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let address = listener.local_addr().expect("loopback address");
        let state = Arc::new(Mutex::new(ServiceState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            while !worker_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => handle_connection(stream, &worker_state),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            address,
            state,
            stop,
            thread: Some(worker),
        }
    }

    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn calls(&self) -> Vec<String> {
        self.state.lock().expect("service state").calls.clone()
    }

    pub fn effect_count(&self) -> usize {
        self.state.lock().expect("service state").effects.len()
    }

    pub fn external_effect_id(&self, key: &str) -> Option<String> {
        self.state
            .lock()
            .expect("service state")
            .effects
            .get(key)
            .map(|(_, identity)| identity.clone())
    }
}

impl Drop for HttpService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.address);
        if let Some(worker) = self.thread.take() {
            worker.join().expect("loopback service thread");
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: &Arc<Mutex<ServiceState>>) {
    const MAX_REQUEST_HEADER_BYTES: usize = 16 * 1024;
    const MAX_REQUEST_BODY_BYTES: usize = 1024;

    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(read) => {
                bytes.extend_from_slice(&buffer[..read]);
                if bytes.len() > MAX_REQUEST_HEADER_BYTES {
                    return;
                }
                if let Some(at) = find_bytes(&bytes, b"\r\n\r\n") {
                    break at;
                }
            }
            Err(_) => return,
        }
    };
    let head = match std::str::from_utf8(&bytes[..header_end]) {
        Ok(head) => head,
        Err(_) => return,
    };
    let mut lines = head.lines();
    let mut request_line = match lines.next().map(str::split_whitespace) {
        Some(line) => line,
        None => return,
    };
    let method = match request_line.next() {
        Some(method) => method.to_owned(),
        None => return,
    };
    let path = match request_line.next() {
        Some(path) => path.to_owned(),
        None => return,
    };
    let mut key = None;
    let mut content_length = 0_usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Idempotency-Key") {
                key = Some(value.trim().to_owned());
            }
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    if content_length > MAX_REQUEST_BODY_BYTES {
        return;
    }
    while bytes.len().saturating_sub(header_end + 4) < content_length {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => return,
            Ok(read) => bytes.extend_from_slice(&buffer[..read]),
        }
    }
    let body = bytes[header_end + 4..header_end + 4 + content_length].to_vec();
    let key = match key {
        Some(key) => key,
        None => return,
    };
    let (status, response) = {
        let mut state = state.lock().expect("service state");
        state.calls.push(format!("{method} {key}"));
        match method.as_str() {
            "POST" if path == "/effects" => {
                if let Some((prior, external)) = state.effects.get(&key) {
                    if prior == &body {
                        (200, service_receipt(external))
                    } else {
                        (409, b"{}".to_vec())
                    }
                } else {
                    let mut external_preimage = Vec::with_capacity(key.len() + body.len() + 16);
                    external_preimage.extend_from_slice(
                        &u64::try_from(key.len())
                            .expect("bounded key length")
                            .to_be_bytes(),
                    );
                    external_preimage.extend_from_slice(key.as_bytes());
                    external_preimage.extend_from_slice(
                        &u64::try_from(body.len())
                            .expect("bounded body length")
                            .to_be_bytes(),
                    );
                    external_preimage.extend_from_slice(&body);
                    let external = format!("effect:{}", &digest(&external_preimage)[7..23]);
                    state.effects.insert(key, (body, external.clone()));
                    (200, service_receipt(&external))
                }
            }
            "GET" if path == format!("/effects/{key}") => state
                .effects
                .get(&key)
                .map_or((404, b"{}".to_vec()), |(_, external)| {
                    (200, service_receipt(external))
                }),
            _ => (404, b"{}".to_vec()),
        }
    };
    let reason = match status {
        200 => "OK",
        409 => "Conflict",
        _ => "Not Found",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&response);
}

fn service_receipt(external: &str) -> Vec<u8> {
    serde_json::to_vec(&ServiceReceipt {
        external_effect_id: external.to_owned(),
    })
    .expect("service receipt")
}

#[derive(Clone, Copy)]
pub struct LocalDecisionAuthority {
    pub current: bool,
}

impl DecisionAuthority for LocalDecisionAuthority {
    fn decision_selection(&self) -> &CapabilitySelection {
        static CELL: std::sync::OnceLock<CapabilitySelection> = std::sync::OnceLock::new();
        CELL.get_or_init(decision_selection)
    }

    fn grant_selection(&self) -> &CapabilitySelection {
        static CELL: std::sync::OnceLock<CapabilitySelection> = std::sync::OnceLock::new();
        CELL.get_or_init(grant_selection)
    }

    fn verify(
        &self,
        recommendation: &Recommendation,
        decision: &Decision,
        grant: &AuthorityGrant,
        budget: CallBudget,
    ) -> std::result::Result<AuthorityStatus, AuthorityValidationError> {
        if budget.remaining_duration_ms() == 0
            || decision.artifact.contract_version != "quire.human-decision/v1"
            || grant.artifact.contract_version != "quire.effect-decision/v1"
            || decision.artifact.attestation != b"it002-decision-attestation"
            || grant.artifact.attestation != b"it002-grant-attestation"
        {
            return Err(AuthorityValidationError::ContractViolation);
        }
        if decision.recommendation_id != recommendation.id
            || decision.recommendation_digest != recommendation.digest
            || decision.authority_grant_id != grant.id
        {
            return Err(AuthorityValidationError::InvalidEvidence);
        }
        Ok(if self.current {
            AuthorityStatus::Current
        } else {
            AuthorityStatus::OutsideValidity
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixtureObservationWire {
    contract_version: String,
    request_id: String,
    external_effect_id: String,
    effect: String,
}

pub struct ContractObservationAuthority;

impl ObservationAuthority for ContractObservationAuthority {
    fn selection(&self) -> &CapabilitySelection {
        static CELL: std::sync::OnceLock<CapabilitySelection> = std::sync::OnceLock::new();
        CELL.get_or_init(observation_selection)
    }

    fn verify(
        &self,
        observation: &Observation,
        budget: CallBudget,
    ) -> std::result::Result<(), ObservationError> {
        if budget.remaining_duration_ms() == 0 {
            return Err(ObservationError::ContractViolation);
        }
        let wire: FixtureObservationWire = serde_json::from_slice(&observation.payload)
            .map_err(|_| ObservationError::ContractViolation)?;
        let external = observation
            .external_effect_id
            .as_ref()
            .ok_or(ObservationError::Unqualified)?;
        if wire.contract_version != "quire.integration.test-observation/v1"
            || wire.request_id != observation.request_id.as_str()
            || wire.external_effect_id != external.as_str()
            || wire.effect != "refund-settled"
        {
            return Err(ObservationError::Unqualified);
        }
        Ok(())
    }
}

pub fn matching_observation(receipt: &Receipt) -> Observation {
    let payload = serde_json::to_vec(&FixtureObservationWire {
        contract_version: "quire.integration.test-observation/v1".to_owned(),
        request_id: receipt.request_id.as_str().to_owned(),
        external_effect_id: receipt.external_effect_id.as_str().to_owned(),
        effect: "refund-settled".to_owned(),
    })
    .expect("contract-bound observation fixture");
    Observation {
        id: effect_id!(
            ObservationId,
            format!("observation:{}", receipt.id.as_str())
        ),
        request_id: receipt.request_id.clone(),
        external_effect_id: Some(receipt.external_effect_id.clone()),
        authority: observation_selection(),
        payload,
    }
}

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct TestDirectory(PathBuf);

impl TestDirectory {
    pub fn new(label: &str) -> Self {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "quire-integration-{label}-{}-{sequence}",
            process::id()
        ));
        fs::create_dir(&path).expect("create isolated IT-002 directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn persisted_fields(request: &EffectRequest) -> PersistedEffectRequest {
    PersistedEffectRequest {
        id: request.id().clone(),
        lock_identity: request.lock_identity(),
        recommendation: request.recommendation().clone(),
        decision: request.decision().clone(),
        authority: request.authority().clone(),
        key: request.key().clone(),
        selection: request.selection().clone(),
        authority_grant_id: request.authority_grant_id().clone(),
        operation: request.operation().clone(),
        canonical_arguments: request.canonical_arguments().to_vec(),
        supersedes: request.supersedes().cloned(),
        canonical_bytes: request.canonical_bytes().to_vec(),
    }
}
