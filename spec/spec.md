---
type: master-requirements
name: quire-integration
org: agent-ix
component_type: rust-library
implementation_language: rust
tags:
  - quire
  - composition
  - integration
  - qualification
depends_on: []
relationships:
  - target: ix://agent-ix/quire-specification/AD-008
    type: references
  - target: ix://agent-ix/quire-specification/AD-011
    type: references
standards_alignment:
  - iso-iec-ieee-29148
  - ieee-828
---
# Master Requirements Specification

## Purpose

This specification defines the private Rust composition and integration boundary
for complete native Quire V1. The library validates one exact ecosystem lock,
coordinates independently owned domain evaluators, preserves their typed results,
mediates authorized external effects, exposes bounded deterministic views, and
records attributable integration and qualification outcomes.

## Scope

### In Scope

- Exact repository, package, definition, provider, toolchain, build, and
  deployment selection under one immutable ecosystem lock.
- Composition of state, temporal, observation, protocol, analysis, backend, and
  tooling results without flattening their independent meanings.
- Attributed human/policy decisions, persistent idempotent effect requests,
  transport-receipt reconciliation, and separate business-effect observations.
- Bounded semantic queries, deterministic rendering inputs, analysis/codegen
  accounting, compatibility impact, stale-evidence classification, and
  IT-070 through IT-078 qualification orchestration.

### Out of Scope

- Source parsing, name/type checking, temporal or protocol interpretation,
  observation authority, proof algorithms, generated-code semantics, and
  evidence-store ownership.
- Direct business-effect execution by an evaluator or analyzer.
- Ambient dependency discovery, moving-branch selection, inferred compatibility,
  inferred workflow relationships, or Boolean promotion of non-success outcomes.
- Public packages, hosted CI, deployment automation, and automatic release
  decisions.

## System Overview

### System Description

`quire-integration` is a pure coordination library around versioned Rust and wire
interfaces I04 through I21. It validates producer artifacts independently, invokes
only explicitly selected capabilities, and emits canonical execution,
compatibility, view, effect, and qualification records.

### Intended Users

- Applications composing native Quire state, temporal, and protocol execution.
- Operators authorizing and reconciling external workflow effects.
- Tooling and assurance consumers running deterministic views and complete-V1
  qualification.

## Requirements Architecture

[StR-001](stakeholder/StR-001-trust-composed-execution.md) defines the accountable
need. US-001 drives exact composed execution, US-002 drives the authorized effect
boundary, and US-003 drives bounded inspection and qualification. FR-001 through
FR-006 define the six owned behaviors. NFR-001 and NFR-002 constrain
reproducibility, resources, purity, and failure containment. IT-001 through IT-004
exercise the real pinned producer and consumer boundaries.

## Ownership Boundary

QSL owns source and semantic packages; state, TL, QObs, and Protocol own domain
semantics; Verification and backend providers own their methods and results;
codegen owns generated artifacts; Quoin owns evidence intake. This repository
owns exact lock validation, coordination, accounting, effect-request
reconciliation, deterministic view composition, and cross-repository
qualification. A successful composition never enlarges a producer claim.

## Error and Failure Model

Every operation returns a closed typed disposition. `incompatible`, `stale`,
`unsupported`, `incomplete`, `refused`, `failed`, and `unexecuted` remain distinct
from `completed`. A failure in one independent item remains attached to that item
unless a contradictory shared prerequisite invalidates the whole composition.

## References

- QSpec reviewed revision `8d0fbad`, merged through QSpec PR #64.
- QSpec AD-008, AD-011, FR-180, FR-182, FR-304, FR-309 through FR-312, and
  interface value contracts FR-323 through FR-338.
- Research #34/#39/#40/#74 through #78.
