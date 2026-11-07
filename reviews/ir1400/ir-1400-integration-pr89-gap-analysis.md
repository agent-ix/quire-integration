---
id: SR-2026110705000814
title: gap-analysis of admitted runtime type-check failure qualification
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-integration; IR-1400; Cargo.toml, Cargo.lock, spec/integration/IT-018-monitor-program-construction.md,
  spec/test-cases/TC-098-monitor-program-construction.md, tests/monitor_program.rs
review_set: subset
---

## Summary

Ticket: IR-1400; PR 89, linked to RT PR 210. Reviewed the producer-admitted integration leg for fallible monitor declared-type checking. Static code/spec review is complete. Public Program construction qualification remains blocked before RT entry by the current strict IR operation catalog, owned by IR-1396 / IR PR 946.

## Verdict

**FAIL: required genuine qualification remains open.** No aggregate pass or completed IR-1400 is claimed.

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
| FND-001 | high | Required genuine constructor qualification has not executed: strict IR catalog admission fails before allocator arming and Program::try_new. Public first-Layout Membership/request/retry, later type-walk, RT-scratch Storage and semantic-locus cells remain unverified; the passing component projection test cannot discharge them. | tests/monitor_program.rs:7093-7210 |

## Verification

Read the actual focused logs supplied by the coding lane: compile and focused Clippy/format/spec checks pass; native_type_projection_keeps_allocation_and_capacity_requests_distinct passes once. The attempted first_layout_type_allocation_returns_native_membership_then_retries fails in strict IR checked-package admission at operation_catalog.rs:278 on the closed vocabulary word carried_member, before parts() returns and before arm(0) or Program::try_new. The later-declared-type, RT-scratch and semantic-locus cases have not run. No reviewer Cargo command or aggregate gate was executed. No genuine allocator denial, public Membership result, retry or scratch result is credited from the passing projection test.

No production code, identity constructor, identity guard, CI workflow or safety lint is changed. The tests reuse the existing live QSL emission, unchanged strict IR read and source-owned lookup; all preparation and expected request observations occur before arming. One-shot allocator auto-disarm is stated accurately. Capacity controls use actual owner error values and explicitly make no capacity-exhaustion claim. The production mapping moves native StorageRequest values and shallow-clones the Arc-backed NodeKey without allocating a new diagnostic payload. No applicable AssuranceProfile was found under spec/.

## Coverage

Actual quoin matrix reports: no coverage_matrix criteria because the module declares no obligations source. The current quire matrix emits engine metadata without a computed criterion population. Computed criterion traceability is therefore unavailable, not zero coverage and not a PASS. Source inspection confirms five paired TC-098 / IT-018-SC-16 test attributes; this is only a structural observation, not evidence-backed computed tagging. No hand-written matrix was created. Plan completion: not assessed.
