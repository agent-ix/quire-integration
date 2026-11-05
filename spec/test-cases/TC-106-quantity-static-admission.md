---
id: TC-106
title: "Qualify original quantity arithmetic and static admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---
# TC-106: Qualify original quantity arithmetic and static admission

## Description

Exercise U12 arithmetic, U19/U24 same-unit addition and U14 static refusals from
the public TC-187 vectors. Borrow the input and expected cells and author original
source. Separate source-value evidence, source admission, owner operation controls,
typed generation refusals and actual generated execution.

## Test Procedure

1. Read the four public IDs through `vectors_of(TestCase::Tc187)`. Bind each
   operation, expected value or stop and allowance through small factual selectors;
   do not duplicate the public prose or a fixture table.
2. Check each U14 original closed source with ordinary compiler CheckingLimits,
   separately from runtime ScalarLimits. Require exact source checker
   IllTyped(TypeMismatch) and catalog `ill_typed/type-mismatch`, original source
   identity and complete operation span. Require checking to refuse before a
   checked package can be published for evaluation or generation. QSL deliberately
   generalizes quantity-specific source causes at this boundary.
3. Separately pass the U14 original magnitudes and genuine admitted unit graph
   identities to the public quantity operation API with all ten limits zero.
   Require two IncompatibleDimensions, two AffineUnitArithmetic (including
   Celsius division by zero) and one DistinctUnits refusal. Require empty admitted
   charges and zero consumption in every counter. Assert admitted centimetre
   scale/identity and Celsius root/offset.
4. For U12 check full original Boolean equality function bodies and select each
   actual left checked subexpression through the public child accessor. Evaluate
   that original node directly with evaluate_closed; do not cut or reconstruct a
   body, evaluate the equality wrapper, or subtract any charge. Independently
   require magnitudes 6, 3, 4 and 1. Construct expected compound units with the
   owner's public API on admitted root keys and verify exact structural terms
   and base-dimension powers, including the empty dimensionless map. Check full
   ordered charge records, empty losses and all ten counters.
5. For U19 and U24 check direct original quantity-returning function bodies.
   Require U19 magnitude 5 in metres at work 5 and the exact unpaid retention
   stop at work 4; require U24 magnitude 3 in centimetres at work 5. Compare each
   full ordered charge record and all ten counters, with no edges and no losses.
   The stopped original body must retain its genuine source location.
6. Emit the original checked packages unchanged and strictly admit their IR.
   Select the actual operation reference for each original body/subexpression.
   Derive scalar items and retain every typed refusal at its measured stage.
   Generate the eligible items. Independently recheck the same original source.
7. For generated additions, obtain operand values from genuine checked literal
   nodes outside compared meters. Transport actual producer-owned structural key
   components and nominal metadata through public NodeKey, DimensionNode,
   UnitNode and UnitGraph constructors; resolve quantities through UnitTable.
   Require each declared operand key to join the admitted emitted graph. Invoke
   each actual generated symbol with fresh identical limits and compare full typed
   outcomes, losses, ordered admitted charges and every counter against the
   independently checked original closed body. No call/child charge subtraction,
   TypeEnvironment, fabricated identity, rewritten IR or replacement evaluator.

8. Extend the original-source slice to U15–U18, U20, U22/U23 and U25–U29.
   Bind each split operation and allowance through borrowed public facts. For
   conversion, comparison, arithmetic and carried-target cases, assert independent
   typed numeric/domain/stop expectations, exact original loci, losses, complete
   ordered charge records and every counter. Keep static refusals separate from
   executable checked bodies. U18 must stop on analytical target size without
   materializing its enormous coefficient.
9. Attempt unchanged original checking, emission, strict reading, scalar
   derivation/generation and actual native execution for each eligible family.
   Preserve exact carried-target, compound-result or other typed refusals as
   Planned generated agreement. Do not attribute carried targets to bounded
   parameter policy. Reuse admitted original nominal metadata and actual checked
   operands in the native caller; no charge subtraction or source reinterpretation.
10. For U22/U25 verify authentic compound input/target syntax and public body
    support before claiming source agreement. If the exact source seam is absent,
    record its measured refusal and retain separately labelled public owner
    conversion controls on genuine admitted compound units. Never substitute a
    power computation with extra charges, a nominal direct bridge, or a fabricated
    checked body. Report a missing public seam for owner routing.

## Expected Results

Application version: 0.1.0.

`tc187_p9_u14_static::original_u14_operations_refuse_at_their_source_occurrence`
passes five source-admission and five separate zero-limit owner operation
controls. The source causes are generic TypeMismatch; the owner operation causes
retain the detailed dimension/affine/unit distinctions. Celsius division by zero
refuses statically instead of reaching Undefined. These controls receive no
generated-agreement credit.

`tc187_u12_arithmetic::public_u12_arithmetic_values_have_their_canonical_compound_units`
passes all four QSL source-value observations with exact compound terms, dimensions,
empty losses, ordered charges and counters. Multiplication and both divisions each
consume work 5, occurrences 2 and result 1; square consumes work 4, occurrences 1
and result 1. Integer high water is 4 and all other counters are zero. Generated
agreement remains Planned under IR-1348 for all four operations: square derivation
refuses RequiresBound on its actual scalar_type/integer exponent; the other three
derive but their actual CG claims refuse ResultTypeMismatch with expected Unit and
found compound_unit. IR-1348 owns the compound-result and integer-exponent generation frontiers.

`tc187_u19_u24_addition::original_additions_retain_their_units_and_exact_unpaid_retention`
passes three actual QSL/native observations. U19 work 5 and U24 work 5 complete
with five ordered events: two identity reads, arithmetic integer amount 4, target
domain, and retention. U19 work 4 stops at UnitResultRetain, consumed 4, next charge
1, retaining exactly the first four events. Integer high water is 4, occurrences
2, work 5/4/5 and result units 1/0/1; every other counter is zero. All observations
have empty losses; the incomplete original body retains its original location.
These are the only generated quantity agreements claimed by this artifact.

This artifact establishes no credit for other TC-187 operations or whole
IT-019-SC-04 completion.

The remaining twelve-row extension is specified but its execution evidence is
pending. A per-operation result must be measured before any additional IT-019
status is upgraded; no whole SC-04 completion is implied.
