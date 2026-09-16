---
id: Task-010
title: "IT-003 — bounded consumer and impact integration"
type: Task
status: not_started
track: I3
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-005
    type: depends_on
  - target: ix://agent-ix/quire-integration/Task-006
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-004
    type: references
  - target: ix://agent-ix/quire-integration/FR-005
    type: references
  - target: ix://agent-ix/quire-specification/FR-297
    type: references
  - target: ix://agent-ix/quire-specification/FR-298
    type: references
  - target: ix://agent-ix/quire-specification/FR-299
    type: references
  - target: ix://agent-ix/quire-specification/FR-333
    type: references
  - target: ix://agent-ix/quire-specification/FR-334
    type: references
  - target: ix://agent-ix/quire-integration/IT-003
    type: verifies
---
# Task-010: IT-003 — bounded consumer and impact integration

## Scope

Run real model-view, renderer, analyzer and codegen consumers over isolated output, then
classify baseline/candidate compatibility and stale evidence through explicit graphs.

## Subtasks

- [ ] Record exact consumer revisions/profiles and approved fixture rights.
- [ ] Exercise IT-003-SC-01 through IT-003-SC-04 with real APIs/readers and filesystem I/O.
- [ ] Compare bytes/maps/loss under permutations and exact/one-over bounds.
- [ ] Repeat equal-input calls to every selected consumer and prove byte-identical,
      deterministic results with no undeclared effects or ambient inputs.
- [ ] Validate every generated output against its real reader without semantic promotion.
- [ ] Revalidate and retain the full canonical owner bytes and identities for the
      FR-297/FR-298/FR-299/FR-333/FR-334 values; do not reconstruct them from the facade.
- [ ] Preserve change classes, impact paths, cycles and unavailable vectors independently.

## Deliverables

- Trace-tagged IT-003 harness and immutable consumer artifacts/accounting.
- Exact owner-contract pass-through plus repeated-call determinism/effect evidence.
- Evidence for StR-001-VC-3.

## Notes

- Unblocks Task-012.
