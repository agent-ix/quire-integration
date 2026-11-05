---
id: TC-102
title: "Execute generated Boolean scalar operations"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: references
---
# TC-102: Execute generated Boolean scalar operations

## Description

Qualify selected SC-05 ExactScalar operation rows against independently checked
QSL operation bodies. This operation boundary is distinct from FR-021 whole-call
sessions. The same original closed checked body supplies CG's emitted claim and
public QSL checked-body evaluation after an independent source acquisition.
Neither evaluator is reproduced.

## Test Procedure

1. Check original QSL source, emit its exact checked-package/v2 bytes and strictly
   admit them. Select each bounded function's original body using the shared
   declaration selector. Derive scalar items and require genuine generated claims
   with original node identities.
2. Independently reacquire the source through parsing, assembly and checking.
   Evaluate each corresponding closed operation body with fresh global allowances.
   Run actual emitted ExactScalar native artifacts using the same operands and
   allowances through the existing generated-artifact helper.
3. Exercise Boolean Not and full binary truth tables for And, Or, Implies,
   Equality/Boolean Equal and NotEqual: 22 authored cases across six public
   operation rows. Boolean variants explicitly use `right_may_stop=false`; the
   `right_may_stop=true` lazy and supplied-stop controls remain Planned. Independently require authored expected truth values rather
   than merely mutual agreement.
4. Compare complete typed operation outcomes, ordered losses, ordered admitted
   charges and all LimitKind consumed counters at generous work and work zero/one.
   Require genuine incomplete WorkUnits terminal records under exhausted work;
   never subtract call overhead, precharge or reconstruct an evaluator envelope.
5. Leave Integer arithmetic/equality/ordering/division/modulo comparison Planned
   under IR-842. The public `FunctionState.body` is a checked node with slots;
   `evaluate_closed` supplies no parameter bindings. The public parameterized
   expression evaluator requires `CheckedExpression`, and the locked API does not
   expose a conversion from the original function body to that expression.
   Comparing a different closed literal body would change the admitted occurrence
   and Integer domains. This is missing integration comparison evidence, not a
   measured derivation refusal or an upstream API blocker. Other scalar families
   and unexecuted boundary controls remain Planned; never claim a whole family.

## Expected Results

Every selected generated symbol executes natively and agrees with its independent
QSL operation authority on the complete exposed observation. Independent Boolean
truth values distinguish matching wrong computations. Exhausted
work preserves the typed stop and all accounting without normalization. Any row
that does not genuinely derive is reported separately and receives no agreement
credit. Full IT-019-SC-05 and family closure remain Planned under IR-842.
