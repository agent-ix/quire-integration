---
id: SR-13521
title: Integration gap analysis — IR-914 integration PR 16
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-integration; PR #16 at 8d82d557df27fd669b3d6ae06bd6ddc2808ee80d;
  Cargo.toml, Cargo.lock, Makefile, README.md, CLAUDE.md, tests/ir914_bound_export.rs,
  tests/qsl_model_owner_admission.rs, tests/support/mod.rs, spec/integration/IT-008,
  IT-010, IT-011, IT-013'
review_set: subset
---

## Summary

Ticket: IR-914. Non-author review of exact PR head 8d82d557df27fd669b3d6ae06bd6ddc2808ee80d. Plan completion: not assessed. The computed matrix has no IT rows, so scenario-to-trace bindings were checked manually. IT-013-SC-01 through SC-08 are tagged; SC-05 and SC-06 are in make kani and make ci. The V1 retirement remains explicit, and the V2 command-export composition is restored at the stated seam. Two previously identified restoration obligations lack dedicated owner tickets and dependency edges.

## Verdict

**CONDITIONAL**.

## Examined

- IT-008-SC-04 (examined): generated consequent activation agrees with native source events, including skipped and entered nested consequents.
- IT-010-SC-02 (examined): the numeric oracle, strategy bundle and Kani bundle retain the authored clause, requirement, model bounds, pre/post observation identities and original source spans; a later unsupported clause exports nothing.
- IT-011-SC-02 (examined): the unmutated run exits successfully and codegen's classify_kani_run classifies the mutated run Falsified; the mutation control fails if the function is not found.
- IT-013-SC-01 (examined): codegen receives two bound envelope items and full positions at indexes 0, 1 and 2; both settlement layers and generated items name 0 and 2, while the driver retains B's exact typed refusal and occurrence at 1.
- IT-013-SC-02 (examined): the claim is IR-confirmed and generated with a nonempty source map and checked bounds.
- IT-013-SC-03 (examined): codegen generates the IR-confirmed Boolean negation claim with a nonempty source map. This control makes no Boolean proof claim.
- IT-013-SC-04 (examined): codegen returns one supported Kani record with a retained scalar harness, and QSL's checked-package call of step(4) completes with the integer 5.
- IT-013-SC-05 (examined): the proof verdict is Verified, and the native integer result equals the checked QSL step(4) result of 5.
- IT-013-SC-06 (examined): real Kani returns Falsified with a counterexample on the generated soundness assertion, rather than a compilation refusal.
- IT-013-SC-07 (examined): a package without an abstraction relation skips export_bindings; every item reaches codegen and no item is refused.
- IT-013-SC-08 (examined): QSL itself returns the model item's typed Unbound refusal; the driver retains that exact refusal and original index, withholds the refused item from codegen and generates both Ok siblings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Boolean implication Kani mutation and composed LLVM consequent activation remain without dedicated restoration tickets or dependency edges. SC-07/08 restore QSL export composition only; broad IR-364/IR-598 and interim IR-132 references do not bind these two retired checks to owned delivery. | spec/integration/IT-008-native-boolean-backend-parity.md:13-19 |

## Coverage

Plan completion: not assessed. `quire matrix --format tsv --scope .` returned only a header, so IT scenario bindings were checked against the source trace tags. Optional semantic review was not selected; changed test and code alignment was checked as part of the mandatory PR code review.

## Evidence disposition — CG #468 and IR-132

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | CG #468 (merged ddd787a) adds a real Kani scalar implication subject proof and false-subject mutation, so the Boolean operation generation and scalar mutation subcase is delivered. It does not run a QSL-authored function-application postcondition or mutate the parsed runtime helper body required by retired IT-011-SC-02; FR-015-AC-40/FR-014-AC-38 remain planned. IR-132 is a Backlog CG vacuity report ticket with no structured IR-914 edge, and no open CG FR-004-AC-16..24 PR was visible at review; it does not own a composed native-vs-generated activation integration test. The two retired full checks still lack concrete restoration ownership/dependency edges. |

## Evidence disposition — IR-917 and IR-918

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | IR-917 now explicitly owns QSL-authored V2 clause implication proof and parsed runtime-helper-only mutation for retired IT-011-SC-02; IR-918 explicitly owns native-source versus generated LLVM consequent activation for retired IT-008-SC-04. Both are dedicated Backlog tickets with structured blocks IR-914 edges, verified by linear issue relation list. The new V2 export composition is covered by IT-013-SC-07/08 and the V1 command path is retired by design. This fixes the ownership/dependency finding; neither restoration ticket is implemented, so IR-914 closure remains blocked by both. |
