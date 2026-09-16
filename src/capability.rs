// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Separate, deny-by-default capability boundaries for composed operations.

use crate::ecosystem_lock::CapabilitySelection;

/// Pure semantic producer capability; it has no persistence or adapter method.
pub trait ProducerCapability {
    /// Independently typed producer input.
    type Input;
    /// Owner-produced result.
    type Output;

    /// Exact lock selection implemented by this capability.
    fn selection(&self) -> &CapabilitySelection;
}

/// Non-authoritative renderer, analyzer, or code-generator capability.
pub trait ConsumerCapability {
    /// Validated consumer input.
    type Input;
    /// Attributable non-authoritative output.
    type Output;

    /// Exact lock selection implemented by this capability.
    fn selection(&self) -> &CapabilitySelection;
}

/// Durable request-state capability, distinct from external dispatch.
pub trait PersistenceCapability {
    /// Persistable request record.
    type Record;

    /// Exact lock selection implemented by this capability.
    fn selection(&self) -> &CapabilitySelection;
}

/// Evidence intake capability, which cannot act as producer or decision authority.
pub trait EvidenceIntakeCapability {
    /// External evidence presented for validation.
    type Evidence;

    /// Exact lock selection implemented by this capability.
    fn selection(&self) -> &CapabilitySelection;
}

/// Scoped human-decision capability, distinct from evidence and source authority.
pub trait DecisionCapability {
    /// Exact decision scope.
    type Scope;
    /// Attributable decision record.
    type Decision;

    /// Exact lock selection implemented by this capability.
    fn selection(&self) -> &CapabilitySelection;
}
