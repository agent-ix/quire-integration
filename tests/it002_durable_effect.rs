// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

#[path = "support/it002.rs"]
mod support;

use std::{
    fs,
    net::SocketAddr,
    path::Path,
    process::{Command, ExitStatus},
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use ix_trace_rs::trace;
use quire_integration::effect_reconciliation::{
    ActorId, AdapterError, AuthorityScope, DeliveryUnknownCause, EffectCoordinator, Error,
    ExternalEffectId, ObservationId, Outcome, Resource, ScopeId,
};
use support::{
    limits, matching_observation, parse_http_response, read_durable, submission, validated_lock,
    AdapterCrash, ContractObservationAuthority, CrashPoint, FileStore, HttpAdapter, HttpService,
    LocalDecisionAuthority, ManifestMode, TestDirectory, CRASH_AFTER_ACCEPTANCE,
    CRASH_AFTER_ATTEMPT, CRASH_AFTER_RESERVATION, ID_LIMIT,
};

fn child(
    root: &Path,
    address: SocketAddr,
    mode: ManifestMode,
    crash: &str,
    expected: &str,
    current: bool,
) -> ExitStatus {
    let mut child = Command::new(std::env::current_exe().expect("IT-002 test executable"))
        .arg("--exact")
        .arg("it002_child_driver")
        .arg("--ignored")
        .arg("--nocapture")
        .env("QINT_IT002_CHILD", "1")
        .env("QINT_IT002_ROOT", root)
        .env("QINT_IT002_ADDRESS", address.to_string())
        .env(
            "QINT_IT002_MANIFEST",
            match mode {
                ManifestMode::Safe => "safe",
                ManifestMode::Unsupported => "unsupported",
            },
        )
        .env("QINT_IT002_CRASH", crash)
        .env("QINT_IT002_EXPECTED", expected)
        .env("QINT_IT002_CURRENT", if current { "yes" } else { "no" })
        .spawn()
        .expect("spawn isolated IT-002 coordinator");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.try_wait().expect("poll isolated IT-002 coordinator") {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed-out IT-002 coordinator");
            let _ = child.wait();
            panic!("isolated IT-002 coordinator exceeded 15-second deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn assert_exit(status: ExitStatus, expected: i32) {
    assert_eq!(
        status.code(),
        Some(expected),
        "unexpected child status {status}"
    );
}

#[trace("TC-003")]
#[test]
#[ignore = "private subprocess entry point; parent scenarios invoke it explicitly"]
fn it002_child_driver() {
    if std::env::var_os("QINT_IT002_CHILD").is_none() {
        return;
    }
    let root = std::env::var_os("QINT_IT002_ROOT").expect("child root");
    let address = std::env::var("QINT_IT002_ADDRESS")
        .expect("child address")
        .parse::<SocketAddr>()
        .expect("child socket address");
    let mode = match std::env::var("QINT_IT002_MANIFEST").as_deref() {
        Ok("safe") => ManifestMode::Safe,
        Ok("unsupported") => ManifestMode::Unsupported,
        other => panic!("unknown child manifest {other:?}"),
    };
    let (store_crash, adapter_crash) = match std::env::var("QINT_IT002_CRASH").as_deref() {
        Ok("reservation") => (CrashPoint::AfterReservation, AdapterCrash::None),
        Ok("attempt") => (CrashPoint::AfterAttempt, AdapterCrash::None),
        Ok("acceptance") => (CrashPoint::None, AdapterCrash::AfterAcceptance),
        Ok("none") => (CrashPoint::None, AdapterCrash::None),
        other => panic!("unknown child crash point {other:?}"),
    };
    let current = std::env::var("QINT_IT002_CURRENT").as_deref() == Ok("yes");
    let lock = validated_lock();
    let mut store = FileStore::new(root, store_crash, lock.identity());
    let mut adapter = HttpAdapter::new(address, mode, adapter_crash);
    let outcome = EffectCoordinator::new(limits()).coordinate(
        &lock,
        &submission(mode, "caller-key:refund:42", 42),
        &mut store,
        &mut adapter,
        &LocalDecisionAuthority { current },
        &ContractObservationAuthority,
    );
    match std::env::var("QINT_IT002_EXPECTED").as_deref() {
        Ok("delivered") => assert!(matches!(outcome, Ok(Outcome::Delivered(_))), "{outcome:?}"),
        Ok("reconciled") => {
            assert!(matches!(outcome, Ok(Outcome::Reconciled(_))), "{outcome:?}")
        }
        Ok("unknown") => assert!(
            matches!(
                outcome,
                Ok(Outcome::DeliveryUnknown {
                    cause: DeliveryUnknownCause::ReconciliationUnavailable,
                    ..
                })
            ),
            "{outcome:?}"
        ),
        Ok("retry-unsupported") => {
            assert_eq!(outcome, Err(Error::RetryUnsupported), "{outcome:?}")
        }
        Ok("crash") => panic!("configured crash point returned: {outcome:?}"),
        other => panic!("unknown child expectation {other:?}"),
    }
}

#[trace("TC-003", "FR-003-AC-1", "FR-003-AC-2")]
#[test]
fn persistence_and_open_attempt_survive_real_process_termination() {
    let service = HttpService::start();
    let after_reservation = TestDirectory::new("it002-after-reservation");
    assert_exit(
        child(
            after_reservation.path(),
            service.address(),
            ManifestMode::Safe,
            "reservation",
            "crash",
            true,
        ),
        CRASH_AFTER_RESERVATION,
    );
    assert_eq!(
        read_durable(after_reservation.path()).event_kinds(),
        ["request_persisted"]
    );
    assert!(service.calls().is_empty());
    assert!(child(
        after_reservation.path(),
        service.address(),
        ManifestMode::Safe,
        "none",
        "delivered",
        true,
    )
    .success());
    assert_eq!(
        read_durable(after_reservation.path()).event_kinds(),
        ["request_persisted", "attempt_prepared", "receipt"]
    );

    let attempt_service = HttpService::start();
    let after_attempt = TestDirectory::new("it002-after-attempt");
    assert_exit(
        child(
            after_attempt.path(),
            attempt_service.address(),
            ManifestMode::Safe,
            "attempt",
            "crash",
            true,
        ),
        CRASH_AFTER_ATTEMPT,
    );
    assert!(attempt_service.calls().is_empty());
    assert_eq!(
        read_durable(after_attempt.path()).event_kinds(),
        ["request_persisted", "attempt_prepared"]
    );
    assert!(child(
        after_attempt.path(),
        attempt_service.address(),
        ManifestMode::Safe,
        "none",
        "delivered",
        true,
    )
    .success());
    assert_eq!(
        attempt_service.calls(),
        ["GET caller-key:refund:42", "POST caller-key:refund:42"]
    );
    assert_eq!(
        read_durable(after_attempt.path()).event_kinds(),
        [
            "request_persisted",
            "attempt_prepared",
            "reconciliation",
            "attempt_prepared",
            "receipt"
        ]
    );
}

#[trace("TC-003", "FR-003-AC-2")]
#[test]
fn post_acceptance_termination_reconciles_or_stays_delivery_unknown() {
    let safe_service = HttpService::start();
    let safe = TestDirectory::new("it002-post-acceptance-safe");
    assert_exit(
        child(
            safe.path(),
            safe_service.address(),
            ManifestMode::Safe,
            "acceptance",
            "crash",
            true,
        ),
        CRASH_AFTER_ACCEPTANCE,
    );
    assert_eq!(safe_service.effect_count(), 1);
    let accepted_external = safe_service
        .external_effect_id("caller-key:refund:42")
        .expect("provider accepted effect identity");
    assert_eq!(
        read_durable(safe.path()).event_kinds(),
        ["request_persisted", "attempt_prepared"]
    );
    assert!(child(
        safe.path(),
        safe_service.address(),
        ManifestMode::Safe,
        "none",
        "reconciled",
        true,
    )
    .success());
    assert_eq!(
        safe_service.calls(),
        ["POST caller-key:refund:42", "GET caller-key:refund:42"]
    );
    assert_eq!(safe_service.effect_count(), 1);
    assert_eq!(
        read_durable(safe.path()).receipt_external_effect_id(),
        Some(accepted_external.as_str()),
        "reconciled durable receipt identifies the provider-accepted effect"
    );

    let unsupported_service = HttpService::start();
    let unsupported = TestDirectory::new("it002-post-acceptance-unsupported");
    assert_exit(
        child(
            unsupported.path(),
            unsupported_service.address(),
            ManifestMode::Unsupported,
            "acceptance",
            "crash",
            true,
        ),
        CRASH_AFTER_ACCEPTANCE,
    );
    assert!(child(
        unsupported.path(),
        unsupported_service.address(),
        ManifestMode::Unsupported,
        "none",
        "unknown",
        true,
    )
    .success());
    assert_eq!(unsupported_service.calls(), ["POST caller-key:refund:42"]);
    assert_eq!(unsupported_service.effect_count(), 1);
    assert_eq!(
        read_durable(unsupported.path()).event_kinds(),
        ["request_persisted", "attempt_prepared", "reconciliation"]
    );
}

#[trace("TC-003", "FR-003-AC-2", "FR-003-AC-3", "FR-003-AC-4")]
#[test]
fn replay_authority_expiry_and_contract_observation_stay_independent() {
    let service = HttpService::start();
    let directory = TestDirectory::new("it002-authority-observation");
    let coordinator = EffectCoordinator::new(limits());
    let lock = validated_lock();
    let mut store = FileStore::new(directory.path(), CrashPoint::None, lock.identity());
    let mut adapter = HttpAdapter::new(service.address(), ManifestMode::Safe, AdapterCrash::None);
    let valid = submission(ManifestMode::Safe, "caller-key:refund:42", 42);
    let authority = LocalDecisionAuthority { current: true };
    let receipt = match coordinator
        .coordinate(
            &lock,
            &valid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        )
        .expect("fresh authorized delivery")
    {
        Outcome::Delivered(receipt) => receipt,
        other => panic!("fresh request was not delivered: {other:?}"),
    };
    let calls = service.calls();
    assert_eq!(calls, ["POST caller-key:refund:42"]);
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &valid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Ok(Outcome::Delivered(receipt.clone()))
    );
    assert_eq!(
        service.calls(),
        calls,
        "durable replay made no transport call"
    );

    let durable_before_conflict = read_durable(directory.path());
    let conflict = submission(ManifestMode::Safe, "caller-key:refund:42", 43);
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &conflict,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Err(Error::KeyConflict)
    );
    assert_eq!(service.calls(), calls);
    assert_eq!(
        read_durable(directory.path()),
        durable_before_conflict,
        "conflicting key reuse changed no durable state"
    );
    let durable_before_invalid_authority = read_durable(directory.path());

    let mut invalid = submission(ManifestMode::Safe, "caller-key:invalid", 42);
    invalid.authority = None;
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &invalid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Err(Error::AuthorityMissing)
    );
    let mut invalid = submission(ManifestMode::Safe, "caller-key:invalid", 42);
    invalid.authority.as_mut().expect("grant").scope = AuthorityScope::All;
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &invalid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Err(Error::AuthorityOverBroad)
    );
    let mut invalid = submission(ManifestMode::Safe, "caller-key:invalid", 42);
    invalid.decision.actor = ActorId::new("operator:bob", ID_LIMIT).expect("actor");
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &invalid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Err(Error::AuthorityMismatch)
    );
    let invalid = submission(ManifestMode::Safe, "caller-key:expired", 42);
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &invalid,
            &mut store,
            &mut adapter,
            &LocalDecisionAuthority { current: false },
            &ContractObservationAuthority,
        ),
        Err(Error::AuthorityExpired)
    );
    assert_eq!(service.calls(), calls);
    let mut invalid = submission(ManifestMode::Safe, "caller-key:invalid", 42);
    invalid.authority.as_mut().expect("grant").scope = AuthorityScope::Exact(
        ScopeId::new("workflow:refund:foreign", ID_LIMIT).expect("foreign scope"),
    );
    assert_eq!(
        coordinator.coordinate(
            &lock,
            &invalid,
            &mut store,
            &mut adapter,
            &authority,
            &ContractObservationAuthority,
        ),
        Err(Error::AuthorityMismatch)
    );
    assert_eq!(
        read_durable(directory.path()),
        durable_before_invalid_authority,
        "invalid authority and conflicting key changed no durable state"
    );

    let mut foreign = matching_observation(&receipt);
    foreign.id = ObservationId::new("observation:foreign", ID_LIMIT).expect("observation");
    foreign.external_effect_id =
        Some(ExternalEffectId::new("effect:foreign", ID_LIMIT).expect("foreign effect identity"));
    assert_eq!(
        coordinator.observe(
            &lock,
            &valid.key,
            &foreign,
            &mut store,
            &ContractObservationAuthority,
            &authority,
        ),
        Err(Error::ForeignObservation)
    );
    let observation = matching_observation(&receipt);
    assert_eq!(
        coordinator.observe(
            &lock,
            &valid.key,
            &observation,
            &mut store,
            &ContractObservationAuthority,
            &authority,
        ),
        Ok(Outcome::EffectObserved(observation))
    );

    let expired_service = HttpService::start();
    let expired = TestDirectory::new("it002-expired-open-attempt");
    assert_exit(
        child(
            expired.path(),
            expired_service.address(),
            ManifestMode::Safe,
            "attempt",
            "crash",
            true,
        ),
        CRASH_AFTER_ATTEMPT,
    );
    assert!(child(
        expired.path(),
        expired_service.address(),
        ManifestMode::Safe,
        "none",
        "retry-unsupported",
        false,
    )
    .success());
    assert_eq!(
        expired_service.calls(),
        ["GET caller-key:refund:42"],
        "expiry permits reconciliation but no new dispatch"
    );

    let accepted_service = HttpService::start();
    let accepted = TestDirectory::new("it002-expired-accepted-attempt");
    assert_exit(
        child(
            accepted.path(),
            accepted_service.address(),
            ManifestMode::Safe,
            "acceptance",
            "crash",
            true,
        ),
        CRASH_AFTER_ACCEPTANCE,
    );
    assert!(child(
        accepted.path(),
        accepted_service.address(),
        ManifestMode::Safe,
        "none",
        "reconciled",
        false,
    )
    .success());
    assert!(child(
        accepted.path(),
        accepted_service.address(),
        ManifestMode::Safe,
        "none",
        "delivered",
        false,
    )
    .success());
    assert_eq!(
        accepted_service.calls(),
        ["POST caller-key:refund:42", "GET caller-key:refund:42"],
        "expiry preserves reconciliation and durable receipt replay"
    );
}

