---
id: IT-018
title: "Retain monitor construction parts with producer-origin identities"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-runtime/interface-002
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-012
    type: verifies
---
# IT-018: Retain monitor construction parts with producer-origin identities

## Objective

Exercise the RT monitor construction boundary while preserving the producer ownership
of semantic identity. These are controlled executable-part construction tests. They do
not qualify a temporal plan producer, entry execution, or the complete FR-276 monitor.

## Target Integration

Live QSL source emission → IR strict reader → retained structural identity → shared
value/type declarations → RT `monitor::Program::try_new`.

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
represented as the temporal program emitted by QSL.

## Test Procedure

1. Drop construction preparation after successful retention. IT-018-SC-01: all four
   entry associations, parameter/result types, expression locations, shared declarations,
   initial values and independently selected metadata remain available through the owner.
2. Vary exact premises. IT-018-SC-02: width 64, nonnegative activation capacity including
   zero and values above host width, and admitted affine/negative-scale Time units retain
   their values; invalid widths, out-of-width gaps, negative uncertainty/capacity and a
   foreign dimension refuse construction.
3. Vary executable edges and argument ownership. IT-018-SC-03: duplicate keys/locations,
   unknown addresses, cycles, foreign arguments and missing or mistyped fourth entries
   return the expected typed refusal before execution. Shared DAGs and distinct inline
   locations remain valid; a 20,000-level inline DAG validates on a 512 KiB native stack.
4. Share compatible entry functions. IT-018-SC-04: equivalent event/watermark signatures
   may retain one common function address without inventing another semantic key.

## Expected Results

The complete target passes every construction control against the lock-selected RT
candidate. Each control retains its independent positive or refusal oracle. No passing
construction test is reported as real-producer temporal execution or full FR-276 credit.
