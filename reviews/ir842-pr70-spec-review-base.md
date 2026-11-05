---
id: SR-2026110414560088
title: "Spec review of IR-842 PR 70 IT-019 quantity type status"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; PR #70; spec/integration/IT-019-generated-oracle-qsl-agreement.md"
review_set: base
---
## Summary

Reviewed the changed IT-019 evidence row against the complete public TC-187 U30/U31 cells and QSpec FR-142-AC-11/12. The PR updates status evidence only; it changes no requirement statement, dependency relationship or domain object, so no additional spec-analysis lens applies. No AssuranceProfile applies.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The U30/U31 row marks all three U30 unit-namespace misses Passed, but the test's `Quantity<T>` source declares `record T` rather than the authored dimension `T`. | spec/integration/IT-019-generated-oracle-qsl-agreement.md:224 |

## Verdict

CONDITIONAL. Exercise the dimension T case through public QSL before retaining the whole U30 row as Passed; U31 and the remaining U30 branches support the stated check-only limit.

## Coverage

The changed row distinguishes static QSL checking from generated-oracle equality and keeps all other public IDs Planned. Its complete-cell claim for U30 is premature until `Quantity<T>` uses the authored dimension T. The changed IT-019 file validates.
