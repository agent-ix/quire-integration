---
id: SR-13335
title: "IR-914 quire-integration PR 16 code review (with Rust lane): export refusal withheld from CG, V1 tests retired"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration@58ba9c35e592c12e554a2be4f18b9e6d809bcec1 vs origin/main 74eb0a8; Cargo.toml, Cargo.lock, CLAUDE.md, README.md, reviews/26-10-19-ir914-round3-rust-review.md, tests/ir914_bound_export.rs, tests/qsl_model_owner_admission.rs, tests/support/mod.rs, deleted tests/qsl_codegen_composition.rs and tests/qsl_kani_exemplar.rs"
review_set: subset
---

## Summary

Ticket: IR-914. PR agent-ix/quire-integration#16, head 58ba9c3. The Rust lane is folded into this
file. The reviewer is not the author. The review used a fresh detached worktree at
/home/peter/dev/worktrees/integration-pr16-review with one target dir inside it, scoped to
`git diff origin/main...HEAD`.

What the reviewer measured:

- **The e2e test exercises quire-driver#86's partition.** `drive()` at the locked driver commit
  e2ba5df (same tree as #86 head 7c43963; the later commit only changes the PR record) partitions
  `export_bindings` results into `bound: Vec<bool>`, records `Err` items as `ItemRefusal`, emits
  `ContractRequestPosition::Excluded` for them, and calls the real CG `negotiate_contract_package`
  and `generate_routed`. The test's `Upstream` delegates compile, read, backend-kind, negotiate,
  route and generate to `QslCg` (the production impl). It injects only `has_abstraction_relation
  -> true` and the `export_bindings` vector. That matches the stated limit. At QSL 9567ed6,
  `CheckedPackage::bind_abstraction(&mut self, ..)` exists, but `CheckedPackage` is not `Clone`
  and `CheckedUnit` exposes only `&CheckedPackage`. A driver `Upstream` therefore cannot attach a
  relation to the compiled package. Nothing else is faked. The injected `BoundItem.bindings` and
  `Unbound.elements` are empty, and `drive` reads neither.
- **Mutation check on the partition assertion**, run by the reviewer:
  - M1, test vector: item 1 returns `Ok`, so the Err item reaches CG. Killed: the envelope has 3
    items, not 2.
  - M2, driver patched locally: the `Err(refusal)` arm returns `Ok(true)`, so the Err item is
    counted as bound and enters the envelope. Killed: the envelope has 3 items, not 2.
  - M3, driver patched locally: the `Excluded` position is dropped, which compacts the indexes.
    Killed: `SettlementCount` fault.
  - The unmutated patched-driver control passed.
- **The V1 subject really disappeared upstream.** QSL #952 (`7967e924b`, "Delete the native lower
  command and the lowering module") removed `lowering::{lower, lower_for}` and
  `command::{lower, lower_for}`. QSL 9567ed6 has neither. quire-contract-ir 000e461 has no
  producer of a V1 `BoundPackage` from V2. CG's V1 consumers (`generate_bound_oracles`,
  `generate_bound_strategy`, `generate_kani_bundle`) therefore have no upstream input.
- No compatibility layer, shim or V1 re-export. Tags are `#[trace(...)]` only, with no `/// Trace:`
  lines. Test-first order holds: the e2e test (882ad2d) came before the lock moves, and the first
  V2 assertion (ef34584) came before the retirement commit (58ba9c3).
- **Lock.** Each first-party crate has one copy (`cargo tree -d` lists no duplicate first-party
  crate). QSL resolves to the single revision 9567ed6, CG to 84673279, IR to 000e461 and Runtime
  to 3cc88c4. filament-core-data appears at two revisions, rev 3173a8f and tag
  semantic-schema-v0.3.0, but for different crates. That comes from quire-rs 0.50.1's own pin and
  is not a duplicate crate. quire-driver is locked to a feature branch (FND-001).
- **Gates at PR head**, run by the reviewer: `make fmt-check` pass; `make lint` pass; `make test`
  10 passed, 0 failed; `make deny` "licenses ok"; `make audit-unsafe` pass;
  `quire validate --okf spec --scope .` exit 0 with advisory dangling-reference and
  archetype warnings. Baseline origin/main 74eb0a8: `make lint` pass; `make test` result is in the
  Verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | quire-driver is a dev-dependency on the unmerged feature branch `code/ir-914-bound-only-cg-request` (lock e2ba5df). Merging this PR before driver#86 lands and is repointed to `main` leaves integration main locked to a branch that squash-merge deletes. The Cargo.toml comment "Every first-party crate is `branch = \"main\"`" is now false | Cargo.toml:20,32, Cargo.lock quire-driver |
