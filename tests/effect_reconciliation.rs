// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-003/TC-009/TC-011: authorized, persistent effect reconciliation.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier, Mutex,
    },
    thread,
    time::Duration,
};

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_integration::ecosystem_lock::{
    validate, CapabilityBinding, CapabilitySelection, ComponentBinding, Limits as LockLimits,
    LockSelection, PackageBinding, Pin, PinState, RepositoryBinding, RoleBinding, WorkflowBinding,
};
use quire_integration::effect_reconciliation::{
    ActionId, ActorId, AdapterError, AdapterManifest, AttemptId, AuthorityGrant, AuthorityGrantId,
    AuthorityScope, AuthorityStatus, AuthorityValidationError, AuthorizedEffectAdapter, CallBudget,
    Decision, DecisionAuthority, DecisionId, DeliveryUnknownCause, DispatchPermit, DispatchResult,
    DuplicateGuarantee, EffectCoordinator, EffectRequest, EffectRequestId, EffectStore, Error,
    Event, ExternalEffectId, IdempotencyKey, Limits, MonotonicClock, Observation,
    ObservationAuthority, ObservationError, ObservationId, OperationId, Outcome, OwnerArtifact,
    PersistedEffectRequest, Receipt, ReceiptId, Recommendation, RecommendationId,
    ReconciliationDisposition, ReconciliationPermit, ReconciliationResult, ReconciliationSupport,
    Record, Reservation, Resource, ScopeId, State, StoreError, SubjectId, Submission,
};
use quire_integration::{
    ArtifactId, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, PackageId,
    ProviderId, RepositoryId, RoleId, ToolchainId, WorkflowInstanceId,
};
use sha2::{Digest as _, Sha256};

const ID_LIMIT: u64 = 256;

trait FixtureIdentity: Sized {
    fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self>;
}

macro_rules! fixture_identities {
    ($($identity:ty),+ $(,)?) => {
        $(
            impl FixtureIdentity for $identity {
                fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self> {
                    Self::new(value, maximum)
                }
            }
        )+
    };
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
    T::fixture_new(value, ID_LIMIT).expect("bounded fixture identity")
}

macro_rules! effect_id {
    ($identity:ty, $value:expr) => {
        <$identity>::new($value, ID_LIMIT).expect("bounded effect fixture identity")
    };
}

fn revision(seed: char) -> String {
    format!("git:{}", seed.to_string().repeat(40))
}

fn exact<T: quire_integration::ecosystem_lock::PinIdentity>(identity: T, seed: char) -> Pin<T> {
    Pin::new(identity, &revision(seed), PinState::Exact, ID_LIMIT).expect("exact fixture pin")
}

fn primary_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("effect-primary"),
        id::<ProviderId>("primary-provider"),
        id::<BuildId>("primary-build"),
        id::<ContractId>("quire.effect-adapter/v1"),
        id::<ArtifactId>("primary-options"),
    )
}

fn alternate_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("effect-alternate"),
        id::<ProviderId>("alternate-provider"),
        id::<BuildId>("alternate-build"),
        id::<ContractId>("quire.effect-adapter/v1"),
        id::<ArtifactId>("alternate-options"),
    )
}

fn observation_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("observation-authority"),
        id::<ProviderId>("qobs-provider"),
        id::<BuildId>("qobs-build"),
        id::<ContractId>("quire.observation-authority/v1"),
        id::<ArtifactId>("qobs-options"),
    )
}

fn decision_authority_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("decision-authority"),
        id::<ProviderId>("decision-provider"),
        id::<BuildId>("decision-build"),
        id::<ContractId>("quire.human-decision/v1"),
        id::<ArtifactId>("decision-options"),
    )
}

fn grant_authority_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("grant-authority"),
        id::<ProviderId>("grant-provider"),
        id::<BuildId>("grant-build"),
        id::<ContractId>("quire.effect-decision/v1"),
        id::<ArtifactId>("grant-options"),
    )
}

fn adapter_manifest(
    selection: CapabilitySelection,
    guarantee: DuplicateGuarantee,
) -> AdapterManifest {
    AdapterManifest {
        selection,
        idempotency_field: "Idempotency-Key".to_owned(),
        duplicate_guarantee: guarantee,
        reconciliation: ReconciliationSupport::Operation(effect_id!(OperationId, "refund.lookup")),
    }
}

fn validated_lock() -> quire_integration::ecosystem_lock::ValidatedLock {
    validated_lock_with_deployment_revision('5')
}

fn validated_lock_with_deployment_revision(
    deployment_revision: char,
) -> quire_integration::ecosystem_lock::ValidatedLock {
    let repository = id::<RepositoryId>("agent-ix/effect-fixture");
    let package = id::<PackageId>("effect-fixture");
    let component = id::<ComponentId>("effect-fixture");
    let selection = LockSelection::new(
        vec![RepositoryBinding::new(
            exact(repository.clone(), 'a'),
            "https://github.com/agent-ix/effect-fixture",
            ID_LIMIT,
        )
        .expect("repository binding")],
        vec![PackageBinding::new(
            exact(package.clone(), 'b'),
            "1.0.0",
            repository.clone(),
            &revision('a'),
            &["default"],
            1,
            ID_LIMIT,
        )
        .expect("package binding")],
        vec![exact(id::<DefinitionId>("timed-refund"), 'c')],
        vec![
            exact(id::<ProviderId>("primary-provider"), 'd'),
            exact(id::<ProviderId>("alternate-provider"), 'e'),
            exact(id::<ProviderId>("qobs-provider"), 'f'),
            exact(id::<ProviderId>("decision-provider"), '0'),
            exact(id::<ProviderId>("grant-provider"), 'b'),
        ],
        vec![exact(id::<ToolchainId>("rust-1.98.1"), '1')],
        vec![
            exact(id::<BuildId>("primary-build"), '2'),
            exact(id::<BuildId>("alternate-build"), '3'),
            exact(id::<BuildId>("qobs-build"), '4'),
            exact(id::<BuildId>("decision-build"), '0'),
            exact(id::<BuildId>("grant-build"), 'b'),
        ],
        vec![exact(id::<DeploymentId>("local-test"), deployment_revision)],
        vec![ComponentBinding::new(
            exact(component.clone(), '6'),
            repository,
            package,
        )],
        vec![
            RoleBinding::new(
                exact(id::<RoleId>("effect-primary"), '7'),
                component.clone(),
            ),
            RoleBinding::new(
                exact(id::<RoleId>("effect-alternate"), '8'),
                component.clone(),
            ),
            RoleBinding::new(
                exact(id::<RoleId>("observation-authority"), '9'),
                component.clone(),
            ),
            RoleBinding::new(exact(id::<RoleId>("decision-authority"), '0'), component),
            RoleBinding::new(
                exact(id::<RoleId>("grant-authority"), 'b'),
                id::<ComponentId>("effect-fixture"),
            ),
        ],
        vec![
            CapabilityBinding::new(primary_selection()),
            CapabilityBinding::new(alternate_selection()),
            CapabilityBinding::new(observation_selection()),
            CapabilityBinding::new(decision_authority_selection()),
            CapabilityBinding::new(grant_authority_selection()),
        ],
        vec![WorkflowBinding::new(
            exact(id::<WorkflowInstanceId>("refund:fixture"), 'a'),
            id::<DefinitionId>("timed-refund"),
        )],
        vec![],
    );
    let limits = LockLimits::for_selection(&selection, 64 * 1024, ID_LIMIT).expect("lock limits");
    validate(&selection, limits).expect("validated exact lock")
}

