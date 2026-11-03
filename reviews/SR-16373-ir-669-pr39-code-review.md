---
id: SR-16373
title: "IR-669 PR 39 U-row code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock, tests/quantity_source_agreement.rs, tests/scalar_agreement_quantity.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-187 U01 through U39"
review_set: subset
---

## Summary

Ticket: IR-669. Reviewed draft PR #39 at ccad0b4841a63c2159865c0f05d6f484a72f5226 against IT-017 and QSpec TC-187. The new carried cases use a real checked QSL package and the public quantity runtime. The PR is explicitly a partial U slice. Compilation stops upstream in IR-1284 before these tests compile.

## Verdict

FAIL for the current trace and accounting claims. One high-severity family coverage claim and two medium-severity accounting evidence gaps remain. No passing test or aggregate gate is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new carried tests and preserved quantity tests trace IT-017-SC-01, which requires every QSpec TC-187 ID to execute or have an exact ticket-backed upstream PENDING route. `selected_quantity_row` only validates the six named carried IDs and the other tests filter selected IDs; runnable U02, U09, U12 and other rows are silently absent. A new QSpec U ID would pass this suite. This partial PR may remain partial, but it cannot claim SC-01 until an exhaustive executable dispatcher exists. | tests/scalar_agreement_quantity.rs:370; tests/scalar_agreement_quantity.rs:507; tests/quantity_source_agreement.rs:36 |
| FND-002 | medium | U01's test traces IT-017-SC-03 but creates temporary QSL and runtime meters and discards them. It compares values only, so a changed charge sequence or counter on either side passes despite the charge-agreement trace. | tests/quantity_source_agreement.rs:36-173 |
| FND-003 | medium | The U16/U17/U18/U29 limit tests compare QSL and runtime meter sequences with each other and assert the final shortage fields, but they do not assert QSpec's required admitted charge prefix. For example, U16 requires one read, one edge and two rational events before the exact refusal; replacing an earlier point with another equal-cost point on the shared runtime path can pass both sides. | tests/scalar_agreement_quantity.rs:385-475; tests/scalar_agreement_quantity.rs:555-700 |

## Coverage

Examined the complete PR diff: Cargo.lock and both new quantity test files. Checked IT-017-SC-01 through SC-05 and QSpec TC-187 U01 through U39, especially U07/U08/U16/U17/U18/U29 carried conversion and denial paths. The tests select QSpec rows through its public API; no copied corpus, vendored source, production API change, or CI workflow change appears in the diff. The carried helper compares full typed QSL and runtime outcomes, losses, ordered charges and counters after the separately asserted function.call envelope. Static `cargo fmt --all -- --check` and `git diff --check` pass. `quoin matrix --repo ... --json` reports no coverage_matrix criteria because the installed module has no obligations source; the trace judgment used the spec and test source directly. Focused compilation and the final gate are blocked by QSL/quire-eval IR-1284, so test behavior has not been verified on this head.
