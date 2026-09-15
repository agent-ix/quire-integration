---
id: FR-003
title: "Reconcile authorized idempotent effects"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-002
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-182
    type: references
  - target: ix://agent-ix/quire-specification/FR-329
    type: references
  - target: ix://agent-ix/quire-specification/FR-338
    type: references
---
# FR-003: Reconcile authorized idempotent effects

## Description

When an attributed human or policy decision accepts a formal recommendation, the
workflow boundary SHALL validate its authority and persist one canonical effect
request before dispatching an explicitly selected adapter.

## Inputs

Recommendation/result identity and digest; immutable decision and authority grant;
effect parameters; adapter manifest declaring its idempotency field, duplicate-
request guarantee and reconciliation operation; caller-owned idempotency key;
persisted request, attempt and receipt history; optional authority-qualified
effect observation; and limits.

## Outputs

A rejected recommendation, pending/reconciled request, attributed transport
receipt, separately linked effect observation, or typed failure/incomplete result.

## Behavior

- The boundary SHALL validate actor, authority, action, subject, scope, expiry,
  decision evidence, adapter, and effect parameters before persistence or dispatch.
- The boundary SHALL derive the request identity from the selected adapter,
  decision, idempotency key, operation, and canonical arguments.
- The boundary SHALL persist the complete pending request before calling the
  adapter.
- The boundary SHALL persist an attempt state before each adapter call.
- When a prior attempt has no conclusive receipt, the boundary SHALL invoke the
  adapter's selected reconciliation operation before considering another call.
- The boundary SHALL issue another adapter call only when the manifest guarantees
  that byte-identical requests carrying the same key resolve to the same external
  effect identity.
- When neither conclusive reconciliation nor that idempotency guarantee is
  available, the boundary SHALL return `delivery-unknown` without another call.
- The boundary SHALL refuse reuse of an idempotency key with unequal canonical
  request content.
- The boundary SHALL retain a transport receipt as delivery/acceptance evidence
  only.
- The boundary SHALL accept business-effect truth only from a matching
  authority-qualified observation that re-enters through QObs.
- The boundary SHALL preserve each decision and request revision immutably with
  explicit supersession lineage.
- An evaluator, analyzer, renderer, or qualification runner SHALL NOT invoke an
  effect adapter directly.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | Valid authority and a fresh key persist and dispatch one canonical request with an attributed receipt. | integration-testing (TC-003) |
| FR-003-AC-2 | Exact duplicate delivery and crash/retry reconcile the same request without a distinct effect identity or second business effect; a repeated transport call occurs only under the selected adapter's idempotency guarantee. | fault-injection (TC-003) |
| FR-003-AC-3 | Missing, expired, mismatched, or over-broad authority refuses before persistence or adapter invocation. | negative-abuse-testing (TC-003) |
| FR-003-AC-4 | Reusing a key with different decision, adapter, operation, or arguments refuses as an identity contradiction. | property-based-testing (TC-003) |
| FR-003-AC-5 | Recommendation, decision, pending request, and transport receipt cannot establish the business effect; only the matching QObs fact can. | integration-testing (TC-003) |
| FR-003-AC-6 | Direct adapter invocation from every non-boundary component is absent from the public and internal call graph. | architecture-conformance (TC-009) |

## Error Conditions

Missing, expired, mismatched, or over-broad authority; decision/recommendation
mismatch; conflicting idempotency-key reuse; unsupported adapter guarantees;
inconclusive reconciliation; persistence failure; transport failure; malformed
receipt; foreign effect observation; and one-over limits return distinct typed
states. Persistence failure invokes no adapter. An inconclusive post-call state
returns `delivery-unknown` and never establishes a business effect.

## Dependencies

- QSpec FR-182 owns the workflow behavior; FR-329 owns the effect decision/receipt
  wire; FR-338 owns attributed human decisions.
- QObs owns business-effect observation authority.
