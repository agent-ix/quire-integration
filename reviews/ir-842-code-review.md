---
id: SR-20261107100901
title: "Code review of quire-integration PR 87 generated-oracle versus QSL scalar agreement"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR 87 head b3979568aa8388a1303d130080593b7e0d9bfe55 diff against origin/main merge-base 8f3bcc2d; Cargo.lock; tests/generated_oracle_*.rs; tests/tc187_*.rs; tests/support/tc187_p9_source.rs; tests/support/family_*; tests/qsl_model_owner_admission.rs; tests/monitor_selected_facts.rs; tests/temporal_formula.rs"
review_set: subset
---

## Summary

Ticket: IR-842. PR: quire-integration#87. Code review with the rust-review
lane folded in. Contract: the accepted FR-014/018/021 required-cell map
(IR-842 comment cd25a010), CG FR-014 at the locked CG revision cd184a37
(AC-2, AC-6, AC-48, AC-61..65), IT-019 and TC-102/103/104/106/112 as
changed, and the planner rulings in the brief.

What is sound. The new Integer, Rational, Decimal, IEEE and Text tests
genuinely check QSL source, emit checked-package/v2 bytes, strictly admit
them through the IR reader, derive `ir_confirmed` CG claims, generate the
actual crate, and execute it natively. They compare the full outcome,
losses, ordered charges and every counter against an independent QSL
evaluation of the same checked body, and they assert the expected values
independently. The lazy Boolean right closure panics if invoked. TC-187
facts are read through `vectors_of(TestCase::Tc187)` by reference, with no
copied fixtures. Two mutations were caught: native work limit plus one
(tc187_ir842_conversions u04) and a changed Integer operand 7 to 6
(generated_oracle_integer_operations). The Cargo.lock change only moves
first-party cohort revisions (QSL 2ed49b65, IR 4f0ef630, CG cd184a37,
runtime a483bf5c, driver b96a51a1, exact 9ded73f6, QSV a16416bf, QSpec
b09800f7). It adds two edges between crates that were already in the lock
(driver to contract-runtime, quire-spec-language to qsl-eval/quire-eval).
No new external crate comes in.

Computed-operand replay. When an operand has its own charges, the native
driver first replays the owner operations on the case's meter
(`quire_semantic_value::quantity::evaluate_quantity_unit`), and only then
calls the generated symbol. QSL charges the same work on its own meter. For
completed cases this is a legitimate way to line up the two ledgers: the
whole ordered charge list is still compared, so a mismatch in the prefix
would fail the test rather than hide. It is not legitimate for stop cases.
When the stop falls inside the prefix, `break 'case` returns before the
generated symbol runs. That observation compares the runtime owner operation
with QSL, not the CG artifact, yet it is counted as generated agreement
(FND-001).

Focused test run. 30 touched targets ran at head with their own
CARGO_TARGET_DIR. 29 pass. `qsl_model_owner_admission` fails 2 of 6 tests
(FND-002). The child generated-crate builds are hard-wired to flock
/tmp/titan-cargo.lock (tests/support/generated_oracle.rs:92, which predates
this PR). The lock was free when the run started, so there was no wait.

## Verdict

