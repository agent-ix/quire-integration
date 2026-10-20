---
id: IT-013
title: "Compose bound QSL exports with V2 codegen"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-307
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-010
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-039
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-017
    type: verifies
---
# IT-013: Compose bound QSL exports with V2 codegen

## Objective

Check that the driver preserves the original request indexes when a QSL
binding export refuses one item, while the remaining items reach real codegen.
Record the separate V2 scalar outcomes from QSL-emitted checked-package bytes.

## Target Integration

QSL parse, check and emit → contract IR admission → driver export partition →
codegen package negotiation, routing and generation. The scalar controls use
QSL-emitted V2 bytes → contract IR admission → codegen derivation.

## Preconditions

The driver fixture injects the binding-export result because this QSL-compiled
unit has no attachable abstraction relation. All other QSL, IR, driver and
codegen operations use their public production APIs. The injected `Unbound`
refusal has an empty element list; QSL's own export only emits `Unbound` when
at least one element is unbound, so this fixture checks transport of the
typed refusal and makes no claim to exercise QSL's relation binding.

## Inputs

Three checked integer items A, B and C with export results `Ok`, `Err`, `Ok`
at driver indexes 0, 1 and 2, plus a QSL-emitted V2 package with integer
addition and Boolean negation operations.

## Test Procedure

1. Drive A, B and C with the injected export result through real codegen.
   IT-013-SC-01: codegen receives two bound envelope items and full positions
   at indexes 0, 1 and 2; both settlement layers and generated items name 0
   and 2, while the driver retains B's exact typed refusal and occurrence at 1.
2. Derive and generate a scalar claim for the admitted integer addition node.
   IT-013-SC-02: the claim is IR-confirmed and generated with a nonempty
   source map and checked bounds.
3. Derive a scalar claim for the admitted Boolean negation node.
   IT-013-SC-03: codegen returns its typed `OperationNotDerivable` refusal for
   that node. This control makes no Boolean generation or proof claim.
4. Drive an unbound QSL-compiled integer addition through the production
   `QslCg` path with a Kani generation context.
   IT-013-SC-04: codegen returns one supported Kani record with a retained
   scalar harness, and QSL's checked-package call of `step(4)` completes with
   the integer 5.
5. Run the retained harness and oracle package under installed Kani, then
   invoke its staged native scalar operation with operand 4.
   IT-013-SC-05: the proof verdict is `Verified`, and the native integer
   result equals the checked QSL `step(4)` result of 5.
6. Replace exactly the generated harness's integer `Add` operation with
   `Subtract`, keeping the checked QSL source and generated identity.
   IT-013-SC-06: real Kani returns `Falsified` with a counterexample on the
   generated soundness assertion, rather than a compilation refusal.

## Expected Results

An export-refused item never enters the codegen envelope or generated item
list, and the next bound item's driver index remains 2. The V2 controls keep
their generated and refused results distinct. The routed integer scalar
proves and produces the same concrete value as QSL native evaluation, while
an arithmetic mutation falsifies that proof.
