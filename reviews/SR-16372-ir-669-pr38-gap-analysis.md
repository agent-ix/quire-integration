---
id: SR-16372
title: "IR-669 PR 38 T-row gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock and tests/text_source_agreement.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-186 T01 through T16"
review_set: subset
---

## Summary

Ticket: IR-669. Planless changed-test trace and evidence audit of draft PR #38. QSpec's public TC-186 rows are dispatched to live tests or the ticket-backed T06b and T09 partial routes. The current focused T test binary passes all 17 tests.

## Verdict

PASS after round-1 dispositions for the reviewed trace and evidence gaps. The aggregate gate remains pending. The committed copy of this artifact still needs refresh to match the current evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | T11 and T16 are tagged to charge or exact admission criteria, but the T11 denial checks omit shortage fields and T16 drops the denied Meter before asserting its prefix and counters. These gaps allow a wrong accounting result to satisfy the trace. | tests/text_source_agreement.rs:668 |

## Coverage

Plan completion: not assessed. `quoin matrix --repo ... --json` reported no coverage_matrix criteria because the installed module declares no `obligations:` source. I used a manual trace scan of IT-017-SC-01 through SC-05, QSpec TC-186 T01 through T16, and the changed `#[trace]` tests. T06b distinct provenance is PENDING IR-1286; T09 case/order identity is PENDING IR-1300 / QSL-1574. Their available value and owner-change portions execute, and the unknown-row route fails closed. No reverse ownership gap, stub, ignored test, vendored fixture, or copied corpus was found in the changed files. Optional semantic review was not separately requested. The focused T test binary passes 17 tests; the aggregate gate is pending.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | T11 now checks denial metadata, prefix, and counters; T16 checks the retained denied meter's prefix and counters. |
