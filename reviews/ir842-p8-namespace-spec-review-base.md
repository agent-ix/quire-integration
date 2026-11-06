---
id: SR-2026110414560094
title: spec-review-base review of TC-107 namespace and emission
type: SpecReview
analysis: base
scope: agent-ix/quire-integration; spec/test-cases/TC-107-tc187-namespace-and-emission.md,
  tests/tc187_p8_namespace.rs, tests/tc187_p8_emission.rs, tests/support/tc187_p8_source.rs
review_set: subset
---

## Summary

Ticket: IR-842. Review of quire-integration PR #73, limited to the four new TC-107 namespace/emission files. No candidate edits or gate execution by this reviewer. No applicable AssuranceProfile exists in this repository's spec tree.

U33's literal and operation-member controls use original public QSL intake/check/link/emission and native strict IR admission. Literal magnitude, original source correspondence, unit identity, parameter type, model owner and operation anchor are asserted. U32 record/unit separation and reserved-name location pass. Its duplicate-unit test deliberately characterizes the earlier owner refusal; it does not qualify the required quantity-use locus. The typed payload retains both declaration spans, with primary 97..99 versus required 144..156. The original red receipt confirms this mismatch; TC-107 and PR prose retain the unresolved cell under IR-842.

The test helper does not forge checked packages, keys or graph DTOs. The domain document is authored test input admitted by public APIs; emitted bytes remain unchanged. Public vector strings bind both complete U32/U33 rows and are not a copied fixture corpus. Required-feature support is declared to the native reader; no generated execution or value agreement is inferred. Existing dispatcher and IT-019 remain outside this change and require the lead's precise handoff.

Focused receipts inspected: five tests passed, confined Clippy completed, formatting and TC validation reported successful. These are coder receipts, not reviewer reruns. The dispatch reserves one final scoped gate to the lead; broad build, deny, Kani and make ci were not run here.

## Verdict

**PASS** — no defects in this bounded change; U32 remains partial and the exact-location cell remains Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

TC-107 has a unique ID, concrete public inputs and outcomes, bounded procedure, valid reference-shaped links and an explicit measured partial disposition. Test steps are specifications of the integration checks rather than duplicated upstream requirements. No new hash, pin, catalog or file-tracking machinery is introduced. The related TC-233 and TC-280 families remain explicitly unclaimed.

## Examined Units

