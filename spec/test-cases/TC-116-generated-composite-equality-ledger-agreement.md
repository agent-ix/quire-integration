---
id: TC-116
title: "Compare composite equality outcomes and per-charge ledgers with the generated oracle"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-018
    type: references
---
# TC-116: Compare composite equality outcomes and per-charge ledgers with the generated oracle

## Description

Execute the FR-018 composite equality oracle for tuple, record, nested record,
bounded sequence and nullary union operands against independent QSL authority on the same
original source and the same typed arguments. Compare the typed outcome, every
ordered admitted charge request and every counter, using quire-exact's
production scoped charge capture on both sides.

## Test Procedure

1. Author one original complete-profile source declaring a Boolean tuple, a
   Boolean record, a record nesting that record with an `Int[0,9]` field, a
   `Sequence<Int[0,9]>[0,4]`, a nullary union `Choice { First, Second }` and an
   Integer-payload union `PayloadChoice`. For each shape author an Equal function and a
   NotEqual function comparing two parameters, and closed constructors for
   distinct values of that shape.
2. Check and emit the source, strictly admit the package, select each equality
   body, derive each operand descriptor from its genuine referenced parameter
   node, lower the occurrences, authenticate the contract and invoke public
   composite equality generation with the admitted context.
3. Obtain every argument by running an original constructor through the checked
   package on the host and through the public checked closure in the native
   driver; the native driver reports the constructed arguments and the host
   requires them to match.
4. For each shape and operator, compare each constructor value with itself and
   with each other value. For each pair run the independent checked body on the
   arguments with the production charge capture enabled, and the generated
   oracle with the same capture, at work allowances 1024, 0, 1, 2 and 3. Compare
   the full typed outcome, the ordered admitted requests (point, work, result,
   reservations and ordered size requests), the points of denied requests and all
   ten counters. Require the independent Boolean answer at the ample allowance
   and, at each exhausted allowance, a WorkUnits stop whose charge point is the
   single denied request.
5. At the ample allowance run the QSL call-observing ledger on the same function
   and arguments and require its entries after the leading FunctionCall request
   to equal the admitted requests captured around the checked body.
6. Replace one operand descriptor of one comparison with the other shape's type
   while a second comparison is unchanged; require the typed OperandTypeMismatch
   refusal and an unchanged sibling claim apart from its symbol numbering.

## Expected Results

`generated_oracle_fr018_operation_ledger::composite_equality_ledgers_match_the_generated_oracle`
passes 144 observations: twelve input pairs under both operators, each pair
contributing its reported inputs and five allowance observations. Generated and
independent observations are identical in outcome, admitted per-charge
requests, denied points and counters, and the QSL call ledger agrees at the
ample allowance.
`generated_oracle_fr018_operation_ledger::changed_operand_descriptor_refuses_without_disturbing_a_sibling`
passes the typed refusal control, which gives no execution credit.
The Integer-payload union `PayloadChoice` is refused by CG with the typed
`CompositeContextMismatch`; the test asserts that refusal for both operators and
credits no payload-union execution.
