---
id: IT-019
title: "Compare executed generated oracles with independent QSL authority"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-018
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-021
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-485
    type: references
---

# IT-019: Compare executed generated oracles with independent QSL authority

## Objective

Establish integration-owned agreement between the QSL value authority and
executed, generated codegen oracles for the applicable scalar, composite
equality and function application families. A generated claim map, a direct
runtime call, or a proof verdict alone does not establish this agreement.
Keep QSL admission, IR admission, CG selection and native execution as
separate observed boundaries. IT-017's QSL-versus-runtime scalar results and
IT-013's single integer native result do not discharge this procedure.

## Target Integration

QSL public parse, check and checked-package emission → exact emitted V2 bytes
→ Contract IR public strict admission → CG public scalar, composite equality
or exact-function generator → compile and call the actual generated native
oracle symbol. Independently reacquire the original QSL source, select the
checker-issued owner and occurrence, and evaluate it through fresh public QSL
authority. FR-021 uses whole-call FR-485 sessions; FR-014 and FR-018 may use
public checked-expression evaluation for complete operation observations, as
defined in SC-05 below. For TC-187, obtain authored vector IDs and prose through
`quire_specification::vectors_of(TestCase::Tc187)` and
`QUALIFICATION_VECTORS`, by reference. Integration owns executable dispatch
for those IDs; it neither copies the corpus nor implements QSL, IR or CG
semantics. The public vector API's prose is not itself an executable request.

## Preconditions

The integration Cargo.lock resolves one coherent QSL, Contract IR, CG and
QSpec cohort. The Rust toolchain can compile the generated crate locally.
The native path invokes generated code without Kani or a combined proof/native
executor. For each run, both sides receive the same admitted claim occurrence,
operands, domain, type, limits and fresh meter; the QSL comparison authority
does not register the generated body or derive its expected outcome from the
generated source. Unsupported or unavailable owner behavior remains Planned
with the exact missing public path and an open owner ticket. A Planned row is
not an agreement result.

## Inputs

The first executable source is the exact unit supplied by
`actual_generated_function_preserves_the_whole_qsl_call_envelope`:

```qsl
language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function combine using v(left: Int[0,9], right: Int[0,9]): Integer pure { left + right }
function exercise using v(left: Int[0,9], right: Int[0,9]): Integer pure { combine(left,right) }
```

Call `combine` with Integer arguments 4 and 3 under global work limits 0, 1
and 1024. Select the real `combine` declaration, body and `exercise` call
occurrence from QSL's checked package; do not fabricate node keys or substitute
an interpreted body.
Include a healthy sibling and a separately refused item for binding controls.
Subsequent cases select applicable TC-187 rows from the public vector API and
applicable CG FR-014 scalar, FR-018 composite equality, and FR-021 function
application families. Each row is identified by owner vector ID or CG family
and operation, and split into distinct admission and evaluation observations
where the owner row contains both.

## Test Procedure

1. Compile the `combine` source through QSL's public checked-package path,
   admit its exact emitted bytes with Contract IR's public reader, derive the
   real function, body and call roots, generate through CG's public exact
   function API, compile the produced crate, and invoke its actual
   `GeneratedExactFunctionClaim::oracle_symbol`. In a separately acquired
   FR-485 session for the original source, look up the checker-issued
   callable and apply it with the same arguments and a fresh meter at work=1.
   Compare the complete outcome variant and payload, `Location`, ordered
   admitted charges, every exposed consumed counter and denied-charge fields
   directly, without normalizing or subtracting the function-call envelope.
   IT-019-SC-01: both paths return the same `Incomplete` with
   `WorkUnits`, limit 1, consumed 1, next charge 1 and
   `IntegerArithmeticOperands`; their one admitted `FunctionCall` charge,
   all counters and `Some(Location { Body { function: "combine", index: 0 },
   depth: 0, last_index: None })` agree exactly. A missing generated location,
   a synthetic QSL evaluator, or a nonexecuted oracle fails this criterion.
2. Re-run that identical selected function and arguments with fresh sessions
   and meters at work=0 and work=1024. Compare the entire outcome, location,
   ordered charges and every counter for each limit; retain any located losses
   in the completed envelope.
   IT-019-SC-02: work=0 stops at `FunctionCall` before body work with no
   admitted charge and the same real body location; work=1024 completes with
   integer 7 and identical located losses, charge sequence and counters.
   Neither result can compensate for a failure at work=1.
