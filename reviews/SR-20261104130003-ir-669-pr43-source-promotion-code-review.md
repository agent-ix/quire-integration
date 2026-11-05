---
id: SR-20261104130003
title: "IR-669 PR 43 Decimal source promotion review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock and tests/scalar_agreement_decimal.rs; IT-017-SC-01 through IT-017-SC-05; QSpec TC-185 D25-D28"
review_set: subset
---

## Summary

Reviewed PR #43 at `ea96e27043220c4bad4487221e9196c1ff1566d6`, focusing on the new Decimal literal source checks after QSL began admitting the forms. The locked QSpec TC-185 D25-D28 rows and IT-017-SC-01 through SC-05 are the behavioral authority. The prior SR-16375 review covered the earlier S2-pending code head; this review covers the later source promotion.

## Verdict

CONDITIONAL. The D26/D27 two-sided value, loss, ordered-charge, counter, and denial comparisons exercise the authored paths. Two test defects remain in the new source-routing assertions.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | D25 and D28 accept a list containing the expected refusal alongside other wrong refusals, so they do not assert the exact admission decision required by IT-017-SC-04 and TC-185. | tests/scalar_agreement_decimal.rs:1358; tests/scalar_agreement_decimal.rs:1526 |
| FND-002 | low | The dispatcher still labels D25-D28 source arms PENDING at S2 even though these arms now execute S4 and D26/D27 execute S6; the coverage comment misstates the current route. | tests/scalar_agreement_decimal.rs:396 |

## Coverage

- D25: explicit conversion and mixed named profiles are asserted at S4. Both ambiguous literal bodies and two admitted literal bodies now reach S4. The ambiguous-refusal test uses `any` rather than checking the complete refusal result.
- D26: both declared result modes compare QSL and RT outcomes, typed loss, ordered charges, counters, and work/integer-bit denials. The retained `(100,2)` representation and no-rounding behavior have RT oracles.
- D27: QSL and RT compare the rounded result, rational loss, charges, counters, and scale-expansion denial. The RT oracle independently checks the authored coefficient and exact loss.
- D28: both type-mismatch sources reach S4 and the D26 counterpart reaches S6. The mismatch test uses `any` rather than checking the complete refusal result.
- The diff changes an integration test and the QSpec lock entry only. No production code, spec/plan file, hosted workflow, duplicate corpus, or new pin changed. The QSpec lock advances from `8f46a3af` to descendant `860e0890`.
- `git diff --check` passed. The coder reported focused Decimal test 1 passed with 2 existing QSL-1393 tests ignored, plus scoped formatting and Clippy. I did not run the aggregate gate; the lead owns the final-head gate.

## Assurance Context

No applicable AssuranceProfile is declared under this repository's `spec/` tree.

## Dispositions

| FND | outcome | reason |
|---|---|---|
| FND-001 | fixed | Commit `45dd7cb` requires exactly one S4 refusal for each D25 and D28 adverse body and checks its typed cause. |
| FND-002 | fixed | Commit `45dd7cb` replaces the stale S2 PENDING comments with the actual S4 admission and S6a agreement routes. |

The disposition pass reviewed `ea96e270..45dd7cb` at final head `45dd7cb7423796dd2012327730f7f57aa254d2de`. No new finding was found. `git diff --check` passed; the lead owns the final aggregate gate.
