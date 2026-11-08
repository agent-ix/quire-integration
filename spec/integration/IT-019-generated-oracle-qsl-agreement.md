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
4. Enumerate TC-187 through QSpec's public vector API and select the rows
   required by CG FR-014-AC-6, AC-48 and AC-61 through AC-65. Classify each
   selected operation as admission-only, eligible generated evaluation, or
   Planned with its exact missing path and owner ticket. Other public vector
   IDs do not become IR-724 generated-agreement obligations merely by appearing
   in the QSpec export; U39 quantity negation is outside the current CG
   `QuantityOperator` and the FR-014 acceptance set.
   Dispatch eligible rows to authentic QSL source → emitted bytes → IR
   admission → relevant CG public generation → executed native oracle and
   independent QSL evaluation. Compare full results, typed refusal or
   incompleteness, locations, exposed ordered charges and counters. Check the
   QSpec-stated expected result as an additional oracle where it specifies
   one; never treat admission-only rows as generated value agreement.
   IT-019-SC-04: every AC-selected row and required operation has one explicit
   classification and observed path or ticket-backed Planned entry; silent
   skips, copied fixtures and ticketless Planned entries fail.
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
     The `TextAdmission` descriptor is caller ingress, outside this
     source-backed operation comparison: CG FR-014 defines no checked text
     admission application and QSL emits a `Text("...")` literal as `value`.
     Its owner dispositions belong to CG TC-024, not this matrix.
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
| TC-187 U12 generated arithmetic agreement | Passed (four completed and four unpaid-retention native observations) | TC-106 / `tc187_u12_arithmetic::public_u12_arithmetic_values_have_their_canonical_compound_units` emits the original checked package, strictly admits it through IR, derives four `ir_confirmed` CG claims, and executes their actual generated symbols. Product, quotient, square and dimensionless cancellation agree with independently rechecked original QSL operation nodes in typed outcome, empty losses, ordered charge records and every counter at full work limits and one below retention. The four full results have exact magnitudes 6, 3, 4 and 1 and canonical compound identities, including empty dimensionless terms. The four bounded native outcomes match QSL's `UnitResultRetain` stop and unpaid meter; the QSL refusal location is the original operation locus. No equality-wrapper charge is subtracted or inferred. |
| TC-187 U19 same-metre addition and unpaid retention; U24 same-centimetre addition | Passed (generated agreement for three observations) | TC-106 / `tc187_u19_u24_addition::original_additions_retain_their_units_and_exact_unpaid_retention` checks and emits direct closed quantity-returning bodies, strictly admits original IR, generates two actual Add symbols and compares three native observations against independently rechecked original bodies: U19 at work 5 returns 5 m, work 4 stops at UnitResultRetain with consumed 4/next 1; U24 at work 5 returns 3 cm. Full typed outcomes, losses, ordered charge records and all ten counters agree. Integer high water 4, occurrences 2, work 5/4/5, result units 1/0/1, all other counters zero. Actual admitted operand identities and nominal metadata supply native inputs; no call/child charge subtraction or fabricated identities. |
| TC-187 U15 exact inch-to-centimetre and U26 full affine-chain conversion | Passed (four generated observations) | TC-106 / `tc187_p9_remaining_conversions::original_conversion_boundaries_preserve_values_losses_and_unpaid_events`: original closed bodies check/emit/strictly admit/derive/generate. Actual native symbols agree on full typed outcomes, losses, ordered charges and all counters: U15 127/50 cm at L(15,0,2,1,9,1) or second-edge denial at L(7,0,1,1,9,1); U26 -9 u1 at L(6,0,3,1,12,1) or third-edge denial at L(6,0,2,1,12,1). All edges precede arithmetic; the complete three-edge U26 path is retained. |
| TC-187 U16 four, U17 two, U18 one and U29 four carried conversion operation cells | Passed (eleven QSL observations only) | The same TC-106 conversion test checks original carried-source bodies, exact target domains, inexact/member refusals, Decimal rounding value and exact loss, analytical huge-scale denial and every work stop. Full ordered charge records, every counter and exact original locations are independently checked. These eleven source observations carry QSL-only credit; native agreement is in the generated carried row below. |
| TC-187 U07, U08, U16, U17, U18 and U29 generated carried agreement | Passed (eighty native observations on current cohort) | TC-106 / `tc187_p9_remaining_conversions::original_conversion_boundaries_preserve_values_losses_and_unpaid_events`: ten original carried-target conversion bodies (Decimal exact, nearest-even, out-of-domain, below/low-edge/high-edge boundaries with authored `rational(-3,1) m`, `rational(-2,1) m` and `rational(2,1) m` operands, the `4294967295`-scale target, and Int exact, rounded and outside) are emitted unchanged, strictly admitted, derived as `CarriedQuantityConversion` and generated. Each runs at a generous grid, work 0, 1, 4 and 5, plus the vector-stated limits `L(15,5,1,1,6,1)`, `L(15,5,1,1,4,1)`, `L(2,1,0,1,2,0)`, `L(2,1,0,1,1,0)`, `L(u64::MAX,64,0,1,2,1)`, `L(15,0,1,1,4,1)`, `L(2,0,0,1,2,0)` and `L(2,0,0,1,1,0)`. The native symbol agrees with the independently rechecked original QSL conversion body in typed outcome (value, atomic domain refusal, inexact refusal, incomplete stop), ordered Decimal losses, ordered charges and every counter. |
| TC-187 U20 composed affine siblings | Passed (two admission/owner controls and one generated observation) | TC-106 / `tc187_p9_remaining_quantity::original_static_controls_and_public_zero_division_expression_keep_distinct_axes` observes exact source TypeMismatch causes/spans and separate all-zero AffineUnitArithmetic ledgers for u1 addition and u3 multiplication. `original_quantity_boundaries_preserve_composed_units_and_adverse_stops` independently checks cancelled offset zero, exact 3 u2 and no edges, then compares the actual generated Add symbol against the independent original body. Static refusals receive no native credit. |
| TC-187 U23 affine/reversing comparisons | Passed (four generated observations; two separate admission/owner controls) | The same TC-106 quantity tests observe source TypeMismatch at each distinct-unit comparison and separate zero-charge DistinctUnits controls. Four native comparisons agree fully with independent original bodies: Celsius order true at work 9, exact retention stop at work 8, reversing-root order false and Celsius self-equality true. Independent answers, all event amounts, no target-domain/generic equality events, all counters and original loci are asserted. |
| TC-187 U22 nominal mismatch and compound conversion controls | Passed (separate admission, full-source value and owner-only axes) | TC-106 / `genuine_compound_values_retain_separate_source_and_owner_conversion_evidence` retains the actual QSL-produced 4 square metres and checks two whole original let-body values including power work, plus three distinct owner conversion-only ledgers: 4 m2, 40000 cm2 and integer limit 16 stopping at the divide event with consumed 5/next 17. The static-control test separately asserts nominal N_m-to-J source refusal and zero-charge owner dimension refusal. |
| TC-187 U22 isolated original compound conversion | Passed (original QSL conversion-only value and ledger) | TC-106 / `genuine_compound_values_retain_separate_source_and_owner_conversion_evidence` cuts the original conversion subtree, checks it against the genuine inferred compound parameter type, and evaluates it with the original produced 4 square metres in the retained formed-unit frame. The isolated 4 m2 result and conversion-only ordered charges and counters agree with the independently checked owner conversion. No native conversion credit. |
| TC-187 U25 compound-to-joule conversion | Passed (five generated observations) | TC-106 / `tc187_ir842_conversions::u25_original_compound_to_joule_conversion_matches_generated_native_oracles` converts the original `1 kg * 1 m ^ 2 * 1 s ^ -2` product (the vector's `kg*m^2*s^-2`) to `Quantity<J>`. CG derives and generates every operation of the body: the two powers, the two multiplications and the conversion. The native caller runs those generated symbols in order on one fresh meter and QSL evaluates the same checked body, so QSL's expected 1 J, the full outcome, ordered charges and every counter agree at the vector tuple `L(1,0,0,1,3,1)`, a generous grid and work 0, 1 and 2; a stop inside an operand operation is a stop of a generated symbol. |
| TC-187 U25 nominal-to-compound first step | Planned (IR-1403) | The original `Quantity<kg*m^2*s^-2>` conversion target does not parse (InvalidSyntax at the compound unit expression), so the first step N_m to compound has no source form. This is a QSL source-grammar seam owned by IR-1403 (compound unit grammar) and stays required in FR-014-AC-6 until it lands; the compound-to-J step above and the owner-only pivot ledger below keep their separate credit. |
| TC-187 U25 explicit compound pivot owner controls | Passed (owner-only two-step ledger) | TC-106 / `genuine_compound_values_retain_separate_source_and_owner_conversion_evidence` converts actual admitted N_m to the genuine compound kg*m^2*s^-2 unit, carries that exact returned value/identity into the second conversion to J, and independently requires 1 at both steps, final original J identity and each read/domain/retain ledger with no edge and all counters. No nominal direct shortcut or original-source/native agreement is claimed. |
| TC-187 U27 zero-divisor priority | Passed (public original checked-expression operation authority; separate full-package refusal) | TC-106 / `original_static_controls_and_public_zero_division_expression_keep_distinct_axes` preserves full total-function Unproved(Nonzero) and the exact original division span. It separately supplies the authentic original AST to public check_expression/evaluate: work 2 yields DivisionByZero after both identity reads and before edges, work 1 denies the second read with consumed 1/next 1. Complete outcomes, losses, original expression loci, ordered charges and every counter are asserted without call/child subtraction. |
| TC-187 U28 mixed-unit product values and size stops | Passed (three original QSL observations only) | TC-106 / `original_quantity_boundaries_preserve_composed_units_and_adverse_stops` evaluates the original checked product at L(20,0,2,2,11,1), L(13,0,2,2,11,1) and L(19,0,2,2,11,1). It independently asserts 127/500000, original m root exponent 2, complete left-then-right edge/arithmetic ledgers and exact size denials consumed 9/next 14 or consumed 15/next 20, with all counters and source loci. |
| TC-187 U28 generated agreement | Passed (three native observations on current cohort) | TC-106 / `original_quantity_boundaries_preserve_composed_units_and_adverse_stops` emits the unchanged original product, strictly admits it, derives a generated CG claim, and executes its native symbol. At L(20,0,2,2,11,1), the native 127/500000 result with root m exponent 2 agrees with the independently checked original QSL product. The L(13,0,2,2,11,1) and L(19,0,2,2,11,1) native size denials also agree with their independent original QSL observations in typed outcome, empty losses, ordered charges and every counter. |
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
| TC-187 U13/U21 integer-power QSL value controls | Passed for QSL value authority | `tc187_u13_u21_power` binds both complete public vector cells. A source-checked package call reports U13's exact `2^65` denied rational charge, including `IntegerBits` and the over-`u64` next amount. Public checked-expression evaluation reports U21's exact dimensionless one and ordered read/power/target/retention charges, the zero-to-negative-power undefined result after one read, and the work-zero incomplete before that read. The U21 `0 m ^ -1` total-function source is refused by QSL check as `Unproved(Nonzero)` before package emission; its public checked-expression value controls do not create a CG claim. |
| TC-187 U13 huge power and U21 zero-to-zero generated agreement | Passed (two current-cohort native observations) | TC-106 / `tc187_u13_u21_power::public_u13_huge_and_u21_zero_power_match_generated_native_oracles` binds both public vectors, emits genuine checked equality-body operation children, strictly admits them and executes their actual CG quantity-power symbols. The U13 over-`u64` denied `UnitRationalArithmetic` amount and U21 exact dimensionless one agree with independent original QSL operations in complete outcome, losses, ordered charges and every counter. |
| TC-187 U04 affine conversions and U09b scale-one alias | Passed (nine generated observations) | TC-106 / `tc187_ir842_conversions::u04_and_u09b_original_conversions_match_generated_native_oracles`: original `convert<Quantity<K>>(0 degC)`, `convert<Quantity<K>>(32 degF)` and `convert<Quantity<m>>(7 metre_alias)` bodies generate `QuantityConversion` claims whose native results agree with independent QSL evaluation (5463/20 K twice, 7 m) at work 1024, 0 and 1, including every ordered charge, loss and counter. |
| TC-187 U10 exact tuple and named final denial | Passed (two generated observations) | The same TC-106 target converts `100 cm` to `m` under `L(8,0,1,1,6,1)`: result 1 m with the 8-bit multiply amount, and with `unit.result-retain` denied on both the QSL meter and the native meter. Native and QSL agree on complete outcome, ordered charges and every counter. |
| TC-187 U02 normalization and U22 compound area conversion | Passed (21 generated observations) | TC-106: `1 m ^ 0 = 1 m ^ 0` and `(1 m * 1 s) / 1 s` generate Power, Multiply and Divide claims, and `convert<Quantity<cm2>>(2 m ^ 2)` and `convert<Quantity<m2>>(2 m ^ 2)` generate the Power and Conversion claims, at `L(17,0,1,1,6,1)`, `L(16,0,1,1,6,1)` and a generous grid. Each computed operand operation runs through its own generated symbol on the same native meter, so every observation, including a stop inside an operand, executes generated code and compares the whole original body with QSL. QSL's own values are asserted: 1 for `1 m ^ 0` and the quotient, 40000 cm2 and 4 m2 for the conversions. |
| TC-187 U01 carried convert-then-add | Planned | IR-1401 (CG seam): `convert<Quantity<m>>(a) + 1 m` over a `Quantity<cm, Rational[0,1000;1,1]>` parameter derives `QuantityConversion` and `QuantityArithmetic`, but CG generation refuses the conversion claim `OperandTypeMismatch { position: 0, expected: Unit, found: "carried_quantity" }` (`check_operand` in codegen `src/oracle/scalar/lower.rs` admits only `unit` and `compound_unit` operand forms). A bare `Quantity<cm>` parameter instead derives `RequiresBound`. No generated or native agreement is credited. |
| TC-187 U21 zero-to-negative and U27 zero-divisor generated agreement | Planned (QSL, unreachable) | The vector's `undefined` outcome is unreachable from QSL source: QSL check refuses every divisor or base not proved nonzero as `Unproved(Nonzero)` before emission, and a divisor proved nonzero cannot be zero, so no CG claim exists. The refusal stays a typed control (rows above). Positive nonzero-divisor evidence is the existing literal-divisor cells (quantity division by a literal quantity, Decimal literal-divisor division). Owner: QSL language (no change planned). |
| FR-014 Boolean Not/And/Or/Implies with `right_may_stop=false`; Equality/Boolean Equal and NotEqual; short-circuit `right_may_stop=true` | Passed (selected controls on current cohort) | TC-102 / `generated_oracle_family_scalar_operations::selected_scalar_operations_match_independently_checked_qsl_bodies`: 22 original truth cases and three original nested-right short-circuit cases at work 0, 1 and 1024 compare complete outcomes, losses, ordered charges and every counter against fresh QSL evaluation. The three native right closures fail if invoked; independent typed truths and stops are asserted. No supplied-right stop credit. |
| FR-014 Integer Add/Subtract/Multiply/Negate, Less/LessOrEqual/Greater/GreaterOrEqual, Equal/NotEqual over bounded operands | Passed (35 input cases from ten bodies on current cohort) | TC-102 / `generated_oracle_integer_operations::original_integer_operations_agree_with_generated_native_observations`: ten genuine `Int[-9,9]` parameterized source bodies are emitted, strictly admitted and generated; arithmetic results are separately bounded by `Int[-100,100]` and the actual nested operation is selected beneath QSL's checked narrowing node. Each body runs its original input plus operand-endpoint cases (for example `9 + 9`, `-9 - 9`, `-9 * 9`, `-(-9)`, and equal, reversed and endpoint ordering pairs), and every case runs at work 0, 1 and 1024. Independent evaluation of each original whole body agrees with its generated native symbol in typed result, losses, ordered charges and every counter. Multiplication covers `4 * 2 = 8` and `4 * 7 = 28`, the latter outside the operand bound but inside the result bound. |
| FR-014 Rational Add/Subtract/Multiply/Negate/Divide, Integer-operand Divide and Less/LessOrEqual/Greater/GreaterOrEqual | Passed (29 input cases from ten bodies, 87 native observations) | TC-102 / `generated_oracle_rational_operations::original_rational_arithmetic_agrees_with_generated_native_observations`: `Rational[-9,9;1,9]` operand bodies (and `Int[-9,9]` by `Int[1,9]` for the IntegerDivide form) with a separately declared `Rational[-200,200;1,200]` result derive `RationalArithmetic` or rational `Ordering` claims. Noninteger fractions and operand endpoints (`1/2 + 1/3 = 5/6`, `9 + 9`, `-9 * 9`, `1/2 / 3/4 = 2/3`, `-9 / 1/9 = -81`, `7 / 2 = 7/2`, and equal, reversed and endpoint ordering pairs) agree with independent QSL evaluation at work 0, 1 and 1024 in typed outcome, ordered charges and every counter. |
| FR-014 Decimal Add/Subtract/Multiply/Negate/Divide, Round under every rounding mode, and Less/LessOrEqual/Greater/GreaterOrEqual | Passed (39 input cases from 17 bodies, 195 native observations) | TC-102 / `generated_oracle_decimal_operations::original_decimal_operations_agree_with_generated_native_observations`: original `Decimal[-1000,1000;0,2;exact]` arithmetic and ordering bodies, literal-divisor division bodies (rounded, exact and out-of-domain; QSL requires a provably nonzero divisor), and `convert<Decimal[-9,9;0,0;mode]>` bodies for exact, toward-zero, toward-positive, toward-negative, nearest-even and nearest-away over 2.5 and -2.5. Each runs at work 1024, 0, 1, 2 and 3; native results, located Decimal losses, domain and inexact refusals, ordered charges and every counter agree with QSL, and the authored coefficient and scale are asserted for completed cases. |
| FR-014 IEEE Add/Subtract/Multiply/Divide for Float32 and Float64 under every rounding mode, and width conversions in both directions | Passed (1584 native observations) | TC-102 / `generated_oracle_ieee_operations::original_ieee_arithmetic_and_width_conversion_agree_with_generated_native_observations`: 48 arithmetic bodies and 12 conversion bodies are checked with the selected IEEE profile, emitted, strictly admitted and generated. Nine operand pairs (ordinary, inexact, overflowing, subnormal, zero, infinity and not-a-number) and eight conversion operands run at work 1024, 0 and 1; native values, IEEE flag losses, exact-mode refusals, ordered charges and every counter agree with QSL, and the `1 op 3` bit patterns are asserted. |
| FR-014 IEEE NumericEqual, TotalOrder and BitIdentical comparisons | Planned | IR-1401 (CG seam): derivation succeeds (`IeeeComparison { comparison, width: Binary32 }`) for `a = b` and `quire::value::ieee::numericEqual`, `totalOrder` and `bitIdentical`, but generation refuses every claim with `FormMismatch { expected: "binary", found: "call" }` (the `binary_call` shape in codegen `src/oracle/scalar/lower.rs`, whose own comment records that real producers emit `call`). No native agreement is credited. |
| FR-014 Quantity Add/Subtract/Multiply/Divide/Power and Equal/NotEqual/Less/LessOrEqual/Greater/GreaterOrEqual over the AC-6 unit grid | Passed (78 arithmetic and 234 comparison bodies, 1560 native observations) | TC-106 / `tc187_ir842_conversions::quantity_arithmetic_operators_over_the_unit_grid_match_generated_native_oracles` and the two `quantity_comparison_operators_over_the_*_grid_match_generated_native_oracles` targets. The AC-6 grid units `m`, `cm`, `in` (declared `inch`), `rev`, `u2` and `s` are all present: Add/Subtract over m, cm, inch, kg, s, rev and u2; mixed-unit Multiply/Divide; Power with exponents 0, 1, 2, 3, -1 and -2 over m, cm, inch, s and kg; and all six comparisons over m, cm, inch, s, kg, K, rev, degC, degF, u2, J, m2 and cm2. Each runs at a generous grid and work 0, 1, 3 and 5 (6 for comparisons). Distinct-unit comparisons and arithmetic on offset units are statically ill-typed and are covered by the refusal controls, not executed. |
| FR-014 Enum comparison | Planned | FR-014-AC-2 names "enum comparison" among its operation families, which covers enum equality (`quire.op.enum.eq` / `ne`); the language has no ordering on enums. QSpec e53e9487 publishes the standalone `acme/colors` model (`ACME_COLORS_DOMAIN_PACKAGE`) and native source with all four equality and inequality truth arms (`ACME_COLORS_COMPARISON_SOURCE`). `tests/generated_oracle_enum_equality.rs` admits them through QSL and IR by reference, and CG derivation then refuses each of the four comparisons with `BlockedOnUpstream { node_tag: "model", issue: QuireSpecLanguage120 }` on the `variant_type` node. No generated or native enum cell exists; the refusal is a typed control. Owner: CG (the `BlockedOnUpstream` enum-comparison path); ticket IR-1401 (CG batch carrying the model `variant_type` QSL-120 route). |
| FR-014 Boolean And/Or/Implies with the right operand evaluated | Planned | IR-1402 (CG seam): the generated connective takes `right: impl FnOnce() -> Outcome<bool>` with no meter, while QSL charges the right operation (for example `EqualityPlanForm`) on the shared meter, so the ordered ledger cannot agree for an evaluated right. Only the skipped-right controls above are credited. |
| FR-014 Text Equal/NotEqual/Less/LessOrEqual/Greater/GreaterOrEqual under every `TextProfile::ALL` profile | Passed (72 input cases from 36 original bodies on current cohort) | TC-102 / `generated_oracle_text_comparison::original_text_comparisons_agree_with_generated_native_observations`: every `Text[0,16;profile]` comparison body runs both its original input pair and an empty/16-scalar boundary pair, each independently admitted outside fresh comparison meters. Native symbols agree with the original QSL bodies in typed Boolean result, empty losses, ordered charges and every counter at work 0, 1 and 1024; independent truths and work-zero source loci are asserted. Equal uses canonically equivalent `e\u{301}` and `é` to distinguish normalization from raw profiles. |
| FR-014 Truncating, Floor and Euclidean Integer division and Euclidean modulo | Planned | [IR-1391](https://linear.app/agent-ix/issue/IR-1391): an original `: Integer` source checks but CG scalar derivation refuses `MissingBound` because no distinct result `integer_range` is reachable. An `Int[-9,9]` division or `Int[-100,100]` modulo result is refused by QSL checking as `Unproved(Range)` before emission, with proved interval `None..None`. No source-backed generated operation comparison is credited. |
| FR-014 remaining operations and boundaries | Planned | Additional Text input boundaries beyond the empty and 16-scalar pairs. The other Planned entries keep their own rows and owners: IEEE comparisons and U01 (IR-1401), evaluated-right connectives (IR-1402), U25 first step (IR-1403), enum equality (CG `BlockedOnUpstream` row), U21 negative power and U27 (QSL, unreachable), Integer division and modulo (IR-1391). IR-669 QSL/runtime results supply no generated-oracle credit. |
| FR-018 Equal and NotEqual / closed Boolean literals | Passed | TC-104 / `generated_oracle_family_function_equality::primitive_composite_generator_matches_independent_checked_bodies`: 24 complete native operation observations against independently checked original bodies, with independent Boolean truths and exact stops |
| FR-018 Equal and NotEqual / closed Integer literals | Passed | TC-104: 18 complete native operation comparisons over six original inline-literal equal/less/greater cells at three allowances; admitted Literal type IDs equal the actual Integer descriptor type, with independent Boolean answers and exact stops. A changed Boolean descriptor retains OperandTypeMismatch with the original Integer type found and an exactly unchanged healthy sibling claim and binding, without refusal execution credit |
| FR-018 Equal and NotEqual / original protocol finish Boolean Pair binder with itself | Passed | TC-105 / `generated_oracle_family_protocol_equality::original_protocol_finish_equality_retains_its_source_occurrence`: six complete native operation comparisons against independently checked original protocol-slot bodies and authentic binder frames, including exact original location correspondence and work stops; genuine owner metadata constructs native inputs without a TypeEnvironment |
| FR-018 Equal and NotEqual / original Boolean Pair tuple, Boolean Flag record, nested Flag record with an `Int[0,9]` field and bounded `Sequence<Int[0,9]>[0,4]` | Passed | TC-116 / `generated_oracle_fr018_operation_ledger::composite_equality_ledgers_match_the_generated_oracle`: 120 observations, ten authentic constructor-input pairs under both operators at work 1024, 0, 1, 2 and 3; the independent checked body and the generated oracle agree in typed outcome, ordered admitted per-charge requests (points, work, results, reservations and ordered size requests), denied points and every counter using quire-exact's production scoped capture, and the QSL call ledger matches after its leading FunctionCall request |
| FR-018 Equal and NotEqual / changed operand descriptor | Passed (nonexecuted) | TC-116 / `generated_oracle_fr018_operation_ledger::changed_operand_descriptor_refuses_without_disturbing_a_sibling`: an `OperandTypeMismatch` refusal with an unchanged healthy sibling claim, without execution credit |
| FR-018 Equal and NotEqual / union operands (nullary `Choice`, `PayloadChoice`) | Planned | CG defect, owner ticket required: the generated composite equality crate for any union operand does not compile (`rebuild_union` passes `&str` to `UnionMemberDeclaration::resolved`, which takes an `Identifier`), observed on CG main; no union operation is executed |
| FR-018 recursive leaves and `convert<T>` conversion operands | Planned | No authored source cell yet; IR-1381 follow-up |
| FR-021 Subtract, Multiply and genuine subtraction-callee Call cohort | Passed (current cohort) | TC-103 / `generated_oracle_family_function::subtraction_multiplication_and_nested_call_match_fresh_qsl_sessions`: twelve whole-call envelopes including unbounded-result 4 × 7 = 28, twelve argument refusals and three alternate-owner refusals; exact results, locations, located losses, charges and every counter |
| FR-021 CompositeEquality Equal and NotEqual / Boolean, Int[0,9], Boolean Pair tuple, Boolean Flag record, nullary Choice union and Integer-payload PayloadChoice union | Passed (current cohort) | TC-104 / `generated_oracle_family_function_equality::primitive_and_nominal_function_equality_matches_fresh_qsl_sessions`: 96 whole-call envelopes, 38 input refusals and twelve changed-owner refusals; authentic source-constructor nominal inputs include same payload, changed payload and changed member, with independent Boolean answers, exact locations/losses/charges and every counter |
| FR-021 Scalar Negate and equality result/parameter/capability/arity/occurrence/duplicate/ambiguity/declaring-node requests | Passed (nonexecuted) | TC-104: actual unary source UnsupportedOperator plus eight typed generation-boundary controls, each retaining a healthy nominal generated sibling; no execution credit for refused items |
| FR-021 remaining shapes and per-form boundaries | Planned | IR-842: deeper nominal closures and remaining per-form operating controls need their own genuine source constructors, admitted nominal arguments and executed whole-call evidence. FR-485 supports this route independently of the FR-018 operation-ledger gap; passed cohorts supply no whole-family credit |

`TextAdmission` is caller ingress outside this source-backed operation matrix.
CG FR-014 states that a caller admits a `TextPayload` under a `TextType` before
comparison, with no checked `quire.op.numeric.convert` application for text.
QSL FR-093-AC-50 lowers `Text("a")` to `value`/`literal`; selecting it for a
scalar oracle therefore yields `NotApplication`. CG's own TC-024
`tc_024_text_admission_corpus_is_refused_by_ir_today` records the six text
admissions' typed IR refusals as owner evidence. The passing TextComparison
rows above begin with independently admitted operands and claim only comparison.

The current exact cohort is QSL `2ed49b65`, IR `4f0ef630`, driver
`b96a51a1`, CG `cd184a37` and QSpec `b09800f7`. The selected Passed rows
above were executed on this cohort; Planned rows receive no generated
agreement credit from prior dependency locks.

Unsupported generation is a typed nonexecuted observation in its own row;
it cannot be credited as two-sided value agreement. A partial passing slice
updates only the specific rows it executes, leaving the rest Planned with
their exact owner tickets in the implementation report.

## Traceability

This test composes CG FR-014 and FR-018 with public QSL operation authority,
CG FR-021 with QSL FR-485 whole-call authority, and the QSpec TC-187 vector source. Test symbols trace to the applicable
`IT-019-SC-NN` criterion and owner family or vector ID. IT-017 and IT-013
remain separate evidence for their narrower paths.
