---
id: SR-20261102072229
title: "IR-722 initializer quire-integration spec-review/evidence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-integration; IR-722; initializer diff; interface_003, IT-018-SC-05, monitor budget/executor/vm and companion tests"
review_set: subset
---

## Summary

quoin advise ran over the frozen snapshots. All four RT criteria are catalog-inconclusive; selecting executable Test controls is explicit reviewer judgment because the observations are concrete public API outcomes. Current controls leave the recorded helper-call and native-resource subclauses uncovered.

Frozen initializer slice: RT PR195 c962a1ef527ffa8d0656ad441ee98181ad59ce8e over constructor f0fcea6e; integration PR36 97b244931d296060d02f2bbca7567b8007e07ece over 33020725707bd6e559c00090fc33138f6a0ec5a3. Reviewed actual Git snapshots, repository guidance and public interface. No source edits, Cargo, Kani or heavy jobs run. Supplied evidence is five new controls / nineteen monitor_program tests passing, touched integration Clippy and RT all-target/all-feature Clippy passing; the final aggregate is outstanding. Exact-panic, full feature/target/deny gates and complete proofs have not been established for this new source slice. Parent constructor strict coverage and root IEEE proof remain unresolved. Event/watermark/gap dispatch, full temporal producer and FR-276 acceptance are explicitly outside this initializer slice; QSpec #597 continuation is not reviewed.

No applicable AssuranceProfile found. No tests remint semantic keys in RT; the companion controls use the established live QSL/strict-IR admitted identities. Static source tracing supports the immutable type-fact cache induction: literal types are constructor-admitted; Arguments inherit checked invocation operands; selected If/Call mismatches cross shared membership before propagating their declared type. There is no mirror Value representation, host callback, dependency change or weakened gate. Native Invalid retains arena and becomes Membership invariant; other native provenance loss is recorded where defective.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — Findings require disposition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No control executes a Call: the only new Call sits in an unselected recursive branch, while the deep-chain control consists entirely of If nodes. Consequently helper argument forwarding, return continuations, call-edge equality caches and actually selected recursion are untested, despite the helper-chain acceptance clause. Add actual nested Argument/Call and selected-recursion resource controls. | tests/monitor_program.rs:1190-1306 |
| FND-002 | medium | The only shared-membership execution control exercises Invalid for a Boolean initializer. Zero interpreter limits and impossible replacement-vector reservation do not exercise native membership work/element/byte denials. The AC-3 native limit/storage clause and cumulative interpreter-plus-membership accounting have no control; add genuine nonidentical-edge admission with exact stop payload assertions. | tests/monitor_program.rs:1233-1265 |

## Examined scope

