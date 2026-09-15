---
id: TM-001
title: "quire-integration complete V1 test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/quire-integration/StR-001
    type: covers
  - target: ix://agent-ix/quire-integration/FR-001
    type: covers
  - target: ix://agent-ix/quire-integration/FR-002
    type: covers
  - target: ix://agent-ix/quire-integration/FR-003
    type: covers
  - target: ix://agent-ix/quire-integration/FR-004
    type: covers
  - target: ix://agent-ix/quire-integration/FR-005
    type: covers
  - target: ix://agent-ix/quire-integration/FR-006
    type: covers
  - target: ix://agent-ix/quire-integration/NFR-001
    type: covers
  - target: ix://agent-ix/quire-integration/NFR-002
    type: covers
---
# quire-integration complete V1 test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | US-001, FR-001, FR-002 | StR-001-VC-1 via TC-002 | 🚧 planned |
| StR-001 | US-002, FR-003 | StR-001-VC-2 via TC-003 | 🚧 planned |
| StR-001 | US-003, FR-004, FR-005 | StR-001-VC-3 via TC-004 and TC-005 | 🚧 planned |
| StR-001 | US-003, FR-006 | StR-001-VC-4 via TC-006 | 🚧 planned |

## User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| US-001 | US-001-EX-1..2 | TC-001, TC-002 | 🚧 planned |
| US-002 | US-002-EX-1..2 | TC-003, TC-009, TC-011 | 🚧 planned |
| US-003 | US-003-EX-1..2 | TC-004, TC-005, TC-006, TC-010 | 🚧 planned |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..5 | TC-001 | 🚧 planned |
| FR-002 | FR-002-AC-1..5 | TC-002 | 🚧 planned |
| FR-003 | FR-003-AC-1..5 | TC-003 | 🚧 planned |
| FR-003 | FR-003-AC-6 | TC-009 | 🚧 planned |
| FR-004 | FR-004-AC-1..5 | TC-004 | 🚧 planned |
| FR-005 | FR-005-AC-1..5 | TC-005 | 🚧 planned |
| FR-006 | FR-006-AC-1..6 | TC-006 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | Generated properties and boundary checks | TC-007 | 🚧 planned |
| NFR-001 | Static conversion/allocation audit | TC-010 | 🚧 planned |
| NFR-002 | Generated authority-confusion properties | TC-008 | 🚧 planned |
| NFR-002 | Static adapter-capability audit | TC-011 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Validate exact ecosystem locks | Property | P0 | FR-001-AC-1..FR-001-AC-5 | 🚧 planned |
| TC-002 | Compose independent domain results | Property | P0 | StR-001-VC-1, FR-002-AC-1..FR-002-AC-5 | 🚧 planned |
| TC-003 | Reconcile authorized persistent effects | Integration | P0 | StR-001-VC-2, FR-003-AC-1..FR-003-AC-5 | 🚧 planned |
| TC-004 | Compose bounded deterministic consumers | Property | P0 | StR-001-VC-3, FR-004-AC-1..FR-004-AC-5 | 🚧 planned |
| TC-005 | Classify compatibility and stale evidence | Property | P0 | StR-001-VC-3, FR-005-AC-1..FR-005-AC-5 | 🚧 planned |
| TC-006 | Qualify every required integration vector | Integration | P0 | StR-001-VC-4, FR-006-AC-1..FR-006-AC-6 | 🚧 planned |
| TC-007 | Bound and reproduce composed execution | Property | P0 | NFR-001-AC-2 | 🚧 planned |
| TC-008 | Preserve source and decision authority | Property | P0 | NFR-002-AC-2 | 🚧 planned |
| TC-009 | Exclude adapter access outside the effect boundary | Static | P0 | FR-003-AC-6 | 🚧 planned |
| TC-010 | Check every untrusted resource conversion | Static | P0 | NFR-001-AC-1 | 🚧 planned |
| TC-011 | Prove adapter capability containment | Static | P0 | NFR-002-AC-1 | 🚧 planned |

## Option Permutation Matrix

| Test Case | Dimension A | Dimension B | Dimension C | Expected Behavior |
|---|---|---|---|---|
| TC-001 | identity domain: repository/package/definition/provider/toolchain/build/deployment/role | pin state: exact/missing/duplicate/stale/dirty/moving | graph state: compatible/incompatible/unknown/cyclic | accept only one complete exact lock and preserve every classification |
| TC-002 | workflow: ConfigVersion/timed-refund/compensated choreography | result: completed/unsupported/incomplete/refused/failed/unexecuted | completion order: canonical/permuted | preserve each requested owner result exactly once in canonical bytes |
| TC-003 | authority: valid/missing/expired/mismatched/over-broad | request: fresh/exact replay/conflicting replay | failure point: before persistence/after persistence/after adapter acceptance | dispatch only persisted authorized requests and reconcile one effect identity |
| TC-004 | consumer: view/renderer/analyzer/codegen | obligation: supported/unsupported/requires-bound/failed | limit: exact/one-over | account for every item without promoting output to semantic authority |
| TC-005 | change: additive/payload break/behavioral-only/absent | graph: resolved/cyclic/unresolved/foreign | vector: available/unavailable/unsupported | retain exact impact paths and explicit evidence states |
| TC-006 | surface: API/CLI/WASM/workflow/provider/backend/intake | vector: passed/failed/unsupported/incomplete/refused/waived/unexecuted | decision: absent/accept/reject/accept-with-limits | complete only the exact supported evidence scope |

## Constraint Boundary Tests

