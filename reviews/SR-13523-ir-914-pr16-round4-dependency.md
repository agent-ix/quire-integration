---
id: SR-13523
title: Spec dependency review — IR-914 integration PR 16
type: SpecReview
analysis: dependency
scope: 'agent-ix/quire-integration; PR #16 at 8d82d557df27fd669b3d6ae06bd6ddc2808ee80d;
  Cargo.toml, Cargo.lock, Makefile, README.md, CLAUDE.md, tests/ir914_bound_export.rs,
  tests/qsl_model_owner_admission.rs, tests/support/mod.rs, spec/integration/IT-008,
  IT-010, IT-011, IT-013'
review_set: subset
---

## Summary

Ticket: IR-914. Non-author review of exact PR head 8d82d557df27fd669b3d6ae06bd6ddc2808ee80d. IT-013 references QSL FR-307 without claiming full verification of QSL relation authoring. Driver FR-010 and CG FR-039/FR-017 remain verifying relationships for the exercised behavior. No new cyclic or unsupported prerequisite edge is introduced in the spec diff.

## Verdict

**PASS**.

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
| FND-001 | low | No findings (placeholder) | - |
