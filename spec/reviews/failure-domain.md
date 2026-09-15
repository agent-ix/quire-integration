---
id: SR-002
title: "Failure-domain review of composed integration"
type: SpecReview
analysis: failure-domain
scope: "FR-001..FR-006; NFR-001..NFR-002; IT-001..IT-004"
review_set: all
---
# SR-002: Failure-domain review of composed integration

## Summary

The review covered producer/adapter extension failures, domain identity,
evaluation purity, crash recovery and cyclic traversal. The effect protocol had
one high-severity crash-window gap, now resolved in FR-003 and IT-002.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-003 | high | Resolved: after adapter acceptance but before receipt persistence, the original contract could promise no second dispatch without requiring adapter reconciliation or idempotency semantics. FR-003 now requires both capabilities to be declared and returns `delivery-unknown` when neither permits a safe retry. | FR-003; IT-002; TC-003 | missing-requirement |

## Domain Results

- Every repository, package, definition, provider, build, deployment, role,
  workflow, decision, request, receipt, observation and evidence subject retains
  its own identity domain.
- Producer failures remain item-local; shared-prerequisite contradictions name
  only affected items and preserve completed sibling evidence.
- Evaluators, renderers, analyzers and qualification runners have no adapter
  capability. Only the effect coordinator may dispatch after authority and
  persistence checks.
- Dependency and view traversals terminate under preselected node/edge/work
  budgets; cycles and unresolved paths stay explicit.
- The post-call uncertainty state is never promoted to a receipt or business
  effect and never triggers an unsafe blind retry.

## Verdict

PASS. No unresolved failure-domain gap remains.
