---
id: SR-16371
title: "IR-669 PR 38 T-row code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock and tests/text_source_agreement.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-186 T01 through T16"
review_set: subset
---

## Summary

Ticket: IR-669. Reviewed draft PR #38 against IT-017 and QSpec TC-186. The tests use real checked QSL and runtime paths. The focused T binary passes 17 tests. All 18 QSpec TC-186 rows route to runnable controls, including T06b exact source provenance and two-sided equality, and T09 owner, case-set, and order identity.

## Verdict

PASS after fixed dispositions for FND-001 through FND-004. The final aggregate gate remains the lead's responsibility.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | T11's two injected-denial checks accept any `Incomplete` record with the named charge point. They do not assert the limit kind, limit, consumed amount, next charge, or full stop prefix, so a wrong shortage record can pass despite the SC-03 trace and QSpec T11's exact-denial obligation. | tests/text_source_agreement.rs:668 |
| FND-002 | medium | T16 checks the denied `Incomplete` record using a temporary Meter, then discards that meter. The named one-under cases therefore do not assert the admitted charge prefix or consumed counters, so a later or wrong charge sequence can pass the exact-limit test. | tests/text_source_agreement.rs:897 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The committed code-review and gap-analysis artifacts still state that QSL compilation and focused tests are blocked by IR-1284 and include raw SHA prose, although the current T binary passes all 17 tests. This makes the PR's durable verification record false and violates the repository's no-SHA prose rule. | reviews/SR-16371-ir-669-pr38-code-review.md:12; reviews/SR-16372-ir-669-pr38-gap-analysis.md:12 |

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The committed review records still call T06b and T09 PENDING and report only 16 runnable routes, while both now dispatch as runnable among 18 QSpec rows. The durable coverage record is false. | reviews/SR-16371-ir-669-pr38-code-review.md:12; reviews/SR-16372-ir-669-pr38-gap-analysis.md:12 |

## Coverage

Examined the full PR diff: Cargo.lock, tests/text_source_agreement.rs, and the two review artifacts. Checked IT-017-SC-01 through SC-05 and QSpec TC-186 T01 through T16, including T05b and T06b. The public QSpec table has 18 rows, all with runnable routes and no remaining T PENDING case. An unknown ID panics. T06b asserts distinct raw and escaped source spellings, equal retained bytes, and checked-QSL/RT equality and charge agreement for all distinct pairs. T09 asserts changed owner, case set, and case order produce a typed zero-charge refusal, while identical reassembly compares equal. No production API, CI workflow, vendored corpus, or copied fixture changed. Static `git diff --check` and `cargo fmt --all -- --check` pass. `cargo test --locked --test text_source_agreement -- --test-threads=1` passes 17 tests; the aggregate gate remains the lead's responsibility.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | T11 asserts exact QSL/runtime shortage records, the complete admitted-charge prefix, and every consumed counter for each named denial. |
| FND-002 | fixed | T16 retains the denied meter and asserts exact admitted charge points plus all consumed counters for each one-under case. |
| FND-003 | fixed | Both committed review artifacts now state the current focused test evidence, retain the exact pending T06b and T09 routes, and contain no raw SHA prose. |
| FND-004 | fixed | Both review artifacts now state all 18 QSpec T rows are runnable, T06b has two-sided equality, T09 case/order executes, and no T PENDING route remains. |
