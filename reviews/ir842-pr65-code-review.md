---
id: SR-2026110414560071
title: "Code review of IR-842 PR 65 imported unit topology"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #65; tests/tc187_u11_imported_topology.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed frozen head 5367c9c for public TC-187 U11s/t/u and FR-142-AC-18. The tests use public QSL parse, forms, model intake and assembly, with the authored spine-model document read from the QSL checkout and extended only in memory. Exact typed first causes, retained evidence, catalog codes and source spans are asserted. The dispatcher fails closed on unknown IDs. No production API, copied fixture, pin or generated-CG agreement claim is added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public vector checks bind U11s/t/u only to generic fragments (`unit a`, `unit b`, `m2`, `unit z`) and one catalog suffix each. A public row could lose its imported dimension and root, competing local topology, or required retained identity while these hardcoded QSL sources still pass and IT-019 still credits the public rows. Bind each vector's distinctive imported-model and local-defect input and its full first expected refusal to the tests. | tests/tc187_u11_imported_topology.rs:104,140,181; IT-019-SC-04 |

## Verdict

CONDITIONAL. The QSL admission assertions are strong, but their connection to the public vector source and first expected outcome is insufficient for the three-row Passed claim.

## Coverage

Examined TC-187 U11s/t/u, FR-142-AC-18, IT-019-SC-04 and its changed Passed/Planned rows. U11s checks both local cycle edges and the model-selection source span; U11t checks distinct imported dimension/root keys and the local root; U11u checks exact zero scale and local name. All assert catalog codes. The dispatcher recognizes only enumerated IDs and keeps remaining rows Planned. `git diff --check` and `quire validate` passed. The PR reports four focused tests and scoped Clippy passing; no aggregate build was run in this review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at 2d1be13e: each U11s/t/u test compares its public row's full authored input and expected first refusal exactly; U11r's imported-model source is also checked before the t/u references. `tests/tc187_u11_imported_topology.rs:23-46,113-208`. |
