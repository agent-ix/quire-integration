---
id: SR-16371
title: "IR-669 PR 38 T-row code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock and tests/text_source_agreement.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-186 T01 through T16"
review_set: subset
---

## Summary

Ticket: IR-669. Reviewed draft PR #38 at e0099cc against IT-017 and QSpec TC-186. The tests use real checked QSL and runtime paths. Compilation is independently blocked before this test crate by the upstream QSL/quire-eval mismatch IR-1284.

## Verdict

PASS after round-1 dispositions for the reviewed diff. T06b correctly names its distinct-provenance subcase PENDING IR-1286. Compilation, focused tests, and the final aggregate gate remain blocked by IR-1284; no passing test claim is made.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | T11's two injected-denial checks accept any `Incomplete` record with the named charge point. They do not assert the limit kind, limit, consumed amount, next charge, or full stop prefix, so a wrong shortage record can pass despite the SC-03 trace and QSpec T11's exact-denial obligation. | tests/text_source_agreement.rs:668 |
| FND-002 | medium | T16 checks the denied `Incomplete` record using a temporary Meter, then discards that meter. The named one-under cases therefore do not assert the admitted charge prefix or consumed counters, so a later or wrong charge sequence can pass the exact-limit test. | tests/text_source_agreement.rs:897 |

## Coverage

Examined the full PR diff: Cargo.lock and tests/text_source_agreement.rs. Checked IT-017-SC-01 through SC-05, QSpec TC-186 T01 through T16 (including T05b and T06b), and the changed test dispatch and traces. The exhaustive T-ID route fails for an unknown ID and names IR-1286 for T06b's missing provenance. T09 includes owner, case, and order identity changes. No production API, CI workflow, vendored corpus, or copied fixture changed. Static `git diff --check` and `cargo fmt --all -- --check` passed. Focused compilation and tests are blocked before this test crate by IR-1284 (QSL 1d8893f6 / quire-eval 4e59f729).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | T11 asserts exact QSL/runtime shortage records, the complete admitted-charge prefix, and every consumed counter for each named denial. |
| FND-002 | fixed | T16 retains the denied meter and asserts exact admitted charge points plus all consumed counters for each one-under case. |
