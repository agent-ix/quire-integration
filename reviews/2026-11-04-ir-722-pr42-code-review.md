---
id: SR-16369
title: "Code review — IR-722 PR #42 monitor source controls"
type: SpecReview
analysis: code-review
scope: "quire-integration; tests/monitor_program.rs; spec/integration/IT-018-monitor-program-construction.md"
review_set: subset
---

## Summary

Re-reviewed the full PR #42 test diff against `origin/main` at code head `f2fd168d7a0c701208572d30f30dd1b46c88b320`. The string-edge finding from the first pass is fixed; the final controls have discriminating assertions for retained source locations, native resource denials, and rejected progress rollback.

## Verdict

**PASS** — no remaining code-review findings on the final code head. The lane reports the focused monitor target passed 90/90, with scoped Clippy, fmt, and diff checks passing at this head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new `checked_equality_and_union_retention_keep_native_operating_shortages` test branches on `.identifier() == "Empty"` and `.identifier() == "Chosen"` without `#[string_edge]`. Mark this test so the required string-edge lint covers both comparisons. | tests/monitor_program.rs:5133, tests/monitor_program.rs:5247 |

## Review notes

- Compared the added controls with IT-018 SC-01/03/06/08/09/12/13/14. Clock/order/progress metadata, Let/Local, tuple/record projections, collection retention, equality/union scratch stops, and rejected-watermark rollback have discriminating assertions. The authentic RT native work-stop test retains the Work resource, native failure cause, and resolvable arena locus.
- The path-arena identifier exhaustion/storage mapping subcase is covered by the owning QSV integration. On QSV main `30c2493163cf35dd00d6dcaad63a7cbe9b2f5290` (merged PR #94), FR-109-AC-9 records internal constructor/mapping controls and explicitly claims no public injection seam or downstream QSL→IR→RT trigger (`spec/functional/FR-109-bound-supplied-value-traversal.md:178-184`). TC-908 specifies private-counter construction and supplied-run mapping without mutating the process counter (`spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:74-81`) and records the implemented controls (`:130-136`), including `arena_constructor_exhaustion_reports_one_native_unit_without_reusing_an_id` and `exhausted_path_arena_stops_before_a_supplied_run_or_locus_exists`. This is the correct owner boundary; #42 adds no public seam or fixture-forced capacity failure.
- The code lead reports the focused 90/90 test, scoped Clippy, fmt, and diff checks passing at `f2fd168`. They were not rerun in this review pass. No applicable AssuranceProfile is present under `spec/`.
- The subsequent `Cargo.lock` diff changed only source revisions for the eight `qsl-*` packages plus `quire-spec-language` to `213d8594c0c25339bbf15cfb31facc81a63d2eb7` and `quire-contract-model` to `401cb8ff47bd2c60d0bf542b4ecbce71423ea647`; QSV remains at `30c2493163cf35dd00d6dcaad63a7cbe9b2f5290`. `Cargo.toml` is unchanged and retains `branch = "main"` declarations without rev pins. No unrelated dependency entries moved. `producer_origin_equality` passed 8/8 with `--locked` on this exact cohort. The unchanged `quire-contract-ir` package entry remains at 4e4845ae; this review does not claim to move that separate package entry.

## Dispositions

| ID | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Added `#[string_edge]` to the equality/union native-shortage test; the focused monitor target compiled the marked test and passed 90/90. The repository has no separate string-edge script. |
