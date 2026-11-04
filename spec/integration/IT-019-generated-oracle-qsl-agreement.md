---
id: IT-019
title: "Compare generated oracle execution with QSL value authority"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-018
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-021
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---
# IT-019: Compare generated oracle execution with QSL value authority

## Objective

Verify the agreement that CG's T12-A boundary deliberately does not claim:
actual generated scalar, composite and function oracle execution agrees with
QSL's value authority for the same admitted claim, operands and caller limits.
A direct RT operation, generated source inspection or a generated claim map
alone cannot establish this composition.

## Target Integration

Integration composes QSL's public source intake, checking and emission, IR's
strict V2 admission, CG's public selected-item derivation and oracle generation,
and native execution of the actual generated artifacts. The comparison side
uses QSL evaluation of the same checked source and selected claim. QSL, IR, CG
and the shared value kernels retain their domain authority; this test does not
introduce another operator encoder, evaluator, semantic result carrier or proof
checker. The native path does not invoke a Kani backend or a combined
proof/native executor.

## Preconditions

The repository's first-party Cargo dependencies select branch `main` and resolve
one coherent type graph. A native Rust compiler and Cargo are available. Source
fixtures and QSpec vectors remain in their owning repositories and are consumed
through public APIs or by reference. No copied corpus, source revision ledger,
new pin or hosted workflow is required. Generated child compilation uses the
same ticket-owned target directory, serially through the shared Cargo lane,
after the outer test target has been compiled and released that lane.

## Inputs

Use every QSpec TC-187 row exposed by the qualification-vector API, retaining
its case ID and authored expectation. Include the scalar operation families
owned by CG FR-014, composite equality owned by FR-018 and function oracles
owned by FR-021. Author source inputs for each executable operation in those
families; semantic meaning and expected outcomes remain with their owner.
Include the specified exact and one-under limits, undefined and refused
operands, composite owner/type differences, function argument/result boundaries,
selected occurrence differences and sibling claims. Unknown published vector
IDs fail rather than receiving a default result.

## Test Procedure

1. Admit original QSL source and required model documents through public
   producer APIs, emit V2 bytes and admit those exact bytes through IR with
   independently supplied evidence. Retain the same source-side checked
   package for QSL evaluation. Select each CG item from the resulting checked
   package and actual occurrence; do not construct a parallel checked node.
   IT-019-SC-01 (Planned until IR-842 lands): every compared claim has real QSL
   source admission, emission, strict IR admission and selected CG derivation;
   missing or refused input preserves its original typed disposition and locus.
2. Generate each requested scalar, composite and function oracle through its
   public CG API. Compile the returned crate artifacts and execute the selected
   generated native function with the actual operands and caller limits.
   IT-019-SC-02 (Planned until IR-842 lands): each family has actual generated
   artifact execution; replacing it with a direct runtime call, claim-map-only
   assertion, prebuilt expected label or proof-backed executor fails this check.
3. Evaluate the same selected QSL claim with equal semantic inputs and limits.
   Compare the complete outcome kind and completed value, or exact undefined,
   refusal, incomplete or fault cause. Preserve owner/type identity and every
   observable result component; textual success labels are not a value oracle.
   IT-019-SC-03 (Planned until IR-842 lands): scalar, composite and function
   outcomes agree on both sides for every runnable case and each required
   boundary case; a mismatch fails rather than becoming unsupported or pending.
4. Where an owner specifies accounting, compare ordered admitted charge points,
   amounts and consumed counters on fresh meters, then deny the specified charge
   at the exact and one-under limits and compare the first denied charge and
   absence of later charges or partial value. Report source admission and QSL's
   function-call envelope separately from the shared scalar-body accounting.
   IT-019-SC-04 (Planned until IR-842 lands): accounting agrees wherever the
   authority defines it; an unobserved meter or discarded denial field supplies
   no accounting credit.
5. Exercise a different owner or occurrence and an unrelated sibling claim.
   Assert the typed selection/refusal boundary before comparing results, and
   assert that one failed item cannot change a sibling's disposition or inputs.
   IT-019-SC-05 (Planned until IR-842 lands): owner, occurrence, argument/result
   and sibling controls bind generated behavior to the original admitted claim;
   structurally similar input from another selection supplies no substitute.
6. Dispatch every public TC-187 row by ID and split mixed admission/evaluation
   rows into their required observations. An admission-only row retains its
   own observed decision and never counts as generated scalar evaluation.
   Unknown IDs, unrun runnable cases, lost family coverage or silent skips fail
   closure. Record a genuinely unavailable owner seam with its exact existing
   ticket and keep that case Planned; it supplies no agreement or family credit.
   IT-019-SC-06 (Planned until IR-842 lands): every required family and published
   TC-187 observation executes and agrees, or remains explicitly owner-blocked;
   IR-842 stays open while any required observation lacks qualifying evidence.

## Expected Results

Actual generated oracle execution agrees with independently observed QSL
outcomes for every runnable assigned family and TC-187 observation. Admission,
selection, execution and accounting remain separate evidence. A partial family
may be reported truthfully, but no missing or owner-blocked observation is
counted as completed agreement. A corpus copy, unknown row, direct-runtime
substitution, skipped runnable case, typed-disposition collapse or mismatch
fails closure.

## Traceability

TC-099 invokes this integration procedure. Executable controls use `#[trace]`
attributes for the applicable IT-019 success criteria and retain the owning
QSpec vector ID. CG retains its T12-A boundary; this integration owns the
cross-repository agreement claim. IT-017's QSL/RT agreement and IT-013's single
proof/native demonstration do not by themselves back this procedure.
