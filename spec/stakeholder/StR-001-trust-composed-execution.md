---
id: StR-001
title: "Stakeholders need attributable composed execution"
type: StR
relationships:
  - target: ix://agent-ix/quire-integration/FR-001
    type: satisfied_by
  - target: ix://agent-ix/quire-integration/FR-002
    type: satisfied_by
  - target: ix://agent-ix/quire-integration/FR-003
    type: satisfied_by
  - target: ix://agent-ix/quire-integration/FR-004
    type: satisfied_by
  - target: ix://agent-ix/quire-integration/FR-005
    type: satisfied_by
  - target: ix://agent-ix/quire-integration/FR-006
    type: satisfied_by
---
# StR-001: Stakeholders need attributable composed execution

## Stakeholder Need

The composed integration system SHALL keep every execution reproducible,
authority-qualified, and attributable across every participating repository,
domain result, external effect, tool consumer, and qualification decision.

## Rationale

Individually correct components can produce an unsafe system claim when versions
drift, results are flattened, retries duplicate effects, displays become semantic
authority, or incomplete evidence is promoted. Stakeholders need the complete
composition to preserve what each producer actually established and what remains
unknown.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-001-VC-1 | A state workflow, timed refund, and compensated choreography replay to the same multidimensional results under one exact ecosystem lock. | Demonstration (IT-001) |
| StR-001-VC-2 | A duplicate or crash-retried effect request cannot create a second business effect, and only an attributed observation establishes the effect. | Demonstration (IT-002) |
| StR-001-VC-3 | Deterministic views and code consumers account for every displayed, omitted, generated, unsupported, or truncated fact. | Demonstration (IT-003) |
| StR-001-VC-4 | Every required complete-V1 integration vector has an attributed result and no missing vector is promoted to success. | Demonstration (IT-004) |

## Stakeholders

Application owners, operators authorizing effects, assurance decision-makers,
and downstream consumers of native Quire results.

## Context and Assumptions

Every domain producer exposes a versioned typed Rust or canonical wire boundary.
The composition cannot infer missing semantics or prove a producer correct merely
because its interface is structurally compatible.

## Priority and Risk (Informative)

Priority is P0. Incorrect composition can duplicate business effects, reuse stale
evidence, or claim system success from incomplete independent results.