fn limits() -> Limits {
    Limits {
        max_identity_bytes: ID_LIMIT,
        max_argument_bytes: 1024,
        max_request_bytes: 4096,
        max_history_events: 32,
        max_attempts: 4,
        max_receipt_bytes: 1024,
        max_observation_bytes: 1024,
        max_owner_artifact_bytes: 1024,
        max_execution_duration_ms: 10_000,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn submission(selection: CapabilitySelection, guarantee: DuplicateGuarantee) -> Submission {
    let arguments = br#"{"amount":42,"currency":"USD"}"#.to_vec();
    let recommendation = Recommendation {
        id: effect_id!(RecommendationId, "recommendation:refund"),
        digest: "sha256:recommendation-refund".to_owned(),
        action: effect_id!(ActionId, "refund"),
        subject: effect_id!(SubjectId, "order:42"),
        scope: effect_id!(ScopeId, "workflow:refund"),
    };
    let grant_bytes = b"quire.effect-decision/v1:grant:refund:exact-rights".to_vec();
    let authority = AuthorityGrant {
        id: effect_id!(AuthorityGrantId, digest(&grant_bytes)),
        artifact: OwnerArtifact {
            contract_version: "quire.effect-decision/v1".to_owned(),
            canonical_bytes: grant_bytes,
            attestation: b"fixture-grant-attestation".to_vec(),
        },
        issuer: grant_authority_selection(),
        actor: effect_id!(ActorId, "operator:alice"),
        action: recommendation.action.clone(),
        subject: recommendation.subject.clone(),
        scope: AuthorityScope::Exact(recommendation.scope.clone()),
        not_before_ms: 10,
        expires_at_ms: 100,
    };
    let operation = effect_id!(OperationId, "refund.create");
    let decision_bytes =
        b"quire.human-decision/v1:accepted:refund:evidence:rationale:no-prior".to_vec();
    let decision = Decision {
        id: effect_id!(DecisionId, digest(&decision_bytes)),
        artifact: OwnerArtifact {
            contract_version: "quire.human-decision/v1".to_owned(),
            canonical_bytes: decision_bytes,
            attestation: b"fixture-decision-attestation".to_vec(),
        },
        recommendation_id: recommendation.id.clone(),
        recommendation_digest: recommendation.digest.clone(),
        authority_grant_id: authority.id.clone(),
        authority: decision_authority_selection(),
        actor: authority.actor.clone(),
        action: recommendation.action.clone(),
        subject: recommendation.subject.clone(),
        scope: recommendation.scope.clone(),
        adapter: selection.clone(),
        operation: operation.clone(),
        argument_digest: digest(&arguments),
        supersedes: None,
    };
    Submission {
        recommendation,
        decision,
        authority: Some(authority),
        manifest: adapter_manifest(selection, guarantee),
        key: effect_id!(IdempotencyKey, "caller-key:refund:42"),
        operation,
        canonical_arguments: arguments,
        supersedes_request: None,
    }
}

fn persisted_fields(request: &EffectRequest) -> PersistedEffectRequest {
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

type EventLog = Rc<RefCell<Vec<&'static str>>>;

#[derive(Default)]
struct MemoryStore {
    records: BTreeMap<String, Record>,
    events: EventLog,
    fail_reserve_once: bool,
    fail_prepare_once: bool,
    fail_receipt_once: bool,
    fail_delivery_unknown_once: bool,
    substitute_observation_once: bool,
    fail_load_resource_once: Option<Resource>,
    duration_budgets: Vec<u64>,
}

impl MemoryStore {
    fn with_events(events: EventLog) -> Self {
        Self {
            events,
            ..Self::default()
        }
    }

    fn writes(&self) -> usize {
        self.records
            .values()
            .map(|record| record.history.len())
            .sum()
    }

    fn only_record(&self) -> &Record {
        assert_eq!(self.records.len(), 1);
        self.records.values().next().expect("one durable record")
    }

    fn append(
        record: &mut Record,
        expected_revision: u64,
        event: Event,
        state: State,
        limits: Limits,
    ) -> std::result::Result<(), StoreError> {
        if record.revision != expected_revision {
            return Err(StoreError::Conflict);
        }
        let next_len = record
            .history
            .len()
            .checked_add(1)
            .ok_or(StoreError::Exhausted(Resource::HistoryEvents))?;
        if u64::try_from(next_len).map_err(|_| StoreError::Exhausted(Resource::HistoryEvents))?
            > limits.max_history_events
        {
            return Err(StoreError::Exhausted(Resource::HistoryEvents));
        }
        record.history.push(event);
        record.revision = record
            .revision
            .checked_add(1)
            .ok_or(StoreError::Exhausted(Resource::HistoryEvents))?;
        record.state = state;
        Ok(())
    }

    fn attempt_matches(record: &Record, attempt: &AttemptId) -> bool {
        matches!(
            &record.state,
            State::Open(current) | State::DeliveryUnknown(current) if current == attempt
        )
    }

    fn record_budget(&mut self, limits: Limits) {
        self.duration_budgets.push(limits.max_execution_duration_ms);
    }
}

impl EffectStore for MemoryStore {
    fn load(
        &mut self,
        key: &IdempotencyKey,
        limits: Limits,
    ) -> std::result::Result<Option<Record>, StoreError> {
        self.record_budget(limits);
        if let Some(resource) = self.fail_load_resource_once.take() {
            return Err(StoreError::Exhausted(resource));
        }
        Ok(self.records.get(key.as_str()).cloned())
    }

    fn reserve_or_load(
        &mut self,
        request: &EffectRequest,
        limits: Limits,
    ) -> std::result::Result<Reservation, StoreError> {
        self.record_budget(limits);
        if self.fail_reserve_once {
            self.fail_reserve_once = false;
            return Err(StoreError::Unavailable);
        }
        if let Some(existing) = self.records.get(request.key().as_str()) {
            return if existing.request.canonical_bytes() == request.canonical_bytes() {
                Ok(Reservation::Existing(existing.clone()))
            } else {
                Ok(Reservation::Conflict)
            };
        }
        if limits.max_history_events < 1 {
            return Err(StoreError::Exhausted(Resource::HistoryEvents));
        }
        let record = Record {
            request: request.clone(),
            revision: 1,
            state: State::Pending,
            history: vec![Event::RequestPersisted],
        };
        self.records
            .insert(request.key().as_str().to_owned(), record.clone());
        self.events.borrow_mut().push("store:request");
        Ok(Reservation::Inserted(record))
    }

    fn prepare_attempt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        if self.fail_prepare_once {
            self.fail_prepare_once = false;
            return Err(StoreError::Unavailable);
        }
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        if matches!(
            record.state,
            State::Delivered(_) | State::EffectObserved { .. }
        ) {
            return Err(StoreError::Conflict);
        }
        Self::append(
            record,
            expected_revision,
            Event::AttemptPrepared(attempt.clone()),
            State::Open(attempt.clone()),
            limits,
        )?;
        let result = record.clone();
        self.events.borrow_mut().push("store:attempt");
        Ok(result)
    }

    fn record_reconciliation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        disposition: ReconciliationDisposition,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        if !Self::attempt_matches(record, attempt) {
            return Err(StoreError::Conflict);
        }
        Self::append(
            record,
            expected_revision,
            Event::ReconciliationRecorded {
                attempt: attempt.clone(),
                disposition,
            },
            match disposition {
                ReconciliationDisposition::NotDelivered => State::NotDelivered(attempt.clone()),
                ReconciliationDisposition::Inconclusive
                | ReconciliationDisposition::Unavailable => State::DeliveryUnknown(attempt.clone()),
            },
            limits,
        )?;
        let result = record.clone();
        self.events.borrow_mut().push("store:reconciliation");
        Ok(result)
    }

    fn record_dispatch_not_accepted(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        if !Self::attempt_matches(record, attempt) {
            return Err(StoreError::Conflict);
        }
        Self::append(
            record,
            expected_revision,
            Event::DispatchNotAccepted(attempt.clone()),
            State::NotDelivered(attempt.clone()),
            limits,
        )?;
        Ok(record.clone())
    }

    fn record_receipt(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        receipt: &Receipt,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        if self.fail_receipt_once {
            self.fail_receipt_once = false;
            self.events.borrow_mut().push("store:receipt-failed");
            return Err(StoreError::Unavailable);
        }
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        if !Self::attempt_matches(record, attempt) {
            return Err(StoreError::Conflict);
        }
        Self::append(
            record,
            expected_revision,
            Event::ReceiptPersisted(Box::new(receipt.clone())),
            State::Delivered(receipt.clone()),
            limits,
        )?;
        let result = record.clone();
        self.events.borrow_mut().push("store:receipt");
        Ok(result)
    }

    fn record_delivery_unknown(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        attempt: &AttemptId,
        cause: DeliveryUnknownCause,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        if self.fail_delivery_unknown_once {
            self.fail_delivery_unknown_once = false;
            return Err(StoreError::Unavailable);
        }
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        if !Self::attempt_matches(record, attempt) {
            return Err(StoreError::Conflict);
        }
        Self::append(
            record,
            expected_revision,
            Event::DeliveryUnknownRecorded {
                attempt: attempt.clone(),
                cause,
            },
            State::DeliveryUnknown(attempt.clone()),
            limits,
        )?;
        let result = record.clone();
        self.events.borrow_mut().push("store:delivery-unknown");
        Ok(result)
    }

    fn link_observation(
        &mut self,
        key: &IdempotencyKey,
        expected_revision: u64,
        observation: &Observation,
        limits: Limits,
    ) -> std::result::Result<Record, StoreError> {
        self.record_budget(limits);
        let mut substituted;
        let observation = if self.substitute_observation_once {
            self.substitute_observation_once = false;
            substituted = observation.clone();
            if let Some(first) = substituted.payload.first_mut() {
                *first ^= 0xff;
            }
            &substituted
        } else {
            observation
        };
        let record = self
            .records
            .get_mut(key.as_str())
            .ok_or(StoreError::Conflict)?;
        let receipt = match &record.state {
            State::Delivered(receipt) => Some(receipt.clone()),
            State::Pending
            | State::Open(_)
            | State::NotDelivered(_)
            | State::DeliveryUnknown(_) => None,
            State::EffectObserved { .. } => return Err(StoreError::Conflict),
        };
        Self::append(
            record,
            expected_revision,
            Event::ObservationLinked(Box::new(observation.clone())),
            State::EffectObserved {
                receipt,
                observation: Box::new(observation.clone()),
            },
            limits,
        )?;
        let result = record.clone();
        self.events.borrow_mut().push("store:observation");
        Ok(result)
    }
}

#[derive(Clone)]
struct AtomicReservationStore {
    record: Arc<Mutex<Option<Record>>>,
    barrier: Arc<Barrier>,
}

impl AtomicReservationStore {
    fn pair() -> (Self, Self) {
        let record = Arc::new(Mutex::new(None));
        let barrier = Arc::new(Barrier::new(2));
        (
            Self {
                record: Arc::clone(&record),
                barrier: Arc::clone(&barrier),
            },
            Self { record, barrier },
        )
    }
}

impl EffectStore for AtomicReservationStore {
    fn load(
        &mut self,
        _: &IdempotencyKey,
        _: Limits,
    ) -> std::result::Result<Option<Record>, StoreError> {
        self.record
            .lock()
            .map(|record| record.clone())
            .map_err(|_| StoreError::Unavailable)
    }

    fn reserve_or_load(
        &mut self,
        request: &EffectRequest,
        limits: Limits,
    ) -> std::result::Result<Reservation, StoreError> {
        self.barrier.wait();
        let mut slot = self.record.lock().map_err(|_| StoreError::Unavailable)?;
        if let Some(existing) = slot.as_ref() {
            return if existing.request.canonical_bytes() == request.canonical_bytes() {
                Ok(Reservation::Existing(existing.clone()))
            } else {
                Ok(Reservation::Conflict)
            };
        }
        if limits.max_history_events < 1 {
            return Err(StoreError::Exhausted(Resource::HistoryEvents));
        }
        let record = Record {
            request: request.clone(),
            revision: 1,
            state: State::Pending,
            history: vec![Event::RequestPersisted],
        };
        *slot = Some(record.clone());
        Ok(Reservation::Inserted(record))
    }

    fn prepare_attempt(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &AttemptId,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }

    fn record_reconciliation(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &AttemptId,
        _: ReconciliationDisposition,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }

    fn record_dispatch_not_accepted(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &AttemptId,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }

    fn record_receipt(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &AttemptId,
        _: &Receipt,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }

    fn record_delivery_unknown(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &AttemptId,
        _: DeliveryUnknownCause,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }

    fn link_observation(
        &mut self,
        _: &IdempotencyKey,
        _: u64,
        _: &Observation,
        _: Limits,
    ) -> std::result::Result<Record, StoreError> {
        Err(StoreError::Unavailable)
    }
}

#[derive(Clone, Copy)]
enum DispatchPlan {
    Accepted,
    Indeterminate,
    NotAccepted,
    Error,
    InvalidReceipt,
    OversizedReceipt,
}

#[derive(Clone, Copy)]
enum ReconcilePlan {
    Accepted,
    NotDelivered,
    Inconclusive,
    Error,
    ContractViolation,
    InvalidReceipt,
    OversizedReceipt,
}

struct TestAdapter {
    manifest: AdapterManifest,
    events: EventLog,
    dispatch_plan: DispatchPlan,
    reconcile_plan: ReconcilePlan,
    dispatch_calls: u64,
    reconcile_calls: u64,
    permit_checks: u64,
    receipt_payload_bytes: Option<usize>,
    last_dispatch_duration_ms: Option<u64>,
    dispatch_clock_advance: Option<(Arc<ManualMonotonicClock>, u64)>,
    reconciliation_clock_advance: Option<(Arc<ManualMonotonicClock>, u64)>,
}

impl TestAdapter {
    fn accepted(selection: CapabilitySelection, events: EventLog) -> Self {
        Self {
            manifest: adapter_manifest(selection, DuplicateGuarantee::SameKeyAndBytesSameEffect),
            events,
            dispatch_plan: DispatchPlan::Accepted,
            reconcile_plan: ReconcilePlan::Accepted,
            dispatch_calls: 0,
            reconcile_calls: 0,
            permit_checks: 0,
            receipt_payload_bytes: None,
            last_dispatch_duration_ms: None,
            dispatch_clock_advance: None,
            reconciliation_clock_advance: None,
        }
    }

    fn uncertain(
        selection: CapabilitySelection,
        events: EventLog,
        reconcile_plan: ReconcilePlan,
        guarantee: DuplicateGuarantee,
    ) -> Self {
        Self {
            manifest: adapter_manifest(selection, guarantee),
            events,
            dispatch_plan: DispatchPlan::Indeterminate,
            reconcile_plan,
            dispatch_calls: 0,
            reconcile_calls: 0,
            permit_checks: 0,
            receipt_payload_bytes: None,
            last_dispatch_duration_ms: None,
            dispatch_clock_advance: None,
            reconciliation_clock_advance: None,
        }
    }

    fn receipt(&self, request: &EffectRequest) -> Receipt {
        Receipt {
            id: effect_id!(ReceiptId, "receipt:refund:42"),
            request_id: request.id().clone(),
            selection: self.manifest.selection.clone(),
            external_effect_id: effect_id!(ExternalEffectId, "provider-refund:42"),
            payload: self
                .receipt_payload_bytes
                .map_or_else(|| b"accepted".to_vec(), |length| vec![0; length]),
        }
    }

    fn check_dispatch_permit(&mut self, permit: &DispatchPermit, request: &EffectRequest) {
        assert_eq!(permit.request_id(), request.id());
        assert!(permit.attempt_id().as_str().contains(":attempt:"));
        assert_eq!(permit.max_receipt_bytes(), 1024);
        assert!(permit.remaining_duration_ms() > 0);
        self.last_dispatch_duration_ms = Some(permit.remaining_duration_ms());
        self.permit_checks += 1;
    }

    fn check_reconciliation_permit(
        &mut self,
        permit: &ReconciliationPermit,
        request: &EffectRequest,
    ) {
        assert_eq!(permit.request_id(), request.id());
        assert!(permit.attempt_id().as_str().contains(":attempt:"));
        assert_eq!(permit.max_receipt_bytes(), 1024);
        assert!(permit.remaining_duration_ms() > 0);
        self.permit_checks += 1;
    }
}

impl AuthorizedEffectAdapter for TestAdapter {
    fn manifest(&self) -> &AdapterManifest {
        &self.manifest
    }

    fn dispatch(
        &mut self,
        permit: &DispatchPermit,
        request: &EffectRequest,
    ) -> std::result::Result<DispatchResult, AdapterError> {
        self.check_dispatch_permit(permit, request);
        assert_eq!(request.operation().as_str(), "refund.create");
        assert_eq!(
            request.canonical_arguments(),
            br#"{"amount":42,"currency":"USD"}"#
        );
        self.dispatch_calls += 1;
        self.events.borrow_mut().push("adapter:dispatch");
        if let Some((clock, duration_ms)) = &self.dispatch_clock_advance {
            clock.advance(*duration_ms);
        }
        Ok(match self.dispatch_plan {
            DispatchPlan::Accepted => DispatchResult::Accepted(Box::new(self.receipt(request))),
            DispatchPlan::Indeterminate => DispatchResult::Indeterminate,
            DispatchPlan::NotAccepted => DispatchResult::NotAccepted,
            DispatchPlan::Error => return Err(AdapterError::Unavailable),
            DispatchPlan::InvalidReceipt => {
                let mut receipt = self.receipt(request);
                receipt.request_id = effect_id!(EffectRequestId, "effect-request:foreign");
                DispatchResult::Accepted(Box::new(receipt))
            }
            DispatchPlan::OversizedReceipt => {
                let mut receipt = self.receipt(request);
                receipt.payload = vec![0; 1025];
                DispatchResult::Accepted(Box::new(receipt))
            }
        })
    }

    fn reconcile(
        &mut self,
        permit: &ReconciliationPermit,
        request: &EffectRequest,
    ) -> std::result::Result<ReconciliationResult, AdapterError> {
        self.check_reconciliation_permit(permit, request);
        self.reconcile_calls += 1;
        self.events.borrow_mut().push("adapter:reconcile");
        if let Some((clock, duration_ms)) = &self.reconciliation_clock_advance {
            clock.advance(*duration_ms);
        }
        Ok(match self.reconcile_plan {
            ReconcilePlan::Accepted => {
                ReconciliationResult::Accepted(Box::new(self.receipt(request)))
            }
            ReconcilePlan::NotDelivered => ReconciliationResult::NotDelivered,
            ReconcilePlan::Inconclusive => ReconciliationResult::Inconclusive,
            ReconcilePlan::Error => return Err(AdapterError::Unavailable),
            ReconcilePlan::ContractViolation => return Err(AdapterError::ContractViolation),
            ReconcilePlan::InvalidReceipt => {
                let mut receipt = self.receipt(request);
                receipt.request_id = effect_id!(EffectRequestId, "effect-request:foreign");
                ReconciliationResult::Accepted(Box::new(receipt))
            }
            ReconcilePlan::OversizedReceipt => {
                let mut receipt = self.receipt(request);
                receipt.payload = vec![0; 1025];
                ReconciliationResult::Accepted(Box::new(receipt))
            }
        })
    }
}

struct TestObservationAuthority {
    selection: CapabilitySelection,
    qualified: bool,
    calls: Cell<u64>,
}

impl ObservationAuthority for TestObservationAuthority {
    fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    fn verify(
        &self,
        _: &Observation,
        budget: CallBudget,
    ) -> std::result::Result<(), ObservationError> {
        assert!(budget.remaining_duration_ms() > 0);
        self.calls.set(self.calls.get() + 1);
        if self.qualified {
            Ok(())
        } else {
            Err(ObservationError::Unqualified)
        }
    }
}

impl TestObservationAuthority {
    fn valid() -> Self {
        Self {
            selection: observation_selection(),
            qualified: true,
            calls: Cell::new(0),
        }
    }
}

struct TestDecisionAuthority {
    decision_selection: CapabilitySelection,
    grant_selection: CapabilitySelection,
    valid: bool,
    now_ms: u64,
}

impl TestDecisionAuthority {
    fn valid() -> Self {
        Self::at(50)
    }

    fn at(now_ms: u64) -> Self {
        Self {
            decision_selection: decision_authority_selection(),
            grant_selection: grant_authority_selection(),
            valid: true,
            now_ms,
        }
    }
}

impl DecisionAuthority for TestDecisionAuthority {
    fn decision_selection(&self) -> &CapabilitySelection {
        &self.decision_selection
    }

    fn grant_selection(&self) -> &CapabilitySelection {
        &self.grant_selection
    }

    fn verify(
        &self,
        _: &Recommendation,
        _: &Decision,
        grant: &AuthorityGrant,
        budget: CallBudget,
    ) -> std::result::Result<AuthorityStatus, AuthorityValidationError> {
        assert!(budget.remaining_duration_ms() > 0);
        if !self.valid {
            return Err(AuthorityValidationError::Unauthenticated);
        }
        Ok(
            if self.now_ms >= grant.not_before_ms && self.now_ms <= grant.expires_at_ms {
                AuthorityStatus::Current
            } else {
                AuthorityStatus::OutsideValidity
            },
        )
    }
}

#[derive(Default)]
struct ManualMonotonicClock {
    now_ms: AtomicU64,
}

impl ManualMonotonicClock {
    fn advance(&self, duration_ms: u64) {
        self.now_ms.fetch_add(duration_ms, Ordering::SeqCst);
    }
}

impl MonotonicClock for ManualMonotonicClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.now_ms.load(Ordering::SeqCst))
    }
}