#[trace("TC-003", "FR-003-AC-4")]
#[test]
fn unequal_fresh_requests_race_through_one_atomic_reservation() {
    let service = HttpService::start();
    let directory = Arc::new(TestDirectory::new("it002-atomic-race"));
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for amount in [42, 43] {
        let directory = Arc::clone(&directory);
        let barrier = Arc::clone(&barrier);
        let address = service.address();
        workers.push(thread::spawn(move || {
            let coordinator = EffectCoordinator::new(limits());
            let lock = validated_lock();
            let mut store = FileStore::new(directory.path(), CrashPoint::None, lock.identity())
                .with_reservation_barrier(Arc::clone(&barrier));
            let mut adapter = HttpAdapter::new(address, ManifestMode::Safe, AdapterCrash::None);
            let request = submission(ManifestMode::Safe, "caller-key:refund:race", amount);
            barrier.wait();
            coordinator.coordinate(
                &lock,
                &request,
                &mut store,
                &mut adapter,
                &LocalDecisionAuthority { current: true },
                &ContractObservationAuthority,
            )
        }));
    }
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("race worker"))
        .collect::<Vec<_>>();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Ok(Outcome::Delivered(_))))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| **result == Err(Error::KeyConflict))
            .count(),
        1
    );
    assert_eq!(service.effect_count(), 1);
    assert_eq!(service.calls().len(), 1);
    assert_eq!(
        read_durable(directory.path()).event_kinds(),
        ["request_persisted", "attempt_prepared", "receipt"]
    );
}

