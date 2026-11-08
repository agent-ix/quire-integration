---
id: SR-2026110705000816
title: spec-review-ears of admitted runtime type-check failure qualification
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-integration; IR-1400; Cargo.toml, Cargo.lock, spec/integration/IT-018-monitor-program-construction.md,
  spec/test-cases/TC-098-monitor-program-construction.md, tests/monitor_program.rs
review_set: subset
---

## Summary

Ticket: IR-1400; PR 89, linked to RT PR 210. Reviewed the producer-admitted integration leg for fallible monitor declared-type checking. Static code/spec review is complete. Public Program construction qualification remains blocked before RT entry by the current strict IR operation catalog, owned by IR-1396 / IR PR 946.

## Verdict

**PASS for applicable changed statements; implementation qualification pending.** No aggregate pass or completed IR-1400 is claimed.

## Scope

- `IT-018-SC-16` (`spec/integration/IT-018-monitor-program-construction.md`), examined: 16. Exercise fallible declared-type construction. IT-018-SC-16: prepare valid controlled
    `ProgramParts` using the source-owned declaration lookup above before arming a failing
    allocator in a dedicated test binary. Refuse the first allocation inside
    `Program::try_new`, the first Layout binding's shared type-check worklist. Construction
    returns `ProgramRefusalCode::Membership` and no Program, retaining the native allocation
    class and original `StorageRequest` unit and amount. The owner allocator fixture uses
    one-shot denial and automatically disarms after the denied reque
