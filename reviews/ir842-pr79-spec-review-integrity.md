---
id: SR-2026110414560119
title: IR-842 PR 79 spec-review-integrity review
type: SpecReview
analysis: integrity
scope: 'agent-ix/quire-integration; PR #79; spec/test-cases/TC-109-tc187-quantity-conversion.md,
  tests/tc187_p8_conversion.rs'
review_set: subset
---

## Summary

Reviewed PR #79 for IR-842, limited to TC-109 and tests/tc187_p8_conversion.rs. No applicable AssuranceProfile is declared in the reviewed repository. Original source/native observations establish no generated execution credit; generated agreement remains Planned pending shared runner custody. No source/native owner blocker was found. No production code, workflow, shared runner, dispatcher, IT-019, dependency manifest or lock is changed.

The four tests contain eighteen paired operation observations: nine affine/alias observations, three U10 observations, and six compound/power observations. The additional normalization test exercises owner dimension maps directly. Both independently admitted legs use the same underlying quantity kernel; this is integration-path agreement, not an independent generated implementation claim.

## Verdict

PASS — no defects found in the scoped source/native cohort.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The new TC's sections and relationships are coherent. Description, five procedure steps and expected results all preserve the original-source/native-only boundary. Four tests and eighteen paired observations match the stated coverage. U10's exact healthy and denied payload/counter expectations match assertions; source stop locations and no partial quantity are checked. U02 distinguishes native dimension normalization from source-derived dimension declaration representation, and requires original checked quantity subtrees rather than a fabricated declaration.

Identity, purity, metering and topological failure probes were considered within this cohort: independent admission retains original keys; no effect adapter or external side effect is introduced; fresh meters isolate operations; native scope only registers real owner-returned compound units. Source/native shared kernel dependence is not described as generated independence. No FR/StR/NFR traceability chain changes; broader completeness is not claimed.

## Examined Units

