---
id: Task-002
title: "FR-001 — exact ecosystem lock"
type: Task
status: not_started
track: S
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-001
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-001
    type: references
  - target: ix://agent-ix/quire-integration/TC-001
    type: verifies
---
# Task-002: FR-001 — exact ecosystem lock

## Scope

Implement canonical `quire.ecosystem-lock/v1` validation and a constructor-private view that
must exist before any producer or consumer invocation.

## Subtasks

- [ ] Write TC-001 generated lock permutations, mutations and refusal cases first.
- [ ] Model exact repository, package, definition, provider, toolchain, build, deployment,
  component, role and workflow-instance selections.
- [ ] Validate explicit dependency edges and keep reachability distinct from compatibility.
- [ ] Derive canonical bytes/identity independent of discovery order.
- [ ] Prove every invalid or one-over lock returns before invocation counters change.

## Deliverables

- Lock schema, writer/reader, validator and trace-tagged TC-001 suite.
- Stable selection/domain-specific refusal codes.

## Notes

- Unblocks Tasks 003–006 and all integration tracks.
