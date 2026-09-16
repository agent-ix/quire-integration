// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Separate, deny-by-default capability boundaries for composed operations.

use crate::{ecosystem_lock::ValidatedLock, Meter, Result};

/// Pure semantic producer capability; it has no persistence or adapter method.
pub trait ProducerCapability {
    /// Independently typed producer input.
    type Input;
    /// Owner-produced result.
    type Output;

    /// Evaluates one owner input under an already-validated exact ecosystem lock.
    fn evaluate(
        &self,
        lock: &ValidatedLock,
        input: &Self::Input,
        meter: &mut Meter,
    ) -> Result<Self::Output>;
}

/// Non-authoritative renderer, analyzer, or code-generator capability.
pub trait ConsumerCapability {
    /// Validated consumer input.
    type Input;
    /// Attributable non-authoritative output.
    type Output;

    /// Consumes validated semantics under an exact lock without acquiring producer authority.
    fn consume(
        &self,
        lock: &ValidatedLock,
        input: &Self::Input,
        meter: &mut Meter,
    ) -> Result<Self::Output>;
}

/// Durable request-state capability, distinct from external dispatch.
pub trait PersistenceCapability {
    /// Persistable request record.
    type Record;

    /// Persists an admitted record before any adapter invocation.
    fn persist(&mut self, record: &Self::Record, meter: &mut Meter) -> Result<()>;
}

/// Evidence intake capability, which cannot act as producer or decision authority.
pub trait EvidenceIntakeCapability {
    /// External evidence presented for validation.
    type Evidence;

    /// Validates and retains evidence without deciding promotion.
    fn ingest(&mut self, evidence: &Self::Evidence, meter: &mut Meter) -> Result<()>;
}

/// Scoped human-decision capability, distinct from evidence and source authority.
pub trait DecisionCapability {
    /// Exact decision scope.
    type Scope;
    /// Attributable decision record.
    type Decision;

    /// Records a decision limited to the supplied scope.
    fn decide(&mut self, scope: &Self::Scope, meter: &mut Meter) -> Result<Self::Decision>;
}
