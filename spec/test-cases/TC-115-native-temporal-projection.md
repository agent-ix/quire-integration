---
id: TC-115
title: "Native temporal projection, correspondence and join through genuine owners"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-022
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
---
# TC-115: Native temporal projection, correspondence and join through genuine owners

## Description

Execute every IT-022 scenario, which together realize Contract IR TC-461, with
owner-produced inputs only: QSL-emitted packages, `evaluate_v2` reports,
quire-mltl evaluated and mapped results, and QObs owner views. Expected
correspondence and join bytes are computed by the test from the inputs using
FR-350's encoding rules; no frozen vector, digest or copied owner document is
used.

## Test Procedure

Run the test targets `temporal_bridge_definition` (IT-022-SC-09),
`temporal_mapping_binding` (SC-01), `temporal_projection` (SC-02 to SC-08,
SC-10, SC-11), `temporal_binding_axes` (SC-10 member mutations),
`temporal_join` (SC-11 to SC-13, SC-15) and `temporal_limits` (SC-14). Each
test carries the FR-350 criterion trace tags of the scenario it implements.
Assert complete typed owner values and typed refusals, never only an error
string or a trace tag. Run the IT-020 formula and IT-021 observation revision
regressions.

## Expected Results

Every IT-022 scenario passes. Each FR-350 criterion FR-350-AC-1 to AC-6 and
AC-8 to AC-11 is asserted by at least one tagged test. Every refusal leaves no
partial projection or join, each limit admits at its measured count and
refuses one below it, and no bridge operation invokes an evaluator.
