---
id: SR-2026110414560087
title: "Gap analysis of IR-842 PR 70 quantity type checks"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #70; tests/tc187_u30_u31.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Compared QSpec FR-142-AC-11/12 and the complete TC-187 U30/U31 cells with their tagged integration tests. `quire matrix --scope .` reports no obligations source for this integration module, so the acceptance bindings and code were checked manually.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The U30 test is traced to FR-142-AC-11 and marks its complete public cell Passed, but its `Quantity<T>` branch uses a record rather than the required dimension; dimension-name rejection remains untested. | tests/tc187_u30_u31.rs:158 |

## Verdict

CONDITIONAL. The U30 dimension alternative lacks evidence; the other U30 branches and U31 are backed by genuine QSL results.

## Coverage

Plan completion: not assessed. U30/U31 bind the exact complete public input and expected cells before invoking real QSL source intake. The `record T` substitution can miss a dimension/unit namespace bug. U11g3 stays Planned on QSL-1738, U01 on QSL-1737 and IR-1345, and remaining public IDs on IR-842. Generated operation agreement remains Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 8ca9011 the U30 common source declares dimension T, and the `Quantity<T>` alternative supplies no record replacement. Real QSL assembly returns `UnresolvedTypeName` at T with `missing_declaration/missing-name`; focused tests pass. |
