---
id: SR-20261107100903
title: "spec-review of IT-019 and TC-102/103/104/106/112 changes in quire-integration PR 87"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; PR 87 head b3979568aa8388a1303d130080593b7e0d9bfe55; spec/integration/IT-019-generated-oracle-qsl-agreement.md; spec/test-cases/TC-102-generated-scalar-operation-agreement.md; spec/test-cases/TC-103-generated-operation-family-agreement.md; spec/test-cases/TC-104-generated-function-equality-agreement.md; spec/test-cases/TC-106-quantity-static-admission.md; spec/test-cases/TC-112-tc187-carried-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
---

## Summary

Ticket: IR-842. PR: quire-integration#87. A base spec review of the changed
IT-019 procedure and disposition table, and of the changed TC rows. TC-107
and TC-110 are not changed by this diff. `quire validate` over the six
changed files passes (exit 0). The planner rulings in the brief are applied
and not re-litigated; the findings concern places where the text fails to
carry those rulings consistently.

What is sound. The SC-04 narrowing to CG-AC-selected rows matches the
ruling. The TextAdmission exclusion is correctly placed with CG TC-024. The
cohort statement names versions only, with no digest or pin record. The IR-1401
(U01, IEEE comparisons), IR-1402 (evaluated right), IR-1403 (U25 first
step), IR-1381 (FR-018) and IR-1391 (div/mod) rows each name a measured
refusal and their owner.

## Verdict

**FAIL (three medium, three low).** The disposition table contradicts
itself on owner and status for several required cells, and SC-04 forbids
that.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | U21 zero-to-negative and U27 have three rows that disagree. Row 238 says U27 is Planned and links to the "carried-operand CG seam of the U01 row (IR-1401)". Row 253 says U21 negative is Planned under IR-842. Row 258 says both are unreachable from QSL source and stay a typed control, which is the ruling. A reader cannot tell which owner or status applies | spec/integration/IT-019-generated-oracle-qsl-agreement.md:238, 253, 258 |
| FND-002 | medium | The catch-all row "FR-014 remaining operations and boundaries" files IEEE comparisons, Enum, evaluated right, U01, U21 negative, U25 first step and U27 under IR-842. That contradicts their own rows (IR-1401/1402/1403, the fixture/intake seam, typed control). TC-102 also says the IEEE comparisons, Enum and evaluated-right paths "remain Planned under IR-842". SC-04 needs one exact owner per Planned entry | spec/integration/IT-019-generated-oracle-qsl-agreement.md:270; spec/test-cases/TC-102-generated-scalar-operation-agreement.md:81-84 |
| FND-003 | medium | The Passed credit for U25 ("five generated observations ... agree at L(1,0,0,1,3,1)"), U22 and the U02 quotient includes observations that stop in the replayed owner prefix before the generated symbol runs (SR-20261107100901 FND-001). TC-106 repeats the same credit. The rows should count only the observations that executed the generated symbol | spec/integration/IT-019-generated-oracle-qsl-agreement.md:234, 256; spec/test-cases/TC-106-quantity-static-admission.md:131, 133, 138 |
| FND-004 | low | The TC-106 Planned rows for U25's first step and U01 name no owner ticket ("remains Planned", "Planned: CG's operand check ..."). IT-019 names IR-1403 and IR-1401 | spec/test-cases/TC-106-quantity-static-admission.md:133, 142 |
| FND-005 | low | The Enum Planned row names its seam (QSpec acme/shapes fixture versus QSL intake) but no ticket. SC-04 says "ticketless Planned entries fail" | spec/integration/IT-019-generated-oracle-qsl-agreement.md:266 |
| FND-006 | low | TC-102 Expected Results says "All 110 selected cases execute at work 0, 1 and 1024: 330 operation observations". That matches no cohort in the procedure: Boolean 25 + Integer 35 + Text 72 is already 132, Decimal runs at five work levels, and IEEE is 1584 observations | spec/test-cases/TC-102-generated-scalar-operation-agreement.md:97-98 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Rows 238 and 253 are removed; the single U21/U27 row reads Planned (QSL, unreachable) as a typed control, which matches the ruling |
| FND-002 | fixed | The catch-all row keeps only the Text boundaries and points each other entry to its own owner (IR-1401/1402/1403/1391, QSL); TC-102 matches |
| FND-003 | fixed | The U25, U22 and U02 rows in IT-019 and TC-106 now state that every operation runs as generated code; IT-019 adds the rule that only observations where every operation ran generated count |
| FND-004 | fixed | TC-106 U25 names IR-1403 and U01 names IR-1401 |
| FND-005 | still-open | The Enum row now names a real measured seam: CG derivation returns BlockedOnUpstream{node_tag: model} on variant_type, executed by tests/generated_oracle_enum_equality.rs. But it still says "ticket to be assigned by the IR code lead", and SC-04 rejects ticketless Planned entries. The lead decision names IR-1401; cite it in the row (IT-019:264) |
| FND-006 | fixed | TC-102 states the per-cohort counts: 483, 195 and 1584 observations |
