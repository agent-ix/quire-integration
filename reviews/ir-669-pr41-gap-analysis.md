---
id: SR-20261102072328
title: "IR-669 PR 41 structural identity fixture gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; tests/monitor_program.rs; tests/producer_origin_identity.rs; Cargo.lock"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-integration/IT-016
    type: references
  - target: ix://agent-ix/quire-integration/IT-018
    type: references
---

## Summary

Ticket: IR-669. Planless trace and evidence review of the changed test fixtures. Quoin's computed matrix reports no coverage-matrix criteria because the installed module declares no obligations source; the affected trace tags and their assertions were checked manually.

## Verdict

**CONDITIONAL for the PR diff** — no new trace binding gap in the structural-key repair. The lockfile breaks compilation of other test targets, recorded as code-review FND-001; these test results cannot be claimed on this head. This does not claim repository-wide matrix coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. Examined IT-016-SC-01, IT-016-SC-02, IT-016-SC-05, IT-018-SC-01, and IT-018-SC-13, and IT-018-SC-15 against the existing `#[trace]` bindings and affected test symbols. Both changed fixtures retain live QSL emission and strict IR read; declarations and body references resolve through the admitted package before construction of runtime keys. The corrected tests still assert semantic form, member ownership, typed refusals, order, and exact runtime values. The computed matrix had no criteria, so this is a scoped manual trace review, not a repository-wide matrix verdict. The relocked constructor break in unchanged test targets limits runnable evidence outside the focused probes. Focused retained-program and enum-identity probes and the full 86-test monitor-program target passed at this frozen head; the lead owns the final aggregate gate.

## Current status

The constructor migration in the five affected test files closes the cross-target compilation concern noted above. All twelve calls now use the locked QSL two-argument form. No new trace binding or test-evidence gap was found in that incremental diff. Final aggregate gates remain for the lead.
