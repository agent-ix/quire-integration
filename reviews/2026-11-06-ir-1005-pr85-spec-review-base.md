---
id: SR-2026110414560136
title: "IR-1005 PR85 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-1005; PR85; changed temporal production, definition, specifications and tests"
review_set: subset
---

## Summary

Ticket: IR-1005. Reviewed frozen head 3d186f5a0d4bfb5d06d02367cb3a29a5e91e5f6b against origin/main. One existing worktree; no source edits, builds, commits, pushes or owner-side changes. Changed IT-022/TC-115 have valid identities, explicit boundaries, deterministic outcomes, owner allocations and honest incomplete qualification. They do not withdraw FR-350 obligations or claim executed runtime coverage.

## Verdict

**PASS** for this method. No final merge clearance: the source does not compile in the current duplicate-owner cohort and the runtime tests are unverified. Required owner and full-scope gaps remain explicit.

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

Plan completion: not assessed. Scope is the actual changed 21 files and owning external FR-350 criteria, not a whole-repository audit. No applicable AssuranceProfile is present under spec/. Optional unrelated analyses and optional semantic gap fanout were not run. No Tokio/async/unsafe code or hosted workflow change is in this diff. Compile evidence is the coder's focused check3, not a reviewer rerun. The unchanged dependency cohort rule was not waived.

## Evidence

- Actual producer/strict-reader fixture and assertion inspection for projection, definition and join tests; no sealed result fabrication is accepted as coverage.
- FR-350 and TC-461 from the actually locked Contract IR owner; current IR-1005 thread and normative corrections reviewed as context.
- `quire matrix --scope . --strict --format json` returned exit 1: no obligations matched the integration spec scope. This is why the external criterion mapping above is manual and does not claim a clean computed matrix.
- Spec/artifact validation is structural only; it does not replace runtime evidence or correct the code findings.

## Supplemental verification — round 1

Reviewed clean frozen head 2a7e5240006aa75d1da135f1c2d3883e43858ef4 without builds or source edits. Static fixes do not grant runtime clearance. The exact owner source span is now selected by node/role/ordinal; native execution is independent; the public native contradiction operation and retained environment are consumed. New tests use genuine emitter/strict-reader/ledger values. Structural formatting/diff/spec checks are lane-reported; the confined check still has exactly 24 duplicate-owner E0308 diagnostics and no runtime test pass. Full predecessor, model-alias and supported owner-domain gaps remain open.

## Supplemental verification — round 2

Reviewed frozen clean head 1f92176440b1b0f77320d504b10d0b6a0053ba7f without builds or source edits. Original Code FND-001/002/003 remain fixed with no regression in this delta. FND-005/006 are statically fixed by actual retained owner anchor checking and complete typed ActivationFault retention. New tests obtain prior ledgers and faulted instances through genuine native CheckedSubject/evaluation; the bridge itself remains non-evaluating. IT-022 specifies the exact distinctions and TC-115 names all actual feature targets. No new local defect found in the reviewed delta. Compile/runtime clearance remains absent: the round-2 confined check at this exact head completed with exactly 24 E0308 diagnostics, each with a duplicate-owner crate note, and no independently exposed local library error. Alias/cohort/full predecessor/supported-domain acceptance gaps remain open; NOT mergeable.
