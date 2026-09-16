---
id: SR-020
title: "Risk and complexity review of Task 004 effect reconciliation"
type: SpecReview
analysis: risk-complexity
scope: "FR-003; NFR-001; NFR-002; IT-002; Task-004"
review_set: all
---
# SR-020: Risk and complexity review of Task 004 effect reconciliation

## Summary

Task 004 is high-risk because it crosses durable state, external effects, and
authority expiry. Its complexity is contained by one append-only state machine,
one atomic key reservation, opaque call permits, and at most one reconciliation
and one dispatch per coordinator invocation.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Mitigated: receipt loss after provider acceptance can cause duplicate business effects. Attempt-first persistence, reconcile-first recovery, and exact same-key/same-bytes retry guarantees bound the risk. | FR-003-AC-2; IT-002-SC-02..03 |
| FND-002 | high | Mitigated: a public raw dispatch trait could bypass authority and persistence. Task 004 now requires an opaque boundary-minted permit and Task 007 static inventory. | FR-003-AC-6; NFR-002-AC-1; TC-009; TC-011 |
| FND-003 | medium | Mitigated: concurrent first use of one key could create two attempts. The contract now requires atomic global reservation and canonical-byte conflict comparison. | FR-003-AC-4; IT-002-SC-08 |

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| duplicate external effect after crash | medium | high | durable attempt + reconciliation + manifest guarantee |
| expired authority reused for new effect | medium | high | logical-time recheck before every new dispatch |
| forged receipt/observation attribution | low | high | exact request/adapter/effect identity validation |
| journal race or stale writer | medium | high | atomic reserve and revision-checked append |
| unbounded history/arguments | medium | medium | explicit limits and checked conversions |

## Verdict

PASS. High-impact risks have early typed controls and direct negative evidence.