```yaml
scope:
- id: TC-107/Description
  path: spec/test-cases/TC-107-tc187-namespace-and-emission.md
  role: examined
  excerpt: 'Exercise the public U32 and U33 cells by reference to the installed QSpec
    vector

    API. Author source and domain-model inputs here; obtain all checked nodes and

    identities through public QSL intake, checking, linking and emission. Strictly

    admit the original emitted package with Contract IR. This procedure measures

    static admission and emission, without generated execution or value agreement.'
- id: TC-107/Test Procedure
  path: spec/test-cases/TC-107-tc187-namespace-and-emission.md
  role: examined
  excerpt: "1. Bind U32 and U33 to their complete public input and expected strings\
    \ through\n   `vectors_of(TestCase::Tc187)`, requiring one matching row for each\
    \ ID.\n2. Declare a source record and unit both named `ms`. Check a function whose\n\
    \   ordinary type parameter names the record and whose `Quantity<ms>` parameter\n\
    \   names the unit. Follow both original parameter type references through the\n\
    \   emitted package and strict IR admission, requiring distinct record and unit\n\
    \   identities and the corresponding semantic forms.\n3. Assemble two genuine\
    \ source documents into one linked package, each declaring\n "
- id: TC-107/Expected Results
  path: spec/test-cases/TC-107-tc187-namespace-and-emission.md
  role: examined
  excerpt: 'U32 requires all three namespace subcases, including the exact refusal
    locations.

    U33 requires the original quantity literal and admitted domain operation parameter

    to reference the declared `ms` unit. Related TC-233 QL-01 through QL-04 define

    quantity literal wire admission and refusal variants; TC-280 MU-01 through MU-06

    define model fields, collections, operations and invalid argument variants. This

    procedure covers only U33''s literal magnitude 3 and operation parameter type;
    it

    does not claim those related vector families in full.


    Validation results and any owner boundary that prevents a '
- id: TC-107/Measured Disposition
  path: spec/test-cases/TC-107-tc187-namespace-and-emission.md
  role: examined
  excerpt: 'U33''s two bounded admission/emission subcases pass through original QSL
    output

    and strict Contract IR admission. The related TC-233 and TC-280 variant families

    remain outside this procedure''s claim.


    U32 is partial: the same-name record/unit selection and reserved-name refusal

    pass, but duplicate linked units refuse earlier than the public cell requires.

    The real assembly refusal is `DuplicateQuantityName` for `ms`, with both name

    spans retained and primary span at the second unit declaration. In this authored

    source the actual span is 97..99; the required `Quantity<ms>` span is 144..156.

    The '
- id: u32
  path: tests/tc187_p8_namespace.rs
  role: examined
  excerpt: fn u32() {
- id: ordinary_ms_type_and_quantity_ms_select_distinct_original_nodes
  path: tests/tc187_p8_namespace.rs
  role: examined
  excerpt: fn ordinary_ms_type_and_quantity_ms_select_distinct_original_nodes() {
- id: linked_duplicate_ms_units_expose_the_quantity_type_location_blocker
  path: tests/tc187_p8_namespace.rs
  role: examined
  excerpt: fn linked_duplicate_ms_units_expose_the_quantity_type_location_blocker()
    {
- id: quantity_is_reserved_at_the_record_name
  path: tests/tc187_p8_namespace.rs
  role: examined
  excerpt: fn quantity_is_reserved_at_the_record_name() {
- id: u33
  path: tests/tc187_p8_emission.rs
  role: examined
  excerpt: fn u33() {
- id: original_call_literal_is_typed_by_the_declared_ms_node
  path: tests/tc187_p8_emission.rs
  role: examined
  excerpt: fn original_call_literal_is_typed_by_the_declared_ms_node() {
- id: origin
  path: tests/tc187_p8_emission.rs
  role: examined
  excerpt: fn origin() -> Value {
- id: document
  path: tests/tc187_p8_emission.rs
  role: examined
  excerpt: fn document() -> Vec<u8> {
- id: admitted_operation_parameter_names_the_original_ms_unit
  path: tests/tc187_p8_emission.rs
  role: examined
  excerpt: fn admitted_operation_parameter_names_the_original_ms_unit() {
- id: vector
  path: tests/support/tc187_p8_source.rs
  role: examined
  excerpt: 'pub fn vector(id: &str, input: &str, expected: &str) {'
- id: parsed
  path: tests/support/tc187_p8_source.rs
  role: examined
  excerpt: 'pub fn parsed(name: &str, text: &str) -> (RawSourceRef, qsl_forms::ParsedUnit)
    {'
- id: emitted
  path: tests/support/tc187_p8_source.rs
  role: examined
  excerpt: 'pub fn emitted(graph: CheckedGraph, model: Option<(&str, &[u8])>) -> CheckedPackageV2
    {'
bindings:
- test_id: ordinary_ms_type_and_quantity_ms_select_distinct_original_nodes
  ac_id: QSpec-FR-142-AC-13
  trace: correct
- test_id: linked_duplicate_ms_units_expose_the_quantity_type_location_blocker
  ac_id: QSpec-FR-142-AC-13
  trace: correct
- test_id: quantity_is_reserved_at_the_record_name
  ac_id: QSpec-FR-142-AC-13
  trace: correct
- test_id: original_call_literal_is_typed_by_the_declared_ms_node
  ac_id: QSpec-FR-142-AC-14
  trace: correct
- test_id: admitted_operation_parameter_names_the_original_ms_unit
  ac_id: QSpec-FR-142-AC-14
  trace: correct
```
