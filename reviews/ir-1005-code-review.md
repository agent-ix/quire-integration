---
id: SR-2026110414560009
title: IR-1005 combined input-binding code-review
type: SpecReview
analysis: code-review
review_set: subset
scope: agent-ix/quire-contract-ir; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md, spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md,
  spec/temporal/matrix/tests.md; agent-ix/quire-integration; Cargo.toml, Cargo.lock, src/lib.rs, src/temporal.rs,
  tests/temporal_mapping_binding.rs
scope_units:
- id: FR-350:membership
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "Before consuming mapping facts, the bridge SHALL require\n  `subject.belongs_to(admitted_package)`.\
    \ A separately admitted package with\n  equal contents and the same name is a different instance and SHALL NOT\n\
    \  authorize the supplied subject."
- id: FR-350:classification
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "The bridge SHALL re-run `mapping_support`\n  on that subject with the supplied surrounding-execution\
    \ closure and compare\n  the complete classification: support (including premises), retained\n  declaration/profile/activation\
    \ and sealed package/clause/definition selection."
- id: FR-350:refusal
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "A changed disposition, retained field, definition authority or identity, or\n  closure-dependent target\
    \ SHALL refuse before any TL artifact is constructed.\n  Matching package names alone SHALL NOT authorize the\
    \ classification."
- id: FR-350:projection-limit
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "Successful input binding discharges none of the\n  outstanding projection premises below."
- id: TC-461:input-binding
  path: spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "Admit the same producer output twice: equal content and package names must\n   not authorize a subject\
    \ borrowed from the other instance; a freshly read\n   subject belongs to its own instance. Alter support and\
    \ retained definition\n   authority independently, then change the surrounding closure: each mismatch\n   refuses\
    \ before projection. These binding controls alone do not establish\n   full projection or join coverage."
- id: FR-350:inputs
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "An admitted `quire_contract_model::CheckedPackageV2` and the borrowed\n  `qsl_package::checked_views::TemporalSubjectView`\
    \ returned by\n  `temporal_subject(&package, &declaration)` (QSL FR-515)."
- id: FR-350:classifier
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "The `qsl_package::checked_views::Classification` returned by\n  `mapping_support(&subject, surrounding)`\
    \ (QSL FR-527), with its complete\n  support disposition, retained subject and sealed authenticated selection."
- id: FR-350:native-evaluation
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "The checked-view classifier replaces the legacy\n  classification input; it does not replace the native\
    \ evaluation contract."
- id: TC-461:partial-evidence
  path: spec/temporal/matrix/tests.md
  role: examined
  excerpt: 'It is partial TC-461 evidence only. The

    projection, TL reader, native-result join and limit controls above remain Planned;

    this input check does not confer whole-criterion coverage.'
- id: FR-350-AC-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: Given a admitted checked-package/v2 and QSL `Supported` classification, the projection takes its target
    from the classification, emits the required formula and valuation request for each supported row, and each generated
    artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected
    alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`.
---

## Summary

Ticket: IR-1005. Coordinated PRs: quire-contract-ir#897 and quire-integration#44. One independent review, limited to consumer input binding.

Rust lane checks public borrows, typed refusals, ownership and panic paths. The consumer first rejects a foreign borrowed subject, then reclassifies with the supplied closure and compares the entire derived owner value, including its sealed selection. No evaluator or TL constructor exists on this path. Existing fixture lookup resolves Cargo dependency metadata; the model fixture is read by filesystem reference, not copied. Searched src and tests for competing binding helpers; none exists. Manifest dependencies move to production with first-party branch main unchanged. No workflow, gate, unsafe or vendoring change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the input-binding slice only. Full FR-350 projection, native/TL correspondence and limits remain Planned; this review does not qualify them.

## Coverage

No applicable AssuranceProfile was found in either spec tree. Original authored temporal source passes through the real QSL emitter and strict IR reader twice using the dependency-owned external model fixture. The test verifies equal classifications, distinct instance membership, each own-instance success, foreign-instance refusal, altered support refusal, retained definition authority refusal, and closure-dependent refusal. Supplied focused test evidence reports one passed, zero failed, zero ignored; focused Clippy completed successfully. No Cargo, Kani, CBMC or heavy build was run by the reviewer; final full gates remain the lead's responsibility.

QSL-1417: the checked-view consumer enablement is present, supported by the actual regression and its separate note. The legacy temporal Classification remains unbound and is not claimed repaired.
