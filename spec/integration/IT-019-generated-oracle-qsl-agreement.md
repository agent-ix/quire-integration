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
checker-issued owner and occurrence, and evaluate it with a fresh QSL FR-485
session. For TC-187, obtain authored vector IDs and prose through
`quire_specification::vectors_of(TestCase::Tc187)` and
`QUALIFICATION_VECTORS`, by reference. Integration owns executable dispatch
for those IDs; it neither copies the corpus nor implements QSL, IR or CG
semantics. The public vector API's prose is not itself an executable request.

## Preconditions

The integration Cargo.lock resolves one coherent QSL, Contract IR, CG and
QSpec cohort. The Rust toolchain can compile the generated crate locally.
The native path invokes generated code without Kani or a combined proof/native
executor. For each run, both sides receive the same admitted claim occurrence,
operands, domain, type, limits and fresh meter; the QSL comparison session
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
   against independent QSL authority. A typed-refused or unsupported selection
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

The native generated artifact and independent QSL session agree directly on
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
| TC-187 public vector IDs and mixed-row operations | Planned | IR-842: per-ID admission/evaluation dispatch and generated execution remain unimplemented; IR-669 owns separate QSL/runtime scalar evidence |
| FR-014 scalar generated oracles | Planned | IR-842: complete per-operation native/QSL agreement remains unimplemented; existing samples are partial, and IR-669 QSL/runtime results supply no generated-oracle credit |
| FR-018 composite equality generated oracles | Planned | IR-842: per-operation executed native/QSL agreement remains unimplemented |
| FR-021 function application generated oracles | Planned | IR-842: remaining per-operation and boundary controls are unimplemented; the passed combine sample supplies no whole-family credit |

Unsupported generation is a typed nonexecuted observation in its own row;
it cannot be credited as two-sided value agreement. A partial passing slice
updates only the specific rows it executes, leaving the rest Planned with
their exact owner tickets in the implementation report.

## Traceability

This test composes CG FR-014, FR-018 and FR-021 with the QSL FR-485 authority
and QSpec TC-187 vector source. Test symbols trace to the applicable
`IT-019-SC-NN` criterion and owner family or vector ID. IT-017 and IT-013
remain separate evidence for their narrower paths.
