---
id: Task-005
title: "FR-004 — bounded deterministic consumer views"
type: Task
status: not_started
track: C
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-004
    type: references
  - target: ix://agent-ix/quire-integration/TC-004
    type: verifies
---
# Task-005: FR-004 — bounded deterministic consumer views

## Scope

Compose canonical bounded semantic views and complete per-obligation consumer accounting
without allowing presentation or generated artifacts to become authority.

## Subtasks

- [ ] Write TC-004 determinism, frontier, unsupported and authority-isolation tests first.
- [ ] Traverse only selected typed relations from the exact qualified root.
- [ ] Stop before bounds and report the exact omitted frontier.
- [ ] Map every output region/finding/artifact to exact source and semantic identities.
- [ ] Preserve displayed/omitted/generated/unsupported/requires-bound/invalid/failed states.

## Deliverables

- View/query and consumer-accounting modules with trace-tagged property tests.
- Canonical mapping/loss records and no substitute artifacts.

## Notes

- Unblocks Tasks 010 and 011.
