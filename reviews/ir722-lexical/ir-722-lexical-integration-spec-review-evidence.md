---
id: SR-20261102072244
title: "IR-722 lexical quire-integration spec-review/evidence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-integration; IR-722; frozen lexical amendment; interface_004, IT-018-SC-06, monitor source and companion tests"
review_set: subset
---

## Summary

quoin advise ran against both frozen snapshots. The four lexical interface criteria are catalog-inconclusive; executable Test controls are selected by reviewer judgment because these public observations are concrete. AC-4 remains partially evidenced as recorded; source inspection is not counted as the missing public control.

Frozen lexical amendment of the original IR-722 PRs: RT193 2ec49a0c372a418b67d78388a098ce6623fde2c6 over 14b1767, integration35 742bff7417163bcf19d93398acabf1ec0d088520 over 31c384a. Superseded PR195/36 history is context only. All previous SR artifacts are retained untouched by this pass. Read actual frozen Git snapshots, applicable repository guidance, code-review/Rust style and selected scoped spec methods. No source edits, Cargo, proof/heavy jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

Public shape changes are LocalId, Operation::Local/Let and typed InvalidBinding/Binding failure variants. ProgramParts/Layout/Function/Expression fields remain unchanged. Bindings::prepare derives a compact private address index in linear construction work; unique in-range positions among N declarations imply dense positions without host-size semantic caps. Validation propagates function ownership and rejects mixed/foreign binding closures. Local type facts derive from the retained initializer annotation; shared bounded admission handles a different Local result annotation and Let result edges. No mirrored Value/source-key type, synthetic function, callback, parser or new plan wire appears.

VM dataflow was traced: bound operands are canonical literal/supplied addresses; invocation tokens derive from already charged monotonic work, separating recursive invocations. Restore runs after the scoped body and before Let conformance, restoring caller activation and decrementing the active count. The active count includes shadowed operands saved in Restore frames and is included in slot/native reservation accounting. Touched flags cap cleanup to actually activated unique slots; both normal and stopped run results execute cleanup. These source traces are evidence of implementation structure, not substitutes for the missing AC-4 controls.

Supplied evidence: seven lexical controls and all twenty-nine monitor_program tests pass, and touched integration Clippy passes ONLY with the preserved interim old-leaf lock and current RT source. RT all-target/all-feature Clippy passes on Exact a3052598 / QSV 25535387. The committed new integration cohort remains blocked by counterpart driver adoption/CG707 integration; the requested final IR long chain has not run. Root IEEE owner evidence is source/compile-only, not CBMC replay. Prior coverage/cross-repository acceptance and final aggregate remain unresolved. Ordinary constructor/arithmetic/sequence/fold operations and event/watermark/gap execution remain unfinished outside this scoped lexical claim; no full FR-276 or temporal producer qualification is granted.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — Findings require an explicit fix or scope disposition; committed-cohort/aggregate blockers remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-4 remains only partly controlled: the deep-chain scratch test counts distinct active bindings, but no tight-budget recursive-shadow control distinguishes counting the retained caller operand from counting only occupied slots. Each stopped/successful build also creates a fresh Machine, so cleanup before a subsequent invocation of the same Machine is not publicly exercised. Keep these subclauses open until appropriate controls exist; no public API widening or fake temporal acceptance is requested. | tests/monitor_program.rs:1910-1960 |

## Examined scope

