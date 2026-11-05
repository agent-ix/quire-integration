---
id: SR-47002
title: IR-721 AC9 PR47 resumed gap-analysis
type: SpecReview
analysis: gap-analysis
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

Manual changed-scope acceptance-criteria-to-test review only. User excluded fresh repository matrix/coverage/trace/properties scans and Cargo reruns, so this is not a complete Quoin repository audit or a fresh coverage count. Plan completion: not assessed.

The trace attribute binds FR357AC9 and the focused TC098 contribution to a real executed integration path. Model-only success asserts representable original record/field joins, and the second selected read preserves the actual opaque native equality decision from the same emission and row set. The focused log records one passing test, zero failed/ignored/filtered, and the opaque native decision message. The alternate missing-witness assertions are source-reviewed only and receive no executed-path credit. The fixture cannot silently pass with missing model/equality rows: nonempty and uniqueness checks precede selection, strict admission and public selected reads must succeed, and every unexpected native error fails.

No four-entry monitor closure traversal, complete generated-source closure, TypeEnvironment, RT ProgramParts or temporal readiness is asserted. Full AC1-4 remain explicitly outside the test and IR721 remains open. Existing IT018 monitor construction controls are neither rerun nor newly credited. No hollow source scan or local dummy producer replaces these boundaries.

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