struct AdvancingDecisionAuthority {
    decision_selection: CapabilitySelection,
    grant_selection: CapabilitySelection,
    calls: Cell<u64>,
    advance_on_call: u64,
    advance_ms: u64,
    clock: Arc<ManualMonotonicClock>,
}

impl DecisionAuthority for AdvancingDecisionAuthority {
    fn decision_selection(&self) -> &CapabilitySelection {
        &self.decision_selection
    }

    fn grant_selection(&self) -> &CapabilitySelection {
        &self.grant_selection
    }

    fn verify(
        &self,
        _: &Recommendation,
        _: &Decision,
        _: &AuthorityGrant,
        budget: CallBudget,
    ) -> std::result::Result<AuthorityStatus, AuthorityValidationError> {
        assert!(budget.remaining_duration_ms() > 0);
        let call = self.calls.get() + 1;
        self.calls.set(call);
        if call == self.advance_on_call {
            self.clock.advance(self.advance_ms);
        }
        Ok(AuthorityStatus::Current)
    }
}

struct ExpiringDecisionAuthority {
    decision_selection: CapabilitySelection,
    grant_selection: CapabilitySelection,
    calls: Cell<u64>,
    expire_on_call: u64,
}

impl DecisionAuthority for ExpiringDecisionAuthority {
    fn decision_selection(&self) -> &CapabilitySelection {
        &self.decision_selection
    }

