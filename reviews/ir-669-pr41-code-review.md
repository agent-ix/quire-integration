---
id: SR-20261102072327
title: "IR-669 PR 41 structural identity fixture code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; tests/monitor_program.rs; tests/producer_origin_identity.rs; Cargo.lock"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-integration/IT-016
    type: references
  - target: ix://agent-ix/quire-integration/IT-018
    type: references
---

## Summary

Ticket: IR-669. Reviewed draft PR 41's frozen Rust test and lockfile diff against the producer-origin identity requirements. The changed paths resolve each admitted node ID through the same checked package's structural-key table before runtime key decoding. The lockfile refresh selects a QSL constructor incompatible with twelve unchanged test call sites.

## Verdict

**FAIL** — the lockfile refresh makes the repository-wide Rust test and lint targets fail to compile. The two focused probes and all 86 monitor-program tests at the frozen head passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The QSL relock changes `SourceIdentity::new` to two arguments, while twelve existing test call sites still pass four; `make lint` and `make test` fail with E0061. | Cargo.lock:1300; tests/ir914_bound_export.rs:215; tests/scalar_agreement_decimal.rs:127; tests/qsl_model_owner_admission.rs:69; tests/ir669_div07_backend.rs:42; tests/qsl_rt_scalar_agreement.rs:163 |

## Coverage

Examined `tests/monitor_program.rs`, `tests/producer_origin_identity.rs`, and `Cargo.lock`. The Rust lane checked resolver provenance, nested wire IDs, enum member ownership, typed refusals, test trace attributes, and absence of a compatibility path. IT-016-SC-01, IT-016-SC-02, IT-016-SC-05, IT-018-SC-01, and IT-018-SC-13, and IT-018-SC-15 were examined. The lockfile selects one current revision per first-party source repository, but five unchanged test files still use the four-argument QSL constructor; this is a new build break caused by the relock. No CI workflow changed. `git diff --check` passed. Focused retained-program and enum-identity probes passed at this head; the full monitor-program target passed 86 of 86 tests, with none failed or ignored. No heavy build or full gate was run by this reviewer. The locked QSL source at `af77808` defines the two-argument constructor, while all twelve unchanged call sites visibly pass four arguments.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The fix round changes all twelve `SourceIdentity::new` calls in the five affected test files to the locked QSL constructor's two-argument form. A source search finds no remaining four-argument call in `tests/`; the incremental diff contains only those twelve constructor migrations. |

## Current status

The initial FAIL verdict above records the frozen review pass. The finding is fixed in the subsequent narrow fixture API migration. The reviewer found no new defect in that incremental diff. Final aggregate gates remain for the lead; the prior 86-test monitor result applies to the initial reviewed head, not an independently rerun final head.
