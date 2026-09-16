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

Recommendation/result identity and digest; exact immutable, versioned owner bytes
and attestations for the `quire.human-decision/v1` decision and
`quire.effect-decision/v1` grant;
effect parameters; adapter manifest declaring its idempotency field, duplicate-
request guarantee and reconciliation operation; caller-owned idempotency key;
persisted request, attempt and receipt history; optional authority-qualified
effect observation; and limits.

## Outputs

A rejected recommendation, pending/reconciled request, attributed transport
receipt, separately linked effect observation, or typed failure/incomplete result.

## Behavior

- The boundary SHALL validate actor, authority, action, subject, exact scope,
  expiry through the selected authority's trusted time, decision and grant
  attestations through their distinct exact lock-admitted owner contracts, the
  complete lock-admitted adapter selection, and effect parameters before
  persistence or dispatch.
- The trusted composition root SHALL resolve decision authority, grant authority,
  observation authority, store, and adapter implementations from the exact lock;
  an untrusted submission or observation SHALL NOT select implementation values.
- The boundary SHALL accept the caller-owned idempotency key named by FR-329 and
  derive the request identity from the complete selected adapter capability,
  decision, authority grant, idempotency key, operation, and adapter-contract
  canonical arguments. The adapter SHALL NOT replace or reinterpret that key.
- The durable store SHALL reserve the key atomically across all adapter
  selections and compare the complete canonical request bytes on reuse.
- The boundary SHALL persist the complete pending request before calling the
  adapter, including exact owner artifacts and the admitting lock identity.
- The boundary SHALL persist an attempt state before each adapter call.
- When a persisted attempt has no conclusive receipt, including an attempt that
  may have crashed before transport, the boundary SHALL invoke the adapter's
  selected reconciliation operation before considering another call.
- The boundary SHALL issue another adapter call only when the manifest guarantees
  that byte-identical requests carrying the same key resolve to the same external
  effect identity and the authority grant remains current at the supplied logical
  time. Reconciliation and replay of an already durable receipt remain permitted
  after expiry because neither requests a new effect.
- When reconciliation is inconclusive or unavailable and that idempotency
  guarantee is absent, the boundary SHALL return `delivery-unknown` without
  another call. When reconciliation proves non-delivery but the guarantee or
  current authority required for another call is absent, the boundary SHALL
  return a distinct typed incomplete/refusal state without another call.
- The boundary SHALL refuse reuse of an idempotency key with unequal canonical
  request content.
- The boundary SHALL retain a transport receipt as delivery/acceptance evidence
  only.
- The boundary SHALL accept business-effect truth only from a lock-admitted QObs
  authority observation that matches the request identity and, when known, the
  external-effect identity. A missing receipt SHALL NOT prevent a sufficiently
  correlated authoritative observation from establishing the effect. Durable
  replay SHALL revalidate the retained owner and observation authority evidence.
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

Missing, expired, mismatched, or non-exact/over-broad authority;
decision/recommendation mismatch; conflicting global idempotency-key reuse;
unsupported retry guarantees; conclusively undelivered but unauthorized retry;
inconclusive reconciliation; persistence failure; transport failure; malformed
receipt; foreign effect observation; and one-over limits return distinct typed
states. Persistence failure before a request or attempt is durable invokes no
adapter. An inconclusive post-call state returns `delivery-unknown` and never
establishes a business effect.

## Dependencies

- QSpec FR-182 owns the workflow behavior. This facade resolves FR-182's
  adapter-derived-key wording in favor of FR-329's later, explicit caller-owned
  key contract; FR-329 also owns the effect decision/receipt wire, and FR-338
  owns attributed human decisions.
- QObs owns business-effect observation authority.