```yaml
scope:
- id: IT-018-SC-06
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "Execute lexical diagnostics. IT-018-SC-06: compact function-qualified\
    \ slots\n   support scoped reads, exact continuation counts and lazy branches."
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: The controlled executable parts exercise construction, not temporal producer/executor
    qualification.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-004-AC-1
  path: spec/monitor/functional/interface-004-monitor-lexical-bindings.md
  role: examined
  excerpt: Missing, duplicate, sparse and foreign local declarations refuse construction,
    while compact function-qualified bindings retain their original source locations.
- id: interface-004-AC-2
  path: spec/monitor/functional/interface-004-monitor-lexical-bindings.md
  role: examined
  excerpt: A selected Let evaluates its value once and returns its scoped body's result;
    a lazy unselected binding is not activated. A differently annotated local uses
    shared admission and preserves its native failure.
- id: interface-004-AC-3
  path: spec/monitor/functional/interface-004-monitor-lexical-bindings.md
  role: examined
  excerpt: Nested calls and recursive invocations cannot read an outer invocation's
    local as their own. A bind restores the prior invocation's slot on return, and
    an inactive read is a typed invariant failure.
- id: interface-004-AC-4
  path: spec/monitor/functional/interface-004-monitor-lexical-bindings.md
  role: examined
  excerpt: Binding operands respect the scratch ceiling, including a shadowed outer
    operand. A stopped invocation clears only touched slots and cannot leak a binding
    into a subsequent invocation; deep binding chains use the explicit stack.
bindings:
- test_id: lexical_value_is_evaluated_once_within_its_scoped_body
  ac_id: interface-004-AC-2
  trace: correct
- test_id: lexical_missing_duplicate_sparse_and_foreign_declarations_refuse
  ac_id: interface-004-AC-1
  trace: correct
- test_id: lexical_unselected_branch_does_not_activate_a_local
  ac_id: interface-004-AC-2
  trace: correct
- test_id: lexical_unselected_branch_does_not_activate_a_local
  ac_id: interface-004-AC-3
  trace: correct
- test_id: lexical_different_annotation_preserves_native_membership_failure
  ac_id: interface-004-AC-2
  trace: correct
- test_id: lexical_recursive_return_restores_the_outer_binding
  ac_id: interface-004-AC-3
  trace: correct
- test_id: lexical_recursive_invocation_cannot_capture_a_callers_active_local
  ac_id: interface-004-AC-3
  trace: correct
- test_id: lexical_deep_chain_charges_live_bindings_and_uses_explicit_stack
  ac_id: interface-004-AC-4
  trace: correct
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The retained-caller scratch portion is now controlled: four slots stop at spent 4/request 1, five slots succeed with outer now=99 on the preserved interim cohort. The remaining same-Machine subsequent-invocation cleanup subclause still awaits entry-dispatch controls on open IR-722. Full interface-004-AC-4 remains open and is not narrowed. |

Lexical disposition carry inspected current RT af4c3d8effd378cac9cd953b01404aee6f118981 and integration 7fcff282ce80a05527bcd4b3493837a61f74f12d. The new public lexical_recursive_shadow_counts_the_retained_caller_operand control distinguishes a four-slot refusal from a five-slot success and checks restored outer now=99; it closes the caller-shadow accounting portion of the original finding. It builds a fresh Machine for each attempt and therefore does not establish reuse of one stopped Machine. That remainder stays on the existing IR-722 entry-dispatch work; no API widening, acceptance narrowing or full AC-4 credit is granted.

The interface-004 preparation prose is corrected from visits-once to linear preparation, matching its fixed passes. No new lexical runtime defect found. Evidence remains interim-cohort only for integration; current RT Clippy passes on new leaves, but committed integration cohort, our final dependency-head long chain, QSL1463/2106 producer qualification, coverage acceptance and IEEE replay remain unresolved. No Cargo/heavy jobs or source edits by reviewer.

Current verdict remains **CONDITIONAL**: only the same-Machine cleanup remainder is deferred; component/cohort qualification is not complete.

```yaml
dispositions:
- fnd: FND-001
  outcome: deferred
  reason: 'The retained-caller scratch portion is now controlled: four slots stop
    at spent 4/request 1, five slots succeed with outer now=99 on the preserved interim
    cohort. The remaining same-Machine subsequent-invocation cleanup subclause still
    awaits entry-dispatch controls on open IR-722. Full interface-004-AC-4 remains
    open and is not narrowed.'
  after_excerpt: "assert_eq!((stop.limit, stop.spent, stop.requested), (Some(4), 4,\
    \ 1));\n    let monitor = build(BuildPlan {\n        program,\n        budget:\
    \ StepBudget {\n            scratch_elements: 5,\n            ..execution_budget()\n\
    \        },\n    })\n    .unwrap();\n    assert_now(&monitor, 99);"
```

## Dispatch carry disposition

Previously verified tight recursive shadow accounting plus the frozen260cfc3 recursive event stop/retry now directly qualify both original lexical evidence subclauses. Same monitor/machine and budget survive Work stop, retry commits baseline700 with delta0, then703 gives delta3. Broader producer/current-cohort/coverage acceptance stays open.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Previously verified tight recursive shadow accounting plus the frozen260cfc3 recursive event stop/retry now directly qualify both original lexical evidence subclauses. Same monitor/machine and budget survive Work stop, retry commits baseline700 with delta0, then703 gives delta3. Broader producer/current-cohort/coverage acceptance stays open. |

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: // Neither monitor nor machine nor limits is replaced between the
    stop and retry.
```
