---
id: SR-20261104130004
title: "IR-1312 code-review of Decimal source agreement"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1312; spec/integration/IT-017-runtime-scalar-agreement.md; tests/scalar_agreement_decimal.rs"
review_set: base
---

## Summary

D04 preserves the independent original RT type and (125,3)/no-loss assertion; its source type admits both original retained inputs. D05 exact/rounded, D09 successful/injected denial, D14 refusal/work denial, and D17 retained scale execute both real paths. Existing agreement helpers compare exact outcomes, typed losses, ordered scalar charges and every meter counter; only the existing function.call work-unit offset is applied. No original oracle is weakened, no QSpec corpus/type authority is copied, and no gate changes are present.

## Examined scope

```yaml
scope:
- id: IT-017-SC-02
  path: spec/integration/IT-017-runtime-scalar-agreement.md
  role: examined
  excerpt: 'IT-017-SC-02: every runnable evaluation case has a real two-sided observation
    and the same specified value or noncompleted result. A case that cannot execute
    remains PENDING under SC-01 and is not agreement.'
- id: IT-017-SC-03
  path: spec/integration/IT-017-runtime-scalar-agreement.md
  role: examined
  excerpt: 'IT-017-SC-03: every runnable authority-metered evaluation and required
    sweep agrees in charge order, amounts, counters, denial fields, and stop prefix.'
- id: TC-185-D04
  path: spec/integration/IT-017-runtime-scalar-agreement.md
  role: examined
  excerpt: result `(125,3)`, absence of loss, ordered charges, and counters unchanged.
- id: TC-185-D05/D09/D14/D17
  path: spec/integration/IT-017-runtime-scalar-agreement.md
  role: examined
  excerpt: Preserve each operand's coefficient and scale; compare exact-mode refusal,
    rounded value and typed loss, ordered charges and counters, and each authored
    denial.
- id: qsl_decimal_literal_with_denial
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: "fn qsl_decimal_literal_with_denial(\n    body: &str,\n    result_type:\
    \ &str,\n    limits: ScalarLimits,\n    denial: Option<quire_exact::InjectedDenial>,\n\
    ) -> Result<(Outcome<Value>, Vec<ValueLoss>, Meter), Vec<CheckRefusal>> {"
- id: literal_division_agreement
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: "fn literal_division_agreement(\n    body: &str,\n    result_type: &str,\n\
    \    left: &Decimal,\n    right: &Decimal,\n    target: &DecimalType,\n    limits:\
    \ ScalarLimits,\n) {"
- id: d04
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: fn d04() {
- id: d05
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: fn d05() {
- id: d09
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: fn d09() {
- id: d14
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: fn d14() {
- id: d17
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: fn d17() {
- id: assert_decimal_agreement
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: "fn assert_decimal_agreement(\n    kernel: Outcome<DecimalResult>,\n  \
    \  kernel_meter: &Meter,\n    qsl: Outcome<Value>,\n    losses: &[ValueLoss],\n\
    \    qsl_meter: &Meter,\n) {"
- id: assert_scalar_meter_agreement
  path: tests/scalar_agreement_decimal.rs
  role: examined
  excerpt: 'fn assert_scalar_meter_agreement(qsl: &Meter, kernel: &Meter) {'
```

Code scope: qsl_decimal_literal_with_denial, literal_division_agreement, tc185_decimal_vectors dispatch/trace tags, d04/d05/d09/d14/d17, and existing assert_decimal_agreement/assert_scalar_meter_agreement. D12/D06/D15/D13/D18 were context_only for unchanged dispositions.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the changed scope. Reviewed coder focused test and Clippy logs (both successful); no Cargo, aggregate, or Kani commands rerun, as instructed. This is a scoped review, not full IT-017 completion. D06/D15 remain QSL-1393 pending; D12 remains outside this admission claim; D13/D18 remain evaluator-only. No applicable AssuranceProfile was found under spec/. Gap analysis skipped because no production code changed. Extra spec lenses skipped: no new requirement grammar, artifact structure, domain object, or relationship edge changed.
