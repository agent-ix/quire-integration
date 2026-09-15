---
id: FR-006
title: "Record complete composed integration qualification"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-003
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-integration/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-311
    type: references
  - target: ix://agent-ix/quire-specification/FR-336
    type: references
  - target: ix://agent-ix/quire-specification/FR-337
    type: references
  - target: ix://agent-ix/quire-specification/FR-338
    type: references
---
# FR-006: Record complete composed integration qualification

## Description

The qualification coordinator SHALL record an attributed result for every
required inventory cell under one validated ecosystem lock.

## Inputs

Independent conformance corpus and inventory revision; qualification profile;
validated ecosystem lock; exact producer, runtime, monitor, provider, backend,
tool, and consuming-system subjects; local execution limits; and evidence-intake
and human-decision contracts.

## Outputs

Immutable per-vector run records, per-inventory-cell accounting, scoped
qualification disposition, evidence-intake records, and an optional separate
attributed human decision.

## Behavior

- The coordinator SHALL validate corpus, inventory, profile, and lock identities
  before executing a vector.
- The coordinator SHALL execute every applicable IT-070 through IT-078 vector.
- The coordinator SHALL record each required cell as planned, unexecuted, passed,
  failed, unsupported, incomplete, refused, or waived.
- The coordinator SHALL require an attributed scope-limited decision for a waiver.
- The coordinator SHALL retain real producer bytes, results, logs, artifacts,
  limitations, and rights metadata without reinterpreting them.
- The coordinator SHALL distinguish API, CLI, WASM, reference-workflow, provider-
  containment, backend-replay, and evidence-intake results.
- The coordinator SHALL reproduce canonical results for the same corpus and lock.
- The coordinator SHALL derive a different run identity when any lock or corpus
  pin changes.
- The coordinator SHALL NOT promote a missing, unexecuted, failed, unsupported,
  incomplete, refused, or waived required vector to passed.
- The coordinator SHALL NOT make a release decision.
- The coordinator SHALL accept only a separate authenticated human-decision
  record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | Every required integration cell in the 176-row inventory has exactly one attributable current state and every vector retains its exact subject/run inputs. | integration-testing (TC-006) |
| FR-006-AC-2 | Missing, unexecuted, failed, unsupported, incomplete, refused, and waived vectors cannot produce complete qualification. | property-based-testing (TC-006) |
| FR-006-AC-3 | Equal corpus and lock inputs reproduce canonical per-vector and rollup bytes; changing any pin changes the run identity. | metamorphic-testing (TC-006) |
| FR-006-AC-4 | API/CLI/WASM parity, reference workflows, provider containment, backend replay, and evidence intake remain independently attributable. | integration-testing (TC-006) |
| FR-006-AC-5 | Evidence intake validates but does not rerun or reinterpret a producer; limitations and rights metadata constrain the resulting claim. | contract-testing (TC-006) |
| FR-006-AC-6 | Only a matching authenticated human decision can accept, reject, or explicitly limit the qualification scope without changing prior records. | negative-abuse-testing (TC-006) |

## Error Conditions

A corpus/inventory/profile/lock mismatch, duplicate or absent required cell,
producer refusal/failure, unavailable execution surface, rights limitation,
evidence-intake refusal, unauthenticated or over-broad decision, stale evidence,
or exceeded execution bound retains its exact typed state. No such state becomes
passed or complete, and no partial run mutates a prior record.

## Dependencies

- [FR-002](./FR-002-compose-execution-bundles.md) supplies composed execution.
- [FR-003](./FR-003-reconcile-authorized-effects.md) supplies authorized effect
  and observation reconciliation evidence.
- [FR-004](./FR-004-compose-bounded-consumer-views.md) supplies deterministic
  view and tool-consumer accounting.
- [FR-005](./FR-005-classify-compatibility-impact.md) supplies compatibility and
  stale-evidence accounting.
- QSpec FR-311 and FR-336 through FR-338 own qualification, corpus, evidence, and
  human-decision contracts.
