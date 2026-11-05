---
id: TC-110
title: TC-187 source and model quantity namespaces
type: TC
relationships:
  - target: ix://agent-ix/quire-specification/FR-142
    type: verifies
---
# TC-110: TC-187 source and model quantity namespaces

## Description

Exercise the public U34 and U35 vectors by reference through native model intake,
source assembly, checking, checked-package emission and strict IR admission where
applicable. Original minimal model documents supply domain declarations; no checked
keys or owner receipts are fabricated.

## Test Procedure

1. Bind U34 and U35 to their complete public input and expected records.
2. For U34, admit the Rate package, select source ms and model ms independently,
   check both function parameters and tick, and inspect exact unit identities,
   model ownership, and scalar_type/unit paths. Refuse bare Rate at its source
   name. Add unitless imported Length and require the original first refusal,
   retained dimension and carrier, including the emitted owner pointer.
3. For U35, admit independent source, R and F Time dimensions and units, the
   tenth unit and derived Speed. Require distinct native keys and parameter
   types. Refuse missing local Time, missing alias Z, missing R::Speed and
   duplicate source Time with their typed codes and original loci.
4. Present a duplicate native Dimension.identity and require intake refusal at
   the later identity, before assembly. Read unchanged healthy emissions through
   strict IR with their original domain documents.

## Expected Results

Each authored subcase reaches its owning public decision, retaining exact native
identities and loci. This procedure qualifies admission and emission only; it
claims no generated execution or complete SC-04 coverage.

Status: Planned until focused executable checks complete.
