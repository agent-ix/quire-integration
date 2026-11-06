---
id: SR-2026110414560113
title: "code-review of original selected-model quantity observations"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-842; spec/test-cases/TC-113-tc187-model-quantity-values.md; tests/tc187_p8_model_values.rs; tests/support/tc187_p8_model_values.rs"
review_set: subset
---
## Summary

Ticket: IR-842. Review context: quire-integration PR 78. Examined original selected-model admission and source/native observations of public U36/U38/U40, with TC-113's complete per-subcell qualification contract. No applicable AssuranceProfile exists in this repository's spec tree.

## Verdict

**PASS** — no defects found in this scoped change. The full mixed rows and generated comparisons remain Planned. Exactly two conversion failures are measured characterizations; other class-dependent positives remain explicitly unexecuted and unqualified. This verdict accepts the observation cohort, not normative equivalence conversion or whole-row acceptance.

## Analysis

Original authored model documents enter public parse, form, intake, admission, assembly and checking APIs. Actual unit and dimension keys come from owner declarations; class ratios are observed through the owner's EquivalenceClasses API, not reconstructed as integration authority. Public vector inputs and expectations are bound by reference rather than copied fixture corpora. Existing observation::snapshot is reused; no semantic engine is copied.

U36 call admission and independent argument/operation refusals retain exact selected source loci. Four closed product/quotient observations assert magnitude and complete original root/dimension maps. Each successful comparison uses fresh native and QSL meters at matching operation boundaries and compares typed outcome, empty exact losses, ordered charges and every counter. The original whole let body is compared against the full native multiply/divide/convert sequence; no whole-call subtraction is used. Refusal meters remain empty and every counter is checked zero after each operation.

U38/U40 declaration spelling, sorted keys, source spans, exact class ratios, three-root admission, consistent/conflicting cycles, missing Mass/Force and derived Torque/Energy invalid ratio are independently checked. Original conversion expressions to F seconds and G milliseconds encounter source TypeMismatch and real context-native IncompatibleDimensions with zero charges/counters. Neither characterization claims normative conversion acceptance; no temporal substitution or ratio mapper is introduced. The earlier U40 parse fixture error is not treated as an owner failure.

The TC-113 subcell table distinguishes successful admission, actual source/native observations, characterizations, unexecuted positives and generated work. Structural reference to IT-019 resolves in the existing repository. No new FR, domain object, production path, relationship dependency or grammar is introduced. EARS, dependency, object, gap-analysis and optional semantic analyses are not applicable to this test-only scope. Integrity checks cover TC structure, observable assertions, consistency of statuses, original identities, evaluator purity and cycle refusal topology.

## Verification

Reviewed supplied focused final logs: eleven tests pass, scoped Clippy warnings denied passes, formatting check and TC validation pass. Per dispatch, no build, test, gate, census or Kani command was rerun by this reviewer. The dispatching lead owns the final scoped gate. No CI, dispatcher, IT-019, shared support, manifest or lock change is present. This review does not claim a complete repository matrix or a repository-wide gate run.

## Examined scope

