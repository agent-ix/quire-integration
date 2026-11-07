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
   independently; each returns `tl-projection-binding`.
2. IT-022-SC-02 (FR-350-AC-9): Each failure uses exactly the code and premise
   FR-350's binding steps assign. With `request.occurrence` absent, a
   single-occurrence subject projects and a two-occurrence subject returns
   native-premise `occurrence`. A `ClauseOccurrence` with a wrong node, role
   or ordinal, or a request occurrence key unequal to it, returns
   native-premise `occurrence`; a control outside the subject's requirements
   returns native-premise `control`. A wrong `request.binding` returns
   `tl-projection-native-clock`; a counter clock cover returns native-premise
   `clock-cover`; a request correspondence reference other than the bridge
   definition's identity and authority returns native-premise
   `correspondence-reference`. An absent `request.anchor` returns incomplete
   `anchor`; a trigger with no admitted delivery or origin returns incomplete
   `instance`; an instance anchor unequal to `request.anchor`, and an empty
   capture instance, return native-premise `anchor`. Use an opaque
   `request.anchor` spelled like another position's stamp and require the
   numeric anchor of the position whose observation identity equals it. An
   observation reference of another bundle returns native-premise
   `observation-revision`; an absent observation identity returns incomplete
   `observation`; a duplicate bundle observation identity returns
   native-premise `observation`; a position record of another clock identity
   or clock revision returns native-premise `position`; a stamp gap returns
   incomplete `position`; a timestamped range returns native-premise `clock`.
3. IT-022-SC-03 (FR-350-AC-6, AC-9): From admitted `ValuationEvidence`, project
   true and explicit false rows and compare them in the TL trace or history.
   For each `ValuationCause` supply one failing row and require
   `tl-projection-valuation` with that cause and row index; a row with both a
   foreign operand and an out-of-range position returns
   `OperandNotInFormula`, the owner's check order (occurrence, operand,
   position, anchor, capture instance, revision). Two rows with one
   `(position, operand)` and the same value project that value, and the
   record keeps both rows; two such rows with opposite values, and an omitted
   row, return incomplete `valuation{position, operand}`. Request shared and
   instance-bound valuation maps, including `Missing`, `Undefined` and
   aggregate samples, that contradict the rows leave the projection unchanged.
4. IT-022-SC-04 (FR-350-AC-1): Project a future-only formula under complete
   and open surrounding closure and require the closed-trace and online-prefix
   requests. Project each bounded past operator of QSpec FR-092 alone under
   both false-extension profiles with origin-complete history. Repeat with a
   range start of 1 and with `authoritative_origin` `Some(false)` and `None`;
   require incomplete `history`. Every generated artifact equals its owner
   reader's output. When declarations share a checked node, the request's
   subject identity is the selected occurrence's name.
5. IT-022-SC-05 (FR-350-AC-2, AC-3): Unsupported classifications
   (timestamped-event finite window, mixed past and future, infinite trace)
   return `tl-projection-unsupported` with the complete classification. A
   formula with an aggregate atom or a non-`holds` leaf returns
   `tl-projection-formula`; an integer interval without an upper bound or
   with a bound above `u32::MAX` returns native-premise `interval`.
6. IT-022-SC-06 (FR-350-AC-4): Project a fixed-sample subject with a
   nanosecond unit whose epoch and period equal the QObs anchor. Change only
   the period with equal formula bytes, and reinterpret samples as event
   counts; each returns native-premise `clock`. Omit a sample and require
   incomplete `position`. With a millisecond unit, a future target returns
   native-premise `clock-representation` and a past target returns
   `tl-projection-owner` carrying quire-mltl FR-001-AC-9's
   `ExpectedMismatch("clockBinding")`; no unit is converted.
