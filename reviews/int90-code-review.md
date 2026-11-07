---
id: SR-2026110720500001
title: quire-integration PR 90 integration batch code-review (Rust lane)
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-integration; PR #90 head 823e18179ddfc13fb360a882ea37738c747b8212 (merge of #86 at 92f80d8a and #87 at 4c7352c6 onto main 61e2cd8); merges 6ba0be55 and f776481a; commit 823e1817 (tests/temporal_formula.rs, tests/temporal_join.rs, tests/support/monitor_program_adverse.rs, tests/support/temporal_projection.rs, tests/temporal_binding_axes/fixture.rs, tests/temporal_mapping_binding.rs); Cargo.toml; Cargo.lock; reviews/ir-1005-*.md; reviews/ir-842-*.md; tests/tc187_vector_dispatch.rs'
review_set: subset
---

## Summary

Tickets: IR-1005, IR-1378 and IR-842. PR: quire-integration#90. This is one
batch pass with the rust-review lane folded in. It covers the merge, the
post-review commit 823e1817, and a spot-check that the folded PRs' last
findings are fixed. Findings that are already settled were not re-reviewed.

Merge. Both merge commits are clean: `git show --cc` shows no hand-resolved
hunks. The emit-limit change in 823e1817 replaces the remaining
`16 * 1024 * 1024` literals at tests/support/temporal_projection.rs:237,
tests/temporal_binding_axes/fixture.rs:83 and tests/temporal_mapping_binding.rs:78
with `qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT`. No other `EmitLimits::new`
literal remains in tests/ or src/. The Cargo.lock cohort differs from #86's
final lock only by QSpec bdf0e9ae to e53e9487, which came from #87. It differs
from #87's final lock only by the #86 additions (quire-canonical, quire-mltl
70175d91, tl-mltl 8e8149d1). The QSL revision 2ed49b65 is the same in both
PRs, so neither PR's lock breaks the other. The Cargo.toml dependency moves
from #86 (serde, qsl-eval and quire-spec-language promoted to
[dependencies]) are kept. The fmt-only hunk in
tests/support/monitor_program_adverse.rs is correct.

Post-review tests. The R1+R6 test (tests/temporal_join.rs:1647-1691) reaches
L (a native correction against an original TL result) and TL refusal (foreign
request) with genuine owner results. The aggregate-atom test
(tests/temporal_formula.rs:672-695) reaches `TlFormulaCode::Representation`
on a Supported classification: an Unsupported classification returns
`TlFormulaError::Unsupported` first (src/temporal/formula.rs:232-237), so the
Representation result does come from the aggregate node.

Spot-check of #86 (SR-2026110715130801) at head:
- FND-001: fixed by the extracted unit-tested positions.rs.
- FND-002: rational clock, Contradicted, interval and aggregate formula are
  covered; subject-name and encoding are verified at the producing function
  per IT-022.
- FND-003, FND-004 and FND-005: fixed.
- FND-006: row pairs were added.
- FND-007: fixed at native.rs:344-347.
- FND-008: NativeActivation was removed.
- FND-009: native_state is now exhaustive.
- FND-010: the duplicate observation.rs was deleted.

Spot-check of #87 (SR-20261107100901): FND-001 to FND-005 are fixed as
dispositioned. FND-006 (disposition pass 1) is not; see FND-005 below.

## Verdict

