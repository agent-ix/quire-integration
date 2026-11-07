---
id: SR-2026110414560135
title: "IR-1005 PR85 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; IR-1005; PR85; changed temporal production, definition, specifications and tests"
review_set: subset
---

## Summary

Ticket: IR-1005. Reviewed frozen head 3d186f5a0d4bfb5d06d02367cb3a29a5e91e5f6b against origin/main. One existing worktree; no source edits, builds, commits, pushes or owner-side changes. Plan completion: not assessed. Scoped trace, reverse-gap and assertion inspection only. The computed repository matrix returned no obligations; external FR-350 criteria were mapped manually to changed tests. No whole-repository coverage claim or optional semantic-analysis fanout.

## Verdict

**CONDITIONAL** for this method. No final merge clearance: the source does not compile in the current duplicate-owner cohort and the runtime tests are unverified. Required owner and full-scope gaps remain explicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Complete join acceptance remains unqualified: successful exact corrected-pair admission and the Failed/Unsupported first decision rows are not covered by genuine reachable producer results. The correction test currently requires PredecessorAuthority. Preserve the explicit owner blockers, obtain the necessary actual owner capabilities or normative reachability ruling, then execute the complete acceptance cases before claiming FR-350 completion. | tests/temporal_join.rs:831-941 |
| FND-002 | medium | The new projection/join acceptance tests have not compiled or executed. Focused library check3 has 24 duplicate-owner E0308 errors under the current cohort. Formatting and trace tags do not establish owner fixture admission, value assertions, strict-read behavior or operating boundaries. Adopt the corrected IR-1377 cohort and run the actual feature targets and required final checks; this draft has no runtime clearance. | tests/temporal_projection.rs:1-701 |
| FND-003 | medium | The new genuine source fixtures always use one M model alias, one origin clause and no declared captures. The current tests mutate capture-instance labels but do not exercise an actual declared capture or retained capture environment, alias/type/predicate binding mutations, or a later shared-node occurrence source span. Add value-asserting producer controls for these preserved axes, including the concrete retained-ledger and source-span defects. | tests/support/temporal_projection.rs:46-65 |

## Coverage

