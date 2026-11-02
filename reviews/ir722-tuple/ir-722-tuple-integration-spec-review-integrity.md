---
id: SR-20261102072271
title: "IR-722 tuple quire-integration spec-review/integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-integration; IR-722; frozen tuple amendment; interface_006, IT-018-SC-08, monitor projection and owner lifetime"
review_set: subset
---

## Summary

The tuple interface is consistent with retained immutable ownership, shared declaration authority and existing scratch/work channels. Public construction rejects wrong shape/position/type, and temporary owner/result lifetime ordering is explicit.

Frozen original PR193 tuple source 4430b60bd3244b2be603714d2ac3fff07cd87a3d (current trailing-artifact head e66bc784c584548d2c5538d64e1710ef3e45ad3b) over Boolean source af4c3d8; integration PR35 controls 6c3807089f1c178e2c74c39deff17eb63f021720 over 7fcff282. Later record-operation source is excluded. Read frozen Git snapshots, repository guidance, code-review/Rust rules and selected scoped interface methods. No worktree source edits, Cargo/heavy/proof jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

No concrete runtime ownership or selected exact-tuple logic defect was found. Constructor validation resolves a declared tuple receiver and exact position/result type; receiver edges participate in existing cycle/owner validation. The iterative TupleElement continuation evaluates once, retains an Arc to the original canonical CompositeValue plus its position, and stores only an address in operands. Projection cells count in slot and native live-element accounting until invocation end; prepared capacity is bounded by the same scratch ceiling. Receiver owner/declaration/required-slot guards produce typed Projection invariants, while remaining nonidentical slot conformance uses existing bounded admission and retains native failure shape. Shared equality/nominal methods supply type facts rather than a new Value/type checker. Dynamic conformance behavior still needs the recorded control.

The private invoke now retains an owned canonical composite result with Arc::clone before clearing values, arguments, touched bindings and projection owners. This matches the current public State/Step entry shape and preserves a projected State after temporary cells release; no subtree is cloned. Operands copied through calls/locals remain stable indices for the invocation lifetime. Exact-tuple controls inspect distinct positions, reject record/position/type errors, assert Arc owner counts, exercise tight projection storage and 2,000 explicit continuations on 512 KiB native stack.

Supplied evidence: four tuple controls and the whole monitor_program target (38 passing tests), plus touched integration Clippy, pass ONLY on the preserved interim lock with current RT source. RT all-target/all-feature Clippy passes on Exact a3052598 / QSV 25535387. The restored committed new integration cohort remains blocked by documented driver adoption errors; no current-cohort target qualification is claimed. Our final dependency-head long chain, real producer, external coverage acceptance and final aggregate remain owed. IEEE evidence is source/compile-only and there is NO backend/CBMC replay credit. Full ordinary operations and event/watermark/gap/FR-276 acceptance remain unfinished; same-Machine lexical cleanup-after-stop remains deferred until dispatch controls.

Plan completion: not assessed

## Verdict

**PASS** — No additional defect within this method; other findings and aggregate blockers remain binding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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
bindings: []
```
