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
- [ ] Validate every generated output against its real reader without semantic promotion.
- [ ] Preserve change classes, impact paths, cycles and unavailable vectors independently.

## Deliverables

- Trace-tagged IT-003 harness and immutable consumer artifacts/accounting.
- Evidence for StR-001-VC-3.

## Notes

- Unblocks Task-012.
