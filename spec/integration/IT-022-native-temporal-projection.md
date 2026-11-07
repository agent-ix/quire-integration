---
id: IT-022
title: "Project, record and join native temporal results through genuine TL owners"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
  - target: ix://agent-ix/quire-integration/IT-020
    type: depends_on
  - target: ix://agent-ix/quire-integration/IT-021
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-043
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-052
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-516
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-527
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-001-AC-11
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-001-AC-12
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-350-AC-9
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-350-AC-11
    type: depends_on
---
# IT-022: Project, record and join native temporal results through genuine TL owners

## Objective

Qualify the complete `quire.native-temporal-correspondence/v1` bridge of
Contract IR FR-350 in this repository: bridge-definition admission, projection,
correspondence strict reading, result joining, join strict reading,
corrections and all seven bridge limits. FR-350 is the normative contract for
inputs, construction order, record members and encoding, join rows, error
codes and limits; this IT composes it with the genuine owner APIs and does not
restate it. Every scenario below is required for FR-350 qualification.

## Target Integration

Original QSL source and model evidence → QSL emitter → strict IR reader →
checked subject and classification → native `EvidenceRequest`,
`ValuationEvidence`, occurrence, instance and observation reference →
IT-021 admitted observation revision and QObs views → IT-020 formula
constructor → tl-syntax and tl-mltl constructors and readers → quire-mltl
request derivation and reading → correspondence record.
Separately, QSL `evaluate_v2` produces the native report and quire-mltl
evaluation and mapping produce the `ValidatedMappedResult`; `join_tl` admits
both through their non-evaluating owner readers and writes the join record.
No bridge operation invokes an evaluator.

## Preconditions

Every fixture value is produced by its genuine owner: the checked package by
the QSL emitter and IR reader, the native report by `evaluate_v2`, TL results
by quire-mltl `report::evaluate`, `report::read`, `contract_ir::map` and
`contract_ir::read`, and QObs views by their owner readers. No test constructs
a substitute owner result, a copied owner schema or a fake identity. The
integration owns `definitions/native-temporal-correspondence-v1.json` with
authority `agent-ix/quire-integration`, identity
`native-temporal-correspondence/v1`, profile
`quire.native-temporal-correspondence/v1` and version 1, all authored literal
values.

## Inputs

The operations and their inputs are exactly those of Contract IR FR-350's
Boundary and interface section: `read_tl_bridge_definition`,
`project_tl` with `NativeProjectionInput { request, valuations, occurrence,
instance, observation }` and `TlProjectionSelection { bridge_definition,
observation, clock, observations, owner_limits, native_report_limits }`,
`read_tl_projection`, `join_tl` and `read_tl_join` with the caller's
`NativeJoinBudget`, and `TlLimits`. Valuations come only from the admitted
`ValuationEvidence` rows; `EvidenceRequest.positions` valuation and aggregate
maps are not read. A clock is identified by the QObs clock view's clock
identity and clock revision.

## Test Procedure

1. IT-022-SC-01 (FR-350-AC-2): Emit source once and admit the bytes twice.
   A local subject and classification project. Substitute the other
   admission's subject, then its classification, and require
   `tl-projection-binding` before formula construction. A package clone
   retaining the admission, including after a move, projects. Alter support,
   a retained field, the definition selection and the surrounding closure
   independently; each refuses with `tl-projection-binding`.
2. IT-022-SC-02 (FR-350-AC-9): Authenticate `request.binding` against the
   selected subject. With `request.occurrence` absent, a single-occurrence
   subject projects and a two-occurrence subject refuses. Supply a
   `ClauseOccurrence` with a wrong node, role and ordinal, a request occurrence
   key unequal to it, and a control outside the subject's requirements; each
   refuses with `tl-projection-native-premise` premise `occurrence`. A
   request trigger with no admitted delivery or origin returns
   `tl-projection-incomplete` with missing premise `instance`. An instance
   anchor unequal to `request.anchor`, and an empty capture instance, each
   refuse with `tl-projection-native-premise` premise `anchor`. Use an
   opaque `request.anchor` spelled like another
   position's stamp and require the numeric anchor of the position whose
   observation identity equals it. Absent, duplicate and foreign observation
   identities, and a position record with another clock identity or clock
   revision, each refuse with their typed premise.
3. IT-022-SC-03 (FR-350-AC-6, AC-9): From admitted `ValuationEvidence`, project
   true and explicit false rows and compare them in the TL trace or history.
   For each `ValuationCause` (occurrence, position, anchor, capture instance,
   revision, operand) supply one failing row and require
   `tl-projection-valuation` with that cause and row index. A duplicate is two
   rows with equal `(position, operand)`; supply one with equal and one with
   opposite values and require the duplicate refusal naming both row indices.
   Omit one required row and require `tl-projection-incomplete`. Add request
   shared and instance-bound valuation maps, including `Missing`, `Undefined`
   and aggregate samples, that contradict the rows and require an unchanged
   projection.
4. IT-022-SC-04 (FR-350-AC-1): Project a future-only formula under complete
   and open surrounding closure and require the closed-trace and online-prefix
   requests. Project each bounded past operator of QSpec FR-092 alone under
   both false-extension profiles with origin-complete history. Repeat with a
   range start of 1 and with `authoritative_origin` `Some(false)` and `None`;
   require `incomplete` with missing premise `history`. Every generated
   artifact equals its owner reader's output. When declarations share a checked
   node, the request's subject identity is the selected occurrence's name.
5. IT-022-SC-05 (FR-350-AC-2, AC-3): Unsupported classifications
   (timestamped, mixed past and future, infinite trace) return the complete
   classification and no projection. A formula with an aggregate atom or a
   non-`holds` leaf returns `tl-projection-formula` and no artifact.
