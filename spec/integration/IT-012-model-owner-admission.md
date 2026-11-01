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
declared `acme/orders` identity. The emitted graph retains that selection in
a `ModelRoot` owner.

## Test Procedure

1. Emit the package and read its bytes through IR's reader.
   IT-012-SC-01: the reader admits the package, its model-root owner names the
   selected domain package, and a document supplied under another identity is
   refused as a missing import at the selected root owner's identity locus.
2. Read each admitted model declaration's structural node identity and its
   exact `{kind, identity, node}` model owner, excluding the package root.
   IT-012-SC-02: the owners name the selected domain identity and the exact
   Gadget and Widget declarations, with no version member.
3. Change only the authored domain document's package version and repeat the
   same identity-based unit selection and reader evidence construction.
   IT-012-SC-03: the selected model-root identities and all model declaration
   node identities remain equal across the two document versions.

## Expected Results

Every step uses QSL's real emitter bytes and IR's real reader. A refused
package, an owner carrying a version, or model node ids that change with the
domain package version fails the test.
