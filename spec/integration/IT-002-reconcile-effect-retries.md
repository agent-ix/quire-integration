---
id: IT-002
title: "Reconcile authorized effect retries"
type: IT
relationships:
  - target: ix://agent-ix/quire-integration/FR-003
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-211
    type: references
  - target: ix://agent-ix/quire-specification/IT-078
    type: references
---
# IT-002: Reconcile authorized effect retries

## Objective

Verify the real persistence and adapter boundary for authorization, crash/retry,
duplicate delivery, receipt reconciliation, and separately observed business effect.

## Target Integration

The test invokes the real effect coordinator, durable local request store, and a
contract-faithful local HTTP adapter test service. HTTP transport and persistence
are real. The external provider is replaced only by this local service because no
production payment endpoint is authorized; request/response fixtures are checked
against the selected adapter contract. The local service exposes the same stable
idempotency-key and reconciliation operations required of a selected production
adapter.

## Preconditions

The local durable store and adapter service are healthy and empty, one exact
adapter manifest is locked, and a unique authorized decision/request subject is
allocated for the run.

## Inputs

Valid, missing, expired, cross-scope, and over-broad authority decisions; fresh,
replayed, and conflicting idempotency keys; injected termination after persistence
and after adapter acceptance; matching and foreign QObs effect observations.

## Test Procedure

1. Persist and dispatch a valid authorized request.
   - IT-002-SC-01: one canonical pending record precedes one adapter call.
2. Terminate after request persistence and before attempt persistence, restart,
   and dispatch; then terminate after attempt persistence and before transport,
   restart, and reconcile before any further dispatch.
   - IT-002-SC-02: the same request/key is retained; an open durable attempt is
     always reconciled first because its crash window is indistinguishable from
     post-acceptance failure.
3. Terminate after adapter acceptance but before receipt persistence, restart,
   and reconcile first with an idempotent adapter and then with a manifest that
   offers neither conclusive reconciliation nor an idempotency guarantee.
   - IT-002-SC-03: the idempotent adapter yields the same external effect identity;
     the unsupported manifest yields `delivery-unknown` without another call.
4. Repeat a delivered request and then reuse its key with different canonical content.
   - IT-002-SC-04: replay is idempotent and conflicting reuse refuses before dispatch.
5. Submit each invalid authority variant.
   - IT-002-SC-05: the store and adapter observe no new request.
6. Record a transport receipt, then matching and foreign effect observations.
   - IT-002-SC-06: only the matching authority-qualified observation establishes
     the business effect.
7. Expire the grant after an attempt is durable and exercise reconciliation,
   receipt replay, and repeated dispatch.
   - IT-002-SC-07: reconciliation and durable receipt replay remain available;
     every new dispatch refuses before transport after expiry.
8. Race two fresh submissions with one caller key and unequal canonical requests.
   - IT-002-SC-08: one atomic reservation wins and the conflicting request cannot
     persist an attempt or invoke the adapter.

## Expected Results

At most one external effect request exists per canonical authorized request.
Persistence survives termination, uncertain delivery reconciles or remains
explicit, adapter-guaranteed retries preserve one effect identity, receipt and
effect remain distinct, authority expiry cannot authorize a new dispatch, and no
evaluator owns an adapter capability.