    fn grant_selection(&self) -> &CapabilitySelection {
        &self.grant_selection
    }

    fn verify(
        &self,
        _: &Recommendation,
        _: &Decision,
        _: &AuthorityGrant,
        budget: CallBudget,
    ) -> std::result::Result<AuthorityStatus, AuthorityValidationError> {
        assert!(budget.remaining_duration_ms() > 0);
        let call = self.calls.get().saturating_add(1);
        self.calls.set(call);
        Ok(if call >= self.expire_on_call {
            AuthorityStatus::OutsideValidity
        } else {
            AuthorityStatus::Current
        })
    }
}

#[test]
#[trace("TC-003", "FR-003-AC-1")]
fn tc003_fresh_authorized_request_persists_before_one_dispatch_and_retains_receipt() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events.clone());

    let delivered = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("fresh authorized request");
    let receipt = match delivered {
        Outcome::Delivered(receipt) => receipt,
        other => panic!("expected durable delivery, got {other:?}"),
    };
    assert_eq!(receipt.selection.role().as_str(), "effect-primary");
    assert_eq!(receipt.selection.provider().as_str(), "primary-provider");
    assert_eq!(receipt.selection.build().as_str(), "primary-build");
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 0);
    assert_eq!(adapter.permit_checks, 1);
    assert_eq!(
        events.borrow().as_slice(),
        [
            "store:request",
            "store:attempt",
            "adapter:dispatch",
            "store:receipt",
        ]
    );
    assert!(matches!(store.only_record().state, State::Delivered(_)));

    let exact_replay = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("exact receipt replay");
    assert_eq!(exact_replay, Outcome::Delivered(receipt.clone()));
    let expired_replay = request.clone();
    let expired_replay = coordinator
        .coordinate(
            &lock,
            &expired_replay,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::at(101),
            &TestObservationAuthority::valid(),
        )
        .expect("durable receipt replay remains available after expiry");
    assert_eq!(expired_replay, Outcome::Delivered(receipt));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 0);

    let reserve_events = EventLog::default();
    let mut reserve_failure = MemoryStore::with_events(reserve_events.clone());
    reserve_failure.fail_reserve_once = true;
    let mut reserve_adapter = TestAdapter::accepted(primary_selection(), reserve_events);
    let error = coordinator
        .coordinate(
            &lock,
            &request,
            &mut reserve_failure,
            &mut reserve_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect_err("request persistence failure must refuse");
    assert_eq!(error, Error::PersistenceFailure);
    assert_eq!(reserve_failure.writes(), 0);
    assert_eq!(reserve_adapter.dispatch_calls, 0);

    let prepare_events = EventLog::default();
    let mut prepare_failure = MemoryStore::with_events(prepare_events.clone());
    prepare_failure.fail_prepare_once = true;
    let mut prepare_adapter = TestAdapter::accepted(primary_selection(), prepare_events);
    let error = coordinator
        .coordinate(
            &lock,
            &request,
            &mut prepare_failure,
            &mut prepare_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect_err("attempt persistence failure must refuse");
    assert_eq!(error, Error::PersistenceFailure);
    assert_eq!(prepare_failure.writes(), 1);
    assert_eq!(prepare_adapter.dispatch_calls, 0);
    let retry = coordinator
        .coordinate(
            &lock,
            &request,
            &mut prepare_failure,
            &mut prepare_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("durable pending request resumes after prepare failure");
    assert!(matches!(retry, Outcome::Delivered(_)));
    assert_eq!(prepare_adapter.dispatch_calls, 1);

    let manifest_events = EventLog::default();
    let mut manifest_store = MemoryStore::with_events(manifest_events.clone());
    let mut manifest_adapter = TestAdapter::accepted(primary_selection(), manifest_events);
    let mut forged_manifest = request;
    forged_manifest.manifest.duplicate_guarantee = DuplicateGuarantee::None;
    let error = coordinator
        .coordinate(
            &lock,
            &forged_manifest,
            &mut manifest_store,
            &mut manifest_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect_err("caller cannot strengthen or replace the adapter-owned manifest");
    assert_eq!(error, Error::AdapterMismatch);
    assert_eq!(manifest_store.writes(), 0);
    assert_eq!(manifest_adapter.dispatch_calls, 0);
}

#[test]
#[trace("TC-003", "FR-003-AC-3")]
fn tc003_authority_variants_refuse_before_store_or_adapter() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let baseline = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );

    let mut missing = baseline.clone();
    missing.authority = None;
    let mut expired = baseline.clone();
    expired.authority.as_mut().expect("grant").expires_at_ms = 49;
    let mut not_yet_valid = baseline.clone();
    not_yet_valid
        .authority
        .as_mut()
        .expect("grant")
        .not_before_ms = 51;
    let mut mismatched_actor = baseline.clone();
    mismatched_actor.authority.as_mut().expect("grant").actor =
        effect_id!(ActorId, "operator:mallory");
    let mut mismatched_scope = baseline.clone();
    mismatched_scope.authority.as_mut().expect("grant").scope =
        AuthorityScope::Exact(effect_id!(ScopeId, "workflow:other"));
    let mut mismatched_action = baseline.clone();
    mismatched_action.authority.as_mut().expect("grant").action = effect_id!(ActionId, "capture");
    let mut mismatched_subject = baseline.clone();
    mismatched_subject
        .authority
        .as_mut()
        .expect("grant")
        .subject = effect_id!(SubjectId, "order:other");
    let mut recommendation_mismatch = baseline.clone();
    recommendation_mismatch.recommendation.digest = "sha256:foreign".to_owned();
    let mut over_broad = baseline.clone();
    over_broad.authority.as_mut().expect("grant").scope = AuthorityScope::All;
    let mut decision_version_skew = baseline.clone();
    decision_version_skew.decision.artifact.contract_version = "quire.human-decision/v2".to_owned();
    let mut grant_version_skew = baseline.clone();
    grant_version_skew
        .authority
        .as_mut()
        .expect("grant")
        .artifact
        .contract_version = "quire.effect-decision/v2".to_owned();

    for (candidate, expected) in [
        (missing, Error::AuthorityMissing),
        (expired, Error::AuthorityExpired),
        (not_yet_valid, Error::AuthorityExpired),
        (mismatched_actor, Error::AuthorityMismatch),
        (mismatched_scope, Error::AuthorityMismatch),
        (mismatched_action, Error::AuthorityMismatch),
        (mismatched_subject, Error::AuthorityMismatch),
        (recommendation_mismatch, Error::DecisionMismatch),
        (over_broad, Error::AuthorityOverBroad),
        (decision_version_skew, Error::DecisionMismatch),
        (grant_version_skew, Error::DecisionMismatch),
    ] {
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        let mut adapter = TestAdapter::accepted(primary_selection(), events);
        let error = coordinator
            .coordinate(
                &lock,
                &candidate,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("invalid authority must refuse");
        assert_eq!(error, expected);
        assert_eq!(store.writes(), 0);
        assert_eq!(adapter.dispatch_calls, 0);
        assert_eq!(adapter.reconcile_calls, 0);
    }

    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    let unauthenticated = TestDecisionAuthority {
        decision_selection: decision_authority_selection(),
        grant_selection: grant_authority_selection(),
        valid: false,
        now_ms: 50,
    };
    let error = coordinator
        .coordinate(
            &lock,
            &baseline,
            &mut store,
            &mut adapter,
            &unauthenticated,
            &TestObservationAuthority::valid(),
        )
        .expect_err("self-consistent forged authority must still refuse");
    assert_eq!(error, Error::AuthorityMismatch);
    assert_eq!(store.writes(), 0);
    assert_eq!(adapter.dispatch_calls, 0);
}

#[test]
#[trace("TC-003", "FR-003-AC-4")]
fn tc003_conflicting_key_reuse_refuses_for_every_request_identity_component() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let mut runner = proptest::test_runner::TestRunner::new(ProptestConfig::with_cases(64));
    runner
        .run(&(0_u8..5, "[a-z]{1,12}"), |(mutation, suffix)| {
            let events = EventLog::default();
            let mut store = MemoryStore::with_events(events.clone());
            let baseline = submission(
                primary_selection(),
                DuplicateGuarantee::SameKeyAndBytesSameEffect,
            );
            let mut primary_adapter = TestAdapter::accepted(primary_selection(), events.clone());
            coordinator
                .coordinate(
                    &lock,
                    &baseline,
                    &mut store,
                    &mut primary_adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect("baseline request");
            let writes = store.writes();

            let mut conflicting = baseline;
            let selection = match mutation {
                0 => {
                    conflicting
                        .decision
                        .artifact
                        .canonical_bytes
                        .extend_from_slice(suffix.as_bytes());
                    conflicting.decision.id = effect_id!(
                        DecisionId,
                        digest(&conflicting.decision.artifact.canonical_bytes)
                    );
                    primary_selection()
                }
                1 => {
                    let grant_artifact =
                        &mut conflicting.authority.as_mut().expect("grant").artifact;
                    grant_artifact
                        .canonical_bytes
                        .extend_from_slice(suffix.as_bytes());
                    let grant =
                        effect_id!(AuthorityGrantId, digest(&grant_artifact.canonical_bytes));
                    conflicting.authority.as_mut().expect("grant").id = grant.clone();
                    conflicting.decision.authority_grant_id = grant;
                    primary_selection()
                }
                2 => {
                    let alternate = alternate_selection();
                    conflicting.manifest.selection = alternate.clone();
                    conflicting.decision.adapter = alternate.clone();
                    alternate
                }
                3 => {
                    let operation =
                        OperationId::new(format!("refund.mutated-{suffix}"), ID_LIMIT).unwrap();
                    conflicting.operation = operation.clone();
                    conflicting.decision.operation = operation;
                    primary_selection()
                }
                _ => {
                    conflicting.canonical_arguments = suffix.into_bytes();
                    conflicting.decision.argument_digest = digest(&conflicting.canonical_arguments);
                    primary_selection()
                }
            };
            let mut conflicting_adapter = TestAdapter::accepted(selection, events);
            let error = coordinator
                .coordinate(
                    &lock,
                    &conflicting,
                    &mut store,
                    &mut conflicting_adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect_err("global key must reject unequal canonical request bytes");
            prop_assert_eq!(error, Error::KeyConflict);
            prop_assert_eq!(store.writes(), writes);
            prop_assert_eq!(primary_adapter.dispatch_calls, 1);
            prop_assert_eq!(conflicting_adapter.dispatch_calls, 0);
            prop_assert_eq!(conflicting_adapter.reconcile_calls, 0);
            Ok(())
        })
        .expect("generated request-identity conflicts");
}

#[test]
#[trace("TC-003", "FR-003-AC-4")]
fn tc003_atomic_first_use_race_has_one_global_key_winner() {
    fn capture_request(submission: &Submission) -> EffectRequest {
        let lock = validated_lock();
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        store.fail_prepare_once = true;
        let mut adapter = TestAdapter::accepted(primary_selection(), events);
        assert_eq!(
            EffectCoordinator::new(limits())
                .coordinate(
                    &lock,
                    submission,
                    &mut store,
                    &mut adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect_err("capture after durable reservation"),
            Error::PersistenceFailure
        );
        store.only_record().request.clone()
    }

    let first = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let mut second = first.clone();
    second
        .decision
        .artifact
        .canonical_bytes
        .extend_from_slice(b":concurrent-competitor");
    second.decision.id = effect_id!(
        DecisionId,
        digest(&second.decision.artifact.canonical_bytes)
    );
    let first = capture_request(&first);
    let second = capture_request(&second);
    assert_eq!(first.key(), second.key());
    assert_ne!(first.canonical_bytes(), second.canonical_bytes());

    let (mut left_store, mut right_store) = AtomicReservationStore::pair();
    let left = thread::spawn(move || left_store.reserve_or_load(&first, limits()));
    let right = thread::spawn(move || right_store.reserve_or_load(&second, limits()));
    let results = [
        left.join().expect("left racer").expect("left store"),
        right.join().expect("right racer").expect("right store"),
    ];
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Reservation::Inserted(_)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Reservation::Conflict))
            .count(),
        1
    );
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_uncertain_attempt_reconciles_before_any_retry() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    store.fail_receipt_once = true;
    let mut adapter = TestAdapter::accepted(primary_selection(), events.clone());

    let first = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("post-acceptance persistence failure is explicit");
    assert!(matches!(first, Outcome::DeliveryUnknown { .. }));
    assert!(matches!(store.only_record().state, State::Open(_)));
    assert_eq!(adapter.dispatch_calls, 1);

    let second = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("restart reconciles the open attempt");
    assert!(matches!(second, Outcome::Reconciled(_)));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 1);
    assert_eq!(adapter.permit_checks, 2);
    assert_eq!(
        events.borrow().as_slice(),
        [
            "store:request",
            "store:attempt",
            "adapter:dispatch",
            "store:receipt-failed",
            "adapter:reconcile",
            "store:receipt",
        ]
    );
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_unsafe_uncertain_attempt_is_delivery_unknown_without_retry() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());

    for (plan, expected_error, expected_disposition) in [
        (
            ReconcilePlan::Inconclusive,
            None,
            ReconciliationDisposition::Inconclusive,
        ),
        (
            ReconcilePlan::NotDelivered,
            Some(Error::RetryUnsupported),
            ReconciliationDisposition::NotDelivered,
        ),
    ] {
        let request = submission(primary_selection(), DuplicateGuarantee::None);
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        let mut adapter =
            TestAdapter::uncertain(primary_selection(), events, plan, DuplicateGuarantee::None);
        let first = coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("indeterminate dispatch is retained");
        assert!(matches!(first, Outcome::DeliveryUnknown { .. }));

        let resumed = coordinator.coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        );
        match expected_error {
            Some(expected) => assert_eq!(resumed.expect_err("retry must refuse"), expected),
            None => assert!(matches!(
                resumed.expect("inconclusive reconciliation remains explicit"),
                Outcome::DeliveryUnknown { .. }
            )),
        }
        assert_eq!(adapter.dispatch_calls, 1);
        assert_eq!(adapter.reconcile_calls, 1);
        assert!(store.only_record().history.iter().any(|event| {
            matches!(
                event,
                Event::ReconciliationRecorded { disposition, .. }
                    if *disposition == expected_disposition
            )
        }));
    }
}

