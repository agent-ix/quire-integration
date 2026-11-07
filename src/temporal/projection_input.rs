// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compose authenticated native input and a strictly admitted TL formula.
//!
//! The TL valuation request and correspondence record are not yet constructed.

use qsl_eval::temporal::evidence::EvidenceRequest;
use qsl_package::checked_views::{Classification, Closure, TemporalSubjectView};
use quire_contract_ir::CheckedPackageV2;
use quire_observation::authority::observation::View as ObservationView;

use super::{
    admit_native_tl_input, construct_tl_formula, NativeInputError, NativeInputLimits,
    NativeTlInput, TlFormulaComponent, TlFormulaError, TlFormulaLimits,
};

/// Separate ceilings for native input and owner formula construction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectionInputLimits {
    /// Native position scan.
    pub native: NativeInputLimits,
    /// TL formula construction.
    pub formula: TlFormulaLimits,
}

/// Stable error category for the composed prerequisite.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionInputCode {
    /// Native request admission failed.
    Native,
    /// Formula construction or owner admission failed.
    Formula,
}

/// The first prerequisite that refused, without a partial projection.
#[derive(Debug, thiserror::Error)]
pub enum ProjectionInputError {
    /// Native request refusal, preserving its typed cause.
    #[error("native TL input refused: {0}")]
    Native(#[from] NativeInputError),
    /// Formula refusal, preserving its typed cause.
    #[error("TL formula input refused: {0}")]
    Formula(#[from] TlFormulaError),
}

impl ProjectionInputError {
    /// Stable machine-readable category.
    pub const fn code(&self) -> ProjectionInputCode {
        match self {
            Self::Native(_) => ProjectionInputCode::Native,
            Self::Formula(_) => ProjectionInputCode::Formula,
        }
    }
}

/// Inputs validated against one supplied checked-package instance.
///
/// This value is a preparation result, not a TL projection or FR-350 decision.
#[derive(Debug)]
pub struct PreparedTlProjectionInput<'a> {
    native: NativeTlInput<'a>,
    formula: TlFormulaComponent,
}

impl<'a> PreparedTlProjectionInput<'a> {
    /// Authenticated native owner request, occurrence, observation, and models.
    pub const fn native(&self) -> &NativeTlInput<'a> {
        &self.native
    }

    /// Strictly owner-admitted TL formula and original checked leaf handles.
    pub const fn formula(&self) -> &TlFormulaComponent {
        &self.formula
    }
}

/// Prepare native and TL formula prerequisites from the same admitted package.
///
/// Classification is owner-recomputed by formula construction. The selected
/// native binding is authenticated against that same borrowed subject. No TL
/// valuation request or correspondence record is inferred, and no evaluator runs.
pub fn prepare_tl_projection_input<'a>(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    classification: &Classification,
    surrounding: Closure,
    request: &'a EvidenceRequest,
    observation: &'a ObservationView,
    limits: ProjectionInputLimits,
) -> Result<PreparedTlProjectionInput<'a>, ProjectionInputError> {
    let formula = construct_tl_formula(
        package,
        subject,
        classification,
        surrounding,
        limits.formula,
    )?;
    let native = admit_native_tl_input(package, subject, request, observation, limits.native)?;
    Ok(PreparedTlProjectionInput { native, formula })
}
