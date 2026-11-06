---
id: SR-2026110414560123
title: "IR-842 PR 80 spec-review-base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-842 PR 80; remaining U15-U18/U20/U22/U23/U25-U29 original quantity evidence; five changed files"
review_set: subset
---

## Summary

Reviewed the remaining original quantity qualification slice and its evidence boundaries. No defects found within the frozen scope.

## Verdict

PASS for the scoped static review. Aggregate gate qualification remains pending with the lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Application version: 0.1.0. Ticket: IR-842. Frozen PR 80 scope only. No applicable AssuranceProfile was found. No production, dependency, gate or CI change. No source edits, builds, tests or final gates performed by reviewer; final aggregate qualification belongs to the lead. Author logs show focused compile success, conversion 1/1 and quantity 3/3 passes, addition 1/1 and U12 1/1 regression passes. Historical probe-clippy log is failing exploratory evidence; it is not a green gate receipt. Final focused compile follows the added shared-support dead-code allowance. No full final gate result is claimed.

Fifteen conversion budgets and eight quantity/comparison budgets independently assert outcome, loss, source locus, complete ordered admitted records and all ten counters. Two additional U27 public checked-expression budgets preserve original AST, full-package Unproved(Nonzero), exact division span, DivisionByZero priority and unpaid second read. They do not manufacture a whole-function package. Nine native snapshots (U15 two, U26 two, U20 one, U23 four) compare outcome/loss/ordered ledger/all counters against a fresh independently checked original source. Source loci are checked separately; native scalar snapshots claim no source-envelope location.

Five source TypeMismatch refusals retain actual original spans, separately from five zero-limit owner controls. Two whole U22 let-body values include power preparation; they are expressly distinct from three isolated owner conversion ledgers. U25 carries the actual first converted compound identity/value into its second owner step, with exact 1 J and two complete ledgers. No isolated QSL conversion-only/native credit is claimed for these five owner controls. Authentic U22 subtree checking refuses TypeMismatch; U25 compound-target grammar refuses InvalidSyntax at the asterisk. These original source seams and compound CG support remain Planned under IR-1348.

Seven isolated carried sources U16-U18/U29 each assert strict-reader InvalidPackage/InvalidDerivedShape at scalar_type/carried_quantity before CG. IR-1349 retains both that precursor and later selector/renderer work; no later stage is claimed reached. U28 source values/size denials remain separate from actual CG ResultTypeMismatch expected Unit/found compound_unit; U27/U28 generated agreement remains Planned under IR-1348. U18 analytical denial never materializes the huge coefficient.

Searched shared support for existing accounting, charge, source_text and original_with helpers; the extension reuses the existing source helper and common oracle snapshot instead of creating a competing evaluator or identity authority. Native metadata is transported from genuine admitted declarations; exact source operands supply native inputs. No call subtraction, precharge, replacement evaluator, rewritten IR or invented identity is present. Existing dispatcher, P8 and other lane files are untouched.

Base specification checklist applied to TC-106 steps 8–10 and owned IT-019 rows. No new FR/NFR statements, domain objects, relationship edges or specification structure are introduced, so EARS/integrity/dependency and other specialized lenses were not selected. Counts and Passed/Planned qualifications match actual assertions and logs; no whole TC-187 or SC-04 completion is claimed. No hash, digest, revision or file-tracking machinery was introduced.

## Examined scope

```yaml
scope:
- id: tests/support/tc187_p9_source.rs
  path: tests/support/tc187_p9_source.rs
  role: examined
  excerpt: //! Original quantity source acquisition and public closed-body observations.
- id: tests/tc187_p9_remaining_conversions.rs
  path: tests/tc187_p9_remaining_conversions.rs
  role: examined
  excerpt: //! Original exact and carried quantity conversions at their charged boundaries.
- id: tests/tc187_p9_remaining_quantity.rs
  path: tests/tc187_p9_remaining_quantity.rs
  role: examined
  excerpt: //! Original affine, comparison and compound quantity operation boundaries.
- id: spec/test-cases/TC-106-quantity-static-admission.md
  path: spec/test-cases/TC-106-quantity-static-admission.md
  role: examined
  excerpt: 'title: "Qualify original quantity arithmetic and static admission"'
- id: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'title: "Compare executed generated oracles with independent QSL authority"'
- id: QSpec-TC-187-U15
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 in` to `cm` under `L(15,0,2,1,9,1)`, then under `L(7,0,1,1,9,1)`
- id: QSpec-TC-187-U16
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Evaluate U07 under `L(15,5,1,1,6,1)`; repeat `exact` and then `nearest-even`
    under `L(15,5,1,1,4,1)`
