---
id: TC-098
title: "Monitor construction with admitted semantic identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-018
    type: verifies
---
# TC-098: Monitor construction with admitted semantic identities

## Description

Verify IT-018's construction controls through the public RT construction API.

## Test Procedure

Run the complete `monitor_program` integration target. Emit authored QSL source,
admit it with the strict IR reader, and obtain every source identity from the
admitted package. Exercise the retained owner, exact premises, malformed closure,
shared-DAG and compatible-entry controls specified by IT-018.

## Expected Results

Every IT-018 success criterion passes. Producer or reader refusal is a failure;
no test skips an unavailable boundary or remints an identity from constituent parts.
