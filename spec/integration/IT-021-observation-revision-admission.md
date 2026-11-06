---
id: IT-021
title: "Admit an exact observation revision from a genuine authority bundle"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
---
# IT-021: Admit an exact observation revision from a genuine authority bundle

## Objective

Qualify a partial observation-input prerequisite for FR-350 using genuine owner
values. The component SHALL admit only canonical v1 bundle bytes accepted by the
QObs historical reader and containing the exact independently admitted observation.
It SHALL retain the admitted bundle, selected observation and actual prior receipt.

## Target Integration

Real QObs admission, document derivation, publication and strict readers → integration
`read_observation_revision` → constructor-private `AdmittedObservationRevision`.
Neither temporal evaluator is invoked.

## Preconditions

The caller supplies independent QObs Context, bundle Selection and genuine observation
View. A correction supplies its exact previous component receipt. QObs owns bundle
lineage validation; historical reading establishes neither global current-head nor
absence of unknown forks. An unchanged observation may remain in a corrected bundle.

## Inputs

Canonical bundle bytes, Context, Selection, selected observation View, optional prior
receipt, native QObs Limits and component limits. Defaults are 16,777,216 bytes and
65,536 scan visits. Component bytes count the supplied current bundle and selected
observation bytes before owner reading. Each record and component-stamp scan visit
costs one work unit, charged before inspecting that item. Owner derivation, parsing,
allocation and internal comparisons use native limits and are excluded from component
work claims. Previous receipts are borrowed without recursive copying or traversal.

## Test Procedure

1. IT-021-SC-01: Admit genuine initial and corrected v1 bundles through
   `bundle::read_revision`, supplying the previous receipt's actual bundle View.
   Compare complete returned typed owner values and bytes; retain the exact previous
   receipt. Reject missing, foreign and non-direct predecessors using native errors.
2. IT-021-SC-02: Require exact canonical observation-document bytes in the bundle's
   Observation record and its exact role, identity, contract and revision component
   stamp. Reject foreign selected observations and equal-content envelope revision
   substitutions. Admit a corrected bundle retaining the unchanged observation.
3. IT-021-SC-03: Delegate malformed, noncanonical and trailing-byte refusal to QObs;
   preserve the original typed owner error. Expose stable Copy component codes for
   owner refusal, absent exact observation and limit exhaustion, with no partial receipt.
4. IT-021-SC-04: Check current-input bytes before owner reading and each scan visit
   before inspection. Exact settings admit; one less refuses at setting+1. Each call
   has fresh counters. Byte limits precede owner reads; scan limits precede selection
   acceptance. Preserve all genuine owner values and prior bytes unchanged.

## Expected Results

The public component returns only a partial observation revision receipt. Selected
observation-document lineage, QSL capture/native binding, duplicate valuation evidence,
complete correspondence, native/TL result predecessors, `ValidatedTlJoin`, `join_tl`,
`read_tl_join` and complete FR-350 resource accounting remain Planned. No final Boolean
or agreement is established. This evidence does not complete any whole FR-350 criterion.
