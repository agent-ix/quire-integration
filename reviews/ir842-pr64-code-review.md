---
id: SR-2026110414560068
title: "Code review of IR-842 PR 64 U11 first-defect order"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #64; tests/tc187_u11_order.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed the frozen PR #64 Rust diff against public QSpec TC-187 U11h/i/j/k/m and FR-142-AC-15/16. The new tests parse genuine QSL source, build forms, and assemble a package, asserting the first typed refusal, retained identity or values, catalog code and source span. The dispatcher fails closed on unknown public IDs. U01 and U11g3 remain Planned with named owners. No production API, unsafe code, vendored artifact, pin or CI gate changes appear. No applicable AssuranceProfile was found under `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public vector checks bind U11h/i/j/k/m only by generic names such as `unit z`, `unit a`, `m_alt`, `u_self`, or `dimension Area`; U11m checks only `missing-name`. A published vector could change its competing defect or remove U11m's first cycle while the hardcoded QSL sources still pass and IT-019 still credits the public rows. Bind each row's distinguishing authored defect and order fragments and both U11m alternatives and expected outcomes to the public vector. | tests/tc187_u11_order.rs:93,113,134,152,170; IT-019-SC-04 |

## Verdict

CONDITIONAL. The real QSL refusal assertions are strong, but the link from their hardcoded source choices to the public vector inputs and expectations is too weak for the five-row coverage claim.

## Coverage

Examined QSpec TC-187 U11h/i/j/k/m and FR-142-AC-15/16; IT-019-SC-04 and its Passed/Planned rows; the new Rust tests and public dispatcher. U11h asserts zero scale before unknown target, U11i an earlier unknown target before a later zero scale, U11j two roots before self-cycle, U11k the earlier dimension's cycle before the later zero scale, and U11m both dimension orders. The U01 Planned record reflects IR-1345's merged lowerer with remaining QSL-1737 emission and IR-1345 AC4 agreement; U11g3 names QSL-1738. `cargo fmt --all -- --check`, `quire validate` on IT-019 and `git diff --check` passed. Coder reported focused Cargo tests 6/6 plus dispatcher 1/1 and scoped Clippy passing on the refreshed QSL lock; no Cargo build was repeated during review.
