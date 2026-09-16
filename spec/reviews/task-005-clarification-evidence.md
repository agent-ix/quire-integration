---
id: SR-029
title: "Evidence-method review of Task 005 bounded consumers"
type: SpecReview
analysis: evidence
scope: "FR-004; TC-004; Task-005; Task-007; Task-010"
review_set: all
---
# SR-029: Evidence-method review of Task 005 bounded consumers

## Summary

The evidence review distinguishes feature-local universal properties from later
real-tool integration. TC-004 owns generated property, contract, abuse and
compile-time evidence; Task 010 retains real service evidence.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | Resolved: AC-2’s universal “every item” claim was assigned integration testing while TC-004 is a property suite. It now uses property evidence; Task 010 separately owns real renderer/analyzer/codegen integration. | FR-004-AC-2; TC-004; IT-003 |
| FND-002 | medium | Resolved: TC-004 omitted displayed, omitted, generated, invalid, refused/incomplete and owner mapping-loss cases. The matrix now names every independent axis and adverse phase. | spec/tests.md TC-004 |

## Method Decisions

- Generated permutations compare complete canonical view, frontier, mapping,
  loss and accounting bytes against independent test oracles.
- Exact/one-over properties cover each feature-local work and retention bound.
- Negative tests assert zero consumer calls for preflight failures and no partial
  artifact for unsupported, malformed or exhausted cases.
- Compile-fail/API-surface evidence proves consumer artifacts cannot construct
  checked semantics or qualification decisions.
- Task 007 owns the repository-wide allocation/conversion and authority audit;
  Task 010 owns real pinned consumer and filesystem behavior.

## Verdict

PASS. Each claim has a proportionate evidence method and honest owner.
