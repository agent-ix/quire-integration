---
id: SR-010
title: "Rust review of the Task-002 exact ecosystem lock"
type: SpecReview
analysis: code-review
scope: "Task-002, src/ecosystem_lock.rs, src/capability.rs, schemas/quire.ecosystem-lock-v1.schema.json and tests/ecosystem_lock.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-integration/Task-002, type: reviews }
  - { target: ix://agent-ix/quire-integration/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-integration/TC-001, type: reviews }
---
# SR-010: Rust Review of the Task-002 Exact Ecosystem Lock

## Summary

Reviewed the lock model, strict reader/writer, capability seam, resource preflight,
content identities and TC-001 properties against the repository idioms and Rust-review
checklist. Independent review ran iteratively; every high and medium finding was fixed and
re-reviewed rather than waived.

## Verdict

**PASS for Task-002.** No unresolved correctness, idiom, authority-boundary, panic, unsafe,
conversion, resource, determinism, trace or test-quality finding remains in the reviewed
subset.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: producer and consumer capabilities were initially callable without the lock. Both public traits now require the constructor-private `ValidatedLock`, and TC-001 executes the real producer seam only after validation. | `src/capability.rs`; FR-001-AC-3 |
| FND-002 | high | Resolved: caller-asserted revision labels and unbounded field constructors could retain moving or over-bound input. Pins now accept only bounded canonical Git/SHA-256 identities; remotes, versions, repository revisions, features and contracts are bounded before retention, and moving versions refuse. | `Pin`; `RepositoryBinding`; `PackageBinding`; `DependencyEdge` |
| FND-003 | high | Resolved: output exhaustion was detected after semantic allocations. Allocation-free borrowed serializers now measure output before semantic allocation; final serialization reserves the checked exact capacity. | `preflight_output`; `serialize_wire`; NFR-001-AC-1 |
| FND-004 | high | Resolved: the first package wire omitted the explicit semantic version. The bounded exact SemVer value now participates independently from the content revision in input accounting, canonical bytes, lock identity and run identity. | `PackageBinding`; `PackageWire`; FR-001-AC-2 |
| FND-005 | medium | Resolved: package features were an opaque order-sensitive string. They are now a bounded, sorted, duplicate-free set with schema parity and reversed-order equality evidence. | `retain_feature_set`; FR-001-AC-1 |
| FND-006 | medium | Resolved: dependency-free locks were rejected and aggregate refusals lacked an identity. Dependencies are optional, required domains remain nonempty, and aggregate input/traversal/document/output refusals identify the exact lock contract. | `preflight`; `Refusal`; TC-001 |

## Rust Review

| Check | Result |
| --- | --- |
| API and authority boundaries | `ValidatedLock` has no public constructor. Every producer/consumer capability requires it, while candidate mutation remains confined to prevalidation types. |
| Exactness and identity | Repository/artifact pins are content addressed, package SemVer is exact, resource limits are bound into the wire, and every meaning-affecting mutation changes lock and run identities. |
| Bounds and allocation | Counts and field bytes are checked before retention; output is measured without collection allocation; checked integer conversions precede exact-capacity allocation; traversal is finite and preflighted. |
| Errors and topology | Stable refusal codes preserve domain and offending identity. Reachable, unreachable, unresolved and cyclic graph states stay separate from five compatibility states; actual cycles are detected. |
| Determinism | Canonical sorting covers every set-valued wire member, including package features. A 128-case generated permutation property and exact strict-reader byte comparison cover discovery order and noncanonical input. |
| Test quality and trace | Fourteen TC-001 tests include three 128-case properties, all twenty reachability/compatibility combinations, every collection/byte boundary, package-version and feature negatives, real capability invocation, schema validation and zero-dependency closure. Oracles are independent of production sorting. |
| Safety surface | The crate forbids unsafe code at the repository gate. No new async, lock, blocking bridge, background work, recursive traversal, unchecked cast or caller-controlled panic exists. |

## Verification

Formatting, warning-denied all-target/all-feature Clippy, nineteen locked tests, warning-denied
rustdoc, release build, compiler-enforced unsafe audit, cargo-deny advisories/bans/licenses/sources,
strict native Quoin validation and `git diff --check` pass. The schema digest is pinned and its
emitted document validates under Draft 2020-12.

## Boundary

This review closes Task-002 only. It does not claim Tasks 003–013, IT-001–IT-004 or the complete
V1 stakeholder outcome complete.
