// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent decision-table predicates over actual owner values.

use std::collections::BTreeSet;

use qsl_eval::temporal::{
    evidence::{Settlement, Truth},
    ProgressClosure,
};
use quire_mltl::{
    contract_ir::{MappedOutcome, NonValueKind, ValidatedMappedResult},
    report::{AssessmentExecution, ResultRelationKind, SettlementBasis, TemporalTruth},
};
use quire_observation::authority::{availability, completeness, OpenClosed};
use quire_spec_language::temporal::{Report, ReportOutcome};

use super::{
    Budget, TlJoinCause as Cause, TlJoinDecision as Decision, TlJoinError, TlJoinOwnerRefusals,
    ValidatedTlJoin,
};
use crate::temporal::ValidatedTlProjection;

pub(super) struct Compared {
    pub decision: Decision,
    pub causes: Vec<Cause>,
    pub value: Option<bool>,
}

pub(super) fn rows(
    projection: &ValidatedTlProjection<'_>,
    native: Option<&Report>,
    mapped: Option<&ValidatedMappedResult>,
    missing_result: bool,
    refusals: &TlJoinOwnerRefusals,
    predecessor_conflict: bool,
    previous: Option<&ValidatedTlJoin<'_>>,
    budget: &mut Budget,
) -> Result<Compared, TlJoinError> {
    let source = super::source_result(mapped);
    let wrong_subject = source.is_some_and(|result| {
        result.request() != projection.request()
            || result.subject_identity() != projection.request().subject_identity()
            || result.correspondence_identity() != projection.request().correspondence_identity()
    });
    let refused = refusals.native.is_some()
        || refusals.mapped.is_some()
        || wrong_subject
        || native.is_some_and(|result| result.outcome() == ReportOutcome::Refused)
        || source.is_some_and(|result| result.execution() == AssessmentExecution::Refused);
    let failed = projection.native().request.execution
        == qsl_eval::temporal::evidence::ExecutionDisposition::Failed
        || source.is_some_and(|result| result.execution() == AssessmentExecution::Failed);
    let unavailable = missing_result
        || mapped.is_some_and(|result| {
            matches!(
                result.outcome(),
                MappedOutcome::NonValue {
                    reason: NonValueKind::Unavailable
                }
            )
        })
        || native.is_some_and(|result| {
            matches!(
                result.outcome(),
                ReportOutcome::Inactive
                    | ReportOutcome::Unactivated
                    | ReportOutcome::ActivationUnknown
            )
        })
        || source
            .is_some_and(|result| result.availability().state() != availability::State::Available);
    let unsupported =
        source.is_some_and(|result| result.execution() == AssessmentExecution::Unsupported);
    let incomplete = native.is_some_and(|result| {
        matches!(
            result.outcome(),
            ReportOutcome::Missing
                | ReportOutcome::Indeterminate
                | ReportOutcome::Limit
                | ReportOutcome::Meter
                | ReportOutcome::CoveredPositions
                | ReportOutcome::WindowLimit
                | ReportOutcome::EvictedValuation
        )
    }) || source.is_some_and(|result| {
        result.execution() == AssessmentExecution::ResourceIncomplete
            || result.completeness().state() == completeness::State::Incomplete
    });
    let mut equal_final = None;
    let mut equal_pending = false;
    let mut mismatch = predecessor_conflict;
    let mut tl_predecessor_authority_pending = false;
    if let (Some(native), Some(mapped)) = (native, mapped) {
        let result = mapped.source();
        let original_native = native.predecessor().is_none();
        let original_tl = result.relation_kind() == ResultRelationKind::Original;
        if original_native != original_tl || (original_native && previous.is_some()) {
            mismatch = true;
        } else if !original_native {
            if previous.is_none() {
                mismatch = true;
            } else {
                // Successful corrected agreement still needs full TL authority,
                // but it cannot mask an independently established decision row.
                tl_predecessor_authority_pending = true;
            }
        }
        let support_equal = support_equal(projection, native, result.decision_support(), budget)?;
        let premises_equal = axes_equal(projection, mapped)
            && settlement_equal(native.settlement(), result.settlement())
            && support_equal;
        let native_final = native
            .truth()
            .and_then(|truth| super::final_truth(truth, native.settlement()));
        let tl_final = match mapped.outcome() {
            MappedOutcome::Value { value } if result.is_final() => Some(value),
            MappedOutcome::Value { .. } | MappedOutcome::NonValue { .. } => None,
        };
        let both_pending = native.truth() == Some(Truth::Pending)
            && result.truth() == TemporalTruth::Pending
            && native.settlement() == Some(Settlement::Unsettled)
            && result.settlement() == SettlementBasis::Unsettled;
        let contradicted = result.completeness().state() == completeness::State::Contradicted;
        if let (Some(left), Some(right)) = (native_final, tl_final) {
            mismatch |= left != right || !premises_equal || contradicted;
            if left == right && premises_equal && !contradicted && !mismatch {
                equal_final = Some(left);
            }
        } else if both_pending {
            let open = projection.native().request.decision_closure == Some(ProgressClosure::Open)
                && result.decision_scope_closure().state() == OpenClosed::Open;
            mismatch |= !premises_equal || !open || contradicted;
            equal_pending = premises_equal && open && !contradicted && !mismatch;
        } else if contradicted {
            mismatch = true;
        }
    }
    let predicates = [
        (refused, Decision::Refused, Cause::ResultRefused),
        (failed, Decision::Failed, Cause::ResultFailed),
        (unavailable, Decision::Unavailable, Cause::ResultUnavailable),
        (unsupported, Decision::Unsupported, Cause::ResultUnsupported),
        (incomplete, Decision::Incomplete, Cause::ResultIncomplete),
        (mismatch, Decision::Conflict, Cause::ResultMismatch),
    ];
    let mut decision = None;
    let mut causes = Vec::new();
    for (matched, row, cause) in predicates {
        if matched {
            budget.cause()?;
            causes.push(cause);
            decision.get_or_insert(row);
        }
    }
    if equal_pending {
        budget.cause()?;
        causes.push(Cause::ResultPending);
    }
    if tl_predecessor_authority_pending && decision.is_none() {
        return Err(TlJoinError::PredecessorAuthority {
            native_encoding_usage: budget.native_encoding_usage,
            native_predecessor_usage: budget.native_predecessor_usage,
        });
    }
    let decision = decision
        .or_else(|| equal_final.map(|_| Decision::EqualFinal))
        .or_else(|| equal_pending.then_some(Decision::EqualPending))
        .ok_or(TlJoinError::ResultMismatch)?;
    Ok(Compared {
        decision,
        causes,
        value: if decision == Decision::EqualFinal {
            equal_final
        } else {
            None
        },
    })
}

