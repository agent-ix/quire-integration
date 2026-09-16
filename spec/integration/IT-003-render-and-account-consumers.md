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
and unsupported obligations, completion-order permutations, exact
FR-297/FR-298/FR-299/FR-307/FR-310/FR-311/FR-333/FR-334/FR-336/FR-337 owner
fields/values plus canonical owner bytes only where those owners define them, and
explicit typed evidence dependency graphs including cycles and unresolved edges.

## Test Procedure

1. Query and render each supported bounded view repeatedly under equal and permuted
   input order, using a selected consumer whose declared effect/rights inventory is empty.
   - IT-003-SC-01: repeated equal-input calls produce byte-identical bytes, ordering,
     identity maps, and loss records with no ambient filesystem, network, clock, locale,
     or path observations.
2. Exceed each traversal/output bound and request an unsupported view/obligation.
   - IT-003-SC-02: the exact frontier/accounting appears and no substitute artifact exists.
3. Run the analyzer and code generator, retain each canonical owner value unchanged,
   and validate each emitted output mapping with its strict owner reader.
   - IT-003-SC-03: every requested obligation has one source-bound disposition and
     every FR-297/FR-298/FR-299/FR-333/FR-334 owner byte string and identity survives
     adapter pass-through exactly.
4. Strict-validate the real comparison/correspondence and FR-310 graph fields, strict-read
   canonical FR-336 vector and FR-337 intake bytes, compare baseline/candidate locks and
   traverse only policy-oriented edges through bounded Task-006 projections.
   - IT-003-SC-04: independent structural/behavioral/presence tuples, affected paths,
     stale evidence, cycles,
     and unavailable regression vectors remain distinct; all owner bytes/fields, typed
     identities, relations, scope, provenance, rights and limitations survive unchanged.

## Expected Results

Every view and consumer output is deterministic, bounded, and source-attributed;
repeated equal-input calls are byte-identical and full owner contracts pass unchanged.
Rendering and generated syntax do not become authority. Compatibility impact and
stale evidence remain evidence states rather than inferred semantic truth.
