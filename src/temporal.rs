// SPDX-License-Identifier: AGPL-3.0-or-later
//! The admitted subject/classification boundary of the FR-350 temporal bridge.
//!
//! This check admits no TL projection and invokes neither evaluator.

use qsl_package::checked_views::{
    mapping_support, Classification, Closure, MappingRefusal, TemporalSubjectView,
};
use quire_contract_ir::CheckedPackageV2;

/// Why a supplied classification cannot be consumed for this admitted subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingBindingRefusal {
    /// The classification was issued by a different package admission.
    ForeignClassification,
    /// The subject borrows a different admitted package instance.
    ForeignSubject,
    /// The selected clause cannot be classified under its admitted profile.
    Classification(MappingRefusal),
    /// The disposition or retained authenticated selection differs.
    ClassificationMismatch,
}

/// Verify the package instance before accepting any supplied mapping facts.
///
/// Reclassification compares the entire owner value, including support,
/// retained declaration/profile/activation and the sealed package/clause/
/// definition selection. Matching package names alone grants no authority.
/// A successful check is only an input precondition: outstanding projection
/// premises still have to be discharged by the bridge.
pub fn validate_mapping_binding(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    classification: &Classification,
    surrounding: Closure,
) -> Result<(), MappingBindingRefusal> {
    if !subject.belongs_to(package) {
        return Err(MappingBindingRefusal::ForeignSubject);
    }
    if !classification.belongs_to(package) {
        return Err(MappingBindingRefusal::ForeignClassification);
    }
    let expected =
        mapping_support(subject, surrounding).map_err(MappingBindingRefusal::Classification)?;
    if &expected != classification {
        return Err(MappingBindingRefusal::ClassificationMismatch);
    }
    Ok(())
}

mod formula;

pub use formula::{
    construct_tl_formula, TlFormulaCode, TlFormulaComponent, TlFormulaError, TlFormulaLeaf,
    TlFormulaLimit, TlFormulaLimits,
};

mod observation_revision;

pub use observation_revision::{
    read_observation_revision, AdmittedObservationRevision, ObservationAdmissionCode,
    ObservationAdmissionError, ObservationAdmissionLimit, ObservationAdmissionLimits,
};

mod projection;

pub use projection::{
    project_tl, read_tl_projection, ExpectedTlProjection, NativeProjectionInput, TlInput, TlLimits,
    TlMissingPremise, TlProjectionCode, TlProjectionError, TlProjectionLimit, TlProjectionPremise,
    TlProjectionSelection, TlProjectionUsage, ValidatedTlProjection,
    PROFILE as TL_CORRESPONDENCE_PROFILE,
};

mod join;
pub use join::{
    join_tl, read_tl_join, ExpectedTlJoin, TlJoinCause, TlJoinCode, TlJoinDecision, TlJoinError,
    TlJoinOwnerRefusals, ValidatedTlJoin, VERSION as TL_JOIN_VERSION,
};

mod definition;
pub use definition::{
    read_tl_bridge_definition, TlBridgeDefinitionError, ValidatedTlBridgeDefinition,
    TL_BRIDGE_DEFINITION,
};
