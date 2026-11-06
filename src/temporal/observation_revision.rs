// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-021: exact observation selection in a historical QObs bundle.

use quire_observation::authority::{bundle, observation, Context, Error, Limits};

/// Ceilings for component input bytes and record/component scan visits.
/// Owner-internal work remains subject to the separately supplied native limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservationAdmissionLimits {
    /// Sum of current bundle and selected observation canonical bytes.
    pub max_bytes: u64,
    /// One unit before each embedded-record or component-stamp inspection.
    pub max_work: u64,
}

impl Default for ObservationAdmissionLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16_777_216,
            max_work: 65_536,
        }
    }
}

/// Closed component counter vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationAdmissionLimit {
    /// Current input bytes.
    Bytes,
    /// Component scan visits.
    Work,
}

/// Stable component refusal catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationAdmissionCode {
    /// Native historical owner admission refused.
    OwnerRead,
    /// Exact selected observation is absent from the admitted bundle.
    ObservationMismatch,
    /// A component ceiling was exceeded.
    LimitReached,
}

impl ObservationAdmissionCode {
    /// Stable wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OwnerRead => "observation-admission-owner-read",
            Self::ObservationMismatch => "observation-admission-selection-mismatch",
            Self::LimitReached => "observation-admission-limit-reached",
        }
    }

    /// Every catalog entry.
    pub const fn all() -> &'static [Self] {
        &[
            Self::OwnerRead,
            Self::ObservationMismatch,
            Self::LimitReached,
        ]
    }

    /// Parse a catalog spelling.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|entry| entry.as_str() == code)
    }
}

/// A typed refusal never exposes a partial receipt.
#[derive(Debug, thiserror::Error)]
pub enum ObservationAdmissionError {
    /// Preserve the complete native owner error, including its usage and code.
    #[error("observation owner admission refused: {0}")]
    OwnerRead(#[source] Error),
    /// Full observation bytes or the corresponding exact component stamp differ.
    #[error("exact selected observation is absent from the admitted bundle")]
    ObservationMismatch,
    /// Component work stopped before the first excess action.
    #[error("observation admission {limit:?} limit reached: {value} > {setting}")]
    LimitReached {
        /// Exhausted counter.
        limit: ObservationAdmissionLimit,
        /// First value exceeding the setting.
        value: u64,
        /// Configured ceiling.
        setting: u64,
    },
}

impl ObservationAdmissionError {
    /// Stable refusal code independent of display text.
    pub const fn code(&self) -> ObservationAdmissionCode {
        match self {
            Self::OwnerRead(_) => ObservationAdmissionCode::OwnerRead,
            Self::ObservationMismatch => ObservationAdmissionCode::ObservationMismatch,
            Self::LimitReached { .. } => ObservationAdmissionCode::LimitReached,
        }
    }
}

/// Partial observation-input admission, never a native/TL join or agreement.
///
/// Retains the actual admitted historical bundle, independently admitted selected
/// observation and exact previous receipt. Bundle lineage does not establish
/// selected observation-document lineage, QSL capture binding or result lineage.
#[derive(Debug)]
pub struct AdmittedObservationRevision<'a> {
    bundle: bundle::View,
    observation: &'a observation::View,
    predecessor: Option<&'a AdmittedObservationRevision<'a>>,
    work: u64,
}

impl<'a> AdmittedObservationRevision<'a> {
    /// Complete owner-admitted current bundle, including canonical bytes.
    pub fn bundle(&self) -> &bundle::View {
        &self.bundle
    }
    /// Original independently admitted selected observation value and bytes.
    pub fn observation(&self) -> &'a observation::View {
        self.observation
    }
    /// Actual previous receipt; no identity-only replacement is retained.
    pub fn predecessor(&self) -> Option<&'a AdmittedObservationRevision<'a>> {
        self.predecessor
    }
    /// Component scan visits; excludes all owner-internal work.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// Strictly admit historical v1 bundle bytes and their exact selected observation.
///
/// Native Context/Selection remain independent expectations. QObs validates the
/// exact bundle predecessor; no current-head/no-fork claim is made. All scans
/// finish before returning a receipt. No temporal evaluator is invoked.
#[allow(clippy::too_many_arguments)]
pub fn read_observation_revision<'a>(
    bytes: &[u8],
    context: Context<'_>,
    selection: &bundle::Selection,
    observation: &'a observation::View,
    predecessor: Option<&'a AdmittedObservationRevision<'a>>,
    owner_limits: Limits,
    limits: ObservationAdmissionLimits,
) -> Result<AdmittedObservationRevision<'a>, ObservationAdmissionError> {
    let input_bytes = u128::try_from(bytes.len())
        .unwrap_or(u128::MAX)
        .saturating_add(u128::try_from(observation.bytes().len()).unwrap_or(u128::MAX));
    if input_bytes > u128::from(limits.max_bytes) {
        return Err(limit_error(
            ObservationAdmissionLimit::Bytes,
            limits.max_bytes,
        ));
    }
    let bundle = bundle::read_revision(
        bytes,
        context,
        selection,
        predecessor.map(|prior| &prior.bundle),
        owner_limits,
    )
    .map_err(ObservationAdmissionError::OwnerRead)?;
    let mut work = 0;
    let mut record_matches = false;
    for record in bundle.payload().records() {
        charge(&mut work, limits.max_work)?;
        if matches!(record.payload(), bundle::EmbeddedPayloadRef::Observation(_))
            && record.canonical_document_bytes() == observation.bytes()
        {
            record_matches = true;
        }
    }
    let mut stamp_matches = false;
    for component in bundle.payload().components() {
        charge(&mut work, limits.max_work)?;
        if component.role() == bundle::ComponentRole::Observation
            && component.identity() == observation.identity().as_str()
            && component.contract() == observation.contract()
            && component.revision() == observation.revision()
        {
            stamp_matches = true;
        }
    }
    if !record_matches || !stamp_matches {
        return Err(ObservationAdmissionError::ObservationMismatch);
    }
    Ok(AdmittedObservationRevision {
        bundle,
        observation,
        predecessor,
        work,
    })
}

fn charge(work: &mut u64, setting: u64) -> Result<(), ObservationAdmissionError> {
    if *work >= setting {
        return Err(limit_error(ObservationAdmissionLimit::Work, setting));
    }
    *work += 1;
    Ok(())
}

fn limit_error(limit: ObservationAdmissionLimit, setting: u64) -> ObservationAdmissionError {
    ObservationAdmissionError::LimitReached {
        limit,
        value: setting.saturating_add(1),
        setting,
    }
}
