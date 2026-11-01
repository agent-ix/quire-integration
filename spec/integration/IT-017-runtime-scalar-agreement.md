---
id: IT-017
title: "Compare QSL and runtime scalar outcomes on the authoritative corpus"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-runtime/FR-006
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-007
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-192
    type: references
  - target: ix://agent-ix/quire-specification/TC-185
    type: references
  - target: ix://agent-ix/quire-specification/TC-193
    type: references
  - target: ix://agent-ix/quire-specification/TC-186
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---

# IT-017: Compare QSL and runtime scalar outcomes on the authoritative corpus

## Objective

Restore the QSL-versus-runtime agreement evidence removed from the runtime by
IR-430. A changed scalar value, disposition, typed cause, charge, or counter on
either side must fail. A source-admission case is not a successful scalar
comparison, and an unavailable case cannot disappear from the corpus.
Generated codegen oracles have a separate owner.

## Target Integration

The locked QSpec corpus supplies vector identities and expected results. QSL
parses, checks, and links authored native source into a checked package;
CheckedPackageEvaluation or evaluate_closed evaluates a checked expression
using the lock-selected quire_exact::Meter. The runtime side calls its public
exact and semantic-value operations with the same admitted operands, types,
owner identities, and ScalarLimitsV1. Identities originate in QSL admission,
never in locally constructed bytes or constituent parts.

## Preconditions

Cargo.lock selects one revision each of QSpec's
quire-specification-qualification package, QSL, runtime, and their shared exact
and semantic-value crates. Cargo metadata locates the selected QSpec package
manifest. The test reads the five explicitly named canonical TC Markdown
paths below from that package. It copies no QSpec table, uses no sibling
checkout or ambient branch, and adds no separate source revision pin. A
missing package, TC source, producer, or required tool fails the run. The test
does not publish or dispatch a hosted workflow.

## Inputs

The lock-selected QSpec files are:

- spec/test-cases/TC-192-integer-division-profiles.md
- spec/test-cases/TC-185-exact-decimal-semantics.md
- spec/test-cases/TC-193-ieee-exceptional-and-rounding-profiles.md
- spec/test-cases/TC-186-text-and-enum-identity-semantics.md
- spec/test-cases/TC-187-quantity-and-unit-conversion.md

Parse every row ID from each vector table. Split rows with multiple
operations or admission and evaluation phases into named subcases. At
authoring, TC-187 has 70 U rows, including suffixed U11 rows; RT's older
"30 vectors" statement is not the census. The selected QSpec document
controls each run. DIV-02, D02, F03, T01, U01, U10, and admission-only U11a
are useful first discriminating cases, but do not substitute for the rest.
Use each QSpec limit tuple and fresh meters for each side and denial.

## Test Procedure

1. Read all five TC documents from the lock-selected QSpec package. Build a
   run ledger with one entry per row and one entry per distinct subcase of a
   mixed row. Each entry names its source TC and row, expected observation,
   classification as evaluated, admission-only, or unavailable, runnable test
   symbol and observed result when present, and precise unavailability reason
   when absent. Reject duplicates, missing or unclassified rows, and missing
   vector tables.
   IT-017-SC-01: the ledger covers exactly the selected QSpec rows and
   subcases, with no row counted as run because it was merely listed.
2. For every evaluated entry, evaluate the checked QSL expression and the
   corresponding public runtime operation under equal inputs and limits.
   Compare the completed value and Debug rendering, or the exact noncompleted
   disposition and typed cause, with QSpec and each other. Never turn
   Undefined, Refused, Incomplete, or an evaluation fault into a Boolean or
   completed value. For typed arguments that require QSL call, assert its
   function.call envelope separately and compare the scalar charge suffix;
   report both raw meters. Prefer evaluate_closed for closed expressions.
   IT-017-SC-02: every evaluated subcase has a real two-sided observation
   and equal value or noncompleted result; an unrun evaluated subcase fails
   the completion gate.
3. For every evaluated row whose authority specifies accounting, compare
   the complete ordered charge-point and amount sequence, consumed counters,
   and first denied charge under the exact and one-under limit. Deny each
   named charge independently with a fresh meter. Assert Incomplete's limit
   kind, limit, consumed, next charge and charge point, with no later charge
   or partial result. Run the arithmetic, rounding, profile, conversion, and
   independent-oracle sweeps specified by RT TC-018 through TC-022. Name an
   unmetered authority row as such instead of calling it charge agreement.
   IT-017-SC-03: every authority-metered evaluated subcase and required
   sweep agrees in charge order, amounts, counters, denial fields, and stop
   prefix.
4. For each admission-only entry, run QSL source, type, or package admission
   and assert its exact admitted or refused result and provenance before
   scalar evaluation. Record that no runtime scalar operation ran and no
   evaluation charge was consumed. U11 owner selection and topology cases,
   DIV-03 and DIV-09, D25, T05b, and T09's stale-key half receive this
   treatment where their source rows require it. A mixed row's evaluated
   half remains governed by steps 2 and 3.
   IT-017-SC-04: each admission-only subcase has its own observed decision
   and is excluded from the evaluated agreement count.
5. Report evaluated, admission-only, unmetered, and unavailable entries by
   row and subcase, with exact observation and test symbol. An unsupported QSL
   operation, missing producer expression, or missing runtime entry is
   unavailable with the exact missing API or behavior and an owning issue;
   do not replace it with a local approximation.
   IT-017-SC-05: the full RT TC-018 through TC-022 agreement claim passes
   only with zero unavailable and unrun evaluated entries, all admission
   observations, and all step-3 accounting and sweeps. A partial run reports
   its completed subset and leaves the associated FR-006/FR-007 and TC rows
   open.

## Expected Results

All five QSpec corpora are accounted for at the lock-selected revision. Every
evaluated row and required sweep agrees on result, refusal, charge sequence,
counters, and first stop. Every admission-only subcase retains its own
decision and provenance. A changed QSpec row without a ledger entry, skipped
comparison, unowned unavailable API, mismatched charge, or copied QSpec
fixture fails completion. A passing starter subset is partial evidence, not
completion of RT FR-007-AC-1 through AC-6.

## Traceability

TC-097 invokes this procedure. Implementation symbols trace to the
applicable IT-017 success criterion and RT FR-006/FR-007 criterion, and
report the QSpec TC and vector row exercised. The per-row ledger is a run
result derived from the locked QSpec source, not a committed duplicate corpus.
