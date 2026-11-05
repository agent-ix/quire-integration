---
id: SR-47003
title: IR-721 AC9 PR47 resumed spec-review-base
type: SpecReview
analysis: base
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

Applied the base checklist to the changed TC098 description, procedure, expected results and added FR357 relationship. Existing IT018 ownership is preserved, while AC9 is described as a separate explicit-ID IR selected-fact boundary. The procedure identifies its original record source and QSL-owned monitor fixture, exact row preservation and public adapter; expected results match the observed model join plus opaque native equality decision. The future missing-witness branch is explicitly unobserved, avoiding false evidence credit. No complete closure/RT readiness claim or whole-ticket completion is introduced. No requirement statement, verification method, AC ID, matrix or domain definition is altered; EARS, integrity, failure-domain and object analyses add no applicable changed material. No new file-tracking hashes/pins, copied fixtures or private workstation paths occur in the reviewed diff outside the normal Cargo lock representation. Explicit TC structure validation passed in supplied evidence.

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
