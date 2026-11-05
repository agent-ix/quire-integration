---
id: SR-47001
title: IR-721 AC9 PR47 resumed code-review
type: SpecReview
analysis: code-review
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

Rust lane included. Changed test uses the real public emit_plan_root, strict read_checked_package and project_selected_facts, with no private enum paths, duplicate evaluator or fabricated semantic keys. Original data authored in selected_record_document is admitted through package_input alongside the separate QSL-owned ConfigVersion fixture located using the existing Cargo-metadata helper. It is source data, not a retained-row fixture or copied external document. The real producer emits the record/field IDs; both document byte buffers are supplied as reader evidence. The authored function makes the record reachable, while the temporal subject remains ConfigVersion. Generated queries require alias v. Reader limits are explicit finite amounts suitable for the measured generated package, not ambient or unlimited defaults.

The model-only read independently requires the exact selected producer declaration and field row, count name, order zero, Required presence and Integer type. The original field lookup requires uniqueness. A separate read selects one actual application/occurrence from the same final emission. Its opaque equality decision must equal the native FR353 read; no semantic equality decision is recomputed in test code. Success keeps model rows unchanged and returns exactly one selected equality. The alternate error branch accepts only UnavailableEqualityWitness and checks native application, witness, locus, occurrence and nested materialization/work fields; this branch is precise but was not executed in the observed successful run.

The unchanged-row handoff uses QSL's existing public Emission::equality_witness adapter. Source inspection of emit/equality_reader.rs confirms the closed producer mapping preserves all variant payloads, ordinals and roles and intentionally retains final selected structural nodes. Consumer assertions independently compare application source-map/key/location, fact correspondence/key/owner/type, and source-unit rows back to this final emission. No copied witness translator remains. Public outcomes and exact producer rows are the integration oracle; they do not assert a second domain authority.

The test is deterministic, bounded and single-threaded apart from existing library internals. No unsafe, async task, external daemon or mutable shared checkout state is introduced. SPDX matches the repository. Existing Cargo fixture lookup is reused. qsl-analyze is dev-only, branch-main like its sibling crates, and lock resolution unifies the composed first-party crates; no pin or production QSL dependency is added to IR. Existing fmt, scoped warnings-denied Clippy, locked metadata, TC validation and actual test logs were inspected. The review does not substitute these for the lead's final gate.

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
