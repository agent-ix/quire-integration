---
id: SR-2026110414560086
title: "Code review of IR-842 PR 70 quantity type checks"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #70; tests/tc187_u30_u31.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed frozen head cf2d827 for the complete TC-187 U30/U31 public cells and QSpec FR-142-AC-11/12. The new tests use public QSL parse, assembly and check, with exact source spans and typed outcomes. No production API, dependency, copied fixture or workflow changes occur.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | U30's `Quantity<T>` case supplies `record T` while the public cell specifies dimension `T`; the assertion can pass if QSL incorrectly accepts a dimension in the unit position. Exercise the actual dimension-name miss. | tests/tc187_u30_u31.rs:158 |

## Verdict

CONDITIONAL. U30's dimension-name refusal needs a failing oracle for the authored dimension case. The other U30 alternatives and both U31 branches exercise real QSL behavior.

## Coverage

Examined QSpec FR-142-AC-11/12 and TC-187 U30/U31. `TIME_UNITS` declares `dimension Time`, then the U30 missing-name branch adds `record T`; those are different namespace cases. `f(3 ms)`, `f(3 s)`, `Quantity<nothing>`, bare `d: ms`, returning `d`, and converted-result refusal have real typed checks and exact relevant locations. The dispatcher remains fail-closed. Independent focused tests pass 3/3; formatting, IT-019 validation and diff check pass. No full gate was run in review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised head 8ca9011 changes the real public source to `dimension T; unit s : T; unit ms : T` and removes the substitute `record T`; the `Quantity<T>` assembly refusal now checks typed missing-name and the exact T span. Focused tests pass 2+1, and scoped Clippy passes. |