#[test]
#[trace("TC-003", "FR-003-AC-5")]
fn tc003_only_matching_authority_qualified_observation_establishes_effect() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    let receipt = match coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("delivered request")
    {
        Outcome::Delivered(receipt) => receipt,
        other => panic!("expected receipt, got {other:?}"),
    };
    assert!(matches!(store.only_record().state, State::Delivered(_)));

    let observation = Observation {
        id: effect_id!(ObservationId, "observation:refund:42"),
        request_id: receipt.request_id.clone(),
        external_effect_id: Some(receipt.external_effect_id.clone()),
        authority: observation_selection(),
        payload: b"qualified-provider-effect".to_vec(),
    };
    let unqualified = TestObservationAuthority {
        selection: observation_selection(),
        qualified: false,
        calls: Cell::new(0),
    };
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &observation,
                &mut store,
                &unqualified,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("unqualified observation"),
        Error::ForeignObservation
    );

    let wrong_capability = TestObservationAuthority {
        selection: alternate_selection(),
        qualified: true,
        calls: Cell::new(0),
    };
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &observation,
                &mut store,
                &wrong_capability,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("an effect adapter is not observation authority"),
        Error::AdapterMismatch
    );

    let authority = TestObservationAuthority {
        selection: observation_selection(),
        qualified: true,
        calls: Cell::new(0),
    };
    let mut foreign_request = observation.clone();
    foreign_request.request_id = effect_id!(EffectRequestId, "effect-request:foreign");
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &foreign_request,
                &mut store,
                &authority,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("foreign request"),
        Error::ForeignObservation
    );
    let mut foreign_effect = observation.clone();
    foreign_effect.external_effect_id = Some(effect_id!(ExternalEffectId, "effect:foreign"));
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &foreign_effect,
                &mut store,
                &authority,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("foreign provider effect"),
        Error::ForeignObservation
    );
    assert!(matches!(store.only_record().state, State::Delivered(_)));

    let observed = coordinator
        .observe(
            &lock,
            &request.key,
            &observation,
            &mut store,
            &authority,
            &TestDecisionAuthority::valid(),
        )
        .expect("matching QObs authority fact");
    assert_eq!(observed, Outcome::EffectObserved(observation.clone()));
    assert!(matches!(
        &store.only_record().state,
        State::EffectObserved {
            receipt: Some(stored_receipt),
            observation: stored_observation,
        } if stored_receipt == &receipt && **stored_observation == observation
    ));
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_restart_from_pretransport_open_attempt_reconciles_before_dispatch() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    store.fail_prepare_once = true;
    let mut adapter = TestAdapter::accepted(primary_selection(), events);

    assert_eq!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("injected attempt-write failure"),
        Error::PersistenceFailure
    );
    let pending = store.only_record().clone();
    let attempt = AttemptId::new(
        format!("{}:attempt:1", pending.request.id().as_str()),
        ID_LIMIT,
    )
    .expect("attempt identity");
    store
        .prepare_attempt(pending.request.key(), pending.revision, &attempt, limits())
        .expect("simulate crash after attempt persistence and before transport");

    let resumed = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("open attempt is reconciled");
    assert!(matches!(resumed, Outcome::Reconciled(_)));
    assert_eq!(adapter.dispatch_calls, 0);
    assert_eq!(adapter.reconcile_calls, 1);
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_safe_retry_requires_reconciliation_current_grant_and_same_effect_guarantee() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::uncertain(
        primary_selection(),
        events,
        ReconcilePlan::Inconclusive,
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );

    let first = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("first transport is indeterminate");
    assert!(matches!(
        first,
        Outcome::DeliveryUnknown {
            cause: DeliveryUnknownCause::IndeterminateTransport,
            ..
        }
    ));
    let original_request = store.only_record().request.clone();
    adapter.dispatch_plan = DispatchPlan::Accepted;
    let retried = coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("same-effect guarantee permits retry after reconciliation");
    let receipt = match retried {
        Outcome::Delivered(receipt) => receipt,
        other => panic!("expected retried delivery, got {other:?}"),
    };
    assert_eq!(receipt.request_id, *original_request.id());
    assert_eq!(store.only_record().request, original_request);
    assert_eq!(adapter.reconcile_calls, 1);
    assert_eq!(adapter.dispatch_calls, 2);
}

