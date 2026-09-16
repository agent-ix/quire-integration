---
id: SR-041
title: "Scope and boundary review of Task 006 compatibility impact"
type: SpecReview
analysis: scope-boundary
scope: "FR-005; Task-006; Task-010; Task-011; QSpec FR-307/310/311/336/337"
review_set: all
---
# SR-041: Scope and boundary review of Task 006 compatibility impact

## Summary

The scope review assigns Task 006 only pure feature-local traversal, overlays and
accounting. Owner compatibility/evidence/vector contracts, real adapters, execution
dispositions and release authority remain outside that boundary.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Resolved: Task 006 owns explicit bounded comparison/graph/policy/manifest facade contracts and retains imported FR-307/310/336/337 fields without claiming those external owner wires; propagation/freshness remain non-authoritative overlays. | FR-005; Task-006 |
| FND-002 | high | Resolved: Task 010 owns guaranteed real-owner contract/pass-through tests, while Task 011 alone owns vector execution, pass/waiver and release decisions. | Task-010; IT-003; Task-011 |
| FND-003 | medium | Resolved: NFR-002 now explicitly constrains FR-005 so graph, evidence, decision and policy inputs cannot become source, evaluator, evidence or release authority. | NFR-002; FR-005 |
| FND-004 | medium | Resolved: Task 006 strict-reads exact FR-337 intake bytes and references them from local graph nodes; Task 010 owns real-owner pass-through and the facade cannot rewrite rights or limitations. | FR-005 Inputs; Task-010; IT-003 |

## Responsibility allocation

| Boundary | Owner | Class | Contract |
|---|---|---|---|
| Local impact traversal/overlays | Task 006 | core | FR-005 / TC-005 |
| Comparison correspondence | QSpec FR-307 owner | external guaranteed | Task 010 contract test |
| Evidence graph/intake | QSpec FR-310/FR-337 owners | external guaranteed | Task 010 contract test |
| Vector definition/execution | QSpec FR-336 / FR-311 + Task 011 | external/qualification | IT-003 then qualification ledger |
| Release decision | Task 011/human authority | external authority | no Task 006 promotion |

## Verdict

PASS. No owner contract or authority is duplicated inside Task 006.
