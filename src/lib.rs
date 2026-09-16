// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Version-locked composition and integration runtime for native Quire V1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod capability;
mod common;
pub mod ecosystem_lock;

pub use capability::{
    ConsumerCapability, DecisionCapability, EffectAdapterCapability, EvidenceIntakeCapability,
    PersistenceCapability, ProducerCapability,
};
pub use common::{
    ArtifactId, Budget, BuildId, CanonicalEncode, CanonicalMap, CanonicalSet, ComponentId,
    DefinitionId, DeploymentId, Disposition, Error, ErrorCode, Meter, PackageId, ProviderId,
    RepositoryId, ResourceDimension, Result, RoleId, RunId, ToolchainId, Usage, WorkflowInstanceId,
};