- FR-350-AC-1 (examined): Given an admitted checked-package/v2 and QSL `Supported` classification, the projection takes its target from the classification, emits the required formula and valuation request for each supported row, and each generated artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`.
- FR-350-AC-2 (examined): Timestamped-event, mixed past-and-future, infinite-trace unbounded and other unsupported classifications return `unsupported` with every named dimension and no formula, request or correspondence record; a finite-window-to-closed-trace equivalence claim is refused.
- FR-350-AC-3 (examined): Mutating any preserved source/span, package/declaration/profile, model/type/predicate binding, anchor, capture, clock, observation, interval, closure/history, TL selection or bridge-definition axis changes the corresponding exposed typed member and identifies that axis; a non-Boolean leaf never becomes a proposition.
- FR-350-AC-4 (examined): Fixed-sample projection retains epoch, period and unit, refuses a changed period or event-count reinterpretation with equal formula bytes, and reports a missing required sample as `incomplete`.
- FR-350-AC-5 (examined): Every join decision row is reached; only an equal-final pair carries a Boolean; every non-final pair preserves its causes and all result axes, and a correction is accepted only with its exact direct predecessor value.
- FR-350-AC-6 (examined): Missing, duplicate, swapped, replayed or foreign valuation cells; owner-reader rejection; malformed or noncanonical projection/join data; and each one-over resource limit return one deterministic non-admitted/incomplete result, with no partial artifact and no evaluator invocation. Exact limits admit.
- FR-350-AC-8 (examined): Each `TlLimits` setting uses its stated default, charges before work, admits exactly at the setting and returns `incomplete` one past it with `limit-reached{limit, value, setting}` and no partial artifact.
- IT-022 (examined): The same IR-1005 integration leg also owns strict correspondence reading and result joining. Their implementation and tests remain required before this leg can claim FR-350 completion; a projection-only result is not completion.
- TC-115 (examined): The strict projection and join readers require complete canonical expected-value controls; join tests use genuine explicitly assessed owner reports, independently of the non-evaluating bridge. Exact corrected-pair success remains pending the named owner admission capability and is not claimed by an authority-refusal test.

Plan completion: not assessed. Initial scope was the 21 changed files and owning external FR-350 criteria; subsequent sections record the scoped follow-up source/test deltas. This is not a whole-repository audit. No applicable AssuranceProfile is present under spec/. Optional unrelated analyses and optional semantic gap fanout were not run. No Tokio/async/unsafe code or hosted workflow change is in this diff. Compile evidence is the coder's focused check3, not a reviewer rerun. The unchanged dependency cohort rule was not waived.

## Evidence

- Actual producer/strict-reader fixture and assertion inspection for projection, definition and join tests; no sealed result fabrication is accepted as coverage.
- FR-350 and TC-461 from the actually locked Contract IR owner; current IR-1005 thread and normative corrections reviewed as context.
- `quire matrix --scope . --strict --format json` returned exit 1: no obligations matched the integration spec scope. This is why the external criterion mapping above is manual and does not claim a clean computed matrix.
- Spec/artifact validation is structural only; it does not replace runtime evidence or correct the code findings.

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | A genuine native Failed execution regression is now present, but its runtime execution is blocked by IR-1377. Exact corrected-pair success and Unsupported first-row reachability remain explicitly unqualified; predecessor owner APIs/ruling still pending. |
| FND-002 | still-open | Fresh confined check /tmp/IR-1005-review-fixes-check.log at 2a7e524 exits 101 with exactly 24 E0308 duplicate-owner type errors. No independently exposed local library error remains in that check; feature tests still cannot compile/execute before IR-1377. No runtime or final-gate clearance. |
| FND-003 | still-open | Five genuine binding-axis regressions now cover declared captures, real prior ledger, unchanged payload reuse, changed payload contradiction, type/predicate changes and later shared-node source spans. They are unexecuted, and the required authored model alias mutation/order cases remain blocked on FND-004; full original coverage finding is not yet closed. |
| FND-001 | still-open | Round 2: actual native Failed execution regression source exists, but all runtime qualification remains blocked. Full exact corrected-pair acceptance and genuine Unsupported first-row reachability still require owner capabilities/ruling; refusal-only correction evidence is not success. |
| FND-002 | still-open | Round 2: confined check /tmp/IR-1005-review-round2-check.log at 1f921764 finished with exactly 24 E0308 diagnostics and 24 duplicate-owner crate notes, with no independently exposed local library error. New tests and full acceptance behavior remain uncompiled/unexecuted; no IR-1377 cohort adoption or final gate pass claimed. |
| FND-003 | still-open | Round 2: genuine declared-capture, retained ledger/environment, type/predicate, later source occurrence and newly retained-anchor/fault controls now exist in source. They remain unexecuted, and required authored model-alias mutation/order evidence is still absent pending Code FND-004. Full original coverage finding remains open. |
| FND-001 | still-open | Supplemental native adoption: genuine native correction issuance/full-prior substitution/strict-read and native owner-limit regression source is added, using actual supplied prior Reports. TL exact predecessor validation remains an honest authority refusal; full correction success, Unsupported row reachability and runtime acceptance remain unqualified. |
| FND-002 | still-open | Supplemental native adoption at 061f8d7 is not built: QSL-1795 API is still pending its owner final gate/merge, no consumer lock refresh or override was used, and IR-1377 duplicate owner types still block the integration cohort. New tests have no executed pass and no final combined-tree gate has run. |
| FND-003 | still-open | Supplemental native adoption retains prior genuine capture/type/predicate/source controls and adds actual native predecessor checks. Authored alias mutation/order evidence and runtime qualification remain open; structural roots are not aliases. |
| FND-001 | still-open | Round 4: native/TL foreign corrected-control source is added, but the helper currently cannot obtain a correction because it does not advance input identity (Code FND-008). TL exact predecessor authority, Unsupported row reachability and full runtime acceptance remain unqualified. |
| FND-002 | still-open | Round 4: no Cargo or dependency refresh ran at 1dcc314 while QSL-1795 owner final gate/merge is pending. Integration cohort/runtime acceptance remains blocked, and no runtime/final combined-tree gate pass is claimed. |
| FND-003 | still-open | Round 4: existing genuine capture/type/predicate/source controls remain present; alias mutation/order facts and runtime verification remain unresolved. The new correction helper requires the independent FND-008 fix. |
| FND-001 | still-open | Round 5: correction controls now have a structurally coherent genuine progress advance and owner admission prerequisites, but their runtime execution remains unverified. Complete cross-owner corrected agreement and genuine Unsupported row acceptance remain open pending owner capabilities/ruling and actual tests. |
| FND-002 | still-open | Round 5 at 44512a7: no Cargo, manifest or dependency refresh ran while the native QSL-1795 owner final gate/merge is pending. Integration cohort/runtime qualification and final gate remain absent; source inspection and formatting are not a compiled test pass. |
| FND-003 | still-open | Round 5: genuine capture/type/predicate/source and progress-correction fixture source is present; actual authored model alias/order facts and runtime assertions are still unqualified. No alias is fabricated or inferred from structural roots. |
| FND-001 | still-open | Round 6: genuine progress-corrected and properly isolated mixed-relation test source now resolves the local FND-008/009 defects, but no runtime pass has occurred. TL full predecessor success, Unsupported row reachability and complete cross-owner acceptance remain open. |
| FND-002 | still-open | Round 6: no Cargo, manifest or dependency refresh ran during these fixes while the native owner final gate/merge is pending. Source checks do not establish compilation or runtime fixture admission; integration cohort and final gate remain unqualified. |
| FND-003 | still-open | Round 6: genuine capture/type/predicate/source/progress/correction fixtures are present and the mixed case now isolates the intended relation with a valid prior. Authored alias/order facts and actual runtime assertions remain unqualified. |

## Supplemental verification — round 1

Reviewed clean frozen head 2a7e5240006aa75d1da135f1c2d3883e43858ef4 without builds or source edits. Static fixes do not grant runtime clearance. The exact owner source span is now selected by node/role/ordinal; native execution is independent; the public native contradiction operation and retained environment are consumed. New tests use genuine emitter/strict-reader/ledger values. Structural formatting/diff/spec checks are lane-reported; the confined check still has exactly 24 duplicate-owner E0308 diagnostics and no runtime test pass. Full predecessor, model-alias and supported owner-domain gaps remain open.

## Supplemental verification — round 2

Reviewed frozen clean head 1f92176440b1b0f77320d504b10d0b6a0053ba7f without builds or source edits. Original Code FND-001/002/003 remain fixed with no regression in this delta. FND-005/006 are statically fixed by actual retained owner anchor checking and complete typed ActivationFault retention. New tests obtain prior ledgers and faulted instances through genuine native CheckedSubject/evaluation; the bridge itself remains non-evaluating. IT-022 specifies the exact distinctions and TC-115 names all actual feature targets. No new local defect found in the reviewed delta. Compile/runtime clearance remains absent: the round-2 confined check at this exact head completed with exactly 24 E0308 diagnostics, each with a duplicate-owner crate note, and no independently exposed local library error. Alias/cohort/full predecessor/supported-domain acceptance gaps remain open; NOT mergeable.

## Supplemental native predecessor adoption

Reviewed frozen clean 061f8d7ca95cf40f228c6401a87bc8b37981e275, delta since d045754, statically without builds or source edits. Read the actual QSL-1795 owner implementation from its existing owner worktree; its final gate/merge is pending, not a consumer pin. The consumer uses actual ReportPredecessorLimits, bounded native writing, exact validate_predecessor against the supplied previous join native value, and CheckedSubject::with_predecessor for strict admission. Embedded prior bytes are not re-read to mint authority. Native writer/validator receipts remain distinct from seven bridge counters and shared IR allowance; no invented reader receipt or reset native budget is claimed. Canonical nativeAdmitted is derived from actual strict-read success, not accepted as caller trust. Genuine correction/substitution/writer-bound/shared-budget test source is present but unexecuted. Original Code FND-001/002/003/005/006 remain statically fixed. New FND-007 blocks local clearance; model alias, TL predecessor, supported-domain, cohort and runtime gaps remain open. NOT mergeable.

## Supplemental verification — round 4

Reviewed clean frozen head 1dcc314c2849c6d1d5a504a278cf0d0e8bf95f6a, actual delta from 061f8d7, without builds or source edits. FND-007 production refusal ordering is statically fixed: no pre-validation shortcut skips the native strict reader, actual unrelated owner refusals remain first-row Refused, only InvalidPredecessor becomes correction Conflict, and unavailable TL predecessor authority cannot suppress already matched non-value rows. Full native validation remains against the actual previous join native value and retains bounded receipts. New FND-008 identifies an unreachable correction fixture, not a manufactured-result workaround. Original Code FND-001/002/003/005/006 remain fixed; alias and full cohort/TL-predecessor/supported-domain/runtime gaps remain open. No invalid Boolean agreement found in this delta, but NOT mergeable.

## Supplemental verification — round 5

Reviewed frozen clean head 44512a7b3beb97fe27ad71c8778aa4ab4989c8e2, including ae7f362 progress fixture change and the confined evidence::ResultIdentity namespace correction, without builds or source edits. FND-008 is statically fixed: actual QObs producer/reader progress and closure watermark advances from 1 to 2, native progress comes from those owner values, and the original package/formula/clock/carrier/correspondence remain coherent. QObs boundary validation permits the new watermark; native InputIdentity includes progress while excluding ledger/relation, and TL correction context permits changed progress with stable formula/clock/subject/profile. One helper asserts relation/ledger-only refusal, distinct actual input identities, Evaluated native correction, full owner predecessor validation and strict native read before downstream assertions. Existing Code FND-001/002/003/005/006/007 remain fixed statically. No new local defect in this delta. Alias/cohort/TL-predecessor/Unsupported/past-unit/runtime gaps remain open. NOT mergeable.

## Supplemental verification — round 6

Reviewed frozen clean head 3a46a1021808eaf98fc7a43504ac997d01444842, including the confined ca7de913 and 3a46a102 mixed-context fixes, without builds or source edits. FND-009 is statically fixed: the TL Original result is genuinely assessed for the current watermark-2 owner request, its relation and exact request are asserted, and the actual valid previous join enables native correction admission before the mixed-relation Conflict/no-Boolean assertion. Thus wrong-request Refused or missing-prior Conflict cannot stand in for the intended branch. FND-008 remains fixed through the original owner progress/identity/strict-admission helper. No new local defect in this delta. Code FND-001/002/003/005/006/007/008/009 are statically fixed; alias FND-004 and full cohort/TL-predecessor/Unsupported/past-unit/runtime qualification remain open. NOT mergeable.
