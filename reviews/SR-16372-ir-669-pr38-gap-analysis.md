---
id: SR-16372
title: "IR-669 PR 38 T-row gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock and tests/text_source_agreement.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-186 T01 through T16"
review_set: subset
---

## Summary

Ticket: IR-669. Planless changed-test trace and evidence audit of draft PR #38. QSpec's 18 public TC-186 rows all dispatch to runnable controls; the focused T binary passes 17 tests. T06b compares checked-QSL and RT equality for all distinct value pairs while retaining raw and escaped source provenance.

## Verdict

PASS after the T06b two-sided observation fix. The final aggregate gate remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | T11 and T16 are tagged to charge or exact admission criteria, but the T11 denial checks omit shortage fields and T16 drops the denied Meter before asserting its prefix and counters. These gaps allow a wrong accounting result to satisfy the trace. | tests/text_source_agreement.rs:668 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | T06b constructs raw and escaped Text through checked QSL, but all pairwise equality assertions call only RT compare_text. Its runnable comparison therefore lacks the checked-QSL side required by IT-017-SC-02. | tests/text_source_agreement.rs:496 |

## Coverage

Plan completion: not assessed. `quoin matrix --repo ... --json` reported no coverage_matrix criteria because the installed module declares no `obligations:` source. I used a manual trace scan of IT-017-SC-01 through SC-05, QSpec TC-186 T01 through T16, and the changed `#[trace]` tests. T06b exact distinct source provenance, checked-QSL/RT equality and charge agreement execute. T09 changed owner, case set, and case order distinctions execute; identical reassembly succeeds. All 18 rows are runnable, no T PENDING route remains, and the unknown-row route fails closed. No reverse ownership gap, stub, ignored test, vendored fixture, or copied corpus was found in the changed files. Optional semantic review was not separately requested. The focused T test binary passes 17 tests; the aggregate gate is pending.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | T11 now checks denial metadata, prefix, and counters; T16 checks the retained denied meter's prefix and counters. |
| FND-002 | fixed | T06b now evaluates checked QSL equality for raw/escaped, raw/runtime, and escaped/runtime pairs at exact Text[2,2;binary-utf8] bounds, comparing completed Boolean results and full meter agreement with RT. |
