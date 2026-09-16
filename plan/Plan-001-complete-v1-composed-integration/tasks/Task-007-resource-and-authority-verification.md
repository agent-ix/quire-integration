---
id: Task-007
title: "NFR-001/NFR-002 — resource and authority verification"
type: Task
status: done
track: G
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
  - target: ix://agent-ix/quire-integration/NFR-001
    type: references
  - target: ix://agent-ix/quire-integration/NFR-002
    type: references
  - target: ix://agent-ix/quire-integration/TC-007
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-008
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-009
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-010
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-011
    type: verifies
---
# Task-007: NFR-001/NFR-002 — resource and authority verification

## Scope

Close generated and static evidence for resource containment, authority separation and the
single permitted adapter-dispatch path across all implemented feature modules.

## Subtasks

- [x] Exercise equal-input, permutation, timeout and exact/one-over properties globally.
- [x] Prove completed siblings survive exhaustion and unexecuted siblings stay explicit.
- [x] Inventory every untrusted conversion/allocation and its checked bound.
- [x] Audit imports/call graph so only FR-003 can reach dispatch after persistence.
- [x] Attempt renderer/analyzer/codegen/evidence/decision authority confusion adversarially.

## Deliverables

- Trace-tagged TC-007 through TC-011 suites and static inventories.
- Zero unauthorized dispatches, ambient lookups and unsafe blocks.

## Notes

- Failures return to their owning feature task; this gate adds no alternative behavior.
- Unblocks Task-012.
