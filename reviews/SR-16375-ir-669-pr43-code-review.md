---
id: SR-16375
title: "IR-669 PR 43 decimal refresh code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; Cargo.lock and tests/scalar_agreement_decimal.rs; IT-017-SC-01 through SC-05; QSpec TC-185 D01-D28"
review_set: subset
---

## Summary

Reviewed PR #43 at final documentation head 4aeebe140cd2aa82782433878fa01dbfdf6c0640. Code was reviewed at 7c22cc46376f04794b1a6832e9bb84c2913549e2 against current integration main, QSpec TC-185, QSL source behavior, and structured ticket states. The change updates Decimal composition for QSpec #616: D24 uses the authored Rational-to-Decimal conversion; D26/D27 keep their RT evaluator controls while the corresponding literal source arms are explicitly checked at S2 and remain pending under QSL-1394; D28 similarly records the precise S2 blocker before its required S4 type mismatch can run. D13/D18 remain RT-only evaluator-boundary vectors by definition. Existing unguarded division rows remain QSL-1393 pending.

## Verdict

PASS. No code-review finding in the reviewed diff. The final aggregate gate remains the lead's responsibility.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-000 | low | No findings in the reviewed diff. | — |

## Coverage

- D24: the RT division oracle and QSL Rational parameter conversion compare result representation and loss. The cases exercise nearest-even success, scale-expansion denial, and exact-mode refusal. The shared agreement helper compares outcome/loss and ordered meter records/counters, including QSL's function-call work offset.
- D26: both toward-zero and exact RT multiplication outcomes preserve the retained `(100,2)` coefficient/scale behavior, assert no rounding charge, and check result-retain and arithmetic denial boundaries. The QSL literal arms assert the exact `UnrepresentedConstruct(ExactNumber)` S2 refusal under QSL-1394.
- D27: the RT division asserts rounded representation, exact rational loss, mode/scale, work charge, and scale-expansion denial. Its QSL literal arm asserts the same typed S2 blocker.
- D28: all three published source bodies assert the exact S2 `UnrepresentedConstruct(ExactNumber)` result. No S4 type-mismatch claim is made before that source stage succeeds.
- D25: existing named-type admission and mixed-profile type mismatch remain executable. The four decimal-literal source arms assert the typed S2 blocker.
- D13 and D18: QSpec labels these evaluator-boundary cases because their operands are outside the target membership domain/scale. RT oracles remain runnable; the comments no longer claim QSL source is pending for these rows.
- Division D04/D05/D06/D09/D12/D14/D15/D17 remain explicit QSL-1393 pending while their RT oracles remain intact.
- The vector dispatch enumerates D01 through D28 and panics for an unknown ID. The test obtains vectors from the locked QSpec crate; no copied vector catalog or changed Cargo manifest is introduced.
- Diff is limited to Cargo.lock and the Decimal integration test. No production code, hosted workflow, compatibility shim, or vendored material changed.

Evidence supplied by the coder and read back from the branch: `cargo test --test scalar_agreement_decimal` passed 1 test, 0 failed, with 2 existing QSL-1393 probes ignored; scoped Clippy with warnings denied, rustfmt, and `git diff --check` passed. I did not run heavy gates.

## Tracker note

QSL-1619 intentionally remains In Progress for PR #2277, covering D25/D26/D27/D28 source forms. QSpec #616 resolves D13/D18/D24. The final PR body and IR-1312 comment 94f054e1 cite QSL-1394/QSL-1619 for the pending source forms and QSL-1393 for division rows. The code review remains unchanged; the final head adds documentation only.
