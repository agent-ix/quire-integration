---
id: IT-011
title: "Discharge and mutate the function-application Kani exemplar"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-033
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-015
    type: references
---
# IT-011: Discharge and mutate the function-application Kani exemplar

## Objective

Show that the Kani proof codegen generates for a QSL function-application
clause discharges real SUCCESS checks, and that the
generated module's own logic is load-bearing: a mutation inside it turns a
verified proof into a failed one.

## Target Integration

Native parse/link/check/package → `lowering::lower_for` with `integer-ir/v1` →
contract IR reader → quire-contract-codegen `generate_kani_bundle` → real
`cargo kani` over the bundle and a hand-written subject → codegen's
`classify_kani_run`.

## Preconditions

`cargo kani` is available locally. The generated crates fetch the
`quire-contract-runtime` that codegen's generated manifests name, and the
mutation control copies that same resolved checkout, located through
`cargo metadata`. Missing prerequisites fail the test. No hosted workflow is
dispatched.

## Inputs

The invariant `amount < 1000 implies amount + 1 <= 1000` over one scalar state
value `amount: 0..1000`, bound to `PopulationRule@7/population_rule`: a
function application applying `+` and a comparison directly to a value. Its
Kani adapter uses the canonical zero-dependency Boolean true precondition, and
its subject consumes the input and produces no result, because the clause's
ABI derives no subject result.

## Test Procedure

1. Run the generated Kani bundle with the healthy subject and read the
   SUCCESS-check discharge count from Kani's `** <failed> of <total> failed`
   summary, excluding UNREACHABLE and UNDETERMINED checks.
   IT-011-SC-01: the run exits successfully and discharges at least one
   SUCCESS check.
2. Confirm the compiled postcondition calls
   `quire_contract_runtime::operators::implies_short_circuit`, verify the
   unmutated run, then rerun the same bundle and subject against a copy of the
   runtime in which the parsed `fn implies_short_circuit` has its body replaced
   by `R::from(false)`, ignoring its parameters.
   IT-011-SC-02: the unmutated run exits successfully and codegen's
   `classify_kani_run` classifies the mutated run `Falsified`; the mutation
   control fails if the function is not found.
3. Rerun the same bundle and subject with the one `+` in the compiled
   postcondition oracle replaced by its left operand (an `add` returning its
   left operand).
   IT-011-SC-03: the unmutated run exits successfully and codegen's
   `classify_kani_run` classifies the mutated run `Falsified`; the mutation
   control fails if the oracle does not contain exactly one `+`.

## Expected Results

Both runs use real `cargo kani`. A zero discharge count, a missing summary line
or an unchanged verdict under mutation fails the test.
