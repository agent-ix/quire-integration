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

The first executable function is authentic QSL source
`combine(left:Int[0,9],right:Int[0,9]) {left+right}` called with admitted
operands whose sum is 7, under global work limits 0, 1 and 1024. Select the
real `combine` declaration, body and call occurrence from QSL's checked
package; do not fabricate node keys or substitute an interpreted body.
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
5. Exercise the applicable FR-014 scalar, FR-018 composite equality and
   FR-021 function application families, including completed, denied,
   refused and unsupported selections. Use the same authentic pipeline and
   independent QSL authority for runnable cases. For a CG unsupported or
   typed-refused selection, assert its exact nonexecuted disposition and
   sibling isolation; do not turn it into a Boolean verdict or QSL outcome.
   IT-019-SC-05: each named family and operation has an executable two-sided
   result or an explicit ticket-backed Planned entry. Runnable results agree
   in the full exposed envelope. A passing scalar sample cannot mark the
   composite or function family complete.
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

This artifact defines future evidence; it does not credit a test by merely
describing it. Update each row only from an executed test symbol and observed
result. The initial disposition is:

| Scope | Status | Required evidence before Passed |
| --- | --- | --- |
| `combine` work=1, 0 and 1024 | Planned | Native generated symbol and independent FR-485 whole-envelope equality at all three limits |
| Owner and occurrence mismatch; refused item and healthy sibling | Planned | Exact public refusals, no refused execution or charge, unchanged sibling |
| TC-187 public vector IDs and mixed-row operations | Planned | Per-ID admission/evaluation dispatch, or exact owner-ticket-backed missing path |
| FR-014 scalar generated oracles | Planned | Per-operation executed native/QSL agreement or owner-ticket-backed missing path |
| FR-018 composite equality generated oracles | Planned | Per-operation executed native/QSL agreement or owner-ticket-backed missing path |
| FR-021 function application generated oracles | Planned | Per-operation executed native/QSL agreement or owner-ticket-backed missing path |

Unsupported generation is a typed nonexecuted observation in its own row;
it cannot be credited as two-sided value agreement. A partial passing slice
updates only the specific rows it executes, leaving the rest Planned with
their exact owner tickets in the implementation report.

## Traceability

This test composes CG FR-014, FR-018 and FR-021 with the QSL FR-485 authority
and QSpec TC-187 vector source. Test symbols trace to the applicable
`IT-019-SC-NN` criterion and owner family or vector ID. IT-017 and IT-013
remain separate evidence for their narrower paths.
