---
id: TC-102
title: "Execute generated Boolean and bounded Integer scalar operations"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: references
---
# TC-102: Execute generated Boolean and bounded Integer scalar operations

## Description

Qualify selected SC-05 ExactScalar operation rows against independently checked
QSL operation bodies. This operation boundary is distinct from FR-021 whole-call
sessions. The bounded parameter source supplies real emitted operand bounds to
CG; separate closed bodies in the same authored source supply the exact literal
operands to public QSL checked-body evaluation. Neither evaluator is reproduced.

## Test Procedure

1. Check original QSL source, emit its exact checked-package/v2 bytes and strictly
   admit them. Select each bounded function's original body using the shared
   declaration selector. Derive scalar items and require genuine generated claims
   with original node identities and checked bounds for Integer operands.
2. Independently reacquire the source through parsing, assembly and checking.
   Evaluate each corresponding closed operation body with fresh global allowances.
   Run actual emitted ExactScalar native artifacts using the same operands and
   allowances through the existing generated-artifact helper.
3. Exercise Boolean Not and full binary truth tables for And, Or, Implies,
   Equality/Boolean Equal and NotEqual. Exercise bounded Integer Add, Subtract,
   Multiply, Negate, Equality/Integer Equal and NotEqual, and Ordering/Integer
   Less, LessOrEqual, Greater and GreaterOrEqual with unequal and equal operands.
   Include IntegerModulo with nonzero and zero divisors if genuine source derives.
   Independently require authored expected values, not merely mutual agreement.
4. Compare complete typed operation outcomes, ordered losses, ordered admitted
   charges and all LimitKind consumed counters at generous work and work zero/one.
   Require genuine incomplete WorkUnits terminal records under exhausted work;
   never subtract call overhead, precharge or reconstruct an evaluator envelope.
5. Preserve actual typed derivation or admission refusals separately. Integer
   division law selection, unavailable modulo dispatch, other scalar families,
   and unexecuted boundary controls remain Planned under IR-842. Do not label
   unsupported integration dispatch an upstream API blocker or claim a whole
   family from this selected cohort.

## Expected Results

Every selected generated symbol executes natively and agrees with its independent
QSL operation authority on the complete exposed observation. Independent Boolean
truth values and Integer results distinguish matching wrong computations. Exhausted
work preserves the typed stop and all accounting without normalization. Any row
that does not genuinely derive is reported separately and receives no agreement
credit. Full IT-019-SC-05 and family closure remain Planned under IR-842.
