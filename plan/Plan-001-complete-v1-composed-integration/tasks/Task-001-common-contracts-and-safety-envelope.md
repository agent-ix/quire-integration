---
id: Task-001
title: "NFR-001/NFR-002 — common contracts and safety envelope"
type: Task
status: not_started
track: S
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/NFR-001
    type: references
  - target: ix://agent-ix/quire-integration/NFR-002
    type: references
  - target: ix://agent-ix/quire-integration/TC-007
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-008
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-010
    type: verifies
  - target: ix://agent-ix/quire-integration/TC-011
    type: verifies
---
# Task-001: NFR-001/NFR-002 — common contracts and safety envelope

## Scope

Replace the scaffold greeting with shared domain identities, closed dispositions, checked
resource budgets, canonical encoding and typed capability foundations.

## Subtasks

- [ ] Write failing domain-separation, bound and authority-confusion tests first.
- [ ] Define identity newtypes for every repository/component/role/artifact/run domain.
- [ ] Define closed non-success states without Boolean conversions or default promotion.
- [ ] Implement checked integer/budget helpers and deterministic canonical collections.
- [ ] Define separate producer, consumer, persistence, adapter, evidence and decision traits.

## Deliverables

- Shared Rust modules and trace-tagged TC-007/TC-008 foundation tests.
- No unsafe code, ambient lookup or cross-capability conversion.

## Notes

- Freeze these public primitives before Task-002 and parallel feature work.
- Unblocks Task-002.
