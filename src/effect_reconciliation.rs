// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Crash-safe coordination for authorized external effects.
//!
//! The only adapter call site in this crate is the coordinator's private
//! invocation path. Adapters receive an opaque permit which can be minted only
//! after the request and attempt are durable.

use std::sync::Arc;
use std::time::{Duration, Instant};

use sha2::{Digest as _, Sha256};

use crate::ecosystem_lock::{CapabilitySelection, LockIdentity, ValidatedLock};

const OBSERVATION_AUTHORITY_CONTRACT: &str = "quire.observation-authority/v1";
const EFFECT_ADAPTER_CONTRACT: &str = "quire.effect-adapter/v1";
const DECISION_AUTHORITY_CONTRACT: &str = "quire.human-decision/v1";
const GRANT_AUTHORITY_CONTRACT: &str = "quire.effect-decision/v1";

macro_rules! effect_identity {
    ($name:ident, $domain:literal, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Constructs a bounded, nonempty identity.
            pub fn new(value: impl AsRef<str>, max_bytes: u64) -> Result<Self> {
                let value = value.as_ref();
                let length = u64::try_from(value.len()).map_err(|_| Error::IntegerOverflow)?;
                if length > max_bytes {
                    return Err(Error::ResourceExhausted(Resource::IdentityBytes));
                }
                if value.trim().is_empty() {
                    return Err(Error::InvalidIdentity);
                }
                Ok(Self(value.to_owned()))
            }

            /// Returns the exact identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

effect_identity!(
    RecommendationId,
    "recommendation",
    "Formal recommendation identity."
);
effect_identity!(DecisionId, "decision", "Attributed decision identity.");
effect_identity!(
    AuthorityGrantId,
    "authority-grant",
    "Authority-grant identity."
);
effect_identity!(ActorId, "actor", "Attributed actor identity.");
effect_identity!(ActionId, "action", "Authorized action identity.");
effect_identity!(SubjectId, "subject", "Authorized subject identity.");
effect_identity!(ScopeId, "scope", "Exact authority scope identity.");
effect_identity!(
    IdempotencyKey,
    "idempotency-key",
    "Caller-owned global idempotency key."
);
effect_identity!(OperationId, "operation", "Adapter operation identity.");
effect_identity!(
    EffectRequestId,
    "effect-request",
    "Canonical effect-request identity."
);
effect_identity!(AttemptId, "attempt", "Durable adapter-attempt identity.");
effect_identity!(ReceiptId, "receipt", "Transport-receipt identity.");
effect_identity!(
    ExternalEffectId,
    "external-effect",
    "External provider effect identity."
);
effect_identity!(
    ObservationId,
    "observation",
    "Business-effect observation identity."
);

/// Resource dimension refused by the effect boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resource {
    /// Identity bytes.
    IdentityBytes,
    /// Canonical argument bytes.
    ArgumentBytes,
    /// Complete canonical request bytes.
    RequestBytes,
    /// Durable history events.
    HistoryEvents,
    /// Adapter attempts.
    Attempts,
    /// Receipt bytes.
    ReceiptBytes,
    /// Observation bytes.
    ObservationBytes,
    /// Canonical owner artifact and attestation bytes.
    OwnerArtifactBytes,
    /// Total effect-coordination execution duration.
    ExecutionDuration,
}

/// Stable effect-boundary failure or refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// An identity is empty or whitespace-only.
    InvalidIdentity,
    /// Checked integer conversion or arithmetic failed.
    IntegerOverflow,
    /// A caller-lowered resource limit was exceeded.
    ResourceExhausted(Resource),
    /// No authority grant was supplied.
    AuthorityMissing,
    /// The authority grant is not current for a new dispatch.
    AuthorityExpired,
    /// Actor, action, subject, grant, or scope does not match the decision.
    AuthorityMismatch,
    /// The supplied scope is broader than the exact decision scope.
    AuthorityOverBroad,
    /// The decision does not bind the supplied recommendation or request.
    DecisionMismatch,
    /// The manifest or adapter is not the exact lock-admitted selection.
    AdapterMismatch,
    /// A global caller key already names unequal canonical request bytes.
    KeyConflict,
    /// A further transport call is not authorized by current authority and manifest guarantee.
    RetryUnsupported,
    /// Durable state could not be read or advanced atomically.
    PersistenceFailure,
    /// The selected adapter failed after admission.
    AdapterFailure,
    /// A receipt is not attributed to the exact request and adapter.
    MalformedReceipt,
    /// An observation is unqualified or belongs to another request/effect.
    ForeignObservation,
}

impl Error {
    /// Returns the stable machine-readable code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidIdentity => "QINT-EFFECT-INVALID-IDENTITY",
            Self::IntegerOverflow => "QINT-EFFECT-INTEGER-OVERFLOW",
            Self::ResourceExhausted(_) => "QINT-EFFECT-RESOURCE-EXHAUSTED",
            Self::AuthorityMissing => "QINT-EFFECT-AUTHORITY-MISSING",
            Self::AuthorityExpired => "QINT-EFFECT-AUTHORITY-EXPIRED",
            Self::AuthorityMismatch => "QINT-EFFECT-AUTHORITY-MISMATCH",
            Self::AuthorityOverBroad => "QINT-EFFECT-AUTHORITY-OVER-BROAD",
            Self::DecisionMismatch => "QINT-EFFECT-DECISION-MISMATCH",
            Self::AdapterMismatch => "QINT-EFFECT-ADAPTER-MISMATCH",
            Self::KeyConflict => "QINT-EFFECT-IDEMPOTENCY-CONFLICT",
            Self::RetryUnsupported => "QINT-EFFECT-RETRY-UNSUPPORTED",
            Self::PersistenceFailure => "QINT-EFFECT-PERSISTENCE-FAILURE",
            Self::AdapterFailure => "QINT-EFFECT-ADAPTER-FAILURE",
            Self::MalformedReceipt => "QINT-EFFECT-MALFORMED-RECEIPT",
            Self::ForeignObservation => "QINT-EFFECT-FOREIGN-OBSERVATION",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for Error {}

/// Effect-boundary result.
pub type Result<T> = std::result::Result<T, Error>;

/// Caller-lowered finite ceilings for effect coordination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum bytes in any identity.
    pub max_identity_bytes: u64,
    /// Maximum canonical argument bytes.
    pub max_argument_bytes: u64,
    /// Maximum canonical request bytes.
    pub max_request_bytes: u64,
    /// Maximum retained journal events.
    pub max_history_events: u64,
    /// Maximum durable attempts.
    pub max_attempts: u64,
    /// Maximum receipt payload bytes.
    pub max_receipt_bytes: u64,
    /// Maximum observation payload bytes.
    pub max_observation_bytes: u64,
    /// Maximum bytes in each owner artifact envelope.
    pub max_owner_artifact_bytes: u64,
    /// Maximum total wall-clock duration for one coordinator invocation.
    pub max_execution_duration_ms: u64,
}

/// Trusted monotonic time source used to enforce an invocation duration.
///
/// Implementations supplied through [`EffectCoordinator::with_monotonic_clock`]
/// must never move backwards. The default coordinator uses [`Instant`].
pub trait MonotonicClock: Send + Sync {
    /// Returns elapsed monotonic time from an arbitrary, stable origin.
    fn now(&self) -> Duration;
}

#[derive(Debug)]
struct SystemMonotonicClock {
    origin: Instant,
}

impl SystemMonotonicClock {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

#[derive(Clone, Copy)]
struct ExecutionTimer<'a> {
    clock: &'a dyn MonotonicClock,
    started_at: Duration,
    limit: Duration,
}

impl<'a> ExecutionTimer<'a> {
    fn start(max_duration_ms: u64, clock: &'a dyn MonotonicClock) -> Result<Self> {
        if max_duration_ms == 0 {
            return Err(Error::ResourceExhausted(Resource::ExecutionDuration));
        }
        Ok(Self {
            clock,
            started_at: clock.now(),
            limit: Duration::from_millis(max_duration_ms),
        })
    }

    fn remaining_millis(self) -> Result<u64> {
        let elapsed = self
            .clock
            .now()
            .checked_sub(self.started_at)
            .ok_or(Error::ResourceExhausted(Resource::ExecutionDuration))?;
        let remaining = self
            .limit
            .checked_sub(elapsed)
            .ok_or(Error::ResourceExhausted(Resource::ExecutionDuration))?;
        let millis = u64::try_from(remaining.as_millis()).map_err(|_| Error::IntegerOverflow)?;
        if millis == 0 {
            return Err(Error::ResourceExhausted(Resource::ExecutionDuration));
        }
        Ok(millis)
    }

    fn check(self) -> Result<()> {
        self.remaining_millis().map(|_| ())
    }

    fn call_budget(self) -> Result<CallBudget> {
        Ok(CallBudget {
            remaining_duration_ms: self.remaining_millis()?,
        })
    }
}

/// Cooperative finite duration budget supplied to authority ports before each
/// invocation. Implementations must refuse or cancel work that cannot complete
/// within the remaining duration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallBudget {
    remaining_duration_ms: u64,
}

impl CallBudget {
    /// Remaining total coordinator duration at this call boundary.
    #[must_use]
    pub const fn remaining_duration_ms(self) -> u64 {
        self.remaining_duration_ms
    }
}

/// Exact versioned artifact bytes owned by an external authority domain.
///
/// This integration crate retains the artifact without redefining its wire
/// semantics; the selected [`DecisionAuthority`] validates the bytes and their
/// projection before they can authorize an effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerArtifact {
    /// Exact owner contract version.
    pub contract_version: String,
    /// Owner-canonical artifact bytes, including its evidence and lineage.
    pub canonical_bytes: Vec<u8>,
    /// Authority-produced attestation over the canonical bytes.
    pub attestation: Vec<u8>,
}

/// Exact or impermissibly broad authority scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorityScope {
    /// One exact scope.
    Exact(ScopeId),
    /// An ambient/all-subject scope, forbidden at this boundary.
    All,
}

/// Immutable formal recommendation reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recommendation {
    /// Recommendation identity.
    pub id: RecommendationId,
    /// Owner-produced semantic digest.
    pub digest: String,
    /// Recommended action.
    pub action: ActionId,
    /// Recommended subject.
    pub subject: SubjectId,
    /// Exact recommendation scope.
    pub scope: ScopeId,
}

/// Immutable authority grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityGrant {
    /// Grant identity.
    pub id: AuthorityGrantId,
    /// Exact externally owned grant artifact.
    pub artifact: OwnerArtifact,
    /// Exact owner authority that issued this grant.
    pub issuer: CapabilitySelection,
    /// Authorized actor.
    pub actor: ActorId,
    /// Authorized action.
    pub action: ActionId,
    /// Authorized subject.
    pub subject: SubjectId,
    /// Exact or broad scope.
    pub scope: AuthorityScope,
    /// First admitted logical millisecond.
    pub not_before_ms: u64,
    /// Last admitted logical millisecond, inclusive.
    pub expires_at_ms: u64,
}

