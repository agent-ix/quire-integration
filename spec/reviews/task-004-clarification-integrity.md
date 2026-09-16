---
id: SR-017
title: "Integrity review of Task 004 effect reconciliation"
type: SpecReview
analysis: integrity
scope: "FR-003; NFR-001; NFR-002; IT-002; Task-004"
review_set: all
---
# SR-017: Integrity review of Task 004 effect reconciliation

## Summary

Completeness, consistency, atomicity, and testability were checked across the
Task 004 requirement, constraints, integration scenario, and plan task. The
clarified state machine is total over fresh, pending, open, delivered,
delivery-unknown, and observed states.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: key uniqueness and request comparison were stated separately, leaving concurrent conflicting submissions underspecified. FR-003 now requires one global atomic reservation and full canonical-byte comparison. | FR-003; IT-002-SC-08 | missing-requirement |
| FND-002 | medium | Resolved: observation matching implicitly depended on a retained receipt. FR-003 now permits an authoritative request-correlated observation to establish an effect even when the receipt was lost. | FR-003-AC-5 | missing-requirement |

## Traceability

| Obligation | Required State/Transition | Evidence Owner |
|---|---|---|
| FR-003-AC-1 | validated -> reserved -> attempt-prepared -> delivered | TC-003 |
| FR-003-AC-2 | open attempt -> reconcile -> receipt or typed uncertainty | TC-003 |
| FR-003-AC-3 | invalid authority -> refusal with zero store/adapter activity | TC-003 |
| FR-003-AC-4 | global key conflict -> refusal before attempt/dispatch | TC-003 |
| FR-003-AC-5 | qualified observation -> effect observed; receipt alone does not | TC-003 |
| FR-003-AC-6 / NFR-002-AC-1 | opaque permit and sole call site | TC-009 / TC-011 |

## Verdict

PASS. The slice is internally consistent and every normative transition is
observable without conflating delivery and business truth.
