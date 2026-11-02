---
id: IT-018
title: "Retain monitor construction parts with producer-origin identities"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-runtime/interface_002
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_003
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-012
    type: verifies
---
# IT-018: Retain monitor construction parts with producer-origin identities

## Objective

Exercise the RT monitor construction boundary while preserving the producer ownership
of semantic identity. These are controlled executable-part construction and ordinary
initializer tests. They do not qualify a temporal plan producer, temporal entry
semantics, or the complete FR-276 monitor.

## Target Integration

Live QSL source emission → IR strict reader → retained structural identity → shared
value/type declarations → RT `monitor::Program::try_new` → bounded ordinary initializer.

## Preconditions

Run the emitter and reader selected by Cargo.lock. Missing tools, refused source,
refused package, missing declarations, and incompatible dependencies fail the target.
No checked-package bytes or derived identity values are copied as fixtures. RT retains
its dependency boundary and does not import QSL or IR.

## Inputs

Author a source containing the declaration identities needed by the construction
controls. Obtain every key from its uniquely selected admitted declaration. Construct
controlled ordinary expression arenas and shared nominal values, then vary references,
types, ownership, source locations and exact premises. These controlled arenas are not
represented as the temporal program emitted by QSL. Their ordinary diagnostic records
use `born` and `amount` fields and do not instantiate FR-487 activation/result records.
A genuinely admitted derived sequence key supplies the nested-key storage probe; its
placement in the controlled layout grants no temporal-root admission or execution credit.

## Test Procedure

1. Drop construction preparation after successful retention. IT-018-SC-01: all four
   entry associations, parameter/result types, expression locations, shared declarations,
   initial values and independently selected metadata remain available through the owner.
2. Vary exact premises. IT-018-SC-02: width 64, nonnegative activation capacity including
   zero and values above host width, and admitted affine/negative-scale Time units retain
   their values; invalid widths, out-of-width gaps, negative uncertainty/capacity and a
   foreign dimension refuse construction. Retain the required final Integer
   event_position with initial literal value 0 under QSpec #595, including after
   two ordered buffers. Missing, Boolean/text, optional, extra or misordered State
   fields, noncanonical buffer names/types and buffer-count mismatches refuse at
   State before later entry-signature checks. A complete otherwise valid closure
   without event_position also refuses; no compatibility branch supplies it.
   These are construction observations, not event-count execution acceptance.
3. Vary executable edges and argument ownership. IT-018-SC-03: duplicate keys/locations,
   unknown addresses, cycles, foreign arguments and missing or mistyped fourth entries
   return the expected typed refusal before execution. Shared DAGs and distinct inline
   locations remain valid; a 20,000-level inline DAG validates on a 512 KiB native stack.
4. Share compatible entry functions. IT-018-SC-04: equivalent event/watermark signatures
   may retain one common function address without inventing another semantic key.

5. Execute ordinary diagnostic initializers. IT-018-SC-05: two builds retain the
   same immutable owner while owning distinct state replacement slots. Zero work,
   stack or operand limits stop with the matching resource. An impossible scratch
   reservation preserves state, owner and the old budget; successful replacement
   changes only operating limits. A lazy branch does not invoke its unselected
   recursive call. A 2,000-level lazy chain executes on a 512 KiB native stack,
   while a smaller explicit continuation budget stops it. A Boolean initializer
   body returns a typed invariant with its native invalid-membership cause and
   original source key retained. These controls cover the initial executor interface only;
   they do not qualify temporal on_event, on_watermark or on_gap semantics.

## Expected Results

The complete target passes every construction control against the lock-selected RT
candidate. Each control retains its independent positive or refusal oracle. No passing
construction test is reported as real-producer temporal execution or full FR-276 credit.
