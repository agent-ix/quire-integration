---
id: SR-20261102072245
title: "IR-722 lexical quire-integration spec-review/scope-boundary"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-integration; IR-722; frozen lexical amendment; interface_004, IT-018-SC-06, monitor source and companion tests"
review_set: subset
---

## Summary

RT owns lexical addresses, activation lifetime and typed faults; existing QSV equality/membership owns conformance, and IR/QSL owns source admission/semantic meaning. No source identity is created by LocalId. Real producer lowering and temporal execution remain separate.

Frozen lexical amendment of the original IR-722 PRs: RT193 2ec49a0c372a418b67d78388a098ce6623fde2c6 over 14b1767, integration35 742bff7417163bcf19d93398acabf1ec0d088520 over 31c384a. Superseded PR195/36 history is context only. All previous SR artifacts are retained untouched by this pass. Read actual frozen Git snapshots, applicable repository guidance, code-review/Rust style and selected scoped spec methods. No source edits, Cargo, proof/heavy jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

Public shape changes are LocalId, Operation::Local/Let and typed InvalidBinding/Binding failure variants. ProgramParts/Layout/Function/Expression fields remain unchanged. Bindings::prepare derives a compact private address index in linear construction work; unique in-range positions among N declarations imply dense positions without host-size semantic caps. Validation propagates function ownership and rejects mixed/foreign binding closures. Local type facts derive from the retained initializer annotation; shared bounded admission handles a different Local result annotation and Let result edges. No mirrored Value/source-key type, synthetic function, callback, parser or new plan wire appears.

VM dataflow was traced: bound operands are canonical literal/supplied addresses; invocation tokens derive from already charged monotonic work, separating recursive invocations. Restore runs after the scoped body and before Let conformance, restoring caller activation and decrementing the active count. The active count includes shadowed operands saved in Restore frames and is included in slot/native reservation accounting. Touched flags cap cleanup to actually activated unique slots; both normal and stopped run results execute cleanup. These source traces are evidence of implementation structure, not substitutes for the missing AC-4 controls.

Supplied evidence: seven lexical controls and all twenty-nine monitor_program tests pass, and touched integration Clippy passes ONLY with the preserved interim old-leaf lock and current RT source. RT all-target/all-feature Clippy passes on Exact a3052598 / QSV 25535387. The committed new integration cohort remains blocked by counterpart driver adoption/CG707 integration; the requested final IR long chain has not run. Root IEEE owner evidence is source/compile-only, not CBMC replay. Prior coverage/cross-repository acceptance and final aggregate remain unresolved. Ordinary constructor/arithmetic/sequence/fold operations and event/watermark/gap execution remain unfinished outside this scoped lexical claim; no full FR-276 or temporal producer qualification is granted.

Plan completion: not assessed

## Verdict

**PASS** — No additional defect in this method; other findings and qualification blockers remain binding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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
bindings: []
```