#[trace("TC-003", "FR-003-AC-2")]
#[test]
fn equal_fresh_requests_share_one_serialized_transport_attempt() {
    let service = HttpService::start();
    let directory = Arc::new(TestDirectory::new("it002-equal-atomic-race"));
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let directory = Arc::clone(&directory);
        let barrier = Arc::clone(&barrier);
        let address = service.address();
        workers.push(thread::spawn(move || {
            let coordinator = EffectCoordinator::new(limits());
            let lock = validated_lock();
            let mut store = FileStore::new(directory.path(), CrashPoint::None, lock.identity())
                .with_reservation_barrier(Arc::clone(&barrier));
            let request = submission(ManifestMode::Safe, "caller-key:refund:equal-race", 42);
            coordinator.coordinate(
                &lock,
                &request,
                &mut store,
                &mut HttpAdapter::new(address, ManifestMode::Safe, AdapterCrash::None),
                &LocalDecisionAuthority { current: true },
                &ContractObservationAuthority,
            )
        }));
    }
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("equal-race worker"))
        .collect::<Vec<_>>();
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Ok(Outcome::Delivered(_)))),
        "equal concurrent replay outcomes: {results:?}"
    );
    assert_eq!(service.effect_count(), 1);
    assert_eq!(service.calls(), ["POST caller-key:refund:equal-race"]);
    assert_eq!(
        read_durable(directory.path()).event_kinds(),
        ["request_persisted", "attempt_prepared", "receipt"]
    );
}

