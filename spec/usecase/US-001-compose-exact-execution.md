---
id: US-001
title: "Compose an exact native execution"
type: US
relationships:
  - target: ix://agent-ix/quire-integration/StR-001
    type: traces_to
---
# US-001: Compose an exact native execution

## Story

**As an** application integrating native Quire capabilities
**I want** one version-locked execution containing every requested domain result
**So that** I can act on a reproducible whole without losing independent failures,
uncertainty, or provenance.

## Context

State, temporal, observation, protocol, analysis, and backend results have
different authorities and non-success meanings. Composition coordinates them but
does not reinterpret them.

## Acceptance Examples (Illustrative)

### US-001-EX-1: Complete timed-refund execution

- **Given** compatible pinned packages and a qualified observation bundle
- **When** the application runs the timed-refund workflow
- **Then** one ordered bundle retains every domain result and shared prerequisite.

### US-001-EX-2: Incompatible producer

- **Given** one selected producer whose contract digest is not admitted by the lock
- **When** the application requests composition
- **Then** the mismatch remains explicit and no compatible-looking version is used.

## Priority and Risk (Informative)

P0 because every later adapter, view, and qualification result depends on exact
composition.