/// Immutable attributed decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    /// Decision identity.
    pub id: DecisionId,
    /// Exact externally owned decision artifact.
    pub artifact: OwnerArtifact,
    /// Bound recommendation identity.
    pub recommendation_id: RecommendationId,
    /// Bound recommendation digest.
    pub recommendation_digest: String,
    /// Bound authority grant.
    pub authority_grant_id: AuthorityGrantId,
    /// Exact owner authority that authenticated this decision.
    pub authority: CapabilitySelection,
    /// Deciding actor.
    pub actor: ActorId,
    /// Approved action.
    pub action: ActionId,
    /// Approved subject.
    pub subject: SubjectId,
    /// Approved exact scope.
    pub scope: ScopeId,
    /// Approved adapter selection.
    pub adapter: CapabilitySelection,
    /// Approved operation.
    pub operation: OperationId,
    /// SHA-256 of the approved canonical arguments.
    pub argument_digest: String,
    /// Prior immutable decision revision superseded by this decision.
    pub supersedes: Option<DecisionId>,
}

/// Guarantee governing a repeated transport call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuplicateGuarantee {
    /// No safe repeated-call guarantee.
    None,
    /// Same caller key and byte-identical request resolve to the same external effect.
    SameKeyAndBytesSameEffect,
}

/// Reconciliation capability declared by the selected adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationSupport {
    /// No reconciliation operation is available.
    None,
    /// Exact operation used to reconcile an attempt.
    Operation(OperationId),
}

/// Exact selected adapter contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterManifest {
    /// Complete lock-admitted capability selection.
    pub selection: CapabilitySelection,
    /// Exact transport field carrying the caller-owned key.
    pub idempotency_field: String,
    /// Repeated-call guarantee.
    pub duplicate_guarantee: DuplicateGuarantee,
    /// Reconciliation support.
    pub reconciliation: ReconciliationSupport,
}

/// One effect request before admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submission {
    /// Formal recommendation.
    pub recommendation: Recommendation,
    /// Attributed decision.
    pub decision: Decision,
    /// Optional authority grant.
    pub authority: Option<AuthorityGrant>,
    /// Selected adapter contract.
    pub manifest: AdapterManifest,
    /// Caller-owned global key.
    pub key: IdempotencyKey,
    /// Requested operation.
    pub operation: OperationId,
    /// Adapter-contract canonical arguments.
    pub canonical_arguments: Vec<u8>,
    /// Prior immutable effect request superseded by this request.
    pub supersedes_request: Option<EffectRequestId>,
}

/// Canonical request persisted before adapter invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRequest {
    id: EffectRequestId,
    lock_identity: LockIdentity,
    recommendation: Recommendation,
    decision: Decision,
    authority: AuthorityGrant,
    key: IdempotencyKey,
    selection: CapabilitySelection,
    authority_grant_id: AuthorityGrantId,
    operation: OperationId,
    canonical_arguments: Vec<u8>,
    supersedes: Option<EffectRequestId>,
    canonical_bytes: Vec<u8>,
}

/// Store-decoded fields used for strict effect-request reconstruction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedEffectRequest {
    /// Previously persisted content-derived identity.
    pub id: EffectRequestId,
    /// Lock that admitted the original request.
    pub lock_identity: LockIdentity,
    /// Exact recommendation referenced by the owner decision.
    pub recommendation: Recommendation,
    /// Exact owner decision artifact and validated projection.
    pub decision: Decision,
    /// Exact owner grant artifact and validated projection.
    pub authority: AuthorityGrant,
    /// Caller-owned key.
    pub key: IdempotencyKey,
    /// Exact selected adapter.
    pub selection: CapabilitySelection,
    /// Bound authority grant.
    pub authority_grant_id: AuthorityGrantId,
    /// Exact adapter operation.
    pub operation: OperationId,
    /// Adapter-contract canonical arguments.
    pub canonical_arguments: Vec<u8>,
    /// Prior request revision, when present.
    pub supersedes: Option<EffectRequestId>,
    /// Complete canonical identity preimage.
    pub canonical_bytes: Vec<u8>,
}

impl EffectRequest {
    /// Reconstructs a strict persisted request without trusting duplicated fields.
    ///
    /// The content-derived identity is recomputed and every exposed field is
    /// checked against the canonical length-framed preimage before the value is
    /// returned to a store implementation.
    pub fn from_persisted(fields: PersistedEffectRequest, limits: Limits) -> Result<Self> {
        let PersistedEffectRequest {
            id,
            lock_identity,
            recommendation,
            decision,
            authority,
            key,
            selection,
            authority_grant_id,
            operation,
            canonical_arguments,
            supersedes,
            canonical_bytes,
        } = fields;
        let request = Self {
            id,
            lock_identity,
            recommendation,
            decision,
            authority,
            key,
            selection,
            authority_grant_id,
            operation,
            canonical_arguments,
            supersedes,
            canonical_bytes,
        };
        validate_persisted_request_fields(&request, limits)?;
        Ok(request)
    }

    /// Content-derived request identity.
    #[must_use]
    pub const fn id(&self) -> &EffectRequestId {
        &self.id
    }

    /// Exact validated lock that admitted this request.
    #[must_use]
    pub const fn lock_identity(&self) -> LockIdentity {
        self.lock_identity
    }

    /// Exact recommendation retained for authority revalidation.
    #[must_use]
    pub const fn recommendation(&self) -> &Recommendation {
        &self.recommendation
    }

    /// Exact owner decision retained for authority revalidation.
    #[must_use]
    pub const fn decision(&self) -> &Decision {
        &self.decision
    }

    /// Exact owner grant retained for authority revalidation.
    #[must_use]
    pub const fn authority(&self) -> &AuthorityGrant {
        &self.authority
    }

    /// Caller-owned key.
    #[must_use]
    pub const fn key(&self) -> &IdempotencyKey {
        &self.key
    }

    /// Exact selected adapter.
    #[must_use]
    pub const fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    /// Bound authority-grant identity.
    #[must_use]
    pub const fn authority_grant_id(&self) -> &AuthorityGrantId {
        &self.authority_grant_id
    }

    /// Exact adapter operation.
    #[must_use]
    pub const fn operation(&self) -> &OperationId {
        &self.operation
    }

    /// Adapter-contract canonical arguments.
    #[must_use]
    pub fn canonical_arguments(&self) -> &[u8] {
        &self.canonical_arguments
    }

    /// Prior immutable request superseded by this request, if any.
    #[must_use]
    pub const fn supersedes(&self) -> Option<&EffectRequestId> {
        self.supersedes.as_ref()
    }

    /// Complete canonical identity preimage.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// Attributed transport receipt, distinct from business-effect truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// Receipt identity.
    pub id: ReceiptId,
    /// Exact request identity.
    pub request_id: EffectRequestId,
    /// Adapter that issued the receipt.
    pub selection: CapabilitySelection,
    /// External provider effect identity.
    pub external_effect_id: ExternalEffectId,
    /// Bounded adapter payload.
    pub payload: Vec<u8>,
}

/// Authority-qualified business-effect observation input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    /// Observation identity.
    pub id: ObservationId,
    /// Correlated request identity.
    pub request_id: EffectRequestId,
    /// External effect identity when the observer can establish it.
    pub external_effect_id: Option<ExternalEffectId>,
    /// Exact QObs capability that qualified this observation.
    pub authority: CapabilitySelection,
    /// Bounded observation payload.
    pub payload: Vec<u8>,
}

/// One append-only durable journal event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// Canonical request reservation.
    RequestPersisted,
    /// Attempt made durable before transport.
    AttemptPrepared(AttemptId),
    /// Adapter conclusively reported that transport did not accept the attempt.
    DispatchNotAccepted(AttemptId),
    /// Non-accepting reconciliation result made durable before any retry.
    ReconciliationRecorded {
        /// Attempt that was reconciled.
        attempt: AttemptId,
        /// Durable reconciliation disposition.
        disposition: ReconciliationDisposition,
    },
    /// Attributed receipt made durable.
    ReceiptPersisted(Box<Receipt>),
    /// Uncertain delivery retained explicitly with its typed cause.
    DeliveryUnknownRecorded {
        /// Attempt whose delivery is uncertain.
        attempt: AttemptId,
        /// Boundary event that caused uncertainty.
        cause: DeliveryUnknownCause,
    },
    /// Authoritative business-effect observation linked.
    ObservationLinked(Box<Observation>),
}

/// Projected durable state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    /// Request is durable and no attempt is open.
    Pending,
    /// Attempt is durable and its delivery is not conclusive.
    Open(AttemptId),
    /// Reconciliation proved that the attempt was not delivered.
    NotDelivered(AttemptId),
    /// Attributed receipt is durable.
    Delivered(Receipt),
    /// Delivery remains uncertain and may be reconciled later.
    DeliveryUnknown(AttemptId),
    /// QObs has established the business effect.
    EffectObserved {
        /// Optional durable transport receipt.
        receipt: Option<Receipt>,
        /// Authority-qualified business observation.
        observation: Box<Observation>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum JournalProjection {
    Pending,
    Open(AttemptId),
    NotDelivered(AttemptId),
    DeliveryUnknown(AttemptId),
    Delivered(Box<Receipt>),
    EffectObserved {
        observation: Box<Observation>,
        receipt: Option<Box<Receipt>>,
    },
}

/// Complete durable record returned by a store port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Canonical request.
    pub request: EffectRequest,
    /// Monotonic compare-and-swap revision.
    pub revision: u64,
    /// Current projection.
    pub state: State,
    /// Immutable history.
    pub history: Vec<Event>,
}

/// Atomic reservation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reservation {
    /// This caller inserted the canonical request.
    Inserted(Record),
    /// The exact request already existed.
    Existing(Record),
    /// The global key already named unequal canonical bytes.
    Conflict,
}

/// Store-port failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreError {
    /// I/O or backend failure.
    Unavailable,
    /// Compare-and-swap revision or state changed.
    Conflict,
    /// A caller-lowered store/history limit was exceeded.
    Exhausted(Resource),
}

/// Durable non-accepting reconciliation disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconciliationDisposition {
    /// Provider proved that transport did not accept the attempt.
    NotDelivered,
    /// Provider could not determine whether transport accepted the attempt.
    Inconclusive,
    /// The selected adapter offers no usable reconciliation operation.
    Unavailable,
}

