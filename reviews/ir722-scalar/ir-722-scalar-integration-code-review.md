---
id: SR-20261102072297
title: "IR-722 scalar quire-integration code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-722; frozen scalar amendment and interface003 doc fix; interface_008, IT-018-SC-10"
review_set: subset
---

## Summary

No concrete scalar runtime bug found. The frozen API-doc omission is recorded then verified fixed at dbea2d5; same-Machine evidence remains deferred.

Frozen scalar source on original RT193 f3927c041579b7ae678f0c431e8b5127ce98a318 and integration35 5a51d1dcca86f98f762a6726e244c0be2f75e67b. The later interface003-only documentation fix dbea2d55b7a925010a0c1bf709740ca034eb2756 is included for disposition; constructor spec/production continuation is excluded. Read frozen Git snapshots, repository guidance, Rust/code review and selected scoped interface methods. No worktree source edits, Cargo/heavy/proof jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

No concrete arithmetic/order/type/ownership defect was found. Binary child frames are pushed right then left, hence execute left first; scalar handling pops right then left and delegates exactly to shared evaluate_integer_arithmetic/order_numbers. Integer/Int inputs and arithmetic result annotations are checked; ordering requires Boolean. The declared Int result interval is forwarded to the native operation. Native Meter is created once per invocation with required explicit ScalarLimits and reused across selected operations, including helpers; no host-width arithmetic or private numeric semantics appear.

Completed integers move into canonical Value storage and are addressed by a private append-only computed arena. Computed/projection/binding/argument/operand addresses all participate in scratch/native live-element accounting and clear after invoke; a canonical composite result is retained first. Native Boolean order results use existing canonical constants. Incomplete retains LimitKind, exact Integer next_charge and original native outcome/charge point; refusal/undefined retain native cause and source as Scalar invariant evidence. No partial scalar becomes a value or temporal outcome. Resource boxing and non-Copy exact requested fields are explicit public-shape changes; no fallback budget/default is invented. RefCell denial state is only borrowed after synchronous native reservation callbacks return.

Eight scalar controls cover independently authored >128-bit decimal expectations, twelve ordering rows, bounded-domain refusal, invalid annotations, native shortage/source/charge evidence, cumulative result units, inactive scalar branches, left-first native failure and computed scratch. Preserved whole monitor_program target has 52 passing tests and touched Clippy ONLY on the archived interim cohort. RT all-target/all-feature Clippy and three existing native controls pass on current Exact a3052598 / QSV 25535387. Restored committed cohort, our final dependency-head long chain, producer, external coverage acceptance and final aggregate remain unqualified. Same-Machine cleanup after stop remains explicitly open until dispatch. Full phase 2/event/watermark/gap/FR-276 remains unfinished; IEEE has NO backend replay credit.

Plan completion: not assessed

## Verdict

Initial scoped verdict: **CONDITIONAL**. Current broader readiness remains CONDITIONAL; gap-analysis remains FAIL for unresolved acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The stopped-invocation/later-execution clause remains unqualified: scalar scratch/resource controls rebuild a fresh Machine after failure, so they do not prove that one stopped Machine clears computed values and cumulative state before re-entry. Keep this known clause open until entry-dispatch controls; no API widening or fabricated RT identity fixture is requested. | tests/monitor_program.rs:scalar stop controls |

## Examined scope

```yaml
scope:
- id: IT-018-SC-10
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: Execute exact scalar diagnostics.
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: The controlled executable parts exercise construction, not temporal producer/executor
    qualification.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-008-AC-1
  path: spec/monitor/functional/interface-008-monitor-scalar-operations.md
  role: examined
  excerpt: Arithmetic and all four order relations agree with independent exact expectations,
    including integers beyond host width and bounded result refusal.
- id: interface-008-AC-2
  path: spec/monitor/functional/interface-008-monitor-scalar-operations.md
  role: examined
  excerpt: Explicit native limits apply cumulatively across selected operations; inactive
    branches do not charge, and denial retains its exact request, native cause and
    source evidence without a value or temporal verdict.
- id: interface-008-AC-3
  path: spec/monitor/functional/interface-008-monitor-scalar-operations.md
  role: examined
  excerpt: Computed values consume scratch; ordinary interpreter stops retain their
    exact units, and a stopped invocation cannot commit state or contaminate later
    execution.
bindings:
- test_id: integer_arithmetic_uses_exact_independent_values_beyond_host_width
  ac_id: interface-008-AC-1
  trace: correct
- test_id: integer_native_shortage_retains_exact_charge_and_original_source_without_a_value
  ac_id: interface-008-AC-2
  trace: correct
- test_id: integer_order_relations_follow_independent_three_way_truth_rows
  ac_id: interface-008-AC-1
  trace: correct
- test_id: integer_bounded_result_refusal_preserves_native_cause_as_an_invariant
  ac_id: interface-008-AC-1
  trace: correct
- test_id: integer_native_limits_are_cumulative_and_lazy_branches_do_not_charge
  ac_id: interface-008-AC-2
  trace: correct
- test_id: integer_computed_canonical_values_consume_explicit_scratch
  ac_id: interface-008-AC-3
  trace: correct
- test_id: integer_operations_refuse_noninteger_operands_and_wrong_result_annotations
  ac_id: interface-008-AC-1
  trace: correct
- test_id: integer_binary_operands_stop_on_the_left_native_operation_before_the_right
  ac_id: interface-008-AC-2
  trace: correct
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The existing IR-722 dispatch work retains same-Machine cleanup-after-stop qualification. Current public build controls use fresh machines; no full AC-3 credit, acceptance narrowing, API widening or fake semantic-identity fixture is granted. |

The documentation fix explicitly adds required scalar limits, per-invocation/cumulative native accounting, zero ceilings, exact requested units and ScalarFailure evidence. Other constructor ancestry/source changes are outside this carry.

```yaml
dispositions:
- fnd: FND-001
  outcome: deferred
  reason: The existing IR-722 dispatch work retains same-Machine cleanup-after-stop
    qualification. Current public build controls use fresh machines; no full AC-3
    credit, acceptance narrowing, API widening or fake semantic-identity fixture is
    granted.
```

Current method verdict after the documented disposition: **CONDITIONAL**. All broader qualification blockers remain explicit.
