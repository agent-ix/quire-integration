---
id: FR-004
title: "Compose bounded deterministic consumer views"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-003
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-304
    type: references
  - target: ix://agent-ix/quire-specification/FR-309
    type: references
---
# FR-004: Compose bounded deterministic consumer views

## Description

When a model view, renderer, analyzer, or code generator is requested, the
integration boundary SHALL account for every requested fact or obligation.

## Inputs

Validated checked/IR package, qualified subject root, traversal relation, filter,
projection, consumer profile, renderer/tool/options lock, requested obligation
identities, and finite traversal/output limits.

## Outputs

Canonical semantic view and identity-to-region map, or analysis/generated artifact
accounting with per-item disposition, provenance, loss, and omitted frontier.

## Behavior

- The boundary SHALL traverse only the selected typed relations from the exact
  subject root.
- The boundary SHALL validate the exact semantic input and consumer profile
  before traversal.
- The boundary SHALL order set-valued results by canonical semantic identity.
- The boundary SHALL stop before the finite result or work bound is exceeded.
- When traversal is truncated, the boundary SHALL report incomplete with the exact
  omitted frontier.
- The boundary SHALL account for each requested fact or obligation as displayed,
  omitted, generated, unsupported, requires-bound, invalid, or failed.
- The boundary SHALL preserve exact source and semantic identities for every
  output region, finding, and generated artifact.
- The boundary SHALL emit no substitute artifact for an unsupported obligation.
- The boundary SHALL NOT infer semantic relationships from layout, adjacency,
  labels, heuristic findings, generated syntax, or parser acceptance.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | Equal package/query/profile/lock inputs produce byte-identical ordered views, artifacts, mappings, and loss records. | property-based-testing (TC-004) |
| FR-004-AC-2 | Every displayed, omitted, generated, unsupported, or failed item maps to its exact source and semantic identity. | integration-testing (TC-004) |
| FR-004-AC-3 | Exact traversal bounds complete, while one-over traversal returns incomplete with an exact frontier and no completeness claim. | property-based-testing (TC-004) |
| FR-004-AC-4 | Unknown subjects, foreign relations, profile mismatch, and over-bound output refuse or remain incomplete without an inferred replacement. | negative-abuse-testing (TC-004) |
| FR-004-AC-5 | Rendering, heuristic analysis, generated text, or downstream parser acceptance cannot change source semantics or satisfy a proof/qualification claim. | property-based-testing (TC-004) |

## Error Conditions

An unknown or foreign subject, invalid relation, profile/lock mismatch,
unsupported obligation, consumer failure, malformed output mapping, or exceeded
traversal/work/output bound returns a typed refused, unsupported, failed, or
incomplete accounting record. No substitute artifact or completeness claim is
emitted.

## Dependencies

- QSpec FR-304 owns deterministic model views.
- QSpec FR-309 owns analysis and code-generation consumer accounting.
