---
id: SR-2026110414560084
title: "Gap analysis of IR-842 PR 69 imported base-dimension order"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #69; tests/tc187_u11_imported_base.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Compared the three TC-187 public cells and FR-142-AC-16/18 with the tests and changed IT-019 status. The computed Quire matrix has no obligation rows because the integration module declares no obligations source, so bindings and assertions were inspected manually.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-142-AC-16/18 mention-order clauses for U11z1/z2 are tagged and marked Passed, but no assertion can fail when the valid P.Area or Q.T turn is visited in the opposite order before P.L. | tests/tc187_u11_imported_base.rs:168 |

## Verdict

CONDITIONAL for the imported-dimension ordering clauses. The observed P.L refusal, retained carrier, code and owner span are backed; the broader SC-04 generated-operation agreement remains Planned.

## Coverage

Plan completion: not assessed. U11y/z1/z2 have full exact public vector identity/expected-cell checks and real QSL results. U11z1 and U11z2 need competing-defect controls to establish the stated order. U11g3 remains Planned under QSL-1738, U01 under QSL-1737 and IR-1345, and other public rows under IR-842. No generated-CG equality credit is claimed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised head 3fe42749 adds real selected-model adverse controls for both ordering clauses. U11z1 selects P.Area's TwoRoots before P.L; U11z2 selects P.L before equally unitless Q.T. Focused tests pass. |
