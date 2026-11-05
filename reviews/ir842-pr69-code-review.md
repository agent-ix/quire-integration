---
id: SR-2026110414560083
title: "Code review of IR-842 PR 69 imported base-dimension order"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #69; tests/tc187_u11_imported_base.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed frozen head b6ddd843 for TC-187 U11y/z1/z2 against the complete public cells and QSpec FR-142-AC-16/18. The tests use public QSL parse, selected-model intake and assembly, and assert typed refusal, carrier, code and span. No production API, fixture copy, Cargo pin or workflow changes occur.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | U11z1 and U11z2 tests cannot detect reversal of the claimed imported-dimension turn order: the intervening P.Area and Q.T turns are valid, so either order yields the asserted P.L refusal. Add adverse controls with competing first defects, or limit the claimed order coverage. | tests/tc187_u11_imported_base.rs:168 |

## Verdict

CONDITIONAL. The final typed refusals are real, but the two ordering assertions named by the test and IT-019 do not yet have a failing oracle for reversed turns.

## Coverage

Examined TC-187 U11y/z1/z2 and FR-142-AC-16/18. U11y correctly observes the imported P.L refusal before a local Area declaration. For U11z1, a walker that checks P.L before P.Area still returns P.L/m2; for U11z2, a walker that checks Q.T before P.L also returns P.L/x because Q.T is valid. The QSL source and model documents are supplied to the real public intake path, so this is a test-oracle gap rather than a test double. Coder focused tests passed 3/3 and dispatcher 1/1; scoped Clippy, fmt and IT-019 validation passed. The lead owns the final gate.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised head 3fe42749 adds a public QSL U11z1 P.Area two-root competitor and a U11z2 dual-unitless-base competitor. Reversing the imported turns now changes the first typed refusal, and both focused tests and scoped Clippy pass. |
