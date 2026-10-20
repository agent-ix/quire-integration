---
id: IT-011
title: "Discharge and mutate the function-application Kani exemplar"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-codegen/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-015
    type: references
---
# IT-011: Discharge and mutate the function-application Kani exemplar

## Current Status

The former V1 projection input and its test binary were retired when the locked
QSL API removed that input. IT-011-SC-01 through IT-011-SC-04 have no current
integration test. IT-013's V2 scalar addition proof and mutant, and CG's V2
scalar Boolean implication control (FR-015-AC-118), do not prove a QSL-authored
function-application postcondition through a clause-identified oracle and
harness. IT-011-SC-02 is the first V2 restoration slice. SC-01, SC-03 and
SC-04 remain planned and require their own live checks before they count as
verification evidence.

## Objective

Show that a QSL-authored function-application postcondition admits a V2 clause
claim and that the generated Kani proof depends on the authoritative runtime's
Boolean implication helper. A healthy run must verify; changing only that
helper's parsed body to return false must falsify the same claim.

## Target Integration

QSL source → native parse/link/check/package → admitted `CheckedPackageV2` →
clause-identified FR-014 Boolean oracle (FR-014-AC-38) → FR-015 V2 clause
harness (FR-015-AC-40) → installed Kani → codegen's run classification and
counterexample playback. The clause node, not a referenced condition node,
identifies the claim and the generated oracle.

## Preconditions

Installed Kani runs both proof executions and exports reports that codegen can
classify. The test obtains the authoritative `quire-contract-runtime` source
through the resolved dependency, and performs the mutation in an isolated
checkout of that source. The generated proof resolves the same runtime source
with only the parsed helper body changed in the mutant run. No runtime source,
binary, schema or fixture is copied into this repository. Missing Kani,
admission, helper-call, helper-body, report or playback prerequisites fail the
test. No hosted workflow is dispatched.

## Inputs

Author the function-application postcondition
`amount < 1000 implies amount + 1 <= 1000` in QSL for an operation with one
parameter `amount: Int[0, 1000]`. Both occurrences of `amount` read that
operation parameter, not a state field or a subject-produced post-state value.
Obtain the exact admitted `CheckedPackageV2` from QSL's source-facing
facade during the test. The test identifies its `postcondition`
`state`/`state_clause` node and uses that node's identity for both the FR-014
oracle and FR-015 harness; if QSL does not emit and admit this shape, SC-02 is
not satisfied. Under FR-015-AC-39, the harness draws `amount` once within its
declared inclusive domain and passes that exact value to the compiled subject;
it draws no post-state value. Both condition reads, the oracle and the
independent native expectation use that same value. The bound comes from the
admitted package, not from a caller-supplied range. Selection and subject
binding follow FR-015's V2 clause contract; this scenario does not define a
state-field accessor, subject ABI or QSL package format.

## Test Procedure

1. Generate the clause-identified oracle and V2 clause harness from the
   QSL-authored admitted package. Record the QSL source, admitted clause node
   identity, oracle claim identity and harness claim identity in the test
   evidence. Confirm that the harness embeds the FR-014 oracle function byte
   for byte and that the compiled postcondition calls
   `quire_contract_runtime::operators::implies_short_circuit`. Independently
   of the runtime helper and oracle, the harness checks the implication's
   native Boolean expectation `!left || right`, where
   `left = amount < 1000` and `right = amount + 1 <= 1000`, using the same
   drawn `amount` passed to the compiled subject and oracle. Neither this
   expectation nor the oracle calls the runtime implication helper. A helper
   result and oracle result agreeing with each other cannot alone satisfy
   this check.
2. Run installed Kani on the healthy generated proof. Require a successful
   execution, a `Verified` classification and a satisfied non-vacuity cover.
   Keep the QSL source, admitted clause identity, oracle, harness and proof
   subject fixed for the mutation run.
3. In the isolated checkout of the authoritative runtime source, parse the
   `implies_short_circuit` function and replace only its body with a return of
   false, ignoring its parameters. Require exactly the intended helper body
   and compiled postcondition call to be found; an absent or ambiguous match
   fails the control. Run the same generated claim with installed Kani against
   that changed runtime. Require codegen to classify it `Falsified` and decode
   a concrete playback within the admitted input domain. Replay the concrete
   operands against the independent native implication: it is true while the
   mutated helper returns false. The test fails if source, clause identity,
   oracle, harness, subject, native expectation or any runtime code outside
   that one parsed body changes between the runs.

IT-011-SC-02: all three steps pass. A generation-time Boolean, a missing real
Kani run, a verdict without a satisfied healthy cover, a refused report, a
counterexample outside the admitted domain or a mutation that leaves the
claim verified does not satisfy the scenario. The test carries
`#[trace("IT-011-SC-02")]`.

## Remaining Planned Scenarios

- IT-011-SC-01: require a healthy real-Kani run with at least one discharged
  SUCCESS check. This is not inferred from the SC-02 `Verified` verdict alone.
- IT-011-SC-03: mutate the clause's arithmetic `+` to a non-equivalent
  `left + right + 1`, require `Falsified` and decode the in-domain
  `amount_current = 999` playback. This is separate from the runtime-helper
  mutation of SC-02 and remains unverified.
- IT-011-SC-04: attribute at least one SUCCESS check to every claimed generated
  module by exact canonical file identity. A path prefix or the same relative
  path in another crate does not count. This remains unverified.

## Expected Results

For SC-02, the healthy clause proof is `Verified` with its cover satisfied;
the runtime-helper-body mutant is `Falsified` with concrete in-domain playback
whose native implication is true. The same QSL source, admitted clause claim,
oracle, harness and subject serve both runs. None of SC-01, SC-03 or SC-04 is
claimed complete by this result.
