---
id: SR-2026110414560065
title: IR-1346 integration code-review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1346 AC4; Cargo.lock; tests/generated_oracle_family_function_equality.rs; TC-104; IT-019 SC-05"
review_set: subset
---

## Summary

Reviewed quire-integration app version 0.1.0, PR #63, owning ticket IR-1346 AC4 and umbrella IR-842. Scope is the four changed files: Cargo.lock, the primitive composite test, TC-104 and IT-019 SC-05 Integer rows. No applicable AssuranceProfile exists under spec. All added lines and contextual checking, selection, observation and native-execution helpers were inspected.

The admitted source is authentic QSL emission read through IR; inline Integer Literal.type_id is checked against its actual scalar descriptor and both consumer descriptors. Fourteen original closed bodies (eight Boolean and six Integer) must generate. The six Integer cells cover both operators and equal/less/greater nonnegative pairs. Fresh independent QSL checking evaluates the same original body at work 1024, 0 and 1. Independent Boolean truths and exact WorkUnits/EqualityPlanForm stops constrain the authority. Full typed Outcome, losses, ordered charges and every LimitKind counter flow through the existing mechanical snapshot. Returned environment/oracle symbols execute raw CG artifacts. No copied evaluator, fabricated type environment, IR rewrite, meter subtraction or precharge was introduced. The wrong actual Boolean descriptor requires position 0, expected Boolean and found Some(actual Integer).

Cargo.toml remains unchanged with ordinary branch-main dependencies. The lock refresh has one coherent QSL source and normal CG/IR updates; no source pin or new dependency was added. Existing FR-021 78-envelope and protocol six-operation cohorts are unchanged context, not refreshed execution proof. QSL-1736 and QSL-1739 and other remaining SC-05 families remain Planned.

Execution qualification is BLOCKED. Inspected author formatter and targeted Quire logs; the coherent Clippy log fails in upstream CG replay conversion on three new CallSiteRefusal variants before this test compiles. No local workaround is requested. Native execution has not run; 42 is required, never observed evidence at this head. Historical Boolean evidence is not refreshed-head proof. AC4 has not passed. The draft and both nonclosing Refs remain appropriate; umbrellas stay open. The lead owns the final scoped gate. This reviewer ran no build, Cargo, Clippy, native test, Kani, global matrix, advisor or aggregate gate.

## Verdict

CONDITIONAL for static scoped code-review. One medium assertion gap requires correction before review acceptance. Runtime qualification remains BLOCKED, and this is no AC4 or merge-readiness verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Changed-descriptor control checks only Generated and does not verify that the healthy sibling binding remains unchanged | tests/generated_oracle_family_function_equality.rs:600 |

A generator regression could alter the healthy sibling oracle/environment symbols while still returning Generated; this control would pass, because subsequent native execution uses the original output instead of the changed-descriptor output. Compare the unchanged sibling claim against its original claim (including its binding), following the existing exact-function sibling-control pattern.

## Coverage

Base checklist, relevant Rust implementation/testing rules, duplication, ownership, assertions, trace binding and honest status reporting were inspected. No requirement grammar, relationship, domain model or structure changed; additional spec lenses are inapplicable. No optional semantic analysis or enforced workflow was run.

- TC-104-step-6 (examined), spec/test-cases/TC-104-generated-function-equality-agreement.md: Change only one
   Integer consumer descriptor to an actual admitted Boolean type and require
   OperandTypeMismatch at position 0, expected Boolean and found original Integer,
   with a healthy generated sibling and no execution credit for the refusal.
- TC-104-Expected-Results (examined), spec/test-cases/TC-104-generated-function-equality-agreement.md: requires 42 FR-018 operation observations: 24 Boolean truth-table observations
and 18 closed nonnegative Integer observations, covering Equal/NotEqual over
equal, less and greater pairs at work 0, 1 and 1024.
- IT-019-SC-05 (examined), spec/integration/IT-019-generated-oracle-qsl-agreement.md: Compare
   every public operation outcome, loss, ordered admitted charge and consumed
   counter without subtraction or precharging.
- IT-019-Integer-status (examined), spec/integration/IT-019-generated-oracle-qsl-agreement.md: Pending the CG main replay refusal conversion fix and native execution
- IR-1346-AC4 (examined), IR-1346: AC4: IR-842 SC-05 FR-018 Integer rows move from non-executed to executed in a follow-up integration slice.
- FR-021-retained-cohort (context_only), spec/test-cases/TC-104-generated-function-equality-agreement.md: passes 78 whole-call comparisons, 32 input refusals and ten changed-owner refusals.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The added assert_eq compares the complete healthy sibling claim against the original claim, including its returned environment and oracle symbols and source binding. The Generated guard is retained. |

