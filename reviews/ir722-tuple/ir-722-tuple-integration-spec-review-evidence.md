---
id: SR-20261102072272
title: "IR-722 tuple quire-integration spec-review/evidence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-integration; IR-722; frozen tuple amendment; interface_006, IT-018-SC-08, monitor projection and owner lifetime"
review_set: subset
---

## Summary

quoin advise ran on frozen RT and integration snapshots. New interface criteria are catalog-inconclusive; executable Test controls are chosen by reviewer judgment. Arc-count and tight-budget assertions are meaningful, but the dynamic-conformance contract remains untested as recorded. Interim green does not qualify the restored committed cohort.

Frozen original PR193 tuple source 4430b60bd3244b2be603714d2ac3fff07cd87a3d (current trailing-artifact head e66bc784c584548d2c5538d64e1710ef3e45ad3b) over Boolean source af4c3d8; integration PR35 controls 6c3807089f1c178e2c74c39deff17eb63f021720 over 7fcff282. Later record-operation source is excluded. Read frozen Git snapshots, repository guidance, code-review/Rust rules and selected scoped interface methods. No worktree source edits, Cargo/heavy/proof jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

No concrete runtime ownership or selected exact-tuple logic defect was found. Constructor validation resolves a declared tuple receiver and exact position/result type; receiver edges participate in existing cycle/owner validation. The iterative TupleElement continuation evaluates once, retains an Arc to the original canonical CompositeValue plus its position, and stores only an address in operands. Projection cells count in slot and native live-element accounting until invocation end; prepared capacity is bounded by the same scratch ceiling. Receiver owner/declaration/required-slot guards produce typed Projection invariants, while remaining nonidentical slot conformance uses existing bounded admission and retains native failure shape. Shared equality/nominal methods supply type facts rather than a new Value/type checker. Dynamic conformance behavior still needs the recorded control.

The private invoke now retains an owned canonical composite result with Arc::clone before clearing values, arguments, touched bindings and projection owners. This matches the current public State/Step entry shape and preserves a projected State after temporary cells release; no subtree is cloned. Operands copied through calls/locals remain stable indices for the invocation lifetime. Exact-tuple controls inspect distinct positions, reject record/position/type errors, assert Arc owner counts, exercise tight projection storage and 2,000 explicit continuations on 512 KiB native stack.

Supplied evidence: four tuple controls and the whole monitor_program target (38 passing tests), plus touched integration Clippy, pass ONLY on the preserved interim lock with current RT source. RT all-target/all-feature Clippy passes on Exact a3052598 / QSV 25535387. The restored committed new integration cohort remains blocked by documented driver adoption errors; no current-cohort target qualification is claimed. Our final dependency-head long chain, real producer, external coverage acceptance and final aggregate remain owed. IEEE evidence is source/compile-only and there is NO backend/CBMC replay credit. Full ordinary operations and event/watermark/gap/FR-276 acceptance remain unfinished; same-Machine lexical cleanup-after-stop remains deferred until dispatch controls.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — Findings require disposition; broader qualification blockers remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | All four tuple controls use a receiver whose actual declaration equals its annotation, so they exercise only the exact=true projection path. The new conforming-dynamic-tuple/native_type_fact and nonidentical shared-admission path is untested despite the public contract. Add a control with genuinely admitted matching declaration/conformance facts and a distinct actual tuple declaration; preserve correct selected slot and native failure/resource behavior without fabricating source identities or supertype facts. | tests/monitor_program.rs:2230-2436 |

## Examined scope

```yaml
scope:
- id: IT-018-SC-08
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "Execute canonical tuple diagnostics. IT-018-SC-08: declared positions\
    \ select\n   distinct State values, and temporary owner references are released\
    \ before\n   build returns."
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: The controlled executable parts exercise construction, not temporal producer/executor
    qualification.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-006-AC-1
  path: spec/monitor/functional/interface-006-monitor-tuple-projection.md
  role: examined
  excerpt: A projected canonical State remains valid after temporary projection owners
    and preparation objects drop; tuple position order determines the selected value.
- id: interface-006-AC-2
  path: spec/monitor/functional/interface-006-monitor-tuple-projection.md
  role: examined
  excerpt: Record receivers, missing positions and mismatched result annotations refuse
    before entry execution.
- id: interface-006-AC-3
  path: spec/monitor/functional/interface-006-monitor-tuple-projection.md
  role: examined
  excerpt: Retained projection addresses are charged to scratch; deep projections
    use the explicit machine, and source keys/body paths remain available without
    copying a subtree.
- id: interface_006
  path: spec/monitor/functional/interface-006-monitor-tuple-projection.md
  role: context_only
  excerpt: 'A conforming dynamic tuple uses the same positional meaning.

    Shared type equality and native nominal conformance preserve already-admitted

    slot type facts; any remaining nonidentical conformance boundary uses the shared

    bounded membership walker.'
bindings:
- test_id: tuple_projected_state_outlives_the_temporary_owner_in_declared_position_order
  ac_id: interface-006-AC-1
  trace: correct
- test_id: tuple_projection_rejects_record_receivers_missing_positions_and_wrong_types
  ac_id: interface-006-AC-2
  trace: correct
- test_id: tuple_projection_retained_addresses_respect_scratch_budget
  ac_id: interface-006-AC-3
  trace: correct
- test_id: tuple_deep_projections_use_explicit_continuations_without_subtree_copies
  ac_id: interface-006-AC-3
  trace: correct
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified genuine-key, explicitly test-owned native-edge diagnostic and both native-fact/bounded-admission paths; exact after_excerpt below. |

Tuple disposition inspected RT 1b7ad105c008ef58282b2b2b36285b548bfbb1e5 (the tuple path is retained inside shared projection handling) and integration 55a53605667f81e854c8843a6ff3742427640d27. The new control uses genuinely emitted/strict-IR-read StatePairChild and BoundedIntegerPair declarations/shapes; native with_supertypes edges are explicitly test-owned, not claimed emitted by QSL. The first path has distinct actual declaration and preserves projected second State now=99 through native type facts. The Int[0,10]→Integer slot is nonidentical and outside the native composite/reference shortcut, reaching bounded supplied membership: work 4 produces resource Work with original admission present, while a generous budget succeeds. This closes the RT code-path coverage finding as a native diagnostic; it does not qualify producer-origin generalization edges or a temporal program.

The preserved dynamic control and whole 43-test target pass only on the archived interim cohort. Restored committed-cohort compilation, final own dependency-head long chain, real producer, external criterion acceptance, full FR-276 and IEEE backend remain unresolved. Existing same-Machine cleanup-after-stop remainder stays deferred until dispatch. No source edits, Cargo or heavy/proof jobs by reviewer.

Current result: Tuple source-path evidence finding **fixed**; committed-cohort/aggregate readiness remains **CONDITIONAL**.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "assert!(\n        failure.admission.is_some(),\n        \"original\
    \ native membership cause is retained\"\n    );\n    assert_eq!(failure.resource.unwrap().resource,\
    \ StepResource::Work);"
```
