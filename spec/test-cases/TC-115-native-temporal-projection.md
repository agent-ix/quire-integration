---
id: TC-115
title: "Native temporal projection admission and owner-reader controls"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-022
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/TC-461
    type: references
---
# TC-115: Native temporal projection admission and owner-reader controls

## Description

Exercise the public `project_tl` operation from genuine source emission and
strict owner admission. The fixture authors inputs and calls original owner
constructors; it copies no producer output or owner schema.

## Test Procedure

Run `temporal_mapping_binding` and `temporal_projection`. Check owner admission
membership independently from semantic equality; native row population, explicit
false, duplicate and changed bindings; future/past target selection and independent
closure axes; origin history and fixed sampling; exact subject/profile names;
actual formula, map, trace/history and request reader values; and measured exact
and one-short component limits with no partial success.

## Expected Results

Each IT-022 criterion has an executed value assertion or the exact preserved
owner refusal. Unreachable supported owner combinations remain documented
upstream work. This test does not substitute projection evidence for join or
strict correspondence-reader coverage.
