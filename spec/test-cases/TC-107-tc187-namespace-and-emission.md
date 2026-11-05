---
id: TC-107
title: "TC-187 quantity namespaces and original quantity emission"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-019
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---
# TC-107: TC-187 quantity namespaces and original quantity emission

## Description

Exercise the public U32 and U33 cells by reference to the installed QSpec vector
API. Author source and domain-model inputs here; obtain all checked nodes and
identities through public QSL intake, checking, linking and emission. Strictly
admit the original emitted package with Contract IR. This procedure measures
static admission and emission, without generated execution or value agreement.

## Test Procedure

1. Bind U32 and U33 to their complete public input and expected strings through
   `vectors_of(TestCase::Tc187)`, requiring one matching row for each ID.
2. Declare a source record and unit both named `ms`. Check a function whose
   ordinary type parameter names the record and whose `Quantity<ms>` parameter
   names the unit. Follow both original parameter type references through the
   emitted package and strict IR admission, requiring distinct record and unit
   identities and the corresponding semantic forms.
3. Assemble two genuine source documents into one linked package, each declaring
   a unit `ms`, with a `Quantity<ms>` use. Require the typed ambiguous-name
   refusal and its exact source location at that quantity type. Separately parse
   `record Quantity` and require invalid syntax at the reserved name.
4. Check and emit an original `f(3 ms)` call. Select the actual emitted quantity
   literal, require its exact magnitude, and require its type reference to be
   the original declared `ms` unit node after strict IR admission.
5. Author a domain-package object operation with a parameter `typeRef` naming
   `ix://quire/unit/ms`. Admit the native model through public intake beside
   source unit `ms`; check its resolved operation parameter and follow the
   emitted member type to that same original unit node. Do not fabricate a
   checked package, producer key or checked graph.
6. Record the measured result of each subcase. A refusal at an earlier native
   boundary is retained as an owner blocker, never converted into positive
   coverage of the requested boundary.

## Expected Results

U32 requires all three namespace subcases, including the exact refusal locations.
U33 requires the original quantity literal and admitted domain operation parameter
to reference the declared `ms` unit. Related TC-233 QL-01 through QL-04 define
quantity literal wire admission and refusal variants; TC-280 MU-01 through MU-06
define model fields, collections, operations and invalid argument variants. This
procedure covers only U33's literal magnitude 3 and operation parameter type; it
does not claim those related vector families in full.

Validation results and any owner boundary that prevents a required subcase are
reported with the frozen change. No runtime evaluation, resource-accounting,
generated oracle agreement, or whole SC-04 completion follows from these checks.
