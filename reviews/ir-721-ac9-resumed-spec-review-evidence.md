---
id: SR-47005
title: IR-721 AC9 PR47 resumed spec-review-evidence
type: SpecReview
analysis: evidence
scope: agent-ix/quire-integration; Cargo.toml, Cargo.lock, tests/monitor_selected_facts.rs,
  tests/support/mod.rs, spec/test-cases/TC-098-monitor-program-construction.md; FR357AC9
  owner context
review_set: subset
---

## Summary

PR47, head branch titan/monitor-selected-emission; ticket IR-721. Independent resumed review of the frozen candidate against origin/main. No applicable AssuranceProfile exists in spec/. This review supersedes the historical source-only AC9 qualification limit; those earlier artifacts remain unchanged as history. No repository edits or Cargo commands were performed by this reviewer. The lead owns the one final scoped gate.

## Verdict

PASS within the changed scope; zero defects. No whole-repository assurance or parent-ticket closure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Manual scoped evidence qualification, not a completed catalog advisor run. The required live advisor depends on forbidden new properties/coverage measurements, so it was not run for this invocation; no catalog-wide method-mismatch conclusion is asserted. The owning FR357AC9 already authors Test and explicitly permits opaque native equality success or the precise native missing-witness alternative. The observed focused integration test supports the former, with representable model rows checked separately. Producer correspondence and selected-reader result assertions are behavioral and can fail on changed retention, identities, ordering, type, selected equality or propagation. The test uses the supported producer adapter, not hand-authored IR equality rows.

Supplied focused-final.log records one passing test in the real path and identifies opaque native decision. clippy-fixed.log, fmt-final.log, docs-final.log and metadata.log record the author checks. They were independently read, not rerun. Historical source-only review artifacts reflect the earlier driver blocker and do not supply runtime acceptance; these resumed results supersede that qualification limit. The parent ticket and broad AC1-4 stay open/Planned. The one final scoped gate remains with the lead; no Kani, feature sweep, global count or extra runtime path was claimed.

## Examined units

```yaml
scope:
- id: FR-357-AC-9
  path: agent-ix/quire-contract-ir/spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: 'PLANNED/UNRUN (cross-repo `quire-integration` test using the available
    IR selected-fact API). A `quire-integration` test obtains a real final `PlanRoot.emission`
    through QSL, selects admitted IDs from its available model declaration/field rows
    and one equality application/occurrence pair, translates QSL equality rows without
    changing their values, and exercises the public IR selected-fact read on those
    explicit IDs. It verifies successful joins for representable selected model facts
    and an opaque FR-353 equality decision or its native typed missing-witness result
    for the selected equality. It '
- id: TC-098-selected-fact-control
  path: spec/test-cases/TC-098-monitor-program-construction.md
  role: examined
  excerpt: 'Run `monitor_selected_facts` separately for FR-357-AC-9. Obtain a real
    final

    QSL `PlanRoot.emission`, admit its bytes strictly, and select model declaration,

    field and equality application/occurrence IDs from that same emission.'
- id: FR-357-AC-1
  path: agent-ix/quire-contract-ir/spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly
    admitted generated monitor root with four entries yields one complete selected
    type/declaration closure; records, tuples, collections, unions, enum cases, object
    fields and source units retain original identities, producer order, presence,
    shape and member/payload types sufficient to build the canonical `TypeEnvironment`.
    Shared references do not duplicate declarations.
- id: FR-357-AC-2
  path: agent-ix/quire-contract-ir/spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing,
    foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved
    reachable declaration, unrepresentable type or native nominal/Exact refusal returns
    its typed cause and first locus, with no partial view or invented identity.
- id: FR-357-AC-3
  path: agent-ix/quire-contract-ir/spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes
    and unsupported declarations visits only its reachable nodes/edges and charged
    identity/value material using an iterative worklist. Equal inputs and allowances
    give equal complete results or first work stops; one below the required work refuses
    without a partial view.
- id: FR-357-AC-4
  path: agent-ix/quire-contract-ir/spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded
    IR projection over admitted package facts and original QSL witnesses; it finds
    no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse
    or witness fabrication.
```
