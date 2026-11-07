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

For IT-018-SC-16, run the fallible-type-construction cases in the dedicated
`monitor_program` test binary, which uses the owner's thread-local failing allocator.
Compile the authored QSL fixture and strictly admit its unchanged emission before arming
denial. Obtain every declaration, field, source location and type identity through the
existing checked-package lookup; unavailable or refused lookup fails the test. Construct
all input parts and expected native request observations before denial. Refuse the first
actual allocation inside `Program::try_new`, assert `Membership`, the original native
allocation class and exact request unit/amount, and no Program. Record that the owner
failing allocator automatically disarms after its one denied request; do not credit this
as denial throughout refusal projection. Inspect the production mapping separately for
absence of allocation and allocating clones while constructing the refusal. Reconstruct
the same valid parts with allocation enabled for a successful retry. Independently deny the first RT-owned validation scratch reservation
and assert `Storage`. Retain semantic unknown-type/operator-ineligible controls and their
original precedence. Distinguish native allocation versus capacity projection using actual
owner error values; describe these as projection controls rather than allocator/capacity
exhaustion evidence. Do not introduce a test identity constructor, bypass an identity guard,
or claim that a controlled arena is a genuine emitted temporal program.

Run `monitor_selected_facts` separately for FR-357-AC-9. Obtain a real final
QSL `PlanRoot.emission`, admit its bytes strictly, and select model declaration,
field and equality application/occurrence IDs from that same emission. The
selected record model document is authored by this test; the monitor subject
uses QSL's ConfigVersion document by reference. Translate
its closed equality-row vocabulary through the producer-owned public
`equality_witness` adapter without changing values. Compare exact model
rows and field order, presence and type, then compare the public selected-fact
read with the native FR-353 equality result. A native unavailable-witness refusal
must retain its precise typed cause and locus, with no partial selected view.

## Expected Results

Every IT-018 success criterion passes. Producer or reader refusal is a failure;
no test skips an unavailable boundary or remints an identity from constituent parts.

The focused selected-fact control passes with successful joins for the selected
original record and required integer field, followed by an opaque native FR-353
equality decision preserved in the public selected-fact result. It also retains
the precise assertion path for a future native typed missing-witness result;
that alternative was not observed in this run. This qualification covers only
these explicit representable model facts and the selected equality.
It does not traverse monitor entries or establish a complete generated-source
closure, `TypeEnvironment`, RT `ProgramParts` or temporal execution readiness.
FR-357 AC1-4 remain outside this control.

The first Layout owner allocation denial returns a native `Membership` refusal rather than
aborting, losing the original request or publishing a Program. The allocation-enabled retry
succeeds, RT scratch retains `Storage`, and semantic failures preserve their original meaning
and order. This is constructor qualification only.
