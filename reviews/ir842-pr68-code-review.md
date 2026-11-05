---
id: SR-2026110414560080
title: "Code review of IR-842 PR 68 multidocument first refusals"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #68; tests/tc187_u11_multidoc_transitive.rs, tests/tc187_vector_dispatch.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: subset
---
## Summary

Reviewed frozen head fa4513db for U11q, U11x and U11z3 against both complete public TC-187 input and expected cells and QSpec FR-142-AC-17. Each test asserts a real public QSL parse, forms and assembly result, not a fixture result. No production code, copied fixture, Cargo pin or CI workflow changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. U11q reverses two distinct source documents and checks the first defect despite opposite offsets; U11x checks the owning D/K term without e and the earlier zero-scale e with it; U11z3 checks e before a reached cycle and both retained cycle edges without e. Each refusal checks the typed cause, exact catalog code and selected span.

## Coverage

The full public TC-187 cells are compared verbatim before any QSL observations. The QSL public assemble_units path currently returns a TargetCycle topology for U11q's self-target; the public TC-187 cell requires the definition-cycle code and retained self-edge, both checked here. A nearby QSL internal test expects QuantityCycle for a similar source, which is an upstream test/path discrepancy rather than a defect introduced by this PR. Focused coder tests passed 4/4, scoped Clippy, fmt and diff check passed; the lead owns the final gate.
