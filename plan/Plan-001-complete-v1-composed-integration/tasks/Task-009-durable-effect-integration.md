---
id: Task-009
title: "IT-002 — durable effect and crash/retry integration"
type: Task
status: not_started
track: I2
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-003
    type: references
  - target: ix://agent-ix/quire-integration/IT-002
    type: verifies
---
# Task-009: IT-002 — durable effect and crash/retry integration

## Scope

Exercise real local durable persistence and a contract-faithful HTTP adapter through both
crash windows, reconciliation, duplicate delivery and QObs effect observation.

## Subtasks

- [ ] Implement isolated durable store and local adapter test service with selected manifest.
- [ ] Exercise IT-002-SC-01 through IT-002-SC-06 with actual process termination/restart.
- [ ] Verify reconciliation before retry and `delivery-unknown` without safe guarantees.
- [ ] Verify invalid authority/key reuse changes neither store nor adapter state.
- [ ] Establish effect truth only from the matching QObs observation.

## Deliverables

- Trace-tagged service integration harness, adapter contract fixture and crash evidence.
- Evidence for StR-001-VC-2.

## Notes

- No production payment endpoint is authorized; the local service is the declared boundary.
- Unblocks Task-012.
