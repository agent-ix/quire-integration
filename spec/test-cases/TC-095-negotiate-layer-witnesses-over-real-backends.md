---
id: TC-095
title: "Negotiate layer witnesses over the real backends"
type: TC
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-345
    type: verifies
  - target: ix://agent-ix/quire-integration/IT-015
    type: references
---
# TC-095: Negotiate layer witnesses over the real backends

## Description

Execute every step of [IT-015](../integration/IT-015-negotiate-layer-witnesses-over-real-backends.md)
against the real backend registry and quire-contract-codegen's `negotiate_*`.

## Test Procedure

Call `xtask::refinement::layering::compare` over `REAL_CORPUS` with the real
negotiation function and require each of the five layers' witness item to settle
`supported` under at least one candidate. Call it again with no negotiation
function and require identical case and edge results. Seed an unsupported
witness for one layer and remove another layer's `witness` entry in separate
corpus copies, and require the regression and the tool failure IT-015 states.

## Expected Results

The real-backend run gives verdict success with five `holds` witness results.
The seeded unsupported witness gives a `regression` naming the layer and each
candidate's identity and disposition. The removed entry gives a tool failure.
A skipped negotiation call, an unavailable registry or an unsupported layer
witness in the unseeded run fails the test.