3. Submit genuine wrong-owner and wrong-occurrence selections through the
   public QSL/IR/CG boundaries, then submit a mixed request containing a
   refused item and a healthy sibling. Preserve each stage's typed refusal,
   original request index, source owner and occurrence; invoke no oracle for
   a refused item.
   IT-019-SC-03: a mismatch is refused before generated execution or meter
   charge, the refusal is distinguishable from a QSL lookup or input refusal,
   and the healthy sibling's generated symbol, disposition and independent
   QSL comparison are unchanged. No replacement identity or index is minted.
4. Enumerate TC-187 through QSpec's public vector API. For each authored ID,
   classify its distinct operations as admission-only, eligible generated
   evaluation, or Planned with a named open owner ticket and missing path.
   Dispatch eligible rows to authentic QSL source → emitted bytes → IR
   admission → relevant CG public generation → executed native oracle and
   independent QSL evaluation. Compare full results, typed refusal or
   incompleteness, locations, exposed ordered charges and counters. Check the
   QSpec-stated expected result as an additional oracle where it specifies
   one; never treat admission-only rows as generated value agreement.
   IT-019-SC-04: every API row and required operation has one explicit
   classification and observed path or ticket-backed Planned entry; unknown
   IDs, silent skips, copied fixtures and ticketless Planned entries fail.
   Only executed two-sided rows with matching owner expectations count as
   generated-oracle/QSL agreement.
5. Select the required operation set from the public owner types resolved by
   this integration's Cargo.lock, rather than a copied catalog or fixed count:
   - FR-014 selects every variant of CG `ExactScalarOperation`, expanded by
     every operator and operand-kind variant in its fields. Include all
     owner-defined division profiles, IEEE widths, comparisons and rounding
     choices, text profiles, and both values of Boolean `right_may_stop` where
     meaningful for the connective. Domain values and boundary cases follow
     FR-014 and the selected TC-187 expectations, rather than an enumeration
     of arbitrary numeric endpoints. Preserve owner-defined invalid
     combinations as typed refusals, not silently omitted operations.
   - FR-018 selects both `EqualityOperatorKind` variants for every operand
     declaration and collection shape consumed by the public composite
     equality generator, including recursively reached shapes and leaf kinds
     defined by the owner `CompositeShape` and `ValueType` contracts. Expand
     `EqualityOperandDescriptor` source-only and conversion-target forms on
     each operand wherever the owner permits them; unsupported combinations
     retain their own typed dispositions.
   - FR-021 selects every `ExactFunctionBody` variant, expanded by its integer
     operator or equality operator and owner-permitted operand shapes, and
     the nested-call form with a genuine selected callee. Include the
     argument, result, owner, occurrence and capability boundaries required
     by FR-021 for each form.
   For every selected operation/shape/choice, record a distinct row naming its
   public owner variant, genuine QSL source-to-IR-to-CG route and executable
   test symbol. Each row must contain an executed two-sided result, an
   observed typed nonexecuted generation disposition with sibling isolation,
   or `Planned` with the exact missing admission/generation/authority path
   and an open owner ticket. Unimplemented integration dispatch is owned by
   IR-842; it is not attributed to an unavailable upstream semantic feature.
   A new owner variant adds a required row; a missing or unknown variant
   fails inventory closure, rather than inheriting another row's result.
   IT-019-SC-05: every operation selected by this owner-derived rule has an
   explicit disposition. Runnable cases compare the full exposed envelope
   against independent QSL authority. FR-014 and FR-018 operation qualification
   may use fresh, independently acquired public checked-expression evaluation
   of the same admitted source occurrence, operands, domain and type. Compare
   every public operation outcome, loss, ordered admitted charge and consumed
   counter without subtraction or precharging. This supplies operation evidence,
   distinct from FR-021's FunctionCall-level FR-485 session envelope; it supplies
   no callable owner or session-location evidence. A missing complete public
   observation leaves that row Planned with the exact missing API path. A
   different source body with mismatched Integer domains is not a substitute.
   A typed-refused or unsupported selection
   is never converted to a Boolean QSL outcome, and a passing scalar sample
   cannot mark the composite or function family complete.
