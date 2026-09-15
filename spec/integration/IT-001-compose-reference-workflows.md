---
id: IT-001
title: "Compose the locked reference workflows"
type: IT
relationships:
  - target: ix://agent-ix/quire-integration/FR-001
    type: verifies
  - target: ix://agent-ix/quire-integration/FR-002
    type: verifies
  - target: ix://agent-ix/quire-specification/IT-070
    type: references
  - target: ix://agent-ix/quire-specification/IT-071
    type: references
  - target: ix://agent-ix/quire-specification/IT-072
    type: references
---
# IT-001: Compose the locked reference workflows

## Objective

Verify that the real pinned Rust producers execute ConfigVersion, timed-refund,
and fan-out/join/compensation workflows through one exact lock and one composed
facade without result flattening.

## Target Integration

The test links exact QSL, state, TL, QObs, Protocol, Verification, Contract IR,
and generated-oracle Rust artifacts into the real `quire-integration` facade. No
producer, strict reader, file load, or execution path is mocked.

## Preconditions

Every pinned repository revision is locally available, each package builds under
the locked Rust toolchain/features, and the conformance fixture rights record
allows its exact bytes in this private test.

## Inputs

The version-locked ConfigVersion, timed-refund, and compensated choreography
packages; healthy, boundary, violating, partial, late-revision, unsupported, and
resource-exhausted inputs; and exact/one-over limits.

## Test Procedure

1. Validate the complete ecosystem lock and all producer artifacts through their
   real strict readers.
   - IT-001-SC-01: every expected pin is accepted and no ambient lookup occurs.
2. Run each workflow through the selected state, temporal, observation, protocol,
   analysis, and backend producers.
   - IT-001-SC-02: every requested result is returned exactly once with its owner bytes.
3. Compose the results while permuting producer completion order.
   - IT-001-SC-03: canonical execution bundles remain byte-identical.
4. Repeat with unsupported, incomplete, failed, and contradictory shared inputs.
   - IT-001-SC-04: independent siblings remain visible and shared contradictions
     refuse only the affected composition.

## Expected Results

All three workflows produce their declared multidimensional results under one
exact lock. No unsupported or incomplete result becomes Boolean success, and no
producer semantics are duplicated by the facade.
