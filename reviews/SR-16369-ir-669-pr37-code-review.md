---
id: SR-16369
title: "IR-669 PR 37 F-row code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock, tests/ieee_source_agreement.rs, tests/scalar_agreement_ieee_text.rs; IT-017-SC-01 through IT-017-SC-05, TC-097"
review_set: subset
---

## Summary

Ticket: IR-669. Reviewed the initial draft PR #37 against IT-017 and QSpec TC-193. The tests exercise real checked QSL and runtime paths, but selected runnable obligations and the row-inventory guard are missing.

## Verdict

FAIL. One high-severity inventory gap and four medium-severity test gaps remain. Compilation is independently blocked by the upstream QSL/quire-eval cohort mismatch IR-1284; no passing test or aggregate gate is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-193 tests only call selected_vector for named rows; no test dispatches every QSpec F ID to an executable case or ticket-backed PENDING case. A newly added row or an omitted runnable row passes silently, despite IT-017-SC-01 and TC-097 requiring unknown IDs and ticketless omissions to fail. | tests/ieee_source_agreement.rs:34-47 |
| FND-002 | medium | F10 runs with unbounded LIMITS only. The required exact I(64,2,4,1) meter and independently denied ieee.result-retain observation are absent, so a QSL/runtime regression on that named stop is undetected. | tests/ieee_source_agreement.rs:1128-1156 |
| FND-003 | medium | F13 runs divide-by-zero cases but never runs the immediately following exact no-flag operation required by TC-193. A sticky flag carried across operations would pass this test. | tests/scalar_agreement_ieee_text.rs:223-244 |
| FND-004 | medium | F20 covers only mixed-width ADD. Its runnable mixed-width numericEqual, Float-plus-exact, and direct Float-to-Decimal/Int admission refusals under I(0,0,0,0) are unobserved, although the test name and SC-04 trace imply the F20 admission row. | tests/ieee_source_agreement.rs:902-954 |
| FND-005 | medium | F17 denial assertions omit the shortage limit and next_charge, the complete consumed counters, and the stop-prefix comparison required for a named charge denial; the test can pass with a wrong shortage record. | tests/ieee_source_agreement.rs:717-784 |

## Coverage

Examined the full PR diff: Cargo.lock and the two new IEEE test files. Checked IT-017-SC-01 through SC-05 and TC-097 against QSpec TC-193 F01-F34, with focused attention to F10/F13/F17/F20 and pending producer subcases. No CI workflow, production API, vendored corpus, or copied artifact changed. Static `git diff --check` and `cargo fmt --all -- --check` passed. Focused compilation and local tests were blocked before test code by IR-1284 (QSL/quire-eval API skew).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | The exhaustive runner marks F01/F02 and F17 Partial but invokes their full charge-agreement tests. Current locked QSL has no IEEE equality metering (IR-1282), so these ticket-backed PENDING cases fail the F-only suite after the unrelated compile repair and prevent landing runnable F rows independently. | tests/ieee_source_agreement.rs:1554-1595 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | The new F01/F02 value-only `#[test]` still traces IT-017-SC-03, although it checks no meter charge, counter, or denial; its runnable trace falsely claims accounting evidence while that subcase is PENDING IR-1282. | tests/ieee_source_agreement.rs:774-788 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `f_case` exhaustively matches published F IDs, panics on unknown IDs, and the runner iterates QSpec vectors. |
| FND-002 | fixed | F10 now runs the I(64,2,4,1) tuple and injects a named result-retain denial through the bounded comparison helper. |
| FND-003 | fixed | F13 now runs an immediately following exact no-flag ADD after divide-by-zero cases. |
| FND-004 | fixed | F20 now checks mixed-width equality, mixed-width ADD, Float plus exact integer, and direct Float to Decimal/Int source admissions with a zero-limit runtime meter. |
| FND-005 | fixed | F17 now asserts shortage limit, next charge, expected consumed amounts, complete counters, and ordered admitted charge prefix. |
| FND-006 | still-open | The Partial runner still calls the blocked F01/F02 and F17 metering assertions; preserve them as PENDING or split value-only runnable assertions from the blocked charge assertions so the F-only suite can pass without IR-1282. |
| FND-006 | fixed | F01/F02 and F17 runnable paths now compare values only; strict metering/denial functions remain reachable in the `Partial` record but are not executed until IR-1282. |
| FND-007 | still-open | Remove IT-017-SC-03 from the F01/F02 value-only test trace; keep SC-03 on the later activated strict metering test when IR-1282 lands. |
| FND-007 | fixed | F01/F02 value-only test now traces SC-01/SC-02 only; the inventory runner traces SC-01 only, while active charge tests retain SC-03. |

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | F20 source admission helper accepts any `IllTyped` cause for the newly added mixed-width totalOrder and bitIdentical calls. FR-397-AC-24 requires `ill_typed/type-mismatch`; a different ill-typed failure would satisfy this test, so it does not prove the specified refusal. | tests/ieee_source_agreement.rs:1304-1311,1323-1325 |

