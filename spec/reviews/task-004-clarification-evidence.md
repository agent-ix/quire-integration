---
id: SR-019
title: "Evidence-method review of Task 004 effect reconciliation"
type: SpecReview
analysis: evidence
scope: "FR-003; NFR-002-AC-1; IT-002; TC-003; TC-009; TC-011"
review_set: all
---
# SR-019: Evidence-method review of Task 004 effect reconciliation

## Summary

Native Quoin advice was compared with the authored methods. Example/unit advice
is useful within the suites but cannot replace fault injection, generated key
mutation, real-boundary integration, or static absence evidence.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | Confirmed by judgment: Quoin recommends example/unit supplements for FR-003-AC-3 through AC-6, while the authored negative, property, integration, and architecture methods directly match the quantified trust-boundary claims. | FR-003-AC-3..AC-6 |

## Method Decisions

- Ordered unit/fault tests prove persist-before-dispatch and the feature-local
  request, attempt, transport, receipt, and restart fault windows without
  requiring unsafe process termination in the library suite.
- Property tests mutate decision, authority, adapter-selection, operation, and
  argument identities independently under a reused caller key.
- Integration testing remains necessary for durable restart and real local HTTP
  behavior; Task 004 does not claim Task 009 evidence early.
- Compile-fail visibility checks plus source/call-site inventory are the direct
  evidence for absence of unauthorized dispatch paths.
- Exact/one-over tests cover every Task 004 structural limit and assert zero
  transport activity on pre-call refusal. Task 007 still owns shared
  producer-invocation/duration metering (TC-007/TC-010), and Task 009 owns real
  store/HTTP restart evidence plus trusted registry wiring.

## Deterministic Evidence

`quoin advise --repo . --mismatch-only` reports the repository's existing 36 of
44 recommendation mismatches, zero uncatalogued methods, and one inconclusive
obligation. No Task 004 method was changed solely to satisfy advisory heuristics.

## Verdict

PASS. Each Task 004 claim has a proportionate evidence method and honest owner.
