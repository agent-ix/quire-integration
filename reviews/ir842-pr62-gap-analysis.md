---
id: SR-2026110414560066
title: "Gap analysis of IR-842 PR 62 U11 local values"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #62; tests/tc187_u11_local_values.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Scoped gap analysis of the four newly claimed TC-187 admission rows. The local Quire module declares no `obligations:` source, so `quoin matrix --repo ... --json` returned no coverage-matrix criteria; trace bindings were checked manually against QSpec TC-187 and FR-142-AC-15/16. No plan was selected.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the presence of real public QSL admission evidence in this bounded slice. IT-019-SC-04 remains Planned overall; this verdict gives no generated agreement credit. The U11e vector-input oracle weakness is recorded in the code-review artifact.

## Coverage

The two new tests carry `#[trace]` tags for QSpec TC-187 and the owning FR-142 criteria. They execute public parse, forms and assembly; U11e's separate source runs assert typed NonIdentityRoot with scale 2/offset 0 and scale 1/offset 1, while U11g1/g2/g4 assert typed unanchored refusal and positive anchored maps. The dispatcher reads all public IDs and fails closed on unknown IDs. Plan completion: not assessed.
