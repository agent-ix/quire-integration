---
id: SR-16369
title: "IR-669 PR 37 F-row code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock, tests/ieee_source_agreement.rs, tests/scalar_agreement_ieee_text.rs; IT-017-SC-01 through IT-017-SC-05, TC-097"
review_set: subset
---

## Summary

Ticket: IR-669. Reviewed draft PR #37 at 7624808b against IT-017 and QSpec TC-193. The tests exercise real checked QSL and runtime paths, but selected runnable obligations and the row-inventory guard are missing.

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

Examined the full PR diff: Cargo.lock and the two new IEEE test files. Checked IT-017-SC-01 through SC-05 and TC-097 against QSpec TC-193 F01-F34, with focused attention to F10/F13/F17/F20 and pending producer subcases. No CI workflow, production API, vendored corpus, or copied artifact changed. Static `git diff --check` and `cargo fmt --all -- --check` passed. Focused compilation and local tests are blocked before test code by IR-1284 (QSL 1d8893f6 / quire-eval 4e59f729).

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

At frozen head 666a460c (QSpec b1feb484), static `git diff --check` passed. Reviewed the new FMA, ordering, identity, extrema, F20 admission, F34 width paths and exhaustive F dispatcher. FMA and callable comparisons make independent QSL and RT observations of values, flags/losses and ordered scalar charge suffixes; F34 compares values at both widths. Prior FND-001 through FND-007 remain fixed. Focused compilation and final gate remain unverified because the pinned quire-driver API is skewed (IR-1277).

## Disposition pass 5 (6bd56e51)

| FND | outcome | reason |
| --- | --- | --- |
| FND-008 | fixed | The F20 mixed-width equality, totalOrder, bitIdentical, and addition source cases now require CheckCause::IllTyped(IllTypedCause::TypeMismatch). |