#[test]
#[trace("TC-003", "FR-003-AC-2", "FR-003-AC-3")]
fn tc003_expiry_allows_read_only_reconciliation_but_forbids_new_dispatch() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());

    let reconcile_events = EventLog::default();
    let mut reconcile_store = MemoryStore::with_events(reconcile_events.clone());
    let mut reconcile_adapter = TestAdapter::uncertain(
        primary_selection(),
        reconcile_events,
        ReconcilePlan::Accepted,
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let current = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    coordinator
        .coordinate(
            &lock,
            &current,
            &mut reconcile_store,
            &mut reconcile_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("initial indeterminate transport");
    let expired = current.clone();
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &expired,
                &mut reconcile_store,
                &mut reconcile_adapter,
                &TestDecisionAuthority::at(101),
                &TestObservationAuthority::valid(),
            )
            .expect("read-only reconciliation remains permitted"),
        Outcome::Reconciled(_)
    ));
    assert_eq!(reconcile_adapter.dispatch_calls, 1);
    assert_eq!(reconcile_adapter.reconcile_calls, 1);

    let refused_events = EventLog::default();
    let mut refused_store = MemoryStore::with_events(refused_events.clone());
    let mut refused_adapter = TestAdapter::uncertain(
        primary_selection(),
        refused_events,
        ReconcilePlan::NotDelivered,
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    coordinator
        .coordinate(
            &lock,
            &current,
            &mut refused_store,
            &mut refused_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("initial indeterminate transport");
    assert_eq!(
        coordinator
            .coordinate(
                &lock,
                &expired,
                &mut refused_store,
                &mut refused_adapter,
                &TestDecisionAuthority::at(101),
                &TestObservationAuthority::valid(),
            )
            .expect_err("expired grant cannot authorize another transport call"),
        Error::RetryUnsupported
    );
    assert_eq!(refused_adapter.dispatch_calls, 1);
    assert_eq!(refused_adapter.reconcile_calls, 1);
}

#[test]
#[trace("TC-003", "FR-003-AC-3")]
fn tc003_authority_is_rechecked_immediately_before_transport() {
    let lock = validated_lock();
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    let authority = ExpiringDecisionAuthority {
        decision_selection: decision_authority_selection(),
        grant_selection: grant_authority_selection(),
        calls: Cell::new(0),
        expire_on_call: 5,
    };
    assert_eq!(
        EffectCoordinator::new(limits())
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &authority,
                &TestObservationAuthority::valid(),
            )
            .expect_err("grant expires after attempt persistence but before transport"),
        Error::AuthorityExpired
    );
    assert_eq!(authority.calls.get(), 5);
    assert_eq!(adapter.dispatch_calls, 0);
    assert!(matches!(store.only_record().state, State::Open(_)));
}

#[test]
#[trace("TC-003", "FR-003-AC-1", "FR-003-AC-2")]
fn tc003_post_call_failures_are_typed_and_conclusive_nonacceptance_is_retryable() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );

    for (plan, cause) in [
        (DispatchPlan::Error, DeliveryUnknownCause::AdapterFailure),
        (
            DispatchPlan::InvalidReceipt,
            DeliveryUnknownCause::InvalidReceipt,
        ),
        (
            DispatchPlan::OversizedReceipt,
            DeliveryUnknownCause::InvalidReceipt,
        ),
    ] {
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        let mut adapter = TestAdapter::accepted(primary_selection(), events);
        adapter.dispatch_plan = plan;
        let outcome = coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("post-call uncertainty is a typed incomplete outcome");
        assert!(matches!(
            outcome,
            Outcome::DeliveryUnknown {
                cause: actual,
                ..
            } if actual == cause
        ));
        assert!(store.only_record().history.iter().any(|event| matches!(
            event,
            Event::DeliveryUnknownRecorded { cause: actual, .. } if *actual == cause
        )));
    }

    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    store.fail_delivery_unknown_once = true;
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    adapter.dispatch_plan = DispatchPlan::Error;
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("failure to persist uncertainty remains conservative"),
        Outcome::DeliveryUnknown {
            cause: DeliveryUnknownCause::AdapterFailure,
            ..
        }
    ));
    assert!(matches!(store.only_record().state, State::Open(_)));

    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    adapter.dispatch_plan = DispatchPlan::NotAccepted;
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("conclusive nonacceptance is a durable incomplete outcome"),
        Outcome::NotDelivered(_)
    ));
    adapter.dispatch_plan = DispatchPlan::Accepted;
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("current grant and same-effect guarantee permit later retry"),
        Outcome::Delivered(_)
    ));
    let persisted = store.only_record().request.clone();
    let request_bytes = u64::try_from(persisted.canonical_bytes().len()).expect("request length");
    let identity_bytes = u64::try_from(persisted.id().as_str().len()).expect("identity length");
    let owner_bytes = [
        &persisted.decision().artifact,
        &persisted.authority().artifact,
    ]
    .into_iter()
    .map(|artifact| {
        u64::try_from(
            artifact.contract_version.len()
                + artifact.canonical_bytes.len()
                + artifact.attestation.len()
                + 24,
        )
        .expect("owner envelope length")
    })
    .max()
    .expect("owner artifacts");
    let mut reconstruction_limits = limits();
    reconstruction_limits.max_request_bytes = request_bytes;
    reconstruction_limits.max_identity_bytes = identity_bytes;
    reconstruction_limits.max_owner_artifact_bytes = owner_bytes;
    EffectRequest::from_persisted(persisted_fields(&persisted), reconstruction_limits)
        .expect("exact persisted request, identity, and owner-envelope limits");
    for (resource, mutate) in [
        (Resource::RequestBytes, 0_u8),
        (Resource::IdentityBytes, 1_u8),
        (Resource::OwnerArtifactBytes, 2_u8),
    ] {
        let mut one_under = reconstruction_limits;
        match mutate {
            0 => one_under.max_request_bytes -= 1,
            1 => one_under.max_identity_bytes -= 1,
            _ => one_under.max_owner_artifact_bytes -= 1,
        }
        assert_eq!(
            EffectRequest::from_persisted(persisted_fields(&persisted), one_under)
                .expect_err("one-over persisted boundary"),
            Error::ResourceExhausted(resource)
        );
    }
    assert_eq!(adapter.dispatch_calls, 2);
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_deadline_crossing_after_adapter_call_remains_typed_and_reconcilable() {
    let lock = validated_lock();
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let mut bounded = limits();
    bounded.max_execution_duration_ms = 100;

    for (plan, expected_cause) in [
        (
            DispatchPlan::Accepted,
            DeliveryUnknownCause::ReceiptPersistenceFailure,
        ),
        (DispatchPlan::Error, DeliveryUnknownCause::AdapterFailure),
        (
            DispatchPlan::NotAccepted,
            DeliveryUnknownCause::PostBoundaryPersistenceFailure,
        ),
    ] {
        let clock = Arc::new(ManualMonotonicClock::default());
        let coordinator = EffectCoordinator::with_monotonic_clock(bounded, clock.clone());
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        let mut adapter = TestAdapter::accepted(primary_selection(), events);
        adapter.dispatch_plan = plan;
        adapter.dispatch_clock_advance = Some((clock, 101));

        assert!(matches!(
            coordinator
                .coordinate(
                    &lock,
                    &request,
                    &mut store,
                    &mut adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect("post-dispatch deadline crossing is typed uncertainty"),
            Outcome::DeliveryUnknown { cause, .. } if cause == expected_cause
        ));
        assert!(matches!(store.only_record().state, State::Open(_)));
        assert_eq!(adapter.dispatch_calls, 1);

        adapter.dispatch_clock_advance = None;
        assert!(matches!(
            coordinator
                .coordinate(
                    &lock,
                    &request,
                    &mut store,
                    &mut adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect("open attempt reconciles on the next invocation"),
            Outcome::Reconciled(_)
        ));
        assert_eq!(adapter.dispatch_calls, 1);
        assert_eq!(adapter.reconcile_calls, 1);
    }

    let clock = Arc::new(ManualMonotonicClock::default());
    let coordinator = EffectCoordinator::with_monotonic_clock(bounded, clock.clone());
    let uncertain_request = submission(primary_selection(), DuplicateGuarantee::None);
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::uncertain(
        primary_selection(),
        events,
        ReconcilePlan::Accepted,
        DuplicateGuarantee::None,
    );
    coordinator
        .coordinate(
            &lock,
            &uncertain_request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("initial uncertain dispatch");
    adapter.reconciliation_clock_advance = Some((clock, 101));
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &uncertain_request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("post-reconciliation deadline crossing is typed uncertainty"),
        Outcome::DeliveryUnknown {
            cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
            ..
        }
    ));
    assert!(matches!(
        store.only_record().state,
        State::DeliveryUnknown(_)
    ));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 1);

    adapter.reconciliation_clock_advance = None;
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &uncertain_request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("uncertain attempt remains reconcilable on a fresh invocation"),
        Outcome::Reconciled(_)
    ));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 2);
}