```yaml
scope:
- id: IT-018-SC-05
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "Execute ordinary diagnostic initializers. IT-018-SC-05: two builds retain\
    \ the\n   same immutable owner while owning distinct state replacement slots."
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: The controlled executable parts exercise construction, not temporal producer/executor
    qualification.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-003-AC-1
  path: spec/monitor/functional/interface-003-monitor-executor.md
  role: examined
  excerpt: A retained admitted program builds after preparation drops, executes its
    actual initializer and keeps its complete canonical State and fixed owner. Two
    builds have independent state slots, consume no event and produce no obligation
    result.
- id: interface-003-AC-2
  path: spec/monitor/functional/interface-003-monitor-executor.md
  role: examined
  excerpt: Zero or insufficient work, stack and scratch limits stop initialization
    through the resource channel and return no Monitor. A lazy If does not execute
    its unselected branch; a helper chain spends bounded explicit continuations without
    Rust recursion.
- id: interface-003-AC-3
  path: spec/monitor/functional/interface-003-monitor-executor.md
  role: examined
  excerpt: A dynamically wrong function result or operand reaches the shared conformance
    boundary and returns its typed failure rather than a false property result. Native
    limit/storage failures preserve the operating-stop channel.
- id: interface-003-AC-4
  path: spec/monitor/functional/interface-003-monitor-executor.md
  role: examined
  excerpt: Replacing operating limits leaves the canonical State and fixed Program
    unchanged; a failed replacement reservation preserves the old limits. No compatibility
    default or live rebind is exposed.
bindings:
- test_id: executor_build_retains_owner_and_initializes_independent_state_slots
  ac_id: interface-003-AC-1
  trace: correct
- test_id: executor_operating_stops_and_budget_replacement_are_typed_and_atomic
  ac_id: interface-003-AC-2
  trace: correct
- test_id: executor_operating_stops_and_budget_replacement_are_typed_and_atomic
  ac_id: interface-003-AC-4
  trace: correct
- test_id: executor_lazy_branch_does_not_enter_an_unselected_recursive_call
  ac_id: interface-003-AC-2
  trace: correct
- test_id: executor_wrong_initializer_result_preserves_native_membership_failure
  ac_id: interface-003-AC-3
  trace: correct
- test_id: executor_deep_lazy_chain_uses_bounded_explicit_stack
  ac_id: interface-003-AC-2
  trace: correct
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified actual fix source and applicable cohort-specific evidence; exact after_excerpt below. |
| FND-002 | fixed | Verified actual fix source and applicable cohort-specific evidence; exact after_excerpt below. |

Disposition round 1 inspected RT e638365b223a57c88228d6eec7bd897a78b28951 and integration 71f447a1366fa46be0185ea5d96762c23d862b91. Native MembershipStop now carries StepResourceStop plus original SuppliedAdmissionError and resolvable arena; pre-arena identifier failure retains boxed PathArenaCapacity. Positive nested Call/Argument helpers preserve the second State argument (now=99) across scopes; selected recursion stops at the explicit stack ceiling. Private native controls cover genuine QSV Option<Boolean> work/element denials, live operand-plus-reservation accounting, and cumulative byte reservation at the production permit boundary. The current QSV walker requests AdditionalElements, not Bytes; no native byte failure is fabricated or credited. The public work-stop control asserts native cause/arena and combined spent=3.

Evidence is cohort-specific: twenty-two integration monitor_program controls and touched Clippy pass only on the explicitly preserved interim old-leaf lock using current RT source. RT standalone native three tests and all-target/all-feature Clippy pass on Exact a3052598 / QSV 25535387. The committed new integration cohort has not passed target compilation; supplied blocker information identifies CG adoption and six driver adoption errors. Interim green is not committed-lock qualification. Final aggregate, constructor coverage119/external bindings and root IEEE proof remain unresolved; no merge-ready or full FR-276 claim.

Constructor carry f0fcea6e to ce04eebb was inspected separately through Git: no production source delta; FR-276 documentation adds the published event_position/activation-position explanation, Cargo.lock selects the new single Exact/QSV cohort, and the existing exhaustive consumer test explicitly admits InexactConversion. Supplied two consumer tests pass; this does not establish broader cohort or proof readiness. This carry changes none of constructor SR originals. Reviewer ran no Cargo/heavy job and edited no worktree source.

Current result: Source/evidence findings fixed within reviewed slice; committed integration-cohort and aggregate gates remain unresolved.

```yaml
new_findings: []
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "assert_eq!(\n        now,\n        &Integer::from(99_u64),\n   \
    \     \"second argument survives nested invocation and scope rollback\"\n    );\n\
    }"
- fnd: FND-002
  outcome: fixed
  after_excerpt: "let native = failure\n        .admission\n        .expect(\"original\
    \ native membership stop retained\");\n    match (expected, native.failure) {\n\
    \        (\n            StepResource::Work,\n            quire_semantic_value::supplied_membership::SuppliedAdmissionError::Limit\
    \ {\n                locus, ..\n            },\n        ) => {\n            assert_eq!((stop.spent,\
    \ stop.requested), (3, 1));\n            assert!(native.arena.resolve(locus).is_some());\n\
    \        }\n        _ => panic!(\"native cause must match the operating stop,\
    \ without an invariant\"),\n    }"
```

## Final fix and canonical-kind carry

Final disposition carry inspected RT 466677da59ad91fa95de5bfaa12d3417f0debc93 and integration 71f447a1366fa46be0185ea5d96762c23d862b91. All three native diagnostic tests now bind only interface-003-AC-3, removing false TC-200 case credit; focused native three tests pass on Exact a3052598 / QSV 25535387. Original native-cause, positive Call/Argument, selected-recursion and membership-budget findings remain fixed. The newly introduced literal QSpec revision was removed at 466677da; the QSpec #597 / FR-487-AC-12 normative reference remains.

The atom-projection normative carry was checked against published QSpec #597: preserve the canonical Incomplete limit kind atom_projection, exact AtomId and monitor.atom_projection setting, without an allow-list, new carrier/State/root/API, host callback or conversion to StepResourceStop. No executor implementation or initializer test is credited with this temporal outcome. The two added FR-276 SHALL statements are concrete and singular; the new EARS method is clean, with only pre-existing Reporting grammar warning treated as context.

Evidence remains cohort-qualified: twenty-two integration tests/touched Clippy pass on the saved interim old-leaf lock with current RT source, while standalone RT native tests/Clippy pass on the new Exact/QSV cohort. The committed new integration lock remains blocked before target compilation by counterpart adoption errors; no current-cohort integration green is claimed. Constructor ce04eebb docs/lock/explicit InexactConversion test-arm carry was inspected with no production source delta. Parent coverage119/cross-repository discharge, root IEEE proof/backend ruling and final aggregate remain unresolved; no merge-ready or full FR-276 claim. Reviewer ran no Cargo/heavy job and edited no worktree source.

Current result: CONDITIONAL for committed-cohort qualification; inspected source/evidence findings are fixed.

```yaml
new_findings: []
dispositions: []
```
