---
id: SR-2026110414560041
title: "IR-1283 IEEE source agreement gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR #52 8d9c4e76f9a57de8f1867574eaf3da9d91a34ef0; tests/ieee_source_agreement.rs against IT-017-SC-01/02/03 and QSpec TC-193 F24/F25/F26/F27/F28/F31/F32/F33"
review_set: subset
---

# IR-1283 IEEE source agreement gap analysis

## Summary

Ticket: IR-1283. `quire matrix --scope . --format json` returned only engine metadata because this integration module has no declared traceability model; the reviewed test-to-criterion bindings were checked manually. The F-row dispatcher now runs all eight targeted QSpec IDs and the F26 signed-zero conversion subcase, with no PENDING arm. The new tests use real QSL check, link, evaluation and emitted carrier paths plus the owned Exact operations. Each changed test carries IT-017 trace tags. No production source changed, so the reverse code-to-spec check found no new orphan production behavior. Plan completion: not assessed. Optional repository-wide semantic review was not requested; this is a changed-slice acceptance check.

## Verdict

**CONDITIONAL.** The changed slice leaves the explicit Debug-rendering portion of IT-017-SC-02 unasserted. The separate denial-field finding is in the code review artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new F24–F33 value branches compare typed values and bits but never compare the completed values' Debug rendering, although IT-017-SC-02 requires both. A changed QSL or Exact Debug representation with unchanged value equality would pass these tests. Assert rendering equality for completed QSL and native results in the shared helpers and F32/F33 direct branches. | tests/ieee_source_agreement.rs:410-419,492-508,1740-1836,1892-1901; spec/integration/IT-017-runtime-scalar-agreement.md:90-108 |

## Coverage

The F24/F25/F26/F27/F28/F31/F32/F33 routes have value-asserting tests and explicit trace attributes. Quire matrix has no local traceability model, so no computed binding verdict is available; manual bindings below are limited to this diff. SC-02 is partial for Debug rendering; SC-03 is partial for denial fields (code review FND-001). Plan completion: not assessed.

## Examined scope

```yaml
scope:
  - id: IT-017-SC-01
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every row offered by QSpec's scalar-vector API reaches its specified evaluation or admission path, or is named PENDING with an open upstream owner ticket and the exact missing path."
  - id: IT-017-SC-02
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every runnable evaluation case has a real two-sided observation and the same specified value or noncompleted result."
  - id: IT-017-SC-03
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every runnable authority-metered evaluation and required sweep agrees in charge order, amounts, counters, denial fields, and stop prefix."
  - id: TC-193-F24/F25/F26/F27/F28/F31/F32/F33
    path: spec/test-cases/TC-193-ieee-exceptional-and-rounding-profiles.md
    role: context_only
    excerpt: "Negate binary32 `0x3f800000` under `ScalarLimitsV1 { integer_bits: 32, decimal_digits: 0, scale_expansion: 0, text_input_bytes: 0, text_scalars: 0, normalized_scalars: 0, unit_edges: 0, value_occurrences: 1, work_units: 2, result_units: 1 }`"
  - id: tests/ieee_source_agreement.rs
    path: tests/ieee_source_agreement.rs
    role: examined
    excerpt: "fn tc193_f32_ieee_negation_preserves_bits_and_rounding_independence() {"
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: IT-017-SC-02
    related_ids: [IT-017-SC-03]
    path: tests/ieee_source_agreement.rs
    lines: "410-419,492-508,1740-1836,1892-1901"
    excerpt: "every runnable evaluation case has a real two-sided observation and the same specified value or noncompleted result."
    finding: "Completed F24-F33 comparisons never assert the Debug rendering required by IT-017 step 2."
bindings:
  - {test_id: tc193_f24_f25_f26_f31_width_conversions_agree, ac_id: IT-017-SC-02, trace: correct}
  - {test_id: tc193_f24_f27_f28_exact_conversions_agree, ac_id: IT-017-SC-02, trace: correct}
  - {test_id: tc193_f32_ieee_negation_preserves_bits_and_rounding_independence, ac_id: IT-017-SC-02, trace: correct}
  - {test_id: tc193_f33_negation_meter_agrees, ac_id: IT-017-SC-02, trace: correct}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Completed float and rational shared helpers, F32 direct and double negation paths, binary64 paths, and F33 direct branch now assert equality of QSL and runtime Debug rendering. Verified at PR #52 head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`. |

## Final disposition

**PASS.** FND-001 is fixed at PR #52 head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`; no new finding in the fix diff.

## Lock refresh verification

At PR #52 head `b4ca5a2e81218630429acba25652183c6c23dca4`, the IEEE test diff is byte-identical to the reviewed head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`; the only relevant dependency delta is the first-party QSL/IR lock refresh. All previously checked F-row dispatch, two-sided rendering, and IT-017 trace bindings remain present. The existing FND-001 disposition remains fixed; no new gap finding. **PASS** for this confined refresh review.
