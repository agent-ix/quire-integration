---
id: SR-2026110414560092
title: code-review review of IR-842 PR 74 quantity refusals
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-integration; PR #74; spec/test-cases/TC-108-tc187-quantity-refusals.md,
  tests/tc187_p8_static_refusals.rs'
review_set: subset
---
## Summary

Ticket: IR-842. Reviewed PR #74 bounded U03/U05/U09 refusal cohort.

## Verdict

PASS — no findings in the examined scope.

Rust lane and language-independent integrity review found no defect. The three public API tests cover twelve source cases: U03, both affine units crossed with all five operations, and U09. Every case demands the entire singleton typed checking refusal, function/body path and exact source region. Native calls use admitted UnitGraph units and their original IDs, exercise both allowance extremes and require exact typed refusal, no ordered charges and ten zero counters. U09 obtains a distinct authentic model owner via public intake and inspects both same-named base declarations, retained maps, scale, offset and roots. No checked JSON, fake static-check meter, copied evaluator or generated comparison appears.

Searched existing TC-187 tests and support helpers: the new setup follows existing dependency-resolved model-fixture access, without adding a fixture copy or alternative owner semantic implementation. The owner model document is authored in memory before admission. Existing private test helpers are not a shared callable fixture API. Operator matching is exhaustive; test-owned invariant panics cannot be reached by production input. No unsafe, concurrency, public production API, workflow, manifest or lock change occurs.

The supplied focused execution receipt reports three tests passed (all twelve cases); supplied scoped Clippy completed with warnings denied, formatting and TC validation passed. These are coder receipts inspected by this reviewer, not independent executions. No build, test gate, census, Kani or optional semantic fanout was run during review; the lead retains the final scoped gate. Native owner implementation was inspected to establish that type checking precedes evaluation and charging.

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
