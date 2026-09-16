---
id: SR-021
title: "Scope and boundary review of Task 004 effect reconciliation"
type: SpecReview
analysis: scope-boundary
scope: "FR-003; NFR-002; IT-002; Task-004"
review_set: all
---
# SR-021: Scope and boundary review of Task 004 effect reconciliation

## Summary

The facade owns authorization checks, canonical request identity, durable
coordination, and attribution. Adapter semantics, business-effect truth, domain
recommendations, and release authority remain outside this crate.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: the existing generic adapter capability exposed raw dispatch outside the sole effect boundary. Task 004 requires a boundary-minted opaque permit and no other public construction path. | FR-003-AC-6; NFR-002-AC-1; Task-004 | wrong-requirement |
| FND-002 | medium | Resolved: observation correlation was underspecified when no receipt survived. QObs remains the authority and may correlate by request identity plus external identity when known. | FR-003-AC-5 | missing-requirement |

## Responsibility Allocation

| Boundary | Owner | Guarantee |
|---|---|---|
| recommendation and formal decision | domain producer / attributed actor | immutable identities and approved parameters |
| key reservation and attempt journal | quire-integration effect coordinator/store port | atomicity, ordering, immutable lineage |
| request canonicalization | selected adapter contract + facade validator | bounded canonical bytes; no adapter key substitution |
| transport and reconciliation | selected adapter | attributed receipt and declared duplicate semantics |
| business-effect observation | lock-admitted QObs authority | independent effect truth |
| complete call-graph proof | Task 007 | repository-wide TC-009/TC-011 evidence |

## Out-of-Scope Confirmation

Task 004 does not implement a production payment provider, infer domain success
from delivery, create human authority, replace QObs, or claim the later real HTTP
and process-restart integration evidence.

## Verdict

PASS. Every effect responsibility has one owner and no authority is duplicated.
