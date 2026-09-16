---
id: SR-027
title: "Integrity review of Task 005 bounded consumers"
type: SpecReview
analysis: integrity
scope: "FR-004; TC-004; Task-005"
review_set: all
---
# SR-027: Integrity review of Task 005 bounded consumers

## Summary

Completeness, consistency, atomicity and testability were checked across the
view and consumer-accounting halves. Canonical traversal, inventory closure,
region validity and no-substitute invariants now form one consistent contract.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: no traversal order, visited key, root rule, filter timing or cycle rule existed, making deterministic output unprovable. FR-004 now fixes canonical breadth-first traversal and post-discovery filtering. | FR-004 Behavior; FR-004-AC-1 | missing-requirement |
| FND-002 | high | Resolved: an “exact omitted frontier” was promised without defining what is observable under a work limit. It is now the immediate canonical unexpanded queue over already validated adjacency, never the undiscovered closure. | FR-004 Behavior; FR-004-AC-3 | wrong-requirement |
| FND-003 | medium | Resolved: malformed-map refusal lacked range, identity and multiplicity invariants. Half-open bounded regions, profile UTF-8 boundaries and no duplicate/cross-wired obligation mappings are now required. | FR-004 Behavior; FR-298; FR-299 | missing-requirement |

## Traceability

| Obligation | Required evidence |
|---|---|
| FR-004-AC-1 | permutation and meaning-mutation properties over complete canonical bytes |
| FR-004-AC-2 | generated mixed-state inventory with exact identities and distinct axes |
| FR-004-AC-3 | exact/one-over traversal, result, frontier and accounting bounds |
| FR-004-AC-4 | phase-specific negative cases and zero-call preflight refusals |
| FR-004-AC-5 | layout/heuristic/syntax/parser metamorphisms preserve checked semantics |
| FR-004-AC-6 | compile-time and API-surface proof of no authority conversion |

## Verdict

PASS. The clarified slice is internally consistent and fully observable.
