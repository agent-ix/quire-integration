---
id: SR-16376
title: "IR-669 PR 43 decimal refresh gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock and tests/scalar_agreement_decimal.rs; IT-017-SC-01 through SC-05; QSpec TC-185 D01-D28"
review_set: subset
---

## Summary

Planless changed-test trace audit of PR #43. Code was reviewed at 7c22cc46376f04794b1a6832e9bb84c2913549e2; final documentation head is 4aeebe140cd2aa82782433878fa01dbfdf6c0640. QSpec TC-185 now has 28 Decimal rows. The dispatcher routes each published ID D01-D28; unknown IDs panic. No new QSL completion claim is made for source forms still blocked at S2.

## Verdict

PASS for the scoped diff. No uncovered runnable obligation or unowned pending source portion was found in the changed test.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-000 | low | No findings in the reviewed diff. | — |

## Coverage

- D24 is a runnable QSL/RT agreement using `convert<Decimal[...]>(q)` with a Rational parameter. Result, loss metadata, ordered charges, counters, scale-expansion refusal, and exact-mode refusal are checked.
- D13/D18 remain RT-only evaluator-boundary rows by QSpec definition; their behavior is represented by the source-admitted D26/D27 counterparts. They are not mislabeled as QSL pending rows.
- D26/D27 RT oracles are runnable. The authored QSL literal expressions are separately routed PENDING QSL-1394 because QSL `build_unit` currently reports `UnrepresentedConstruct(ExactNumber)` at S2. D28's QSL S4 type mismatch is also PENDING QSL-1394 because its literal source cannot reach S4. D25's literal source arms have the same typed S2 pending assertion; its named-type admission and mismatch controls still run.
- Existing division source rows retain QSL-1393 pending attribution, while RT outcomes remain executable.
- The integration test discovers vectors through the locked QSpec API rather than a local copy. The QSL-1394 pending checks assert the current precise producer refusal so they fail visibly when the constructor becomes representable and the route needs promotion.
- `quoin matrix` reports no `coverage_matrix` because this module declares no `obligations:` source. I therefore manually traced TC-185 D01-D28 and IT-017-SC-01 through SC-05 against the changed dispatch and test functions. No reverse ownership gap, ignored test added by this PR, copied vector catalog, or untracked behavior was found.

Focused evidence reported on the exact code head: Decimal test 1 passed, 0 failed, 2 existing QSL-1393 probes ignored; scoped Clippy, fmt, and diff check passed. Full aggregate gates remain pending.

## Tracker note

QSL-1619 intentionally remains In Progress for PR #2277, covering D25/D26/D27/D28 source forms. QSpec #616 resolves D13/D18/D24. The final PR body and IR-1312 comment 94f054e1 cite QSL-1394/QSL-1619 for pending source forms and QSL-1393 for division rows. The code review remains unchanged; the final head adds documentation only.
