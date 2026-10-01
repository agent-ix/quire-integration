---
id: SR-005
title: "QSL-342 per-module SUCCESS attribution code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration@2d4bb1e7d2b3defc81346ee3a24e269e007f879c; Cargo.lock, spec/integration/IT-011-function-application-kani-exemplar.md, tests/qsl_kani_exemplar.rs"
review_set: subset
---
# QSL-342 per-module SUCCESS attribution code and Rust review

## Summary

Ticket: QSL-342 (per-module attribution follow-up). PR: quire-integration#13, base f063091, one commit 2d4bb1e. Scope is `git diff f063091...HEAD` only. This is a code review with the rust-review lane folded in. Codegen behaviour was checked against the b75e1ac checkout.

Lock. The only Cargo.lock change is quire-contract-codegen baab597 -> b75e1ac, which is CG main (#210). Every first-party crate has one revision: QSL d81193f9, IR/model 0a889f9, runtime ccc722bd. CG b75e1ac's own lock has the same IR and runtime. Its QSL pin, a28a5578, is an ancestor of d81193f9. The coder's claim holds.

Was dropping operators.rs right? Yes. ADR-011 §2.3 (origin/main) says a gate claims "the code it claims", and item 1 needs a discharged check location inside each claimed module. Before PR #9, the claimed module was the compiled postcondition oracle (`PopulationRule@7/population_rule`), which lives in codegen's generated bundle. `operators.rs` was never on a claimed list. `implies_short_circuit` at runtime ccc722bd is `if antecedent { consequent() } else { R::from(true) }`. It has no arithmetic, indexing or assertion, so Kani can never place a check there, correct or not. The consequent closure, which holds the `+`, is defined in the bundle, so its checks land in `src/lib.rs`. Runtime code is claimed by RT's own §2.3 gate (agent-ix/quire-contract-runtime#53), not by this exemplar. The subject is empty, so `implies_short_circuit` is not a shared helper under item 3. SC-02 stays as an extra control. This is not a vacuity finding. The stated reason in the code comment is the wrong one (FND-001).

Moving the subject to src/subject.rs. Nothing is weakened. `pub use subject::subject` keeps `crate::subject` resolving. CG's `execute_kani_obligation` check that `src/lib.rs` contains the harness still holds for IT-010's `prove`. `HEALTHY_SUBJECT` is `pub fn subject(_amount_current: i64) {}`, which has no checks. So `src/lib.rs` now holds only generated code, and the census can no longer count subject checks as generated-code checks. The coder's probe that claimed `src/subject.rs` and failed is the expected result for an empty subject.

Path canonicalization. `crate_root.join(&location.file)` keeps an absolute Kani path as it is, and resolves a relative one against the crate Kani ran in (`current_dir`). Both sides go through `fs::canonicalize`, so a symlinked tmp dir or a non-canonical absolute dependency path compares correctly. `unknown`, `/rustc/...` and other paths that do not exist resolve to `None` and are not attributed. A misspelt `CLAIMED_MODULES` entry panics in `claimed_module_files`. `BundleRun` keeps the `GeneratedCrate` alive until the census runs. Each run gets a fresh tempdir, so no stale report can be read. A missing report (`NotFound`) goes to CG as `None`, which CG refuses on a successful exit. Any other read error panics.

Console scraping. `discharged_success_checks` is deleted. The healthy count, the checks and both mutation verdicts come from `classify_kani_run` over the exported report. The transcript reaches CG only as `text`, which CG uses solely to extract the playback block. The one remaining `.contains(...)` in the file reads the generated bundle source, not Kani output. The hand-built `cargo kani ... --export-json` argv is needed: CG's `kani_launch_command`/`execute_kani_obligation` take a `KaniExecutableHarness`, and `KaniArtifactBundle` has no conversion to one.

Unit test oracle. `success_census_attributes_checks_by_exact_file` would fail a string-prefix match (`src/lib.rs.orig`), a crate-relative suffix match (the generated crate's own `src/operators.rs`, and `src/operators.rs` in another root), counting `unknown`, and counting UNREACHABLE/SATISFIED. The expected counts (1, 2) and (0, 0) come from the fixture, not from the code. Ran it focused: 1 passed (target/ deleted afterwards).

Gate evidence: the coder's log `qi-342-attrib-ci.log` ends `head=2d4bb1e7… exit=0`. All five `qsl_kani_exemplar` tests pass in it. It was not re-run (Kani 0.68 launcher, per the brief).

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | The `CLAIMED_MODULES` doc gives "Kani reports no check inside it" as why `operators.rs` is not claimed. That is the census justifying the list, which is the FB-10 pattern. The real reasons are ownership (RT's §2.3 gate claims runtime code, and this exemplar claims codegen's oracle) and that `implies_short_circuit` contains no checkable operation. | tests/qsl_kani_exemplar.rs:389-395 |
| FND-002 | low | IT-011-SC-04's sentence "A file whose path the module's path is a prefix of" is garbled. It should say a file whose path only starts with the module's path (for example `src/lib.rs.orig`). | spec/integration/IT-011-function-application-kani-exemplar.md:64 |

## Verdict

The Rust is clean. The new panics are test-only and carry messages. There are no integer casts. `success_census` borrows correctly and is O(checks x modules) over small inputs. The lock moved only CG. Path matching is exact and canonical, and it is robust to symlinks and to Kani printing paths relative or absolute. No Kani console text is parsed for a verdict or count. Dropping `operators.rs` is right on ownership grounds. Moving the subject out makes the attribution more precise.

Oracle strength: with one claimed module, which also holds the harness's own assertion, SC-04 is a weak check on its own. It fails only if every SUCCESS check is located outside the generated bundle. A stub oracle that returns `true` would still pass the census. SC-03's mutation is what shows the oracle is load-bearing. This is allowed by ADR-011 §2.3, which attributes by module, and the bundle is one module. It is recorded here, not as a defect. Both findings are low. The PR is mergeable once they and SR-006 FND-001 are fixed in-PR, which is cheap.
