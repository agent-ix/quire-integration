---
id: Plan-001
title: "quire-integration — complete V1 composed integration"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-integration/StR-001
    type: references
  - target: ix://agent-ix/quire-integration/FR-001
    type: references
  - target: ix://agent-ix/quire-integration/FR-002
    type: references
  - target: ix://agent-ix/quire-integration/FR-003
    type: references
  - target: ix://agent-ix/quire-integration/FR-004
    type: references
  - target: ix://agent-ix/quire-integration/FR-005
    type: references
  - target: ix://agent-ix/quire-integration/FR-006
    type: references
  - target: ix://agent-ix/quire-integration/NFR-001
    type: references
  - target: ix://agent-ix/quire-integration/NFR-002
    type: references
---
# Implementation Plan: complete V1 composed integration

This plan turns the reviewed C10–C14 specification into a Rust coordination runtime and
qualification harness. The repository consumes exact owner artifacts and preserves every
domain disposition; it does not reimplement producer semantics, manufacture authority or
make a release decision.

## Requirements Summary

### Stakeholder Requirements

- [ ] **StR-001**: keep composed execution reproducible, authority-qualified and attributable.

### Functional Requirements

- [x] **FR-001**: validate one exact canonical ecosystem lock before invocation.
- [x] **FR-002**: compose every requested independent domain result exactly once.
- [x] **FR-003**: persist, dispatch and reconcile authorized idempotent effect requests.
- [ ] **FR-004**: compose bounded deterministic views and consumer accounting.
- [ ] **FR-005**: classify compatibility impact and stale evidence independently.
- [ ] **FR-006**: record every required complete-V1 qualification cell and vector.

### Non-Functional Requirements

- [ ] **NFR-001**: reproduce all outputs under finite preselected resource budgets.
- [ ] **NFR-002**: contain semantic authority and external-effect capabilities by type.

## Dependency Graph

### Core dependency edges

- `NFR-001 + NFR-002 -> FR-001`: canonical limits and deny-by-default capability types are
  part of the validated lock boundary, not later hardening.
- `FR-001 -> FR-002, FR-003, FR-004, FR-005`: every coordinator consumes one validated exact
  lock and may not select ambient implementations.
- `FR-002 + FR-003 + FR-004 + FR-005 -> FR-006`: qualification can account for a complete
  system only after every independent execution/effect/consumer/impact path emits evidence.
- `FR-006 -> StR-001`: the stakeholder claim is demonstrated only by IT-001 through IT-004
  under the same lock and inventory.

### Shared dependencies

- Stable identity newtypes, canonical encoding, checked limits, closed dispositions and
  constructor-private validated views are shared infrastructure and must have one owner.
- Durable effect state is deliberately separate from the pure in-memory producer and
  consumer coordination surfaces.
- The explicit typed dependency traversal used by views and compatibility shares bounded
  mechanics but retains distinct result semantics.

### Cross-cutting constraints

- `NFR-001` applies before allocation, traversal, dispatch, persistence, arithmetic and
  output retention in every module.
- `NFR-002` applies to public constructors, internal trait objects, adapter dispatch and
  every test/qualification execution path.

### The seams

`src/lib.rs` becomes exports only. Shared identity/limit/encoding primitives sit below
lock, execution, effect, consumer, impact and qualification modules. Owner producers enter
through typed capability traits backed by exact lock entries. Persistence and HTTP adapter
traits are exercised by real local implementations in integration tests; no evaluator can
hold either adapter capability.

## Test Plan

### Unit and property tests

- [x] **TC-001**: canonical lock identity, pin mutation, domain separation, classifications
  and pre-invocation refusals for FR-001-AC-1 through FR-001-AC-5.
- [x] **TC-002**: exact result multiplicity, sibling dispositions, completion-order
  metamorphisms, shared contradictions and unsupported producers for FR-002-AC-1..5.
- [ ] **TC-003**: persist-before-dispatch, crash/reconcile, authority abuse, key conflict and
  receipt/effect separation for FR-003-AC-1 through FR-003-AC-5.
- [ ] **TC-004**: deterministic bounded views, identity maps, exact frontiers and consumer
  accounting for FR-004-AC-1 through FR-004-AC-5.
- [ ] **TC-005**: four impact classes, explicit paths, cycles/unknowns, stale evidence and
  regression-vector selection for FR-005-AC-1 through FR-005-AC-5.
- [ ] **TC-006**: 176-cell inventory accounting, no-promotion rollups, reproducible run
  identity, evidence intake and scoped decisions for FR-006-AC-1 through FR-006-AC-6.

### Cross-cutting verification

- [ ] **TC-007**: generated exact/one-over budgets, timeouts, sibling retention and canonical
  equality for NFR-001-AC-2.
- [ ] **TC-008**: generated authority-confusion attempts and source/decision separation for
  NFR-002-AC-2.
- [ ] **TC-009**: static proof that adapter invocation exists only in the effect boundary for
  FR-003-AC-6.
