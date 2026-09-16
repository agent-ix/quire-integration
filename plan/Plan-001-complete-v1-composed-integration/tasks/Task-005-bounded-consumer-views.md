---
id: Task-005
title: "FR-004 — bounded deterministic consumer views"
type: Task
status: done
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

- [x] Write TC-004 determinism, frontier, state-axis and authority-isolation tests first.
- [x] Validate one owner-produced checked projection and closed query under the exact lock.
- [x] Traverse only selected typed relations by canonical breadth-first order.
- [x] Stop before bounds and report exact usage plus the immediate pending frontier.
- [x] Map every output region/finding/artifact to exact source and semantic identities.
- [x] Preserve aggregate, facade-item and owner mapping-loss axes without translation.
- [x] Enforce pure consumer admission, complete obligation accounting and no promotion.

## Deliverables

- View/query and consumer-accounting modules with trace-tagged property tests.
- Canonical feature-local facade accounting/loss records and no substitute artifacts.

## Notes

- Unblocks Tasks 010 and 011.
