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
