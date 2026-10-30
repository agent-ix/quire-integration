---
id: TC-096
title: "Observe runtime identities from a QSL-admitted package"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-016
    type: references
  - target: ix://agent-ix/quire-contract-runtime/FR-012
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-007
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-273
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-277
    type: verifies
---
# TC-096: Observe runtime identities from a QSL-admitted package

## Description

Execute IT-016 with live QSL checked-package V2 output and IR-admitted
structural identities. This is the producer-origin replacement for the
identity-dependent rows moved from RT TC-033, TC-201, TC-203, TC-025/TC-026 and
seven TC-194 cases.

## Test Procedure

Run the locked producer and reader, then execute IT-016-SC-01 through SC-05.
Tag each implementation test with its moved RT criterion and record the
corresponding symbol in the row mapping. Compare each result, refusal, meter
state and charge order to the named RT criterion; retain the RT controls that
do not depend on producer identity.

## Expected Results

All five IT-016 success criteria pass, and every moved row has an executed
symbol with the original observable oracle. A skipped producer, invented
semantic key, copied package bytes, or removed criterion fails this case.
