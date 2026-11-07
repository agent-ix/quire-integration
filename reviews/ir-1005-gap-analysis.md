---
id: SR-2026110715130802
title: IR-1005 quire-integration PR 86 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-integration; PR #86 head c9e45a5d334bb2e9128788b3b2a29350c11aaf51; spec/integration/IT-022-native-temporal-projection.md, spec/test-cases/TC-115-native-temporal-projection.md, src/temporal/**, tests/temporal_*.rs, tests/support/temporal_*.rs'
review_set: subset
relationships:
  - target: ix://agent-ix/quire-integration/IT-022
    type: references
  - target: ix://agent-ix/quire-integration/TC-115
    type: references
---

## Summary

Ticket: IR-1005 (with IR-1378). This is a planless repository audit of the criteria chain for PR #86: IT-022 scenarios SC-01 to SC-16, TC-115's target table, and the FR-350 criteria it verifies (AC-1 to AC-6 and AC-8 to AC-11; AC-7 is withdrawn).

`quire matrix --scope . --format json` (quire 0.38.3, engine 0.52.1) exits 0 but returns only the engine envelope. The module set declares no traceability model for this repository, so the audit uses a grep trace index over `#[trace(...)]` attributes, as the method allows.

Plan completion: not assessed

## Verdict

CONDITIONAL. Every IT-022 scenario and every live FR-350 criterion carries at least one tagged test in the target TC-115 assigns, so no criterion is untagged. The findings below are spec-level defects in FR-350 (contract owner quire-contract-ir) and a small reverse gap. Test-depth gaps behind the tags are recorded in the code-review SR (SR-2026110715130801), which carries the high finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-350-AC-8's max_bytes count n is not determined by the inputs. The quire-mltl request wire embeds the lowered owner byte ceilings (limits.max_input_bytes and max_output_bytes), so the request and correspondence record lengths depend on the max_bytes setting itself. The limits test has to iterate to a fixed point, and at setting n-1 a shorter decimal ceiling could make the record fit. The clause that an owner refusal at a lowered ceiling becomes max_bytes limit-reached cannot be reached either: the bridge checks every owner input (bundle, views) before any owner runs, and the owner outputs (request references, trace) are smaller than the bundle. FR-350 needs a correction in quire-contract-ir | quire-mltl src/request.rs:356,745,1072-1075; tests/temporal_limits.rs:219-232; src/temporal/projection/owners.rs:40-58 |
| FND-002 | medium | FR-350 R4 (TL unsupported) cannot be reached by a genuine pair. quire-mltl classifies Unsupported only for UnsupportedProfile, UnsupportedPastNode or FutureNodeUnsupported. A bridge-built request for a QSL Supported subject never yields these, and a mapped result for any other request is TL refused (R1) first. So FR-350-AC-5's "for each of R1 to R8 a genuine owner-produced pair" is unsatisfiable for R4, and the R1-R6 pair matrix has no satisfiable R4 pairs. Contract correction needed | quire-mltl src/report.rs:735-760; src/temporal/join.rs:659-665,749-753 |
| FND-003 | medium | FR-350's Corrections section does not say whether L applies when one side is absent. The bridge evaluates L only when both results are supplied. A literal reading of "exactly one side is a correction" (an absent report is not a correction) would add result-mismatch to an R3 (unavailable) join of a TL correction with no native report, so two implementers would emit different cause lists and different join bytes | src/temporal/join.rs:689-708 |
| FND-004 | low | Reverse gap: public surface with no owning FR-350 or IT-022 statement. This covers TlJoinOwnerRefusals, ValidatedTlJoin::native_encoding_usage and native_predecessor_usage, ValidatedTlProjection::source_positions, and the extra TlProjectionError variants (PropositionMap, SyntaxRead, SyntaxEncoding, History, ExactNumber, NonIntegerNumber, Canonical, NativeActivation) that code() folds into FR-350 codes. None is specified or listed in FR-350's Outputs | src/temporal/join.rs:194-210,283-290; src/temporal/projection.rs:328-363,536-539 |

## Coverage

Trace index (grep over `#[trace]` in tests/):

- IT-022-SC-01 is in temporal_mapping_binding.
- IT-022-SC-02 to SC-07, SC-11 and SC-16 are in temporal_projection (SC-11 also in temporal_join).
- IT-022-SC-08 and SC-10 are in temporal_binding_axes.
- IT-022-SC-09 is in temporal_bridge_definition.
- IT-022-SC-12, SC-13 and SC-15 are in temporal_join.
- IT-022-SC-14 is in temporal_limits.

The FR-350 criteria are tagged as follows:

- AC-1 and AC-4: temporal_projection.
- AC-2: temporal_projection and temporal_mapping_binding.
- AC-3 and AC-11: temporal_bridge_definition and temporal_binding_axes.
- AC-5: temporal_join.
- AC-6: temporal_projection and temporal_join.
- AC-8: temporal_limits.
- AC-9: temporal_projection and temporal_binding_axes.
- AC-10: temporal_join and temporal_binding_axes.

This matches TC-115's target table. Untagged criteria: none. Ignored-only tests: none.

Max_causes (FR-350-AC-8, join) is exercised in temporal_join (ordered_causes_have_their_own_exact_and_one_short_ceiling) under an SC-12 tag, not in temporal_limits.

Stubs and coverage inflation: no todo!, unimplemented! or placeholder returns in src/temporal. The SC-15 inspection (no call path to evaluate_v2, quire_mltl::report::evaluate or report::read from src/) is confirmed by grep. The expected correspondence record (tests/support/temporal_expected.rs) is built member by member from the inputs, with no digest or frozen vector. Its single-position assumption is recorded as SR-2026110715130801 FND-001.

Semantic review: not run as a separate opt-in pass. Intent, test and code were checked inline by the code-review lane.
