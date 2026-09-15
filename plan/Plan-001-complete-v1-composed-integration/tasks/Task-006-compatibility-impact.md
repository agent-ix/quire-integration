---
id: Task-006
title: "FR-005 — compatibility impact and stale evidence"
type: Task
status: not_started
track: D
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-005
    type: references
  - target: ix://agent-ix/quire-integration/TC-005
    type: verifies
---
# Task-006: FR-005 — compatibility impact and stale evidence

## Scope

Classify structural/behavioral change, explicit dependency impact, stale evidence and
regression-vector availability without inferring compatibility from reachability or silence.

## Subtasks

- [ ] Write TC-005 four-class, path, cycle, unknown and stale-evidence properties first.
- [ ] Traverse only explicit typed dependency edges under finite bounds.
- [ ] Retain auditable changed-to-affected paths and circular/unresolved unknowns.
- [ ] Keep stale support distinct from counterevidence and semantic falsehood.
- [ ] Select every required regression vector once and expose unavailable/unsupported ones.

## Deliverables

- Compatibility/impact record types, traversal and trace-tagged tests.
- Canonical regression-vector inventory and explicit unknown/refused states.

## Notes

- Unblocks Tasks 010 and 011.
