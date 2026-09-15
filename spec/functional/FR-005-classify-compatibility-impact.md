---
id: FR-005
title: "Classify compatibility impact and stale evidence"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-003
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-310
    type: references
---
# FR-005: Classify compatibility impact and stale evidence

## Description

When a locked ecosystem candidate differs from a baseline, the library SHALL
classify structural compatibility, supported behavioral assessment, affected
subjects, stale evidence, and unknown outcomes independently.

## Inputs

Validated baseline and candidate locks; exact changed artifact identities;
versioned compatibility rules; typed requirement, assumption, model, binding,
implementation, method, provider, environment, result, evidence, and decision
dependency graph; and bounds.

## Outputs

A canonical impact record containing changed facts, explicit dependency paths,
required regression vectors, and per-subject compatibility/evidence state.

## Behavior

- The library SHALL derive potential impact only by bounded traversal of explicit
  typed dependency edges.
- The library SHALL preserve circular and unresolved dependency paths as unknown.
- The library SHALL classify an additive compatible change, payload break,
  unchanged-payload behavioral regression, and absent result distinctly.
- The library SHALL mark dependent evidence stale without claiming that its
  subject became false.
- The library SHALL select every regression vector required by the declared
  policy.
- The library SHALL identify every unsupported or unavailable regression vector.
- The library SHALL NOT infer compatibility from equal schema bytes, repository
  reachability, semantic-name similarity, or absence of a failing result.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | Additive compatible change, payload break, behavioral regression with unchanged schema, and absent result yield four distinct impact records. | unit-testing (TC-005) |
| FR-005-AC-2 | Every affected or stale item includes an auditable path from the changed identity through typed dependency edges. | property-based-testing (TC-005) |
| FR-005-AC-3 | Circular, unresolved, foreign, or over-bound graphs remain unknown/refused and cannot become compatible or incompatible by default. | property-based-testing (TC-005) |
| FR-005-AC-4 | Stale evidence loses current support status without becoming counterevidence or a false semantic result. | property-based-testing (TC-005) |
| FR-005-AC-5 | Regression selection includes every policy-required vector exactly once and exposes each unavailable vector. | property-based-testing (TC-005) |

## Error Conditions

An invalid baseline/candidate lock, foreign identity, unknown compatibility rule,
cycle, unresolved edge, malformed dependency path, missing policy vector, or
exceeded traversal bound returns an explicit unknown, unavailable, unsupported,
or refused state. None defaults to compatible, incompatible, current, or passed.

## Dependencies

- [FR-001](./FR-001-validate-exact-ecosystem-lock.md) supplies baseline and
  candidate lock identities.
- QSpec FR-310 owns typed semantic-subject/evidence relationships.
