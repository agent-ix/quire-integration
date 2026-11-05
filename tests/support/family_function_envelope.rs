// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact observed whole-call envelope comparison shared by function cohorts.
use quire_eval::{Evaluation, FamilyOutcome};
use quire_exact::{Outcome, Value};

/// Compare every public evaluation field while retaining the measured value variants.
pub fn same_envelope(expected: &Evaluation, actual: &Evaluation) {
    assert_eq!(expected.location, actual.location, "entire result location");
    assert_eq!(expected.losses, actual.losses, "ordered located losses");
    match (&expected.outcome, &actual.outcome) {
        (
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(expected))),
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(actual))),
        ) => assert_eq!(expected, actual),
        (
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(expected))),
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(actual))),
        ) => assert_eq!(expected, actual),
        (
            FamilyOutcome::Evaluated(Outcome::Incomplete(expected)),
            FamilyOutcome::Evaluated(Outcome::Incomplete(actual)),
        ) => assert_eq!(expected, actual),
        _ => panic!(
            "required Boolean/Integer completion or exact incomplete shape: {:?} versus {:?}",
            expected.outcome, actual.outcome
        ),
    }
}

