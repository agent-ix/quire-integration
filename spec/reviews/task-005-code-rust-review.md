---
id: SR-033
title: "Rust code review of Task 005 bounded consumer views"
type: SpecReview
analysis: code-review
scope: "src/consumer_views.rs; tests/consumer_views.rs; FR-004; Task-005"
review_set: all
---
# SR-033: Rust code review of Task 005 bounded consumer views

## Summary

An independent Rust review examined the Task 005 implementation for repository
idioms, trait seams, canonical traversal, panic/unsafe surface, bounded retention,
consumer error precedence, authority isolation, and test traceability. All findings
were fixed; the final review found no remaining Task 005 Rust defect.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: the coordinator-owned sink checks result count, per-item and aggregate output bytes, and mapping count before copying borrowed provider payloads; a later refusal discards all previously accepted payload-bearing siblings. | FR-004-AC-3; FR-004-AC-4; `ConsumerOutputSink` | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: sink exhaustion or invalidity takes precedence over a consumer error returned after refused submission, preserving `incomplete`/`requires-bound` or the exact invalidity instead of masking it as consumer failure. Direct exhausted-plus-error and invalid-plus-error regressions cover both paths. | FR-004-AC-4; TC-004 | implementation-bug-despite-evidence |
| FND-003 | high | Resolved: traversal charges canonical work before expansion, retains the full immediate edge frontier, removes already-visited queue entries before timer checks, and preserves exact independent usage dimensions. | FR-004-AC-1; FR-004-AC-3; TC-004 | implementation-bug-despite-evidence |
| FND-004 | medium | Resolved: foreign obligation definitions are item-local invalid while valid siblings invoke; malformed maps, closed UTF-8/binary profiles, inventory mismatches, consumer failures, output exhaustion, and finalization deadlines retain distinct typed outcomes. | FR-004-AC-2; FR-004-AC-4; TC-004 | correct-requirement-no-evidence |
| FND-005 | medium | Resolved: runtime type-identity tautologies were removed. A stable-Rust negative trait-implementation assertion plus four Task 005 compile-fail doctests prove that views, permits, facade runs, and accounting records cannot convert into or construct owner authority types. | FR-004-AC-6; TC-004 | correct-requirement-no-evidence |
| FND-006 | low | No remaining Task 005 Rust defect. Strict real-consumer adapters and unchanged canonical owner-value pass-through remain owned by Task 010. | Task-010; IT-003 |  |

## Gates

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`: 61 ordinary tests, including 14 TC-004 tests
- `cargo test --doc --all-features`: five compile-fail doctests
- release build and rustdoc with warnings denied
- dependency policy and RustSec audit
- zero-unsafe audit
- native strict Quoin validation
- `git diff --check`

## Verdict

PASS for Task 005. Deferred real-consumer integration is explicitly owned by
Task 010 and does not weaken the feature-local bounded, deterministic boundary.
