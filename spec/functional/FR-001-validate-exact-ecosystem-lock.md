---
id: FR-001
title: "Validate an exact ecosystem lock"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-001
    type: implements
  - target: ix://agent-ix/quire-specification/FR-335
    type: references
---
# FR-001: Validate an exact ecosystem lock

## Description

When a composed or qualification run is requested, the library SHALL validate one
canonical `quire.ecosystem-lock/v1` before invoking any producer or consumer.

## Inputs

Repository remotes and immutable revisions; package identities, versions, and
dependency closure; definition identities and digests; provider/toolchain/build
options; exact capability bindings; deployment selections; required-role
inventory; and resource limits.

## Outputs

A constructor-private validated lock view and content-derived lock identity, or a
typed refusal identifying the missing, duplicate, stale, dirty, unknown,
incompatible, or over-bound selection.

## Behavior

- The library SHALL validate each identity in its declared domain.
- The library SHALL order set-valued lock members canonically before deriving the
  lock identity.
- The library SHALL distinguish repository, package, semantic definition,
  provider, toolchain, build, deployment, role, component, and workflow-instance
  identities.
- The library SHALL bind each required role to exactly one selected provider,
  build, compatibility contract, and canonical option set; the role's component
  mapping SHALL resolve the exact owning package and its enabled features.
- The library SHALL require every dependency edge to name exact source and target
  identities plus the required compatibility contract.
- The library SHALL report unresolved reachability as unknown compatibility.
- The library SHALL NOT treat graph reachability as demonstrated compatibility or
  incompatibility.
- The library SHALL refuse every required ambient, moving, unpinned, or
  current-default selection.
- The library SHALL validate the complete lock before producer invocation.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | Equivalent complete locks serialize identically and derive the same identity independent of discovery order. | property-based-testing (TC-001) |
| FR-001-AC-2 | Mutating any meaning-affecting pin changes the lock and run identity. | property-based-testing (TC-001) |
| FR-001-AC-3 | Missing, duplicate, stale, dirty, unknown, cross-domain, or one-over selections refuse before any producer runs. | unit-testing (TC-001) |
| FR-001-AC-4 | Multiple roles per component and multiple components per repository remain distinct and explicitly mapped; every role resolves one exact provider, build, contract, option set, owning package, and enabled-feature set. | unit-testing (TC-001) |
| FR-001-AC-5 | Reachable, affected, stale, compatible, incompatible, and unknown remain distinct classifications. | property-based-testing (TC-001) |

## Error Conditions

A missing, duplicate, stale, dirty, moving, unknown, cross-domain,
contract-incompatible, malformed, or one-over selection returns a typed lock
refusal before any producer or consumer invocation. The refusal identifies the
exact selection and identity domain without substituting a current default.

## Dependencies

- QSpec FR-335 owns the ecosystem-lock value contract.
- QSpec AD-011 owns the integration and qualification architecture.
