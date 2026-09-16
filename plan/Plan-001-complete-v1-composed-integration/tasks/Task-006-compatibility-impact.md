---
id: Task-006
title: "FR-005 — compatibility impact and stale evidence"
type: Task
status: done
track: D
priority: P0
relationships:
  - target: ix://agent-ix/quire-integration/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-005
    type: references
  - target: ix://agent-ix/quire-integration/TC-005
    type: verifies
---
# Task-006: FR-005 — compatibility impact and stale evidence

## Scope

Classify structural/behavioral change, explicit dependency impact, stale evidence and
regression-vector availability without inferring compatibility from reachability or silence.

## Subtasks

- [x] Write TC-005 identity, axis-cross-product, topology, bound, freshness,
      policy/vector and canonicalization properties first.
- [x] Strict-read the feature-local comparison, graph, policy and vector-manifest
      contracts plus FR-336 corpus and FR-337 evidence-intake contracts through exact
      lock-admitted readers with caller-lowered bounds.
- [x] Traverse only policy-oriented owner edges under finite local bounds.
- [x] Propagate affectedness and staleness by independent monotone effect transfer,
      exact-only scope matching and per-root/effect canonical witnesses.
- [x] Preflight the complete effect-oriented topology atomically before SCC analysis.
- [x] Retain canonical shortest changed-to-affected witnesses, cycle witnesses,
      exact incomplete frontiers and independent siblings.
- [x] Keep owner support/contradiction bytes distinct from the freshness overlay and truth.
- [x] Strict-read and preserve referenced FR-337 intake bytes, rights and limitations.
- [x] Select every policy vector once with independent requirement and availability axes.
- [x] Bind every owner input, limit and state axis into canonical directional run identity.

## Deliverables

- Feature-local compatibility/impact overlays, bounded traversal and trace-tagged tests.
- Canonical policy/vector inventory with independent requirement/availability states.
- Bounded strict-reader seams for Task-006 comparison/graph/policy/manifest contracts
  that retain exact FR-307/310 fields and FR-336/337 owner bytes without substitution.

## Notes

- Unblocks Tasks 010 and 011.
