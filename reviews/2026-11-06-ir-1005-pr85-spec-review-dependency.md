---
id: SR-2026110414560139
title: "IR-1005 PR85 spec-review/dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-integration; IR-1005; PR85; changed temporal production, definition, specifications and tests"
review_set: subset
---

## Summary

Ticket: IR-1005. Reviewed frozen head 3d186f5a0d4bfb5d06d02367cb3a29a5e91e5f6b against origin/main. One existing worktree; no source edits, builds, commits, pushes or owner-side changes. The links and ownership are sound. Final readiness remains conditional on IR-1377 (luna:IR:codex1), IR-1378, exact full native/TL predecessor authority and supported clock representation. No pins/patches/second-copy conversions were added. Conditional context is not an extra defect row.

## Verdict

**CONDITIONAL** for this method. No final merge clearance: the source does not compile in the current duplicate-owner cohort and the runtime tests are unverified. Required owner and full-scope gaps remain explicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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
