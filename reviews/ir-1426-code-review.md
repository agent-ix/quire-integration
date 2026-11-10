---
id: SR-001
title: "Code and Rust review of monitor denial controls"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1426; PR 100; tests/monitor_tick_producer.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-integration/IT-018", type: references}
---

## Summary

Independent review of frozen PR 100 at ed5dc6f3c678473ff5967dc7728a7f70b93213f9. The entire changed test file diff, both new denial oracles, existing producer preparation and runtime input/build boundaries, and the shared one-shot allocator were inspected. No production code, public API, dependency, CI, held branch or cross-repository file copy is changed.

## Verdict

PASS for this scoped test change; no actionable defects found. Missing admitted order is refused before state/baseline changes in both arrival orders, and a correctly associated retry consumes exactly one event. Runtime build denial executes a genuine fallible allocation and checks typed Resource/Storage, absent configured limit, no monitor and successful retry using the same immutable prepared program. This is not full TC-200 or full FR-276-AC-13 acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

No applicable AssuranceProfile was found under spec/. Rust API, implementation, safety/concurrency, boundary and testing guidance was applied; no Tokio path is touched. The shared FailingAllocator delegates to System, is thread-local and disarms after its one real denial. Preparation and assertions remain outside the armed region. No persistent-OOM or allocation-free refusal-projection claim is made.

- FR-276-AC-4, examined: "Changing the counter identity, unit, width, profile or admitted order never reuses the first plan's monitor state or settles its obligation; equal timestamps without admitted order never acquire causality from arrival order."
- FR-276-AC-13, examined: "For build and a step, each `StepBudget` field set one below its measured use returns the stated `StepResource` with exact `limit`, `spent` and `requested`; a refused allocation returns `StepResource::Storage` with no `limit`; source inspection shows `PathArenaCapacity` surfaced in `path_capacity`."
- FR-276-AC-6, context_only: "Build consumes no event and emits no verdict; a second build has independent state, and neither monitor can be rebound to another plan."
- FR-276-AC-7, context_only: "An unsupported entry operation yields `Unsupported`, and RT allocation or initialization failure returns typed build failure with no monitor."
- FR-276-AC-8, context_only: "an RT executor invariant failure returns `Failed`, terminates the monitor and rejects later input."
- FR-276-AC-14, context_only: "The RT PR alone does not discharge the producer-admitted construction cells."
- TC-200, context_only: "Every foreign or incomplete binding is refused before it can settle an obligation; no order is inferred from equal ticks."
- IT-018, context_only: "Exercise the RT monitor construction boundary while preserving the producer ownership of semantic identity."

AC-4 is examined for missing-order input refusal only: the producer's immutable program still owns an admitted order; this PR does not manufacture an orderless plan. AC-13 is examined only for a runtime build allocation refusal, not ProgramParts construction, Layout/parameter/result/expression type checks, literal-membership denial, every StepBudget field or arena-ID exhaustion. Other criteria above are context only; the ticket records the absent product APIs instead of adding test-only mutation, rebinding, fault-injection, parts-export or counter-setting APIs.

All new tests exercise existing production seams. The whole state uses the file's retained debug fingerprint helper, and the baseline is asserted separately. Removing the order comparison, committing a refused input, emitting the wrong failure class, returning a monitor despite denied storage, or failing a subsequent valid retry would fail the corresponding oracle.

Verified recorded checks: scoped rustfmt/diff checks; clippy with warnings denied exit 0; test-binary compilation exit 0; two new exact tests and an existing foreign-binding control each ran one test and passed with exit 0. Logs: /tmp/ir1426-{fmt,clippy-resume,build,order-test,allocation-test,binding-control}.log. Fixture tests ran without flock after a Cargo/rustc-clear check because preparation invokes cargo metadata. No Cargo or test command was run by this reviewer.

The actual codex exec review completed exit 0, inspection only, no actionable bugs (/tmp/ir1426-codex-review.txt and .log). It ran no Cargo, rustc, formatting or test command. The lead owns the post-review affected-binary verification and authorized merge request.

