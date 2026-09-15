---
id: Task-004
title: "FR-003 — authorized effect reconciliation"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-003
    type: references
  - target: ix://agent-ix/quire-integration/TC-003
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-009
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-011
    type: verifies
---
# Task-004: FR-003 — authorized effect reconciliation

## Scope

Implement the only adapter-capable boundary: validate authority, persist request/attempt,
dispatch or reconcile safely, and keep receipts distinct from observed business effects.

## Subtasks

- [ ] Write TC-003 authority, key-conflict and injected crash tests first.
- [ ] Define canonical request identity and adapter manifest guarantee types.
- [ ] Persist pending request and attempt state before each call.
- [ ] Reconcile uncertain prior attempts before retry and emit `delivery-unknown` when unsafe.
- [ ] Accept business-effect truth only through a matching QObs authority fact.

## Deliverables

- Effect coordinator, persistence/adapter ports and trace-tagged unit/fault tests.
- Static call-graph evidence hooks for TC-009 and TC-011.

## Notes

- No other module may import the dispatch capability.
- Unblocks Tasks 009 and 011.
