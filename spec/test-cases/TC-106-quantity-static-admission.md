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
4. Assert IncompatibleDimensions for metre plus second and Celsius plus second;
   AffineUnitArithmetic for Celsius addition and Celsius divided by zero Celsius;
   and DistinctUnits for centimetre plus metre. Require the exact ill-typed
   catalog code, original operation span and source identity for each refusal.
   Check the admitted operand unit identities and Celsius affine offset.

## Expected Results

All five original operations refuse statically at their own original source
occurrence. The affine zero-divisor case never becomes a runtime Undefined
outcome. No executable checked package, evaluation or generated oracle is
published. Runtime charging is unreachable at this source admission boundary;
this does not establish a runtime meter comparison or generated agreement.
Application version: 0.1.0. Execution evidence remains pending until the focused
source-admission test runs successfully.
