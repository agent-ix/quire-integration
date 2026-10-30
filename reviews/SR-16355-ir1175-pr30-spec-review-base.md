---
id: SR-16355
title: "Base specification review of producer-origin runtime identity"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; PR #30 ebfd2afb; spec/integration/IT-016-producer-origin-runtime-identity.md; spec/test-cases/TC-096-producer-origin-runtime-identity.md; IR-1175"
review_set: subset
---

## Summary

IT-016 and TC-096 state a live QSL producer to IR reader to exact runtime test boundary. The moved quantity, equality and function criteria remain explicit. Three specification gaps remain: a prohibited revision record, missing FR-008 linkage for the TC-025/026 rows, and missing FR-011-AC-7 in the moved unit-owner mapping.

## Verdict

**FAIL** — the revision-record requirement conflicts with this repository's no-pin/no-tracking-record guidance; two additional coverage gaps require correction.

## Examined

- IT-016 Objective and Target Integration (examined): admitted QSL-origin semantic keys survive IR reading and are consumed by QSV, quire-exact and RT.
- IT-016 Preconditions (examined): run lock-selected QSL producer and IR reader; producer absence fails rather than skips.
- IT-016 Inputs (examined): authored scenarios use admitted identities and no copied bytes.
- IT-016-SC-01 (examined): FR-012-AC-4/5, exact unit and enum owners, graph causes, canonical dimensions and terms.
- IT-016-SC-02 (examined): FR-007-AC-12/16/17/20-25, FR-011-AC-2/4/6 and FR-275-AC-23 retain quantity oracles.
- IT-016-SC-03 (examined): FR-277-AC-1/2/3/4/5/6 and TC-025/026 foreign-reference and composite/enum equality outcomes.
- IT-016-SC-04 (examined): FR-273-AC-2/3/5/8 retain application validation and refusal ordering.
- IT-016-SC-05 (examined): every moved row must map to an executed symbol and observed outcome.
- IT-016 Expected Results and Traceability (examined): admitted identity and criterion binding.
- TC-096 Description, Test Procedure and Expected Results (examined): live package output and all five IT-016 scenarios.
- FR-012-AC-4/5, FR-011-AC-7, FR-008-AC-5/6 (context_only): RT source obligations checked against the moved test rows.
- IR-1175 AC1-5 (context_only): ticket scope checked against the PR; ticket text treated as evidence claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | SC-05 requires a recorded lock-selected source revision, contrary to the no-pin/no-tracking-record rule; report the app version and observed command/outcome without a source-revision record. | spec/integration/IT-016-producer-origin-runtime-identity.md:89 |
| FND-002 | medium | SC-03 names TC-025/026 but omits their owning FR-008-AC-5/6 criteria and IT-016/TC-096 omit the FR-008 relationship, leaving the foreign-reference and composite/enum rows without explicit criterion mapping. | spec/integration/IT-016-producer-origin-runtime-identity.md:74 |
| FND-003 | medium | SC-01 moves both FR-011-AC-7-tagged unit-owner tests yet omits FR-011-AC-7 from the criterion mapping, so the new test can pass without binding that moved row. | spec/integration/IT-016-producer-origin-runtime-identity.md:60 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | f255ee13 reports producer command, application version and observed outcome; Cargo.lock remains the sole dependency revision record. |
| FND-002 | fixed | f255ee13 adds FR-008-AC-5/6 to SC-03 and FR-008 relationships to IT-016 and TC-096. |
| FND-003 | fixed | f255ee13 adds FR-011-AC-7 to SC-01, retaining the meter-consumed portion in RT. |
