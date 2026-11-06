---
id: IT-016
title: "Carry producer-origin identity into exact runtime observations"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-runtime/FR-012
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-007
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-008
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-011
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-273
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-275
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-277
    type: verifies
---
# IT-016: Carry producer-origin identity into exact runtime observations

## Objective

Restore the identity-sensitive observations moved from RT's TC-033, TC-201,
TC-203, TC-025/TC-026 and TC-194 tests. Their semantic node keys must originate
in a QSL-admitted checked package, survive contract IR reading, and designate the
same owners when QSV, quire-exact and RT consume them. The test must retain each
value, refusal, charge and ordering oracle named in IR-1175; moving a fixture
does not retire its acceptance criterion.

## Target Integration

QSL source and its runnable checked-package V2 producer → emitted bytes →
contract IR checked-package reader → admitted declaration, member, unit,
expression and function identities → QSV / quire-exact / RT public entry points.
The source of each semantic key is the admitted package, including for negative
tests whose *operation* is malformed after admission.
Unit identity controls may instead use QSL's public admitted context from the
same real producer run. This retained-context route includes checked but unused
unit declarations; it is distinct from a claim that their nodes occur in the
emitted bytes or pass the strict reader.

## Preconditions

Use the QSL checkout selected by this repository's Cargo lock and its installed
`emit_checked_v2` example. Emit the checked package during the test from authored
source, then read those bytes with the locked contract IR reader. Refusal or
absence of the producer, reader, or required declaration is a test failure, not
a skipped case. `Cargo.lock` is the canonical record of the selected source
revision. The test report identifies the producer command, application version
and observed outcome, and points to that lock for the source revision; it adds
no separate revision record. Neither package bytes nor derived identity values
are frozen as fixtures.
For retained-context unit controls, compile authored source through QSL's
`compile_admitted` facade and take the checked unit facts from its
`AdmittedContext`. Build the QSV unit graph from those same checked facts. A
strict-reader claim requires a separate emitted-byte check that locates every
root used by that claim in the canonical bytes.

## Inputs

Author source for the unit, enum, quantity, composite and function scenarios
needed by the RT test rows listed below. Read any QSL-owned corpus directly from
its locked checkout. Use admitted structural identities from emitted packages
for strict-reader controls, or from the same QSL compilation's retained
`AdmittedContext` for the checked-unused-unit handoff control. Never construct
semantic node keys from a digest, byte string or constituent parts in an RT
test. Vary malformed operations and meter limits while retaining the real
producer run as the common identity source.

## Test Procedure

1. Emit and read the package, then select the admitted unit declaration, enum
   declaration and members and their owners. Exercise RT's TC-033
   FR-012-AC-4/5, the FR-011-AC-7 unit-graph insertion and malformed-order
   checks, and the owner checks previously in `exact_unit_owner.rs` and
   `exact_vocabulary.rs`.
   IT-016-SC-01: the exact declaration/member owner and unit-graph causes,
   canonical dimension, compound term order and equality results match those
   criteria, including a refused malformed member or term and the stable
   iteration or first cause required by FR-011-AC-7. The retained RT
   FR-012-AC-3/6/7, FR-007-AC-13 and meter-consumed portions of FR-011-AC-7
   remain independently runnable.
2. Exercise the admitted units through the former TC-201 quantity scenarios in
   `exact_quantity_conversion.rs` and `exact_quantity_owner.rs`.
   IT-016-SC-02: FR-007-AC-12/16/17/20/21/22/23/24/25, FR-011-AC-2/4/6
   and FR-275-AC-23 hold for their stated completed values, target owners,
   first typed causes, zero-charge cases, charge order and amounts, denial
   fields, and exact versus non-exact handoff. The independent rational oracle
   and compound-unit second conversion remain checks, not just smoke runs.
3. Apply the former TC-203 and TC-025/TC-026 equality cases using admitted
   composite declarations, enum variants and foreign references.
   IT-016-SC-03: FR-277-AC-1/2/3/4/5/6 retain their exact Boolean results,
   seventeen-pair schedule, typed refusal, capacity denial and charge-prefix
   oracles. FR-008-AC-5/6 retain the foreign-reference universe boundary,
   distinct-owner result and refusal observations, while composites with
   different declarations compare unequal and enums compare by variant.
   RT's separate FR-277-AC-8 source inspection remains in RT.

   QSL-1720 supplies two admitted producer-origin unit routes. A real declared
   root and a derived one-term compound over that same root have distinct
   structural `UnitId` arms, ordered declared before compound; the former
   "equal-byte" wording does not describe an additional current source value.
   A separate 32-root source with a long package name reaches the actual exact
   handoff's oversized `Identity(InputBytes)` refusal through checked unused
   roots retained by `AdmittedContext`. This second route proves the retained
   context and handoff, not strict-reader acceptance of unused nodes. The
   short-name, touched-root control separately proves emitted-byte admission.
   Synthetic identities do not satisfy either route. RT's broader TC-026
   equality matrix and TC-201 quantity matrix retain their own acceptance
   status until the corresponding consumer observations are executed.
4. Apply admitted functions and expressions through the seven moved TC-194
   identity-dependent cases in `exact_function_application.rs`.
   IT-016-SC-04: FR-273-AC-2/3/5/8 retain arity and argument-validation
   precedence, the one-call charge and body-order observations, and distinct
   typed foreign-expression, unknown-function, meter-borrow and reentry
   refusals, with their original no-extra-charge outcomes. The remaining
   TC-194/TC-195 runtime cases remain runnable in RT.
5. For each moved RT test row, bind its criterion tags to the runnable
   integration test symbol, and report the producer command, application
   version and exact result or refusal observed in this run. Identify
   `Cargo.lock` as the source-revision record used for this run.
   IT-016-SC-05: every IR-1175 AC1–5 row has a mapped, executed symbol and
   observable oracle; a missing row, skipped case, copied producer fixture,
   frozen output digest or locally invented semantic key fails the gate.

## Expected Results

The emitted producer identities used by strict-reader controls survive IR
reading; retained-context controls use genuine checked identities from that
same lock-selected producer. All five success criteria hold on the same
lock-selected dependency cohort. A structurally valid but foreign identity, a
wrong result or refusal, a changed charge sequence, or an unavailable producer
fails the test.

## Traceability

TC-096 binds this integration procedure to the implementation. The test modules
are `tests/producer_origin_identity.rs` and
`tests/producer_origin_unit_identity.rs`; their symbols and trace tags are
completed with the implementation and recorded against the moved RT rows.