```yaml
scope:
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: '# TC-109: Original quantity conversion and native dimension normalization'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: '## Description'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: 'Exercise public TC-187 U02, U04, U09b and U10 through original checked
    source

    and native owner operations. Generated agreement remains Planned pending shared

    runner custody. Source and native observations alone give no generated credit.'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: '## Test Procedure'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: "1. Bind each public vector's complete input and expected cells by reference.\n\
    \   Parse, form, assemble, check and link original source. Select each original\n\
    \   checked function body by its actual function identity. Evaluate that body\
    \ on\n   a fresh operation meter without a function-call envelope.\n2. Independently\
    \ admit identical source through the public replay facade. Resolve\n   real source\
    \ quantities through its unit table. Convert through the native\n   owner API\
    \ with fresh identical limits. Compare full outcomes, losses, ordered"
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: "admitted charge records and all ten counters, and check source locations.\n\
    3. Require exact 5463/20 K from original 0 degC and 32 degF; require exact 7 m\n\
    \   from a distinct scale-one alias while preserving distinct original unit IDs.\n\
    4. For 100 cm to m use U10's exact tuple. Require 1 m, six ordered charges,\n\
    \   all counters, and the eight-bit multiply amount. Inject denial specifically\n\
    \   at the first UnitResultRetain event on fresh real meters; require the exact\n\
    \   prior prefix, no partial quantity, and an independent healthy sibling."
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: "5. Normalize real admitted L and T dimension maps with owner multiplication,\n\
    \   division and zero power. Require L and the empty sorted map, with zero\n \
    \  exponents absent. Select original checked quantity product/division/power\n\
    \   subnodes through their authored equality operand and typed children. Compare\n\
    \   evaluation of the original quotient subtree with native multiplication then\n\
    \   division on the same meter, registering the actual returned compound unit.\n\
    \   Compare original zero-power evaluation with native power. Do not fabricate"
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: source dimension declarations or replacement unit identities.
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: '## Expected Results'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: '`tc187_p8_conversion` passes four tests, including eighteen complete source/native

    operation observations. U04''s two original affine conversions return exact

    5463/20 K; U09b returns exact 7 m with distinct original unit keys under the same

    native dimension node. Each also agrees at work 0 and 1, stopping at the original

    conversion''s identity-read and edge respectively, with no partial quantity.'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: 'U10 returns exact 1 m under the public tuple. Its full ordered charge
    records are

    identity-read (integer bits 7, occurrence 1), edge (edge 1), rational multiply

    (bits 8), rational add (bits 3), target-domain (no size request), and result-retain

    (occurrence 1). Every other size request is absent. All ten counters in public

    LimitKind order are [8,0,0,0,0,0,1,1,6,1]. Named final denial reports WorkUnits,

    limit 5, consumed 5, next charge 1 at UnitResultRetain; the entire admitted prefix

    matches the first five healthy records, counters are [8,0,0,0,0,0,1,1,5,0], and'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: 'no quantity is returned. A fresh independent sibling completes with the
    original

    healthy ledger. Source and native outcomes and full accounting agree in each case;

    source stops retain the original operation location and exact paths have no loss.'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: 'U02 normalizes genuine admitted L/T dimension maps to L and the empty
    map with

    no zero exponents. Original closed `(1 m * 1 s) / 1 s` and `1 m ^ 0` subtrees

    agree with native owner composition at work 0, 1 and 1024. Successful magnitudes

    are exact 1 and compound unit identities retain respectively only the original

    metre root at exponent 1 and no terms. Genuine original checked types and native

    returned identified units agree. Stops preserve original product or power

    locations and full ledgers and counters.'
- id: TC-109
  path: spec/test-cases/TC-109-tc187-quantity-conversion.md
  role: examined
  excerpt: 'This establishes source/native coverage only. Generated execution remains

    Planned pending shared runner custody. Native normalization does not claim

    source-derived zero-dimension declaration representation support.'
- id: bind
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn bind(id: &str, input: &str, expected: &str) {'
- id: rational
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn rational(n: i64, d: i64) -> Rational {'
- id: admitted
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn admitted(source: &str) -> AdmittedContext {'
- id: limits
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn limits() -> ScalarLimits {
- id: u10_limits
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn u10_limits() -> ScalarLimits {
- id: counters
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn counters(meter: &Meter) -> [u64; 10] {'
- id: conversion
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn conversion(
- id: charge
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn charge(
- id: conversion_stops
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn conversion_stops(declarations: &str, from: &str, to: &str, n: i64)
    {'
- id: completed
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: 'fn completed(outcome: Outcome<Value>, context: &AdmittedContext, target:
    &str, expected: Rational) {'
- id: original_affine_conversions_and_scale_one_alias_match_native_exactly
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn original_affine_conversions_and_scale_one_alias_match_native_exactly()
    {
- id: original_u10_exact_tuple_and_named_final_denial_keep_full_prefix
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn original_u10_exact_tuple_and_named_final_denial_keep_full_prefix() {
- id: native_normalization_uses_original_admitted_dimension_maps
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn native_normalization_uses_original_admitted_dimension_maps() {
- id: original_quantity_subnodes_preserve_native_normalized_compound_identities
  path: tests/tc187_p8_conversion.rs
  role: examined
  excerpt: fn original_quantity_subnodes_preserve_native_normalized_compound_identities()
    {
- id: IT-019-SC-04
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: context_only
  excerpt: '   IT-019-SC-04: every API row and required operation has one explicit'
bindings:
- test_id: original_affine_conversions_and_scale_one_alias_match_native_exactly
  ac_id: QSpec-TC-187
  trace: correct
- test_id: original_affine_conversions_and_scale_one_alias_match_native_exactly
  ac_id: QSpec-FR-142
  trace: correct
- test_id: original_u10_exact_tuple_and_named_final_denial_keep_full_prefix
  ac_id: QSpec-TC-187
  trace: correct
- test_id: original_u10_exact_tuple_and_named_final_denial_keep_full_prefix
  ac_id: QSpec-FR-142
  trace: correct
- test_id: native_normalization_uses_original_admitted_dimension_maps
  ac_id: QSpec-TC-187
  trace: correct
- test_id: native_normalization_uses_original_admitted_dimension_maps
  ac_id: QSpec-FR-142
  trace: correct
- test_id: original_quantity_subnodes_preserve_native_normalized_compound_identities
  ac_id: QSpec-TC-187
  trace: correct
- test_id: original_quantity_subnodes_preserve_native_normalized_compound_identities
  ac_id: QSpec-FR-142
  trace: correct
```
