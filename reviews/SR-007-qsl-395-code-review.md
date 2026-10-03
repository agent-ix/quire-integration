---
id: SR-007
title: "IT-012 model-owner admission code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration@1915d6c4e612040c15b78ff871b730e5ae77869a; Cargo.toml, Cargo.lock, tests/qsl_model_owner_admission.rs, tests/support/mod.rs, tests/qsl_kani_exemplar.rs, tests/qsl_codegen_composition.rs"
review_set: subset
---
# IT-012 model-owner admission code and Rust review

## Summary

Ticket: QSL-395 (A3k). PR quire-integration#14. Code review with the Rust lane folded in, scoped to `git diff origin/main...HEAD`.

- Measured at head: `cargo test --locked --test qsl_model_owner_admission` passes 4/4.
- Measured on current mains (scratch copy, `cargo update -p quire-spec-language -p quire-contract-model -p quire-contract-codegen`, giving QSL c83be711, IR 74087d29, CG 7f0d6d08): the new test fails to compile with two E0609 errors (no field `version` on `CheckedDomainPackageRef`, lines 157 and 202). Every other test binary compiles. With those two assertions removed, all 4 tests pass.
- SC-02's oracle is independent: the test hand-writes the canonical `quire.structural-node/v1` preimage with owner `{identity, kind, node}` and hashes it with `sha2`. It does not call QSL or IR code to build it. A `version` member in the owner would change each id.
- SC-03 changes the version for real: the unit selects `2.0.0`, the document is relabelled, and the lock's selection is asserted to differ. On current mains it differs through the digest, which still holds.
- Repairs to existing tests (`Category`, `category().exit_code()`, `as_str()` symbols, runtime by `branch = "main"`, profile line without version/digest) follow upstream API changes. Correct.
- Unrelated lock movement (cc 1.6.0, libc 0.2.190, uuid 1.27.0, lazy_static 1.5.1): semver-compatible transitive registry bumps, lock-only. Acceptable; no finding.
- quire-specification-qualification: absent from origin/main's Cargo.lock as well (grep at origin/main finds nothing), so this diff drops nothing. No finding.
- quire-canonical moves from tag v0.3.0 to branch main b4bb97a5. That is what QSL main and IR main lock. Correct.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | The lock pins QSL 8244dd2b and IR cbcd790a, both before QSL-395 itself: QSL #621 (c83be711, "lock selections bind by identity, no version") and IR #259 (c5fa773, "Selections bind by identity, not version"). The new tests assert `selection.version` in the lock, the exact field this ticket removed, so the PR verifies the superseded behaviour and does not compile on current mains. Fix in this PR: move QSL to c83be711, IR to 74087d29 and CG to 7f0d6d08 with a scoped `cargo update -p`, then delete the two version assertions. SC-03's `assert_ne!` on `model_selections` stays as the oracle, through the digest. Measured: 4/4 pass after that. | tests/qsl_model_owner_admission.rs:157, tests/qsl_model_owner_admission.rs:202, Cargo.lock |
| FND-002 | low | `domain_package` relabels with `text.replace("1.0.0", version)`. That rewrites seven fields: source.version, package.version, moduleVersion and four generatorVersion. SC-03 and its test name claim a version-only change. No false pass results, because package.version is among them, but the test does not do what it says. Set only `package.version` through serde_json, and assert the relabelled document reads it back. | tests/qsl_model_owner_admission.rs:43-47 |
| FND-003 | low | The refusal test matches `CheckedPackageDispatchResult::Refused(_)` with any cause. A refusal for an unrelated reason would also pass. Assert the refusal's code or path at `/lock/model_selections/0`. | tests/qsl_model_owner_admission.rs:185-188 |

## Verdict

Not mergeable yet. The tests have real oracles: SC-02 computes node ids independently, and SC-03 really changes the version. But the PR is locked to a QSL and IR older than QSL-395, and it asserts the lock's version field that QSL-395 deleted. FND-001 must be fixed in this PR. FND-002 and FND-003 are small in-PR test fixes.

## Dispositions

| FND | outcome | sha/reason |
|---|---|---|
| FND-001 | fixed | 7e77ab35 (round 1: lock moves to QSL c83be711, IR 75805e2e and CG 7f0d6d08; both `selection.version` asserts are deleted; SC-03's `assert_ne!` on `model_selections` stays as the oracle through the digest. Reviewer re-ran `cargo test --locked --test qsl_model_owner_admission` at 8c7f5bd8: 4/4 pass) |
| FND-002 | fixed | 7e77ab35 (round 1: `domain_package` sets only `package.version` through serde_json and asserts it reads back) |
| FND-003 | fixed | 8c7f5bd8 (round 1: asserts `MissingImport` at `/lock/model_selections/0/digest`. 7e77ab35 added the code assert and 8c7f5bd8 corrected the path. `MissingImport` with cause `MissingSelection` is IR's own branch in `admit_document` when no document is supplied under the selected digest, which is the scenario the test builds. IR's mismatch path, `StaleDependency`/`ByteDigestMismatch`, needs a document under the selected digest with other bytes. That is a different scenario and is not in IT-012-SC-01) |