- id: QSpec-TC-187-U17
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `3 m` into U08's target under `L(2,1,0,1,2,0)`, then under `L(2,1,0,1,1,0)`
- id: QSpec-TC-187-U18
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 m` into target `Decimal[0,1;0,4294967295;exact]` under `L(18446744073709551615,64,0,1,2,1)`
- id: QSpec-TC-187-U20
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Under `L(0,0,0,0,0,0)`, add `1 u1 + 2 u1` and multiply `1 u3 * 1 m`; add
    `1 u2 + 2 u2` under `L(4,0,0,2,5,1)`
- id: QSpec-TC-187-U22
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 N_m` to `J` under `L(0,0,0,0,0,0)`; convert compound `4 m^2`
    to `m2` under `L(3,0,0,1,3,1)`; convert compound `4 m^2` to `cm2` under `L(17,0,1,1,6,1)`,
    then under `L(16,0,1,1,6,1)`
- id: QSpec-TC-187-U23
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Order `1 degC < 2 degC` under `L(15,0,2,2,9,1)`, then under `L(15,0,2,2,8,1)`;
    order `1 rev < 2 rev` and test `1 degC == 1 degC`; under `L(0,0,0,0,0,0)`, order
    `0 degC < 32 degF` and test `0 degC == 32 degF`
- id: QSpec-TC-187-U25
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 N_m` to compound `kg*m^2*s^-2` under `L(1,0,0,1,3,1)`, then
    convert that result to `J` under the same tuple
- id: QSpec-TC-187-U26
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 u2` to `u1` under `L(6,0,3,1,12,1)`, then under `L(6,0,2,1,12,1)`
- id: QSpec-TC-187-U27
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Divide `1 cm / 0 cm` under `L(1,0,0,2,2,0)`, then under `L(1,0,0,2,1,0)`
- id: QSpec-TC-187-U28
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Multiply `1 cm * 1 in` under `L(20,0,2,2,11,1)`, then under `L(13,0,2,2,11,1)`,
    then under `L(19,0,2,2,11,1)`
- id: QSpec-TC-187-U29
  path: quire-specification/spec/test-cases/TC-187-quantity-and-unit-conversion.md
  role: examined
  excerpt: Convert `1 in` into an integer target with domain `[-2,2]` under `exact`
    and then `nearest-even`, each under `L(15,0,1,1,4,1)`; convert `3 m` into that
    integer target under `L(2,0,0,1,2,0)`, then under `L(2,0,0,1,1,0)`
- id: QSpec-FR-142-AC-1
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Exact compatible zero-offset conversion followed by arithmetic produces
    the declared result unit and value.
- id: QSpec-FR-142-AC-2
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Incompatible dimensions, including distinct dimension nodes with equal
    base-dimension maps under direct conversion, then affine-unit arithmetic, then
    distinct units where the identical unit is required, refuse `ill_typed` at type
    checking before any charge.
- id: QSpec-FR-142-AC-3
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: A lossy conversion returns its exact loss classification or refuses when
    the target permits no loss.
- id: QSpec-FR-142-AC-4
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Normalized dimension maps remove zero exponents and make multiplication/division
    inverse for admitted quantities.
- id: QSpec-FR-142-AC-5
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Exact affine conversion composes through the unique canonical root; arithmetic
    on a unit with a nonzero composed offset refuses; quantity ordering compares root
    values.
- id: QSpec-FR-142-AC-6
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Multiplication, division and integer power produce the exact value in the
    canonical compound-unit identity, including dimensionless cancellation, `0^0 =
    1` and an undefined zero base under a negative exponent.
- id: QSpec-FR-142-AC-7
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Exact-bound accounting succeeds and denial of a named next unit charge
    returns incomplete without a partial quantity or compound identity; every schedule
    boundary, including decimal-target rounding and membership, occurs at its defined
    charge position.
- id: QSpec-FR-142-AC-9
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Equality or ordering of quantities in distinct units refuses `ill_typed`
    before any charge, while identical-unit equality and ordering compare exact root
    values after the left and then the right root path.
- id: QSpec-FR-142-AC-10
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: A conversion with a compound side compares normalized base-dimension maps,
    traverses only the declared side's root path, and admits a declared-to-compound-to-declared
    pivot in two explicit steps.
```