**FAIL.** Two high findings. Generated-agreement credit is given to
observations where the generated symbol never ran, and a touched target
fails at head under the PR's own cohort bump.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Computed-operand replay counts stops inside the owner-operation prefix as generated observations. An instrumented run (eprintln just before the generated call) shows the generated symbol runs in only 1 of 5 U25 observations (roomy 1024). The vector's own tuple L(1,0,0,1,3,1) and works 0/1/2 all `break 'case` inside the replayed multiply/power, before the generated conversion. The same holds for U22 (3 of 5 per function; work 0/1 never reach it) and the U02 quotient (2 of 6). IT-019 still credits "five generated observations" for U25 and "21 generated observations" for U02/U22, and says the full outcome 1 J agrees at L(1,0,0,1,3,1). That observation is a runtime-versus-QSL check relabelled as generated agreement. Failure scenario: a generated conversion that mishandles work stops or tight tuples still passes every one of those cells | tests/support/tc187_p9_source.rs:400-430, 483, 709-760; tests/tc187_ir842_conversions.rs:153-176, 179-197, 201-210 |
| FND-002 | high | `qsl_model_owner_admission` fails at head. `model_nodes_are_owned_by_the_content_only_model_owner` gets [Gadget, Rock, Widget] where [Gadget, Widget] is expected, and `a_version_only_change_of_the_domain_package_keys_the_same_model_nodes` gets 3 nodes where 2 are expected. The PR edits this target (EmitLimits) and its Cargo.lock bump brings in the QSL cohort that now emits the extra model node, but the expectations were not updated. This is not the known temporal_mapping_binding/#86 issue | tests/qsl_model_owner_admission.rs:37, 264, 275; Cargo.lock:1256 |
| FND-003 | medium | The vector dispatcher contradicts the IT-019 owner rows and the planner rulings. U01 is Planned under ["IR-842"], not IR-1401. U13/U21 are still `QslValue` with "IR-842 ... generated quantity power ... missing", although U13 huge and U21 0^0 now execute. U27 keeps remaining_owner IR-1348. U28 says "IR-1348 CG compound-result generation" missing, while IT-019 records three passed native observations. U25 is `GeneratedAgreement` with no Planned entry for its first step (IR-1403). IT-019-SC-04 needs one exact classification and owner per selected operation, and the machine-checked dispatcher is the ledger that enforces it | tests/tc187_vector_dispatch.rs:108, 110-118, 144-148, 165-168, 353-360, 440-444, 455-458 |
| FND-004 | medium | The new quantity generated cells assert only mutual QSL/native agreement. `source::bind` checks that the vector text contains "5463/20", "1 m", "40000", "1 J" and so on, but it never compares the observed outcome with them. Both sides call the same QSV owner conversion and arithmetic, so a shared owner defect (for example a wrong affine offset) passes. IT-019 step 4 requires the QSpec-stated expected result as an additional oracle. IT-019 states values such as "5463/20 K twice, 7 m" as if they were asserted | tests/support/tc187_p9_source.rs:22-32, 506-509; tests/tc187_ir842_conversions.rs:72-84, 93-120, 153-176, 179-197 |
| FND-005 | low | New deprecated `/// Trace:` doc lines were added alongside `#[trace]` | tests/generated_oracle_integer_operations.rs:164; tests/generated_oracle_text_comparison.rs:134; tests/tc187_u13_u21_power.rs:262 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The dispatcher's new record of the U25 first-step (IR-1403) and U21 zero-to-negative (QSL) Planned remainders is a loop that only asserts `observed.contains(id)`. The owner string goes only into the panic message and is never compared, so the check passes whatever the owner or remainder is. U25 step 1 still has no `Planned` dispatch entry with a ticket | tests/tc187_vector_dispatch.rs:449-457 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | b33d374: computed_operations derives a CG claim for every computed operand node, and prepare_operand calls each one's generated symbol on the case meter; the owner-API replay is gone. U25 at L(1,0,0,1,3,1) and work 0/1/2 now stop inside generated code. tc187_ir842_conversions passes 9/9 at b33d374 |
| FND-002 | fixed | DECLARATIONS = [Gadget, Rock, Widget] per QSL FR-094; qsl_model_owner_admission passes 6/6 at b33d374 |
| FND-003 | fixed | QslValue variant removed; U13/U21/U28 are GeneratedAgreement, U01 is Planned under IR-1401, U27 is a QSL typed control. The U25 first-step residue is recorded as FND-006 |
| FND-004 | fixed | assert_value requires QSL's completed magnitude and unit at a generous allowance for U04, U09b, U10, U22, U25 and U02 before generation. It caught the 4 m ^ 2 source (16 m2), which was corrected to 2 m ^ 2 |
| FND-005 | fixed | the three new /// Trace: lines are removed; the remaining ones at tc187_u13_u21_power.rs:133/202 predate this PR |
