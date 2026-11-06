---
id: SR-2026110414560125
title: "Gap analysis of IR-842 PR 81 TC-187 dispatcher"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #81; tests/tc187_vector_dispatch.rs and cited merged TC-187 evidence"
review_set: subset
---
## Summary

Compared the 70 public TC-187 IDs, dispatcher classifications, merged tests and IT-019/TC-106–TC-113 status rows. `quire matrix --scope . --format json` returned engine metadata but no obligations matrix for this integration module, so trace and coverage bindings were checked manually. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The dispatcher declares IR-842 as the remaining owner of U22/U25 original compound conversion and bridge agreement while IT-019 marks those axes Planned under IR-1348. Its route assertion would pass even though the dependency handoff is wrong. | tests/tc187_vector_dispatch.rs:148 |

## Verdict

CONDITIONAL for owner traceability. The 70-ID fail-closed enumeration and separate Planned generated axes otherwise match the merged evidence inspected. Update U22/U25 owner routing to IR-1348, then disposition this finding on the revised head.

## Coverage

Checked original-operation native comparisons for U15/U19/U20/U23/U24/U26, QSL value-only U12/U16–U18/U28/U29, partial U22/U25/U27, p8 static and model routes, and the distinct QSL-1738 U11g3 blocker. Source tests, merged IT-019 rows, TC-107–TC-113 and public QSpec cells were compared by row. The matrix engine exposed no obligation rows; no whole-ticket completion is claimed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised head df5cea0 routes U22/U25 to IR-1348 and makes that ticket the expected owner in the dispatcher assertion, matching IT-019. |
