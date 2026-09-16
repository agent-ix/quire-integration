---
id: SR-047
title: "Rust review of Task-009 durable effect integration"
type: SpecReview
analysis: code-review
scope: "Task-009 filesystem store, HTTP adapter/service, process crash harness and IT-002 evidence"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-integration/Task-009, type: reviews }
  - { target: ix://agent-ix/quire-integration/FR-003, type: reviews }
  - { target: ix://agent-ix/quire-integration/IT-002, type: reviews }
---
# SR-047: Rust Review of Task-009 Durable Effect Integration

## Summary

Independently reviewed the Task-009 Rust integration harness using the repository Rust-review
checklist. The review covered crash durability, process concurrency, strict persisted-state
reconstruction, adapter bounds and deadlines, authority controls, panic surfaces, trace evidence
and adverse test quality. Every implementation finding was remediated and reviewed again.

## Verdict

**PASS for the implemented IT-002-SC-01..05 and SC-07/08 boundary.** No unresolved correctness,
idiom, persistence, concurrency, resource, deadline, panic, unsafe, trace or test-quality finding
remains in that subset. This verdict does not claim the QObs-owned SC-06 handoff.

## Resolved findings

| ID | Severity | Resolution |
|---|---|---|
| FND-001 | high | Replaced directory-existence reservation and process-local cached revisions with an OS-released cross-process transaction lock, reload-before-CAS writes and one durable state-file publication point. |
| FND-002 | high | Unique temporary files are file-synced, atomically renamed and followed by parent-directory fsync; process termination releases the transaction lock without an empty-directory tombstone. |
| FND-003 | high | Persisted reads are byte-bounded before allocation, history/payload bounds are rechecked, unknown fields refuse, and every disk identity reconstructs fallibly without `expect` or panic. |
| FND-004 | high | Equal and unequal requests enter a deterministic reservation overlap; the equal request produces one serialized HTTP attempt and the unequal request has one winner without overwrite. |
| FND-005 | medium | Adapter connect, write and bounded manual reads share one monotonic permit deadline; response size, exact single content length and exact JSON content type are validated. |
| FND-006 | medium | Observation authority now round-trips in durable state; receipt reconciliation proves the stored external effect identity equals the provider-accepted identity. |
| FND-007 | medium | Exact foreign scope joins missing, expired, over-broad and cross-actor controls, each proving no store or adapter mutation. |
| FND-008 | low | Child processes have a kill/reap deadline, and the private ignored child driver runs only through explicit parent invocations rather than counting as vacuous evidence. |

## Verification

The targeted suite reports seven passing parent scenarios and one intentionally ignored private
child entry point; every explicit crash subprocess passes. Warning-denied Clippy and formatting
pass. The complete repository CI gate passes, including all tests/doctests, cargo-deny and the
unsafe audit. Native Quoin `0.23.1-87-gb3d02d4` strict validation reports no finding.

Reviewed source digests are `74a3093de93e5d769747bd34779e60800e8eccc0e5e6dc42a993d1dc974dc82a`
for `tests/support/it002.rs` and
`6fab53841295db9b98ca902ff000ac3da34cfea35a51a1ee95b140e5f9f14013` for
`tests/it002_durable_effect.rs`.

## Boundary

The local observation contract exercises the lock-selected authority seam but is not QObs
evidence. Task-009 and IT-002 remain partial until QObs publishes the immutable downstream
business-effect handoff required by SC-06.