- [ ] **TC-010**: static inventory of checked untrusted conversions and allocations for
  NFR-001-AC-1.
- [ ] **TC-011**: static call-graph proof that every dispatch path crosses authority and
  persisted-request checks for NFR-002-AC-1.

### Integration tests

- [ ] **IT-001**: compose ConfigVersion, timed-refund and compensated choreography through
  pinned real strict readers and producers.
- [ ] **IT-002**: persist, dispatch, terminate, reconcile and observe one authorized effect
  through a real local store and contract-faithful HTTP adapter.
- [ ] **IT-003**: run real bounded query, renderer, analyzer and codegen consumers plus impact
  classification over isolated filesystem output.
- [ ] **IT-004**: execute/account for IT-070 through IT-078 and evidence intake across the
  reviewed 176-row inventory.

## Remaining Work

### Track S: Shared critical path

- **S1 = Task-001** common contracts and safety envelope — Hard; exit: every public path uses
  domain-safe identities, closed states, canonical bytes and checked limits.
- **S2 = Task-002** exact ecosystem lock — Hard; exit: no producer can run from an incomplete,
  moving, cross-domain or merely reachable selection.

### Tracks A–D: Independent post-lock features

- **A1 = Task-003** execution bundles — Hard; exit: all requested owner results survive
  canonical composition with exact item-local state.
- **B1 = Task-004** authorized effect reconciliation — Hard; exit: crash/retry cannot issue an
  unsafe second call or turn a receipt into business-effect truth.
- **C1 = Task-005** bounded consumer views — Hard; exit: every consumer item and omitted
  frontier is source-attributed without semantic promotion.
- **D1 = Task-006** compatibility impact — Hard; exit: impact paths, unknowns and stale
  evidence remain independently classified.

### Track G: Cross-cutting gate

- **Gate = Task-007** resource and authority verification — Hard; measures exact/one-over
  containment plus call-graph boundaries; pass: TC-007 through TC-011 bind all four NFR
  criteria and FR-003-AC-6 with zero unauthorized path.

### Tracks I1–I3: Real integration surfaces

- **I1 = Task-008** reference workflow composition — Hard; exit: three pinned workflows
  compose reproducibly through real producers.
- **I2 = Task-009** durable effect integration — Hard; exit: termination at both crash windows
  reconciles or remains `delivery-unknown` without duplication.
- **I3 = Task-010** bounded consumer integration — Hard; exit: real renderer/analyzer/codegen
  output and compatibility evidence remain deterministic and attributable.

### Track Q: Qualification and completion

- **Q1 = Task-011** qualification ledger and decision boundary — Hard; exit: each inventory
  cell has exactly one immutable state and no non-pass state completes a scope.
- **Q2 = Task-012** complete-V1 campaign — Hard; exit: every applicable IT-070..IT-078 vector
  is executed or explicitly unavailable under one lock.
- **Gate = Task-013** completion and promotion readiness — Medium; measures strict validation,
  trace coverage and repository gates; pass: 51/51 rows backed, zero status lies and no open
  high/medium review finding.

## Parallel Execution Summary

```text
Task-001 -> Task-002 -> Task-003 -> Task-008 --\
                    \-> Task-004 -> Task-009 ---+
                    \-> Task-005 --------------+-> Task-010 --\
                    \-> Task-006 ----------------------------+-> Task-011 -> Task-012 -> Task-013
                              \-> Task-007 -------------------/
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-001 | S | NFR-001, NFR-002 | TC-007..TC-011 | done |
| Task-002 | S | FR-001 | TC-001 | done |
| Task-003 | A | FR-002 | TC-002 | done |
| Task-004 | B | FR-003 | TC-003, TC-009, TC-011 | done |
| Task-005 | C | FR-004 | TC-004 | not_started |
| Task-006 | D | FR-005 | TC-005 | not_started |
| Task-007 | G | NFR-001, NFR-002 | TC-007..TC-011 | not_started |
| Task-008 | I1 | FR-001, FR-002 | IT-001 | not_started |
| Task-009 | I2 | FR-003 | IT-002 | not_started |
| Task-010 | I3 | FR-004, FR-005 | IT-003 | not_started |
| Task-011 | Q | FR-006 | TC-006 | not_started |
| Task-012 | Q | StR-001, FR-006 | IT-004 | not_started |
| Task-013 | Gate | StR-001, FR-001..FR-006, NFR-001..NFR-002 | TC-001..TC-011, IT-001..IT-004 | not_started |

## Coordination Rules

- Freeze common identity, disposition, limit and canonical-encoding types before parallel
  feature work; changes after Task-002 require every track to rebase and rerun properties.
- Use separate feature modules and test files for Tasks 003–006; `lib.rs` and shared modules
  have a single writer.
- Keep durable-store and HTTP adapter dependencies out of pure producer/consumer modules.
- Record exact upstream revisions and rights before importing any fixture or schema bytes.
- Do not mark matrix rows complete until executable trace tags bind the exact criteria.
- Evidence intake unavailability remains unsupported/incomplete; no mock may turn it passed.
