---
id: FR-002
title: "Compose independent domain results"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-001
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-180
    type: references
  - target: ix://agent-ix/quire-specification/FR-328
    type: references
---
# FR-002: Compose independent domain results

## Description

When a validated execution request selects domain items, the facade SHALL emit one
canonical `quire.execution-bundle/v1` containing every requested item exactly once.

## Inputs

A validated FR-001 lock; exact checked package, workflow, observation authority,
requested state/temporal/protocol/analysis items, producer capabilities, and
caller-lowered limits.

## Outputs

An immutable ordered execution bundle with request, shared prerequisites,
per-domain results, and bundle disposition, or a typed pre-dispatch refusal.

## Behavior

- The facade SHALL validate every producer artifact through its owning strict
  reader before invocation.
- The facade SHALL dispatch only the exact supported Rust producers selected by
  the validated request.
- The facade SHALL invoke a producer only when the exact lock admits its contract,
  implementation, features, and options.
- The facade SHALL preserve each producer result byte string, identity,
  disposition, truth, execution, adequacy, completeness, and provenance axis.
- The facade SHALL include unsupported, incomplete, refused, failed, and
  unexecuted requested items in the ordered bundle.
- The facade SHALL order items by selected domain and semantic identity rather
  than completion time.
- The facade SHALL compute the bundle disposition without relabeling any sibling.
- If a shared prerequisite contradicts a result selection, then the facade SHALL
  refuse composition and identify every affected item.
- The facade SHALL NOT parse source, reinterpret a result, or select a substitute
  producer.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | State workflow, timed-refund, and compensated choreography requests each retain every requested domain result exactly once under one lock. | integration-testing (TC-002) |
| FR-002-AC-2 | Independent unsupported, incomplete, refused, failed, and completed siblings remain present with unchanged identities and dispositions. | property-based-testing (TC-002) |
| FR-002-AC-3 | Completion-order permutation produces byte-identical canonical bundles. | metamorphic-testing (TC-002) |
| FR-002-AC-4 | A shared-prerequisite contradiction refuses the bundle and names every affected item without erasing unaffected producer evidence. | integration-testing (TC-002) |
| FR-002-AC-5 | Unknown producer versions or unsupported capabilities emit no substitute invocation or result. | contract-testing (TC-002) |

## Error Conditions

An invalid producer artifact, unselected implementation, contract/feature/option
mismatch, duplicate or missing requested item, contradictory shared prerequisite,
producer refusal/failure, timeout, or exceeded bound returns a typed item-local
outcome or pre-dispatch composition refusal. No error path relabels a completed
sibling or synthesizes a substitute result.

## Dependencies

- [FR-001](./FR-001-validate-exact-ecosystem-lock.md) supplies the accepted lock.
- QSpec FR-328 owns the execution-bundle contract; domain repositories own every
  embedded result.
