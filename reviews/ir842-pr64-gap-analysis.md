---
id: SR-2026110414560069
title: "Gap analysis of IR-842 PR 64 U11 first-defect order"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #64; tests/tc187_u11_order.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Scoped gap analysis of the five newly claimed TC-187 admission rows. `quoin matrix --repo . --json` reports no coverage-matrix criteria because this module declares no `obligations:` source; the QSpec TC-187 and FR-142-AC-15/16 bindings were checked manually. No plan was selected.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the presence of real public QSL assembly evidence in this bounded admission slice. IT-019-SC-04 remains Planned overall, and this verdict gives no generated oracle agreement credit. The public-vector binding weakness is recorded in the code-review artifact.

## Coverage

The new tests carry `#[trace]` tags for QSpec TC-187 and FR-142-AC-15/16. They exercise public parse, forms and assembly with six first-refusal runs, asserting typed causes, retained values or edges, codes and spans. The dispatcher reads every public ID and fails closed on unknown IDs. The IT-019 Passed row names admission evidence only; the adjacent Planned row retains the remaining IDs and owner gaps. Plan completion: not assessed.