6. Publish the per-vector and per-family evidence table with status
   `Passed`, `Failed` or `Planned`, test symbol, observed boundary, and owner
   ticket for each Planned entry. Mark a criterion passed only when all of its
   required executable controls pass; keep the overall broad agreement
   Planned while any required row or family remains Planned or Failed.
   IT-019-SC-06: the report distinguishes shared-kernel value equality from
   independent admission, selection and actual generated execution evidence;
   it makes no whole-family or whole-TC-187 claim from a minimal sample.

## Expected Results

The native generated artifact and independent public QSL authority agree directly on
each runnable case's complete exposed result and accounting. The work=1
`combine` location is a required equality, not a tolerated difference.
Refused and unsupported items produce no executed oracle, and healthy siblings
retain their identities and outcomes. Unexecuted rows remain visibly Planned
with owner tickets; any mismatch, silent omission or fabricated replacement
fails the relevant criterion.

## Coverage Status

Coverage is partial. Update each row only from an executed test symbol and
observed result. The native-only `function_agreement::actual_generated_function_preserves_the_whole_qsl_call_envelope`
test executes the actual exported generated symbol against a fresh independent
FR-485 session. Its complete typed results, locations, located losses, ordered
charges and every counter agree at global work limits 1, 0 and 1024. At work=1
both preserve `Body { function: "combine", index: 0 }`, depth 0 and no last
index, with the exact `IntegerArithmeticOperands` incomplete payload. Work=0
retains the body location and no admitted charge; work=1024 returns Integer 7.
The same test rejects a valid subtraction source substituted for the selected
addition package with `CheckedFunctionError::OwnerIdentityMismatch` before any
charge, then successfully executes the original source. This single invocation
after a source refusal establishes the owner boundary. The separate SC-03
mixed-request test below establishes occurrence refusal and healthy-sibling
isolation. The current disposition is:

