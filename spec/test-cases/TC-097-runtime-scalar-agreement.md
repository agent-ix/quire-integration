---
id: TC-097
title: "Compare locked QSpec scalar vectors on QSL and the runtime"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-017
    type: references
  - target: ix://agent-ix/quire-contract-runtime/FR-006
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-007
    type: verifies
---

# TC-097: Compare locked QSpec scalar vectors on QSL and the runtime

## Description

Run the QSL-versus-runtime scalar comparison in IT-017 against QSpec TC-192,
TC-185, TC-193, TC-186, and TC-187 selected by this repository's Cargo.lock.
Keep source-admission and unavailable subcases visible instead of counting
them as evaluated agreement.

## Test Procedure

Execute IT-017-SC-01 through IT-017-SC-05. Obtain every vector row from the
lock-selected QSpec source, run QSL and runtime paths with matched inputs and
limits, inject each specified denial on a fresh meter, and emit the per-row
and subcase ledger with test symbols and observations.

## Expected Results

Every evaluated row agrees on the value or exact noncompleted result and,
where specified, charge order and counters. Admission-only rows retain their
own QSL decision and no scalar charge. The full RT TC-018 through TC-022
claim remains open if any row or required sweep is unrun, unavailable, or
mismatched.
