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

Status: Partial; the exact qualified and outstanding subcases appear below.

## Qualification Status

| Public subcase | Observed boundary | Status |
| --- | --- | --- |
| U34 source ms and model ms, both parameters, tick | Genuine native intake, exact distinct unit keys and unit IDs, scales and model root; parameter references and model owner/path survive checked emission and strict IR admission; tick's native dimension is Rate | Passed |
| U34 bare Rate | Typed missing-name refusal at the original Rate span | Passed |
| U34 unitless Length | Typed missing-name retaining the original imported Length identity and m2; assembly and public compiler agree at the model-selection source span | Partial: graph owner pointer Planned |
| U35 three Time owners, tenth and Speed | Native intake and checking, distinct dimension keys and parameter types; tenth scale and complete Speed dimension map; emission and strict IR admission | Passed |
| U35 missing local Time, Z::Time and R::Speed | Each typed missing-name refusal at its original name span | Passed |
| U35 duplicate source Time | Typed ambiguity retaining both original name spans and locating the later one | Passed |
| U35 duplicate model Dimension.identity | Native intake currently returns malformed-declaration, retaining the exact repeated identity, later member path and original reader pointer | Characterized; required conflicting-binding Planned |

U34 remains partial because the public assembly and compiler refusal APIs return
an original model-selection source span, without the required checked-graph owner
pointer. No graph pointer is inferred from a refused source. U35 remains partial:
the complete actual duplicate-intake payload is a characterization, not acceptance
of the required conflicting-binding result. The unresolved normative results stay
under IR-842 pending precise owner routing. No generated runtime result or full
SC-04 qualification is claimed.
