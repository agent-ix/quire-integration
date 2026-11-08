---
id: TC-112
title: "Compare original carried quantity conversion with native conversion"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
---
# TC-112: Compare original carried quantity conversion with native conversion

## Description

Exercise public TC-187 U07 and U08 through original checked QSL conversion
bodies and independently invoked native carried conversion. Source names the
inch unit `inch` because `in` is reserved; its scale is exactly 127/5000 metres.
The generated carried-target and rounding agreement is executed by
`tc187_p9_remaining_conversions` and recorded in IT-019; complete SC-04 remains Planned.

## Test Procedure

1. Bind the public U07/U08 identifiers and complete input/expected cells.
   Parse, form, assemble, check and link original complete-profile source.
   Select its original closed function body and genuine admitted unit table.
2. Evaluate each original body with public evaluate_closed and a fresh meter.
   Independently invoke convert_quantity with its actual admitted units and
   QuantityTarget::Carried Decimal carrier on identical fresh limits.
3. For U07 use exact and nearest-even Decimal[-100,100;2,2] targets.
   Assert native coefficient, scale and every DecimalLoss field before
   mechanically normalizing native Decimal to the QSL quantity representation.
   Check full outcomes, located losses/refusals, ordered charges and all ten
   counters. Exercise work four target-domain denial and result-retain denial.
4. For U08 use Decimal[-2,2;0,0;exact] with -2, 2, -3 and 3 metres.
   Negative operands are genuine rational(-2,1) and rational(-3,1) quantity
   literals. Check exact boundary values, atomic outside-domain refusal and
   fresh work-one target-domain denial before membership.

## Expected Results

U07 nearest-even produces coefficient 3, scale 2 and loss with exact numerator
127, denominator 5000, rounded coefficient 3, scale 2 and mode nearest-even.
Exact carried conversion refuses InexactConversion retaining the complete
Decimal target before target-domain, including at work four. Nearest-even
stops at target-domain with work four and at retention with no result allowance.

U08 completes at -2 and 2 and refuses DecimalOutOfDomain at -3 and 3 after
target-domain and before retention, with no value or loss. Work one denies
target-domain before membership. Every source/native comparison preserves
complete typed outcome and accounting without subtracting call charges.
The generated carried conversion leg is executed in TC-106's conversion tests and IT-019.
