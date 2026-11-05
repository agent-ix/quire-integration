---
id: SR-2026110414560105
title: spec-review review of TC-111 equivalence admission
type: SpecReview
analysis: base
scope: agent-ix/quire-integration; spec/test-cases/TC-111-tc187-equivalence-admission.md;
  tests/tc187_p8_equivalence_admission.rs
review_set: subset
---

## Summary

Ticket: IR-842. PR context: agent-ix/quire-integration#76. Admission-only evidence; no generated agreement, aggregate AC acceptance or whole SC-04 credit. No applicable AssuranceProfile exists under spec/. Review selection follows the dispatched scoped methods; optional semantic fanout is excluded.

Base TC review: description, five-step procedure and expected results define the public-native admission seam and exact evidence. The procedure preserves setup failures as failures and explicitly excludes generated agreement, whole SC-04 acceptance and issue closure. TC-111 and its three references are well formed; all new tests carry the declared owner criterion trace. No new FR/US or duplicated owner requirement is introduced.

## Examined Scope

```yaml
scope:
- id: "TC-111:Description\n\nExercise every U37 equivalence admission alternative\
    \ through public native model\nintake and package assembly. Bind the original\
    \ public vector by reference through\n`vectors_of(TestCase::Tc187)`. Original\
    \ minimal R/F model documents declare real\nTime/Length roots and derived Speed,\
    \ R's Area, and F's affine local unit. These\ninputs are domain documents, never\
    \ reconstructed checked packages or receipts.\n\n## Test Procedure\n\n1. Parse\
    \ original complete-profile source, build forms, admit selected original\n   model\
    \ documents, and assemble without requiring unrelated body checking.\n2. Cover\
    \ imported/imported and imported/local pairs, missing aliases, dimensions\n  \
    \ and ratio units, self pairs, duplicate pairs in both orientations and with\n\
    \   equal and differing ratios, wrong-dimension units, zero/negative ratios,\n\
    \   imported composed affine offset, and Time versus Area.\n3. Exercise Dur without\
    \ its base authority and with that authority before and\n   after the derived\
    \ pair. Exercise both Speed ratios without authorities, with\n   either authority\
    \ omitted, and with both authorities in both orders and on\n   either side of\
    \ the derived pair.\n4. Exercise consistent and inconsistent ABC cycles in literal\
    \ forward and reverse\n   source order. Assert the actual closing declaration\
    \ and complete original\n   retained declaration sequence, including exact spans.\n\
    5. Compare full typed refusal causes, codes and primary spans. Acquire retained\n\
    \   dimension keys from genuine successful alias-unit controls, separately from\n\
    \   admitted model scalar dimension keys. For successes assert exact original\n\
    \   row order/spans, owner package, canonical native dimension pairs, rational\n\
    \   scales, least class roots and exact relative sizes.\n\n## Expected Results\n\
    \nThe new equivalence admission target must satisfy all authored U37 alternatives.\n\
    Model intake must succeed even for affine-unit cases; an intake error is a setup\n\
    failure, never an accepted equivalence refusal. Duplicate and inconsistent cycle\n\
    refusals preserve actual source order and the last closing primary locus. Derived\n\
    pairs use complete base authority independent of source phase ordering.\n\nThis\
    \ is admission-only evidence. It establishes no generated-oracle agreement,\n\
    value/accounting comparison, whole SC-04 completion, or IR-842 closure. Any real\n\
    owner disagreement remains a failing receipt with its exact typed facts.:1"
  path: spec/test-cases/TC-111-tc187-equivalence-admission.md
  role: examined
  excerpt: ''
- id: public_u37_is_bound_by_reference
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn public_u37_is_bound_by_reference() {
- id: basic_pairs_names_and_self_pair_keep_native_identity_and_locus
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn basic_pairs_names_and_self_pair_keep_native_identity_and_locus() {
- id: duplicate_pairs_keep_both_original_declarations_in_either_order
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn duplicate_pairs_keep_both_original_declarations_in_either_order() {
- id: wrong_units_values_affine_offset_and_area_keep_full_typed_payloads
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn wrong_units_values_affine_offset_and_area_keep_full_typed_payloads()
    {
- id: dur_uses_base_authority_before_derived_phase_in_both_source_orders
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn dur_uses_base_authority_before_derived_phase_in_both_source_orders()
    {
- id: speeds_require_both_base_pairs_and_exact_ratio_in_all_phase_orders
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn speeds_require_both_base_pairs_and_exact_ratio_in_all_phase_orders()
    {
- id: abc_cycles_preserve_sizes_and_actual_closing_locus_when_reversed
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn abc_cycles_preserve_sizes_and_actual_closing_locus_when_reversed() {
- id: document
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn document(identity: &str, area: bool) -> Vec<u8> {'
- id: assemble
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn assemble(body: &str) -> (String, Result<PackageDeclarations, AssemblyRefusal>)
    {'
- id: span
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn span(source: &str, text: &str) -> Span {'
- id: nominal
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn nominal(package: &PackageDeclarations, source: &str, name: &str) ->
    NodeKey {'
- id: Keys::control
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: fn control() -> Self {
- id: refused
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn refused(body: &str, primary: &str, expected: impl FnOnce(&str) -> AssemblyCause,
    code: &str) {'
- id: fault
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn fault(fault: EquivalenceFault, retained: Vec<EquivalenceRetained>)
    -> AssemblyCause {'
- id: admitted
  path: tests/tc187_p8_equivalence_admission.rs
  role: examined
  excerpt: 'fn admitted(body: &str, pairs: &[(&str, [&str; 2], Rational)], keys: &Keys)
    -> EquivalenceClasses {'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Repository census/matrix and fresh build/test/gate/Kani execution were excluded by dispatch. This is a manual scoped gap review, not a repository-wide gap verdict. Existing focused receipts were inspected: /tmp/ir842-p8-equivalence-test.log (7 pass), /tmp/ir842-p8-equivalence-clippy.log, /tmp/ir842-p8-equivalence-fmt.log and /tmp/ir842-p8-equivalence-quire.log. Final scoped gate remains lead-owned.

## Verdict

PASS within the two-file admission scope. No defects found.
