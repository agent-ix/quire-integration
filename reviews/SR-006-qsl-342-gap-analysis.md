---
id: SR-006
title: "QSL-342 per-module SUCCESS attribution gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration@2d4bb1e7d2b3defc81346ee3a24e269e007f879c; tests/qsl_kani_exemplar.rs, spec/integration/IT-011-function-application-kani-exemplar.md"
review_set: subset
---
# QSL-342 per-module SUCCESS attribution gap analysis

## Summary

Ticket: QSL-342, the per-module attribution scope from the 2026-10-01 reopen comment. PR quire-integration#13. This is a manual check of acceptance criteria against tests.

- Bump QI to CG main: met. b75e1ac is CG main (#210). It is the only lock move.
- Attribute SUCCESS checks to each CLAIMED_MODULES module by location file: met by `success_census` over CG's `KaniCheckResult`. The match is SUCCESS-only, on an exact canonical path.
- Fail, naming the module, when a claimed module discharges none: implemented inline in `healthy_run_discharges_success_checks_in_every_claimed_module`. No automated check covers the failure path (FND-001).
- Replace Kani-prose scraping with CG's parse: met. `discharged_success_checks` is deleted, and the healthy count and both mutation verdicts come from `classify_kani_run`.
- IT-011-SC-01: healthy run exits successfully and `run.success_checks > 0`. Backed.
- IT-011-SC-02 and IT-011-SC-03: unchanged assertions, now read through `classify(...).outcome`. Backed.
- IT-011-SC-04: the attribution rule (prefix, other crate, relative and absolute, non-SUCCESS) is backed by `success_census_attributes_checks_by_exact_file`, and the real-run census by the healthy test. The clause "a claimed module attributed none fails the test, which names it" is not backed (FND-001).
- Parked, not in scope: the Verified read (IR-464), TerminalValue (CG step 5) and the arithmetic control branch.

Bindings: the healthy test maps to IT-011-SC-01 and IT-011-SC-04, which is correct. The unit test maps to IT-011-SC-04, which is correct. The connective test maps to IT-011-SC-02 and the addition test to IT-011-SC-03, both correct. The crate is test-only, so there is no untraced production code.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | IT-011-SC-04's failure clause ("a claimed module attributed none fails the test, which names it") has no test. The `unreached` filter and its assert are inline in the real-Kani healthy test, which cannot reach them on a healthy run. A mutant `filter(\|_\| false)` passes every test. Extract the filter into a helper and assert, in the unit test's zero-census case, that it returns the claimed module paths. | tests/qsl_kani_exemplar.rs:547-557 |

## Verdict

The QSL-342 attribution scope is delivered, apart from one untested failure clause of SC-04. SC-01 to SC-03 stay backed. The fix is a small in-PR test. The PR is mergeable after that.

## Dispositions

| FND | outcome | sha/reason |
|---|---|---|
| FND-001 | fixed | e4425e9745f2a2ec53315659841dfee1b3c8c36f (round 1: `unreached` and `assert_every_module_reached` extracted; the census unit test asserts `unreached` on full, zero and partial censuses and catches the panic message naming the unreached module. Reviewer mutants in a throwaway worktree: `filter(\|_\| false)` fails at :780, and `true \|\| unreached.is_empty()` fails at the catch_unwind expect_err; unmutated passes) |
