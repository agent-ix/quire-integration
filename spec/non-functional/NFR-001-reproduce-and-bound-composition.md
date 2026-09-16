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

The library SHALL produce byte-identical canonical outputs for identical validated
inputs under finite preselected deterministic budgets for every input byte, retained
item, dependency edge, traversal step, producer invocation and output byte. Operational
wall-clock deadlines SHALL bound execution; deadline-derived outcomes are excluded from
the equal-input canonical-output comparison because elapsed time is not a semantic input.

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
| NFR-001-AC-2 | Deterministic work or retention exhaustion leaves each completed independent sibling intact and marks every unexecuted sibling explicitly. | fault-injection (TC-007) |
| NFR-001-AC-3 | Operational deadline expiry returns the operation's documented typed outcome: an atomic no-output failure, an item-local timeout, or a structurally admitted incomplete accounting envelope. No deadline-derived outcome is compared under the equal-input canonical-output metric. | fault-injection (TC-007) |