7. IT-022-SC-07 (FR-350-AC-1, AC-9): Supply independent QObs clock, decision
   and surrounding progress and closure, completeness and availability views.
   Closures compare by state and progress by the decimal watermark. An absent
   decision closure returns incomplete `complete-input`; differing decision
   closure, surrounding closure and progress return native-premise
   `decision-closure`, `surrounding-closure` and `progress`; absent or
   incomplete completeness returns incomplete `complete-input`; contradicted
   completeness returns native-premise `completeness`. No result reader is
   invoked.
8. IT-022-SC-08 (FR-350-AC-9): Before any TL artifact, the native
   `ActivationInput::contradiction` admission runs against the exact
   obligation and ledger; a changed payload under a retained trigger returns
   native-premise `activation`. A redelivery uses the retained payload and
   capture environment. A retained `Missing` fault returns incomplete
   `activation` and a `Refused` fault native-premise `activation`; missing,
   null and stale captures and an evicted capture return incomplete
   `capture`; wrong-type and other-anchor captures return native-premise
   `capture`. The source-map entry is selected by node, role and ordinal; two
   declarations sharing a node select different entries, and zero matches
   return native-premise `source`.
9. IT-022-SC-09 (FR-350-AC-3, AC-11): `read_tl_bridge_definition` admits the
   authored record; changed authority, identity, profile and version each
   refuse; unknown, duplicate, missing, trailing and non-canonical members
   refuse; the exact record length as `max_bytes` admits and one less refuses.
10. IT-022-SC-10 (FR-350-AC-3, AC-10, AC-11): Compute the expected
    correspondence record from the inputs by FR-350's member table and
    encodings and require byte equality, with no JSON number, for a future, a
    past and a fixed-sample projection. Mutate each member's source input
    alone; exactly that member changes and `read_tl_projection` of the
    original bytes returns `tl-projection-correspondence-mismatch`. The
    `models` member equals `selected_model_packages()` in ascending order, and
    two packages selecting the same models in different source orders yield
    an identical member. The record's request bytes equal the owner-read
    request and contain no record bytes; a wrong subject or profile name
    refuses.
11. IT-022-SC-11 (FR-350-AC-6): For projection and join bytes, add an unknown
    member, duplicate, remove and reorder members, add whitespace and a
    trailing byte; each returns `tl-projection-correspondence-mismatch` or
    `tl-join-expected-mismatch` with no partial value.
12. IT-022-SC-12 (FR-350-AC-5): From genuine native and TL results reach each
    FR-350 row R1 to R8 with its decision, comparison, causes and Boolean,
    including an equal-final pair whose premises differ and a final/pending
    pair (R6), a TL `Failed` result (R2), a native `InvalidPredecessor` (R6),
    and a failed native execution with an evaluated Boolean report (R2, not
    R7). For each pair of rows R1 to R6 build one pair satisfying both and
    require the first decision and both causes once in row order. The mapped
    result is re-read with `MappingSelection::for_result`. Compute the
    expected join record by FR-350's rules and require byte equality;
    `read_tl_join` with any other expected input returns
    `tl-join-expected-mismatch`.
13. IT-022-SC-13 (FR-350-AC-5, AC-10): This scenario requires the owner
    operation quire-mltl
    `ValidatedTemporalResult::validate_supplied_predecessor` with its public
    `direct_predecessor_bytes` (quire-mltl FR-001). Advance a real QObs
    progress watermark from 1 to 2 over one carrier to create genuine native
    and TL corrections; first show a relation-only change is rejected by the
    native owner. With `previous` holding both direct predecessors, the
    correction reaches R7. A one-sided correction, an original with
    `previous`, a correction without `previous`, a previous join whose native
    side was not admitted, a native `InvalidPredecessor`, and TL
    `missing-predecessor`, `unexpected-predecessor` and
    `predecessor-mismatch` each reach R6 with no Boolean. A different complete
    predecessor with equal result identity on each side reaches R6 and fails
    `read_tl_join`. A foreign corrected native report keeps its earlier
    `BindingMismatch` (R1). A native `PredecessorLimit` at one below its
    measured charge returns `tl-join-limit-reached` with the owner's resource
    and attempted charge, and an already charged `NativeJoinBudget` is never
    reset.
