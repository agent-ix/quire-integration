---
id: IT-003
title: "Render and account for bounded consumers"
type: IT
relationships:
  - target: ix://agent-ix/quire-integration/FR-004
    type: verifies
  - target: ix://agent-ix/quire-integration/FR-005
    type: verifies
  - target: ix://agent-ix/quire-specification/IT-074
    type: references
  - target: ix://agent-ix/quire-specification/IT-075
    type: references
---
# IT-003: Render and account for bounded consumers

## Objective

Verify real deterministic model-view, renderer, analysis, and code-generation
consumer integrations plus compatibility-impact and stale-evidence accounting.

## Target Integration

The test loads a real pinned checked/IR package, invokes the selected Rust query,
renderer, analyzer, and codegen APIs, writes their actual outputs to an isolated
filesystem directory, and validates the emitted artifacts with their real readers.
No file I/O or consumer API is mocked.

## Preconditions

The exact consumer implementations and profiles are present under the ecosystem
lock, an isolated output directory is empty, and baseline/candidate locks define
additive, payload-breaking, behavioral-only, and absent-result variants.

## Inputs

Bounded model-view queries, exact and one-over traversal/output limits, supported
and unsupported obligations, completion-order permutations, and explicit typed
evidence dependency graphs including cycles and unresolved edges.

## Test Procedure

1. Query and render each supported bounded view twice under permuted input order.
   - IT-003-SC-01: bytes, ordering, identity maps, and loss records agree exactly.
2. Exceed each traversal/output bound and request an unsupported view/obligation.
   - IT-003-SC-02: the exact frontier/accounting appears and no substitute artifact exists.
3. Run the analyzer and code generator and validate each emitted output mapping.
   - IT-003-SC-03: every requested obligation has one source-bound disposition.
4. Compare baseline/candidate locks and traverse the evidence dependency graph.
   - IT-003-SC-04: four change classes, affected paths, stale evidence, cycles,
     and unavailable regression vectors remain distinct.

## Expected Results

Every view and consumer output is deterministic, bounded, and source-attributed.
Rendering and generated syntax do not become authority. Compatibility impact and
stale evidence remain evidence states rather than inferred semantic truth.
