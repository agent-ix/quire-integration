---
id: SR-13444
title: IT-013 and V1 retirement base review
type: SpecReview
analysis: base
scope: agent-ix/quire-integration; spec/integration/IT-008-native-boolean-backend-parity.md,
  spec/integration/IT-010-config-version-numeric-backends.md, spec/integration/IT-011-function-application-kani-exemplar.md,
  spec/integration/IT-013-bound-export-and-v2-codegen.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-integration/IT-013
  type: reviews
---

## Summary

Ticket: IR-914. The three retired V1 specifications now mark their historical scenarios unmet. IT-013 states six narrower V2 scenarios with distinct trace tags. No new hash, digest, pin, copied schema, or test-file census is introduced by the spec diff.

## Verdict

PASS.

## Examined

IT-008 (examined): The V1 QSL projection inputs used by the former composition test were removed from the locked QSL API. Its test binary was retired; IT-008-SC-01 through IT-008-SC-04 have no current integration test and remain unmet. IT-013-SC-03 records a typed V2 Boolean-generation refusal, not Boolean parity or LLVM activation. The procedure below preserves the intended integration behavior for a future V2 path; it is not an available path in this cohort.
IT-010 (examined): The V1 `lowering::lower_for` and command export inputs used by the former composition and proof tests were removed from the locked QSL API. Both test binaries were retired. IT-010-SC-02 has only a partial V2 scalar-claim control in IT-013-SC-02. IT-013-SC-04 and SC-05 additionally retain and prove one V2 integer scalar harness and compare its native result with QSL; they do not exercise the ConfigVersion clauses. IT-010-SC-03 through IT-010-SC-05 have no current test. This document retains the desired ConfigVersion integration behavior, not a claim that the historical procedure remains executa
IT-011 (examined): The V1 projection input for the former Kani exemplar was removed from the locked QSL API, and its test binary was retired. IT-011-SC-01 through IT-011-SC-04 have no current integration test. V2 integer addition can produce a supported scalar harness (IT-013-SC-04), a real verified Kani run with successful checks and a matching native result (IT-013-SC-05), and a falsified arithmetic mutant (IT-013-SC-06). Those narrower scalar controls do not restore this scenario's function-application postcondition, counterexample replay, or exact per-module SUCCESS census. The procedure below retains that d
IT-013 (examined): Check that the driver preserves the original request indexes when a QSL binding export refuses one item, while the remaining items reach real codegen.
IT-013-SC-01 (examined): codegen receives two bound envelope items and full positions at indexes 0, 1 and 2; both settlement layers and generated items name 0 and 2, while the driver retains B's exact typed refusal and occurrence at 1.
IT-013-SC-02 (examined): the claim is IR-confirmed and generated with a nonempty source map and checked bounds.
IT-013-SC-03 (examined): codegen returns its typed `OperationNotDerivable` refusal for that node. This control makes no Boolean generation or proof claim.
IT-013-SC-04 (examined): codegen returns one supported Kani record with a retained scalar harness, and QSL's checked-package call of `step(4)` completes with the integer 5.
IT-013-SC-05 (examined): the proof verdict is `Verified`, and the native integer result equals the checked QSL `step(4)` result of 5.
IT-013-SC-06 (examined): real Kani returns `Falsified` with a counterexample on the generated soundness assertion, rather than a compilation refusal.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
