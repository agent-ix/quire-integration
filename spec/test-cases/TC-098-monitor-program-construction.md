---
id: TC-098
title: "Monitor construction with admitted semantic identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-018
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: verifies
---
# TC-098: Monitor construction with admitted semantic identities

## Description

Verify IT-018's construction controls through the public RT construction API,
and FR-357-AC-9's separate explicit selected-fact boundary through the public IR reader.

## Test Procedure

Run the complete `monitor_program` integration target. Emit authored QSL source,
admit it with the strict IR reader, and obtain every source identity from the
admitted package. Exercise the retained owner, exact premises, malformed closure,
shared-DAG and compatible-entry controls specified by IT-018.

Run `monitor_selected_facts` separately for FR-357-AC-9. Obtain a real final
QSL `PlanRoot.emission`, admit its bytes strictly, and select model declaration,
field and equality application/occurrence IDs from that same emission. Translate
its closed equality-row vocabulary without changing values. Compare exact model
rows and field order, presence and type, then compare the public selected-fact
read with the native FR-353 equality result. A native unavailable-witness refusal
must retain its precise typed cause and locus, with no partial selected view.

## Expected Results

Every IT-018 success criterion passes. Producer or reader refusal is a failure;
no test skips an unavailable boundary or remints an identity from constituent parts.

The selected-fact control remains Planned/Unrun until its focused target passes.
Its intended qualification covers only explicit representable model facts and
the selected equality's opaque decision or native typed missing-witness result.
It does not traverse monitor entries or establish a complete generated-source
closure, `TypeEnvironment`, RT `ProgramParts` or temporal execution readiness.
FR-357 AC1-4 remain outside this control.
