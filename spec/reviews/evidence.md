---
id: SR-005
title: "Evidence-method review of complete V1 integration"
type: SpecReview
analysis: evidence
scope: "StR-001; FR-001..FR-006; NFR-001..NFR-002; IT-001..IT-004; TM-001"
review_set: all
---
# SR-005: Evidence-method review of complete V1 integration

## Summary

Native Quoin advice over 44 obligations reports 36 method recommendation
mismatches, zero uncatalogued methods and one inconclusive criterion after
method-name normalization. The inconclusive method is confirmed by judgment as
an integration test because it crosses real durable persistence and adapter
boundaries; the recommendation mismatches are retained for the evidence-boundary
reasons below.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-007 | low | Confirmed by judgment: the catalog is inconclusive for FR-003-AC-1; `Integration Test (TC-003)` is required because a unit seam cannot prove persist-before-dispatch or attributed adapter receipt behavior. | FR-003-AC-1; TC-003; IT-002 |

## Method Decisions

- Lock identity, completion-order, bounded-view, compatibility and resource
  invariants use generated property tests.
- Real persistence, local HTTP adapter, strict reader and cross-project producer
  boundaries use integration tests; no persistence, file I/O or semantic
  producer is mocked.
- Adapter reachability, untrusted integer conversion and capability containment
  use static call-graph/source audits in TC-009 through TC-011.
- The advisor's example/unit recommendations do not replace property,
  metamorphic, contract, fault-injection, negative-abuse or architecture checks
  whose claims quantify across values, execution orders, crashes, trust
  boundaries or the call graph. Fixed examples remain cases within those
  suites.
- The advisor classifies each quantified NFR metric as performance
  benchmarking. These metrics constrain exact counts, canonical equality and
  architectural absence rather than elapsed time or throughput, so their
  authored methods provide the direct oracle.
- IT-004 retains unsupported/incomplete when the real Quoin evidence-intake
  capability is unavailable; a mock cannot produce passing evidence.

## Deterministic Evidence

Native `quoin 0.23.1-260-gf9aa659 advise --mismatch-only` reports 36
mismatches and zero uncatalogued methods across 44 obligations. `quoin advise
--inconclusive-only` reports only FR-003-AC-1. The method families above and the
FR-003-AC-1 decision are explicitly review judgment, not catalog verdicts.

## Verdict

PASS. Every obligation has a confirmed method and a planned evidence producer.
