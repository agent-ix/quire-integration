---
id: SR-2026110414560023
title: IR-842 PR46 code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-integration; PR46; eight changed files; IT-019 SC-01..06; TC-099
review_set: subset
---

## Summary

IR-842 existing PR46; frozen head 01f4b978fc6a07a57c2f55c740e96c3950078fa2. Reviewed the entire eight-file diff, prior comment 06f97c6d and custody 8d4f1596. No applicable AssuranceProfile found. This review changes no branch files and posts findings only to IR-842.

## Verdict

CONDITIONAL within changed scope. Overall PR is blocked by the high base-spec finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The work=0 and work=1024 controls only compare the two paths; assert_same_call also accepts identical CallFailure or stop outcomes. A shared regression can replace the required work=0 FunctionCall denial or work=1024 Completed(Integer(7)) with matching failures and leave this test green. Assert the specified terminal variant/payload, work=0 body location and empty admitted ledger independently for those two limits, as already done for work=1. | tests/support/function_observation.rs:198 |

## Coverage

Rust lane: actual public QSL acquisition/emission, strict IR admission, authenticated lowering and CG generation are used; generated artifacts are written unchanged and their exported symbol is compiled/executed by a separate integration driver. The child acquires the original source independently, obtains the owner callable from its QSL registry and compares full typed Evaluation/CallFailure, locations, ordered losses, admitted charges and every LimitKind counter. Public locked CG owner_path code confirms generated bodies are registered only on the generated side; the authority registry receives no generated factory. Foreign subtraction source is refused before charges and followed by a healthy call. Scalar helper observations intentionally omit locations and integer whole-call accounting parity; they claim partial TC-099 only. One non-discriminating work=0/1024 expected-shape gap is recorded below. No production implementation, duplicate evaluator, fixture corpus or CI weakening added.

## Verification limits

No Cargo, test, gate, global census/matrix/coverage or Kani invocation by reviewer. Existing merged-driver-combine-locus compile/native receipts both exit 0; native log records one passing combine test with five filtered out. Earlier successful Boolean-matrix and bare-parameter receipts are historical focused runs, not final-head family requalification; failed intermediate diagnostic receipts were not treated as passing. Native generated child warnings about cfg(kani) and unused imports remain visible; no warning-denied child qualification is claimed. Final scoped gate belongs to the lead. No whole-family/TC-187/IR-842 completion is credited.

## Examined units

```yaml
scope:
- id: IT-019-SC-01
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-01: both paths return the same `Incomplete` with\n   `WorkUnits`,\
    \ limit 1, consumed 1, next charge 1 and\n   `IntegerArithmeticOperands`; their\
    \ one admitted `FunctionCall` charge,\n   all counters and `Some(Location { Body\
    \ { function: \"combine\", index: 0 },\n   depth: 0, last_index: None })` agree\
    \ exactly. A missing generated location,\n   a synthetic QSL evaluator, or a nonexecuted\
    \ oracle fails this criterion."
- id: IT-019-SC-02
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-02: work=0 stops at `FunctionCall` before body work with no\n\
    \   admitted charge and the same real body location; work=1024 completes with\n\
    \   integer 7 and identical located losses, charge sequence and counters.\n  \
    \ Neither result can compensate for a failure at work=1."
- id: IT-019-SC-03
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-03: a mismatch is refused before generated execution or meter\n\
    \   charge, the refusal is distinguishable from a QSL lookup or input refusal,\n\
    \   and the healthy sibling's generated symbol, disposition and independent\n\
    \   QSL comparison are unchanged. No replacement identity or index is minted."
- id: IT-019-SC-04
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-04: every API row and required operation has one explicit\n\
    \   classification and observed path or ticket-backed Planned entry; unknown\n\
    \   IDs, silent skips, copied fixtures and ticketless Planned entries fail.\n\
    \   Only executed two-sided rows with matching owner expectations count as\n \
    \  generated-oracle/QSL agreement."
- id: IT-019-SC-05
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-05: each named family and operation has an executable two-sided\n\
    \   result or an explicit ticket-backed Planned entry. Runnable results agree\n\
    \   in the full exposed envelope. A passing scalar sample cannot mark the\n  \
    \ composite or function family complete."
- id: IT-019-SC-06
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "IT-019-SC-06: the report distinguishes shared-kernel value equality from\n\
    \   independent admission, selection and actual generated execution evidence;\n\
    \   it makes no whole-family or whole-TC-187 claim from a minimal sample."
- id: IT-019-Inputs
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'The first executable function is authentic QSL source

    `combine(left:Int[0,9],right:Int[0,9]) {left+right}` called with admitted

    operands whose sum is 7, under global work limits 0, 1 and 1024. Select the

    real `combine` declaration, body and call occurrence from QSL''s checked

    package; do not fabricate node keys or substitute an interpreted body.

    Include a healthy sibling and a separately refused item for binding controls.

    Subsequent cases select applicable TC-187 rows from the public vector API and

    applicable CG FR-014 scalar, FR-018 composite equality, and FR-021 function

    application families.'
- id: IT-019-Coverage
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: Coverage is partial. Combine work=1, 0 and 1024 and foreign-source refusal
    are Passed; wrong-occurrence/mixed-request, TC-187 and complete FR-014/018/021
    remain Planned.
- id: TC-099
  path: spec/test-cases/TC-099-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: Execute IT-019-SC-01 through SC-06. Obtain the published TC-187 cases by
    reference through QSpec public vector API and dispatch every case explicitly.
- id: tests/generated_oracle_qsl_agreement.rs
  path: tests/generated_oracle_qsl_agreement.rs
  role: examined
  excerpt: Complete added file reviewed.
- id: tests/support/function_agreement.rs
  path: tests/support/function_agreement.rs
  role: examined
  excerpt: Complete added file reviewed.
- id: tests/support/function_observation.rs
  path: tests/support/function_observation.rs
  role: examined
  excerpt: Complete added file reviewed.
- id: tests/support/generated_oracle.rs
  path: tests/support/generated_oracle.rs
  role: examined
  excerpt: Complete added file reviewed.
- id: tests/support/oracle_observation.rs
  path: tests/support/oracle_observation.rs
  role: examined
  excerpt: Complete added file reviewed.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: First-party cohort refresh; no Cargo.toml dependency changes.
```