#[test]
#[trace("TC-003", "FR-003-AC-2")]
fn tc003_reconciliation_failures_remain_typed_delivery_unknown() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(primary_selection(), DuplicateGuarantee::None);

    for (plan, expected_cause) in [
        (
            ReconcilePlan::Error,
            DeliveryUnknownCause::ReconciliationUnavailable,
        ),
        (
            ReconcilePlan::ContractViolation,
            DeliveryUnknownCause::ReconciliationContractViolation,
        ),
        (
            ReconcilePlan::InvalidReceipt,
            DeliveryUnknownCause::InvalidReceipt,
        ),
        (
            ReconcilePlan::OversizedReceipt,
            DeliveryUnknownCause::InvalidReceipt,
        ),
    ] {
        let events = EventLog::default();
        let mut store = MemoryStore::with_events(events.clone());
        let mut adapter =
            TestAdapter::uncertain(primary_selection(), events, plan, DuplicateGuarantee::None);
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("initial indeterminate dispatch");
        assert!(matches!(
            coordinator
                .coordinate(
                    &lock,
                    &request,
                    &mut store,
                    &mut adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect("reconciliation uncertainty is retained"),
            Outcome::DeliveryUnknown { cause, .. } if cause == expected_cause
        ));
        assert_eq!(adapter.dispatch_calls, 1);
        assert_eq!(adapter.reconcile_calls, 1);
        assert!(matches!(
            store.only_record().state,
            State::DeliveryUnknown(_)
        ));
        assert!(matches!(
            coordinator
                .coordinate(
                    &lock,
                    &request,
                    &mut store,
                    &mut adapter,
                    &TestDecisionAuthority::valid(),
                    &TestObservationAuthority::valid(),
                )
                .expect("a third replay remains parseable and typed"),
            Outcome::DeliveryUnknown { cause, .. } if cause == expected_cause
        ));
        assert_eq!(adapter.dispatch_calls, 1);
        assert_eq!(adapter.reconcile_calls, 2);
    }

    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::uncertain(
        primary_selection(),
        events,
        ReconcilePlan::Accepted,
        DuplicateGuarantee::None,
    );
    coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("initial indeterminate dispatch");
    store.fail_receipt_once = true;
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("reconciliation receipt persistence failure stays uncertain"),
        Outcome::DeliveryUnknown {
            cause: DeliveryUnknownCause::ReceiptPersistenceFailure,
            ..
        }
    ));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 1);
    assert!(matches!(
        store.only_record().state,
        State::DeliveryUnknown(_)
    ));
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("receipt persistence recovery remains parseable"),
        Outcome::Reconciled(_)
    ));
    assert_eq!(adapter.dispatch_calls, 1);
    assert_eq!(adapter.reconcile_calls, 2);
}

