---
id: SR-023
title: "Rust code review of Task 004 effect reconciliation"
type: SpecReview
analysis: code-review
scope: "src/effect_reconciliation.rs; tests/effect_reconciliation.rs; FR-003; Task-004"
review_set: all
---
# SR-023: Rust code review of Task 004 effect reconciliation

## Summary

An independent Rust review examined the Task 004 implementation for repository
idioms, authority and trait seams, replay integrity, crash safety, bounds,
panic/unsafe surface, and test traceability. All Task 004 findings were fixed;
the final review found no remaining Task 004 defect.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: exact HumanDecision and effect-decision/grant artifacts, their distinct lock selections, trusted validity result, and QObs evidence are retained and revalidated on durable replay; grant validity is rechecked immediately before transport. | FR-003-AC-1; FR-003-AC-3; `EffectRequest`; `validate_record_for_lock` | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: append events now retain complete receipts and observations, state replay requires exact equality, store substitutions refuse, and every open attempt reconciles before a possible retry. | FR-003-AC-2; FR-003-AC-5; `Event`; `validate_record` | implementation-bug-despite-evidence |
| FND-003 | high | Resolved: repeated reconciliation uncertainty no longer appends a corrupt duplicate transition; third replay and later recovery remain parseable and never redispatch unsafely. | FR-003-AC-2; TC-003 | implementation-bug-despite-evidence |
| FND-004 | high | Resolved: dispatch permits are minted after final authority validation, all authority/store/adapter ports receive the remaining total duration, sub-millisecond remainder refuses conservatively, and post-boundary deadline crossings normalize to typed uncertainty. Deterministic clock tests prove reduced budgets and safe reconciliation after overrun. | FR-003-AC-1..AC-3; NFR-001; TC-003 | implementation-bug-despite-evidence |
| FND-005 | medium | Resolved: reconciliation contract violations and post-boundary persistence failures have semantically accurate typed causes and direct tests. | FR-003-AC-2; TC-003 | implementation-bug-despite-evidence |
| FND-006 | medium | Resolved: strict persisted reconstruction checks every duplicated field and canonical frame; atomic first-use races, crash windows, supersession, and all Task 004 structural limits have direct tests. | FR-003-AC-1..AC-5; TC-003 | correct-requirement-no-evidence |
| FND-007 | low | No remaining Task 004 Rust defect. Real store/HTTP restart and trusted registry wiring remain owned by Task 009; repository-wide containment by Tasks 009/011; shared producer/duration metering and the full allocation audit by Task 007/TC-010. | Task-007; Task-009; Task-011 |  |

## Gates

- `make fmt-check`
- `make lint` with Clippy warnings denied
- `make test`: 47 ordinary tests and one compile-fail doctest
- `make build`: release profile
- `make deny`: advisories, bans, licenses, and sources pass; only existing
  unmatched-license allowlist warnings remain
- `make audit-unsafe`
- rustdoc with warnings denied
- `git diff --check`

## Verdict

PASS for Task 004. The deferred work is explicitly owned by later plan tasks and
does not weaken the feature-local crash, replay, authority, or bounds contract.
