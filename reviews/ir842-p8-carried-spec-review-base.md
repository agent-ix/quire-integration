---
id: SR-2026110414560112
title: "spec-review/base of P8 carried conversion"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; spec/test-cases/TC-112-tc187-carried-conversion.md; tests/tc187_p8_carried_conversion.rs"
review_set: subset
---

## Summary

Ticket: IR-842. PR: quire-integration#77. Scoped independent review of the two new files. No actionable defects found.

## Verdict

**PASS** for the scoped change. Generated agreement and complete SC-04 remain Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Both public U07/U08 complete input and expected cells are bound through the installed public vector API. Original authored source passes parsing, forms, assembly, checking and linking. Unit IDs come from original nominal spans and the admitted unit table; the selected checked body retains the actual Decimal carrier and target unit. Native convert_quantity uses QuantityTarget::Carried and fresh identical limits. Public evaluate_closed executes the original operation body without a FunctionCall envelope or charge subtraction.

The two tests execute fourteen comparisons: six U07 and eight U08. Independent expected assertions inspect native Decimal representation before normalization, all five DecimalLoss fields, the full exact InexactTarget::Decimal refusal, both signed endpoint values and exact DecimalOutOfDomain target. Stops inspect charge point, limit kind, consumed and next charge. Explicit ordered charge-point lists detect missing/extra stages; full source/native admitted-charge equality and iteration over LimitKind::ALL compare complete charges and all ten counters. Completion checks original source quantity and admitted target unit; source losses and failure locations must equal the original body location. Refused/incomplete outcomes propagate untouched, and empty loss vectors exclude leaked partial loss. The adapter only transports Decimal to rational quantity after native representation/loss assertions; it does not erase refusal kind, target, stage or location.

Mutation reasoning: wrong rounded coefficient/scale/loss fails direct assertions; wrong exact refusal target fails typed equality; completing outside the domain fails exact refusal equality; premature membership/retention fails ordered points; an extra FunctionCall fails the full ordered native/source ledger; wrong source carrier or target unit fails original checked-type/identity checks. No local conversion algorithm, copied checked JSON, reminted owner keys, fabricated meter, shared helper edit or dependency/gate change exists. Negative literals use rational(-2,1) and rational(-3,1), avoiding quantity-negation work.

Base checklist applied to every new TC-112 procedure and expected-result unit and the implemented evidence. Public authority is referenced, not redeclared. TC-112 is unique in this reviewed scope and references existing IT-019. The references relationship is not a prerequisite/dependency-policy change. No FR/NFR/StR statements, domain objects, structural schema or requirement prerequisite graph changed: EARS, integrity, dependency and other lenses are inapplicable to this test-case addition.

Inspected supplied executed receipts: focused test reports 2 passed, 0 failed, 0 ignored; scoped Clippy finishes successfully with warnings denied; formatting has only existing stable-channel import-option warnings; TC-112 validation has only the module inverse-edge advisory. Reviewer performed no builds, gates, census, Kani or optional semantic fanout. Lead owns ONE final scoped gate. Shared dispatcher, IT-019, support, manifest and lock remain unchanged. Generated carried-target/rounding agreement and complete SC-04 remain Planned; no generated execution credit is awarded.

## Examined Units

```yaml
scope:
- id: TC-112:unit-1
  path: spec/test-cases/TC-112-tc187-carried-conversion.md
  role: examined
  excerpt: 'Exercise public TC-187 U07 and U08 through original checked QSL conversion

    bodies and independently invoked native carried conversion. Source names the

    inch unit `inch` because `in` is reserved; its scale is exactly 127/5000 metres.

    This is source/native evidence only. Generated carried target and rounding

    agreement and complete SC-04 remain Planned.'
- id: TC-112:unit-3
  path: spec/test-cases/TC-112-tc187-carried-conversion.md
  role: examined
  excerpt: "1. Bind the public U07/U08 identifiers and complete input/expected cells.\n\
    \   Parse, form, assemble, check and link original complete-profile source.\n\
    \   Select its original closed function body and genuine admitted unit table.\n\
    2. Evaluate each original body with public evaluate_closed and a fresh meter.\n\
    \   Independently invoke convert_quantity with its actual admitted units and\n\
    \   QuantityTarget::Carried Decimal carrier on identical fresh limits.\n3. For\
    \ U07 use exact and nearest-even Decimal[-100,100;2,2] targets.\n   Assert native\
    \ coefficient, scale and every DecimalLoss field before\n   mechani"
- id: TC-112:unit-5
  path: spec/test-cases/TC-112-tc187-carried-conversion.md
  role: examined
  excerpt: 'U07 nearest-even produces coefficient 3, scale 2 and loss with exact numerator

    127, denominator 5000, rounded coefficient 3, scale 2 and mode nearest-even.

    Exact carried conversion refuses InexactConversion retaining the complete

    Decimal target before target-domain, including at work four. Nearest-even

    stops at target-domain with work four and at retention with no result allowance.'
- id: TC-112:unit-6
  path: spec/test-cases/TC-112-tc187-carried-conversion.md
  role: examined
  excerpt: 'U08 completes at -2 and 2 and refuses DecimalOutOfDomain at -3 and 3 after

    target-domain and before retention, with no value or loss. Work one denies

    target-domain before membership. Every source/native comparison preserves

    complete typed outcome and accounting without subtracting call charges.

    The generated carried conversion leg remains Planned.

    '
- id: bind
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn bind(
- id: checked
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn checked(
- id: limits
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn limits(
- id: decimal_type
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn decimal_type(
- id: compare
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn compare(
- id: rational
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn rational(
- id: assert_decimal
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn assert_decimal(
- id: stop
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn stop(
- id: u07_original_carried_exact_and_nearest_even_preserve_loss_and_precedence
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn u07_original_carried_exact_and_nearest_even_preserve_loss_and_precedence(
- id: u08_original_carried_boundaries_refuse_atomically_after_target_domain
  path: tests/tc187_p8_carried_conversion.rs
  role: examined
  excerpt: fn u08_original_carried_boundaries_refuse_atomically_after_target_domain(
```
