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

Restore the QSL-versus-runtime agreement evidence removed from RT by IR-430.
A changed scalar value, disposition, typed cause, charge, or counter on either
side must fail. Source-admission behavior must remain distinct from scalar
evaluation. This test does not compare generated codegen oracles; that
composition has a separate owner.

## Target Integration

QSpec owns the public scalar-vector source for TC-192 integer division,
TC-185 decimal, TC-193 IEEE, TC-186 text and enum, and TC-187 quantity and
unit behavior. `quire_specification::{QUALIFICATION_VECTORS, TestCase, vectors_of}`
exposes each authored row's case, ID, input prose, and expected-result prose.
Integration dispatches each ID to executable behavior, including distinct
operations within a mixed row; an unknown ID fails. It consumes that API
without reading or reconstructing QSpec's Markdown. QSL parses, checks, and links original
native source into a checked package; CheckedPackageEvaluation or
evaluate_closed evaluates checked expressions with the shared exact Meter.
The runtime side calls public exact and semantic-value operations with the
same admitted operands, types, owner identities, and ScalarLimitsV1. Any
identity originates in QSL admission, never locally constructed bytes.

## Preconditions

The repository's Cargo.lock supplies one coherent set of QSpec, QSL, RT,
exact, and semantic-value dependencies. QSpec's public row API is available
under STD-612. The rows contain authored prose, not typed executable operands;
integration owns per-ID executable dispatch and admission/evaluation
classification. Full RT agreement remains open until every required row and
sweep runs. Missing producer support, a required
operation, or a required tool
fails the run. The test neither copies another repository's corpus nor
dispatches a hosted workflow.

## Inputs

Use all rows exposed by QSpec's authoritative API for TC-192, TC-185,
TC-193, TC-186, and TC-187. Each row supplies its source test and ID,
authored input and expected-result prose. Integration interprets that prose
through explicit per-ID executable cases and splits mixed rows into distinct
operations or admission phases. Use QSpec's stated
limit tuples and fresh meters for each side and each denial. The RT TC-018
through TC-022 generated operand sweeps and independent rational oracles
remain required; derive operands within this test rather than copying a
fixture from another repository. DIV-02, D02, F03, T01, U01, U10, and
admission-only U11a are useful initial exercises but cannot substitute for
the full API.

## Test Procedure

1. Obtain the authoritative rows for all five QSpec scalar test cases
   through QSpec's public API. Dispatch every row ID to executable evaluation
   or admission cases; split a mixed row into its distinct required observations.
   An unknown row ID fails. The owner's prose identifies the obligation but is
   not a typed executable request; no Markdown scan or separate tracking ledger
   substitutes for behavior.
   IT-017-SC-01: every row offered by QSpec's scalar-vector API
   reaches its specified evaluation or admission path without a silent skip.
2. For each evaluation case, evaluate the checked QSL expression and the
   corresponding public RT operation under equal inputs and limits. Compare
   the completed value and Debug rendering, or the exact noncompleted
   disposition and typed cause, against QSpec and each other. Undefined,
   Refused, Incomplete, and evaluation faults remain distinct. For typed
   arguments requiring QSL call, assert its function.call envelope
   separately before comparing scalar charges; report raw meter results.
   Prefer evaluate_closed for closed expressions.
   IT-017-SC-02: every evaluation case has a real two-sided observation and
   the same specified value or noncompleted result; an operation that cannot
   execute on either side fails.
3. For each evaluation case whose authority specifies accounting, compare
   the complete ordered charge-point and amount sequence, consumed counters,
   and first denied charge under the exact and one-under limit. Deny each
   named charge independently on fresh meters. Assert Incomplete's limit
   kind, limit, consumed amount, next charge and charge point, with no later
   charge or partial result. Run the arithmetic, rounding, profile,
   conversion, and independent-oracle sweeps specified by RT TC-018 through
   TC-022. An authority case without a charge oracle remains a value/result
   comparison and cannot be claimed as charge agreement.
   IT-017-SC-03: every authority-metered evaluation and required sweep
   agrees in charge order, amounts, counters, denial fields, and stop prefix.
4. For each admission case, run the QSL source, type, or package admission
   path and assert its exact admitted or refused result and provenance
   before scalar evaluation. No RT scalar operation or evaluation charge
   may be substituted for it. D25 is admission-only; DIV-17 and DIV-18
   combine admission observations with complete-facet scalar evaluations.
   DIV-14 requires its own QSL/SMT negotiation and backend-verdict
   observation, outside the scalar two-sided comparison. The admission
   phases of U11, DIV-03, DIV-09, T05b, and T09 are further examples;
   their QSpec-owned case definitions control the complete set.
   IT-017-SC-04: every admission case has its own observed decision and is
   never counted as two-sided scalar evaluation.
5. Fail the full RT TC-018 through TC-022 agreement gate if any QSpec case
   cannot run, any required admission observation or sweep is missing, or
   any two-sided result or accounting observation differs. A preliminary
   subset of tests may be reported as partial evidence, but it cannot mark
   FR-006/FR-007 or TC-018 through TC-022 complete.
   IT-017-SC-05: completion requires all offered QSpec cases and the full
   RT procedures, with no unsupported or omitted operation.

## Expected Results

Every QSpec evaluation case executes on QSL and RT and agrees on its
specified result, refusal, charges, counters, and first stop. Every
admission case retains its own decision and provenance. An absent QSpec
API, skipped operation, mismatch, or copied corpus fails the completion
gate. A successful starter subset is partial evidence only.

## Traceability

TC-097 invokes this procedure. Each runnable test symbol traces to the
applicable IT-017 success criterion and RT FR-006/FR-007 criterion, with
the QSpec test and case ID supplied by the owner API. QSpec owns corpus
completeness; integration asserts the behavior of every case it supplies.
