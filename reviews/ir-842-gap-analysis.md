---
id: SR-20261107100902
title: "Gap analysis of quire-integration PR 87 against the IR-842 required-cell map and CG FR-014"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; PR 87 head b3979568aa8388a1303d130080593b7e0d9bfe55; required-cell map IR-842 comment cd25a010; quire-contract-codegen FR-014 at cd184a37 (AC-2, AC-6, AC-48, AC-61..65); spec/integration/IT-019-generated-oracle-qsl-agreement.md; tests/generated_oracle_*.rs; tests/tc187_*.rs"
review_set: subset
---

## Summary

Ticket: IR-842. PR: quire-integration#87. Planless and scoped to the PR's
required-cell map. Plan completion: not assessed. The planner rulings in the
brief are taken as given: U32/U34/U35/U39 are outside the required set;
enum `<` is absent; enum equality is Planned on the QSpec-fixture versus
QSL-intake seam; the U25 first step is on IR-1403; U27/U21 undefined is
unreachable, so it is a typed control; U01 and the IEEE comparisons are on
IR-1401; evaluated-right connectives are on IR-1402; FR-018 is on IR-1381;
div/mod is on IR-1391.

Cells checked against executing tests:

- Integer arithmetic, ordering and equality with endpoints: covered.
- Rational arithmetic, division and ordering: covered.
- Decimal arithmetic, rounding in every mode and ordering: covered.
- IEEE arithmetic and width conversion for both widths in every mode:
  covered.
- Text across all six profiles: covered.
- Boolean skipped-right: covered.
- FR-021 Multiply 4 × 7 and the PayloadChoice union: covered.
- U04, U09b, U10, U13 huge, U21 0^0, U28 (three tuples), and the carried
  U07/U08/U16/U17/U18/U29: covered, with the generated symbol actually
  executed. The literal operands need no replay.
- U22, U25 and the U02 quotient: covered only for the observations where
  the generated symbol runs (see SR-20261107100901 FND-001).

Orphan code: none. All new test support has an owning TC/IT row.

## Verdict

**FAIL (two medium gaps).** Two required cells are credited as Passed
without a generated execution, or without the grid AC-6 names.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014-AC-6 requires "the generated grid of quantity arithmetic, comparisons and powers over `m`, `cm`, `in`, `rev`, `u2` and `s`". The grid tests use m, cm, inch, kg and s for arithmetic, m, cm and kg for powers, and m, cm, inch, s, kg, K, degC, degF, J, m2 and cm2 for comparisons. `rev` and `u2` never appear, `s` and `inch` get no power, and there is no affine refusal row in the grid. IT-019 nevertheless marks "FR-014 Quantity ... over the declared unit grid" Passed | tests/tc187_ir842_conversions.rs:217-306; spec/integration/IT-019-generated-oracle-qsl-agreement.md:265 |
| FND-002 | medium | The U02 product normalization `(L*T)` is never a generated cell. In `(1 m * 1 s) / 1 s` only the Divide is generated; the product is computed by the replayed QSV owner `evaluate_quantity_unit(Multiply)`. AC-6 asks for normalization "through a product and a quotient", and IT-019 credits U02 as Passed (21 observations with U22) | tests/tc187_ir842_conversions.rs:201-210; tests/support/tc187_p9_source.rs:732-744; spec/integration/IT-019-generated-oracle-qsl-agreement.md:256 |
