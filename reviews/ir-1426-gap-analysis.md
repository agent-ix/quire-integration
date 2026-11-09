---
id: SR-002
title: "Scoped trace and gap review of monitor denial controls"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; IR-1426; PR 100; new monitor_tick_producer denial tests"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-integration/IT-018", type: references}
---

## Summary

Scoped changed-test trace, real-seam and coverage-inflation audit at ed5dc6f3c678473ff5967dc7728a7f70b93213f9. Both new test bindings and their source paths were inspected against current runtime FR-276 and TC-200. No Test Matrix, requirement or plan was authored.

## Verdict

CONDITIONAL only because repository-wide computed-matrix evidence is outside this dispatch and was not run. Manual scoped mapping found no introduced defect or inflated complete-criterion claim. The delivered denial controls are mergeable as explicitly partial evidence after the lead's final affected-binary check. This is not a repository-wide gap-analysis PASS, full TC-200 qualification or ticket-completion assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

No plan was supplied. No applicable AssuranceProfile was found. A broad computed matrix and repository-wide semantic audit were not run, per the bounded review dispatch. The two added trace attributes were checked directly, not inferred from test names or claimed as a new computed-matrix result.

- FR-276-AC-4, examined: "Changing the counter identity, unit, width, profile or admitted order never reuses the first plan's monitor state or settles its obligation; equal timestamps without admitted order never acquire causality from arrival order."
- FR-276-AC-13, examined: "For build and a step, each `StepBudget` field set one below its measured use returns the stated `StepResource` with exact `limit`, `spent` and `requested`; a refused allocation returns `StepResource::Storage` with no `limit`; source inspection shows `PathArenaCapacity` surfaced in `path_capacity`."
- FR-276-AC-6, context_only: "Build consumes no event and emits no verdict; a second build has independent state, and neither monitor can be rebound to another plan."
- FR-276-AC-7, context_only: "An unsupported entry operation yields `Unsupported`, and RT allocation or initialization failure returns typed build failure with no monitor."
- FR-276-AC-8, context_only: "an RT executor invariant failure returns `Failed`, terminates the monitor and rejects later input."
- FR-276-AC-14, context_only: "The RT PR alone does not discharge the producer-admitted construction cells."
- TC-200, context_only: "Every foreign or incomplete binding is refused before it can settle an obligation; no order is inferred from equal ticks."
- IT-018, context_only: "Exercise the RT monitor construction boundary while preserving the producer ownership of semantic identity."

Verified bindings: step_3_equal_tick_events_without_an_admitted_order_are_refused -> FR-276-AC-4 (correct missing-order refusal subset); step_11_build_allocation_denial_preserves_the_prepared_program_for_retry -> FR-276-AC-13 (correct runtime build-allocation subset). Both also carry TC-200. The module and PR explicitly distinguish runtime build from producer-admitted ProgramParts/type-check construction and retain other Planned rows as open.

The new tests compose source emission, genuine producer preparation and the runtime, with no fabricated semantic identity, altered producer plan, copied fixture, new product API or semantic reimplementation. Assertions require actual typed refusal, unchanged state/baseline or absence of a monitor, and a working valid retry. The allocator reports a real denied request, not a simulated failure result.

AC-6 rebinding, AC-7 arbitrary altered executable bindings, AC-8 producer-plan invariant injection, AC-13/14 private ProgramParts construction/type-check denial and global arena-identifier exhaustion remain outside the deliverable and documented on IR-1426. Immutable/closed product boundaries are not weakened solely to manufacture those tests. Source inspection confirms PathCapacity projection is distinct from ordinary vector allocation; the test's absent path_capacity does not claim that exhaustion path.

The existing logs each show one test for the two new exact controls and existing binding control, all passing exit 0. Clippy and test compilation also passed. No new ignored test, removed trace, stub, no-op production path, dependency pin or weakened gate was introduced. The lead's post-review affected-binary verification remains pending.

