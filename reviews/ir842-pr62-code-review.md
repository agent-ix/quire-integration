---
id: SR-2026110414560065
title: "Code review of IR-842 PR 62 U11 local values"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #62; tests/tc187_u11_local_values.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed the frozen PR #62 diff and the public QSpec TC-187 U11e/g1/g2/g4 rows. The real QSL assembly tests assert exact typed causes, retained scale/offset or dimension/unit names, catalog codes, spans and anchored dimension maps. No production API, unsafe code, vendored artifact, pin or CI gate changes appear in this diff. No applicable AssuranceProfile was found under `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The U11e public-vector check matches only `unit m : L` and `invalid-value`, so either of the two authored source forms can change or disappear in QSpec while the two hardcoded QSL runs still pass and the test still claims both public forms. Bind the vector input to both `m = 2` and `m = 1 + 1` forms before crediting both. | tests/tc187_u11_local_values.rs:151; IT-019-SC-04 |

## Verdict

CONDITIONAL. The actual QSL value and refusal assertions are strong, but U11e's link to both public authored inputs is too weak for the two-form coverage claim.

## Coverage

Examined QSpec TC-187 U11e/g1/g2/g4 and FR-142-AC-15/16, IT-019-SC-04 and the changed tests/dispatcher. The U11g1/g2/g4 original sources refuse at unanchored `L`; distinct anchored controls assert `L^2`, `L^3`, `L^4` through public UnitGraph, without giving generated-CG credit. The dispatcher fails closed on unknown public IDs. `quire validate` on IT-019, `git diff --check`, and `cargo fmt --check --all` passed. Coder reported focused Cargo tests 3/3 and scoped Clippy pass; no Cargo build was repeated during review.
