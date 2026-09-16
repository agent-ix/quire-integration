---
id: SR-039
title: "Evidence review of Task 006 compatibility impact"
type: SpecReview
analysis: evidence
scope: "FR-005-AC-1..FR-005-AC-6; TC-005; IT-003"
review_set: all
---
# SR-039: Evidence review of Task 006 compatibility impact

## Summary

Native `quoin advise --json` was run over every FR-005 criterion. AC-2 through
AC-5 include property-based testing among deterministic recommendations. AC-1 and
AC-6 are classified as example-shaped, but property evidence is retained by explicit
judgement because their oracles are respectively the complete generated Cartesian
product and universal permutation/mutation metamorphisms.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | Judgement recorded: the advisor recommends unit/BDD evidence for AC-1 and AC-6, while TC-005 deliberately combines fixed named examples with generated closed-axis enumeration and permutation/mutation properties. | FR-005-AC-1; FR-005-AC-6; TC-005 |
| FND-002 | medium | Resolved: AC-2 owns graph permutation, disjoint-effect convergence and canonical-witness properties; AC-3 owns atomic-topology/traversal/phase/boundary generation; AC-4 owns freshness/relation metamorphisms; AC-5 owns policy/vector exact-once properties; AC-6 owns canonical permutation/mutation/direction identity. | FR-005-AC-2..AC-6; TC-005 |
| FND-003 | medium | Resolved: Task 010/IT-003 separately owns contract tests against real FR-307/310 fields, FR-336/337 owner bytes and FR-311 execution/accounting providers; TC-005 does not claim that integration evidence. | Task-010; IT-003 |

## Verdict

PASS. Every feature-local obligation has an executable evidence strategy and the
two advisor mismatches are documented method judgements rather than defaults.
