---
id: Task-008
title: "IT-001 — reference workflow composition"
type: Task
status: not_started
track: I1
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-003
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-001
    type: references
  - target: ix://agent-ix/quire-integration/FR-002
    type: references
  - target: ix://agent-ix/quire-integration/IT-001
    type: verifies
---
# Task-008: IT-001 — reference workflow composition

## Scope

Compose the pinned ConfigVersion, timed-refund and compensated choreography workflows
through real owner strict readers and producers under one exact lock.

## Subtasks

- [ ] Record exact dependency revisions, features, schemas and fixture rights.
- [ ] Wire real QSL, State, TL, QObs, Protocol, Verification, IR and backend capabilities.
- [ ] Exercise IT-001-SC-01 through IT-001-SC-04 without semantic/file-load mocks.
- [ ] Permute completion order and adverse sibling dispositions.
- [ ] Retain every owner byte and identity for independent audit.

## Deliverables

- Reproducible trace-tagged IT-001 harness and exact ecosystem lock fixture.
- Evidence for StR-001-VC-1.

## Notes

- Missing compatible external revisions are explicit blockers, never substitute shims.
- Unblocks Task-012.