- `TC-098` (`spec/test-cases/TC-098-monitor-program-construction.md`), examined: For IT-018-SC-16, run the fallible-type-construction cases in the dedicated
`monitor_program` test binary, which uses the owner's thread-local failing allocator.
Compile the authored QSL fixture and strictly admit its unchanged emission before arming
denial. Obtain every declaration, field, source location and type identity through the
existing checked-package lookup; unavailable or refused lookup fails the test. Construct
all input parts and expected native request observations before denial. Refuse the first
actual allocation inside `Program::try_new`, assert `Membership`, the original nativ
- `denied_owner_type_walk` (`tests/monitor_program.rs`), examined: fn denied_owner_type_walk(input: &ProgramParts) -> (StorageRequest, usize) {
    arm(0);
    let result = input.types.try_check_type(&input.layout.state.value_type);
    let denied = disarm();
    let Err(TypeCheckError::Allocation(request)) = result else {
        panic!("the direct owner worklist must refuse its first allocation: {result:?}");
    };
    let bytes = denied.expect("the owner's real worklist allocation was denied");
    assert!(bytes > 0);
    assert_eq!(request.unit(), StorageU
- `first_layout_type_allocation_returns_native_membership_then_retries` (`tests/monitor_program.rs`), examined: fn first_layout_type_allocation_returns_native_membership_then_retries() {
    let input = parts();
    let state_key = input.layout.state.key.clone();
    let (expected, expected_bytes) = denied_owner_type_walk(&input);

    arm(0);
    let result = Program::try_new(input);
    let denied = disarm();
    let Err(refusal) = result else {
        panic!("a denied first Layout type walk must not publish a Program");
    };
    assert_eq!(denied, Some(expected_bytes));
    assert_eq!(refusal.code,
- `later_declared_type_walk_denials_keep_native_membership_and_source` (`tests/monitor_program.rs`), examined: fn later_declared_type_walk_denials_keep_native_membership_and_source() {
    // Five Layout owner walks precede the RT scratch reservation. The four
    // function signatures contain twelve owner walks before expression types.
    for (successful, source) in [(6, "init"), (7, "event"), (18, "init")] {
        let input = parts();
        let expected_node = key(source);
        let (expected, expected_bytes) = denied_owner_type_walk(&input);
        arm(successful);
        let result = Progra
- `rt_validation_scratch_denial_stays_storage_after_five_layout_walks` (`tests/monitor_program.rs`), examined: fn rt_validation_scratch_denial_stays_storage_after_five_layout_walks() {
    let input = parts();
    let root = input.layout.root.clone();
    // Each of the five Layout bindings completes one owner worklist reservation;
    // the next reservation belongs to RT's function-key validation scratch.
    arm(5);
    let result = Program::try_new(input);
    let denied = disarm();
    let Err(refusal) = result else {
        panic!("denied RT validation scratch must not publish a Program");
    };

- `declared_type_semantics_keep_original_locus_and_layout_precedence` (`tests/monitor_program.rs`), examined: fn declared_type_semantics_keep_original_locus_and_layout_precedence() {
    // Time is a genuine admitted dimension identity, not a composite declaration
    // in this environment. The negative control never constructs a fresh key.
    let not_a_composite = ValueType::Composite(key("Time"));
    for (which, value_type) in [
        (0, not_a_composite.clone()),
        (0, ValueType::Population(None)),
        (0, ieee_type_collection(CollectionKind::Set)),
        (1, ieee_type_collection(Col
- `native_type_projection_keeps_allocation_and_capacity_requests_distinct` (`tests/monitor_program.rs`), examined: fn native_type_projection_keeps_allocation_and_capacity_requests_distinct() {
    // These actual owner error values test projection only. They are not
    // observations of allocator denial or impossible capacity exhaustion.
    for request in [
        StorageRequest::new(7, StorageUnit::Bytes),
        StorageRequest::new(19, StorageUnit::AdditionalElements),
    ] {
        for error in [
            TypeCheckError::Allocation(request),
            TypeCheckError::Capacity(request),

- `Cargo.toml` (`Cargo.toml`), examined:
qsv-failing-allocator = { git = "https://github.com/agent-ix/quire-semantic-value", branch = "main" }
qsl-analyze = { git = "https://github.com/agent-ix/quire-spec-language", branch = "main" }
quire-specification = { package = "quire-specification-qualification", git = "https://github.com/agent-ix/quire-specification", branch = "main" }
# QSL-334: the composed QSL -> IR -> codegen tests live here, because nothing depends on
# this crate. Every first-party crate is `branch = "main"`, so each res
- `Cargo.lock` (`Cargo.lock`), examined:
version = "0.0.0"
source = "git+https://github.com/agent-ix/quire-semantic-value?branch=main#f9b3afd689f45126c79102e51e8ed23c6a6de38a"

[[package]]
name = "quire-canonical"
version = "0.3.0"
source = "git+https://github.com/agent-ix/quire-canonical?branch=main#13d0fe0cfff4cc1592e8d78a4ee3f45cfdbbc9

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verification

Read the actual focused logs supplied by the coding lane: compile and focused Clippy/format/spec checks pass; native_type_projection_keeps_allocation_and_capacity_requests_distinct passes once. The attempted first_layout_type_allocation_returns_native_membership_then_retries fails in strict IR checked-package admission at operation_catalog.rs:278 on the closed vocabulary word carried_member, before parts() returns and before arm(0) or Program::try_new. The later-declared-type, RT-scratch and semantic-locus cases have not run. No reviewer Cargo command or aggregate gate was executed. No genuine allocator denial, public Membership result, retry or scratch result is credited from the passing projection test.

No production code, identity constructor, identity guard, CI workflow or safety lint is changed. The tests reuse the existing live QSL emission, unchanged strict IR read and source-owned lookup; all preparation and expected request observations occur before arming. One-shot allocator auto-disarm is stated accurately. Capacity controls use actual owner error values and explicitly make no capacity-exhaustion claim. The production mapping moves native StorageRequest values and shallow-clones the Arc-backed NodeKey without allocating a new diagnostic payload. No applicable AssuranceProfile was found under spec/.

## Coverage

Actual quoin matrix reports: no coverage_matrix criteria because the module declares no obligations source. The current quire matrix emits engine metadata without a computed criterion population. Computed criterion traceability is therefore unavailable, not zero coverage and not a PASS. Source inspection confirms five paired TC-098 / IT-018-SC-16 test attributes; this is only a structural observation, not evidence-backed computed tagging. No hand-written matrix was created. Plan completion: not assessed.

The changed artifacts are an integration test procedure and its test case, not new FR/NFR/StR requirement statements. EARS grammar is not imposed on imperative procedure steps. Changed-document Quire validation succeeds; no new applicable EARS finding was reported.

## Final replacement review scope

Frozen candidate: `910a9b282f824c63b37cda93fb45a03b2a05e218`; base `origin/main` is `1be150219e949233cbfdb0e53fadb204674b8093`. HEAD, pushed PR head and clean worktree agree. Full 33-file diff examined, including clean fixture getter/document-held identity migrations and lock changes. No code, gate, source identity guard or frozen branch was edited by the reviewer.

Independently read every exact receipt in `/tmp/IR-1400-integration-focused-receipts.json`: 79 distinct target/test pairs and logs, each reporting the named test `ok`, 1 passed and 0 failed. Distribution: 5 monitor allocator/semantic controls, 23 projection, 19 join, 14 binding, 5 limits, 2 owner agreement, 10 formula and 1 mapping. All-test package compilation and focused Clippy logs finish successfully; fmt and Quire logs were inspected. No reviewer Cargo, runtime test, aggregate, deep or Kani run was made. Lead owns the single final aggregate gate. No repository-wide coverage or upstream FR-350 acceptance is claimed.

The five constructor controls prepare source-owned unchanged strict-admitted inputs before allocator arming. First Layout and later function/expression walks retain native Membership request unit/amount/locus and retry; scratch denial stays Storage; semantic TypeMismatch/layout precedence remains distinct. Capacity cases project real owner error variants and make no actual-overflow claim. Published RT type_refusal mapping moves requests and clones only the Arc-backed NodeKey.

The port borrows the original opaque EvaluatedEvidence and never constructs/clones/escapes its immutable result. QSL published carrier definition and FR-526 were inspected. Original typed Err is distinct from absence and NonValue, is never admitted authority, and carries diagnostic data. Subject/occurrence/source and original request identity bind success before outcome comparison. Native ledger relation and direct prior identity/lineage are checked; TL retains its strict reader and predecessor checks. No join/projection evaluator invocation exists. Current request atom maps use the owner atom-index and instance-bound mapping, select the actual trigger, distinguish false from absent/Missing/Undefined, and retain original observation authority/revision. Capture anchor checks compare the original request anchor; cumulative owner NativeJoinBudget is passed unchanged. Native diagnostic work and bytes are checked before diagnostic copying; strict readers rebuild exact canonical expected records and reject mutation/foreign scope. Retired Report byte readers and predecessor-byte work/storage counters are not restored.

No applicable AssuranceProfile exists under spec/. Optional calibrated criterion-strength/Jev, security/compliance, fuzz/property and architecture campaigns were not selected. EARS has no new FR/NFR/StR statement in this diff; changed IT/TC prose is outside that grammar lens. Object/integrity/scope-boundary analyses add no separate changed artifact kind: this port changes consumption of owner-defined values, not a new local domain object, module schema or responsibility boundary. A dependency analysis is additionally applicable to the new FR-526 relationship and is recorded separately. Plan completion: not assessed.

Changed criterion audit is scoped to IT-018-SC-16 and all IT-022-SC-01 through IT-022-SC-16, TC-098 and TC-115. All authored scenario IDs were read against actual assertion bodies and the 79 exact logs; retired valuation duplicate/capture-instance/report-byte controls are explicitly identified as successor-inapplicable, never credited as passing. Future/past/fixed-sample owner construction, independent record mutation, unknown atom/foreign revision, selected instance capture, genuine R7/R8, precedence/cause order, foreign source, native failure, genuine correction and strict-read mutation controls have concrete tests. Defensive/unrepresentable cells are documented in TC-115 rather than claimed executed. Global computed matrix/inventory was intentionally not run for this blocker-only review; historical matrix unavailability remains historical, and no global completeness claim is made.

## Current verdict

**PASS — scoped current contract and supplied evidence; aggregate pending**. Earlier verification and verdict paragraphs above are historical and are superseded by this section.

### Examined units

```yaml
scope:
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: '# This file is automatically @generated by Cargo.

    # It is not intended for manual editing.

    version = 4


    [[package]]

    name = "agent-ix-extraction-frontend"

    versio'
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: '[package]

    name = "quire-integration"

    version = "0.1.0"

    edition = "2021"

    rust-version = "1.98"

    description = "Version-locked composition and integration runtime '
- id: spec/integration/IT-018-monitor-program-construction.md
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "---\nid: IT-018\ntitle: \"Retain monitor construction parts with producer-origin identities\"\
    \ntype: IT\nrelationships:\n  - target: ix://agent-ix/quire-contract-runti"
- id: spec/integration/IT-022-native-temporal-projection.md
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "---\nid: IT-022\ntitle: \"Project, record and join native temporal results through genuine\
    \ TL owners\"\ntype: IT\nrelationships:\n  - target: ix://agent-ix/quire-contr"
- id: spec/test-cases/TC-098-monitor-program-construction.md
  path: spec/test-cases/TC-098-monitor-program-construction.md
  role: examined
  excerpt: "---\nid: TC-098\ntitle: \"Monitor construction with admitted semantic identities\"\ntype:\
    \ TC\nrelationships:\n  - target: ix://agent-ix/quire-integration/IT-018\n    ty"
- id: spec/test-cases/TC-115-native-temporal-projection.md
  path: spec/test-cases/TC-115-native-temporal-projection.md
  role: examined
  excerpt: "---\nid: TC-115\ntitle: \"Native temporal projection, correspondence and join through genuine\
    \ owners\"\ntype: TC\nrelationships:\n  - target: ix://agent-ix/quire-integ"
- id: src/temporal.rs
  path: src/temporal.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! The admitted subject/classification boundary of the FR-350 temporal bridge.

    //!

    //! This check admits no TL pr'
- id: src/temporal/join.rs
  path: src/temporal/join.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350: non-evaluating admission and comparison of original owner results.


    use std::collections::BTreeSet;


    u'
- id: src/temporal/projection.rs
  path: src/temporal/projection.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! IT-022: atomic projection through the original native and TL owner APIs.


    use qsl_eval::temporal::evidence::{A'
- id: src/temporal/projection/native.rs
  path: src/temporal/projection/native.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Native correspondence checks over original immutable owner values.

    //!

    //! Steps run in the order FR-350 assig'
- id: src/temporal/projection/record.rs
  path: src/temporal/projection/record.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Bridge-owned correspondence record; owner documents remain complete byte values.

    //!

    //! The record is RFC 878'
- id: tests/generated_oracle_qsl_agreement.rs
  path: tests/generated_oracle_qsl_agreement.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Actual generated scalar/composite/function execution against checked QSL.


    #[path = "support/generated_oracle.'
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! RT construction controls using identities from a live QSL producer and strict IR reader.

    //! The controlled ex'
- id: tests/monitor_program_adverse.rs
  path: tests/monitor_program_adverse.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Real producer/IR/driver diagnostic boundaries; runtime construction remains unqualified.


    #[path = "support/mo'
- id: tests/monitor_selected_facts.rs
  path: tests/monitor_selected_facts.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Real PlanRoot emission to the bounded selected-fact reader (FR-357-AC-9).


    mod support;


    use ix_trace_rs::trac'
- id: tests/producer_origin_identity.rs
  path: tests/producer_origin_identity.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! IT-016 / TC-096: runtime identities originate in QSL source admitted by IR.


    #[path = "support/producer_origin'
- id: tests/support/monitor_program_adverse.rs
  path: tests/support/monitor_program_adverse.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Original AC3 inputs passed through the\
    \ public QSL producer and strict IR reader.\n\nuse qsl_analyze::{\n    monit"
- id: tests/support/temporal_expected.rs
  path: tests/support/temporal_expected.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! The expected correspondence record, built independently from the owner inputs

    //! by FR-350''s member table and'
- id: tests/support/temporal_join_owners.rs
  path: tests/support/temporal_join_owners.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Explicit owner fixture preparation shared by the join and limit targets.

    //!

    //! Evaluation happens here, outs'
- id: tests/support/temporal_projection.rs
  path: tests/support/temporal_projection.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350: genuine native occurrence, evidence and TL-owner projection.


    #[path = "observation_revision/mod.rs"]

    '
- id: tests/temporal_binding_axes.rs
  path: tests/temporal_binding_axes.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350 preserved source occurrence, declared capture and predicate/type axes.

    #![recursion_limit = "256"] // T'
- id: tests/temporal_binding_axes/fixture.rs
  path: tests/temporal_binding_axes/fixture.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Original-source packages for capture and shared-declaration correspondence.


    use qsl_eval::temporal::evidence:'
- id: tests/temporal_formula.rs
  path: tests/temporal_formula.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350 input binding through the public QSL producer and strict IR reader.


    mod support;


    use ix_trace_rs::tra'
- id: tests/temporal_join.rs
  path: tests/temporal_join.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350: join genuine native and mapped TL results without evaluation in the bridge.


    #[path = "support/tempora'
- id: tests/temporal_limits.rs
  path: tests/temporal_limits.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350 limits: each count is computed from the inputs, never from the bridge''s usage.


    #[path = "support/tempo'
- id: tests/temporal_mapping_binding.rs
  path: tests/temporal_mapping_binding.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350 input binding through the public QSL producer and strict IR reader.


    mod support;


    use ix_trace_rs::tra'
- id: tests/temporal_owner_agreement.rs
  path: tests/temporal_owner_agreement.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Source-backed Boolean observations through the native and TL bridge owners.


    #[path = "support/temporal_join_o'
- id: tests/temporal_projection.rs
  path: tests/temporal_projection.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-350: genuine native occurrence, evidence and TL-owner projection.


    mod support;


    use ix_trace_rs::trace;

    us'
- id: IT-018-SC-16
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: "16. Exercise fallible declared-type construction. IT-018-SC-16: prepare valid controlled\n\
    \    `ProgramParts` using the source-owned declaration lookup above before arming a failing\n    allocator\
    \ in a dedicated test binary. Refuse the first allocation inside\n    `Program::try_new`, the first\
    \ Layout binding's shared type-check worklist. Construction\n    returns `ProgramRefusalCode::Membership`\
    \ and no Program, retaining the native allocation\n    class and original `StorageRequest` unit and\
    \ amount. The owner allocator fixture uses\n    one-shot denial and automatically disarms after the\
    \ denied reque"
- id: IT-022-SC-01
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "1. IT-022-SC-01 (FR-350-AC-2): Emit source once and admit the bytes twice.\n   A local subject\
    \ and classification project. Substitute the other\n   admission's subject, then its classification,\
    \ and require\n   `tl-projection-binding` before formula construction. A package clone\n   retaining\
    \ the admission, including after a move, projects. Alter support,\n   a retained field, the definition\
    \ selection and the surrounding closure\n   independently; each returns `tl-projection-binding`."
- id: IT-022-SC-02
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "2. IT-022-SC-02 (FR-350-AC-9): Each failure uses exactly the code and premise\n   FR-350's\
    \ binding steps assign. With `request.occurrence` absent, a\n   single-occurrence subject projects\
    \ and a two-occurrence subject returns\n   native-premise `occurrence`. A `ClauseOccurrence` with\
    \ a wrong node, role\n   or ordinal, or a request occurrence key unequal to it, returns\n   native-premise\
    \ `occurrence`; a control outside the subject's requirements\n   returns native-premise `control`.\
    \ A wrong `request.binding` returns\n   `tl-projection-native-clock`; a counter clock cover returns\
    \ native-premise\n   `cloc"
- id: IT-022-SC-03
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "3. IT-022-SC-03 (FR-350-AC-6, AC-9; QSL FR-526): Project explicit true and\n   false native\
    \ request-map values and compare the genuine TL trace or history.\n   Replacing one map entry replaces\
    \ that cell's value and changes the projected\n   input. Missing, undefined and omitted cells return\
    \ incomplete\n   `valuation{position, operand}`. A key outside the checked formula returns\n   `tl-projection-valuation`\
    \ with its original request position and atom index.\n   Instance-bound atoms read only the selected\
    \ trigger's map; an unrelated\n   instance or shared value cannot supply them. Observation authorit"
- id: IT-022-SC-04
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "4. IT-022-SC-04 (FR-350-AC-1): Project a future-only formula under complete\n   and open surrounding\
    \ closure and require the closed-trace and online-prefix\n   requests. Project each bounded past operator\
    \ of QSpec FR-092 alone under\n   both false-extension profiles with origin-complete history. Repeat\
    \ with a\n   range start of 1 and with `authoritative_origin` `Some(false)` and `None`;\n   require\
    \ incomplete `history`. Every generated artifact equals its owner\n   reader's output. When declarations\
    \ share a checked node, the request's\n   subject identity is the selected occurrence's name."
- id: IT-022-SC-05
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "5. IT-022-SC-05 (FR-350-AC-2, AC-3): Unsupported classifications\n   (timestamped-event finite\
    \ window, mixed past and future, infinite trace)\n   return `tl-projection-unsupported` with the complete\
    \ classification. A\n   formula with an aggregate atom or a non-`holds` leaf returns\n   `tl-projection-formula`;\
    \ an integer interval without an upper bound or\n   with a bound above `u32::MAX` returns native-premise\
    \ `interval`."
- id: IT-022-SC-06
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "6. IT-022-SC-06 (FR-350-AC-4): Project a fixed-sample subject with a\n   nanosecond unit whose\
    \ epoch and period equal the QObs anchor. Change only\n   the period with equal formula bytes, and\
    \ reinterpret samples as event\n   counts; each returns native-premise `clock`. Omit a sample and\
    \ require\n   incomplete `position`. With a millisecond unit, a future target returns\n   native-premise\
    \ `clock-representation` and a past target returns\n   `tl-projection-owner` carrying quire-mltl FR-001-AC-9's\n\
    \   `ExpectedMismatch(\"clockBinding\")`; no unit is converted."
- id: IT-022-SC-07
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "7. IT-022-SC-07 (FR-350-AC-1, AC-9): Supply independent QObs clock, decision\n   and surrounding\
    \ progress and closure, completeness and availability views.\n   Closures compare by state and progress\
    \ by the decimal watermark. An absent\n   decision closure returns incomplete `complete-input`; differing\
    \ decision\n   closure, surrounding closure and progress return native-premise\n   `decision-closure`,\
    \ `surrounding-closure` and `progress`; absent or\n   incomplete completeness returns incomplete `complete-input`;\
    \ contradicted\n   completeness returns native-premise `completeness`. No result reader is\n "
- id: IT-022-SC-08
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "8. IT-022-SC-08 (FR-350-AC-9): Before any TL artifact, the native\n   `ActivationInput::contradiction`\
    \ admission runs against the exact\n   obligation and ledger; a changed payload under a retained trigger\
    \ returns\n   native-premise `activation`. A redelivery uses the retained payload and\n   capture\
    \ environment. A retained `Missing` fault returns incomplete\n   `activation` and a `Refused` fault\
    \ native-premise `activation`; missing,\n   null and stale captures and an evicted capture return\
    \ incomplete\n   `capture`; wrong-type and other-anchor captures return native-premise\n   `capture`.\
    \ The source-m"
- id: IT-022-SC-09
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "9. IT-022-SC-09 (FR-350-AC-3, AC-11): `read_tl_bridge_definition` admits the\n   authored\
    \ record; changed authority, identity, profile and version each\n   refuse; unknown, duplicate, missing,\
    \ trailing and non-canonical members\n   refuse; the exact record length as `max_bytes` admits and\
    \ one less refuses."
- id: IT-022-SC-10
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "10. IT-022-SC-10 (FR-350-AC-3, AC-10, AC-11): Compute the expected\n    correspondence record\
    \ from the inputs by FR-350's member table and\n    encodings and require byte equality, with no JSON\
    \ number, for a future, a\n    past and a fixed-sample projection. Mutate each member's source input\n\
    \    alone; exactly that member changes and `read_tl_projection` of the\n    original bytes returns\
    \ `tl-projection-correspondence-mismatch`. The\n    `models` member equals `selected_model_packages()`\
    \ in ascending order, and\n    two packages selecting the same models in different source orders yield\n\
    \    an ident"
- id: IT-022-SC-11
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "11. IT-022-SC-11 (FR-350-AC-6): For projection and join bytes, add an unknown\n    member,\
    \ duplicate, remove and reorder members, add whitespace and a\n    trailing byte; each returns `tl-projection-correspondence-mismatch`\
    \ or\n    `tl-join-expected-mismatch` with no partial value."
- id: IT-022-SC-12
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "12. IT-022-SC-12 (FR-350-AC-5): From genuine native and TL results reach each\n    FR-350\
    \ row R1 to R8 with its decision, comparison, causes and Boolean,\n    including an equal-final pair\
    \ whose premises differ and a final/pending\n    pair (R6), a TL `Failed` result (R2), a native `InvalidPredecessor`\
    \ (R6),\n    and a failed native execution with an evaluated Boolean report (R2, not\n    R7). For\
    \ each pair of rows R1 to R6 build one pair satisfying both and\n    require the first decision and\
    \ both causes once in row order. The mapped\n    result is re-read with `MappingSelection::for_result`.\
    \ Comput"
- id: IT-022-SC-13
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "13. IT-022-SC-13 (FR-350-AC-5, AC-10; QSL FR-526): Create genuine native\n    corrections\
    \ by changing inputs and supplying the original evaluator-written\n    ledger and direct predecessor\
    \ identity. A relation-only change is refused\n    by the owner. With both exact issued direct predecessors,\
    \ correction reaches\n    R7. One-sided correction, original with `previous`, correction without\n\
    \    `previous`, a previously unadmitted native side and a different prior issued\n    carrier reach\
    \ R6 without a Boolean. Original checked subject, occurrence,\n    source map and request mismatch\
    \ remain R1 before c"
- id: IT-022-SC-14
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "14. IT-022-SC-14 (FR-350-AC-8): `TlLimits::default()` equals FR-350's seven\n    settings\
    \ and defaults (`temporal.correspondence.max_bytes` 16,777,216,\n    `max_nodes` 65,536, `max_positions`\
    \ 1,048,576, `max_valuations`\n    16,777,216, `max_support` 1,048,576, `max_causes` 64, `max_work`\n\
    \    4,294,967,296). For each limit and each operation that charges it under\n    FR-350 (projection:\
    \ bytes, nodes, positions, valuations, work; join:\n    bytes, support, causes, work; each strict\
    \ reader: bytes), the test\n    computes the expected count `n` from the inputs by FR-350's Limits\
    \ table,\n    without re"
- id: IT-022-SC-15
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "15. IT-022-SC-15 (FR-350-AC-6): After each join, the retained native carrier\n    is the supplied\
    \ original carrier and the retained TL source result equals the\n    supplied one with unchanged owner\
    \ usage; no bridge operation has a call\n    path to `evaluate_temporal_evidence`, `quire_mltl::report::evaluate`\
    \ or\n    `quire_mltl::report::read`."
- id: IT-022-SC-16
  path: spec/integration/IT-022-native-temporal-projection.md
  role: examined
  excerpt: "16. IT-022-SC-16 (FR-350-AC-2, AC-6, AC-9): For each case below, supply one\n    input exhibiting\
    \ only that failure and require exactly the stated code\n    and premise (NP = `tl-projection-native-premise`,\
    \ INC =\n    `tl-projection-incomplete`) with no TL artifact:\n    - Admission (step 1): subject of\
    \ another admission, classification of\n      another admission, reclassification mismatch: `tl-projection-binding`;\n\
    \      `Unsupported` classification: `tl-projection-unsupported`.\n    - Occurrence (step 2): `arena_of`\
    \ failure, request occurrence key unequal\n      to the occurrence, absent key with z"
- id: FR-350-AC-1
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: 'Given an admitted package and a QSL `Supported` classification, projection takes its target
    from the classification and returns owner-admitted formula, proposition map, trace or history and
    request values for every supported row: future-only under complete and open surrounding closure (closed-trace '
- id: FR-350-AC-2
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: Timestamped-event (finite-window), mixed past-and-future and infinite-trace classifications
    return `tl-projection-unsupported` carrying the complete classification and no formula, request or
    record. A changed support, retained field, definition selection or surrounding closure, a foreign
    subject, or
- id: FR-350-AC-3
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: Mutating each correspondence-record member's source input independently (source entry, declaration,
    lock, subject profile, model selection, binding, interval, anchor, capture instance, capture, trigger,
    payload, clock, each observation view, surrounding, origin, each position's observation reference
- id: FR-350-AC-4
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: Fixed-sample projection with a nanosecond unit retains epoch, period and unit and matches them
    to the owner anchor by exact equality; a changed period or event-count reinterpretation with equal
    formula bytes returns native-premise `clock`; a missing sample returns incomplete `position`; a non-nanose
- id: FR-350-AC-5
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: For each of R1 to R8 a genuine owner-produced pair reaches that row with the stated decision,
    comparison and cause list, excluding the defensive row R4, including an equal-final pair with `P`
    false and a final/pending pair (R6), a TL `Failed` result (R2) and a native `InvalidPredecessor` (R6);
    for e
- id: FR-350-AC-6
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: Each binding step failure returns the code and premise FR-350 assigns it; a row failing several
    valuation checks returns the first cause in the owner's check order; repeated agreeing rows project
    their value and are all recorded; disagreeing or missing rows return incomplete `valuation{position,
    ope
- id: FR-350-AC-7
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: 'Withdrawn: the earlier fixed byte vector, SHA-256 known answers and derived correspondence
    reference were removed under the no-hash rule. This id is retained and never reused.'
- id: FR-350-AC-8
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: For each of the seven limits, `TlLimits::default()` returns the stated default; with the other
    six settings above their counts, an operation whose expected count `n`, computed by the test from
    its inputs by the Limits table, admits at setting `n` and, at setting `n - 1`, returns `limit-reached{limit
- id: FR-350-AC-9
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: 'Given an authenticated `TraceBinding`, the exact occurrence, the assessed instance and complete
    `ValuationEvidence`, projection uses those values and maps the opaque `request.anchor` by exact equality
    with one position''s observation identity in the admitted bundle to that position''s numeric anchor, '
- id: FR-350-AC-10
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: The projection and join retain the exact supplied package admission; the `models` member equals
    `selected_model_packages()` in its ascending identity order, and two packages selecting the same model
    set in different source orders yield an identical `models` member. A correction join retains the comp
- id: FR-350-AC-11
  path: /home/peter/.cargo/git/checkouts/quire-contract-ir-c717a33c309ad29d/43ef764/spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: context_only
  excerpt: The test constructs the expected correspondence record and join record independently from the
    owner inputs by the member tables and encodings above (no JSON number appears) and requires byte equality
    with the bridge's output for a future, a past and a fixed-sample projection and for an R7 and a corr
- id: FR-526-AC-1
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: An `Original` request returns `lineage` 0 and relation `Original`, and the returned ledger
    holds its entry for the request's key.
- id: FR-526-AC-2
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Superseding(r1)` request with changed inputs, the ledger from `r1`'s call, and the same
    package, clause, trigger and correspondence returns `lineage` 1 and relation `Superseding` naming
    `r1.result_identity`.
- id: FR-526-AC-3
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: After the AC-2 call, `r1` is byte-equal to its value before the call and the returned ledger
    holds the new entry as the latest.
- id: FR-526-AC-4
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: An `Invalidating(r1)` request under the conditions of AC-2 returns relation `Invalidating`
    and the same `lineage`.
- id: FR-526-AC-5
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: The truth, settlement and support of the AC-4 result equal those of a `Superseding(r1)` request
    with the same inputs.
- id: FR-526-AC-6
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Superseding(r1)` request and an `Invalidating(r1)` request with `r1`'s `input_identity`
    each return `Err(CallFailure::Admission)` with cause `unchanged-input` and an empty charge log.
- id: FR-526-AC-7
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Superseding` request without a `ledger` returns `Err(CallFailure::Admission)` with cause
    `no-ledger`.
- id: FR-526-AC-8
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: With a ledger holding `r1` and `r2` (`r2` superseding `r1`), a `Superseding(r1)` request returns
    `Err(CallFailure::Admission)` with cause `predecessor-not-latest`, and a `Superseding(r2)` request
    is accepted.
- id: FR-526-AC-9
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A request for another clause of the same package naming `r1`, and a request for the same clause
    in another package naming `r1`, each return `Err(CallFailure::Admission)` with cause `predecessor-unknown`.
- id: FR-526-AC-10
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A request whose trigger identity differs from `r1`'s in one byte, naming `r1`, returns `Err(CallFailure::Admission)`
    with cause `predecessor-unknown`.
- id: FR-526-AC-11
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A request whose `correspondence` differs from `r1`'s, naming `r1`, returns `Err(CallFailure::Admission)`
    with cause `correspondence-changed`.
- id: FR-526-AC-12
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: With the `lineage` ceiling set to 2, the third correction of a chain (lineage 3) returns an
    `Ok` `Incomplete` result naming `lineage` with `activation = Unavailable` and no truth, the returned
    ledger holding no entry for it, and with the ceiling set to 3 it completes.
- id: FR-526-AC-13
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: The evaluator computes `lineage` from the predecessor entry's lineage with checked `u64` arithmetic
    (`u64::checked_add` in the lineage counter, which has no wrapping or saturating form), so an entry
    at `u64::MAX` (unreachable by calls) cannot yield a smaller lineage.
- id: FR-526-AC-14
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A chain of three corrections `r1`, `r2`, `r3` returns `lineage` 1, 2, 3, each naming its direct
    predecessor, with `r1` and `r2` unchanged after `r3`.
- id: FR-526-AC-15
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: An `Original` request over a ledger that holds an entry for its key returns `lineage` 0 and
    replaces that entry.
- id: FR-526-AC-16
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: The same request, relation and ledger evaluated twice give equal results and equal ledgers,
    charge log included.
- id: FR-526-AC-17
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A compile-fail test that constructs an `EvidenceLedger` entry outside the evaluator fails to
    compile.
- id: FR-526-AC-18
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: An empty trigger identity is refused at construction of `SemanticTriggerIdentity`.
- id: FR-526-AC-19
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A non-empty trigger identity is echoed byte for byte by every result of a three-correction
    chain.
- id: FR-526-AC-20
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: For each of **R**'s three packages (FR-517), an `Original` call returns a result whose `subject`
    has `package_identity`, `package_version`, `clause`, `occurrence` and `source_map` equal to the values
    the test reads from the package and the selected occurrence (FR-515 `source_span`), and whose `corre
- id: FR-526-AC-21
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: Over the twin package of FR-516-AC-32 (one node, occurrences 7 and 3), two calls with equal
    positions, valuations and correspondence, each with the `binding` of its own occurrence's clock row,
    selecting occurrences 3 and 7 return results whose `subject` agrees in `package_identity`, `package_version
- id: FR-526-AC-22
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: 'In a unit the test authors under the event-position profile (**R** holds no such clause),
    over the clause `on origin eventually[0,1] (holds(a) and holds(b))` with `a` and `b` true at position
    0 and false at position 1, `Complete { through: 1 }` and `Closed` scope, the result is `True` and
    `support` '
- id: FR-526-AC-23
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: Over AC-22's clause with `b`'s valuation at position 0 withdrawn, and over a fixed-sample clause
    whose decision support holds a pair at a silent sample (FR-516-AC-44), no `support` entry names a
    pair the request admitted no valuation for, and every entry of every result of TC-1321 names a request
    po
- id: FR-526-AC-24
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: 'With `r1` from one request and `r1''` from a request that differs from it in one valuation
    so that their truths differ, a `Superseding(r1)` request carrying the ledger returned by `r1''`''s
    call returns `Err(CallFailure::Admission)` with cause `predecessor-not-latest`, and a `Superseding(r1'')`
    request '
- id: FR-526-AC-25
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: Over the twins of AC-21, with `r3` the result for occurrence 3, a `Superseding(r3)` request
    selecting occurrence 7 with `r3`'s ledger and changed inputs returns `Err(CallFailure::Admission)`
    with cause `predecessor-unknown`, and `r3` and its ledger are unchanged.
- id: FR-526-AC-26
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Superseding(r1)` request carrying a clone of `r1`'s returned ledger, and one carrying that
    ledger moved out of `r1`, each return a result equal to the request carrying the ledger itself; a
    compile-fail test that sets an entry's `result_identity`, `input_identity`, correspondence or lineage
    outsid
- id: FR-526-AC-27
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Superseding(r1)` request with changed inputs completes with the `lineage` ceiling 1, and
    with the ceiling 0 returns an `Ok` `Incomplete` result with `LimitReached` naming `lineage`, `value`
    0 and setting `temporal_evidence.lineage`, result `lineage` 1, relation `Superseding` naming `r1.result_ide
- id: FR-526-AC-28
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A `Meter` charged `U` work units by one call and then passed to AC-27's stopped call reports,
    after the stop, `U` plus the units of the stopped call's charge log, never less than `U`; a third
    call on the same meter continues from that total.
- id: FR-526-AC-29
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: Over the clause of AC-1, a request that is AC-1's with one extra `valuations` entry keyed by
    an atom index the formula does not have (such as 99) at position 1 returns `Err(CallFailure::Admission)`
    with cause `valuation-atom-unknown`, catalog code `invalid_runtime_input`/`invalid-value` at member
    `v
- id: FR-526-AC-30
  path: /home/peter/.cargo/git/checkouts/quire-spec-language-c986aa40394a5ad7/935d93e/spec/functional/FR-526-chain-a-temporal-evidence-result-to-its-direct-predecessor-and-refuse-a-relation-that-changes-nothing.md
  role: context_only
  excerpt: A real checked-package temporal evaluation returns an evaluator-issued immutable full-result
    carrier. Repeated immutable result and predecessor borrows retain the original truth, support, settlement,
    checked subject and ledger without further meter charges. A changed-input correction names the origi
```

## Final publication disposition (pass 3)

Final targeted disposition candidate: `871e306e6df9d37ac47d8428c51cb0ab7c48e9e8`, verified clean and equal to pushed PR89 head. Since the already-reviewed typed-wire fix b47357b, only Cargo.lock and IT-022 pending-successor wording changed; integration production code is unchanged.

Contract IR PR965 is actually MERGED at 2026-11-08T13:48:17Z with merge 079fb8523eea679e9cff7d2a2b900c73a49a5849. Fetched owner origin/main agrees, and both IR root/model entries in integration's ordinary main lock adopt that publication. The actual lock-selected FR-350, TC-461 and AD-009 source was read. FR-350 now normatively selects original shared/selected-trigger valuation maps with owner native atom mapping; explicit false remains distinct from absent/Missing/Undefined. Its request disposition table explicitly excludes aggregate sample maps from TL construction while retaining whole-request identity binding. FR-350 now requires the original opaque native call carrier, immutable complete result authority, typed Err distinct from absence and NonValue, original subject/source/request binding, native ledger lineage, and native issued identity/typed diagnostic records. It preserves the complete TL predecessor-byte validation and retains the FR-350 verification relationship and seven bridge counters. Retired native Report/ValuationEvidence APIs and predecessor-byte limits are not reinstated. TC-461 and AD-009 describe the same successor. Local IT-022's claim now names the actually published matching normative boundary. This resolves the concrete consistency defect without dropping the verifies claim or treating the direct port request as a specification waiver.

The normal refresh coherently advances all QSL packages to published 5ff99930804e036486ed045708a48554ebca622c; its changed surface includes grammar/forms/emission/planner work and has not been represented as a no-op dependency update. No new material dependency defect was established in this targeted disposition. The actual post-adoption first-Layout and native refusal smoke logs each name the intended exact test and report 1 passed, 0 failed. The scoped post-adoption Clippy log finishes successfully. Earlier 79 exact tests belong to the earlier 935d93e QSL cohort and six typed-wire tests to the b47357b cohort; none is claimed rerun on 5ff9993. The lead's final aggregate gate on the actual final dependency cohort remains pending. No reviewer Cargo, runtime, full review, aggregate, deep or Kani run was made.

**Latest review status: PASS; all real findings have latest outcome fixed, no open findings.** This supersedes historical pending/current-verdict statements above, while preserving their original findings and disposition history. SR813 FND-002 was fixed in disposition 2; SR814 FND-001 was fixed in disposition 1; SR815 FND-002 is fixed in disposition 3. Other original placeholder rows are not findings. Lead owns final gate and merge. Plan completion: not assessed.

## Targeted test-harness disposition (pass 4)

Targeted review of 7ebf6ed89c517868348b206865f001a6df483d51 only: four test-support files introduce one private Cargo-child builder for metadata, original QSL emission and generated native execution. The global flock is acquired by the actual child process; launching the parent test executable directly outside the lock removes the observed parent/child lock cycle without bypassing child serialization. Original Cargo argv, generated compiler/target arguments, data and assertions are preserved; no production or Makefile/CI changes occur. Thermal nice/ionice/job/incremental/debug settings are centralized. The actual decimal direct receipt passes 1 test. The actual producer direct receipt fails before emission with compiler mismatch E0514, exposing the missing common selected-compiler setting. New finding FND-003 is open and routed to the existing coder/PR. No broad review, Cargo command or aggregate gate was run by the reviewer. Prior real findings remain fixed; final gate and this new code defect remain pending.

## Targeted compiler disposition (pass 5)

At local ad141be9fb76e39060ec2a10b7f4075ca588e535, the selected-toolchain sibling rustc lookup, existence assertion and RUSTC env are now centralized in cargo_command::command, while generated native execution consumes exactly that same compiler selection rather than duplicating it. This repairs FND-003 without changing Cargo arguments, target, data, assertions or child locking. The actual direct producer-selected-compiler log reports genuine QSL AdmittedV2, the named test ok, 1 passed/0 failed after 214.78s. The earlier failing receipt is retained. Prior decimal direct execution already passed with the same sibling-compiler semantics; it is not claimed rerun here. Actual compile-only inventory has 64 distinct executable artifacts and build-finished success=true.

A separate scoped lint receipt is red: /tmp/IR-1400-thermal-clippy.log reports duplicate_mod for multiple declarations of the shared helper in a test crate. New medium FND-004 is therefore still-open and routed to the existing coder/PR, without a lint suppression. Latest successful scoped lint/fmt evidence and pushed frozen candidate confirmation remain pending; local HEAD currently differs from GH head 3d593f895b2c088377e73085b7fab8201ce95863. No reviewer Cargo/runtime or aggregate gate was executed. Earlier findings remain fixed. Overall current review is CONDITIONAL on FND-004 and the lead final gate.

## Targeted import-once disposition (pass 6)

Targeted FND-004 disposition on frozen pushed clean 00d8c82df6f90ea0ec77a8719aacdd55d3740b53. Reviewed only the six support/import files changed after ad141be. Producer now imports the shared support command reexport; the one protocol test combining generated execution and metadata aliases the existing generated command module. The locator moves unchanged to qsl_root.rs and resolves that single module in either parent layout. No lint allow is added. Machine comparison verifies the complete command builder is identical to ad141be apart from pub(super) becoming pub(crate), and the locator is identical apart from replacing its module declaration with a parent-module import. Selected Cargo/RUSTC, child flock, thermal environment, target directories and args retain the reviewed behavior. No production, public product API, lock, specification, data, assertions or Makefile/CI changes.

Final scoped Clippy covering lib, generated decimal, combined protocol and producer identity finishes successfully in /tmp/IR-1400-thermal-clippy-fixed.log; final fmt and whole-diff whitespace logs were inspected. The final compile-only JSON inventory reports 64 distinct actual executable artifacts and build-finished success=true. Actual ad141be decimal receipt passes the whole sole-test target (1/0, zero filtered); actual producer receipt passes its named real-emission test (1/0, 17 filtered), with genuine AdmittedV2. Both logs were read; these are runtime-behavior evidence from ad141be with import-only equivalence verified, not claimed reruns at 00d8c82. Earlier failing compiler and duplicate-module logs remain preserved. No reviewer Cargo/runtime/aggregate run. Lead remaining runtime gate remains pending, including the producer's other 17 cases.

**Latest review status: PASS; all real findings latest fixed, no open findings and no new finding in this targeted fix.** This supersedes historical pending verdicts above without rewriting findings or prior dispositions. Code SR813 FND-002/FND-003/FND-004, gap SR814 FND-001 and spec SR815 FND-002 all have latest fixed outcomes. Other placeholder rows are not findings. Lead owns remaining final gate and merge. Plan completion: not assessed.

## Targeted current-consumer disposition (pass 7)

Frozen clean pushed candidate ce46e430db8c70241e2f29b9277a09db3d3c3e46 reviewed narrowly against abedca631c1531fbebe4fb4e685ede0570670ee7. All 16 changed files are test/support consumers. No production/API/dependency/lock/specification/owner fixture-data/Makefile/CI changes. Current consumer helpers use the owner's resolved graph, preserving original selection identity and strict source/evidence assertions. The initializer adverse case finds its original document-holder index before deleting the same actual wire body. No identity is minted, rekeyed or substituted. Lazy generated Boolean closures accept the actual Meter parameter and retain the panic guard proving short circuit.

Current enum derivation checks all four original body identities and exact Equal/NotEqual EnumComparison operations; this is derivation only and claims no native enum agreement or additional PR87 cell. Current modulus refusal checks KaniFormRefused/OperationNotRendered, both inferred finite operand ranges and their original admitted bound membership, and still requires no scalar execution or generated items. Quantity controls retain exact native cause, namespace and original location: duplicate ms plus unresolved use, newly parsed compound expression's result mismatch, and unit-versus-type missing names. These are current owner-stage oracles, not mock replacements or weakened success assertions. All seven observed defect groups are recorded below and have verified fixed outcomes.

Independently read original failure excerpts and all 19 latest exact logs in /tmp/IR-1400-runtime-fix-receipts.json: every original (target,failed test) has exactly one mapped current test with the named test ok, 1 passed/0 failed. Original failure logs and failed intermediate U30 attempt remain preserved. Eighteen successful controls used prebuild-2; U30 used its final dedicated retry build. Final source then had formatting/import-only housekeeping; no exact controls are claimed rerun after that housekeeping. Final committed scoped Clippy (14 targets), fmt, diff checks pass; final selected compile-only inventory has 13 distinct executables and build-finished success=true. This was a targeted static/evidence disposition, with no reviewer Cargo/runtime, broad review, aggregate, deep or Kani run.

The lead reports prior final-gate carry of lib11, cohort1, decimal1 and producer1 and the remaining binary run with 377 passes/19 failures/4 pre-existing ignores; this disposition verifies the 19 repaired exact receipts and does not claim to have rerun the previously passing gate. Docs and deny are lead gate evidence, not rerun here. Earlier 79 and typed-wire tests retain their original cohort provenance. Lead owns final gate reconciliation and merge.

The stale PR87-owned IT-019 lines 264/268 and TC-102 lines 84/85 were explicitly outside this authorized repair and remain untouched; lead's separate owner handoff retains their prose correction. This review grants no native enum agreement credit and requests no PR87 feature/cell/branch work.

**Latest review status: PASS; all real findings have latest fixed outcomes and no open findings remain.** No new unresolved issue was found in the repaired patch. Historical findings and earlier failed outcomes below remain unchanged. Plan completion: not assessed.
