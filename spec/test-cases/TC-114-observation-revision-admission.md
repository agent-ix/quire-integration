---
id: TC-114
title: "Exact observation selection and bundle predecessor admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-021
    type: verifies
---
# TC-114: Exact observation selection and bundle predecessor admission

## Description

Exercise the public partial observation admission component using integration-authored
inputs passed through real QObs owner APIs, without copied owner fixture documents.

## Test Procedure

Run `temporal_observation_revision`. Compare genuine initial/corrected bundle and
selected observation values and retained prior bytes. Test unchanged observation in
corrected bundles, foreign and revised observation substitutions, missing/foreign/
non-direct predecessors, malformed/noncanonical/trailing bytes and exact/one-over
component byte and scan-work limits. Assert native typed refusals and stable component
error codes. Run temporal mapping binding and formula construction regressions.

## Expected Results

All IT-021 component criteria pass. Every rejection yields no receipt. No temporal
evaluator runs and no full projection, correspondence or native/TL join is claimed.