6. IT-022-SC-06 (FR-350-AC-4): Project a fixed-sample subject whose epoch,
   period and unit equal the QObs anchor. Change only the period with equal
   formula bytes, and reinterpret samples as event counts; each refuses.
   Omit a sample and require `incomplete`. Select an authored unit for a past
   fixed-sample request that quire-mltl FR-001-AC-9 does not admit and require
   the owner's typed `ExpectedMismatch("clockBinding")` refusal with no
   converted value. This refusal case is evidence for the refusal clause of
   FR-350-AC-4 only; the positive clauses require their own passing cases.
7. IT-022-SC-07 (FR-350-AC-1, AC-9): Supply independent QObs clock, decision
   and surrounding progress and closure, completeness and availability views.
   Each disagreement with the native request refuses with its premise
   (`decision-closure`, `surrounding-closure`, `progress`, `completeness`);
   incomplete completeness is `incomplete`. No result reader is invoked.
8. IT-022-SC-08 (FR-350-AC-9): Before any TL artifact, the native
   `ActivationInput::contradiction` admission runs against the exact obligation
   and ledger. A changed payload under a retained trigger refuses. A
   redelivery uses the retained payload and capture environment. Missing,
   null, stale, wrong-type and other-anchor captures, a retained `Missing` or
   `Refused` fault and an evicted capture each return their typed result. The
   source-map entry is selected by node, role and ordinal; two declarations
   sharing a node select different entries.
9. IT-022-SC-09 (FR-350-AC-3, AC-11): `read_tl_bridge_definition` admits the
   authored record; changed authority, identity, profile and version each
   refuse; unknown, duplicate, missing, trailing and non-canonical members
   refuse; the exact record length as `max_bytes` admits and one less refuses.
10. IT-022-SC-10 (FR-350-AC-3, AC-10, AC-11): Compute the expected
    correspondence record from the inputs by FR-350's member table and
    encoding rules and require byte equality for a future, a past and a
    fixed-sample projection. Mutate each member's source input alone; exactly
    that member changes and `read_tl_projection` of the original bytes
    refuses. Reorder selected model identities and require a different
    `models` member. The record's request bytes equal the owner-read request
    and contain no record bytes; a wrong subject or profile name refuses.
11. IT-022-SC-11 (FR-350-AC-6): For projection and join bytes, add an unknown
    member, duplicate, remove and reorder members, add whitespace and a
    trailing byte; each refuses with no partial value.
12. IT-022-SC-12 (FR-350-AC-5): From genuine native and TL results reach each
    FR-350 row R1 to R8 with its decision, comparison, causes and Boolean. For
    each pair of rows R1 to R6 build one pair satisfying both and require the
    first decision and both causes once in row order. A failed native
    execution with an evaluated Boolean report reaches R2, not R7. Compute the
    expected join record by FR-350's rules and require byte equality;
    `read_tl_join` with any other expected input refuses.
13. IT-022-SC-13 (FR-350-AC-5, AC-10): This scenario requires the owner
    operation quire-mltl `ValidatedTemporalResult::validate_supplied_predecessor`
    with its public `direct_predecessor_bytes` (quire-mltl FR-001). Advance
    a real QObs progress watermark
    from 1 to 2 over one carrier to create genuine native and TL corrections;
    first show a relation-only change is rejected by the native owner. With
    `previous` holding both direct predecessors, the correction reaches R7.
    A one-sided correction, an original with `previous`, a correction without
    `previous`, a previous join whose native side was not admitted, a native
    `InvalidPredecessor`, and TL `missing-predecessor`,
    `unexpected-predecessor` and `predecessor-mismatch` each reach R6 with no
    Boolean. A different complete predecessor with equal result identity on
    each side reaches R6 and fails `read_tl_join`. A foreign corrected native
    report keeps its earlier `BindingMismatch` (R1). A native
    `PredecessorLimit` at one below its measured charge returns
    `tl-join-limit-reached` with the owner's resource and attempted charge,
    and an already charged `NativeJoinBudget` is never reset.
14. IT-022-SC-14 (FR-350-AC-8): `TlLimits::default()` equals FR-350's seven
    settings and defaults (`temporal.correspondence.max_bytes` 16,777,216,
    `max_nodes` 65,536, `max_positions` 1,048,576, `max_valuations`
    16,777,216, `max_support` 1,048,576, `max_causes` 64, `max_work`
    4,294,967,296). For each limit and each operation that charges it under
    FR-350 (projection: bytes, nodes, positions, valuations, work; join:
    bytes, support, causes, work; each strict reader: bytes), read the
    measured usage `n` at ample settings; setting `n` admits and setting
    `n - 1` returns `limit-reached{limit, value: n, setting: n - 1}` with the
    rejected unit unvisited and no partial value. Two limits crossed by one
    action return only the first in FR-350's table order. An owner refusal at
    a byte ceiling lowered to `max_bytes` is `max_bytes` limit-reached.
15. IT-022-SC-15 (FR-350-AC-6): After each join, the retained native report
    equals the supplied report and the retained TL source result equals the
    supplied one with unchanged owner usage; no bridge operation has a call
    path to `evaluate_v2`, `quire_mltl::report::evaluate` or
    `quire_mltl::report::read`.

## Expected Results

Every scenario passes with genuine owner values. Each supported projection
returns owner-admitted artifacts and the exact correspondence record; each
failed premise returns its FR-350 typed error with no partial value. Every
join row, cause order, correction outcome and strict-read refusal matches
FR-350. Each of the seven limits admits at its measured count and refuses one
below it with the named cause. No bridge operation invokes an evaluator.
