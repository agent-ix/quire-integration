---
id: SR-20261102072315
title: "IR-722 dispatch batch quire-integration spec-review/scope-boundary"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-integration; original IR-722 PR35; frozen composite/sequence/union/equality/Boolean-meter/entry-dispatch batch; interface_009..012"
review_set: subset
---

## Summary

Exact/QSV own value semantics, verified member/equality plans, unit/conversion facts and native charge schedules. Driver owns independent input/authority admission and canonical value encoding. RT owns bounded interpretation, association comparison, commit/rollback and retained results; source owns outcomes/history/event_position.

Frozen original RT #193 production 65bd26971bbf23a9bce4432ef459536fc1b75dcd and original integration #35 260cfc3df529e895be47a83ac002d753f1fd2aa4. This is the same independent reviewer, one batched pass over composite construction, Sequence/size/flatten/fold, native union/equality, Boolean native metering and entry dispatch since the scalar slice. Old scalar doc finding is already fixed at dbea2d5; its finding ownership is retained in the original artifact. No duplicate old finding is opened here. Repository guidance, code/Rust style references and applicable scoped spec methods were consulted. No source/test edits, Cargo/Kani/CBMC or heavy jobs were performed. No applicable AssuranceProfile was found.

Canonical construction orders record operands by native declaration, rejects incomplete/duplicate/foreign/optional members, checks each selected operand before later execution, then uses shared admitted-slot/composite retention. Tuple and union payloads preserve positional order. Sequence construction meters before each selected element; flatten preserves duplicate occurrence order; fold evaluates source then identity once, owns explicit accumulator/item bindings and uses continuation frames with charged retained projections. Static type equivalence is cached only for validated expression/type edges; dynamic membership remains native and bounded. Shared verified union members and CheckedEquality are retained, including conversions/enum/unit metadata; no label or comparison model is reconstructed. Boolean results now call shared retain_boolean after the selected lazy branch. Scratch reservations guard canonical slot count/bytes and live interpreter addresses; no arbitrary allocator OOM recovery is promised.

Entry dispatch compares all attached canonical association components and range before source entry execution. Input values receive native membership admission. Modular deltas handle widths 16 and 64 without a 64-bit shift; initial event delta is zero. Gap selects on_gap last, with the same input admission. State/baseline/progress commit only after a completed canonical Step; poll does not execute, and watermark does not move the event baseline. Watermark authority/assertion coverage is explicitly driver-owned; nonempty opaque bytes do not independently confer authority. Source invariants terminate subsequent input, ordinary input/resource failures do not. MonitorStep retains canonical Results and immutable Program qualification. invoke creates a fresh native Meter and clears frames, bindings, folds, projections and computed values on both success and failure. No concrete production logic defect was found in the frozen scope.

Evidence read from qualification-logs/titan-ir722/dispatch: whole monitor_program80/0 at9a48f47, supplemental terminal control1/0 and final dispatch6/0 at260cfc3; do not claim a whole81 run. Touched Clippy passes on the archived explicit INTERIM coherent cohort only. RT all-target/all-feature Clippy is supplied on Exact a305/QSV255. Restored committed IT candidate cohort reaches unchanged driverbd140 and fails six adoption errors; it is not green. Real temporal producer lowering/qualification, final own dependency-head long chain, full aggregate, external coverage acceptance and IEEE backend remain unqualified. IEEE evidence is source/compile-only, with no backend replay. FR-276/TC200 full feature acceptance is not credited. QSpec595 event_position and597 canonical atom_projection remain authoritative source-program semantics, not RT mirror statuses.

Plan completion: not assessed

## Verdict

Scoped method verdict: **PASS**. This is not full feature or merge acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: IT-018
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: 'type: integration'
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: // These associations and bodies are diagnostic controls, not a driver
    admission claim.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-009-AC-1
  path: spec/monitor/functional/interface-009-monitor-composite-construction.md
  role: examined
  excerpt: Canonical record and tuple construction preserve declaration order and
    original member/value owners; reversed source member order does not change execution
    order.
- id: interface-009-AC-2
  path: spec/monitor/functional/interface-009-monitor-composite-construction.md
  role: examined
  excerpt: Missing, duplicate, foreign or unknown required members, optional record
    declarations, wrong result shapes and wrong tuple arity refuse before operand
    execution.
- id: interface-009-AC-3
  path: spec/monitor/functional/interface-009-monitor-composite-construction.md
  role: examined
  excerpt: An earlier stopped operand or bounded membership check prevents later execution;
    slot storage and native composite retention denials retain their typed operating
    channels without a partial value or state commit.
- id: interface-010-AC-1
  path: spec/monitor/functional/interface-010-monitor-sequence-operations.md
  role: examined
  excerpt: Construction, exact size and flattening preserve duplicates, occurrence
    order and canonical child ownership; wrong kinds, shapes and lexical slots refuse.
- id: interface-010-AC-2
  path: spec/monitor/functional/interface-010-monitor-sequence-operations.md
  role: examined
  excerpt: A noncommutative left fold has an independent expected result, empty input
    returns its declared identity, lexical captures remain scoped and recursive invocations
    restore outer bindings.
- id: interface-010-AC-3
  path: spec/monitor/functional/interface-010-monitor-sequence-operations.md
  role: examined
  excerpt: Native element/visit/formation/retain charges and explicit work/stack/scratch
    limits stop before later selected work, retain exact typed evidence, expose no
    partial value and leave later invocation state clean.
- id: interface-011-AC-1
  path: spec/monitor/functional/interface-011-monitor-unions-and-equality.md
  role: examined
  excerpt: Verified union construction, full-identity case tests and payload reads
    preserve member/child ownership; foreign, unsealed or malformed bindings refuse
    and an inactive payload read cannot invent a value.
- id: interface-011-AC-2
  path: spec/monitor/functional/interface-011-monitor-unions-and-equality.md
  role: examined
  excerpt: Shared checked equality produces independently expected equal/unequal results
    for selected ordinary values and retains the original conversion/enum/unit semantics
    without a replacement status or comparison model.
- id: interface-011-AC-3
  path: spec/monitor/functional/interface-011-monitor-unions-and-equality.md
  role: examined
  excerpt: Earlier stopped payloads prevent later execution; native retention/equality
    shortages and scratch denial retain exact typed evidence without a partial result
    or state commit.
- id: interface-012-AC-1
  path: spec/monitor/functional/interface-012-monitor-entry-dispatch.md
  role: examined
  excerpt: Changed package/clause/subject/occurrence/clock/counter/unit/width/profile/order
    or invalid required values refuse before entry execution, with state and reading/progress
    baselines unchanged.
- id: interface-012-AC-2
  path: spec/monitor/functional/interface-012-monitor-entry-dispatch.md
  role: examined
  excerpt: Baseline events use delta zero, later admitted readings use the declared
    modular width, and polls or valid inclusive watermarks never move that reading
    baseline; regressing or foreign progress cannot settle.
- id: interface-012-AC-3
  path: spec/monitor/functional/interface-012-monitor-entry-dispatch.md
  role: examined
  excerpt: A stopped event/watermark is unconsumed, preserves all committed state
    and allows same-machine lexical/fold re-entry; a source invariant terminates subsequent
    input and remains distinct from input/resource failures.
- id: interface-012-AC-4
  path: spec/monitor/functional/interface-012-monitor-entry-dispatch.md
  role: examined
  excerpt: A completed step returns original canonical Results with its retained program
    association after preparation drops, preserving foreign-plan qualification without
    a replacement identity or temporal verdict.
bindings: []
```
