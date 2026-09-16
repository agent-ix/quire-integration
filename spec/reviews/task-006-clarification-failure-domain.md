---
id: SR-036
title: "Failure-domain review of Task 006 compatibility impact"
type: SpecReview
analysis: failure-domain
scope: "FR-005; TC-005; Task-006"
review_set: all
---
# SR-036: Failure-domain review of Task 006 compatibility impact

## Summary

The failure-domain review examined strict-reader failure, identity collisions,
policy purity, graph topology, bounds and partial results. Each failure phase now
has one deterministic outcome and no failure can default to compatibility.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Resolved: malformed, duplicate, conflicting and authority-foreign owner values refuse before traversal; an admitted strict-reader failure or duration expiry is failed with no envelope; unresolved/cycle-only premises are local unknowns. | FR-005 Bounds and phase outcomes; Error Conditions |
| FND-002 | high | Resolved: per-root/effect visited keys, monotone effect union and effect-specific SCC classification distinguish self-loops/cycles from diamonds/shared descendants, preserve converging disjoint effects and independently grounded siblings. | FR-005 Canonical impact traversal; FR-005-AC-2; FR-005-AC-3 |
| FND-003 | high | Resolved: every input/work/retention/duration dimension has a finite ceiling; topology demand is atomically cardinality-preflighted, traversal is charge-before-expansion with an exact frontier, and envelope overflow refuses. | FR-005 Bounds and phase outcomes; TC-005 |
| FND-004 | medium | Resolved: policy rules are closed pure values with no callbacks/ambient defaults, and absent behavioral results require a closed comparable-result inventory rather than silence. | FR-005 Identity and owner contracts; Evidence freshness and regression vectors |
| FND-005 | high | Resolved: NFR-001 distinguishes reproducible deterministic exhaustion from each operation's documented deadline-derived outcome; Task 006 duration expiry is specifically a typed no-envelope failure and makes no canonical-equality claim. | NFR-001-AC-2; NFR-001-AC-3; FR-005 Bounds and phase outcomes |

## Verdict

PASS. The implementation can assign each adverse fixture one exact oracle.