| Constraint | Boundary Type | Test Value | Test Case | Expected |
|---|---|---|---|---|
| lock collection limits | minimum | zero optional and one required member | TC-001 | exact required closure accepted |
| lock collection limits | below minimum | missing required member | TC-001 | refusal before invocation |
| lock collection limits | maximum | exact caller-lowered limit | TC-001 | accepted |
| lock collection limits | above maximum | effective limit + 1 | TC-001 | refusal before allocation or traversal |
| execution result count | exact | requested item count | TC-002 | every item appears once |
| execution result count | duplicate | one duplicated result identity | TC-002 | typed contradiction |
| view traversal/output | maximum | exact effective bound | TC-004 | complete result |
| view traversal/output | above maximum | effective bound + 1 | TC-004 | incomplete with exact frontier |
| dependency traversal | cycle boundary | first repeated identity | TC-005 | finite unknown path, never compatible by default |
| qualification inventory | exact | all 176 rows accounted | TC-006 | scoped rollup may complete |
| qualification inventory | below exact | one required row/vector absent | TC-006 | containing scope incomplete |

## State Transition Tests

| Test Case | Initial State | Event | Expected State |
|---|---|---|---|
| TC-003 | no request | valid authorized submission | pending persisted before dispatch |
| TC-003 | pending without receipt | restart and exact retry | same request reconciled |
| TC-003 | receipt recorded | matching QObs effect observation | effect established separately |
| TC-003 | receipt recorded | foreign or incomplete observation | effect remains unestablished |
| TC-005 | current evidence | meaning-affecting dependency change | evidence stale, not false |
| TC-006 | planned vector | execution result | one immutable terminal result linked to exact inputs |
| TC-006 | incomplete qualification | scoped human waiver | waived evidence remains distinct from passed |

## Edge Cases

| ID | Description | Related Req | Test Case | Risk if Untested |
|---|---|---|---|---|
| EC-001 | One repository contains several components and one component supplies several roles | FR-001 | TC-001 | repository identity substitutes for semantic identity |
| EC-002 | An independent sibling completes after another times out | FR-002, NFR-001 | TC-002, TC-007 | completed evidence is erased or timeout becomes global success/failure |
| EC-003 | Adapter accepted a request immediately before process termination | FR-003 | TC-003 | retry duplicates an external effect |
| EC-004 | Rendered labels and generated syntax imply a relation absent from the source graph | FR-004, NFR-002 | TC-004, TC-008 | presentation becomes semantic authority |
| EC-005 | Equal wire schema hides a changed producer behavior | FR-005 | TC-005 | behavioral regression is classified compatible |
| EC-006 | Evidence intake is unavailable while all producer vectors pass | FR-006 | TC-006 | unavailable intake is silently mocked or promoted |
| EC-007 | A waiver is broader than its actor authority or evidence scope | FR-003, FR-006 | TC-003, TC-006 | human decision enlarges unsupported claims |

## Integration Test Matrix

### Cross-Project Integrations

| Integration ID | Purpose | Target Project | Type | Test Cases | Status |
|---|---|---|---|---|---|
| IT-001 | Compose three reference workflows through real strict readers and domain producers | QSL, State, TL, QObs, Protocol, Verification, backend providers | service | TC-001, TC-002 | 🚧 planned |
| IT-002 | Persist, dispatch, crash, retry, and observe one authorized effect | durable request store, local contract-faithful adapter, QObs | service | TC-003, TC-009, TC-011 | 🚧 planned |
| IT-003 | Render, analyze, generate, and classify bounded semantic consumers | QSL checked/IR, renderer, analyzer, codegen | service | TC-004, TC-005, TC-007, TC-008, TC-010 | 🚧 planned |
| IT-004 | Execute and account for complete-V1 qualification | IT-070..IT-078 producers, artifact store, Quoin intake | service | TC-006, TC-007, TC-008 | 🚧 planned |

### Integration Test Details

| Test Case | Integration | Scenario | Input | Expected | Priority |
|---|---|---|---|---|---|
| TC-002 | IT-001 | Reference workflow and adverse sibling composition | exact lock plus healthy, unsupported, incomplete, failed, and contradictory inputs | canonical bundle preserves every owner result; shared contradiction names affected items | P0 |
| TC-003 | IT-002 | Crash/retry, duplicate delivery, authority refusal, and observation match | persisted history, adapter contract, idempotency keys, decisions, QObs facts | at most one request identity; receipt and business effect remain distinct | P0 |
| TC-004 | IT-003 | Deterministic bounded view and consumer accounting | checked package, profiles, exact/one-over limits, obligations | byte-identical outputs or exact incomplete/unsupported accounting | P0 |
| TC-005 | IT-003 | Compatibility and stale-evidence traversal | baseline/candidate locks and explicit dependency graph | distinct change classes, paths, unknowns, and required vector states | P0 |
| TC-006 | IT-004 | Complete inventory execution and evidence intake | independent corpus, 176-row inventory, exact lock, IT-070..IT-078 vectors | one attributable state per required cell without automatic release promotion | P0 |

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|---|---|---|---|
| GAP-001 | No production implementation or executable test harness exists yet | High | retain every test as planned and implement in C10 through C14 order after review approval |
| GAP-002 | Exact compatible producer revisions remain to be selected and locked | High | C10 lock validation must precede every other implementation test |
| GAP-003 | A production external effect endpoint is intentionally unauthorized | Medium | use a local contract-faithful HTTP service and preserve the limitation in evidence |
| GAP-004 | Quoin evidence intake may be unavailable at qualification time | Medium | record the vector unsupported or incomplete; never mock it to passed |

## Test Execution Summary

| Category | Total | Passed | Failed | Blocked | Coverage |
|---|---|---|---|---|---|
| Property | 6 | 0 | 0 | 6 | 0% executed; 100% criteria mapped |
| Integration | 2 | 0 | 0 | 2 | 0% executed; 100% criteria mapped |
| Static | 3 | 0 | 0 | 3 | 0% executed; 100% criteria mapped |
