---
id: NFR-001
title: "Reproduce and bound composed operations"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-integration/FR-001
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-002
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-003
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-004
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-005
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-006
    type: constrains
---
# NFR-001: Reproduce and bound composed operations

## Statement

The library SHALL produce byte-identical outputs for identical validated inputs
under a finite preselected budget for every input byte, retained item, dependency
edge, traversal step, producer invocation, output byte, and execution duration.

## Scope

Applies to lock validation, execution composition, effect reconciliation,
semantic views, compatibility impact, qualification orchestration, strict
readers, and canonical writers.

## Rationale

Cross-repository composition magnifies nondeterminism and resource expansion.
Reproducible outputs and fail-closed budgets are required for replay and evidence.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Equal-input canonical output equality | 100% of generated fixtures | 100% | property-based-testing (TC-007) |
| Retained work after one-over bound | 0 excess items | 0 | property-based-testing (TC-007) |
| Producer invocations after preflight refusal | 0 invocations | 0 | unit-testing (TC-007) |
| Partial validated outputs after refusal | 0 outputs | 0 | unit-testing (TC-007) |

## Verification

Generated tests repeat and permute semantic inputs, exercise exact and one-over
budgets for every collection and execution boundary, and compare complete bytes,
stable codes, usage, and producer invocation counters.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-001-AC-1 | Every externally supplied size, count, bound, duration, and integer conversion is checked before allocation, traversal, dispatch, or arithmetic. | static-quality (TC-010) |
| NFR-001-AC-2 | Timeout or resource exhaustion leaves each completed independent sibling intact and marks every unexecuted sibling explicitly. | fault-injection (TC-007) |
