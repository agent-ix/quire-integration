---
id: SR-20261104130006
title: "IR-721 AC9 PR 47 spec-review/base source review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; Cargo.toml, Cargo.lock, tests/monitor_selected_facts.rs, tests/support/mod.rs, spec/test-cases/TC-098-monitor-program-construction.md; FR-357-AC-9 context in agent-ix/quire-contract-ir"
review_set: subset
---

## Summary

PR 47, head branch titan/monitor-selected-emission; ticket IR-721. Source-only review against integration origin/main. No applicable AssuranceProfile exists in integration spec/. No source/spec/branch edits. No complete closure, TypeEnvironment, ProgramParts, temporal readiness or AC1-4 acceptance is claimed; IR-721 must remain open.

## Verdict

**PASS (source subset only)** — no source defects found. Qualification/mergeability remains BLOCKED, independently of the clean source verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed the changed TC-098 prose and its unchanged IT-018 boundary against actual IR origin/main FR-357-AC-9. The separate monitor_selected_facts target and explicit selection contract match the new test's intended assertions. The earlier producer/reader refusal statement belongs to the preceding IT-018 construction control; the separate selected-fact paragraph expressly permits only the native typed unavailable-witness outcome. The control is expressly Planned/Unrun, excludes four-entry traversal/full generated closure/TypeEnvironment/RT ProgramParts, and leaves AC1-4 outside scope. No new requirement statement, duplicate authority, pin, tracking ledger, fabricated witness or status upgrade is introduced. IDs and added ix URI are structurally consistent by inspection. Only explicit review artifact structural validation is performed; no global link/matrix measurement is claimed.

## Qualification Limit

Runtime qualification and mergeability BLOCKED: existing focused log stops before test execution at quire-driver src/refusal.rs:758, whose exhaustive CheckedPackageRefusalCause match omits UnitMismatch. Actual equality outcome is UNOBSERVED. Existing fmt/docs/locked-metadata success grants no runtime acceptance. No Cargo, Clippy, test, matrix, coverage, trace or property measurement was run by this reviewer. The lead owns the eventual focused execution and final scoped head gate.

## Examined Scope

```yaml
scope:
- id: FR-357-AC-9
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
    (agent-ix/quire-contract-ir origin/main)
  role: examined
  excerpt: PLANNED/UNRUN (cross-repo `quire-integration` test using the available
    IR selected-fact API). A `quire-integration` test obtains a real final `PlanRoot.emission`
    through QSL, selects admitted IDs from its available model declaration/field rows
    and one equality application/occurrence pair, translates QSL equality rows without
    changing their values, and exercises the public IR selected-fact read on those
    explicit IDs. It verifies successful joins for representable selected model facts
    and an opaque FR-353 equality decision or its native typed missing-witness result
    for the selected equ
- id: TC-098
  path: spec/test-cases/TC-098-monitor-program-construction.md
  role: examined
  excerpt: The selected-fact control remains Planned/Unrun until its focused target
    passes.
```
