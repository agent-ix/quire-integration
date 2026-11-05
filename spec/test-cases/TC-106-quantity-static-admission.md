---
id: TC-106
title: "Refuse original quantity operations before runtime admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---
# TC-106: Refuse original quantity operations before runtime admission

## Description

Exercise the five U14 operations from the public TC-187 vector through original
QSL source admission. Borrow the public input and expected cells; author original
source and unit declarations. This is admission evidence, with no generated
execution or cross-stage value agreement claim.

## Test Procedure

1. Read U14 through `vectors_of(TestCase::Tc187)` and require its five operation
   inputs, zero runtime allowance and ill-typed static refusal expectation.
2. Parse and assemble each original closed function body with genuine metre,
   second, centimetre and Celsius declarations. Retain actual declaration keys,
   source identity and the public declaration-region map.
3. Check with ordinary compiler CheckingLimits, separately from the all-zero
   runtime ScalarLimits specified by U14. Require each check to refuse before
   a checked package can be linked or offered to evaluation or generation.
4. Require the exact source checker IllTyped(TypeMismatch) cause and catalog
   code, original operation span and source identity for every refusal. QSL's
   source-checking boundary deliberately generalizes quantity-specific causes.
   Check the admitted operand unit identities and Celsius affine offset.
5. Separately pass the corresponding original magnitudes and genuine admitted
   unit graph identities to the public quantity operation API with every runtime
   limit zero. Require IncompatibleDimensions for the two dimension mismatches,
   AffineUnitArithmetic for Celsius addition and division by zero Celsius, and
   DistinctUnits for centimetre plus metre. Require no admitted charge and zero
   consumption in all ten counters. These owner operation controls are separate
   from source admission and do not establish generated agreement.

## Expected Results

All five original operations refuse statically at their own original source
occurrence. The affine zero-divisor case never becomes a runtime Undefined
outcome. No executable checked package, evaluation or generated oracle is
published. Runtime charging is unreachable at this source admission boundary;
the separate zero-limit owner operation controls establish their own zero-charge
refusals, without generated agreement.
Application version: 0.1.0. The focused
`tc187_p9_u14_static::original_u14_operations_refuse_at_their_source_occurrence`
test passes all five original source admissions and all five zero-limit owner
operation controls. Every source refusal retains `ill_typed/type-mismatch` and
the full original operation span; all owner operation charges and counters are
zero. U14 receives no generated-oracle agreement credit.
