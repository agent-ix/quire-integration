---
id: SR-2026110414560128
title: IR-1339 PR83 code and Rust review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1339 PR83; tests/producer_origin_unit_identity.rs and IT-016"
review_set: subset
---

## Summary

Reviewed PR #83 head `a9a71a87df29e6ec07a77140b9931d7891b38865` against IT-016, QSL-1720's merged producer controls, the repository's Rust and content rules, and the public QSL/QSV/IR APIs. No production API, dependency lock, CI or gate is changed. The new test is 277 lines and substantially repeats QSL's owner test `tests/it/producer_unit_identity.rs` from QSL #2422, including the source family, admitted-context nominal graph reconstruction, meter, compound derivation and 32-root oversized conversion. This is a cross-repo code copy under the global no-copy rule; the integration observation needs an independently owned composition or an authoritative public helper. A distinct assertion does not yet prove the declared arm's key is the selected admitted root.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new integration test substantially duplicates QSL #2422's owner test implementation across repositories, contrary to the no-copy rule; consume an authoritative QSL helper or express the consumer-specific observation without reproducing the owner test body. | tests/producer_origin_unit_identity.rs:20 |
| FND-002 | medium | The declared/compound control never asserts that the declared `UnitId` names the selected admitted root; a wrong declared key from `root_unit.id()` could still satisfy the domain, inequality and ordering assertions. | tests/producer_origin_unit_identity.rs:196 |

## Verdict

FAIL pending FND-001 and FND-002. The oversized path does reach the real `Conversion::identified_quantity` handoff with genuine admitted context identities, asserts the typed fixed bound and required size, and checks no meter admission or scope insertion. The touched-root control reads actual QSL bytes with IR and checks each selected root structural id. Normal-cohort runtime qualification remains blocked by QSL #2409; author focused compilation was still running when this static review began.

## Coverage

Plan completion: not assessed. Optional broad semantic review was not run; the changed tests were checked directly against IT-016-SC-03/05 and FR-275-AC-23. `quire matrix --scope . --format json` returned module metadata but no computed criterion rows for this integration module, so traceability was checked manually from `#[trace]` attributes. No applicable AssuranceProfile was found under this repository's spec root. No Cargo build, test or final gate was run by the reviewer.

## Examined scope

```yaml
scope:
  - id: tests/producer_origin_unit_identity.rs
    path: tests/producer_origin_unit_identity.rs
    role: examined
    excerpt: "Genuine QSL admitted-context unit identities are consumed through QSV and the exact handoff, with a separate strict IR read control."
  - id: IT-016-SC-03
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "FR-277-AC-1/2/3/4/5/6 retain their exact Boolean results, seventeen-pair schedule, typed refusal, capacity denial and charge-prefix oracles."
  - id: IT-016-SC-05
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "every IR-1175 AC1–5 row has a mapped, executed symbol and observable oracle; a missing row, skipped case, copied producer fixture, frozen output digest or locally invented semantic key fails the gate."
  - id: FR-275-AC-23
    path: quire-contract-runtime/spec/exact/functional/FR-275-single-exact-kernel.md
    role: context_only
    excerpt: "QSV's Conversion::identified_quantity returns a Quantity with exact magnitude and target compound-arm UnitId, or HandoffRefusal::Identity with neither value nor identified unit."
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At rewritten PR head `ccdd457de51fde3e04629b2ebabfcb679cd303e8`, the old 277-line QSL-like test commit is absent from the PR history. The structural consumer check now uses the integration repository's existing strict-reader `admitted_units()` fixture. The remaining 172-line retained-context test implements only the actual oversized consumer handoff, with an independently arranged source and graph adapter. |
| FND-002 | fixed | The existing strict-reader fixture selects the root from admitted bytes, and the new check asserts `declared.declared_key() == Some(&root)` before comparing it to the compound arm. |

### Round 1 verdict

PASS for the reviewed code after the focused disposition. Normal-cohort test execution and the final gate remain the code lead's responsibility after QSL #2409.

```rust
let root = admitted.metre;
let declared = UnitId::declared(root.clone());
assert_eq!(declared.declared_key(), Some(&root));
```
