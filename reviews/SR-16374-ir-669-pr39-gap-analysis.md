---
id: SR-16374
title: "IR-669 PR 39 U-row gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock, tests/quantity_source_agreement.rs, tests/scalar_agreement_quantity.rs; IT-017-SC-01 through IT-017-SC-05 and QSpec TC-187 U01 through U39"
review_set: subset
---

## Summary

Ticket: IR-669. Planless changed-test trace and evidence audit of draft PR #39 at ccad0b4841a63c2159865c0f05d6f484a72f5226. The carried conversion rows are executable in source form, but trace tags claim broader criterion coverage than the tests establish.

## Verdict

PASS after round-1 dispositions for the reviewed trace and evidence gaps. Compilation and the final gate remain blocked upstream by IR-1284.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The changed U test files tag IT-017-SC-01 while dispatching only selected QSpec IDs. Missing runnable U IDs and future QSpec IDs do not fail, so the trace can mark the family criterion as covered without its required exhaustive execution or upstream-ticket PENDING route. | tests/scalar_agreement_quantity.rs:370; tests/quantity_source_agreement.rs:36 |
| FND-002 | medium | U01 is tagged to SC-03 without any retained meter observation, and U16/U17/U18/U29 check QSL/runtime meter agreement but not QSpec's named stop prefix. A wrong common charge sequence can therefore satisfy the current accounting evidence. | tests/quantity_source_agreement.rs:36-173; tests/scalar_agreement_quantity.rs:555-700 |

## Coverage

Plan completion: not assessed. `quoin matrix --repo /home/peter/dev/worktrees/ir669-f03-source-agreement --json` returned no coverage_matrix criteria because the module declares no `obligations:` source. I used a manual trace scan of IT-017-SC-01 through SC-05, QSpec TC-187 U01 through U39, and every changed `#[trace]` test. The six carried cases have real checked-QSL and public-runtime paths; U13/U21 power and U39 negation remain ticket-backed upstream work, while other integration-owned U rows remain unimplemented under IR-669. No reverse ownership gap, stub, ignored test, copied fixture, or vendored corpus was found in the changed files. Optional semantic review was not separately requested. Compilation is blocked before the U tests by IR-1284.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Partial U tests no longer trace family-wide SC-01; unimplemented integration-owned U rows remain open under IR-669. |
| FND-002 | fixed | U01's SC-03 trace was removed, and the U16/U17/U18/U29 cases assert authority charge prefixes and relevant counters. |
