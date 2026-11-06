---
id: SR-2026110414560124
title: "Code review of IR-842 PR 81 TC-187 dispatcher"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #81; tests/tc187_vector_dispatch.rs"
review_set: subset
---
## Summary

Reviewed frozen head 2de5d64 against current main 71267ab, the public TC-187 table and merged PRs #72–#80. The diff changes only the fail-closed Rust test dispatcher; it adds no production API, dependency, copied fixture or workflow change. Repo Rust style and test rules were applied.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | U22 and U25 route their remaining original compound conversion and two-step bridge work to IR-842, although merged IT-019 assigns those exact Planned axes to IR-1348. The dispatch test reinforces the wrong owner by expecting IR-842 for every QslControl except U27. | tests/tc187_vector_dispatch.rs:148 |
| FND-002 | low | U36's missing-path text calls for selected-model equivalence conversion, although public U36 explicitly has no dimension equivalence declared. This adds an inapplicable condition to the row's remaining work. | tests/tc187_vector_dispatch.rs:194 |

## Verdict

CONDITIONAL. The changed row set and fail-closed ID enumeration are structurally sound, and the admitted QSL/native evidence is scoped to original operations. Correct the two owner descriptions before merge; the focused dispatch test cannot detect either factual mismatch because it checks the new literals against themselves.

## Coverage

Examined every changed dispatch arm and its assertion path, all public U02–U40 rows affected by this diff, merged IT-019/TC-106–TC-113 status evidence and the referenced integration tests. U12/U27/U28 generated work remains Planned under IR-1348; U16–U18/U29 under IR-1349; U14 has static and zero-limit controls only. Mixed U32/U34/U35/U38/U40 remain Planned, as required. No full suite or Kani run in review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised head df5cea0 routes U22 and U25 to IR-1348 and updates the QslControl owner assertion for those IDs, matching merged IT-019's Planned owner. |
| FND-002 | fixed | Revised head df5cea0 removes the unsupported selected-model equivalence conversion claim for U36; the missing-path text now names its direct dsec and whole-let cases from the public row. |