CONDITIONAL. The production code and the merge are sound, and there is no
high finding. Before merge, the two medium review-record defects need
fixing: IR-1005's earlier SR files were overwritten, and the #86 findings
have no recorded outcome. The low findings are test-intent weaknesses.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The batch overwrites the committed IR-1005 PR #44 review artifacts: reviews/ir-1005-code-review.md (SR-2026110414560009) and reviews/ir-1005-gap-analysis.md (SR-2026110414560010) on main are replaced by #86's SR-2026110715130801/802 under the same slugs. After merge no file in reviews/ holds SR-2026110414560009 or 0010, although Linear comments cite them. Scenario: a reviewer or miner looking up the #44 input-binding review finds only the #86 review | reviews/ir-1005-code-review.md:1-4; reviews/ir-1005-gap-analysis.md:1-4 |
| FND-002 | medium | #86's findings have no recorded outcome. Neither committed SR file has a Dispositions section, and IR-1005 has no reviewer-dispositions comment for SR-2026110715130801 or 802. Code-review FND-001 (high) to FND-010 and gap FND-001 to FND-004 are therefore all undecided at head, although the code fixes for the code-review findings are present (see Summary). Gap FND-001 to FND-003 are FR-350 contract defects and need a deferred row with an owner ticket. Scenario: the merge check reads the latest outcome per finding and finds none | reviews/ir-1005-code-review.md:24-37; reviews/ir-1005-gap-analysis.md:26-33 |
| FND-003 | low | The aggregate-atom test asserts `TlProjectionError::Formula(error).code() == TlProjectionCode::Formula` on a variant it builds itself. That holds for any TlFormulaError by construction, and the test never calls project_tl. If owners::formula_error mapped Representation elsewhere, or project_tl refused the aggregate subject before construction, the test would still pass. #86 FND-002 asked for tl-projection-formula through project_tl | tests/temporal_formula.rs:672-695; src/temporal/projection/owners.rs:60-77 |
| FND-004 | low | The R1+R6 test states that the TL side is refused, but it asserts only the decision, the causes and the value. It never checks joined.native_admitted() or owner_refusals(). L uses the raw report's predecessor, so R6 holds even when the native read refuses. If the native correction failed admission (an R1 from the native side), the test would still pass | tests/temporal_join.rs:1647-1691; src/temporal/join.rs:689-708 |
| FND-005 | low | IR-842 SR-20261107100901 FND-006 is not fixed and has no disposition row. 4c7352c compares the PLANNED_REMAINDERS owner with planned_remainder_owner, a second literal table in the same test. Neither table is tied to dispatch() or to IT-019, so any owner passes as long as both literals change together, and U25 still has no Planned dispatch entry with its ticket | tests/tc187_vector_dispatch.rs:167-186; tests/tc187_vector_dispatch.rs:471-480 |

## Coverage

Focused run (own CARGO_TARGET_DIR in a throwaway worktree at 823e1817) over the
TC-115 targets (temporal_mapping_binding, temporal_projection,
temporal_bridge_definition, temporal_binding_axes, temporal_join,
temporal_limits, plus the temporal_formula and temporal_observation_revision
regressions) and the IR-842 targets (generated_oracle_* and tc187_* that carry
IR-842/IT-019 tags): see Test results below.

## Test results

cargo test --locked, own CARGO_TARGET_DIR, at 823e1817: exit 0 across 27 targets, 107 passed and 0 failed. The TC-115 targets plus the temporal_formula and temporal_observation_revision regressions: 75 passed (projection 22, binding_axes 12, join 18, limits 5, bridge_definition 1, mapping_binding 1, formula 10, observation_revision 6). The 19 IR-842 targets: 32 passed (tc187_ir842_conversions 9/9, tc187_vector_dispatch 1/1, and all generated_oracle_* targets). The build dir and the throwaway worktree were deleted afterwards.

## Dispositions

Round 1, verified at #90 fix-round head ea142805937d0ebf3951fb3065ef3e0a40de5a00. Touched targets were run once: temporal_projection 23/23, temporal_formula 10/10, temporal_join 18/18 and tc187_vector_dispatch 1/1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | reviews/ir-1005-code-review.md and reviews/ir-1005-gap-analysis.md are byte-identical to main 61e2cd8 (SR-2026110414560009 and 0010 restored). #86's SRs are now reviews/ir-1005-pr86-code-review.md and ir-1005-pr86-gap-analysis.md |
| FND-002 | fixed | Both ir-1005-pr86-* files carry a Dispositions section with an outcome for every finding, matching IR-1005 comments c7ed96a8 and 08ea6832 |
| FND-003 | fixed | tests/temporal_projection.rs an_aggregate_atom_is_reported_as_tl_projection_formula drives the aggregate subject through project_tl (via run) and asserts TlProjectionError::Formula and TlProjectionCode::Formula. The hand-built wrap is removed from temporal_formula |
| FND-004 | fixed | The R1+R6 test now asserts joined.native_admitted() and that owner_refusals().native() and mapped() are none, so the refusal comes from the TL request mismatch |
| FND-005 | fixed | planned_remainder(id) returns a RowDispatch::Planned entry. Every executed row is queried, the executed rows with a remainder must be exactly [U21, U25], and U25's remainder must be Planned with tickets ["IR-1403"] |