/// Typed cause retained with uncertain delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryUnknownCause {
    /// Transport returned no conclusive acceptance result.
    IndeterminateTransport,
    /// Adapter failed after receiving a durable dispatch permit.
    AdapterFailure,
    /// Adapter returned an oversized, malformed, or foreign receipt.
    InvalidReceipt,
    /// Reconciliation could not establish delivery either way.
    InconclusiveReconciliation,
    /// The selected adapter has no usable reconciliation operation.
    ReconciliationUnavailable,
    /// The adapter violated its reconciliation contract.
    ReconciliationContractViolation,
    /// A transport or reconciliation result could not be made durable.
    PostBoundaryPersistenceFailure,
    /// An accepted receipt could not be made durable.
    ReceiptPersistenceFailure,
}

/// Durable append-only effect store.
///
/// On every call, `limits.max_execution_duration_ms` is the remaining total
/// coordinator budget, not the original invocation ceiling. Implementations
/// must cooperatively time out or cancel work that cannot finish within it.
pub trait EffectStore {
    /// Loads a record by global caller key without mutating it. Implementations
    /// must preflight encoded lengths against `limits` before allocating.
    fn load(
        &mut self,
        key: &IdempotencyKey,
        limits: Limits,
    ) -> std::result::Result<Option<Record>, StoreError>;

    /// Atomically inserts the request or returns the existing key owner.
    fn reserve_or_load(
        &mut self,
        request: &EffectRequest,
        limits: Limits,
    ) -> std::result::Result<Reservation, StoreError>;

    /// Appends an attempt iff the expected revision is current.
    fn prepare_attempt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;

    /// Appends a non-accepting reconciliation result before any retry decision.
    fn record_reconciliation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        disposition: ReconciliationDisposition,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;

    /// Appends conclusive non-acceptance returned by a dispatch call.
    fn record_dispatch_not_accepted(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;

    /// Appends an attributed receipt iff the attempt remains open.
    fn record_receipt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        receipt: &Receipt,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;

    /// Appends an explicit uncertain-delivery marker.
    fn record_delivery_unknown(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        cause: DeliveryUnknownCause,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;

    /// Appends an authoritative observation.
    fn link_observation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        observation: &Observation,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError>;
}

/// Opaque proof that a specific request and attempt are durable and authorized.
///
/// Downstream code cannot mint this proof directly:
///
/// ```compile_fail
/// use quire_integration::effect_reconciliation::{
///     AttemptId, DispatchPermit, EffectRequestId,
/// };
/// let permit = DispatchPermit {
///     request_id: EffectRequestId::new("request", 64).unwrap(),
///     attempt_id: AttemptId::new("attempt", 64).unwrap(),
/// };
/// ```
#[derive(Debug)]
pub struct DispatchPermit {
    request_id: EffectRequestId,
    attempt_id: AttemptId,
    lock_identity: LockIdentity,
    max_receipt_bytes: u64,
    remaining_duration_ms: u64,
}

impl DispatchPermit {
    /// Durable request authorized by this permit.
    #[must_use]
    pub const fn request_id(&self) -> &EffectRequestId {
        &self.request_id
    }

    /// Durable attempt authorized by this permit.
    #[must_use]
    pub const fn attempt_id(&self) -> &AttemptId {
        &self.attempt_id
    }

    /// Exact lock under which dispatch was admitted.
    #[must_use]
    pub const fn lock_identity(&self) -> LockIdentity {
        self.lock_identity
    }

    /// Caller-lowered receipt bound the adapter must enforce before allocation.
    #[must_use]
    pub const fn max_receipt_bytes(&self) -> u64 {
        self.max_receipt_bytes
    }

    /// Remaining total execution budget at the transport boundary.
    #[must_use]
    pub const fn remaining_duration_ms(&self) -> u64 {
        self.remaining_duration_ms
    }
}

/// Opaque proof authorizing read-only reconciliation of a durable attempt.
#[derive(Debug)]
pub struct ReconciliationPermit {
    request_id: EffectRequestId,
    attempt_id: AttemptId,
    lock_identity: LockIdentity,
    max_receipt_bytes: u64,
    remaining_duration_ms: u64,
}

impl ReconciliationPermit {
    /// Durable request being reconciled.
    #[must_use]
    pub const fn request_id(&self) -> &EffectRequestId {
        &self.request_id
    }

    /// Durable attempt being reconciled.
    #[must_use]
    pub const fn attempt_id(&self) -> &AttemptId {
        &self.attempt_id
    }

    /// Exact lock under which reconciliation was admitted.
    #[must_use]
    pub const fn lock_identity(&self) -> LockIdentity {
        self.lock_identity
    }

    /// Caller-lowered receipt bound the adapter must enforce before allocation.
    #[must_use]
    pub const fn max_receipt_bytes(&self) -> u64 {
        self.max_receipt_bytes
    }

    /// Remaining total execution budget at the reconciliation boundary.
    #[must_use]
    pub const fn remaining_duration_ms(&self) -> u64 {
        self.remaining_duration_ms
    }
}

/// Adapter dispatch result before durable projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchResult {
    /// Provider accepted the request and returned an attributed receipt.
    Accepted(Box<Receipt>),
    /// Provider conclusively did not accept the request.
    NotAccepted,
    /// Transport ended without conclusive delivery knowledge.
    Indeterminate,
}

/// Reconciliation result for one durable attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationResult {
    /// Provider attributes acceptance to this request.
    Accepted(Box<Receipt>),
    /// Provider proves the attempt was not delivered.
    NotDelivered,
    /// Provider cannot establish delivery either way.
    Inconclusive,
    /// No reconciliation operation is available.
    Unavailable,
}

/// Typed adapter-port failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterError {
    /// Transport or provider access failed without a typed delivery result.
    Unavailable,
    /// Adapter returned a response that violates its selected contract.
    ContractViolation,
}

/// Typed observation-authority failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationError {
    /// Observation evidence is not authority-qualified.
    Unqualified,
    /// Observation input violates the selected authority contract.
    ContractViolation,
}

/// Typed owner-authority validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityValidationError {
    /// The actor is not authenticated by the selected owner authority.
    Unauthenticated,
    /// The decision evidence does not authorize the requested effect.
    InvalidEvidence,
    /// The artifact violates the selected authority contract.
    ContractViolation,
}

/// Trusted owner-authority result for the grant's validity interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityStatus {
    /// The grant is current according to the authority's own trusted time source.
    Current,
    /// The grant is not yet valid or has expired.
    OutsideValidity,
}

/// Selected effect adapter. Implementations cannot be invoked without a
/// coordinator-minted permit. The composition root must resolve the
/// implementation from the validated lock; untrusted submissions never supply
/// implementation values.
pub trait AuthorizedEffectAdapter {
    /// Exact lock-selected manifest implemented by this value.
    fn manifest(&self) -> &AdapterManifest;

    /// Dispatches a request only after persistence and authority admission.
    fn dispatch(
        &mut self,
        permit: &DispatchPermit,
        request: &EffectRequest,
    ) -> std::result::Result<DispatchResult, AdapterError>;

    /// Reconciles one durable uncertain attempt without requesting a new effect.
    fn reconcile(
        &mut self,
        permit: &ReconciliationPermit,
        request: &EffectRequest,
    ) -> std::result::Result<ReconciliationResult, AdapterError>;
}

/// Lock-selected observation authority, resolved by the trusted composition
/// root rather than supplied by an untrusted observation.
pub trait ObservationAuthority {
    /// Exact QObs capability selection implemented by this value.
    fn selection(&self) -> &CapabilitySelection;

    /// Verifies that the observation carries authority-owned evidence.
    fn verify(
        &self,
        observation: &Observation,
        budget: CallBudget,
    ) -> std::result::Result<(), ObservationError>;
}

/// Exact lock-selected verifier for owner-produced decisions and grants. The
/// verifier owns trusted time and artifact validation and must be resolved by
/// the trusted composition root.
pub trait DecisionAuthority {
    /// Exact HumanDecision authority capability implemented by this value.
    fn decision_selection(&self) -> &CapabilitySelection;

    /// Exact effect-decision/grant authority capability implemented by this value.
    fn grant_selection(&self) -> &CapabilitySelection;

    /// Authenticates and validates the immutable owner artifacts.
    fn verify(
        &self,
        recommendation: &Recommendation,
        decision: &Decision,
        grant: &AuthorityGrant,
        budget: CallBudget,
    ) -> std::result::Result<AuthorityStatus, AuthorityValidationError>;
}

/// Externally visible coordinator result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Fresh transport delivery was durably recorded.
    Delivered(Receipt),
    /// A prior uncertain attempt was reconciled and durably recorded.
    Reconciled(Receipt),
    /// Delivery is explicitly uncertain; no unsafe retry occurred.
    DeliveryUnknown {
        /// Durable attempt whose delivery is unknown.
        attempt: AttemptId,
        /// Typed reason the boundary cannot conclude delivery.
        cause: DeliveryUnknownCause,
    },
    /// Transport conclusively did not accept the durable attempt.
    NotDelivered(AttemptId),
    /// QObs established the business effect.
    EffectObserved(Observation),
}

/// Sole effect-capable coordinator.
#[derive(Clone)]
pub struct EffectCoordinator {
    limits: Limits,
    clock: Arc<dyn MonotonicClock>,
}

struct CoordinationContext<'a, V, O> {
    timer: ExecutionTimer<'a>,
    decision_authority: &'a V,
    observation_authority: &'a O,
}

