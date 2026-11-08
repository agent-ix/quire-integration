---
id: SR-2026110720500003
title: "IR-1003 integration PR 91 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1003; tests/temporal_owner_agreement.rs and spec/test-cases/TC-115-native-temporal-projection.md"
review_set: base
---

## Summary

Ticket: IR-1003. Frozen candidate f936aa638bd26bfc097b4e9c2d9fbe68dc08d804 against origin/main 1989a0d. Rust lane and manual acceptance alignment: independently expected OR=true and AND=false; two distinct operand valuations; real QSL emission/IR admission, native evaluate_v2 and TL report evaluation/mapping; strict projection and join readers. Exact native, mapped and source bytes are compared. Foreign requests differ and both foreign evaluators produce genuine true values before each substitution yields ResultRefused. Source inspection confirms native owner binding read and TL request identity checks cause refusal. Existing shared helpers are reused; no vendored semantics, unsafe code, gate narrowing or production changes.

## Scope

```yaml
scope:
- id: FR-350-AC-1
  path: quire-contract-ir/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Given an admitted package and a QSL `Supported` classification, projection
    takes its target from the classification and returns owner-admitted formula, proposition
    map, trace or history and request values for every supported row: future-only
    under complete and open surrounding closure (closed-trace and online-prefix targets),
    and each bounded past operator of QSpec FR-092 alone under both false-extension
    profiles with origin-complete history. A non-zero range start or an `authoritative_origin`
    other than `Some(true)` for a past target returns incomplete `history`.'
- id: FR-350-AC-5
  path: quire-contract-ir/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'For each of R1 to R8 a genuine owner-produced pair reaches that row with
    the stated decision, comparison and cause list, excluding the defensive row R4,
    including an equal-final pair with `P` false and a final/pending pair (R6), a
    TL `Failed` result (R2) and a native `InvalidPredecessor` (R6); for each pair
    of rows R1 to R6 a pair satisfying both returns the first decision and both causes
    once in row order; only R7 carries a Boolean; every join retains the complete
    native and TL result values. Each lineage condition of `L` and an equal-identity
    different-content predecessor yields R6, while a '
- id: FR-350-AC-6
  path: quire-contract-ir/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Each binding step failure returns the code and premise FR-350 assigns it;
    a row failing several valuation checks returns the first cause in the owner's
    check order; repeated agreeing rows project their value and are all recorded;
    disagreeing or missing rows return incomplete `valuation{position, operand}`;
    each owner-reader refusal returns `tl-projection-owner`; non-canonical, reordered,
    unknown-member and trailing projection or join bytes return their mismatch code;
    no error leaves a partial value. Joined native and TL values equal the supplied
    values with unchanged owner usage, and no bridge
- id: TC-115
  path: spec/test-cases/TC-115-native-temporal-projection.md
  role: examined
  excerpt: 'The direct`TC-115` tests also carry one exact FR-350 criterion

    per trace attribute. They project two distinct true/false valuation rows through

    conjunction and disjunction, evaluate each owner separately, and strictly read

    the joined Boolean with its original owner bytes. Independently authored formulas

    that both evaluate true must still refuse a foreign native or TL result; equal

    Boolean values alone never authorize a join.'
bindings:
- test_id: two_distinct_valuation_rows_reach_both_owner_results_and_the_strict_join
  ac_id: FR-350-AC-1
  trace: correct
- test_id: two_distinct_valuation_rows_reach_both_owner_results_and_the_strict_join
  ac_id: FR-350-AC-5
  trace: correct
- test_id: two_distinct_valuation_rows_reach_both_owner_results_and_the_strict_join
  ac_id: FR-350-AC-6
  trace: correct
- test_id: equal_true_results_from_another_formula_do_not_authorize_the_join
  ac_id: FR-350-AC-5
  trace: correct
- test_id: equal_true_results_from_another_formula_do_not_authorize_the_join
  ac_id: FR-350-AC-6
  trace: correct
```

## Assurance Context

No AssuranceProfile is declared in this repository's spec tree. Ordinary rubric applies.

## Evidence

Reviewed repository AGENTS.md, CLAUDE.md, README.md, CONTENT_RIGHTS.md and Rust conventions; no CI workflow changes. Accepted existing candidate gate evidence: focused two tests, compile/Clippy, fmt, lint, release build, license and unsafe audit, TC-115 validation. Inspected completed serial runner summary: 64 executables, all exits zero, compile/doc exits zero; reported totals 402 passed and four existing ignored. No redundant compiler or test run, Kani, hosted CI, coverage or matrix measurement performed. PR is MERGEABLE; its generated footer and both commit coauthor trailers match the dispatch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Five bindings are correct for the narrow assertions they exercise. Cross-repository strict credit remains unresolved under IR-724; no strict coverage completion is inferred. Gap analysis is inapplicable because the diff contains only tests and TC prose. Optional spec analyses are not selected: no changed SHALL statement, structural/domain-object edit or new relationship.
