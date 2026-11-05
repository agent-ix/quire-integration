---
id: TC-108
title: "Observe TC-187 source quantity refusals and native charge precedence"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
---
# TC-108: Observe TC-187 source quantity refusals and native charge precedence

## Description

Exercise public TC-187 U03, U05 and U09 through genuine QSL source acquisition.
Keep source checking refusals distinct from supplementary native value refusals.

## Test Procedure

1. Bind each public vector's complete input and expected strings by reference.
   Parse, form and assemble original source; retain the admitted unit graph and
   original unit identities. For U09 admit a selected domain model through public
   model intake, independently of the local dimension owner.
2. Check incompatible length/time addition, every combination of affine Celsius
   and Fahrenheit with addition, subtraction, multiplication, division and power,
   and addition of roots whose base dimensions share the name L but have distinct
   actual semantic keys. Require exact typed causes and complete source locations.
3. Independently invoke the public native quantity operation using the assembled
   units and original unit IDs, under ample and zero allowances. Require the exact
   incompatible-dimension or affine-arithmetic refusal, an empty ordered charge
   sequence and zero consumption for all ten counters. Inspect the two U09
   dimension maps, equal names, unit scales and offsets without constructing keys.

## Expected Results

U03 and U09 refuse source checking with type-mismatch. All ten U05 cases refuse:
ordinary arithmetic has type-mismatch and power has operator-ineligible. Each
refusal identifies the original arithmetic expression and owning function.
The separate native operation preserves IncompatibleDimensions or
AffineUnitArithmetic before charging even when every allowance is zero.

These observations establish source admission/refusal and native value refusal
precedence only. Invalid source produces no checked selected operation, no
runtime Outcome and no generated-oracle agreement. SC-04 remains Planned beyond
this bounded cohort; model normalization and checking limits are separate from
value accounting.