14. IT-022-SC-14 (FR-350-AC-8): `TlLimits::default()` equals FR-350's seven
    settings and defaults (`temporal.correspondence.max_bytes` 16,777,216,
    `max_nodes` 65,536, `max_positions` 1,048,576, `max_valuations`
    16,777,216, `max_support` 1,048,576, `max_causes` 64, `max_work`
    4,294,967,296). For each limit and each operation that charges it under
    FR-350 (projection: bytes, nodes, positions, valuations, work; join:
    bytes, support, causes, work; each strict reader: bytes), the test
    computes the expected count `n` from the inputs by FR-350's Limits table,
    without reading the bridge's usage. Setting `n` admits with reported usage
    `n`, and setting `n - 1` returns `limit-reached{limit, value: n, setting:
    n - 1}` with the rejected unit unvisited and no partial value. Two limits
    crossed by one action return only the first in FR-350's table order. An
    owner refusal at a byte ceiling lowered to `max_bytes` is `max_bytes`
    limit-reached with `value: setting + 1` and the owner error.
15. IT-022-SC-15 (FR-350-AC-6): After each join, the retained native report
    equals the supplied report and the retained TL source result equals the
    supplied one with unchanged owner usage; no bridge operation has a call
    path to `evaluate_v2`, `quire_mltl::report::evaluate` or
    `quire_mltl::report::read`.
16. IT-022-SC-16 (FR-350-AC-2, AC-6, AC-9): For each case below, supply one
    input exhibiting only that failure and require exactly the stated code
    and premise (NP = `tl-projection-native-premise`, INC =
    `tl-projection-incomplete`) with no TL artifact:
    - Admission (step 1): subject of another admission, classification of
      another admission, reclassification mismatch: `tl-projection-binding`;
      `Unsupported` classification: `tl-projection-unsupported`.
    - Occurrence (step 2): `arena_of` failure, request occurrence key unequal
      to the occurrence, absent key with zero or several occurrences: NP
      `occurrence`; control not among the requirements: NP `control`; absent
      `name_of(occurrence)`: NP `subject-name`.
    - Binding and cover (step 3): `TraceBinding::authenticate` failure:
      `tl-projection-native-clock`; cover other than `Exact`: NP
      `clock-cover`.
    - Correspondence reference (step 4): wrong identity, wrong authority
      identity: NP `correspondence-reference`.
    - Anchor, instance and captures (step 5): absent `request.anchor`: INC
      `anchor`; `request.anchor` unequal to the instance anchor, empty capture
      instance: NP `anchor`; trigger evidence not `Admitted`, absent trigger:
      INC `instance`; activation contradiction: NP `activation`; retained
      `Missing` fault: INC `activation`; retained `Refused` fault: NP
      `activation`; retained capture count unequal to the declared captures:
      INC `capture`; retained capture with another name, retained capture at
      another anchor: NP `capture`; no matching origin or guarded delivery:
      INC `instance`; delivery anchor unequal to the instance anchor: NP
      `anchor`; delivered capture count mismatch, `Missing`, `Null` and
      `Stale` captures: INC `capture`; `WrongType` capture, capture value at
      another anchor: NP `capture`; eviction of a selected capture: INC
      `capture`; eviction of a valuation of the selected instance, eviction
      of an unattributed valuation: INC `complete-input`.
    - Observation (step 6): observation reference or revision unequal to the
      bundle's: NP `observation-revision`; two bundle records with one
      observation identity: NP `observation`; no record for a position's
      observation identity: INC `observation`; record of another clock
      identity, record of another clock revision, stamp above the
      completeness `through`: NP `position`; owner anchor of the wrong form
      (event-position sequence authority or position mismatch, fixed-sample
      index mismatch, nanosecond epoch or period mismatch): NP `clock`; epoch
      or period rational with denominator other than 1: NP
      `clock-representation`; non-nanosecond unit for a future target: NP
      `clock-representation`; non-nanosecond unit for a past target:
      `tl-projection-owner`; `request.anchor` matching no position: INC
      `anchor`; matching several positions: NP `anchor`.
    - Positions (step 7): timestamped range: NP `clock`; range bound that is
      not a decimal `u64`: NP `clock-representation`; stamp outside
      `[start, end)`, repeated stamp: NP `position`; range length unequal to
      the position count, gap in ascending stamps: INC `position`; past
      target with range start other than 0, `authoritative_origin` other than
      `Some(true)`: INC `history`.
    - Progress, closure and completeness (step 8): absent decision closure:
      INC `complete-input`; differing decision closure: NP `decision-closure`;
      surrounding closure unequal to the supplied closure, unequal to the
      QObs view: NP `surrounding-closure`; absent progress, differing
      progress, unparsable watermark, timestamped boundary: NP `progress`;
      absent or `Incomplete` request completeness, QObs completeness
      `Incomplete`: INC `complete-input`; QObs completeness `Contradicted`: NP
      `completeness`.
    - Valuations (step 9): each of the six `ValuationCause` values:
      `tl-projection-valuation` with that cause and row index; disagreeing
      rows, missing row: INC `valuation{position, operand}`.
    - Construction: timed interval, integer interval without upper bound,
      bound above `u32::MAX`: NP `interval`; non-`holds` atom and formula
      constructor refusal: `tl-projection-formula`; owner producer or reader
      refusal: `tl-projection-owner`; zero or several matching source-map
      entries: NP `source`; an IR value with a non-integer number:
      `tl-projection-encoding`.

