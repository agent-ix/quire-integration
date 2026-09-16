---
id: Task-004
title: "FR-003 — authorized effect reconciliation"
type: Task
status: done
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

- [x] Write TC-003 authority, key-conflict and injected crash tests first.
- [x] Define canonical request identity, adapter manifest guarantee, and opaque
  dispatch-permit types.
- [x] Atomically reserve the caller key and append pending-request/attempt state
  before each call.
- [x] Reconcile uncertain prior attempts before retry and emit `delivery-unknown` when unsafe.
- [x] Accept business-effect truth only through a matching QObs authority fact.
- [x] Prove exact replay, key-conflict races, grant expiry, and every crash window.

## Deliverables

- Effect coordinator, persistence/adapter ports and trace-tagged unit/fault tests.
- Static call-graph evidence hooks for TC-009 and TC-011.

## Notes

- No other module may construct a dispatch permit or invoke the adapter.
- A repeated dispatch requires the exact same-key/same-bytes guarantee and a
  currently valid authority grant; read-only reconciliation does not.
- Unblocks Tasks 009 and 011.
