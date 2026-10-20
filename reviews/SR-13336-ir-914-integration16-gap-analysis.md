---
id: SR-13336
title: "IR-914 quire-integration PR 16 gap analysis: retired V1 composition and proof coverage"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration@58ba9c35e592c12e554a2be4f18b9e6d809bcec1 vs origin/main 74eb0a8; spec/integration/IT-008, IT-010, IT-011, IT-012; tests/ir914_bound_export.rs, tests/qsl_model_owner_admission.rs, deleted tests/qsl_codegen_composition.rs and tests/qsl_kani_exemplar.rs; PR body and IR-914 lost-coverage inventory"
review_set: subset
---

## Summary

Ticket: IR-914. PR agent-ix/quire-integration#16. This review is planless, so plan completion was
not assessed. `quire matrix --format tsv --scope .` emits no rows for this repository: the
installed module mints no rows for the `IT` archetype. The scenario-to-test check was therefore
done by hand, reading each `IT-0xx-SC-NN` against the tagged tests at the PR head.

Scenario binding at the PR head:

- IT-008-SC-01: bound to `v2_checked_boolean_operation_gets_a_typed_cg_refusal`, a refusal plus one
  source-map region.
- IT-008-SC-02, SC-03 and SC-04: no test.
- IT-010-SC-02: bound to two tests, a claim with non-empty maps and bounds, and a Kani refusal.
- IT-010-SC-03, SC-04 and SC-05: no test.
- IT-011-SC-01 to SC-04: no test. All of IT-011's tests are deleted.
- IT-012-SC-01 to SC-03: unchanged, and still bound.

The reviewer checked each retired test against the QSL/CG/IR cohort locked by this PR (QSL
9567ed6, CG 84673279, IR 000e461, driver e2ba5df):

- **V1 subject gone: true for all ten tests.** QSL #952 deleted `lowering` and
  `command::{lower, lower_for}`. No IR or QSL producer of a V1 `BoundPackage` remains.
- **"No V2 equivalent": true** for the four proptest populations (`generate_bound_strategy` takes
  only a V1 `BoundPackage`, and `generate_i64_strategy` takes caller-declared constraints, not a
  package). Also true for the LLVM activation check and the ConfigVersion state-oracle and
  state-clause replay chain (IR-624 open), and for Boolean `not`/implication (CG's V2 function
  schedule supports only integer add/sub/mul/eq/ne and boolean eq/ne).
- **"No V2 equivalent": false for the Kani rows.** In this cohort the production driver path
  gives `x + 1` a supported Kani scalar harness (`kob_scalar_integer_add_proof`). CG exports
  `execute_kani_scalar_native`, which runs real Kani over such a harness together with a native
  invocation. CG runs it in `tc_027_attested_scalar_receipt_binds_real_proof_and_native_result`.
  See FND-002.
- **Owners: real Linear issues** for IR-364 (V2 strategy chain), IR-624 (state/frame from
  QSL-emitted packages), IR-598 (V2 proof controls), IR-604 (real-Kani lanes) and IR-610 (driver
  Kani run and replay). All are In Progress. Four rows name no ticket (FND-003).

Baseline comparison, run by the reviewer on fresh origin/main 74eb0a8 with its own lock:
`qsl_codegen_composition` 5/5 passed. `qsl_kani_exemplar` 1 passed and 4 failed. The 4 failures
are `healthy_run_*`, both `mutating_*` and `kani_proves_*`; the generated Kani crates fail to
compile with "quire.value.text.unicode-17.0.0/v1 requires Unicode 17.0.0 normalization tables".
The retired Kani tests were already red on main on this host. The composition tests were green on
main's lock.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repo spec was not updated for the retirement. IT-010's Traceability still says every `IT-010-SC-NN` "binds to a real Rust test assertion in `tests/qsl_codegen_composition.rs` or `tests/qsl_kani_exemplar.rs`", and both files are deleted. IT-011 (all four scenarios) and IT-008-SC-02 to SC-04 and IT-010-SC-03 to SC-05 now have no test. IT-008, IT-010 and IT-011 still describe `lowering::lower_for`, the QSL `lower` command, LLVM coverage and real `cargo kani` as current procedure and preconditions. The README and CLAUDE.md were rewritten, but the spec still claims coverage that no test provides | spec/integration/IT-010-config-version-numeric-backends.md (Traceability), spec/integration/IT-011-function-application-kani-exemplar.md, spec/integration/IT-008-native-boolean-backend-parity.md |
| FND-002 | high | The lost-coverage inventory says `healthy_run_discharges_success_checks_in_every_claimed_module`, `mutating_the_compiled_postconditions_addition_falsifies_the_proof` and `success_census_attributes_checks_by_exact_file` have no V2 equivalent because "this package's direct Kani settlement is `OperationNotRendered` ... no harness to run". That is false for this cohort. `drive(&QslCg, ..)` on `x + 1` yields `Supported { kob_scalar_integer_add_proof }` with a harness, and CG ships `execute_kani_scalar_native` for a real Kani plus native run. A V2 SUCCESS-discharge test and an integer-addition proof control are available through the driver this PR already depends on. `success_census_*` was a pure helper test over synthetic `KaniCheckResult`s, and the helper applies to V2 Kani reports unchanged | PR body "Retired proof and replay assertions"; IR-914 comment 131316a7; tests/qsl_model_owner_admission.rs:297-362 |
| FND-003 | medium | Four lost-coverage items have no real owner ticket: Boolean operation generation ("no dedicated ticket"); the Boolean Kani renderer for the `implies_short_circuit` mutation ("no dedicated Linear issue exists"); the QSL command-export replacement ("QSL owns"); and the composed activation check ("QSL/CG owners must restore"). The IR-914 comment says issue creation was unavailable. Each needs a filed Linear issue before the coverage is retired | PR body tables; IR-914 comment 131316a7 |
| FND-004 | medium | The retired `actual_command_exports_*` and `compiled_state_oracle_*` tests executed generated oracles and compared them with native verdicts. No row lists "generated oracle compiled and executed against native" as lost. The replacement only checks that the claim map is non-empty. CG's V2 `generate_exact_function_oracles` covers integer add/eq, and the routed `oracle_artifacts` are compilable. An executed V2 integer oracle vs native check is available and is not listed | PR body row 1; tests/qsl_model_owner_admission.rs:259-274 |

## Verdict

**NOT MERGEABLE.**

- The V1 retirement is genuine, and most "no V2 equivalent" claims hold.
- The repo spec still asserts the retired coverage (FND-001).
- The Kani rows of the inventory rest on a refusal that the production path in this cohort does
  not produce (FND-002). A V2 Kani discharge test is reachable now.
- Four lost items have no owner ticket (FND-003).

The IR-914 e2e partition criterion itself (bound items at `[0, 2]`, B withheld) is covered, and
mutation checks kill three partition mutants (SR-13335).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | IT-008, IT-010 and IT-011 now mark the retired V1 procedures and all historical scenarios unmet. |
| FND-002 | fixed | IT-013 adds a production supported harness, real Kani proof and arithmetic mutation; old function-application and exact module census remain explicitly unmet. |
| FND-003 | still-open | The four restoration items remain only in IR-914 comment e48b14b2; no dedicated Linear issues or dependency edges were filed. |
| FND-004 | fixed | The new proof test compiles and executes the routed scalar oracle and compares the result with QSL; ConfigVersion remains expressly unmet. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | IT-013-SC-05 and SC-06 are ignored by `make ci`; there is no committed real-Kani target or command to make their verification repeatable. | tests/ir914_bound_export.rs:340,425; Makefile:64 |