| Scope | Status | Evidence or remaining work |
| --- | --- | --- |
| `combine` work=1, 0 and 1024 | Passed | `function_agreement::actual_generated_function_preserves_the_whole_qsl_call_envelope`: actual generated native symbol and independent FR-485 whole-envelope equality, including explicit work=1 body location, all three limits, exact ordered charges and every counter |
| Foreign-source owner mismatch followed by original invocation (SC-03 owner control) | Passed | The same test observes typed `OwnerIdentityMismatch`, no admitted charge and zero counters, then the healthy original call |
| Owner and occurrence mismatch; refused item and healthy sibling (SC-03) | Passed | `generated_oracle_sc03::wrong_occurrence_refuses_without_changing_the_healthy_native_call` uses two real QSL call occurrences and an admitted three-root contract: wrong-occurrence arguments yield a typed per-item `BodyMismatch`, while the healthy sibling retains the same generated claim, symbol and emitted library as standalone generation. A wrong full request position yields typed `AccountingIdentity` with the original accounting index and exact IDs; a wrong item node at a valid position yields typed `Identity` with the original request index and exact IDs. Both fail before a claim map exists. The healthy public symbol executes against a fresh QSL session with whole-envelope and value agreement at work=0, 1 and 1024. The existing foreign-source owner test observes typed `OwnerIdentityMismatch` and zero charge. |
| TC-187 U06 zero-scale and U11a two-root admission-only cells | Passed | `tc187_vector_dispatch::public_tc187_ids_fail_closed_and_real_admission_controls_refuse` reads both IDs and owner expectations from `vectors_of(TestCase::Tc187)`, then observes each real QSL source's typed topology refusal, retained identity/value and selected source span before a checked package or generated oracle exists |
| TC-187 U12 four original quantity arithmetic values | Passed (QSL source values only) | TC-106 / `tc187_u12_arithmetic::public_u12_arithmetic_values_have_their_canonical_compound_units` borrows public facts and evaluates actual original checked subexpressions without wrapper charges or AST reconstruction: magnitudes 6, 3, 4 and 1; genuine canonical root terms and base-dimension powers, including empty dimensionless maps; empty losses, exact ordered charges and every counter. Multiplication/divisions use work 5, occurrences 2, result 1; square uses work 4, occurrences 1, result 1. Integer high water is 4; other counters are zero. |
| TC-187 U12 generated arithmetic agreement | Planned | IR-1348; measured typed boundaries in TC-106: square derivation returns RequiresBound on the actual scalar_type/integer exponent. Multiplication and both divisions derive, but their actual CG claims return ResultTypeMismatch with expected Unit and found compound_unit. All four QSL value cells pass, but none has generated execution credit. IR-1348 owns both measured generation frontiers. |
| TC-187 U19 same-metre addition and unpaid retention; U24 same-centimetre addition | Passed (generated agreement for three observations) | TC-106 / `tc187_u19_u24_addition::original_additions_retain_their_units_and_exact_unpaid_retention` checks and emits direct closed quantity-returning bodies, strictly admits original IR, generates two actual Add symbols and compares three native observations against independently rechecked original bodies: U19 at work 5 returns 5 m, work 4 stops at UnitResultRetain with consumed 4/next 1; U24 at work 5 returns 3 cm. Full typed outcomes, losses, ordered charge records and all ten counters agree. Integer high water 4, occurrences 2, work 5/4/5, result units 1/0/1, all other counters zero. Actual admitted operand identities and nominal metadata supply native inputs; no call/child charge subtraction or fabricated identities. |
| TC-187 U14 five static refusals | Passed (admission and owner operation controls) | TC-106 / `tc187_p9_u14_static::original_u14_operations_refuse_at_their_source_occurrence` borrows the public input and expectation cells, checks five original sources and observes exact QSL `IllTyped(TypeMismatch)` / `ill_typed/type-mismatch` refusals at each original operation span and source identity. Genuine admitted units retain centimetre identity/scale and Celsius root/offset. Separate public quantity operations on those admitted identities observe two IncompatibleDimensions, two AffineUnitArithmetic (including Celsius division by zero), and one DistinctUnits refusal under all-zero runtime limits, with no admitted charges and every counter zero. These are source admission and owner operation controls; no generated execution or QSL/native agreement credit. |
| TC-187 U11b/c/d/f/l topology | Passed | `tc187_u11_topology::public_u11_target_refusals_retain_identity_and_source_order` observes five distinct real QSL refusals with exact retained edges, names, dimensions, catalog codes and source spans. The dispatcher reads each ID and expectation through public `vectors_of(TestCase::Tc187)` and fails closed on an unknown ID. These are admission controls, not generated-oracle equality. |
| TC-187 U11e nonidentity roots and U11g1/g2/g4 derived dimensions | Passed | `tc187_u11_local_values::public_u11e_nonidentity_roots_retain_scale_and_offset` observes both separate real U11e sources' typed refusal with retained unit, reduced scale, offset, code and source span. `tc187_u11_local_values::public_u11g_derived_maps_refuse_unanchored_l_and_admit_anchored_controls` observes the three real base-dimension refusals at `L` with exact carrier unit, then independently anchors `L` and observes the admitted `L^2`, `L^3` and `L^4` maps through the public unit graph. The dispatcher reads the four public IDs and expectations and fails closed on unknown IDs. These are admission controls, not generated-oracle equality. |
| TC-187 U11h/i/j/k/m first-defect order | Passed | `tc187_u11_order` reads each public ID and expectation and exercises six real QSL assemblies. U11h retains `z` and zero scale before its unknown target; U11i retains the earlier `a` and unknown target before a later zero scale; U11j retains dimension `L` and both roots before the self-cycle; U11k retains the earlier dimension's self-cycle before a later zero scale; U11m reverses dimension declaration order and observes `Area`'s ordered two-edge cycle or the unanchored base `L`. Every refusal asserts its typed cause, retained identity or value, catalog code and selected source span. These are admission controls, not generated-oracle equality. |
| TC-187 U11s/t/u imported unit topology | Passed | `tc187_u11_imported_topology` reads QSL's authored model document by reference, adds the published imported dimension and root unit in memory, and admits that model through the public selection intake. U11s retains both local cycle edges; U11t retains the model-owned `P.L` and imported root `P.m` with the distinct local second root; U11u retains local `z` and its zero scale. Each real QSL assembly refusal asserts its typed first cause, catalog code and selected source span, with the public vector ID and expectation checked in the dispatcher. These are admission controls, not generated-oracle equality. |
| TC-187 U11r imported root and local centimetre | Passed | `tc187_u11_imported_positive::public_u11r_imported_root_checks_and_emits_with_exact_scale` reads the public U11r vector, selects an authoritative QSL model document by reference, adds its dimension `L` and terminal root `m` in memory, and admits local `cm` through public parse, model intake, assembly, check and checked-package emission. It asserts the model-owned `L` and `m` identities, `cm`'s exact canonical scale `1/100` and zero offset, checked payload links and emitted structural nodes. This is QSL admission and emission evidence, not generated-oracle operation agreement. |
| TC-187 U11n/o/p/w dimension-term first defects | Passed | `tc187_u11_dimension_terms` binds all four public vector IDs to their full authored input and expected cells, then assembles real QSL sources derived from those inputs. U11n retains ordered term edges `A -> B`, `B -> A`; U11o retains the `D -> D` self-edge; U11p and U11w retain the owning dimensions `D` and `A` with unresolved term `K` at its exact source span, before any later unit or cycle check. Each case asserts the catalog code and selected source span. These are admission controls, not generated-oracle equality. |
| TC-187 U11q/x/z3 document and transitive first defects | Passed | `tc187_u11_multidoc_transitive` binds all three public vector IDs to their full authored input and expected cells and executes both alternatives of each row. U11q assembles two distinct QSL source documents in each request order: a padded first T document refuses zero scale at `z`, while first L refuses the retained `u_self -> u_self` target cycle. U11x retains the owning dimension `D` and unknown term `K` without `e`, then selects `e`'s zero scale when added. U11z3 selects `e`'s zero scale before the reached cycle, then without `e` retains ordered `A -> B`, `B -> A` edges at `A`. Each refusal asserts typed cause, exact catalog code and selected source span. These are admission controls, not generated-oracle equality. |
| TC-187 U11y/z1/z2 imported base-dimension order | Passed | `tc187_u11_imported_base` checks each public vector's exact authored input and expected result, then reads QSL's authored model fixture by reference and constructs the selected P/Q dimensions and unit in memory. The public model intake and assembler report typed `UnanchoredBaseDimension` with imported `P.L` identity, first local carrier `m2`/`m2`/`x`, `missing_declaration/missing-name` and the P selection span. U11z1's additional two-root defect in P.Area produces the first `conflicting-definition` before P.L's missing-name; U11z2's additional unitless Q.T still produces P.L's missing-name first when both imported bases are defective. The public U11z2 case separately shows P.L before the later zero-scale t. These are admission controls, not generated-oracle equality. |
| TC-187 U30/U31 quantity type checks | Passed (QSL check only) | `tc187_u30_u31` binds both full public vector cells and exercises real QSL parse, form, assembly and check. U30 admits `f(3 ms)` with the `ms` unit node as the parameter type, refuses `f(3 s)` at the argument with typed `ill_typed/type-mismatch`, and refuses `Quantity<T>`, `Quantity<nothing>` and bare `ms` at each missing unit/type name during assembly. U31 admits returning `d` with the same `ms` node in parameter and result types, then refuses `convert<Quantity<s>>(d)` at the returned expression. Check is a static admission stage and invokes no value operation or meter. These are check controls, not generated-oracle equality. |
| TC-187 U13/U21 integer-power QSL value controls | Passed for QSL value authority; generated agreement Planned | `tc187_u13_u21_power` binds both complete public vector cells. A source-checked package call reports U13's exact `2^65` denied rational charge, including `IntegerBits` and the over-`u64` next amount. Public checked-expression evaluation reports U21's exact dimensionless one and ordered read/power/target/retention charges, the zero-to-negative-power undefined result after one read, and the work-zero incomplete before that read. IR-842 still owns the absent QSL-to-IR-to-CG generated quantity-power and native comparison path. |
| TC-187 remaining public vector IDs and mixed-row operations | Planned | IR-842: dispatched public IDs without a Passed row above still need independent operation controls; U13/U21 additionally lack generated quantity-power comparison. U11g3 stays Planned under QSL-1738: real QSL assembly admits `dimension N = L / L; unit one : N = 1`, but the public `UnitGraph` gives `one` map `N^1`, contradicting TC-187's required empty map. U01 remains Planned: QSL-1737 owns missing public checked-package bounded quantity type emission, and IR-1345 AC4 owns the remaining source-to-CG agreement after its carried_quantity lowerer fix. The dispatcher retains ticket-backed placeholders for these rows. Operation-level dispositions, generated execution and QSL comparison remain unimplemented. IR-669 owns separate QSL/runtime scalar evidence |
| FR-014 Boolean Not/And/Or/Implies with `right_may_stop=false`; Equality/Boolean Equal and NotEqual | Passed | TC-102 / `generated_oracle_family_scalar_operations::selected_scalar_operations_match_independently_checked_qsl_bodies`: 22 truth cases at work 0, 1 and 1024 compare complete operation outcomes, losses, ordered charges and every counter against fresh public evaluation of the same closed source body, with independent typed truth and stop expectations |
| FR-014 remaining operations and Boolean lazy controls | Planned | IR-842: `right_may_stop=true`, parameterized Integer comparisons and the other scalar operation groups need their own complete observations. The public closed-body entry cannot bind function parameters; different literal-body Integer domains receive no agreement credit. IR-669 QSL/runtime results supply no generated-oracle credit |
| FR-018 Equal and NotEqual / closed Boolean literals | Passed | TC-104 / `generated_oracle_family_function_equality::primitive_composite_generator_matches_independent_checked_bodies`: 24 complete native operation observations against independently checked original bodies, with independent Boolean truths and exact stops |
| FR-018 Equal and NotEqual / closed Integer literals | Passed | TC-104: 18 complete native operation comparisons over six original inline-literal equal/less/greater cells at three allowances; admitted Literal type IDs equal the actual Integer descriptor type, with independent Boolean answers and exact stops. A changed Boolean descriptor retains OperandTypeMismatch with the original Integer type found and an exactly unchanged healthy sibling claim and binding, without refusal execution credit |
| FR-018 Equal and NotEqual / original protocol finish Boolean Pair binder with itself | Passed | TC-105 / `generated_oracle_family_protocol_equality::original_protocol_finish_equality_retains_its_source_occurrence`: six complete native operation comparisons against independently checked original protocol-slot bodies and authentic binder frames, including exact original location correspondence and work stops; genuine owner metadata constructs native inputs without a TypeEnvironment |
| FR-018 remaining shapes, conversions and operation boundaries | Planned | IR-842; QSL-1739 tracks remaining operation-ledger support. Original function-body local bindings need their own public entry (QSL-1736 for Integer); public original protocol-slot frames already exist and TC-105 exercises one nominal cohort. Remaining record/union/collection, distinct tuple inputs, recursive/conversion and refusal rows need their own authored matching stage/descriptor/native observations. Direct closed constructor-body charges cannot be subtracted |
| FR-021 Subtract, Multiply and genuine subtraction-callee Call cohort | Passed | TC-103 / `generated_oracle_family_function::subtraction_multiplication_and_nested_call_match_fresh_qsl_sessions`: nine whole-call envelopes, twelve argument refusals and three alternate-owner refusals; exact results, locations, located losses, charges and every counter |
| FR-021 CompositeEquality Equal and NotEqual / Boolean, Int[0,9], Boolean Pair tuple, Boolean Flag record and nullary Choice union | Passed | TC-104 / `generated_oracle_family_function_equality::primitive_and_nominal_function_equality_matches_fresh_qsl_sessions`: 78 whole-call envelopes, 32 input refusals and ten changed-owner refusals; authentic source-constructor nominal inputs, independent Boolean answers, exact locations/losses/charges and every counter |
| FR-021 Scalar Negate and equality result/parameter/capability/arity/occurrence/duplicate/ambiguity/declaring-node requests | Passed (nonexecuted) | TC-104: actual unary source UnsupportedOperator plus eight typed generation-boundary controls, each retaining a healthy nominal generated sibling; no execution credit for refused items |
| FR-021 remaining shapes and per-form boundaries | Planned | IR-842: deeper nominal closures, nonempty union payloads and remaining per-form operating controls need their own genuine source constructors, admitted nominal arguments and executed whole-call evidence. FR-485 supports this route independently of the FR-018 operation-ledger gap; passed cohorts supply no whole-family credit |

Unsupported generation is a typed nonexecuted observation in its own row;
it cannot be credited as two-sided value agreement. A partial passing slice
updates only the specific rows it executes, leaving the rest Planned with
their exact owner tickets in the implementation report.

## Traceability

This test composes CG FR-014 and FR-018 with public QSL operation authority,
CG FR-021 with QSL FR-485 whole-call authority, and the QSpec TC-187 vector source. Test symbols trace to the applicable
`IT-019-SC-NN` criterion and owner family or vector ID. IT-017 and IT-013
remain separate evidence for their narrower paths.
