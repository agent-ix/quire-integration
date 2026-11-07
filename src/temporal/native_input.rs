// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native FR-350 input admission before TL request construction.
//!
//! This in-process receipt retains owner values. It neither evaluates a clause
//! nor asserts that its valuations are complete or suitable for TL projection.

use qsl_eval::temporal::evidence::{
    AuthenticatedBinding, BindingMismatch, ClauseSelection, EvidencePosition, EvidenceRequest,
    SemanticTriggerIdentity,
};
use qsl_package::checked_views::{ClauseOccurrence, TemporalSubjectView};
use quire_contract_ir::{CheckedPackageV2, ClockConfiguration};
use quire_observation::authority::observation::{AnchorRef, View as ObservationView};

/// Ceiling for the request-position scan performed by this component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeInputLimits {
    /// Maximum positions before the anchor scan.
    pub max_positions: u64,
}

impl Default for NativeInputLimits {
    fn default() -> Self {
        Self {
            max_positions: 1_048_576,
        }
    }
}

/// Stable refusal category for native input admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeInputCode {
    /// The subject belongs to another admitted package instance.
    ForeignSubject,
    /// The request does not select one exact clause occurrence.
    Occurrence,
    /// The owner's profile or clock authentication refused the request.
    Binding,
    /// The opaque anchor or admitted observation does not select one position.
    Anchor,
    /// The independently admitted observation has a different numeric anchor.
    ObservationAnchor,
    /// The request has more positions than this component admits.
    LimitReached,
}

/// Why native owner inputs cannot be passed to TL construction.
#[derive(Debug, thiserror::Error)]
pub enum NativeInputError {
    /// The view was built from another package instance.
    #[error("temporal subject belongs to another package instance")]
    ForeignSubject,
    /// Omitted occurrence with several candidates, or an unknown exact key.
    #[error("request does not select one exact clause occurrence")]
    Occurrence,
    /// The original QSL owner refusal is retained.
    #[error("native trace binding refused: {0}")]
    Binding(#[source] BindingMismatch),
    /// A present opaque anchor is foreign or selects no unique position.
    #[error("request anchor does not identify one admitted observation position")]
    Anchor,
    /// QObs anchor disagrees with the selected numeric position.
    #[error("admitted observation anchor does not match the numeric position")]
    ObservationAnchor,
    /// The component refuses before scanning a request over its ceiling.
    #[error("native input positions exceed limit: {value} > {setting}")]
    LimitReached {
        /// Number of supplied positions.
        value: u64,
        /// Caller setting.
        setting: u64,
    },
}

impl NativeInputError {
    /// Stable machine-readable category.
    pub const fn code(&self) -> NativeInputCode {
        match self {
            Self::ForeignSubject => NativeInputCode::ForeignSubject,
            Self::Occurrence => NativeInputCode::Occurrence,
            Self::Binding(_) => NativeInputCode::Binding,
            Self::Anchor => NativeInputCode::Anchor,
            Self::ObservationAnchor => NativeInputCode::ObservationAnchor,
            Self::LimitReached { .. } => NativeInputCode::LimitReached,
        }
    }
}

/// Authenticated native input retained for the TL bridge's later projection.
///
/// The request remains the source of all valuation cells and selected trigger.
/// The numeric anchor comes from its single matched position, after comparison
/// with an independently admitted QObs observation. No opaque identity is parsed.
#[derive(Debug)]
pub struct NativeTlInput<'a> {
    request: &'a EvidenceRequest,
    observation: &'a ObservationView,
    limits: NativeInputLimits,
    occurrence: ClauseOccurrence,
    binding: AuthenticatedBinding,
    anchor: Option<u64>,
}

impl<'a> NativeTlInput<'a> {
    /// Original owner request, including every position and valuation cell.
    pub const fn request(&self) -> &'a EvidenceRequest {
        self.request
    }

    /// Position valuations, borrowed from the original owner request.
    pub fn positions(&self) -> &'a [EvidencePosition] {
        &self.request.positions
    }

    /// Selected instance identity, with original opaque bytes.
    pub fn trigger(&self) -> Option<&'a SemanticTriggerIdentity> {
        self.request.trigger.as_ref()
    }

    /// Exact resolved clause occurrence, including singleton omission.
    pub const fn occurrence(&self) -> &ClauseOccurrence {
        &self.occurrence
    }

    /// QSL-authenticated profile and clock selection.
    pub const fn binding(&self) -> &AuthenticatedBinding {
        &self.binding
    }

    /// Independently admitted observation used to select the numeric anchor.
    pub const fn observation(&self) -> &'a ObservationView {
        self.observation
    }

    /// Component ceiling used for anchor selection.
    pub const fn limits(&self) -> NativeInputLimits {
        self.limits
    }

    /// Numeric owner anchor for the selected observation, if the request has one.
    pub const fn anchor(&self) -> Option<u64> {
        self.anchor
    }
}

/// Admit the actual QSL native request and an independently admitted QObs value.
///
/// The occurrence key is compared by role and ordinal, never by array index.
/// `None` is valid only for a singleton. A present opaque anchor must equal
/// both the selected request position's identity and the QObs observation
/// identity. Its numeric position must agree with QObs's canonical anchor text.
/// No evaluator runs and no valuation totality is inferred here.
pub fn admit_native_tl_input<'a>(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    request: &'a EvidenceRequest,
    observation: &'a ObservationView,
    limits: NativeInputLimits,
) -> Result<NativeTlInput<'a>, NativeInputError> {
    if !subject.belongs_to(package) {
        return Err(NativeInputError::ForeignSubject);
    }
    let occurrence = match (&request.occurrence, subject.occurrences()) {
        (Some(key), rows) => rows
            .iter()
            .find(|candidate| candidate.role == key.role && candidate.ordinal == key.ordinal),
        (None, [only]) => Some(only),
        (None, _) => None,
    }
    .ok_or(NativeInputError::Occurrence)?;
    let binding = request
        .binding
        .authenticate(&ClauseSelection {
            declaration: subject.declaration(),
            profile: subject.profile(),
            clock: subject.clock(),
        })
        .map_err(NativeInputError::Binding)?;

    let count = u64::try_from(request.positions.len()).unwrap_or(u64::MAX);
    if count > limits.max_positions {
        return Err(NativeInputError::LimitReached {
            value: count,
            setting: limits.max_positions,
        });
    }

    let anchor = if let Some(opaque) = request.anchor.as_deref() {
        if opaque != observation.payload().observation_identity() {
            return Err(NativeInputError::Anchor);
        }
        let mut matches = request
            .positions
            .iter()
            .filter(|position| position.observation.identity == opaque);
        let position = matches.next().ok_or(NativeInputError::Anchor)?;
        if matches.next().is_some() {
            return Err(NativeInputError::Anchor);
        }
        let expected = position.position.to_string();
        let admitted = match (&binding.clock, observation.payload().anchor()) {
            (ClockConfiguration::EventPosition { .. }, AnchorRef::EventPosition { position }) => {
                position
            }
            (ClockConfiguration::FixedSample { .. }, AnchorRef::FixedSample { index, .. }) => index,
            (
                ClockConfiguration::TimestampedEvent { .. },
                AnchorRef::Timestamp { instant_nanos },
            ) => instant_nanos,
            _ => return Err(NativeInputError::ObservationAnchor),
        };
        if admitted != expected {
            return Err(NativeInputError::ObservationAnchor);
        }
        Some(position.position)
    } else {
        None
    };
    Ok(NativeTlInput {
        request,
        observation,
        limits,
        occurrence: occurrence.clone(),
        binding,
        anchor,
    })
}
