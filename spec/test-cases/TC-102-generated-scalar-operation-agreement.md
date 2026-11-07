---
id: TC-102
title: "Execute generated Boolean, Integer and Text scalar operations"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: references
---
# TC-102: Execute generated Boolean, Integer and Text scalar operations

## Description

Qualify selected SC-05 ExactScalar operation rows against independently checked
QSL operation bodies. This operation boundary is distinct from FR-021 whole-call
sessions. The same original checked body supplies CG's emitted claim and
public QSL checked-body evaluation after an independent source acquisition.
Neither evaluator is reproduced.

## Test Procedure

1. Check original QSL source, emit its exact checked-package/v2 bytes and strictly
   admit them. Select each function's original body using the shared
   declaration selector. Derive scalar items and require genuine generated claims
   with original node identities.
2. Independently reacquire the source through parsing, assembly and checking.
   Evaluate each corresponding original operation body with fresh global
   allowances and its checked parameter frame where one is declared.
   Run actual emitted ExactScalar native artifacts using the same operands and
   allowances through the existing generated-artifact helper.
3. Exercise Boolean Not and full binary truth tables for And, Or, Implies,
   Equality/Boolean Equal and NotEqual: 22 authored cases across six public
   operation rows with `right_may_stop=false`. Three further original bodies
   put an equality application on the right of And, Or and Implies and require
   `right_may_stop=true`. Their generated right closures fail if invoked on
   the source's short-circuit path. Independently require authored truth values.
4. Compare complete typed operation outcomes, ordered losses, ordered admitted
   charges and all LimitKind consumed counters at generous work and work zero/one.
   Require genuine incomplete WorkUnits terminal records under exhausted work;
   never subtract call overhead, precharge or reconstruct an evaluator envelope.
5. Evaluate eight original parameterized bodies with genuine `Int[-9,9]` slots:
   Add, Subtract, Multiply, Negate, Less, Greater, Integer Equal and
   NotEqual. The arithmetic bodies return `Int[-100,100]`. Execute Multiply at
   both `4 * 2 = 8` and `4 * 7 = 28`; the latter is outside the operand bound
   but inside the separately declared result bound. Supply the same typed values to
   native symbols and check independently expected Integer or Boolean results.
6. Expand each public `TextProfile::ALL` profile over six original
   `Text[0,16;profile]` parameterized bodies: Equal, NotEqual, Less,
   LessOrEqual, Greater and GreaterOrEqual. Admit each authored input through
   the public Text owner before both evaluations, outside their comparison meters.
   Use canonically equivalent `e\u{301}` and `é` for Equal to distinguish the
   four normalization profiles from raw UnicodeScalars and BinaryUtf8. Check
   a second input pair per body at the admitted empty and 16-scalar bounds;
   these pairs cover equality, inequality and ordering at both endpoints.
   Require independent Boolean answers, source locations on exhausted work and the
   complete native/QSL comparison observations.
Other operand kinds, further Text input boundaries and supplied-stop Boolean
paths remain Planned under IR-842; division/modulo remain Planned behind
IR-1391's CG result-bound derivation. These selected cases make no
whole-family claim.

## Expected Results

Every selected generated symbol executes natively and agrees with its independent
QSL operation authority on the complete exposed observation. Independent Boolean
truth values distinguish matching wrong computations. Exhausted
work preserves the typed stop and all accounting without normalization.
At work zero, connectives stop at BooleanResultRetain with next charge 1;
equality stops at EqualityPlanForm with next charge 2. Work one completes every
connective and still stops equality at EqualityPlanForm. Those Boolean stops
have their real locations, no losses, no admitted charges and zero consumption
in every LimitKind counter. All 106 selected cases execute at work 0, 1 and 1024: 318
operation observations. Any row
that does not genuinely derive is reported separately and receives no agreement
credit. Full IT-019-SC-05 and family closure remain Planned under IR-842.
