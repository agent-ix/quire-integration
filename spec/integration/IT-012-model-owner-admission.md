---
id: IT-012
title: "Admit a QSL-emitted package with a content-only ModelOwner"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-094
    type: verifies
---
# IT-012: Admit a QSL-emitted package with a content-only ModelOwner

## Objective

Show that the checked package QSL emits for a unit that selects a domain-package
model is admitted by contract IR's checked-package reader, and that each model
node is keyed by the content-only `ModelOwner`, which carries no version.

## Target Integration

QSL parse → forms → domain-package intake → assemble → check → link → emit
`quire.checked-package/v2` bytes → contract IR `read_checked_package`.

## Preconditions

None beyond a local build. The domain package document is QSL's authored
`tests/fixtures/spine-model.semantic-ir.json`, read from the resolved
`quire-spec-language` checkout.

## Inputs

A unit selecting `acme/orders` and using two of its object types, with the
domain package document supplied to IR's reader as evidence under its
`sha256-jcs` digest.

## Test Procedure

1. Emit the package and read its bytes through IR's reader.
   IT-012-SC-01: the reader admits the package, its lock names the selected
   domain package, and a document supplied under another digest is refused.
2. Compute the id of each model node from its structural preimage, whose owner
   is exactly `{kind, identity, node}`.
   IT-012-SC-02: the ids of the model nodes IR admitted are exactly those.
3. Relabel the domain package document, the unit's selection and the evidence
   to another version, and repeat.
   IT-012-SC-03: the lock's selection differs and the model node ids are the
   same.

## Expected Results

Every step uses QSL's real emitter bytes and IR's real reader. A refused
package, an owner carrying a version, or model node ids that change with the
domain package version fails the test.