#[test]
#[trace("TC-003", "FR-003-AC-5")]
fn tc003_observation_without_receipt_is_validated_exactly_and_bound_to_creation_lock() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(primary_selection(), DuplicateGuarantee::None);
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::uncertain(
        primary_selection(),
        events,
        ReconcilePlan::Inconclusive,
        DuplicateGuarantee::None,
    );
    coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("durable uncertain request");
    let observation = Observation {
        id: effect_id!(ObservationId, "observation:no-receipt"),
        request_id: store.only_record().request.id().clone(),
        external_effect_id: None,
        authority: observation_selection(),
        payload: vec![7; 1024],
    };
    let authority = TestObservationAuthority {
        selection: observation_selection(),
        qualified: true,
        calls: Cell::new(0),
    };

    let mut oversized = observation.clone();
    oversized.payload.push(8);
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &oversized,
                &mut store,
                &authority,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("one-over observation payload"),
        Error::ResourceExhausted(
            quire_integration::effect_reconciliation::Resource::ObservationBytes
        )
    );
    let other_lock = validated_lock_with_deployment_revision('6');
    assert_eq!(
        coordinator
            .observe(
                &other_lock,
                &request.key,
                &observation,
                &mut store,
                &authority,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("a different lock cannot qualify an old request"),
        Error::PersistenceFailure
    );
    store.substitute_observation_once = true;
    assert_eq!(
        coordinator
            .observe(
                &lock,
                &request.key,
                &observation,
                &mut store,
                &authority,
                &TestDecisionAuthority::valid(),
            )
            .expect_err("store cannot substitute same-id observation content"),
        Error::PersistenceFailure
    );

    let events = EventLog::default();
    let mut exact_store = MemoryStore::with_events(events.clone());
    let mut exact_adapter = TestAdapter::uncertain(
        primary_selection(),
        events,
        ReconcilePlan::Inconclusive,
        DuplicateGuarantee::None,
    );
    coordinator
        .coordinate(
            &lock,
            &request,
            &mut exact_store,
            &mut exact_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("second durable uncertain request");
    let observed = coordinator
        .observe(
            &lock,
            &request.key,
            &observation,
            &mut exact_store,
            &authority,
            &TestDecisionAuthority::valid(),
        )
        .expect("exact-limit observation without receipt establishes effect");
    assert_eq!(observed, Outcome::EffectObserved(observation));
    assert!(matches!(
        exact_store.only_record().state,
        State::EffectObserved { receipt: None, .. }
    ));
}

#[test]
#[trace("TC-003", "FR-003-AC-1", "FR-003-AC-4")]
fn tc003_persisted_request_round_trips_strictly_and_supersession_is_immutable() {
    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let original = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    coordinator
        .coordinate(
            &lock,
            &original,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("original request");
    let persisted = store.only_record().request.clone();
    let rebuilt = EffectRequest::from_persisted(
        PersistedEffectRequest {
            id: persisted.id().clone(),
            lock_identity: persisted.lock_identity(),
            recommendation: persisted.recommendation().clone(),
            decision: persisted.decision().clone(),
            authority: persisted.authority().clone(),
            key: persisted.key().clone(),
            selection: persisted.selection().clone(),
            authority_grant_id: persisted.authority_grant_id().clone(),
            operation: persisted.operation().clone(),
            canonical_arguments: persisted.canonical_arguments().to_vec(),
            supersedes: persisted.supersedes().cloned(),
            canonical_bytes: persisted.canonical_bytes().to_vec(),
        },
        limits(),
    )
    .expect("strict reconstruction");
    assert_eq!(rebuilt, persisted);
    let mut mutated_arguments = persisted.canonical_arguments().to_vec();
    mutated_arguments.push(0);
    assert_eq!(
        EffectRequest::from_persisted(
            PersistedEffectRequest {
                id: persisted.id().clone(),
                lock_identity: persisted.lock_identity(),
                recommendation: persisted.recommendation().clone(),
                decision: persisted.decision().clone(),
                authority: persisted.authority().clone(),
                key: persisted.key().clone(),
                selection: persisted.selection().clone(),
                authority_grant_id: persisted.authority_grant_id().clone(),
                operation: persisted.operation().clone(),
                canonical_arguments: mutated_arguments,
                supersedes: persisted.supersedes().cloned(),
                canonical_bytes: persisted.canonical_bytes().to_vec(),
            },
            limits(),
        )
        .expect_err("duplicated fields cannot diverge from canonical bytes"),
        Error::PersistenceFailure
    );
    let mut substituted_time = persisted_fields(&persisted);
    substituted_time.authority.expires_at_ms -= 1;
    assert_eq!(
        EffectRequest::from_persisted(substituted_time, limits())
            .expect_err("persisted authority time must match canonical bytes"),
        Error::PersistenceFailure
    );
    let mut substituted_grant = persisted_fields(&persisted);
    substituted_grant.authority_grant_id =
        effect_id!(AuthorityGrantId, "sha256:foreign-top-level-grant");
    assert_eq!(
        EffectRequest::from_persisted(substituted_grant, limits())
            .expect_err("top-level grant identity must match retained owner artifact"),
        Error::PersistenceFailure
    );

    let old_bytes = persisted.canonical_bytes().to_vec();
    let mut revision = original;
    revision.key = effect_id!(IdempotencyKey, "caller-key:refund:42:revision:2");
    revision.decision.supersedes = Some(revision.decision.id.clone());
    revision
        .decision
        .artifact
        .canonical_bytes
        .extend_from_slice(b":revision:2");
    revision.decision.id = effect_id!(
        DecisionId,
        digest(&revision.decision.artifact.canonical_bytes)
    );
    revision.supersedes_request = Some(persisted.id().clone());
    let events = EventLog::default();
    let mut revision_store = MemoryStore::with_events(events.clone());
    let mut revision_adapter = TestAdapter::accepted(primary_selection(), events);
    coordinator
        .coordinate(
            &lock,
            &revision,
            &mut revision_store,
            &mut revision_adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("new immutable superseding request");
    assert_eq!(
        revision_store.only_record().request.supersedes(),
        Some(persisted.id())
    );
    assert_eq!(persisted.canonical_bytes(), old_bytes);

    let mut self_cycle = revision;
    self_cycle.decision.supersedes = Some(self_cycle.decision.id.clone());
    let events = EventLog::default();
    let mut cycle_store = MemoryStore::with_events(events.clone());
    let mut cycle_adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        coordinator
            .coordinate(
                &lock,
                &self_cycle,
                &mut cycle_store,
                &mut cycle_adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("decision supersession cannot cycle to itself"),
        Error::DecisionMismatch
    );
}

#[test]
#[trace("TC-003", "FR-003-AC-1")]
fn tc003_argument_and_receipt_payload_limits_accept_exact_and_refuse_one_over() {
    let lock = validated_lock();
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let argument_length = u64::try_from(request.canonical_arguments.len()).expect("length");

    let events = EventLog::default();
    let mut bounded_load = MemoryStore::with_events(events.clone());
    bounded_load.fail_load_resource_once = Some(Resource::ReceiptBytes);
    let mut bounded_adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        EffectCoordinator::new(limits())
            .coordinate(
                &lock,
                &request,
                &mut bounded_load,
                &mut bounded_adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("typed load preflight exhaustion"),
        Error::ResourceExhausted(Resource::ReceiptBytes)
    );
    assert_eq!(bounded_adapter.dispatch_calls, 0);

    let mut zero_duration = limits();
    zero_duration.max_execution_duration_ms = 0;
    let events = EventLog::default();
    let mut duration_store = MemoryStore::with_events(events.clone());
    let mut duration_adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        EffectCoordinator::new(zero_duration)
            .coordinate(
                &lock,
                &request,
                &mut duration_store,
                &mut duration_adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("zero execution duration refuses before any boundary invocation"),
        Error::ResourceExhausted(Resource::ExecutionDuration)
    );
    assert_eq!(duration_store.writes(), 0);
    assert_eq!(duration_adapter.dispatch_calls, 0);

    let mut short_duration = limits();
    short_duration.max_execution_duration_ms = 100;
    let clock = Arc::new(ManualMonotonicClock::default());
    let events = EventLog::default();
    let mut duration_store = MemoryStore::with_events(events.clone());
    let mut duration_adapter = TestAdapter::accepted(primary_selection(), events);
    let advancing_authority = AdvancingDecisionAuthority {
        decision_selection: decision_authority_selection(),
        grant_selection: grant_authority_selection(),
        calls: Cell::new(0),
        advance_on_call: 5,
        advance_ms: 40,
        clock: Arc::clone(&clock),
    };
    assert!(matches!(
        EffectCoordinator::with_monotonic_clock(short_duration, clock)
            .coordinate(
                &lock,
                &request,
                &mut duration_store,
                &mut duration_adapter,
                &advancing_authority,
                &TestObservationAuthority::valid(),
            )
            .expect("scripted duration remains within the total budget"),
        Outcome::Delivered(_)
    ));
    assert_eq!(advancing_authority.calls.get(), 5);
    assert_eq!(duration_adapter.last_dispatch_duration_ms, Some(60));
    assert_eq!(duration_store.duration_budgets.last(), Some(&60));

    let mut exact_limits = limits();
    exact_limits.max_argument_bytes = argument_length;
    exact_limits.max_receipt_bytes = 1024;
    let coordinator = EffectCoordinator::new(exact_limits);
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    adapter.receipt_payload_bytes = Some(1024);
    assert!(matches!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect("exact argument and receipt limits"),
        Outcome::Delivered(_)
    ));

    let mut short_limits = limits();
    short_limits.max_argument_bytes = argument_length - 1;
    let coordinator = EffectCoordinator::new(short_limits);
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        coordinator
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("one-over arguments refuse before persistence"),
        Error::ResourceExhausted(quire_integration::effect_reconciliation::Resource::ArgumentBytes)
    );
    assert_eq!(store.writes(), 0);
    assert_eq!(adapter.dispatch_calls, 0);

    let mut exact_history = limits();
    exact_history.max_history_events = 3;
    exact_history.max_attempts = 1;
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    EffectCoordinator::new(exact_history)
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("exact history and attempt limits");

    let mut one_under_history = limits();
    one_under_history.max_history_events = 2;
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        EffectCoordinator::new(one_under_history)
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("one-over history refuses before transport"),
        Error::ResourceExhausted(Resource::HistoryEvents)
    );
    assert_eq!(adapter.dispatch_calls, 0);

    let mut zero_attempts = limits();
    zero_attempts.max_attempts = 0;
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    assert_eq!(
        EffectCoordinator::new(zero_attempts)
            .coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &TestDecisionAuthority::valid(),
                &TestObservationAuthority::valid(),
            )
            .expect_err("one-over attempt count refuses before transport"),
        Error::ResourceExhausted(Resource::Attempts)
    );
    assert_eq!(adapter.dispatch_calls, 0);
}

#[test]
fn tc009_tc011_dispatch_requires_coordinator_permit_after_persistence() {
    let source = include_str!("../src/effect_reconciliation.rs");
    let capability_source = include_str!("../src/capability.rs");
    let crate_root = include_str!("../src/lib.rs");

    assert_eq!(
        source.matches(".dispatch(").count(),
        1,
        "the coordinator must own the sole adapter invocation"
    );
    let permit_fields = source
        .split_once("pub struct DispatchPermit")
        .expect("dispatch permit declaration")
        .1
        .split_once("impl DispatchPermit")
        .expect("dispatch permit implementation")
        .0;
    assert!(!permit_fields.contains("pub request_id"));
    assert!(!permit_fields.contains("pub attempt_id"));
    assert!(!permit_fields.contains("pub lock_identity"));
    assert!(!source.contains("pub fn invoke_adapter"));
    assert!(!capability_source.contains("EffectAdapterCapability"));
    assert!(!crate_root.contains("EffectAdapterCapability"));

    let lock = validated_lock();
    let coordinator = EffectCoordinator::new(limits());
    let request = submission(
        primary_selection(),
        DuplicateGuarantee::SameKeyAndBytesSameEffect,
    );
    let events = EventLog::default();
    let mut store = MemoryStore::with_events(events.clone());
    let mut adapter = TestAdapter::accepted(primary_selection(), events);
    coordinator
        .coordinate(
            &lock,
            &request,
            &mut store,
            &mut adapter,
            &TestDecisionAuthority::valid(),
            &TestObservationAuthority::valid(),
        )
        .expect("coordinator-minted permit dispatches");
    assert_eq!(adapter.permit_checks, 1);
    assert!(matches!(
        store.only_record().history.as_slice(),
        [
            Event::RequestPersisted,
            Event::AttemptPrepared(_),
            Event::ReceiptPersisted(_)
        ]
    ));
}