```yaml
scope:
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '## Description'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: 'Exercise separately reachable source and native quantity cells of public
    TC-187

    U36, U38 and U40 through their original selected models. Bind inputs and expected

    results by reference to the public vector API. A successful admission establishes

    only admission; a partial observation never qualifies an entire mixed row.'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '## Test Procedure'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: "1. Supply original nonempty model documents for acme/rpc, acme/flow and,\
    \ where\n   required, acme/clock. Parse original source, admit the selected documents\
    \ and\n   assemble declarations through public QSL APIs. Retain actual unit and\
    \ dimension\n   identities rather than inventing structural keys.\n2. Execute\
    \ U36 call admission and independent argument, sum, comparison and direct\n  \
    \ conversion refusals. Evaluate its original closed quotient and product bodies,\n\
    \   and the entire Dur bridge let body. Compare native outcomes, canonical unit\n\
    \   terms, dimension maps, exact losses, ordered charges and "
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '## Expected Results'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: 'Eleven focused tests pass. The source calls and refusal loci, four native

    incompatibility refusals, four closed product/quotient stages, and the original

    whole let bridge are observed independently. The five successful quantity stages

    compare complete typed results, empty exact losses, ordered charge records and

    every meter counter. Compound terms and dimension maps use original owner keys.

    The let binding is observed as magnitude three with the one original root term;

    the entire QSL let stage agrees with the native multiply/divide/convert sequence.'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: 'The two explicitly named characterization tests record a measured owner

    disagreement: the original equivalent conversion source refuses checking with

    `IllTyped(TypeMismatch)` at the conversion body, and the native context converter

    refuses `IllTyped { cause: IncompatibleDimensions }` with no admitted charges
    and

    all counters zero. The admitted classes still identify the original dimension

    keys with exact root-size ratios one and one thousand respectively. These are

    characterizations, not successful conversion qualifications. Intended positive

    red probes were executed before characterizations w'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| Public subcell | Source/native qualification | Generated qualification
    |

    | --- | --- | --- |

    | U36 call with R seconds | Source admitted | Planned |

    | U36 calls with F and local seconds | Exact argument TypeMismatch, independently
    | Planned |

    | U36 mixed sum, less-than, and F conversion | Exact source loci; native IncompatibleDimensions,
    no charges/all counters zero | Planned |

    | U36 mixed quotient | Exact one; original two-dimension map and root terms; full
    QSL/native agreement | Planned |

    | U36 same-root quotient | Exact one; empty dimension and root maps; full QSL/native
    agreement | Plann'
- id: TC-113
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: 'No generated runner is exercised by this cohort. No row is promoted to
    Passed.

    All mixed rows and full IT-019-SC-04 remain Planned. The unexecuted positive cells

    above are not reported as individually measured owner failures. Integration does

    not normalize classes, rewrite keys or substitute the temporal conversion API.

    '
- id: u36_calls_and_incompatible_operations_keep_original_source_loci
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u36_calls_and_incompatible_operations_keep_original_source_loci()
- id: u36_native_incompatibility_refuses_before_every_charge
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u36_native_incompatibility_refuses_before_every_charge()
- id: u36_quotients_and_products_preserve_native_canonical_maps_and_ledgers
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u36_quotients_and_products_preserve_native_canonical_maps_and_ledgers()
- id: u36_original_whole_let_bridges_compound_to_dur_with_complete_ledger
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u36_original_whole_let_bridges_compound_to_dur_with_complete_ledger()
- id: u38_u40_original_ratio_spellings_and_three_root_classes_admit
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u38_u40_original_ratio_spellings_and_three_root_classes_admit()
- id: u40_conflicting_cycle_retains_every_original_declaration_in_order
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u40_conflicting_cycle_retains_every_original_declaration_in_order()
- id: u38_missing_mass_unit_and_unanchored_force_keep_original_named_causes
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u38_missing_mass_unit_and_unanchored_force_keep_original_named_causes()
- id: u38_derived_equivalence_admission_and_invalid_ratio_keep_original_dimensions
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u38_derived_equivalence_admission_and_invalid_ratio_keep_original_dimensions()
- id: u38_u40_unconverted_source_operations_and_unrelated_conversion_refuse
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn u38_u40_unconverted_source_operations_and_unrelated_conversion_refuse()
- id: characterize_u38_checked_and_native_conversion_missing_equivalence_effect
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn characterize_u38_checked_and_native_conversion_missing_equivalence_effect()
- id: characterize_u40_checked_and_native_conversion_missing_equivalence_effect
  path: tests/tc187_p8_model_values.rs
  role: examined
  excerpt: fn characterize_u40_checked_and_native_conversion_missing_equivalence_effect()
- id: original-model-intake
  path: tests/support/tc187_p8_model_values.rs
  role: examined
  excerpt: 'pub fn document(identity: &str, missing_mass_root: bool) -> Vec<u8> {'
- id: TC-113-subcell-1
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 call with R seconds | Source admitted | Planned |'
- id: TC-113-subcell-2
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 calls with F and local seconds | Exact argument TypeMismatch, independently
    | Planned |'
- id: TC-113-subcell-3
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 mixed sum, less-than, and F conversion | Exact source loci; native
    IncompatibleDimensions, no charges/all counters zero | Planned |'
- id: TC-113-subcell-4
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 mixed quotient | Exact one; original two-dimension map and root
    terms; full QSL/native agreement | Planned |'
- id: TC-113-subcell-5
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 same-root quotient | Exact one; empty dimension and root maps; full
    QSL/native agreement | Planned |'
- id: TC-113-subcell-6
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 mixed and same-root products | Exact six each; unequal complete
    canonical units/maps; full QSL/native agreement | Planned |'
- id: TC-113-subcell-7
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 direct Dur conversion | Source refusal; native distinct declared
    dimensions, no charges/all counters zero | Planned |'
- id: TC-113-subcell-8
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U36 whole let bridge | Exact three dsec; native binding terms and complete
    original whole-stage agreement | Planned |'
- id: TC-113-subcell-9
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 R/F equivalence declaration | Actual row owner/span, sorted keys,
    scale and class ratio verified | Planned |'
- id: TC-113-subcell-10
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 R seconds to F seconds | Measured source/native refusal characterization;
    intended positive Planned | Planned |'
- id: TC-113-subcell-11
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 R seconds to F milliseconds; reverse F milliseconds conversion |
    Unexecuted positive cells, Planned | Planned |'
- id: TC-113-subcell-12
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 converted equality, converted sum and converted argument | Unexecuted
    positive cells, Planned | Planned |'
- id: TC-113-subcell-13
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 unconverted mixed sum and equality | Source TypeMismatch verified;
    normative native DistinctUnits cause unqualified | Planned |'
- id: TC-113-subcell-14
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 unconverted F argument | Exact source argument TypeMismatch | Planned
    |'
- id: TC-113-subcell-15
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 quotient and two products under equivalence | Unexecuted positive
    canonicalization cells, Planned | Planned |'
- id: TC-113-subcell-16
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 unrelated Length conversion | Exact source refusal; native incompatible
    dimensions without charges | Planned |'
- id: TC-113-subcell-17
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 third-root declaration | Original class admission verified; every
    ordered conversion and edge count unexecuted, Planned | Planned |'
- id: TC-113-subcell-18
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 absent F mass root and Force use | Independent exact named assembly
    causes and original loci verified | Planned |'
- id: TC-113-subcell-19
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 Torque/Energy without equivalence | Source and native direct-conversion
    refusal verified | Planned |'
- id: TC-113-subcell-20
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 Torque/Energy ratio one | Original equivalence admitted with full
    keys/span/ratio; positive conversion unexecuted, Planned | Planned |'
- id: TC-113-subcell-21
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U38 Torque/Energy ratio two | Exact Incompatible payload with both original
    dimension keys and locus verified | Planned |'
- id: TC-113-subcell-22
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 three ratio spellings | Independent original admission, rows/spans
    and exact class relations verified | Planned |'
- id: TC-113-subcell-23
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 R seconds to G milliseconds | Measured source/native refusal characterization;
    intended positive Planned | Planned |'
- id: TC-113-subcell-24
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 reverse conversion and conversion to G seconds | Unexecuted positive
    cells and edge counts, Planned | Planned |'
- id: TC-113-subcell-25
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 quotient and product | Unexecuted positive canonicalization cells,
    Planned | Planned |'
- id: TC-113-subcell-26
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 mixed sum | Source TypeMismatch verified; normative native DistinctUnits
    cause unqualified | Planned |'
- id: TC-113-subcell-27
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 consistent three-edge cycle | Original rows, owners, spans and class
    ratios verified | Planned |'
- id: TC-113-subcell-28
  path: spec/test-cases/TC-113-tc187-model-quantity-values.md
  role: examined
  excerpt: '| U40 inconsistent closing ratio | Full Conflicting payload, original
    closing locus and three declarations in source order verified | Planned |'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
