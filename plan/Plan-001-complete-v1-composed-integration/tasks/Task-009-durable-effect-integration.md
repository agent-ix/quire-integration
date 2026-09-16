---
id: Task-009
title: "IT-002 — durable effect and crash/retry integration"
type: Task
status: in_progress
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

- [x] Implement isolated durable store and local adapter test service with selected manifest.
- [ ] Exercise IT-002-SC-01 through IT-002-SC-06 with actual process termination/restart.
- [x] Verify reconciliation before retry and `delivery-unknown` without safe guarantees.
- [x] Verify invalid authority/key reuse changes neither store nor adapter state.
- [ ] Establish effect truth only from the matching QObs observation.

## Deliverables

- Trace-tagged service integration harness, adapter contract fixture and crash evidence.
- Evidence for StR-001-VC-2.

## Notes

- No production payment endpoint is authorized; the local service is the declared boundary.
- SC-01 through SC-05 and SC-07/SC-08 pass through real filesystem, TCP and child-process
  boundaries. SC-06 remains open because QObs does not yet publish a downstream-consumable
  business-effect handoff; `assurance/task-009-durable-effect-inventory.md` records the exact
  owner gap without inventing a QObs contract or copying unapproved fixtures.
- Rust review SR-047 passes the implemented subset after durability, concurrency, strict-reader,
  deadline and evidence remediation. Gap analysis SR-048 records 7/8 scenario coverage and no
  trace/status lie while preserving the SC-06 blocker.
- Completion of the QObs-owned SC-06 handoff unblocks Task-012.
