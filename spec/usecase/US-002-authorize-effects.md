---
id: US-002
title: "Authorize and reconcile workflow effects"
type: US
relationships:
  - target: ix://agent-ix/quire-integration/StR-001
    type: traces_to
---
# US-002: Authorize and reconcile workflow effects

## Story

**As an** operator responsible for external workflow changes
**I want** recommendations to require attributable authorization and idempotent
request reconciliation
**So that** retries or transport receipts cannot duplicate or falsely establish a
business effect.

## Context

Formal evaluators recommend actions but do not possess operational authority.
Receipt delivery and independently observed business effect are different facts.

## Acceptance Examples (Illustrative)

### US-002-EX-1: Crash after persistence

- **Given** a persisted request whose transport attempt has no recorded receipt
- **When** the coordinator restarts and retries
- **Then** it reconciles the same request identity and idempotency key.

### US-002-EX-2: Receipt without effect

- **Given** an accepted transport receipt but no authority-qualified effect observation
- **When** the workflow is reassessed
- **Then** the effect remains unestablished.

## Priority and Risk (Informative)

P0 because duplicate or fabricated effects cross the formal-system boundary into
real operations.