fn support_equal(
    projection: &ValidatedTlProjection<'_>,
    native: &Report,
    tl: &[String],
    budget: &mut Budget,
) -> Result<bool, TlJoinError> {
    let mut native_identities = BTreeSet::new();
    for support in native.support() {
        budget.support()?;
        let Some(position) = projection
            .native()
            .request
            .positions
            .get(support.row.position)
        else {
            return Ok(false);
        };
        native_identities.insert(position.observation.identity.as_str());
    }
    let mut tl_identities = BTreeSet::new();
    for identity in tl {
        budget.support()?;
        tl_identities.insert(identity.as_str());
    }
    Ok(native_identities == tl_identities)
}

fn axes_equal(projection: &ValidatedTlProjection<'_>, mapped: &ValidatedMappedResult) -> bool {
    let source = mapped.source();
    let expected = projection.request();
    let execution_equal = match projection.native().request.execution {
        qsl_eval::temporal::evidence::ExecutionDisposition::Completed => {
            source.execution() == AssessmentExecution::Completed
        }
        qsl_eval::temporal::evidence::ExecutionDisposition::Failed => {
            source.execution() == AssessmentExecution::Failed
        }
    };
    execution_equal
        && source.request() == expected
        && source.decision_scope_progress() == expected.decision_scope_progress()
        && source.decision_scope_closure() == expected.decision_scope_closure()
        && source.surrounding_execution_progress() == expected.surrounding_execution_progress()
        && source.surrounding_execution_closure() == expected.surrounding_execution_closure()
        && source.completeness() == expected.completeness()
        && source.availability() == expected.availability()
}

fn settlement_equal(native: Option<Settlement>, tl: SettlementBasis) -> bool {
    match (native, tl) {
        (Some(Settlement::ClosedScope), SettlementBasis::ClosedScope)
        | (Some(Settlement::DecisiveWitness), SettlementBasis::DecisiveWitness)
        | (Some(Settlement::DecisiveCounterexample), SettlementBasis::DecisiveCounterexample)
        | (Some(Settlement::Unsettled), SettlementBasis::Unsettled)
        | (Some(Settlement::Unavailable) | None, SettlementBasis::Unavailable) => true,
        _ => false,
    }
}
