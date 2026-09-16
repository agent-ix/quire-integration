---
id: SR-031
title: "Scope and boundary review of Task 005 bounded consumers"
type: SpecReview
analysis: scope-boundary
scope: "FR-004; NFR-002; Task-005; Task-007; Task-010; Task-011"
review_set: all
---
# SR-031: Scope and boundary review of Task 005 bounded consumers

## Summary

The facade owns exact admission, bounded traversal, canonical accounting and
attribution. Semantic truth, mapping vocabularies, consumer implementation,
proof strength and qualification/release decisions remain with their owners.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: the generic ConsumerCapability did not bind a selected implementation or prevent consumer output from becoming semantic input. FR-004 now requires a task-specific admitted seam and forbids output re-entry. | FR-004-AC-5..AC-6; Task-005 | missing-requirement |
| FND-002 | medium | Resolved: local generated-artifact labels risked duplicating upstream mapping-loss and package semantics. FR-004 retains only FR-269's closed labels locally and allocates strict FR-297..FR-299 canonical pass-through to Task 010 real adapters. | FR-269; FR-297..FR-299 | wrong-requirement |

## Responsibility Allocation

| Boundary | Owner | Guarantee |
|---|---|---|
| checked package and semantic graph | selected owner reader | immutable qualified semantics and source projection |
| query/traversal/accounting | quire-integration Task 005 | bounded canonical view and complete facade accounting |
| mapping profile/record/package | QSpec output-mapping owners and Task 010 adapters | exact opaque owner values and target bytes; never reconstructed by Task 005 |
| heuristic/proof method result | QSpec FR-333 provider | explicit claim strength; no relabeling |
| global containment audit | Task 007 | cross-module conversion, allocation and authority evidence |
| real consumers/filesystem | Task 010 | pinned integration and structural observations |
| qualification/release | Task 011 plus human decision | no automatic promotion from consumer outputs |

## Verdict

PASS. Each authority has one owner and no semantic engine is duplicated.