| FND-002 | high | `v2_checked_integer_operation_reports_kani_rendering_refusal` pins `OperationNotRendered` for `quire.op.integer.add` as a "measured limit". In this exact cohort, the production driver path (`drive(&QslCg, ..)` with a Kani context) settles `x + 1` as `Supported { kob_scalar_integer_add_proof }` with a harness. CG documents a Kani renderer for `quire.op.integer.{add,sub,mul,negate}` (negotiate.rs:20-24). The refusal comes from how the test hand-assembles the request (`ContractDriverResultSelection::NotApplicable`, lowering only the operation node), not from a CG limit | tests/qsl_model_owner_admission.rs:298-362 |
| FND-003 | medium | The author wrote a review artifact and committed it under `reviews/26-10-19-ir914-round3-rust-review.md` (SR-010, PASS). The PR body cites it as "the Rust review". Authors must not write review artifacts. Remove the file from the PR | reviews/26-10-19-ir914-round3-rust-review.md |
| FND-004 | medium | The replacement for the retired numeric-oracle test checks only `!source_map.is_empty()` and `!checked_bounds.is_empty()`. The generated oracle is never compiled or executed, and it is never compared with QSL's native result, which the retired test did. Any non-empty bound passes | tests/qsl_model_owner_admission.rs:259-274 |
| FND-005 | medium | New tests are tagged with scenarios they do not satisfy. The test tagged `IT-010-SC-02` (Kani refusal) asserts that no Kani harness is produced, but SC-02 requires the Kani bundle to retain clause, requirement, bounds and spans. `IT-008-SC-01` ("every authored clause and native source locus survives binding") is bound to a refusal test that checks one source-map region | tests/qsl_model_owner_admission.rs:259,277,297 |
| FND-006 | low | The e2e test never asserts that the retained refusal value is the one QSL returned (`driven.item_refusals[0].refusal`). The driver documents it as "QSL's unchanged typed refusal". The injected `Unbound { elements: [] }` is a shape real `qsl_package::export_bindings` never emits (export.rs:84-93 returns `Unbound` only when an element is unbound) | tests/ir914_bound_export.rs:51-56,189-193 |
| FND-007 | low | The e2e trace tags `FR-010-AC-4`, `FR-001-AC-17` and `TC-017` are quire-driver criteria with no repository qualifier. This repo's spec has no such ids, so they cannot resolve here | tests/ir914_bound_export.rs:117 |

## Verdict

**NOT MERGEABLE as is.** The e2e partition test is sound. It drives real QSL compilation, the
driver#86 partition and real CG negotiation and generation. Its only injection is the stated
relation and export vector. Three mutants of the partition are killed. The V1 retirement is
genuine and adds no shim.

Blockers:

- FND-001: merge order. Driver#86 must merge first, then this lock is repointed to driver `main`.
- FND-002: a test asserts a refusal that the production path in this same cohort does not
  produce. The PR body and the IR-914 comment rely on it to justify "no harness to run".
- FND-003: the author-written review file must be removed.

FND-004 and FND-005 weaken what the replacement assertions claim. FND-006 and FND-007 are minor.

Reviewer probe, a temporary test file that was deleted afterwards: `drive(&QslCg, ..)` over
`function step using v(x: Int[0, 9]): Int[1, 10] pure { x + 1 }` returned settlements=1, routed=1,
and `Kani { disposition: Supported { harness_symbol: "kob_scalar_integer_add_proof" }, harness:
Some }`. The same holds for `-x` (`kob_scalar_integer_nega_proof`). An attempted real
`execute_kani_scalar_native` run could not start on this host (`bwrap: setting up uid map:
Permission denied`), so whether that harness verifies was not measured.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Driver #86 remains open, and Cargo.toml still selects its feature branch instead of main. |
| FND-002 | fixed | The hand-assembled integer Kani refusal test is deleted; production `drive(&QslCg)` is covered by a supported-harness test. |
| FND-003 | fixed | The author-written SR-010 artifact was removed from the PR. |
| FND-004 | fixed | The new scalar proof test executes the generated oracle and compares its native result with QSL step(4). |
| FND-005 | fixed | The two narrow V2 tests now trace to IT-013-SC-02 and IT-013-SC-03; IT-008/010 remain marked unmet. |
| FND-006 | fixed | The refusal fixture has a nonempty unbound element and the test asserts exact retained equality. |
| FND-007 | fixed | The partition test now uses the local IT-013 scenario tag. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | README still says the deleted direct integer Kani request test retains a typed refusal; the current test was removed and the production path returns a supported harness. | README.md:35 |
