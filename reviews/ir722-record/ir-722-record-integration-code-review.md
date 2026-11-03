---
id: SR-20261102072283
title: "IR-722 record quire-integration code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-722; frozen record amendment; interface_007, IT-018-SC-09 and tuple diagnostic carry"
review_set: subset
---

## Summary

No concrete source defect found in grouped field preparation, exact reads or canonical owner retention. Code-test alignment remains conditional on the dynamic record scan/order/accounting control.

Frozen record capability on original RT193 operations 1b7ad105c008ef58282b2b2b36285b548bfbb1e5 over tuple source 4430b60; integration35 55a53605667f81e854c8843a6ff3742427640d27 over 6c38070. Existing trailing review files are historical context; later capability amendments are excluded. Read actual frozen Git snapshots, repository guidance, Rust/code review and selected scoped interface methods. No worktree source edits, Cargo/heavy/proof jobs, merge/closure or permission actions performed. No applicable AssuranceProfile found.

No concrete record runtime/index/ownership defect found. Operation retains the shared MemberId, restricted explicitly to required node-owned record members. Constructor checks receiver annotation/owner/shape, then Fields::prepare groups requests by MemberId owner, scans only requested declarations, sorts borrowed field references while retaining original positions, and stores a private expression-indexed position. The locked shared MemberId Ord is owner-first, validating the grouped-prefix partition assumption. The public member identity is never replaced by a slot ordinal, and the RT does not construct a MemberId or source key.

Exact declaration reads use the prepared slot without a field scan. Dynamic reads first use the shared nominal relation and then resolve the original field identifier in actual declaration order, charging before every inspected field; their selected slot type uses existing shared native facts or bounded admission. Required/optional presence is not guessed, effective-model member owners remain unsupported by this slice, and absent physical values produce typed projection invariants. Shared tuple projection cells retain the original Arc owner; root canonical results are retained before temporary cells clear, scratch accounting includes every retained address, and no selected subtree is copied. Dynamic record behavior still needs the recorded control.

The four record controls cover declared-name versus physical-order selection, Arc owner counts, owner/member/shape/result/presence refusal, and tight scratch. A separate tuple control now covers a distinct actual tuple declaration with a test-owned native supertype edge, plus Int[0,10]→Integer bounded slot admission (work 4 preserves native admission failure; a generous budget succeeds). Keys and declaration shapes are genuinely QSL-emitted/strict-IR-read; the native supertype edges are explicitly test-owned and NOT asserted emitted by QSL. This is RT branch diagnostics, not producer relation or temporal qualification.

Supplied evidence: four record controls and the full monitor_program target of 43 passing tests, plus touched Clippy, pass ONLY on the archived interim cohort with current RT source. RT all-target/all-feature Clippy passes on Exact a3052598 / QSV 25535387. The restored committed new cohort remains unqualified due to documented driver adoption blockers. Our final dependency-head long chain, real producer, external coverage acceptance and final aggregate remain owed. IEEE has NO backend/CBMC replay credit. Full phase 2/event/watermark/gap acceptance and same-Machine lexical cleanup-after-stop remain open.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — Findings require disposition; broader qualification blockers remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The four record controls use exact receiver declarations, so none exercises the newly promised conforming-dynamic-record branch that searches the actual declaration order and charges each field visit. Add a distinct dynamic record with different physical field order, correct selected value, and tight work-stop accounting; retain explicit provenance for any test-owned native conformance edge and do not claim it was emitted by QSL. | tests/monitor_program.rs:2485-2668 |

## Examined scope

```yaml
scope:
- id: IT-018-SC-09
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "Execute required record diagnostics. IT-018-SC-09: original node-owned\
    \ member\n   identities select distinct State fields despite their declaration\
    \ order."
- id: IT-018-SC-08
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "A separate native conformance\n   diagnostic uses QSL/IR declaration keys\
    \ and shapes with explicitly test-owned\n   supertype edges."
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: The controlled executable parts exercise construction, not temporal producer/executor
    qualification.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
- id: interface-007-AC-1
  path: spec/monitor/functional/interface-007-monitor-record-projection.md
  role: examined
  excerpt: Repeated reads use the declared member identity and retain the selected
    canonical value after temporary owners drop.
- id: interface-007-AC-2
  path: spec/monitor/functional/interface-007-monitor-record-projection.md
  role: examined
  excerpt: Foreign owners, tuples, absent or optional fields, and mismatched result
    types refuse before execution.
- id: interface-007-AC-3
  path: spec/monitor/functional/interface-007-monitor-record-projection.md
  role: examined
  excerpt: Projection addresses consume scratch and explicit continuations preserve
    source evidence without subtree copies or repeated declaration scans.
bindings:
- test_id: record_members_retain_declared_identity_and_original_value_owners
  ac_id: interface-007-AC-1
  trace: correct
- test_id: record_members_refuse_wrong_owner_missing_member_tuple_and_wrong_result
  ac_id: interface-007-AC-2
  trace: correct
- test_id: record_member_addresses_consume_scratch_before_returning_state
  ac_id: interface-007-AC-3
  trace: correct
- test_id: record_required_read_refuses_optional_presence_without_inventing_an_option_value
  ac_id: interface-007-AC-2
  trace: correct
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified distinct actual record order, selected values and exact 4/5/6 work thresholds in e81881a; the native edge remains explicitly test-owned. |

Disposition inspected integration e81881a1124afb887376a26bf378203a9616f459 against 55a5360, with the reviewed RT1b7ad105 record implementation unchanged. StateHolderChild is genuinely emitted/strict-IR-read with physical order left_state/right_state, opposite the annotated parent. Its native supertype edge is explicitly test-owned and is not claimed emitted by QSL. The new dynamic control observes left=11/right=99, exact parent success at work 4, dynamic successes at 5/6, and one-below resource stops with spent 4/5 and request 1 while admission/invariant are absent. These exact totals discriminate the charged field visits and actual declaration order; no claim is made that the one-below failure occurs inside the scan rather than the later return continuation.

The original dynamic-record evidence finding is fixed as a native diagnostic. Focused control, all 44 monitor_program tests and touched Clippy pass only on the archived interim cohort (dynamic-interim-Cargo.lock); the committed new cohort was restored before push and remains unqualified. Real producer, our final dependency-head long chain, external coverage acceptance, same-Machine cleanup-after-stop, full phase 2 and IEEE backend/replay remain open. No source edits, Cargo/heavy or proof jobs by reviewer.

Current scoped source/evidence finding is fixed; overall committed-cohort/aggregate readiness remains **CONDITIONAL**. Gap-analysis and its unresolved acceptance findings are unchanged.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "let stop = failure.resource.unwrap();\n        assert_eq!(stop.resource,\
    \ StepResource::Work);\n        assert_eq!(\n            (stop.spent, stop.requested),\n\
    \            (u128::from(required_work.checked_sub(1).unwrap()), 1)\n        );\n\
    \        budget.work = required_work;\n        let monitor = build(BuildPlan {\
    \ program, budget }).unwrap();\n        assert_now(&monitor, expected);"
```