impl EffectCoordinator {
    /// Constructs a coordinator with caller-lowered finite limits.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            clock: Arc::new(SystemMonotonicClock::new()),
        }
    }

    /// Constructs a coordinator with a trusted monotonic clock.
    ///
    /// This composition-root seam supports deterministic duration enforcement;
    /// production callers should normally use [`Self::new`].
    #[must_use]
    pub fn with_monotonic_clock(limits: Limits, clock: Arc<dyn MonotonicClock>) -> Self {
        Self { limits, clock }
    }

    fn remaining_limits(&self, timer: ExecutionTimer<'_>) -> Result<Limits> {
        let mut limits = self.limits;
        limits.max_execution_duration_ms = timer.remaining_millis()?;
        Ok(limits)
    }

    /// Coordinates one request or exact replay.
    pub fn coordinate<
        S: EffectStore,
        A: AuthorizedEffectAdapter,
        V: DecisionAuthority,
        O: ObservationAuthority,
    >(
        &self,
        lock: &ValidatedLock,
        submission: &Submission,
        store: &mut S,
        adapter: &mut A,
        authority_verifier: &V,
        observation_authority: &O,
    ) -> Result<Outcome> {
        let context = CoordinationContext {
            timer: ExecutionTimer::start(
                self.limits.max_execution_duration_ms,
                self.clock.as_ref(),
            )?,
            decision_authority: authority_verifier,
            observation_authority,
        };
        let (request, authority_current) = self.validate(
            lock,
            submission,
            adapter.manifest(),
            authority_verifier,
            context.timer,
        )?;
        context.timer.check()?;
        if let Some(existing) = store
            .load(request.key(), self.remaining_limits(context.timer)?)
            .map_err(map_store)?
        {
            context.timer.check()?;
            self.validate_record_for_lock(lock, &existing, &context)?;
            if existing.request.canonical_bytes != request.canonical_bytes {
                return Err(Error::KeyConflict);
            }
            return self.resume(lock, submission, existing, store, adapter, &context);
        }
        if !authority_current {
            return Err(Error::AuthorityExpired);
        }
        let record = match store
            .reserve_or_load(&request, self.remaining_limits(context.timer)?)
            .map_err(map_store)?
        {
            Reservation::Inserted(record) | Reservation::Existing(record) => record,
            Reservation::Conflict => return Err(Error::KeyConflict),
        };
        context.timer.check()?;
        self.validate_record_for_lock(lock, &record, &context)?;
        if record.request.canonical_bytes != request.canonical_bytes {
            return Err(Error::KeyConflict);
        }
        self.resume(lock, submission, record, store, adapter, &context)
    }

    /// Links business-effect truth only through an exact lock-admitted observation authority.
    pub fn observe<S: EffectStore, O: ObservationAuthority, V: DecisionAuthority>(
        &self,
        lock: &ValidatedLock,
        key: &IdempotencyKey,
        observation: &Observation,
        store: &mut S,
        authority: &O,
        decision_authority: &V,
    ) -> Result<Outcome> {
        let context = CoordinationContext {
            timer: ExecutionTimer::start(
                self.limits.max_execution_duration_ms,
                self.clock.as_ref(),
            )?,
            decision_authority,
            observation_authority: authority,
        };
        self.validate_identity_bytes(observation.id.as_str())?;
        self.validate_identity_bytes(observation.request_id.as_str())?;
        if let Some(external_effect_id) = &observation.external_effect_id {
            self.validate_identity_bytes(external_effect_id.as_str())?;
        }
        let observation_bytes =
            u64::try_from(observation.payload.len()).map_err(|_| Error::IntegerOverflow)?;
        if observation_bytes > self.limits.max_observation_bytes {
            return Err(Error::ResourceExhausted(Resource::ObservationBytes));
        }
        let admission = lock
            .admit_capability(authority.selection())
            .map_err(|_| Error::AdapterMismatch)?;
        if admission.contract() != OBSERVATION_AUTHORITY_CONTRACT {
            return Err(Error::AdapterMismatch);
        }
        if &observation.authority != authority.selection() {
            return Err(Error::ForeignObservation);
        }
        context.timer.check()?;
        let record = store
            .load(key, self.remaining_limits(context.timer)?)
            .map_err(map_store)?
            .ok_or(Error::ForeignObservation)?;
        context.timer.check()?;
        self.validate_record_for_lock(lock, &record, &context)?;
        if observation.request_id != *record.request.id() {
            return Err(Error::ForeignObservation);
        }
        if let State::Delivered(receipt) = &record.state {
            if observation.external_effect_id.as_ref() != Some(&receipt.external_effect_id) {
                return Err(Error::ForeignObservation);
            }
        }
        context.timer.check()?;
        authority
            .verify(observation, context.timer.call_budget()?)
            .map_err(|_| Error::ForeignObservation)?;
        context.timer.check()?;
        let linked = store
            .link_observation(
                key,
                record.revision,
                observation,
                self.remaining_limits(context.timer)?,
            )
            .map_err(map_store)?;
        context.timer.check()?;
        self.validate_append(
            &record,
            &linked,
            &Event::ObservationLinked(Box::new(observation.clone())),
        )?;
        self.validate_record_for_lock(lock, &linked, &context)?;
        match linked.state {
            State::EffectObserved {
                observation: stored,
                ..
            } if stored.as_ref() == observation => Ok(Outcome::EffectObserved(*stored)),
            _ => Err(Error::PersistenceFailure),
        }
    }

    fn validate<V: DecisionAuthority>(
        &self,
        lock: &ValidatedLock,
        submission: &Submission,
        adapter_manifest: &AdapterManifest,
        authority_verifier: &V,
        timer: ExecutionTimer<'_>,
    ) -> Result<(EffectRequest, bool)> {
        let admitted = lock
            .admit_capability(&submission.manifest.selection)
            .map_err(|_| Error::AdapterMismatch)?;
        if admitted.contract() != EFFECT_ADAPTER_CONTRACT {
            return Err(Error::AdapterMismatch);
        }
        let authority = submission
            .authority
            .as_ref()
            .ok_or(Error::AuthorityMissing)?;
        if matches!(authority.scope, AuthorityScope::All) {
            return Err(Error::AuthorityOverBroad);
        }
        let AuthorityScope::Exact(authority_scope) = &authority.scope else {
            return Err(Error::AuthorityOverBroad);
        };
        let recommendation = &submission.recommendation;
        let decision = &submission.decision;
        let identity_values = [
            recommendation.id.as_str(),
            recommendation.digest.as_str(),
            recommendation.action.as_str(),
            recommendation.subject.as_str(),
            recommendation.scope.as_str(),
            decision.id.as_str(),
            decision.recommendation_digest.as_str(),
            decision.argument_digest.as_str(),
            authority.id.as_str(),
            authority.actor.as_str(),
            authority.issuer.role().as_str(),
            authority.issuer.provider().as_str(),
            authority.issuer.build().as_str(),
            authority.issuer.contract().as_str(),
            authority.issuer.options().as_str(),
            submission.key.as_str(),
            submission.operation.as_str(),
            submission.manifest.idempotency_field.as_str(),
            submission.manifest.selection.role().as_str(),
            submission.manifest.selection.provider().as_str(),
            submission.manifest.selection.build().as_str(),
            submission.manifest.selection.contract().as_str(),
            submission.manifest.selection.options().as_str(),
        ];
        for value in identity_values {
            self.validate_identity_bytes(value)?;
        }
        if let ReconciliationSupport::Operation(operation) = &submission.manifest.reconciliation {
            self.validate_identity_bytes(operation.as_str())?;
        }
        if let Some(decision) = &decision.supersedes {
            self.validate_identity_bytes(decision.as_str())?;
        }
        if let Some(request) = &submission.supersedes_request {
            self.validate_identity_bytes(request.as_str())?;
        }
        let argument_bytes = u64::try_from(submission.canonical_arguments.len())
            .map_err(|_| Error::IntegerOverflow)?;
        if argument_bytes > self.limits.max_argument_bytes {
            return Err(Error::ResourceExhausted(Resource::ArgumentBytes));
        }
        for artifact in [&decision.artifact, &authority.artifact] {
            self.validate_identity_bytes(&artifact.contract_version)?;
            if owner_artifact_envelope_len(artifact)? > self.limits.max_owner_artifact_bytes {
                return Err(Error::ResourceExhausted(Resource::OwnerArtifactBytes));
            }
        }
        if decision.artifact.contract_version != DECISION_AUTHORITY_CONTRACT
            || authority.artifact.contract_version != GRANT_AUTHORITY_CONTRACT
            || decision.id.as_str() != digest_hex(&decision.artifact.canonical_bytes)
            || authority.id.as_str() != digest_hex(&authority.artifact.canonical_bytes)
        {
            return Err(Error::DecisionMismatch);
        }
        if adapter_manifest != &submission.manifest
            || submission.decision.adapter != adapter_manifest.selection
            || submission.manifest.idempotency_field.trim().is_empty()
        {
            return Err(Error::AdapterMismatch);
        }
        if decision.recommendation_id != recommendation.id
            || decision.recommendation_digest != recommendation.digest
            || decision.action != recommendation.action
            || decision.subject != recommendation.subject
            || decision.scope != recommendation.scope
            || decision.operation != submission.operation
            || decision.argument_digest != digest_hex(&submission.canonical_arguments)
            || decision.supersedes.as_ref() == Some(&decision.id)
        {
            return Err(Error::DecisionMismatch);
        }
        if decision.authority_grant_id != authority.id
            || decision.authority != *authority_verifier.decision_selection()
            || authority.issuer != *authority_verifier.grant_selection()
            || decision.actor != authority.actor
            || decision.action != authority.action
            || decision.subject != authority.subject
            || &decision.scope != authority_scope
        {
            return Err(Error::AuthorityMismatch);
        }
        let authority_admission = lock
            .admit_capability(authority_verifier.decision_selection())
            .map_err(|_| Error::AuthorityMismatch)?;
        if authority_admission.contract() != DECISION_AUTHORITY_CONTRACT {
            return Err(Error::AuthorityMismatch);
        }
        let grant_admission = lock
            .admit_capability(authority_verifier.grant_selection())
            .map_err(|_| Error::AuthorityMismatch)?;
        if grant_admission.contract() != GRANT_AUTHORITY_CONTRACT {
            return Err(Error::AuthorityMismatch);
        }
        timer.check()?;
        let authority_status = authority_verifier
            .verify(recommendation, decision, authority, timer.call_budget()?)
            .map_err(|_| Error::AuthorityMismatch)?;
        timer.check()?;
        let canonical_bytes = canonical_request_bytes(
            submission,
            admitted.lock_identity(),
            self.limits.max_request_bytes,
        )?;
        let request_bytes =
            u64::try_from(canonical_bytes.len()).map_err(|_| Error::IntegerOverflow)?;
        if request_bytes > self.limits.max_request_bytes {
            return Err(Error::ResourceExhausted(Resource::RequestBytes));
        }
        let request_id = {
            ensure_digest_identity_capacity(self.limits.max_identity_bytes)?;
            EffectRequestId::new(digest_hex(&canonical_bytes), self.limits.max_identity_bytes)?
        };
        if submission.supersedes_request.as_ref() == Some(&request_id) {
            return Err(Error::DecisionMismatch);
        }
        Ok((
            EffectRequest {
                id: request_id,
                lock_identity: admitted.lock_identity(),
                recommendation: recommendation.clone(),
                decision: decision.clone(),
                authority: authority.clone(),
                key: submission.key.clone(),
                selection: submission.manifest.selection.clone(),
                authority_grant_id: authority.id.clone(),
                operation: submission.operation.clone(),
                canonical_arguments: submission.canonical_arguments.clone(),
                supersedes: submission.supersedes_request.clone(),
                canonical_bytes,
            },
            authority_status == AuthorityStatus::Current,
        ))
    }

    fn resume<
        S: EffectStore,
        A: AuthorizedEffectAdapter,
        O: ObservationAuthority,
        V: DecisionAuthority,
    >(
        &self,
        lock: &ValidatedLock,
        submission: &Submission,
        record: Record,
        store: &mut S,
        adapter: &mut A,
        context: &CoordinationContext<'_, V, O>,
    ) -> Result<Outcome> {
        self.validate_record_for_lock(lock, &record, context)?;
        match record.state.clone() {
            State::Delivered(receipt) => Ok(Outcome::Delivered(receipt)),
            State::EffectObserved { observation, .. } => Ok(Outcome::EffectObserved(*observation)),
            State::Pending => self.prepare_and_dispatch(lock, record, store, adapter, context),
            State::NotDelivered(_) => {
                if submission.manifest.duplicate_guarantee
                    == DuplicateGuarantee::SameKeyAndBytesSameEffect
                {
                    self.prepare_and_dispatch(lock, record, store, adapter, context)
                        .map_err(|error| {
                            if error == Error::AuthorityExpired {
                                Error::RetryUnsupported
                            } else {
                                error
                            }
                        })
                } else {
                    Err(Error::RetryUnsupported)
                }
            }
            State::Open(attempt) | State::DeliveryUnknown(attempt) => {
                self.reconcile(lock, submission, record, attempt, store, adapter, context)
            }
        }
    }

    fn reconcile<
        S: EffectStore,
        A: AuthorizedEffectAdapter,
        V: DecisionAuthority,
        O: ObservationAuthority,
    >(
        &self,
        lock: &ValidatedLock,
        submission: &Submission,
        record: Record,
        attempt: AttemptId,
        store: &mut S,
        adapter: &mut A,
        context: &CoordinationContext<'_, V, O>,
    ) -> Result<Outcome> {
        let permit = ReconciliationPermit {
            request_id: record.request.id.clone(),
            attempt_id: attempt.clone(),
            lock_identity: lock.identity(),
            max_receipt_bytes: self.limits.max_receipt_bytes,
            remaining_duration_ms: context.timer.remaining_millis()?,
        };
        let result = match &submission.manifest.reconciliation {
            ReconciliationSupport::None => ReconciliationResult::Unavailable,
            ReconciliationSupport::Operation(_) => {
                match adapter.reconcile(&permit, &record.request) {
                    Ok(result) => result,
                    Err(AdapterError::Unavailable) => {
                        return self.persist_delivery_unknown(
                            &record,
                            &attempt,
                            DeliveryUnknownCause::ReconciliationUnavailable,
                            store,
                            context.timer,
                        );
                    }
                    Err(AdapterError::ContractViolation) => {
                        return self.persist_delivery_unknown(
                            &record,
                            &attempt,
                            DeliveryUnknownCause::ReconciliationContractViolation,
                            store,
                            context.timer,
                        );
                    }
                }
            }
        };
        match result {
            ReconciliationResult::Accepted(receipt) => {
                let receipt = *receipt;
                if self.validate_receipt(&record.request, &receipt).is_err() {
                    return self.persist_delivery_unknown(
                        &record,
                        &attempt,
                        DeliveryUnknownCause::InvalidReceipt,
                        store,
                        context.timer,
                    );
                }
                let limits = match self.remaining_limits(context.timer) {
                    Ok(limits) => limits,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                        });
                    }
                };
                let persisted = store.record_receipt(
                    record.request.key(),
                    record.revision,
                    &attempt,
                    &receipt,
                    limits,
                );
                if context.timer.check().is_err() {
                    return Ok(Outcome::DeliveryUnknown {
                        attempt,
                        cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                    });
                }
                match persisted {
                    Ok(persisted)
                        if self
                            .validate_append(
                                &record,
                                &persisted,
                                &Event::ReceiptPersisted(Box::new(receipt.clone())),
                            )
                            .is_ok()
                            && persisted.state == State::Delivered(receipt.clone()) =>
                    {
                        Ok(Outcome::Reconciled(receipt))
                    }
                    Ok(_) | Err(_) => self.persist_delivery_unknown(
                        &record,
                        &attempt,
                        DeliveryUnknownCause::ReceiptPersistenceFailure,
                        store,
                        context.timer,
                    ),
                }
            }
            result @ (ReconciliationResult::NotDelivered
            | ReconciliationResult::Inconclusive
            | ReconciliationResult::Unavailable) => {
                let disposition = match result {
                    ReconciliationResult::NotDelivered => ReconciliationDisposition::NotDelivered,
                    ReconciliationResult::Inconclusive => ReconciliationDisposition::Inconclusive,
                    ReconciliationResult::Unavailable => ReconciliationDisposition::Unavailable,
                    ReconciliationResult::Accepted(_) => return Err(Error::PersistenceFailure),
                };
                let limits = match self.remaining_limits(context.timer) {
                    Ok(limits) => limits,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                        });
                    }
                };
                let reconciled = match store.record_reconciliation(
                    record.request.key(),
                    record.revision,
                    &attempt,
                    disposition,
                    limits,
                ) {
                    Ok(reconciled) => reconciled,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                        });
                    }
                };
                if context.timer.check().is_err()
                    || self
                        .validate_append(
                            &record,
                            &reconciled,
                            &Event::ReconciliationRecorded {
                                attempt: attempt.clone(),
                                disposition,
                            },
                        )
                        .is_err()
                {
                    return Ok(Outcome::DeliveryUnknown {
                        attempt,
                        cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                    });
                }
                let expected_state = match disposition {
                    ReconciliationDisposition::NotDelivered => State::NotDelivered(attempt.clone()),
                    ReconciliationDisposition::Inconclusive
                    | ReconciliationDisposition::Unavailable => {
                        State::DeliveryUnknown(attempt.clone())
                    }
                };
                if reconciled.state != expected_state {
                    return Err(Error::PersistenceFailure);
                }
                if submission.manifest.duplicate_guarantee
                    != DuplicateGuarantee::SameKeyAndBytesSameEffect
                    || !self.authority_is_current(
                        &reconciled.request,
                        context.decision_authority,
                        context.timer,
                    )?
                {
                    if matches!(result, ReconciliationResult::NotDelivered) {
                        return Err(Error::RetryUnsupported);
                    }
                    let cause = match disposition {
                        ReconciliationDisposition::Inconclusive => {
                            DeliveryUnknownCause::InconclusiveReconciliation
                        }
                        ReconciliationDisposition::Unavailable => {
                            DeliveryUnknownCause::ReconciliationUnavailable
                        }
                        ReconciliationDisposition::NotDelivered => {
                            return Err(Error::RetryUnsupported);
                        }
                    };
                    return match reconciled.state {
                        State::DeliveryUnknown(attempt) => {
                            Ok(Outcome::DeliveryUnknown { attempt, cause })
                        }
                        _ => Err(Error::PersistenceFailure),
                    };
                }
                self.prepare_and_dispatch(lock, reconciled, store, adapter, context)
            }
        }
    }

    fn prepare_and_dispatch<
        S: EffectStore,
        A: AuthorizedEffectAdapter,
        V: DecisionAuthority,
        O: ObservationAuthority,
    >(
        &self,
        lock: &ValidatedLock,
        record: Record,
        store: &mut S,
        adapter: &mut A,
        context: &CoordinationContext<'_, V, O>,
    ) -> Result<Outcome> {
        context.timer.check()?;
        self.require_current_authority(&record.request, context.decision_authority, context.timer)?;
        let required_history = u64::try_from(record.history.len())
            .map_err(|_| Error::IntegerOverflow)?
            .checked_add(2)
            .ok_or(Error::IntegerOverflow)?;
        if required_history > self.limits.max_history_events {
            return Err(Error::ResourceExhausted(Resource::HistoryEvents));
        }
        let ordinal = record
            .history
            .iter()
            .filter(|event| matches!(event, Event::AttemptPrepared(_)))
            .count();
        let ordinal = u64::try_from(ordinal)
            .map_err(|_| Error::IntegerOverflow)?
            .checked_add(1)
            .ok_or(Error::IntegerOverflow)?;
        if ordinal > self.limits.max_attempts {
            return Err(Error::ResourceExhausted(Resource::Attempts));
        }
        let attempt_suffix_bytes = b":attempt:"
            .len()
            .checked_add(decimal_digits(ordinal))
            .ok_or(Error::IntegerOverflow)?;
        let attempt_bytes = record
            .request
            .id
            .as_str()
            .len()
            .checked_add(attempt_suffix_bytes)
            .ok_or(Error::IntegerOverflow)?;
        if u64::try_from(attempt_bytes).map_err(|_| Error::IntegerOverflow)?
            > self.limits.max_identity_bytes
        {
            return Err(Error::ResourceExhausted(Resource::IdentityBytes));
        }
        let attempt = AttemptId::new(
            format!("{}:attempt:{ordinal}", record.request.id.as_str()),
            self.limits.max_identity_bytes,
        )?;
        let prepared = store
            .prepare_attempt(
                record.request.key(),
                record.revision,
                &attempt,
                self.remaining_limits(context.timer)?,
            )
            .map_err(map_store)?;
        context.timer.check()?;
        self.validate_append(&record, &prepared, &Event::AttemptPrepared(attempt.clone()))?;
        if prepared.state != State::Open(attempt.clone()) {
            return Err(Error::PersistenceFailure);
        }
        self.invoke_adapter(prepared, attempt, lock.identity(), store, adapter, context)
    }

    fn invoke_adapter<
        S: EffectStore,
        A: AuthorizedEffectAdapter,
        V: DecisionAuthority,
        O: ObservationAuthority,
    >(
        &self,
        prepared: Record,
        attempt: AttemptId,
        lock_identity: LockIdentity,
        store: &mut S,
        adapter: &mut A,
        context: &CoordinationContext<'_, V, O>,
    ) -> Result<Outcome> {
        context.timer.check()?;
        self.require_current_authority(
            &prepared.request,
            context.decision_authority,
            context.timer,
        )?;
        context.timer.check()?;
        let permit = DispatchPermit {
            request_id: prepared.request.id.clone(),
            attempt_id: attempt.clone(),
            lock_identity,
            max_receipt_bytes: self.limits.max_receipt_bytes,
            remaining_duration_ms: context.timer.remaining_millis()?,
        };
        let result = match adapter.dispatch(&permit, &prepared.request) {
            Ok(result) => result,
            Err(_) => {
                return self.persist_delivery_unknown(
                    &prepared,
                    &attempt,
                    DeliveryUnknownCause::AdapterFailure,
                    store,
                    context.timer,
                );
            }
        };
        match result {
            DispatchResult::Accepted(receipt) => {
                let receipt = *receipt;
                if self.validate_receipt(&prepared.request, &receipt).is_err() {
                    return self.persist_delivery_unknown(
                        &prepared,
                        &attempt,
                        DeliveryUnknownCause::InvalidReceipt,
                        store,
                        context.timer,
                    );
                }
                let limits = match self.remaining_limits(context.timer) {
                    Ok(limits) => limits,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                        });
                    }
                };
                let persisted = store.record_receipt(
                    prepared.request.key(),
                    prepared.revision,
                    &attempt,
                    &receipt,
                    limits,
                );
                if context.timer.check().is_err() {
                    return Ok(Outcome::DeliveryUnknown {
                        attempt,
                        cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                    });
                }
                match persisted {
                    Ok(record) => {
                        if self
                            .validate_append(
                                &prepared,
                                &record,
                                &Event::ReceiptPersisted(Box::new(receipt.clone())),
                            )
                            .is_err()
                            || record.state != State::Delivered(receipt.clone())
                        {
                            return Ok(Outcome::DeliveryUnknown {
                                attempt,
                                cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                            });
                        }
                        match record.state {
                            State::Delivered(receipt) => Ok(Outcome::Delivered(receipt)),
                            _ => Ok(Outcome::DeliveryUnknown {
                                attempt,
                                cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                            }),
                        }
                    }
                    Err(_) => Ok(Outcome::DeliveryUnknown {
                        attempt,
                        cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
                    }),
                }
            }
            DispatchResult::NotAccepted => {
                let limits = match self.remaining_limits(context.timer) {
                    Ok(limits) => limits,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                        });
                    }
                };
                let not_delivered = match store.record_dispatch_not_accepted(
                    prepared.request.key(),
                    prepared.revision,
                    &attempt,
                    limits,
                ) {
                    Ok(not_delivered) => not_delivered,
                    Err(_) => {
                        return Ok(Outcome::DeliveryUnknown {
                            attempt,
                            cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                        });
                    }
                };
                if context.timer.check().is_err()
                    || self
                        .validate_append(
                            &prepared,
                            &not_delivered,
                            &Event::DispatchNotAccepted(attempt.clone()),
                        )
                        .is_err()
                {
                    return Ok(Outcome::DeliveryUnknown {
                        attempt,
                        cause: DeliveryUnknownCause::PostBoundaryPersistenceFailure,
                    });
                }
                if not_delivered.state != State::NotDelivered(attempt.clone()) {
                    return Err(Error::PersistenceFailure);
                }
                match not_delivered.state {
                    State::NotDelivered(attempt) => Ok(Outcome::NotDelivered(attempt)),
                    _ => Err(Error::PersistenceFailure),
                }
            }
            DispatchResult::Indeterminate => self.persist_delivery_unknown(
                &prepared,
                &attempt,
                DeliveryUnknownCause::IndeterminateTransport,
                store,
                context.timer,
            ),
        }
    }

    fn persist_delivery_unknown<S: EffectStore>(
        &self,
        prepared: &Record,
        attempt: &AttemptId,
        cause: DeliveryUnknownCause,
        store: &mut S,
        timer: ExecutionTimer<'_>,
    ) -> Result<Outcome> {
        if prepared.state == State::DeliveryUnknown(attempt.clone()) {
            return Ok(Outcome::DeliveryUnknown {
                attempt: attempt.clone(),
                cause,
            });
        }
        let limits = match self.remaining_limits(timer) {
            Ok(limits) => limits,
            Err(_) => {
                return Ok(Outcome::DeliveryUnknown {
                    attempt: attempt.clone(),
                    cause,
                });
            }
        };
        let unknown = store.record_delivery_unknown(
            prepared.request.key(),
            prepared.revision,
            attempt,
            cause,
            limits,
        );
        if timer.check().is_err() {
            return Ok(Outcome::DeliveryUnknown {
                attempt: attempt.clone(),
                cause,
            });
        }
        match unknown {
            Ok(record)
                if self
                    .validate_append(
                        prepared,
                        &record,
                        &Event::DeliveryUnknownRecorded {
                            attempt: attempt.clone(),
                            cause,
                        },
                    )
                    .is_ok()
                    && record.state == State::DeliveryUnknown(attempt.clone()) =>
            {
                match record.state {
                    State::DeliveryUnknown(attempt) => {
                        Ok(Outcome::DeliveryUnknown { attempt, cause })
                    }
                    _ => Ok(Outcome::DeliveryUnknown {
                        attempt: attempt.clone(),
                        cause,
                    }),
                }
            }
            Ok(_) | Err(_) => Ok(Outcome::DeliveryUnknown {
                attempt: attempt.clone(),
                cause,
            }),
        }
    }

    fn validate_receipt(&self, request: &EffectRequest, receipt: &Receipt) -> Result<()> {
        self.validate_identity_bytes(receipt.id.as_str())?;
        self.validate_identity_bytes(receipt.external_effect_id.as_str())?;
        let payload_bytes =
            u64::try_from(receipt.payload.len()).map_err(|_| Error::IntegerOverflow)?;
        if payload_bytes > self.limits.max_receipt_bytes {
            return Err(Error::ResourceExhausted(Resource::ReceiptBytes));
        }
        if receipt.request_id != request.id || receipt.selection != request.selection {
            return Err(Error::MalformedReceipt);
        }
        Ok(())
    }

    fn require_current_authority<V: DecisionAuthority>(
        &self,
        request: &EffectRequest,
        authority: &V,
        timer: ExecutionTimer<'_>,
    ) -> Result<()> {
        if self.authority_is_current(request, authority, timer)? {
            Ok(())
        } else {
            Err(Error::AuthorityExpired)
        }
    }

    fn authority_is_current<V: DecisionAuthority>(
        &self,
        request: &EffectRequest,
        authority: &V,
        timer: ExecutionTimer<'_>,
    ) -> Result<bool> {
        match authority
            .verify(
                &request.recommendation,
                &request.decision,
                &request.authority,
                timer.call_budget()?,
            )
            .map_err(|_| Error::AuthorityMismatch)?
        {
            AuthorityStatus::Current => Ok(true),
            AuthorityStatus::OutsideValidity => Ok(false),
        }
    }

    fn validate_identity_bytes(&self, value: &str) -> Result<()> {
        let length = u64::try_from(value.len()).map_err(|_| Error::IntegerOverflow)?;
        if length > self.limits.max_identity_bytes {
            return Err(Error::ResourceExhausted(Resource::IdentityBytes));
        }
        Ok(())
    }

    fn validate_record(
        &self,
        record: &Record,
        expected_request: Option<&EffectRequest>,
    ) -> Result<()> {
        if expected_request.is_some_and(|expected| expected != &record.request) {
            return Err(Error::PersistenceFailure);
        }
        let history_len =
            u64::try_from(record.history.len()).map_err(|_| Error::IntegerOverflow)?;
        if history_len == 0 || history_len > self.limits.max_history_events {
            return Err(Error::PersistenceFailure);
        }
        if record.revision != history_len {
            return Err(Error::PersistenceFailure);
        }
        validate_persisted_request_fields(&record.request, self.limits)?;
        self.validate_identity_bytes(record.request.id.as_str())?;
        self.validate_identity_bytes(record.request.key.as_str())?;
        self.validate_identity_bytes(record.request.operation.as_str())?;
        if let Some(prior) = &record.request.supersedes {
            self.validate_identity_bytes(prior.as_str())?;
        }
        let argument_bytes = u64::try_from(record.request.canonical_arguments.len())
            .map_err(|_| Error::IntegerOverflow)?;
        if argument_bytes > self.limits.max_argument_bytes {
            return Err(Error::ResourceExhausted(Resource::ArgumentBytes));
        }

        let mut projection: Option<JournalProjection> = None;
        let mut attempts = 0_u64;
        for event in &record.history {
            projection = Some(match event {
                Event::RequestPersisted if projection.is_none() => JournalProjection::Pending,
                Event::RequestPersisted => return Err(Error::PersistenceFailure),
                Event::AttemptPrepared(attempt)
                    if matches!(
                        projection,
                        Some(
                            JournalProjection::Pending
                                | JournalProjection::NotDelivered(_)
                                | JournalProjection::DeliveryUnknown(_)
                        )
                    ) =>
                {
                    attempts = attempts.checked_add(1).ok_or(Error::IntegerOverflow)?;
                    self.validate_identity_bytes(attempt.as_str())?;
                    JournalProjection::Open(attempt.clone())
                }
                Event::AttemptPrepared(_) => return Err(Error::PersistenceFailure),
                Event::DispatchNotAccepted(attempt)
                    if matches!(
                        &projection,
                        Some(JournalProjection::Open(current)) if current == attempt
                    ) =>
                {
                    JournalProjection::NotDelivered(attempt.clone())
                }
                Event::DispatchNotAccepted(_) => return Err(Error::PersistenceFailure),
                Event::ReconciliationRecorded {
                    attempt,
                    disposition,
                } if matches!(
                    &projection,
                    Some(JournalProjection::Open(current) | JournalProjection::DeliveryUnknown(current))
                        if current == attempt
                ) =>
                {
                    match disposition {
                        ReconciliationDisposition::NotDelivered => {
                            JournalProjection::NotDelivered(attempt.clone())
                        }
                        ReconciliationDisposition::Inconclusive
                        | ReconciliationDisposition::Unavailable => {
                            JournalProjection::DeliveryUnknown(attempt.clone())
                        }
                    }
                }
                Event::ReconciliationRecorded { .. } => return Err(Error::PersistenceFailure),
                Event::ReceiptPersisted(receipt)
                    if matches!(
                        projection,
                        Some(JournalProjection::Open(_) | JournalProjection::DeliveryUnknown(_))
                    ) =>
                {
                    self.validate_receipt(&record.request, receipt)?;
                    JournalProjection::Delivered(receipt.clone())
                }
                Event::ReceiptPersisted(_) => return Err(Error::PersistenceFailure),
                Event::DeliveryUnknownRecorded { attempt, .. }
                    if matches!(
                        &projection,
                        Some(
                            JournalProjection::Open(current)
                                | JournalProjection::DeliveryUnknown(current)
                        ) if current == attempt
                    ) =>
                {
                    JournalProjection::DeliveryUnknown(attempt.clone())
                }
                Event::DeliveryUnknownRecorded { .. } => return Err(Error::PersistenceFailure),
                Event::ObservationLinked(observation)
                    if projection.is_some()
                        && !matches!(
                            projection,
                            Some(JournalProjection::EffectObserved { .. })
                        ) =>
                {
                    let receipt = match &projection {
                        Some(JournalProjection::Delivered(receipt)) => Some(receipt.as_ref()),
                        _ => None,
                    };
                    self.validate_observation_for_record(&record.request, receipt, observation)?;
                    let receipt = match &projection {
                        Some(JournalProjection::Delivered(receipt)) => Some(receipt.clone()),
                        _ => None,
                    };
                    JournalProjection::EffectObserved {
                        observation: observation.clone(),
                        receipt,
                    }
                }
                Event::ObservationLinked(_) => return Err(Error::PersistenceFailure),
            });
        }
        if attempts > self.limits.max_attempts {
            return Err(Error::ResourceExhausted(Resource::Attempts));
        }
        let projection = projection.ok_or(Error::PersistenceFailure)?;
        let state_matches = match (&projection, &record.state) {
            (JournalProjection::Pending, State::Pending) => true,
            (JournalProjection::Open(left), State::Open(right))
            | (JournalProjection::NotDelivered(left), State::NotDelivered(right))
            | (JournalProjection::DeliveryUnknown(left), State::DeliveryUnknown(right)) => {
                left == right
            }
            (JournalProjection::Delivered(expected), State::Delivered(receipt)) => {
                expected.as_ref() == receipt
                    && self.validate_receipt(&record.request, receipt).is_ok()
            }
            (
                JournalProjection::EffectObserved {
                    observation: projected_observation,
                    receipt: projected_receipt,
                },
                State::EffectObserved {
                    receipt,
                    observation,
                },
            ) => {
                let receipt_valid = receipt
                    .as_ref()
                    .is_none_or(|receipt| self.validate_receipt(&record.request, receipt).is_ok());
                let receipt_identity_matches = match (projected_receipt, receipt) {
                    (Some(expected), Some(actual)) => expected.as_ref() == actual,
                    (None, None) => true,
                    _ => false,
                };
                projected_observation.as_ref() == observation.as_ref()
                    && receipt_identity_matches
                    && receipt_valid
                    && self
                        .validate_observation_for_record(
                            &record.request,
                            receipt.as_ref(),
                            observation,
                        )
                        .is_ok()
            }
            _ => false,
        };
        if !state_matches {
            return Err(Error::PersistenceFailure);
        }
        Ok(())
    }

    fn validate_record_for_lock<V: DecisionAuthority, O: ObservationAuthority>(
        &self,
        lock: &ValidatedLock,
        record: &Record,
        context: &CoordinationContext<'_, V, O>,
    ) -> Result<()> {
        self.validate_record(record, None)?;
        if record.request.lock_identity != lock.identity() {
            return Err(Error::PersistenceFailure);
        }
        let adapter_admission = lock
            .admit_capability(&record.request.selection)
            .map_err(|_| Error::PersistenceFailure)?;
        if adapter_admission.contract() != EFFECT_ADAPTER_CONTRACT {
            return Err(Error::PersistenceFailure);
        }
        if context.decision_authority.decision_selection() != &record.request.decision.authority
            || context.decision_authority.grant_selection() != &record.request.authority.issuer
        {
            return Err(Error::PersistenceFailure);
        }
        let decision_admission = lock
            .admit_capability(context.decision_authority.decision_selection())
            .map_err(|_| Error::PersistenceFailure)?;
        if decision_admission.contract() != DECISION_AUTHORITY_CONTRACT {
            return Err(Error::PersistenceFailure);
        }
        let grant_admission = lock
            .admit_capability(context.decision_authority.grant_selection())
            .map_err(|_| Error::PersistenceFailure)?;
        if grant_admission.contract() != GRANT_AUTHORITY_CONTRACT {
            return Err(Error::PersistenceFailure);
        }
        context.timer.check()?;
        context
            .decision_authority
            .verify(
                &record.request.recommendation,
                &record.request.decision,
                &record.request.authority,
                context.timer.call_budget()?,
            )
            .map_err(|_| Error::PersistenceFailure)?;
        context.timer.check()?;
        if let State::EffectObserved { observation, .. } = &record.state {
            let authority = context.observation_authority;
            if authority.selection() != &observation.authority {
                return Err(Error::PersistenceFailure);
            }
            let admission = lock
                .admit_capability(&observation.authority)
                .map_err(|_| Error::PersistenceFailure)?;
            if admission.contract() != OBSERVATION_AUTHORITY_CONTRACT {
                return Err(Error::PersistenceFailure);
            }
            context.timer.check()?;
            authority
                .verify(observation, context.timer.call_budget()?)
                .map_err(|_| Error::PersistenceFailure)?;
            context.timer.check()?;
        }
        Ok(())
    }

    fn validate_append(&self, before: &Record, after: &Record, event: &Event) -> Result<()> {
        self.validate_record(after, Some(&before.request))?;
        let expected_revision = before
            .revision
            .checked_add(1)
            .ok_or(Error::IntegerOverflow)?;
        if after.revision != expected_revision
            || after.history.len() != before.history.len().saturating_add(1)
            || !after.history.starts_with(&before.history)
            || after.history.last() != Some(event)
        {
            return Err(Error::PersistenceFailure);
        }
        Ok(())
    }

    fn validate_observation_for_record(
        &self,
        request: &EffectRequest,
        receipt: Option<&Receipt>,
        observation: &Observation,
    ) -> Result<()> {
        self.validate_identity_bytes(observation.id.as_str())?;
        self.validate_identity_bytes(observation.request_id.as_str())?;
        if observation.request_id != request.id {
            return Err(Error::ForeignObservation);
        }
        if let Some(external_effect_id) = &observation.external_effect_id {
            self.validate_identity_bytes(external_effect_id.as_str())?;
        }
        if observation.authority.contract().as_str() != OBSERVATION_AUTHORITY_CONTRACT {
            return Err(Error::ForeignObservation);
        }
        let payload_bytes =
            u64::try_from(observation.payload.len()).map_err(|_| Error::IntegerOverflow)?;
        if payload_bytes > self.limits.max_observation_bytes {
            return Err(Error::ResourceExhausted(Resource::ObservationBytes));
        }
        if let Some(receipt) = receipt {
            if observation.external_effect_id.as_ref() != Some(&receipt.external_effect_id) {
                return Err(Error::ForeignObservation);
            }
        }
        Ok(())
    }
}