### Disposition round 1

Static scoped review PASS after correction. Inspection of the public Eq/PartialEq derives confirms the complete claim comparison includes the Generated payload and its environment/oracle symbols, descriptor and source correspondence. No new finding was introduced by the four added test lines. Original findings remain unchanged above.

Execution qualification remains BLOCKED by the known upstream CG compilation error. No native comparisons were rerun; 42 remains the required observation count, not observed evidence. AC4 and the lead's final scoped gate remain pending. No build, Cargo, native, Clippy, Kani or gate command was run by this reviewer.

Fixed assertion:

```rust
assert_eq!(
        refused.claim_map.items[9], output.claim_map.items[9],
        "changed Integer descriptor preserves the complete healthy sibling claim and binding"
    );
```

## Current source and author execution addendum

PASS for the scoped code/Rust, manual gap and base specification reviews of quire-integration app version 0.1.0, PR #63. No new findings. This addendum supersedes the earlier execution-BLOCKED assessment and the earlier required-only count; those paragraphs describe earlier review rounds. FND-001 remains fixed, and its original finding and disposition are preserved.

The normal branch-main CG dependency refresh incorporates the upstream replay conversion correction. The lock delta changes only the CG source selection; manifests remain unchanged. QSL, IR and CG each resolve to one coherent source selection, with no source pin, local adapter or upstream source edit.

The changed-descriptor control now selects exactly one claim by its original source node identity, rather than assuming descriptor-key claim-map ordering equals authored row ordering. The singleton pattern rejects missing or duplicate matches. It selects the original Integer cell and healthy sibling independently in each output. The refusal still requires position 0, expected actual admitted Boolean ID and found Some(actual admitted Integer ID). The healthy Generated guard and complete claim equality with the original identity-selected output remain, preserving environment/oracle symbols, descriptors and source binding. The main observation loop independently selects each generated claim by original node identity. No change replaces or weakens the full outcome/accounting assertions.

Author evidence inspected: focused Clippy and host compilation finish successfully. The primitive native test passes one test, zero failed, zero ignored: all fourteen generated items run three allowances and the explicit count plus exact output comparison require 42 observations, comprising 18 Integer and 24 Boolean comparisons. Those assertions preserve full typed Outcome, losses, ordered admitted charges and every LimitKind counter, with independent Boolean truth and exact stops. The retained FR-021 native test passes one test, zero failed, zero ignored, requiring its 78 whole-call envelopes, 32 input refusals, ten owner refusals and generation controls. The retained protocol native test passes one test, zero failed, zero ignored and requires six operation comparisons. Initial failed logs are historical diagnostics and supply no success credit. Generated FR-021 compilation emits existing unexpected cfg(kani) warnings; success is not claimed warning-free. Author targeted Quire and formatting output contains advisory/module and stable-channel formatter warnings rather than failures; author reports both checks successful.

Only the actually executed closed Integer rows change to Passed. Parameterized Integer operations remain Planned under QSL-1736; constructed nominal/collection and remaining operation-ledger rows remain Planned under QSL-1739, along with the other open SC-05 work. The PR remains draft and includes nonclosing Refs IR-842 and Refs IR-1346. These observations support the narrow AC4 executed-Integer outcome as author execution evidence, not full SC-05 closure or merge readiness. Both umbrellas stay open. The final scoped gate at the final artifact-bearing head remains pending with the lead.

This reviewer inspected source, dependency and status deltas and author logs only. No build, Cargo, native test, Clippy, Kani, global matrix, advisor or gate command was run. Manual gap analysis remains deliberately scoped; Plan completion: not assessed.

### Addendum examined units

- source-identity-claim-selection (examined), tests/generated_oracle_family_function_equality.rs: filter(|claim| claim.node_id == nodes[index])
- TC-104-Expected-Results (examined), spec/test-cases/TC-104-generated-function-equality-agreement.md: passes 42 FR-018 operation observations: 24 Boolean truth-table observations
and 18 closed nonnegative Integer observations, covering Equal/NotEqual over
equal, less and greater pairs at work 0, 1 and 1024.
- IT-019-Integer-status (examined), spec/integration/IT-019-generated-oracle-qsl-agreement.md: FR-018 Equal and NotEqual / closed Integer literals | Passed
- remaining-operation-status (context_only), spec/test-cases/TC-104-generated-function-equality-agreement.md: FR-018 parameterized Integer operations | Planned
