---
id: Task-011
title: "FR-006 — qualification ledger and decision boundary"
type: Task
status: not_started
track: Q
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-003
    type: depends_on
  - target: ix://agent-ix/quire-integration/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-integration/Task-005
    type: depends_on
  - target: ix://agent-ix/quire-integration/Task-006
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-006
    type: references
  - target: ix://agent-ix/quire-integration/TC-006
    type: verifies
---
# Task-011: FR-006 — qualification ledger and decision boundary

## Scope

Implement immutable inventory/vector accounting, qualification rollups, evidence-intake
records and separate scope-limited human decisions without automatic release promotion.

## Subtasks

- [ ] Write TC-006 inventory, no-promotion, identity and decision-abuse tests first.
- [ ] Validate corpus/inventory/profile/lock before vector execution.
- [ ] Require one exact state per required cell/vector and retain raw outputs/limitations.
- [ ] Derive canonical run/rollup bytes and identity from exact inputs.
- [ ] Validate intake and human decisions without rerunning or enlarging producer claims.

## Deliverables

- Qualification coordinator, ledger/rollup/decision types and trace-tagged tests.
- Explicit passed/failed/unsupported/incomplete/refused/waived/unexecuted states.

## Notes

- Unblocks Task-012 after real integration evidence is also available.