#[trace("TC-003")]
#[test]
fn corrupt_and_oversized_journals_fail_without_panicking() {
    for (label, bytes) in [
        ("unknown-field", br#"{"unexpected":"field"}"#.to_vec()),
        ("malformed-json", b"{".to_vec()),
        ("oversized", vec![b'x'; 300_000]),
    ] {
        let service = HttpService::start();
        let directory = TestDirectory::new(label);
        fs::write(directory.path().join("state.json"), bytes).expect("write invalid journal");
        let lock = validated_lock();
        let outcome = EffectCoordinator::new(limits()).coordinate(
            &lock,
            &submission(ManifestMode::Safe, "caller-key:refund:42", 42),
            &mut FileStore::new(directory.path(), CrashPoint::None, lock.identity()),
            &mut HttpAdapter::new(service.address(), ManifestMode::Safe, AdapterCrash::None),
            &LocalDecisionAuthority { current: true },
            &ContractObservationAuthority,
        );
        let expected = if label == "oversized" {
            Err(Error::ResourceExhausted(Resource::RequestBytes))
        } else {
            Err(Error::PersistenceFailure)
        };
        assert_eq!(outcome, expected, "case {label}");
        assert!(service.calls().is_empty(), "case {label}");
    }
}

#[trace("TC-003", "FR-003-AC-2")]
#[test]
fn adapter_rejects_receipts_beyond_the_permit_bound() {
    let response =
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 5\r\n\r\n12345";
    assert_eq!(
        parse_http_response(response, 4),
        Err(AdapterError::ContractViolation)
    );
    let wrong_type = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\n{}";
    assert_eq!(
        parse_http_response(wrong_type, 4),
        Err(AdapterError::ContractViolation)
    );
    let duplicate_length = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}";
    assert_eq!(
        parse_http_response(duplicate_length, 4),
        Err(AdapterError::ContractViolation)
    );
}
