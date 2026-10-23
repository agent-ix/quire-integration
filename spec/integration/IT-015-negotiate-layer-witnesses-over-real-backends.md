---
id: IT-015
title: "Negotiate each profile layer's witness item over the real backends"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-345
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-331
    type: references
---
# IT-015: Negotiate each profile layer's witness item over the real backends

## Objective

Show that, over the real backend registry, the witness item of each of the five
AD-003 profile layers settles `supported` under at least one candidate backend,
using quire-contract-codegen's `negotiate_*` as the negotiation function of
QSL's profile-layering gate. This is the real-backend half of FR-345-AC-6 and
of the witness half of FR-345-AC-9. QSL's own run passes no negotiation
function, because QSL takes no dependency on quire-contract-codegen
(FR-280-AC-3).

## Target Integration

QSL `xtask::refinement::layering::compare` over `REAL_CORPUS` with the FR-278
composition and the real backend registry → one `WitnessRequest` per layer and
candidate → quire-contract-codegen `negotiate_*` (QSpec FR-331-AC-4) → the
`qsl_route::Disposition` returned to `compare`.

## Preconditions

`xtask` is a dev dependency of quire-integration, taken from the same QSL
source as its other QSL dependencies, so `REAL_CORPUS` is the corpus of the QSL
revision under test. A local build is enough. A missing registry, a missing
corpus entry or an unresolvable dependency fails the test; none counts as a
skipped case.

## Inputs

The corpus at `REAL_CORPUS`, the FR-278 composition, the real backend registry,
and a negotiation function that forwards each `WitnessRequest` (candidate,
`witness_request` kind and extent, compiled clause) to quire-contract-codegen's
`negotiate_*` and returns its disposition.

## Test Procedure

1. Call `compare` with the negotiation function.
   IT-015-SC-01: for each of the five layers, the layer's witness unit admits,
   its witness item is requested once from each candidate backend of its
   `witness_request` kind by backend name, and it settles `supported` under at
   least one candidate; every layer's witness result is `holds` and the verdict
   is success.
2. Read every case and edge result of the call in step 1 and of a call to
   `compare` over the same corpus with no negotiation function.
   IT-015-SC-02: the case and edge results are identical in the two calls, and
   the negotiation call differs only in the layers' witness results being
   decided by negotiation.
3. Call `compare` over a copy of the corpus in which the `witness` entry for one
   layer names a unit whose witness item the real candidates do not support,
   and over a copy with that layer's `witness` entry removed.
   IT-015-SC-03: the first call gives a `regression` naming the layer and each
   candidate's backend identity and disposition, verdict violation; the second
   gives a tool-failure result naming the layer and `witness`, verdict tool
   failure.

## Expected Results

Every step uses the real registry and codegen's real `negotiate_*`. A layer
whose witness item settles `supported` under no candidate, a call that returns
a report other than the gate's, a negotiation function that is never called, or
a missing prerequisite fails the test.
