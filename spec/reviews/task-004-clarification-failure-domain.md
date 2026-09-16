---
id: SR-016
title: "Failure-domain review of Task 004 effect reconciliation"
type: SpecReview
analysis: failure-domain
scope: "FR-003; IT-002; Task-004"
review_set: all
---
# SR-016: Failure-domain review of Task 004 effect reconciliation

## Summary

The review modeled every failure boundary between validation, atomic key
reservation, attempt persistence, transport, receipt persistence,
reconciliation, and observation linking. Open attempts are treated as uncertain
regardless of whether transport was actually reached.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: a crash after attempt persistence but before transport is durably indistinguishable from post-acceptance failure. FR-003 and IT-002 now require reconciliation before any further call. | FR-003; IT-002-SC-02 | missing-requirement |
| FND-002 | high | Resolved: conclusive non-delivery, inconclusive reconciliation, expired authority, and absent duplicate guarantees previously shared an ambiguous retry outcome. They now have distinct no-call rules. | FR-003; IT-002-SC-03; IT-002-SC-07 | missing-requirement |

## Domain Results

- Atomic reserve-or-load prevents two fresh coordinators from owning one key.
- Request-persistence failure and attempt-persistence failure occur before any
  transport call; post-call receipt-persistence failure retains an open attempt.
- Reconciliation may run after grant expiry, but a new dispatch may not.
- Inconclusive reconciliation without the exact manifest guarantee produces
  `delivery-unknown`; conclusive non-delivery without safe retry produces a
  separate typed incomplete/refusal state.
- Receipt attribution and business-effect observation remain separate failure
  domains, including when the receipt was lost.

## Verdict

PASS. Each crash and uncertainty state has a conservative, typed transition.
