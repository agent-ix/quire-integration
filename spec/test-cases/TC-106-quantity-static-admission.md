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
   that original node directly with `evaluate_closed_with_frame`, passing the
   checked function's retained `formed_units` and its genuine zero-argument
   frame; do not cut or reconstruct a body, evaluate the equality wrapper, or
   subtract any charge. Independently require magnitudes 6, 3, 4 and 1.
   Construct expected compound units with the owner's public API on admitted
   root keys and verify exact structural terms
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
and result 1. Integer high water is 4 and all other counters are zero. The
original package emits, strictly admits through IR and yields four `ir_confirmed`
CG claims whose generated native symbols agree with independently rechecked
original operation nodes. Product, quotient, square and dimensionless
cancellation each agree on the completed value, empty losses, ordered charges
and every counter. One-below-retention work limits add four native comparisons
of the exact `UnitResultRetain` outcome and unpaid meter; the independent QSL
refusal location is each original operation locus. No equality-wrapper charges
are included.

`tc187_u19_u24_addition::original_additions_retain_their_units_and_exact_unpaid_retention`
passes three actual QSL/native observations. U19 work 5 and U24 work 5 complete
with five ordered events: two identity reads, arithmetic integer amount 4, target
domain, and retention. U19 work 4 stops at UnitResultRetain, consumed 4, next charge
1, retaining exactly the first four events. Integer high water is 4, occurrences
2, work 5/4/5 and result units 1/0/1; every other counter is zero. All observations
have empty losses; the incomplete original body retains its original location.
These establish the initial three generated addition observations.

The remaining twelve-row extension has the following measured scope. All
executed operation cells independently check the typed answer or exact unpaid
stop, every ordered charge amount, all ten counters, losses and original locus.

| Public ID | Executed evidence | Generated agreement or exact remaining boundary |
| --- | --- | --- |
| U15 | Two original exact conversion observations: 127/50 cm and the second edge denial | Two full original QSL/native comparisons pass |
| U16 | Four original carried Decimal observations: exact refuses at both budgets, nearest-even yields 3/100 m with its exact 127/5000-to-(3,2) loss, or stops before target admission | Planned under [IR-1349](https://linear.app/agent-ix/issue/IR-1349/cg-generate-quantityconvert-into-a-carried-quantity-target-with): each isolated emitted carried source fails strict IR admission before CG |
| U17 | Two original Decimal membership/target-work observations; no retention | Planned under [IR-1349](https://linear.app/agent-ix/issue/IR-1349/cg-generate-quantityconvert-into-a-carried-quantity-target-with), same measured strict-reader precursor |
| U18 | Original analytical DecimalDigits stop: limit 64, consumed 0, next 4294967296; only the source read is admitted | Planned under [IR-1349](https://linear.app/agent-ix/issue/IR-1349/cg-generate-quantityconvert-into-a-carried-quantity-target-with), same measured strict-reader precursor; no enormous coefficient is materialized |
| U20 | Two exact source TypeMismatch refusals and separate zero-limit AffineUnitArithmetic controls; the offset-cancelled sibling yields 3 u2 | One full original QSL/native comparison passes for the positive sibling; admission controls receive no native credit |
| U22 | Nominal N_m-to-J source refusal and separate zero-limit dimension control; two full original let-body values including their power work; isolated original conversion of the genuine QSL-produced 4 square metres and three separate owner conversion-only ledgers | The original conversion subtree checks with its inferred compound parameter type and retained formed-unit frame; its isolated value and ledger agree with the separate owner conversion. Generated conversion agreement remains Planned under [IR-1348](https://linear.app/agent-ix/issue/IR-1348/cgir-generate-quantity-multiplydividecancel-compound-unit-result-and) |
| U23 | Two original distinct-unit source refusals and separate zero-limit owner controls; Celsius order/retention stop, reversing order false and Celsius self-equality true | Four full original QSL/native comparisons pass, with no target-domain or generic equality charge |
| U25 | Direct compound-target source parsing refuses InvalidSyntax at the original asterisk. Two separately labelled owner conversion steps retain the actual intermediate compound value and finish at 1 J, each with read/domain/retain and no edge | Original two-step source/native agreement remains Planned under [IR-1348](https://linear.app/agent-ix/issue/IR-1348/cgir-generate-quantity-multiplydividecancel-compound-unit-result-and): compound target grammar and original compound-input frame authority are absent |
| U26 | Two original exact affine-chain observations: -9 u1 or the third edge denial; the whole three-edge path is retained | Two full original QSL/native comparisons pass |
| U27 | Full total-function source refuses Unproved(Nonzero) at the original division. The authentic original AST admitted through the public checked-expression value API yields DivisionByZero after two reads, or the exact work-1 second-read stop | Generated agreement remains Planned under [IR-1348](https://linear.app/agent-ix/issue/IR-1348/cgir-generate-quantity-multiplydividecancel-compound-unit-result-and); no checked whole-function package is published for this zero-divisor source, and no generated execution is claimed |
| U28 | Three original compound-product observations at integer limits 20, 13 and 19: 127/500000 with root m exponent 2, or exact right-normalization/multiplication size stops | The actual CG claim generates; its native symbol agrees with the original full-value observation in typed outcome, ordered charges and every counter |
| U29 | Four original carried Integer observations: exact inexact-conversion refusal, rounded target-work denial, membership refusal and target-work denial | Planned under [IR-1349](https://linear.app/agent-ix/issue/IR-1349/cg-generate-quantityconvert-into-a-carried-quantity-target-with): each isolated emitted carried source fails strict IR admission before CG |

The carried precursor is observed for seven separately authored complete sources:
exact/rounded Decimal, outside-domain Decimal, huge-scale Decimal,
exact/rounded Integer and outside-domain Integer. Every strict reader returns
InvalidPackage / InvalidDerivedShape at the original scalar_type/carried_quantity
identity. This establishes no CG derivation or renderer execution for those
sources. [IR-1349](https://linear.app/agent-ix/issue/IR-1349/cg-generate-quantityconvert-into-a-carried-quantity-target-with) also owns the later carried-target selector/renderer capability;
fixing only that later stage does not establish the missing reader seam.

The extension executes fifteen original closed-body conversion cells, eight
original closed quantity/comparison cells, two public original checked-expression
zero-division cells, one isolated original compound-conversion cell and ten native comparisons. Two additional full source
compound conversion values include preparation work and are explicitly separate
from the five owner-only isolated compound conversion ledgers. The five separate
zero-limit owner controls are also distinct from source admission.

No whole TC-187 or IT-019-SC-04 completion is implied.
