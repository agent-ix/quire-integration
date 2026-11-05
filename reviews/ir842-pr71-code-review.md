---
id: SR-2026110414560089
title: "Code review of IR-842 PR 71 quantity power values"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #71; tests/tc187_u13_u21_power.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed frozen head 1b8b8c2 for the complete public TC-187 U13 and U21 cells and FR-142-AC-6/7. The tests parse authored QSL source, check a package or expression through public QSL APIs, and assert typed value outcomes and charge order. No production API, dependency, copied fixture or workflow changes occur.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. U13 asserts the exact IntegerBits denial with consumed 2, denied charge 36893488147419103232 and UnitRationalArithmetic point before a value. U21 asserts dimensionless one with the empty compound unit, ordered read/power/target/retention events, a distinct Undefined negative power after one read, and the work-zero denial before read. The dispatcher keeps generated comparison explicitly Planned.

## Coverage

Examined QSpec TC-187 U13 and U21 and FR-142-AC-6/7 against the source, checks and value assertions. The U21 expression is cut from an authored source equality and checked in Kernel mode against an assembled unit package; the function envelope is not used, so no call charge is claimed. Exact full input and expected cells are bound by `bind_vector`. The changed Rust is deterministic, uses public APIs, has trace tags, and adds no production code or unsafe path. Independent focused test result is recorded separately; no full gate was run in review.