fn ensure_digest_identity_capacity(max_identity_bytes: u64) -> Result<()> {
    const SHA256_IDENTITY_BYTES: u64 = 7 + 64;
    if max_identity_bytes < SHA256_IDENTITY_BYTES {
        return Err(Error::ResourceExhausted(Resource::IdentityBytes));
    }
    Ok(())
}

const fn decimal_digits(mut value: u64) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn validate_persisted_request_fields(request: &EffectRequest, limits: Limits) -> Result<()> {
    let request_bytes =
        u64::try_from(request.canonical_bytes.len()).map_err(|_| Error::IntegerOverflow)?;
    if request_bytes > limits.max_request_bytes {
        return Err(Error::ResourceExhausted(Resource::RequestBytes));
    }
    let argument_bytes =
        u64::try_from(request.canonical_arguments.len()).map_err(|_| Error::IntegerOverflow)?;
    if argument_bytes > limits.max_argument_bytes {
        return Err(Error::ResourceExhausted(Resource::ArgumentBytes));
    }
    for value in [
        request.id.as_str(),
        request.key.as_str(),
        request.selection.role().as_str(),
        request.selection.provider().as_str(),
        request.selection.build().as_str(),
        request.selection.contract().as_str(),
        request.selection.options().as_str(),
        request.authority_grant_id.as_str(),
        request.operation.as_str(),
        request.recommendation.id.as_str(),
        request.recommendation.digest.as_str(),
        request.recommendation.action.as_str(),
        request.recommendation.subject.as_str(),
        request.recommendation.scope.as_str(),
        request.decision.id.as_str(),
        request.decision.recommendation_id.as_str(),
        request.decision.recommendation_digest.as_str(),
        request.decision.authority_grant_id.as_str(),
        request.decision.actor.as_str(),
        request.decision.action.as_str(),
        request.decision.subject.as_str(),
        request.decision.scope.as_str(),
        request.decision.operation.as_str(),
        request.decision.argument_digest.as_str(),
        request.decision.artifact.contract_version.as_str(),
        request.decision.authority.role().as_str(),
        request.decision.authority.provider().as_str(),
        request.decision.authority.build().as_str(),
        request.decision.authority.contract().as_str(),
        request.decision.authority.options().as_str(),
        request.authority.id.as_str(),
        request.authority.actor.as_str(),
        request.authority.action.as_str(),
        request.authority.subject.as_str(),
        request.authority.artifact.contract_version.as_str(),
        request.authority.issuer.role().as_str(),
        request.authority.issuer.provider().as_str(),
        request.authority.issuer.build().as_str(),
        request.authority.issuer.contract().as_str(),
        request.authority.issuer.options().as_str(),
    ] {
        if u64::try_from(value.len()).map_err(|_| Error::IntegerOverflow)?
            > limits.max_identity_bytes
        {
            return Err(Error::ResourceExhausted(Resource::IdentityBytes));
        }
    }
    if let Some(prior) = &request.supersedes {
        if u64::try_from(prior.as_str().len()).map_err(|_| Error::IntegerOverflow)?
            > limits.max_identity_bytes
        {
            return Err(Error::ResourceExhausted(Resource::IdentityBytes));
        }
    }
    if let Some(prior) = &request.decision.supersedes {
        if u64::try_from(prior.as_str().len()).map_err(|_| Error::IntegerOverflow)?
            > limits.max_identity_bytes
        {
            return Err(Error::ResourceExhausted(Resource::IdentityBytes));
        }
    }
    ensure_digest_identity_capacity(limits.max_identity_bytes)?;
    if request.id.as_str() != digest_hex(&request.canonical_bytes) {
        return Err(Error::PersistenceFailure);
    }
    let AuthorityScope::Exact(grant_scope) = &request.authority.scope else {
        return Err(Error::PersistenceFailure);
    };
    if request.decision.recommendation_id != request.recommendation.id
        || request.decision.recommendation_digest != request.recommendation.digest
        || request.decision.authority_grant_id != request.authority.id
        || request.authority_grant_id != request.authority.id
        || request.decision.actor != request.authority.actor
        || request.decision.action != request.recommendation.action
        || request.decision.action != request.authority.action
        || request.decision.subject != request.recommendation.subject
        || request.decision.subject != request.authority.subject
        || request.decision.scope != request.recommendation.scope
        || &request.decision.scope != grant_scope
        || request.decision.authority.contract().as_str() != DECISION_AUTHORITY_CONTRACT
        || request.authority.issuer.contract().as_str() != GRANT_AUTHORITY_CONTRACT
        || request.decision.adapter != request.selection
        || request.decision.operation != request.operation
        || request.decision.argument_digest != digest_hex(&request.canonical_arguments)
        || request.decision.supersedes.as_ref() == Some(&request.decision.id)
        || request.decision.artifact.contract_version != DECISION_AUTHORITY_CONTRACT
        || request.authority.artifact.contract_version != GRANT_AUTHORITY_CONTRACT
        || request.decision.id.as_str() != digest_hex(&request.decision.artifact.canonical_bytes)
        || request.authority.id.as_str() != digest_hex(&request.authority.artifact.canonical_bytes)
    {
        return Err(Error::PersistenceFailure);
    }

    let mut input = request.canonical_bytes.as_slice();
    if take_field(&mut input)? != b"quire.effect-request/v1" {
        return Err(Error::PersistenceFailure);
    }
    let lock_identity = take_field(&mut input)?;
    if lock_identity != request.lock_identity.to_hex().as_bytes() {
        return Err(Error::PersistenceFailure);
    }
    for expected in [
        request.selection.role().as_str(),
        request.selection.provider().as_str(),
        request.selection.build().as_str(),
        request.selection.contract().as_str(),
        request.selection.options().as_str(),
    ] {
        if take_field(&mut input)? != expected.as_bytes() {
            return Err(Error::PersistenceFailure);
        }
    }
    for expected in [
        request.authority.issuer.role().as_str(),
        request.authority.issuer.provider().as_str(),
        request.authority.issuer.build().as_str(),
        request.authority.issuer.contract().as_str(),
        request.authority.issuer.options().as_str(),
    ] {
        if take_field(&mut input)? != expected.as_bytes() {
            return Err(Error::PersistenceFailure);
        }
    }
    for expected in [
        request.decision.authority.role().as_str(),
        request.decision.authority.provider().as_str(),
        request.decision.authority.build().as_str(),
        request.decision.authority.contract().as_str(),
        request.decision.authority.options().as_str(),
    ] {
        if take_field(&mut input)? != expected.as_bytes() {
            return Err(Error::PersistenceFailure);
        }
    }
    for expected in [
        request.decision.id.as_str(),
        request.decision.recommendation_id.as_str(),
        request.decision.recommendation_digest.as_str(),
    ] {
        if take_field(&mut input)? != expected.as_bytes() {
            return Err(Error::PersistenceFailure);
        }
    }
    if take_field(&mut input)? != request.authority_grant_id.as_str().as_bytes() {
        return Err(Error::PersistenceFailure);
    }
    for expected in [
        request.authority.actor.as_str(),
        request.authority.action.as_str(),
        request.authority.subject.as_str(),
        request.decision.scope.as_str(),
    ] {
        if take_field(&mut input)? != expected.as_bytes() {
            return Err(Error::PersistenceFailure);
        }
    }
    if take_field(&mut input)? != request.key.as_str().as_bytes()
        || take_field(&mut input)? != request.operation.as_str().as_bytes()
    {
        return Err(Error::PersistenceFailure);
    }
    take_identity_field(&mut input, limits.max_identity_bytes)?;
    for artifact in [&request.decision.artifact, &request.authority.artifact] {
        if take_field(&mut input)? != artifact.contract_version.as_bytes()
            || take_field(&mut input)? != artifact.canonical_bytes
            || take_field(&mut input)? != artifact.attestation
        {
            return Err(Error::PersistenceFailure);
        }
        if owner_artifact_envelope_len(artifact)? > limits.max_owner_artifact_bytes {
            return Err(Error::ResourceExhausted(Resource::OwnerArtifactBytes));
        }
    }
    if take_field(&mut input)? != request.authority.not_before_ms.to_be_bytes()
        || take_field(&mut input)? != request.authority.expires_at_ms.to_be_bytes()
    {
        return Err(Error::PersistenceFailure);
    }
    match take_field(&mut input)? {
        b"none" | b"same-key-same-bytes-same-effect" => {}
        _ => return Err(Error::PersistenceFailure),
    }
    match (
        take_field(&mut input)?,
        request.decision.supersedes.as_ref(),
    ) {
        (b"no-prior-decision", None) => {}
        (b"supersedes-decision", Some(expected)) => {
            if take_field(&mut input)? != expected.as_str().as_bytes() {
                return Err(Error::PersistenceFailure);
            }
        }
        _ => return Err(Error::PersistenceFailure),
    }
    match (take_field(&mut input)?, request.supersedes.as_ref()) {
        (b"no-prior-request", None) => {}
        (b"supersedes-request", Some(expected)) => {
            if take_field(&mut input)? != expected.as_str().as_bytes() {
                return Err(Error::PersistenceFailure);
            }
        }
        _ => return Err(Error::PersistenceFailure),
    }
    match take_field(&mut input)? {
        b"none" => {}
        b"operation" => {
            take_identity_field(&mut input, limits.max_identity_bytes)?;
        }
        _ => return Err(Error::PersistenceFailure),
    }
    if take_field(&mut input)? != request.canonical_arguments || !input.is_empty() {
        return Err(Error::PersistenceFailure);
    }
    Ok(())
}

