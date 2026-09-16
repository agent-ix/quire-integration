---
id: SR-015
title: "Base review of Task 004 effect-coordination clarifications"
type: SpecReview
analysis: base
scope: "FR-003; IT-002; Task-004"
review_set: all
---
# SR-015: Base review of Task 004 effect-coordination clarifications

## Summary

The Task 004 slice was reviewed before implementation against its upstream
effect contract and every crash/retry state. The clarified contract has one
canonical key owner, request identity, retry rule, and observation authority.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: the local and upstream prose disagreed on whether the caller or adapter owns the idempotency key and omitted the authority grant from local request identity. FR-003 now follows FR-329's caller-owned key and includes the grant plus full adapter selection. | FR-003; FR-182; FR-329 | wrong-requirement |
| FND-002 | medium | Resolved: IT-002 referenced unrelated upstream protocol-analysis test IT-073. It now references the owning effect test TC-211 and qualification integration IT-078. | IT-002; TC-211; IT-078 | wrong-requirement |

## Checklist Results

- Inputs name the exact authority, adapter selection, manifest, caller key,
  canonical arguments, durable history, observation authority, and limits.
- Outputs preserve refusal, incomplete, delivery-unknown, receipt, and observed
  effect as distinct states.
- Every pre-dispatch validation and persistence condition has an observable
  zero-call failure result.
- Exact replay, conflicting reuse, crash windows, expiry, reconciliation, and
  observation attribution have explicit verification scenarios.
- Task 004 supplies feature-local static hooks; Task 007 remains responsible for
  the repository-wide TC-009/TC-011 evidence and status.

## Verdict

PASS. No implementation-blocking base-review ambiguity remains.
