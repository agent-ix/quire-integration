---
id: SR-13530
title: "Code and Rust review — IR-699 IR-566 cohort PR 22"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #22 ac966501843bd16db0ec649d63a7c68ba655bfc2; Cargo.lock, tests/dependency_cohort.rs, Cargo.toml, current metadata graph"
review_set: subset
---

## Summary

Independent review of the lock refresh and new Rust dependency graph test. The current resolved graph has one source for each listed shared crate, all three FCD packages use ac62202a, both quire-rs packages use fd1cdd9, and `serde_json_canonicalizer` is absent.

## Verdict

**CONDITIONAL** — the test can pass after a real graph regression because it discards packages with no `source` and does not require all five FCD/quire-rs members.

## Examined

- IR-699-AC-1 (examined): authoritative semantic-value source, coherent QSL/IR/CG/protocol cohort, one instance of each shared type-bearing crate.
- IR-699-AC-2 (context_only): remeasure quire-protocol pin if composed graph requires it; integration graph has no quire-protocol.
- IR-699-AC-3 (context_only): remeasure StageLimits exemplar; that test is retired on current main.
- IR-566 (examined): remove `serde_json_canonicalizer` from the integration lock through current QVC.
- Files examined: `Cargo.lock`, `tests/dependency_cohort.rs`, unchanged `Cargo.toml`, current `cargo metadata --locked` graph, repository conventions. No CI workflow changed. No vendored artifact or copied schema added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The one-source assertion drops local/path packages before counting and the FCD/quire-rs revision assertion never checks the required member names. A second local `quire-contract-model` alongside the git package, or removal of two of three FCD crates, leaves these assertions green despite a broken cohort. Count exact package membership and source identity for each required crate before comparing revisions. | tests/dependency_cohort.rs:29; tests/dependency_cohort.rs:51 |

## Gate context

`cargo metadata --locked` passed on the reviewed head. The focused Rust test was not runnable because published CG in this lock fails compilation against current RT/QSL; this is an external consumer block, not a test verdict. Full gates were not run by this reviewer.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | IT-014-SC-01 requires a locked Git source for each named shared type package, but `members[0].is_some()` also accepts a registry source. If `quire-semantic-value` resolves from crates.io, the test remains green even though the authoritative Git owner is lost. Check the expected repository identity for every listed shared crate. | tests/dependency_cohort.rs:52 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The test now counts all matching package records before inspecting source, requires every named FCD/quire-rs member once, and checks each member's source repository. |
| FND-002 | still-open | Newly found in this disposition pass; the shared-type loop still checks only source presence. |
| FND-002 | fixed | The shared-type loop now checks the package's expected first-party Git repository as well as exact package count; a registry or local source fails. |
