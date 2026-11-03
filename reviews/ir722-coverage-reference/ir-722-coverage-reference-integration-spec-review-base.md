---
id: SR-20261102072322
title: "IR-722 authoritative references and event-position quire-integration spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; original PR35; user ruling(b) source-reference coverage and FR276AC11 sequence"
review_set: subset
---

## Summary

The integration procedure adds a narrow falsifiable event-position sequence and explicitly excludes full producer/TC200 credit. No new requirement statement is authored, so EARS is inapplicable.

Same reviewer, scoped continuation of original RT193 / integration35 at RT90c7eb3133860fb094e2bd33286e0aec0f631031 and ITa2ba6f3c3bf40c60546d265826cc33481222e2c9. User ruling(b) requires authoritative sibling-main test sources by reference, with no mirrored/ported controls and no SHA/digest pin. No old source/SR ownership is replaced. Read actual Git diffs and existing repository/method guidance. No source/test edits, Cargo/heavy/proof/backend runs or merge/closure actions by reviewer. A small scratch Git fixture reproduced integrity failures only; it altered no reviewed tree.

The helper uses argument-vector subprocesses, TemporaryDirectory and file/directory symlinks; it neither executes external monitor tests nor copies their implementation or filters coverage JSON. Temporary cleanup unlinks references, leaving source files intact. Common-Git-directory discovery handles the ordinary RT worktree layout. Existing guards test reference identity/cleanup, feature/dirty-main rejection and an untracked substitute. However git branch/path existence plus ordinary diff do not prove source custody, and the alternate-checkout override lacks repository identity validation. See independently reproduced findings. Local-only reference prototype equivalence151/0 demonstrates preservation of local scan inputs, not accepted external coverage.

The new FR276AC11 control uses genuine emitted/read QSL declaration identities with matching authored source functions and explicitly controlled native ProgramParts bodies. It executes native State reads/construction and exact addition: init count0, baseline event delta0/count1, watermark/poll preserve1, gap reading65500 preserves1 and commits baseline, next65510 delta10/count2. Tick time independently goes0->65506->65516, and State has the four canonical required fields. There is no host event counter, copied temporal-status model or TC200 tag. The controlled executable body is not claimed to be actual temporal producer lowering; no full producer qualification is granted.

Preserved event-position logs show the new control and whole monitor_program86/0 plus touched Clippy PASS on the explicit interim coherent lock only; restored committed candidate cohort remains blocked by six driver adoption errors/IR1277. Real authoritative integration main currently lacks tests/monitor_program.rs, so the helper correctly fails closed with no branch fallback; rollout and actual strict reference computation remain owed after accepted main content exists. The whole-PR prior151unbacked/37introduced context is unchanged by claims here; no baseline114 waiver or current-cohort green is inferred. IEEE/CBMC is explicitly OUT OF SCOPE per ruling; no backend run/credit. Final native and own long-chain evidence from earlier carries is retained, not rerun.

Plan completion: not assessed

## Verdict

Scoped method verdict: **PASS**; rollout/cohort readiness remains conditional.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: IT-018-SC-15
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: '15. Execute the source-owned event-position sequence. IT-018-SC-15: build
    starts'
- id: FR-276-AC-11
  path: tests/monitor_program.rs
  role: examined
  excerpt: fn event_position_counts_ingested_events_and_not_watermarks_polls_or_gap_faults()
    {
bindings: []
```