fn take_field<'a>(input: &mut &'a [u8]) -> Result<&'a [u8]> {
    let length_bytes: [u8; 8] = input
        .get(..8)
        .ok_or(Error::PersistenceFailure)?
        .try_into()
        .map_err(|_| Error::PersistenceFailure)?;
    let length =
        usize::try_from(u64::from_be_bytes(length_bytes)).map_err(|_| Error::IntegerOverflow)?;
    let value = input
        .get(8..8_usize.checked_add(length).ok_or(Error::IntegerOverflow)?)
        .ok_or(Error::PersistenceFailure)?;
    *input = input
        .get(8_usize.checked_add(length).ok_or(Error::IntegerOverflow)?..)
        .ok_or(Error::PersistenceFailure)?;
    Ok(value)
}

fn owner_artifact_envelope_len(artifact: &OwnerArtifact) -> Result<u64> {
    [
        artifact.contract_version.len(),
        artifact.canonical_bytes.len(),
        artifact.attestation.len(),
    ]
    .into_iter()
    .try_fold(0_u64, |total, length| {
        total
            .checked_add(8)
            .and_then(|value| value.checked_add(u64::try_from(length).ok()?))
            .ok_or(Error::IntegerOverflow)
    })
}

fn take_identity_field<'a>(input: &mut &'a [u8], max_identity_bytes: u64) -> Result<&'a str> {
    let value = take_field(input)?;
    if u64::try_from(value.len()).map_err(|_| Error::IntegerOverflow)? > max_identity_bytes {
        return Err(Error::ResourceExhausted(Resource::IdentityBytes));
    }
    let value = std::str::from_utf8(value).map_err(|_| Error::PersistenceFailure)?;
    if value.trim().is_empty() {
        return Err(Error::PersistenceFailure);
    }
    Ok(value)
}