## Expected Results

Every scenario passes with genuine owner values. Each supported projection
returns owner-admitted artifacts and the exact correspondence record; each
failed premise returns its FR-350 typed error with no partial value. Every
join row, cause order, correction outcome and strict-read refusal matches
FR-350. Each of the seven limits admits at its measured count and refuses one
below it with the named cause. No bridge operation invokes an evaluator.

## Retained values and error detail

These public items carry owner detail beside the FR-350 outputs and add no
decision input. Each is exercised by the scenarios named.

- `ValidatedTlProjection::source_positions()` returns, for each TL position in
  ascending stamp order, the index of its entry in `request.positions`
  (IT-022-SC-02, IT-022-SC-10).
- `ValidatedTlJoin::owner_refusals()` returns the typed native strict-read or
  predecessor refusal and the typed TL mapped-result re-read refusal that put
  a side into the `refused` or `lineage` state (IT-022-SC-12, IT-022-SC-13).
- `ValidatedTlJoin::native_encoding_usage()` and `native_predecessor_usage()`
  return the native owner's own usage receipts for the bounded writer and the
  full predecessor validation. They are not bridge counters and never enter
  `TlLimits` accounting (IT-022-SC-13, IT-022-SC-14).
- `TlProjectionError` variants other than the eleven FR-350 codes' primary
  payloads (`PropositionMap`, `SyntaxRead`, `SyntaxEncoding`, `History`,
  `ExactNumber`) carry the typed tl-syntax or tl-mltl constructor or reader
  refusal and map to `tl-projection-owner`; `NonIntegerNumber` and
  `Canonical` map to `tl-projection-encoding` (IT-022-SC-16).

## Checks verified at unit level

QObs's capture selection (quire-observation `src/authority/capture.rs:206-251`)
covers every qualified record at one anchor, and a bundle holds each role once
(`src/authority/bundle.rs:1294-1324`). A genuine bundle therefore carries a
single stamp and cannot hold one observation identity twice. The bridge's
stamp ordering, repeated stamp, gap and range-length checks, the several-anchor
match, the duplicate bundle observation identity, the numeric anchor offset and
the unique source-map entry are defensive for owner input. They are pure
functions over typed identities and stamps (`projection/positions.rs`), verified
by unit tests there rather than through a bundle. The refusals that need an
input no QSL source or QObs producer yields today (`subject-name`,
`tl-projection-encoding` for a non-integer IR number) are verified at the
function that produces them.
