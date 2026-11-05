---
id: SR-2026110414560060
title: "Gap analysis of IR-842 PR 60 U11 topology"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #60; tests/tc187_u11_topology.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Scoped gap analysis of the five newly claimed TC-187 admission rows against public-vector dispatch and real QSL assembly tests. The local Quire module declares no `obligations:` source for a computed coverage matrix, so trace bindings were checked manually against QSpec TC-187 and FR-142-AC-15.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this bounded slice. IT-019-SC-04 remains Planned overall; this verdict gives no credit to U11g3, U01, other TC-187 rows, or generated agreement.

## Coverage

The five refusal tests exercise public parse, forms and assembly, and assert the actual typed faults, retained identities, catalog codes and source spans. `#[trace]` binds the test to QSpec TC-187 and FR-142-AC-15. The generic owner-ticket issue in the dispatcher is recorded in the code-review artifact and is outside the evidence-presence question here. Plan completion: not assessed.