## Current review pass

At the reviewed draft head, static `git diff --check` passed. Reviewed the new FMA, ordering, identity, extrema, F20 admission, F34 width paths and exhaustive F dispatcher against QSpec TC-193. FMA and callable comparisons make independent QSL and RT observations of values, flags/losses and ordered scalar charge suffixes; F34 compares values at both widths. Prior FND-001 through FND-007 remain fixed. Focused compilation and final gate remained unverified because the quire-driver API was skewed (IR-1277).

## Disposition pass 5

| FND | outcome | reason |
| --- | --- | --- |
| FND-008 | fixed | The F20 mixed-width equality, totalOrder, bitIdentical, and addition source cases now require CheckCause::IllTyped(IllTypedCause::TypeMismatch). |

## Disposition pass 6

No new findings in the reviewed delta. The new QSL/driver/IR API adaptation, exact F20 admission cause, and active F01/F02/F17 metering paths have no new finding in the changed code. The focused test evidence is 19 passed and zero failed; the final aggregate gate is reserved for the code lead.

## New findings (disposition pass 7)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | The committed review artifacts leave the initial FAIL verdict and superseded IR-1284/IR-1277/IR-1282 blockers as the latest visible review status. They do not record the now passing focused F and IR-914 checks or that IR-1301 alone holds the aggregate gate. Add a current status appendix to both artifacts while preserving the historical finding and disposition text. | reviews/SR-16369-ir-669-pr37-code-review.md:14-16,64-66; reviews/SR-16370-ir-669-pr37-gap-analysis.md:42-44 |

## Disposition pass 7

The new driver forwarding and lockfile relock contain no code, Rust, or trace finding. IR-914 strict sibling assertion remains intact. Focused IR-914 and IEEE F tests passed in the coding lane; final aggregate gate remains open on IR-1301. FND-009 remains open for the committed review artifact status.

## Current status

The initial FAIL verdict and earlier blocker statements above record their historical review passes. The reviewed Rust and test-code findings are fixed; FND-009 identified this missing current-status record, which this appendix supplies for reviewer disposition. The strict `real_qsl_export_refusal_is_withheld_before_codegen` check passes (1 passed), and `cargo test --test ieee_source_agreement` passes (19 passed, zero failed or ignored). Scoped Clippy passed for both touched test targets with warnings denied. IR-1301 holds the final aggregate gate because QSL still resolves two transitive filament-core-data revisions. The F conversion and negation portions remain ticket-backed PENDING under IR-1283; they are outside the runnable F slice.

## Disposition pass 8

| FND | outcome | reason |
| --- | --- | --- |
| FND-009 | fixed | Both committed review artifacts have current-status appendices; the historical findings and blocker statements remain identifiable as earlier review results. |

## Disposition pass 9

No new finding at the frozen PR head. The final delta updates six F and three IR-914 SourceIdentity constructor calls to the current two-argument API and normally refreshes the first-party dependency cohort. Checked that F agreement helpers still assert QSpec outcomes, independent QSL/runtime equality, ordered charge suffixes, and consumed counters; F20 requires the precise ill-typed admission cause with zero runtime charges. The exhaustive dispatcher still panics on an unknown F ID and routes conversion/negation work to IR-1283 as PENDING or partial. The lead reported focused dependency_cohort 1/1, IEEE F 19/19, IR-914 strict regression 1/1, touched Clippy/fmt/string-edge clean. These results are reviewed evidence, not tests rerun by this reviewer. The lead owns the final aggregate gate.
