---
id: SR-13531
title: "Gap analysis — IR-699 IR-566 cohort PR 22"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #22 ac966501843bd16db0ec649d63a7c68ba655bfc2; Cargo.lock, tests/dependency_cohort.rs, spec/integration/, current metadata graph"
review_set: subset
---

## Summary

Planless, PR-scoped gap analysis of the dependency cohort invariant and its test evidence. The lock currently satisfies the ticket's measured graph properties, but the new test does not bind to a declared spec criterion.

## Verdict

**CONDITIONAL** — the new graph test is untraced in a repository whose integration tests carry `#[trace]` bindings, and the checked-in spec has no acceptance criterion for this lock invariant.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `composed_dependency_cohort_has_one_shared_type_identity` has no `#[trace]` tag and no current spec criterion that states the expected FCD/quire-rs member set or canonicalizer absence. The lock check can run green without appearing as backing evidence in the computed test matrix. Add a criterion for this composition invariant and bind this behavioral test to it. | tests/dependency_cohort.rs:8; spec/integration/ |

## Coverage

Plan completion: not assessed. The repository's `quire matrix --scope . --format json` returned engine metadata only, with no declared traceability model/matrix rows, so this review used the repository's existing `#[trace]` convention and a scoped spec grep as its fallback. The focused Rust test has not executed because the published CG dependency fails compilation against current RT/QSL. No production source changed in this PR; the changed graph is `Cargo.lock`.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | IT-014-SC-01 now states a cohort criterion and the test carries `#[trace("IT-014-SC-01")]`. The matrix engine still emits no rows because this repository has no declared traceability model; the source-level binding is present. |
