---
id: IT-004
title: "Qualify complete composed integration"
type: IT
relationships:
  - target: ix://agent-ix/quire-integration/FR-006
    type: verifies
  - target: ix://agent-ix/quire-specification/IT-070
    type: references
  - target: ix://agent-ix/quire-specification/IT-078
    type: references
---
# IT-004: Qualify complete composed integration

## Objective

Verify that IT-070 through IT-078 execute under one exact ecosystem lock and
produce complete, reproducible, attributable inventory/evidence accounting without
automatic release promotion.

## Target Integration

The test runs the real local Rust qualification coordinator against the pinned
independent corpus, producer/runtime/provider/backend/tool consumers, canonical
artifact store, and Quoin evidence-intake Rust boundary when available. If the
declared intake capability is unavailable, the corresponding vector is recorded
unsupported or incomplete rather than mocked.

## Preconditions

The reviewed 176-row inventory, independent corpus, qualification profile, exact
ecosystem lock, local artifact directory, and all declared producer binaries or
libraries are present. Rights metadata permits each test artifact.

## Inputs

Every applicable positive, boundary, adverse, compatibility, resource, API/CLI/
WASM parity, reference-workflow, provider-containment, backend-replay, and
evidence-intake vector from IT-070 through IT-078.

## Test Procedure

1. Validate the inventory, corpus, profile, lock, and expected vector set.
   - IT-004-SC-01: every required inventory cell has a unique planned vector set.
2. Execute each available vector and retain raw producer artifacts and results.
   - IT-004-SC-02: each vector has one attributable typed state and exact inputs.
3. Withhold, fail, and mark unsupported selected vectors, then recompute rollups.
   - IT-004-SC-03: only the containing scopes lose complete qualification.
4. Repeat the run with identical inputs and then one changed lock pin.
   - IT-004-SC-04: equal inputs reproduce bytes and the changed pin changes run identity.
5. Submit results to the real selected evidence-intake boundary and attach an
   authenticated accept-with-limits decision fixture.
   - IT-004-SC-05: intake preserves producer meaning and the decision cannot exceed
     its evidence or mutate a prior decision.

## Expected Results

Every required cell is attributable as passed, failed, unsupported, incomplete,
refused, waived, or unexecuted. No missing vector is promoted. Evidence and human
decision remain separate immutable artifacts, and the coordinator makes no release
decision itself.
