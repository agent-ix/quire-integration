---
id: SR-2026110715130801
title: IR-1005 quire-integration PR 86 code-review (Rust lane)
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-integration; PR #86 head c9e45a5d334bb2e9128788b3b2a29350c11aaf51; git diff origin/main...HEAD: src/temporal.rs, src/temporal/{definition,formula,join,projection}.rs, src/temporal/projection/{native,owners,record}.rs, tests/temporal_*.rs, tests/support/temporal_*.rs, tests/temporal_projection/observation.rs, tests/temporal_binding_axes/fixture.rs, Cargo.toml, definitions/native-temporal-correspondence-v1.json'
review_set: subset
---

## Summary

Ticket: IR-1005 (with IR-1378). Code review with the rust-review lane folded in, over the one native temporal bridge PR. Contract: quire-contract-ir FR-350 / TC-461 / AD-009 (merged), quire-integration IT-022 and TC-115 (merged, #88). TL result input is the quire-mltl ValidatedMappedResult.

The production bridge follows FR-350's binding steps 1 to 9, construction order, record member table and encodings, join state columns, rows R1 to R8, cause ordering and limit charging closely. No evaluator is called from src/ (grep for evaluate_v2, report::evaluate and report::read finds no match). The expected record is built in the test from the inputs, there are no digest constants for expected values, and owner values come from genuine owner producers and readers. The defects are mostly in test evidence. Every projection fixture has exactly one request position and one bundle record, so the multi-position projection logic is never run. Several premises that FR-350 assigns explicitly are never exercised. There is also one place where the code diverges from the contract on clock representation.

## Verdict

FAIL. There is one high finding: the multi-position path has no tests, and a mutation to it survived. There are also medium findings: assigned premises are not tested, the clock-representation check is skipped for a non-nanosecond past clock, and the AC-1, AC-3, AC-4 and AC-5 matrices are only partly covered. Gate results: the TC-115 focused targets were run by this reviewer (see Coverage), and cargo fmt --check passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Multi-position projection is never exercised: every fixture has one request position and one bundle record, so stamp ordering, gap, repeated stamp, stamp above through, range length, several anchor matches, duplicate bundle observation identity (named in FR-350-AC-9) and the TL anchor offset are untested. The expected-record builder pairs sorted stamps with request index by enumerate, so it assumes request order equals stamp order and could not detect an ordering bug. A mutation that drops the stamp sort survives every test | src/temporal/projection/native.rs:159-232; tests/support/temporal_projection.rs:424-428; tests/support/temporal_expected.rs:88-95; tests/temporal_projection/observation.rs:58-90 |
| FND-002 | medium | Premises that FR-350 explicitly assigns have no test through project_tl: native-premise subject-name, completeness (QObs Contradicted, IT-022-SC-07), source, interval (IT-022-SC-05) and clock-representation for a rational epoch; tl-projection-formula for a non-holds or aggregate atom (IT-022-SC-05, FR-350-AC-3); tl-projection-encoding | src/temporal/projection/native.rs:107-109,271-277,414-426; src/temporal/projection.rs:599-610,537-578; tests/temporal_projection.rs:666-877 |
| FND-003 | medium | FR-350-AC-1's incomplete-history clause is only half tested: authoritative_origin Some(false) is covered, but range start 1 and authoritative_origin None are never exercised (IT-022-SC-04 requires both) | tests/temporal_projection.rs:303-345; src/temporal/projection/native.rs:227-231 |
| FND-004 | medium | FR-350-AC-4 is only partly tested. The missing-sample case (incomplete position) is absent. The non-nanosecond past case asserts only matches!(Owner(_)), not the quire-mltl ExpectedMismatch("clockBinding") refusal that IT-022-SC-06 names, so any owner refusal passes | tests/temporal_projection.rs:911-950 |
| FND-005 | medium | FR-350-AC-3's member-mutation matrix covers 6 of the roughly 25 listed source inputs (trigger scope, origin, order key, position reference, correspondence reference, completeness facts). Source entry, declaration, lock, subject profile, models, binding, interval, anchor, capture instance, capture, trigger, payload, clock, each observation view, surrounding, owner document and valuation row are never mutated | tests/temporal_binding_axes.rs:724-800 |
| FND-006 | medium | FR-350-AC-5 is only partly covered. The R1-R6 pairwise matrix has 3 of 15 pairs (R3+R5, R2+R6, R5+R6). The SC-13 lineage matrix lacks one-sided correction, original with previous, correction without previous and TL missing-predecessor, unexpected-predecessor and predecessor-mismatch as separate cases. R4 has no test (see the gap-analysis SR: the spec is unreachable there) | tests/temporal_join.rs:207-280,727-958; src/temporal/join.rs:689-767 |
| FND-007 | medium | For a non-nanosecond fixed-sample clock with a past target, check_owner_anchor returns Ok before the rational check. A rational epoch or period with a denominator other than 1 then reaches the TL owner, giving tl-projection-owner or an owner admit. FR-350 step 6 states the native-premise clock-representation rule for rationals without a unit condition | src/temporal/projection/native.rs:368-377; src/temporal/projection/owners.rs:169-173 |
| FND-008 | low | An activation contradiction returns TlProjectionError::NativeActivation. Its code() is NativePremise, but its payload is not TlProjectionPremise::Activation, while a Refused fault returns NativePremise(Activation). A caller matching the FR-350 payload "native premise activation" misses contradictions | src/temporal/projection/native.rs:492-501; src/temporal/projection.rs:362-363 |
| FND-009 | low | native_state maps every remaining ReportOutcome through a wildcard arm into final or pending. ReportOutcome is not non_exhaustive, so a new owner outcome would be treated silently as an admitted Evaluated report instead of failing to compile (rust-style: exhaustive matching over closed owner enums) | src/temporal/join.rs:462-463 |
| FND-010 | low | tests/temporal_projection/observation.rs (427 lines) is a near copy of the in-repo tests/support/observation_revision/mod.rs (443 lines). It differs only in anchor, range and scope kind, which should be parameters of the existing fixture | tests/temporal_projection/observation.rs:1-427; tests/support/observation_revision/mod.rs:1-443 |

## Coverage

Production code examined in full: projection.rs (types, Budget, check_intervals, project_with, read_tl_projection), native.rs (steps 1 to 9), owners.rs (formula, proposition map, trace and history construction, lowered limits, owner error mapping), record.rs (every member and encoding against FR-350's table), definition.rs, join.rs (state columns, P, L, R1 to R8, causes, join record), and the temporal.rs ForeignClassification addition.

Matched against FR-350 with no defect: the code vocabulary; admission ordering; occurrence, control and subject-name; binding and cover; correspondence reference; anchor, instance, activation, retained and delivered captures, and evictions; observation revision, duplicate identity and per-position checks; step-7 range and gap logic as written; step-8 closure, progress and completeness; step-9 valuation check order; the record member set and hex, dec, str and IR encodings with no JSON number; the canonical writer limit mapping; the TL and native state columns; the settlement mapping; P; R1 to R8 predicates and cause ordering; max_support charged only when both reads admit; causes charged before the record; PredecessorLimit and NativeBudget as tl-join-limit-reached; bridge-definition strict reading.

Coder-resolved ambiguities checked against FR-350. valuation{position} as the native request index agrees with step 9's PositionNotAdmitted bound. The retained capture count as the retained instance's environment entries agrees with step 5's ledger branch. L evaluated only when both sides are supplied is a spec ambiguity (see the gap-analysis SR).

Changed existing tests (relative to main). temporal_mapping_binding replaces the classification-equality assertion with per-field equality plus an unequal admission binding, which the owner's admission-bound classification requires; it adds ForeignClassification, clone and move. temporal_formula only adopts the TemporalClockSelection::new and EmitLimits API. Neither change weakens an assertion. The NativeJoinBudget never-reset property is still asserted (tests/temporal_join.rs:1058-1121).

Mutations (own worktree): see the Mutation results section appended below.

Gates: cargo fmt --check exit 0, with the stable-toolchain notices only. The full make ci and clippy were not run (the lead owns the single final gate).

## Mutation results

Focused baseline (own CARGO_TARGET_DIR in the throwaway worktree): cargo test on all eight TC-115 targets, exit 0, 64 passed and 0 failed (projection 17, binding_axes 10, join 15, limits 5, bridge_definition 1, mapping_binding 1, formula 9, observation_revision 6).

| Mutation | Change | Targets run | Result |
| --- | --- | --- | --- |
| M1 join table row | remove the native execution Failed condition from R2 (join.rs:740) | temporal_join | killed, exit 101 (native_failed_execution_precedes_equal_boolean_and_retains_all_causes) |
| M2 limit | skip the join max_work charge for support (join.rs:338) | temporal_join, temporal_limits | killed, exit 101 (support_work_and_byte_counters_accept_exact_and_refuse_one_short_with_fresh_usage) |
| M3 encoding rule | declaration ordinal written as a JSON number instead of dec (record.rs:184) | temporal_binding_axes records_equal | killed, exit 101 (records_equal_the_independently_built_expected_bytes_for_future_past_and_sample) |
| M4 TL position order | iterate stamps in descending order (native.rs:221) | temporal_projection, temporal_join, temporal_limits | survived, exit 0 (FND-001) |
| M5 duplicate observation identity | drop the native-premise observation return (native.rs:167) | temporal_projection, temporal_join, temporal_limits | survived, exit 0 (FND-001) |

## Dispositions

Round 1, verified against the code at quire-integration#86 final head 92f80d8a and at the #90 batch head 823e1817.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 92f80d8: stamp order, repeat, gap, range length, several-anchor match, duplicate observation identity and the TL anchor offset moved to src/temporal/projection/positions.rs (order_by_stamp, anchor_stamp, unique_identities, tl_anchor), with unit tests there. IT-022 "Checks verified at unit level" records why a genuine single-stamp QObs bundle cannot reach them |
| FND-002 | still-open | Rational clock, Contradicted completeness and the interval above u32::MAX are now tested through project_tl (tests/temporal_projection.rs:1043, 1064, 1089). Subject-name and encoding are verified at their producing function per IT-022. The aggregate-atom tl-projection-formula case (823e181, tests/temporal_formula.rs:672-695) only checks a hand-built TlProjectionError::Formula, not the project_tl path; pending the #90 fix round (SR-2026110720500001 FND-003) |
| FND-003 | fixed | tests/temporal_projection.rs:1021 past_history_needs_range_start_zero_and_an_authoritative_origin covers range start 1 and authoritative_origin Some(false) and None, each returning incomplete history |
| FND-004 | fixed | The missing sample is incomplete position (positions.rs a_missing_sample_or_range_mismatch_is_incomplete). The past non-nanosecond case asserts the owner field "clockBinding" (tests/temporal_projection.rs:953) |
| FND-005 | fixed | each_mutated_source_input_changes_exactly_its_member_and_the_original_bytes_mismatch plus package_and_view_inputs_change_their_members and valuation_rows_progress_views_and_captures_change_their_members cover the FR-350-AC-3 source inputs (tests/temporal_binding_axes.rs:717, 844, 902) |
| FND-006 | fixed | reachable_row_pairs_return_the_first_decision_and_both_causes (R1+R3, R1+R5, R3+R5, R1+R2, R2+R3, R2+R3+R5) and the 823e181 R1+R6 test extend the pair matrix, with R4 excluded as defensive. each_lineage_condition_reaches_conflict_in_its_own_case separates the SC-13 lineage cases |
| FND-007 | fixed | native.rs:344-347 reads epoch and period as integers before the unit check, so a rational is native-premise clock-representation under every unit (tests/temporal_projection.rs:1043) |
| FND-008 | fixed | The NativeActivation variant is removed; an activation contradiction returns NativePremise(Activation) |
| FND-009 | fixed | native_state matches every ReportOutcome variant explicitly (src/temporal/join.rs native_state), with no wildcard arm |
| FND-010 | fixed | tests/temporal_projection/observation.rs is deleted, and tests/support/observation_revision/mod.rs is parameterized instead (cb94167) |
