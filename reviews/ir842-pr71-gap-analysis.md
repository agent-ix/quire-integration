---
id: SR-2026110414560090
title: "Gap analysis of IR-842 PR 71 quantity power values"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #71; tests/tc187_u13_u21_power.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Compared complete public TC-187 U13/U21 cells and QSpec FR-142-AC-6/7 with the tagged integration tests and changed IT-019 row. `quire matrix --scope .` reports no obligations source for this integration module, so the bindings and assertions were inspected manually.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the claimed QSL value authority. The tests can fail on wrong U13 charge kind, limit, consumed amount, exact denied amount or charge point, and on wrong U21 value, unit, order, undefined reason or read-denial boundary. Neither test claims generated execution or native comparison.

## Coverage

Examined the U13 and U21 public vector identity, full authored input and expected cells; FR-142-AC-6/7; the public source, checked-package and checked-expression QSL routes; and the dispatcher. Other TC-187 public IDs remain Planned in IT-019 and dispatcher. U11g3 remains Planned on QSL-1738. The generated quantity-power comparison remains Planned under IR-842.
