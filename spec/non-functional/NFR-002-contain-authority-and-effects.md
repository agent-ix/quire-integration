---
id: NFR-002
title: "Contain semantic authority and external effects"
type: NFR
quality_attribute: security
relationships:
  - target: ix://agent-ix/quire-integration/FR-002
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-003
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-004
    type: constrains
  - target: ix://agent-ix/quire-integration/FR-006
    type: constrains
---
# NFR-002: Contain semantic authority and external effects

## Statement

The library SHALL keep producer evaluation, effect authorization, adapter
dispatch, rendering, analysis, code generation, evidence intake, and human
decision authority in separately typed capability boundaries with deny-by-default
invocation.

## Scope

Applies to every public API, internal dispatch edge, provider manifest, and test
or qualification execution path in this repository.

## Rationale

An untyped or ambient capability edge could allow a pure evaluator to cause an
external effect or allow a non-authoritative consumer to change semantics.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Non-effect components with adapter-dispatch capability | 0 components | 0 | Static Analysis (TC-008) |
| Ambient/default provider lookups | 0 lookups | 0 | Static Analysis (TC-008) |
| Unsafe Rust blocks | 0 blocks | 0 | Static Analysis (TC-008) |
| Unauthorized dispatches across adverse fixtures | 0 dispatches | 0 | Test (TC-008) |

## Verification

The call graph, public surface, feature graph, provider registry, and unsafe audit
are checked statically. Adverse tests attempt direct, missing-authority,
cross-scope, expired, and key-conflicting dispatch.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-002-AC-1 | Every adapter dispatch is reachable only through the FR-003 authority and persisted-request checks. | Static Analysis (TC-011) |
| NFR-002-AC-2 | Renderer, analyzer, codegen, evidence, and human-decision inputs cannot be consumed as source or evaluator authority. | Property Test (TC-008) |
