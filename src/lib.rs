// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Version-locked composition and integration runtime for native Quire V1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod capability;
mod common;
pub mod compatibility_impact;
pub mod consumer_views;
pub mod ecosystem_lock;
pub mod effect_reconciliation;
pub mod execution_bundle;

pub use capability::{
    ConsumerCapability, DecisionCapability, EvidenceIntakeCapability, PersistenceCapability,
    ProducerCapability,
};
pub use common::{
    ArtifactId, Budget, BuildId, CanonicalEncode, CanonicalMap, CanonicalSet, ComponentId,
    ContractId, DefinitionId, DeploymentId, Disposition, Error, ErrorCode, Meter, PackageId,
    ProviderId, RepositoryId, ResourceDimension, Result, RoleId, RunId, ToolchainId, Usage,
    WorkflowInstanceId,
};
