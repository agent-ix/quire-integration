// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration-owned public bridge definition and strict selected membership.

use quire_canonical::FixedShape;
use serde::{Deserialize, Serialize};

use super::{TlLimits, TlProjectionUsage};

/// Original authored definition, with literal identity and no content-derived seal.
pub const TL_BRIDGE_DEFINITION: &[u8] =
    include_bytes!("../../definitions/native-temporal-correspondence-v1.json");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
struct Definition {
    authority: String,
    identity: String,
    profile: String,
    version: u32,
}

/// Exact definition axis refused during strict selection admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TlBridgeDefinitionError {
    /// The record is longer than `max_bytes`; `value` is `setting + 1`.
    #[error("bridge definition byte limit reached: {value} > {setting}")]
    LimitReached {
        /// First excess value.
        value: u64,
        /// Caller setting.
        setting: u64,
    },
    /// Closed JSON shape, member multiplicity or field types refused.
    #[error("malformed bridge definition")]
    Shape,
    /// Member order or spelling is noncanonical.
    #[error("noncanonical bridge definition")]
    Canonical,
    /// The selected authority differs from the original author.
    #[error("foreign bridge definition authority")]
    Authority,
    /// The selected literal definition identity differs.
    #[error("foreign bridge definition identity")]
    Identity,
    /// The selected profile differs.
    #[error("foreign bridge profile")]
    Profile,
    /// The selected version differs.
    #[error("unsupported bridge definition version")]
    Version,
}

/// Constructor-private admission to the integration-owned definition domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTlBridgeDefinition {
    definition: Definition,
    bytes: Vec<u8>,
    usage: TlProjectionUsage,
}

impl ValidatedTlBridgeDefinition {
    /// Exact original-authority name, never inferred from a profile.
    pub fn authority(&self) -> &str {
        &self.definition.authority
    }
    /// Exact authored identity.
    pub fn identity(&self) -> &str {
        &self.definition.identity
    }
    /// Exact public profile name.
    pub fn profile(&self) -> &str {
        &self.definition.profile
    }
    /// Public definition version.
    pub fn version(&self) -> u32 {
        self.definition.version
    }
    /// Bridge counters of the admitting read: the record length as `bytes`.
    pub fn usage(&self) -> TlProjectionUsage {
        self.usage
    }
    /// Complete strictly admitted original definition value.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Strictly admit a selected definition against this domain's original source.
/// Unknown, duplicate, missing and trailing fields and noncanonical bytes refuse.
pub fn read_tl_bridge_definition(
    bytes: &[u8],
    limits: TlLimits,
) -> Result<ValidatedTlBridgeDefinition, TlBridgeDefinitionError> {
    use TlBridgeDefinitionError as Refusal;
    let max_bytes = limits.max_bytes;
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if length > max_bytes {
        return Err(Refusal::LimitReached {
            value: max_bytes.saturating_add(1),
            setting: max_bytes,
        });
    }
    let definition: Definition = serde_json::from_slice(bytes).map_err(|_| Refusal::Shape)?;
    let canonical = quire_canonical::to_vec(&definition, quire_canonical::Limits::new(max_bytes))
        .map_err(|_| Refusal::Canonical)?;
    if canonical != bytes {
        return Err(Refusal::Canonical);
    }
    let original: Definition =
        serde_json::from_slice(TL_BRIDGE_DEFINITION).map_err(|_| Refusal::Shape)?;
    let axis = if definition.authority != original.authority {
        Some(Refusal::Authority)
    } else if definition.identity != original.identity {
        Some(Refusal::Identity)
    } else if definition.profile != original.profile {
        Some(Refusal::Profile)
    } else if definition.version != original.version {
        Some(Refusal::Version)
    } else {
        None
    };
    if let Some(axis) = axis {
        return Err(axis);
    }
    Ok(ValidatedTlBridgeDefinition {
        definition,
        bytes: canonical,
        usage: TlProjectionUsage {
            bytes: length,
            ..TlProjectionUsage::default()
        },
    })
}
