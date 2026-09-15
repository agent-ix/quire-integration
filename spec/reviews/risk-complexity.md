---
id: SR-006
title: "Risk and complexity review of complete V1 integration"
type: SpecReview
analysis: risk-complexity
scope: "StR-001; FR-001..FR-006; NFR-001..NFR-002"
review_set: all
---
# SR-006: Risk and complexity review of complete V1 integration

## Summary

The highest risks are cross-repository compatibility, crash-safe external
effects, authority containment and complete 176-row qualification. Each high
risk has an early contract/property/integration mitigation and bounded fallback.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-008 | high | Mitigated: a crash between external acceptance and local receipt persistence could duplicate an effect; FR-003/IT-002 now require reconciliation or an adapter idempotency guarantee and preserve `delivery-unknown`. | FR-003; IT-002 |
| FND-009 | high | Mitigated: one shared assumption can make all producer results look compatible; FR-001/FR-002 preserve exact contracts and independent dispositions under generated and reference-workflow tests. | FR-001; FR-002; IT-001 |
| FND-010 | high | Mitigated: a large multi-lane qualification can hide unavailable cells; FR-006 derives one state per required inventory cell and prohibits promotion of every non-pass state. | FR-006; IT-004 |

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|---|---|---|---|---|
| StR-001 | High | Medium | Trust claim spans repositories, effects, tools and evidence | Four independent end-to-end demonstrations and typed non-success states |
| FR-001 | Medium | Medium | Many identity domains and evolving producer contracts | Exact pins, strict readers, mutation/property tests |
| FR-002 | High | Medium | Concurrent cross-producer composition and partial failure | Canonical ordering, item-local outcomes, three reference workflows |
| FR-003 | High | Medium | Durable coordination, authorization and external side effects | Persist-first protocol, reconciliation/idempotency contract, crash injection |
| FR-004 | Medium | Low | Bounded graph traversal and heterogeneous consumers | Exact frontier accounting and generated determinism properties |
| FR-005 | Medium | Medium | Typed dependency impact and behavioral compatibility evidence | Explicit paths, unknown state, regression-vector properties |
| FR-006 | High | Medium | 176-row cross-lane campaign and unavailable intake surfaces | Inventory-derived ledger, exact lock, no promotion, scoped human decision |
| NFR-001 | Medium | Low | Resource/duration containment across all operations | Preselected budgets, exact/one-over properties and conversion audit |
| NFR-002 | High | Low | Security boundary between evaluation and effects | Deny-by-default typed capabilities and static call-graph audit |

## Top Hazards

1. FR-003 uncertain external delivery across a process crash.
2. FR-006 false completeness from a missing or stale qualification vector.
3. NFR-002 accidental adapter capability leakage into an evaluator.
4. FR-002 shared-prerequisite contradiction erasing valid sibling evidence.
5. FR-001 version drift disguised as structural compatibility.

## Failure-Domain Gaps

See SR-002. Its sole high-severity gap is resolved in the reviewed revision.

## Verdict

PASS. High-risk, low-volatility boundaries are front-loaded with contract and
fault-injection evidence; volatile producer selections stay behind exact locks.
