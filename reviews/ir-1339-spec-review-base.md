---
id: SR-2026110414560130
title: IR-1339 PR83 IT-016 base spec review
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-1339 PR83; spec/integration/IT-016-producer-origin-runtime-identity.md"
review_set: subset
---

## Summary

Reviewed the changed IT-016 Target Integration, Preconditions, procedure step 3 and Expected Results against the original Inputs and Traceability. The retained `AdmittedContext` route is accurately distinguished from strict canonical-byte admission, and the equal-byte claim is correctly narrowed to current structural UnitId arms. Two unchanged sections now conflict with or omit the new route.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Inputs still requires all admitted structural identities to come from the emitted package, contradicting the new retained-context route for checked unused roots. | spec/integration/IT-016-producer-origin-runtime-identity.md:65 |
| FND-002 | low | Traceability names only `tests/producer_origin_identity.rs`, omitting the new unit-identity module that implements the new step-3 controls. | spec/integration/IT-016-producer-origin-runtime-identity.md:137 |

## Verdict

CONDITIONAL pending reconciliation of Inputs and Traceability. No new normative requirement, digest/pin record or relationship edge was introduced; the changed wording keeps RT's broader matrix status separate.

## Coverage

Base checklist applied to changed integration spec prose and its immediate unchanged context. No new EARS-style requirement statement, object schema or dependency edge was added, so further spec analysis lenses do not apply. No applicable AssuranceProfile was found. Scoped Quire validation was reported by the coder; the reviewer will validate this artifact directly. No Cargo gate was run.

## Examined scope

```yaml
scope:
  - id: IT-016:Target Integration
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "Unit identity controls may instead use QSL's public admitted context from the same real producer run."
  - id: IT-016:Preconditions
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "For retained-context unit controls, compile authored source through QSL's compile_admitted facade and take the checked unit facts from its AdmittedContext."
  - id: IT-016:Inputs
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "Use admitted structural identities from the emitted package; never construct semantic node keys from a digest, byte string or constituent parts in an RT test."
  - id: IT-016:Test Procedure step 3
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "QSL-1720 supplies two admitted producer-origin unit routes."
  - id: IT-016:Expected Results
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "The emitted producer identities used by strict-reader controls survive IR reading; retained-context controls use genuine checked identities from that same lock-selected producer."
  - id: IT-016:Traceability
    path: spec/integration/IT-016-producer-origin-runtime-identity.md
    role: examined
    excerpt: "The intended test module is tests/producer_origin_identity.rs."
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | IT-016 Inputs now explicitly separates identities from emitted packages for strict-reader controls and checked unused roots from the same QSL compilation's retained `AdmittedContext`. |
| FND-002 | fixed | IT-016 Traceability now names both `tests/producer_origin_identity.rs` and `tests/producer_origin_unit_identity.rs`. |

### Round 1 verdict

PASS for the changed IT-016 wording on PR head `ccdd457de51fde3e04629b2ebabfcb679cd303e8`.

```text
Use admitted structural identities from emitted packages
for strict-reader controls, or from the same QSL compilation's retained
`AdmittedContext` for the checked-unused-unit handoff control.
```
