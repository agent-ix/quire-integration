---
id: IT-022
title: "Admit native temporal input for TL projection"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
---
# IT-022: Admit native temporal input for TL projection

## Objective

Qualify the native input prerequisite of FR-350 using public QSL
`EvidenceRequest` and `TraceBinding` values and an independently admitted
QObs observation. This is an in-process input receipt, not a TL projection,
correspondence record, or final FR-350 decision.

## Target Integration

Strict Contract IR checked package → QSL `TemporalSubjectView` → QSL
`EvidenceRequest` and `TraceBinding::authenticate` → exact clause occurrence
and QObs observation identity → retained native input for later `project_tl`.
Neither evaluator runs.

## Preconditions

The subject is borrowed from the supplied admitted checked package. The
observation has passed the QObs owner reader independently of the request.
The request's positions carry native valuation cells; no Boolean totality or
projection suitability is implied by retaining them.

## Inputs

The public component accepts the admitted package, subject, native request,
admitted observation view, and a caller-configured position-scan ceiling. The request itself supplies the selected
`SemanticTriggerIdentity`, `positions` valuations, optional opaque anchor,
and occurrence key. The subject supplies the owner profile and clock rows.

## Test Procedure

1. IT-022-SC-01: Resolve an explicit occurrence by role and ordinal, or an
   omitted occurrence only when the subject has exactly one. Reject an
   unknown or ambiguous occurrence before returning a receipt.
2. IT-022-SC-02: Authenticate the actual request binding against the selected
   subject's profile and clock through `TraceBinding::authenticate`; preserve
   the owner refusal and reject a foreign package instance.
3. IT-022-SC-03: Match a present opaque anchor exactly against the independently
   admitted observation's identity and exactly one request position's
   observation identity. Compare that position's numeric value with the
   admitted observation's canonical anchor in the authenticated clock family
   without parsing the opaque identity.
   Reject a missing, foreign, duplicate, or conflicting match.
4. IT-022-SC-04: Retain the original request, instance trigger, valuation
   positions, exact occurrence, authenticated binding, admitted observation,
   and numeric anchor as typed values. No evaluator is called.
5. IT-022-SC-05: Refuse before scanning when the request has more positions
   than the component ceiling. Exactly at the setting admits; one over returns
   a typed limit refusal and no receipt.

## Expected Results

The focused real-producer test admits one matched request and preserves its
false shared valuation and true instance-bound valuation. Mutations of the
package instance, binding profile, occurrence, opaque identity, duplicate
position and numeric anchor return distinct typed refusals. This component
does not construct a TL owner request or complete FR-350 acceptance criteria.
