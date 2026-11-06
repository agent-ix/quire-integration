---
id: SR-2026110414560093
title: spec-review/base review of IR-842 PR 74 quantity refusals
type: SpecReview
analysis: base
scope: 'agent-ix/quire-integration; PR #74; spec/test-cases/TC-108-tc187-quantity-refusals.md,
  tests/tc187_p8_static_refusals.rs'
review_set: base
---
## Summary

Ticket: IR-842. Reviewed PR #74 bounded U03/U05/U09 refusal cohort.

## Verdict

PASS — no findings in the examined scope.

The new TC has a unique identifier, concrete procedure and falsifiable expected results, and its IT-019 reference resolves to the existing integration scenario. Source checking and supplementary native-value evidence are explicitly separate. Exact static causes account for the power distinction; U09 requires authentic distinct identity despite equal names and shape. Test procedure and expected results match all three executable tests. SC-04 stays Planned and no runtime Outcome or generated-oracle agreement is claimed.

This change adds a bounded test case, not a new FR, StR, NFR, domain object, prerequisite edge or allocation boundary. US/FR-specific format and coverage rules are inapplicable. No applicable AssuranceProfile was found under spec/. Base review is sufficient; EARS, integrity, dependency, object and other requirement lenses are not applicable to this TC-only addition. Gap-analysis is not applicable because there is no production-code change. No standalone matrix/census was run under the dispatch constraint; trace bindings were manually inspected. No hash, digest, pin, file catalog or tracking mechanism was added. The existing IT-019 reference does not imply completed SC-04 coverage.

## Examined scope

```yaml
scope:
- id: TC-108/Description
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: 'Exercise public TC-187 U03, U05 and U09 through genuine QSL source acquisition.

    Keep source checking refusals distinct from supplementary native value refusals.'
- id: TC-108/Test-Procedure
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: "1. Bind each public vector's complete input and expected strings by reference.\n\
    \   Parse, form and assemble original source; retain the admitted unit graph and\n\
    \   original unit identities. For U09 admit a selected domain model through public\n\
    \   model intake, independently of the local dimension owner."
- id: TC-108/Test-Procedure/2
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: "2. Check incompatible length/time addition, every combination of affine\
    \ Celsius\n   and Fahrenheit with addition, subtraction, multiplication, division\
    \ and power,\n   and addition of roots whose base dimensions share the name L\
    \ but have distinct\n   actual semantic keys. Require exact typed causes and complete\
    \ source locations."
- id: TC-108/Test-Procedure/3
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: "3. Independently invoke the public native quantity operation using the\
    \ assembled\n   units and original unit IDs, under ample and zero allowances.\
    \ Require the exact\n   incompatible-dimension or affine-arithmetic refusal, an\
    \ empty ordered charge\n   sequence and zero consumption for all ten counters.\
    \ Inspect the two U09\n   dimension maps, equal names, unit scales and offsets\
    \ without constructing keys."
- id: TC-108/Expected-Results
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: 'U03 and U09 refuse source checking with type-mismatch. All ten U05 cases
    refuse:

    ordinary arithmetic has type-mismatch and power has operator-ineligible. Each

    refusal identifies the original arithmetic expression and owning function.

    The separate native operation preserves IncompatibleDimensions or

    AffineUnitArithmetic before charging even when every allowance is zero.'
- id: TC-108/Expected-Results/2
  path: spec/test-cases/TC-108-tc187-quantity-refusals.md
  role: examined
  excerpt: 'These observations establish source admission/refusal and native value
    refusal

    precedence only. Invalid source produces no checked selected operation, no

    runtime Outcome and no generated-oracle agreement. SC-04 remains Planned beyond

    this bounded cohort; model normalization and checking limits are separate from

    value accounting.'
- id: incompatible_length_and_time_refuse_at_original_addition
  path: tests/tc187_p8_static_refusals.rs
  role: examined
  excerpt: fn incompatible_length_and_time_refuse_at_original_addition()
- id: all_ten_affine_arithmetic_cases_refuse_before_value_charges
  path: tests/tc187_p8_static_refusals.rs
  role: examined
  excerpt: fn all_ten_affine_arithmetic_cases_refuse_before_value_charges()
- id: same_named_base_dimensions_keep_distinct_actual_owners
  path: tests/tc187_p8_static_refusals.rs
  role: examined
  excerpt: fn same_named_base_dimensions_keep_distinct_actual_owners()
bindings:
- test_id: incompatible_length_and_time_refuse_at_original_addition
  ac_id: TC-108
  trace: correct
- test_id: all_ten_affine_arithmetic_cases_refuse_before_value_charges
  ac_id: TC-108
  trace: correct
- test_id: same_named_base_dimensions_keep_distinct_actual_owners
  ac_id: TC-108
  trace: correct
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
