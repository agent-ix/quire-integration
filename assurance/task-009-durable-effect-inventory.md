---
id: task-009-durable-effect-inventory
title: "Task-009 durable effect integration inventory"
type: AssuranceRecord
status: partial
relationships:
  - target: ix://agent-ix/quire-integration/IT-002
    type: verifies
---
# Task-009 durable effect integration inventory

## Exercised boundaries

| Boundary | Concrete implementation | Evidence |
| --- | --- | --- |
| Durable request store | Isolated filesystem journal; OS-released cross-process transaction lock, reload-before-CAS revisions, bounded strict reconstruction through `EffectRequest::from_persisted`, unique temporary files, file fsync, rename and parent-directory fsync before return | `tests/support/it002.rs`, `tests/it002_durable_effect.rs` |
| Effect adapter | Loopback TCP HTTP/1.1 service and client using the exact selected idempotency header, dispatch path and reconciliation path; permit-derived timeouts and response limits are enforced before allocation | `tests/support/it002.rs` |
| Process termination | Child test process exits immediately after reservation, attempt persistence, or HTTP acceptance | IT-002-SC-01 through IT-002-SC-03 |
| Safe and unsafe retry contracts | Separate closed manifest fixtures for idempotent reconciliation and for no retry/reconciliation guarantee | IT-002-SC-03 |
| Authority and global key | Exact, missing, expired, over-broad, foreign-scope and cross-actor decisions plus deterministically overlapped equal and unequal concurrent key reuse | IT-002-SC-04, IT-002-SC-05, IT-002-SC-07, IT-002-SC-08 |
| Corrupt-state containment | Unknown journal fields, malformed JSON and an over-limit journal fail closed without a panic or transport call | IT-002-SC-01 |

## Original fixture inventory

Both fixtures are original Agent IX test documents licensed under this repository's
`AGPL-3.0-or-later` license. They contain no copied provider material and authorize no
production payment endpoint.

| Path | SHA-256 | Purpose |
| --- | --- | --- |
| `tests/fixtures/it002-effect-adapter-v1.json` | `d297118bba9b3ed9c5e3b54abbe3a67bae9d305bd1f029010927e01eff17587d` | Same-key/same-bytes idempotency and exact-key reconciliation |
| `tests/fixtures/it002-effect-adapter-unsupported-v1.json` | `2e719d2372be558ee8036abd4e047091ac6250d462a005ecaf6ed980162dd80d` | No duplicate-call guarantee and no reconciliation operation |

## Open owner handoff

QObs main revision `924006300f45b38483be1cbdf99b68f899b7d368` exposes general
`AdmissionRequest` admission and authority document readers, but no immutable,
crate-addressable business-effect observation handoff. Its integration examples depend on
QObs/FCD repository test fixtures whose exact bytes are not approved for copying here.

The harness therefore exercises the lock-selected `ObservationAuthority` seam with an
explicit integration-test contract, but does not tag that evidence as IT-002-SC-06 or claim
the QObs-owned subtask complete. Completion requires an immutable QObs revision that
publishes the owner bytes/identity and matching/foreign controls through its public boundary.
