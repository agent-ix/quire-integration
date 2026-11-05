---
id: SR-2026110414560059
title: "Code review of IR-842 PR 60 U11 topology"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #60; tests/tc187_u11_topology.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed the frozen PR #60 diff, the public QSL assembly controls for U11b/c/d/f/l, and the TC-187 dispatcher. No vendoring, new pin, unsafe code, production API, or CI gate change appears in this diff. The five refusal tests assert typed cause, retained values and source spans.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Planned dispatch for U01 and U11g3 reports generic IR-842 ownership and a generic missing path, despite the known QSL-1737 and QSL-1738 producer blockers; the per-row ticket/path claim can pass while the owner record is wrong. | tests/tc187_vector_dispatch.rs:35; IT-019-SC-04 |

## Verdict

CONDITIONAL. The five new admission controls are real and value asserting, but the Planned dispatch must name the exact missing upstream path and open owner ticket for U01 and U11g3. GitHub reported PR #60 as CONFLICTING after main advanced to f0aed0a; IT-019 overlaps and needs coder rebase before the final gate.

## Coverage

Examined QSpec TC-187 U11b/c/d/f/l and FR-142-AC-15; verified test-source selection and assertion strength. U11g3 remains uncredited under QSL-1738. U01 remains uncredited under QSL-1737 and IR-1345. Scope is this PR diff only. `quire validate` on IT-019 and `git diff --check` passed. Coder's focused Cargo targets passed 1/1 each; no Cargo target was rerun in this review.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | fixed: U01 now names QSL-1737 and IR-1345 with the precise producer and lowerer gap; U11g3 names QSL-1738 and its empty-map defect. |
