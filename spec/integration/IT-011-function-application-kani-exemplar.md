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

## Current Status

The V1 projection input for the former Kani exemplar was removed from the
locked QSL API, and its test binary was retired. IT-011-SC-01 through
IT-011-SC-04 have no current integration test. V2 integer addition can
produce a supported scalar harness (IT-013-SC-04); that generation result is
not a real Kani proof, mutation result or check census. The procedure below
retains the desired proof behavior while V2 tests are restored.

## Objective

Show that the Kani proof codegen generates for a QSL function-application
clause discharges real SUCCESS checks, and that the
generated module's own logic is load-bearing: a mutation inside it turns a
verified proof into a failed one.

## Target Integration

Native parse/link/check/package → `lowering::lower_for` with `integer-ir/v1` →
contract IR reader → quire-contract-codegen `generate_kani_bundle` → real
`cargo kani` over the bundle and a hand-written subject → codegen's
`classify_kani_run` → codegen's `decode_falsification` for a falsified run's
counterexample.

## Preconditions

`cargo kani` is available locally and accepts
`-Z unstable-options --export-json <file>`. The generated crates fetch the
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

The following was the V1 procedure. It is not current verification evidence
for the four scenarios until equivalent V2 tests run real Kani.

1. Run the generated Kani bundle with the healthy subject, exporting Kani's
   report, and read its checks through codegen's `classify_kani_run`: the
   SUCCESS-check count and each check's status and source location. Only
   SUCCESS checks count; UNREACHABLE, UNDETERMINED and every other status do
   not. The generated crate holds the bundle in `src/lib.rs` and the subject in
   `src/subject.rs`. The claimed modules are `CLAIMED_MODULES`: `src/lib.rs`,
   the generated bundle. A SUCCESS check is attributed to a claimed module only
   when its location names exactly that module's file, after resolving a
   relative location against the generated crate and canonicalizing both
   paths.
   IT-011-SC-01: the run exits successfully and discharges at least one
   SUCCESS check.
   IT-011-SC-04: every claimed module is attributed at least one SUCCESS
   check, and a claimed module attributed none fails the test, which names
   it. A file whose path only starts with the module's path (for example
   `src/lib.rs.orig` for `src/lib.rs`), or a file with the same crate-relative
   path in another crate, is not attributed to it.
2. Confirm the compiled postcondition calls
   `quire_contract_runtime::operators::implies_short_circuit`, verify the
   unmutated run, then rerun the same bundle and subject against a copy of the
   runtime in which the parsed `fn implies_short_circuit` has its body replaced
   by `R::from(false)`, ignoring its parameters.
   IT-011-SC-02: the unmutated run exits successfully and codegen's
   `classify_kani_run` classifies the mutated run `Falsified`; the mutation
   control fails if the function is not found.
3. Rerun the same bundle and subject with the one `+` in the compiled
   postcondition oracle returning `left + right + 1`, so the rule reads
   `amount < 1000 implies amount + 2 <= 1000`. The mutant is non-equivalent:
   it is false at `amount = 999`. An equivalent mutant (dropping the `+ 1`)
   leaves a rule every in-domain value satisfies, so a proof of it is correct.
   IT-011-SC-03: the unmutated run exits successfully, codegen's
   `classify_kani_run` classifies the mutated run `Falsified`, and codegen's
   `decode_falsification` reads the counterexample as `amount_current = 999`;
   the mutation control fails if the oracle does not contain exactly one `+`.

## Expected Results

Every run in all three procedures uses real `cargo kani`. A zero discharge
count, a claimed module with no SUCCESS check, a report codegen refuses, an
unchanged verdict under either mutation, or a mutated counterexample other than
`amount_current = 999` fails the test.
