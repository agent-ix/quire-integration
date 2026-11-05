---
id: SR-2026110414560098
title: "IR-842 PR 72 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; IR-842 PR 72; original U12/U14/U19/U24 tests and TC-106/IT-019 rows"
review_set: subset
---
## Summary

Reviewed the complete integration change against original public quantity vectors and its actual author evidence. One criterion-level trace metadata gap remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The U12 test checks canonical compound results but its trace attribute names only the parent QSpec-FR-142; add QSpec-FR-142-AC-6 so criterion-level traceability records this evidence. | tests/tc187_u12_arithmetic.rs:18 |

## Verdict

CONDITIONAL. Runtime observations are sound; add the missing criterion trace attribute.

## Coverage

Application version: 0.1.0. Static review only; no builds, tests or final gates rerun. Inspected author compile, focused Clippy, formatting, validation and three passing focused test logs. Formatter reports its existing stable-channel configuration warnings. Author logs are evidence of those runs, not independent gate qualification. No applicable AssuranceProfile was found.

U12 uses four actual checked left subexpressions with owner canonical compound identities, magnitudes 6/3/4/1, dimensions, empty losses, ordered charges and all ten counters. All four generated agreements remain Planned under IR-1348: square derives RequiresBound on genuine scalar_type/integer; product, quotient and dimensionless claims refuse ResultTypeMismatch expected Unit/found compound_unit. No U12 native credit.

U19/U24 generate two Add symbols and compare three complete outcome/loss/ordered-charge/all-counter snapshots at work 5/4/5. Genuine operand keys and nominal metadata are transported through public constructors, with emitted-graph identity joins; no fabricated identity, rewritten IR, local evaluator or charge subtraction appears. The independent QSL authority rechecks original source. U14 checks five original source refusals with exact source spans and generic TypeMismatch, plus five separate owner-operation controls under zero limits with no charges and all counters zero. These controls receive no generated credit. Source locations are asserted separately; scalar native snapshots make no function-envelope location claim.

Only TC-106 and four owned IT-019 disposition rows change; the exclusive dispatcher is untouched. No production, dependency, CI or gate changes occur. Public vector facts are borrowed from the installed qualification dependency; original source is integration-owned. Searched tests/support for native_key, native_units, same_key and original helpers; no existing duplicate identity-transport implementation was found.


Plan completion: not assessed. Quoin 0.29.1 is installed. The supplied Quire 0.38.3 / engine 0.52.1 matrix invocation with the supplied modules emitted engine metadata but no obligation rows, so no computed coverage claim is made. This is a manual scoped AC/evidence check, not a repository-wide gap audit. Optional full semantic audit and plan audit were not run.

## Examined scope

```yaml
scope:
- id: tests/support/tc187_p9_source.rs
  path: tests/support/tc187_p9_source.rs
  role: examined
  excerpt: //! Original quantity source acquisition and public closed-body observations.
- id: tests/tc187_p9_u14_static.rs
  path: tests/tc187_p9_u14_static.rs
  role: examined
  excerpt: //! Original U14 source refusals; admission evidence without runtime/native
    credit.
- id: tests/tc187_u12_arithmetic.rs
  path: tests/tc187_u12_arithmetic.rs
  role: examined
  excerpt: //! Four original U12 arithmetic operations and canonical compound results.
- id: tests/tc187_u19_u24_addition.rs
  path: tests/tc187_u19_u24_addition.rs
  role: examined
  excerpt: //! Original same-unit addition bodies and exact retention boundary.
- id: spec/test-cases/TC-106-quantity-static-admission.md
  path: spec/test-cases/TC-106-quantity-static-admission.md
  role: examined
  excerpt: Qualify original quantity arithmetic and static admission
- id: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: TC-187 U12 generated arithmetic agreement remains Planned under IR-1348.
- id: QSpec-FR-142-AC-6
  path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md
  role: examined
  excerpt: Multiplication, division and integer power produce the exact value in the
    canonical compound-unit identity, including dimensionless cancellation, `0^0 =
    1` and an undefined zero base under a negative exponent.
- id: QSpec-TC-187-U12
  path: quire-specification/spec/test-cases/TC-187
  role: examined
  excerpt: Multiply `2 m * 3 s`, divide `6 m / 2 s`, square `2 m`, and divide `5 m
    / 5 m`
- id: QSpec-TC-187-U14
  path: quire-specification/spec/test-cases/TC-187
  role: examined
  excerpt: Under `L(0,0,0,0,0,0)`, add `1 m + 1 s`, add `1 degC + 1 s`, add `1 degC
    + 1 degC`, divide `1 degC / 0 degC`, and add `1 cm + 1 m`
- id: QSpec-TC-187-U19
  path: quire-specification/spec/test-cases/TC-187
  role: examined
  excerpt: Add `2 m + 3 m` under `L(4,0,0,2,5,1)`, then under `L(4,0,0,2,4,1)`
- id: QSpec-TC-187-U24
  path: quire-specification/spec/test-cases/TC-187
  role: examined
  excerpt: Add `1 cm + 2 cm` under `L(4,0,0,2,5,1)`
```

## Allocation correction

The initial review used SR-2026110414560093 concurrently with another PR. This artifact now uses SR-2026110414560098; its original Linear comment was amended in place to retain one consistent allocation. Finding text and review conclusions are unchanged.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The U12 test now carries QSpec-FR-142-AC-6 in its actual trace attribute. Static inspection confirms the only source change adds this criterion and reformats the attribute. |

### Round 1 verdict

PASS. The sole trace finding is fixed. No build, test or final gate was rerun.

```rust
#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
```