fn canonical_request_bytes(
    submission: &Submission,
    lock_identity: LockIdentity,
    max_request_bytes: u64,
) -> Result<Vec<u8>> {
    let authority = submission
        .authority
        .as_ref()
        .ok_or(Error::AuthorityMissing)?;
    let mut output = Vec::new();
    push_field(&mut output, b"quire.effect-request/v1", max_request_bytes)?;
    push_field(
        &mut output,
        lock_identity.to_hex().as_bytes(),
        max_request_bytes,
    )?;
    for value in [
        submission.manifest.selection.role().as_str(),
        submission.manifest.selection.provider().as_str(),
        submission.manifest.selection.build().as_str(),
        submission.manifest.selection.contract().as_str(),
        submission.manifest.selection.options().as_str(),
        authority.issuer.role().as_str(),
        authority.issuer.provider().as_str(),
        authority.issuer.build().as_str(),
        authority.issuer.contract().as_str(),
        authority.issuer.options().as_str(),
        submission.decision.authority.role().as_str(),
        submission.decision.authority.provider().as_str(),
        submission.decision.authority.build().as_str(),
        submission.decision.authority.contract().as_str(),
        submission.decision.authority.options().as_str(),
        submission.decision.id.as_str(),
        submission.decision.recommendation_id.as_str(),
        submission.decision.recommendation_digest.as_str(),
        authority.id.as_str(),
        authority.actor.as_str(),
        authority.action.as_str(),
        authority.subject.as_str(),
        submission.decision.scope.as_str(),
        submission.key.as_str(),
        submission.operation.as_str(),
        submission.manifest.idempotency_field.as_str(),
    ] {
        push_field(&mut output, value.as_bytes(), max_request_bytes)?;
    }
    for artifact in [&submission.decision.artifact, &authority.artifact] {
        push_field(
            &mut output,
            artifact.contract_version.as_bytes(),
            max_request_bytes,
        )?;
        push_field(&mut output, &artifact.canonical_bytes, max_request_bytes)?;
        push_field(&mut output, &artifact.attestation, max_request_bytes)?;
    }
    push_field(
        &mut output,
        &authority.not_before_ms.to_be_bytes(),
        max_request_bytes,
    )?;
    push_field(
        &mut output,
        &authority.expires_at_ms.to_be_bytes(),
        max_request_bytes,
    )?;
    let guarantee = match submission.manifest.duplicate_guarantee {
        DuplicateGuarantee::None => b"none".as_slice(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect => {
            b"same-key-same-bytes-same-effect".as_slice()
        }
    };
    push_field(&mut output, guarantee, max_request_bytes)?;
    match &submission.decision.supersedes {
        Some(decision) => {
            push_field(&mut output, b"supersedes-decision", max_request_bytes)?;
            push_field(&mut output, decision.as_str().as_bytes(), max_request_bytes)?;
        }
        None => push_field(&mut output, b"no-prior-decision", max_request_bytes)?,
    }
    match &submission.supersedes_request {
        Some(request) => {
            push_field(&mut output, b"supersedes-request", max_request_bytes)?;
            push_field(&mut output, request.as_str().as_bytes(), max_request_bytes)?;
        }
        None => push_field(&mut output, b"no-prior-request", max_request_bytes)?,
    }
    match &submission.manifest.reconciliation {
        ReconciliationSupport::None => push_field(&mut output, b"none", max_request_bytes)?,
        ReconciliationSupport::Operation(operation) => {
            push_field(&mut output, b"operation", max_request_bytes)?;
            push_field(
                &mut output,
                operation.as_str().as_bytes(),
                max_request_bytes,
            )?;
        }
    }
    push_field(
        &mut output,
        &submission.canonical_arguments,
        max_request_bytes,
    )?;
    Ok(output)
}

fn push_field(output: &mut Vec<u8>, value: &[u8], max_request_bytes: u64) -> Result<()> {
    let length = u64::try_from(value.len()).map_err(|_| Error::IntegerOverflow)?;
    let current = u64::try_from(output.len()).map_err(|_| Error::IntegerOverflow)?;
    let next = current
        .checked_add(8)
        .and_then(|with_length| with_length.checked_add(length))
        .ok_or(Error::IntegerOverflow)?;
    if next > max_request_bytes {
        return Err(Error::ResourceExhausted(Resource::RequestBytes));
    }
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn map_store(error: StoreError) -> Error {
    match error {
        StoreError::Exhausted(resource) => Error::ResourceExhausted(resource),
        StoreError::Unavailable | StoreError::Conflict => Error::PersistenceFailure,
    }
}
