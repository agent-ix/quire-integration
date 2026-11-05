---
id: SR-20261104130004
title: "IR-721 AC9 PR 47 code-review source review"
type: SpecReview
analysis: code-review
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

The Rust lane inspected the complete new test, direct qsl-analyze dev dependency and lock delta, existing fixture locator, and resolved producer/reader APIs. The fixture is read by reference from Cargo-resolved QSL; no fixture, producer DTO or evaluator is copied. The adapter exhaustively maps all seven producer witness variants, both operand/unit roles and all four dimension roles. Every application identity, checked node, occurrence, source-map entry, structural key/node, optional owner/type and variant payload is copied unchanged. No identity is reconstructed. Final PlanRoot.emission supplies bytes and every row slice; evidence adds the same original model document. Strict admission precedes all selected IDs. Unique versionNumber selection retains record and original field identity. Declaration and field row equality, field count/name/order/Required/bounded Int type are asserted. Native and selected reads receive the same admitted package and same-emission rows. Success compares opaque decisions; failure permits only UnavailableEqualityWitness, retains requested application, native locus/kind/materialization/attempted work, occurrence and native cause, and requires Err with no partial selected view. Both exhaustive branches assert; no conditional skip or implementation-derived semantic equality oracle exists. Work allowances are finite. No public production API, unsafe/concurrency/wire implementation or CI gate is changed. Existing formatter warnings are disclosed in the supplied log. No source defect was found; compilation of this test and actual branch outcome remain unobserved.

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
- id: selected-fact-test
  path: tests/monitor_selected_facts.rs
  role: examined
  excerpt: '#[trace("TC-098", "FR-357-AC-9")]'
- id: producer-dependency
  path: Cargo.toml
  role: examined
  excerpt: qsl-analyze = { git = "https://github.com/agent-ix/quire-spec-language",
    branch = "main" }
```
