---
id: SR-16370
title: "IR-669 PR 37 F-row gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock, tests/ieee_source_agreement.rs, tests/scalar_agreement_ieee_text.rs; IT-017-SC-01 through IT-017-SC-05, TC-097"
review_set: subset
---

## Summary

Ticket: IR-669. Planless changed-test trace and evidence audit for draft PR #37. The QSpec F-row API is referenced by the tests, but the reviewed family does not yet close every runnable row or make an unknown row fail.

## Verdict

FAIL. The missing F-row inventory gate allows coverage inflation even if every named test passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | No exhaustive TC-193 vector dispatch maps every published F ID to a runnable test or an exact ticket-backed PENDING cause; unknown and omitted runnable rows are invisible to the suite. | tests/ieee_source_agreement.rs:34-47 |
| FND-002 | medium | The new F10 and F17 tests do not verify all named limit and denial fields in TC-193; F13 lacks its operation-local flag sequence, and F20 omits multiple admission-only subcases. These trace-tagged tests therefore do not yet establish all claimed SC-03/SC-04 evidence. | tests/ieee_source_agreement.rs:717-784 |

## Coverage

Plan completion: not assessed. `quoin matrix --repo ... --json` returned no coverage_matrix criteria because the installed module declares no `obligations:` source. Fell back to a manual scan of IT-017-SC-01 through SC-05, TC-097, the changed `#[trace]` tests, and QSpec TC-193 F01-F34. No reverse ownership gap, stub, ignored test, vendored fixture, or copied corpus was found in the changed files. Optional semantic review was not separately requested; the concrete test-to-criterion checks above are the scoped reviewer pass.
