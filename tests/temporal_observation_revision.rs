// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-114: genuine QObs historical bundle and exact observation admission.

#[path = "support/observation_revision/mod.rs"]
mod support;

use ix_trace_rs::trace;
use quire_integration::temporal::{
    read_observation_revision, ObservationAdmissionCode, ObservationAdmissionError,
    ObservationAdmissionLimit, ObservationAdmissionLimits,
};
use quire_observation::authority::{self as a, bundle, ErrorCode, Limits};
use support::Fixture;

fn limits() -> ObservationAdmissionLimits {
    ObservationAdmissionLimits::default()
}
fn owner_limits() -> Limits {
    Limits::owner_max()
}
fn initial(f: &Fixture) -> bundle::Publication {
    bundle::publish(f.context(1, None), &f.selection(None), None, owner_limits()).unwrap()
}
fn owner_code(error: ObservationAdmissionError) -> ErrorCode {
    match error {
        ObservationAdmissionError::OwnerRead(error) => error.code(),
        other => panic!("expected native owner refusal: {other:?}"),
    }
}

#[trace("TC-114", "IT-021-SC-01", "IT-021-SC-02")]
#[test]
fn initial_and_corrected_receipts_retain_exact_owner_values_and_prior_bytes() {
    let f = Fixture::new();
    let first = initial(&f);
    let observation = f.observation(f.observation_document(), 1, None);
    let first_receipt = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &observation,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap();
    assert_eq!(first_receipt.bundle(), first.view());
    assert!(std::ptr::eq(first_receipt.observation(), &observation));
    assert!(first_receipt.predecessor().is_none());
    let saved = first_receipt.bundle().bytes().to_vec();
    let revised_doc = a::observation::derive(
        f.context(2, Some(f.observation_document())),
        &f.observation_selection(),
        owner_limits(),
    )
    .unwrap();
    let revised_observation = f.observation(&revised_doc, 2, Some(f.observation_document()));
    let selection = f.selection(Some(&revised_doc));
    let corrected = bundle::publish(
        f.context(2, Some(first.document())),
        &selection,
        Some(first.lineage()),
        owner_limits(),
    )
    .unwrap();
    let receipt = read_observation_revision(
        corrected.document().bytes(),
        f.context(2, Some(first.document())),
        &selection,
        &revised_observation,
        Some(&first_receipt),
        owner_limits(),
        limits(),
    )
    .unwrap();
    assert_eq!(receipt.bundle(), corrected.view());
    assert!(std::ptr::eq(receipt.observation(), &revised_observation));
    assert!(std::ptr::eq(receipt.predecessor().unwrap(), &first_receipt));
    assert_eq!(receipt.predecessor().unwrap().bundle().bytes(), saved);
    assert_ne!(observation.bytes(), revised_observation.bytes());
    assert_eq!(observation.payload(), revised_observation.payload());
}

#[trace("TC-114", "IT-021-SC-01", "IT-021-SC-02")]
#[test]
fn corrected_bundle_can_retain_unchanged_observation() {
    let f = Fixture::new();
    let first = initial(&f);
    let observation = f.observation(f.observation_document(), 1, None);
    let prior = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &observation,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap();
    let second = bundle::publish(
        f.context(2, Some(first.document())),
        &f.selection(None),
        Some(first.lineage()),
        owner_limits(),
    )
    .unwrap();
    let admitted = read_observation_revision(
        second.document().bytes(),
        f.context(2, Some(first.document())),
        &f.selection(None),
        &observation,
        Some(&prior),
        owner_limits(),
        limits(),
    )
    .unwrap();
    assert_eq!(admitted.observation().revision(), 1);
    assert_eq!(admitted.bundle().revision(), 2);
    assert_eq!(admitted.observation().bytes(), observation.bytes());
}

#[trace("TC-114", "IT-021-SC-02")]
#[test]
fn equal_content_revision_and_foreign_owner_observations_are_not_members() {
    let f = Fixture::new();
    let first = initial(&f);
    let revised_doc = a::observation::derive(
        f.context(2, Some(f.observation_document())),
        &f.observation_selection(),
        owner_limits(),
    )
    .unwrap();
    let revised = f.observation(&revised_doc, 2, Some(f.observation_document()));
    let original = f.observation(f.observation_document(), 1, None);
    assert_eq!(original.payload(), revised.payload());
    let error = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &revised,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap_err();
    assert_eq!(error.code(), ObservationAdmissionCode::ObservationMismatch);
    let mut foreign_owner = f.owner.clone();
    foreign_owner.definition_identity = support::id("foreign-owner");
    let context = a::Context::new(
        a::History::batch(&f.qualified),
        &foreign_owner,
        &f.subject,
        1,
        None,
    );
    let foreign_doc =
        a::observation::derive(context, &f.observation_selection(), owner_limits()).unwrap();
    let foreign = a::observation::read(
        foreign_doc.bytes(),
        context,
        &f.observation_selection(),
        owner_limits(),
    )
    .unwrap();
    let error = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &foreign,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap_err();
    assert_eq!(error.code(), ObservationAdmissionCode::ObservationMismatch);
}

#[trace("TC-114", "IT-021-SC-01", "IT-021-SC-03")]
#[test]
fn missing_foreign_and_non_direct_predecessors_preserve_native_refusal() {
    let f = Fixture::new();
    let first = initial(&f);
    let observation = f.observation(f.observation_document(), 1, None);
    let prior = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &observation,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap();
    let second = bundle::publish(
        f.context(2, Some(first.document())),
        &f.selection(None),
        Some(first.lineage()),
        owner_limits(),
    )
    .unwrap();
    let next = read_observation_revision(
        second.document().bytes(),
        f.context(2, Some(first.document())),
        &f.selection(None),
        &observation,
        Some(&prior),
        owner_limits(),
        limits(),
    )
    .unwrap();
    let third = bundle::publish(
        f.context(3, Some(second.document())),
        &f.selection(None),
        Some(second.lineage()),
        owner_limits(),
    )
    .unwrap();
    for supplied in [None, Some(&next)] {
        let error = read_observation_revision(
            second.document().bytes(),
            f.context(2, Some(first.document())),
            &f.selection(None),
            &observation,
            supplied,
            owner_limits(),
            limits(),
        )
        .unwrap_err();
        assert_eq!(owner_code(error), ErrorCode::PredecessorMismatch);
    }
    let error = read_observation_revision(
        third.document().bytes(),
        f.context(3, Some(second.document())),
        &f.selection(None),
        &observation,
        Some(&prior),
        owner_limits(),
        limits(),
    )
    .unwrap_err();
    assert_eq!(owner_code(error), ErrorCode::PredecessorMismatch);
    let error = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &observation,
        Some(&prior),
        owner_limits(),
        limits(),
    )
    .unwrap_err();
    assert_eq!(owner_code(error), ErrorCode::PredecessorMismatch);
}

#[trace("TC-114", "IT-021-SC-03")]
#[test]
fn malformed_noncanonical_and_trailing_bytes_keep_exact_native_errors() {
    let f = Fixture::new();
    let first = initial(&f);
    let observation = f.observation(f.observation_document(), 1, None);
    let mut trailing = first.document().bytes().to_vec();
    trailing.push(b' ');
    let pretty = serde_json::to_vec_pretty(
        &serde_json::from_slice::<serde_json::Value>(first.document().bytes()).unwrap(),
    )
    .unwrap();
    for bytes in [b"{".as_slice(), trailing.as_slice(), pretty.as_slice()] {
        let native = bundle::read_revision(
            bytes,
            f.context(1, None),
            &f.selection(None),
            None,
            owner_limits(),
        )
        .unwrap_err();
        let error = read_observation_revision(
            bytes,
            f.context(1, None),
            &f.selection(None),
            &observation,
            None,
            owner_limits(),
            limits(),
        )
        .unwrap_err();
        match error {
            ObservationAdmissionError::OwnerRead(actual) => {
                assert_eq!(actual.code(), native.code());
                assert_eq!(actual.usage(), native.usage());
            }
            other => panic!("owner error expected: {other:?}"),
        }
    }
}

#[trace("TC-114", "IT-021-SC-04")]
#[test]
fn exact_limits_admit_one_less_refuses_before_excess_action_and_counters_reset() {
    let f = Fixture::new();
    let first = initial(&f);
    let observation = f.observation(f.observation_document(), 1, None);
    let admitted = read_observation_revision(
        first.document().bytes(),
        f.context(1, None),
        &f.selection(None),
        &observation,
        None,
        owner_limits(),
        limits(),
    )
    .unwrap();
    let exact = ObservationAdmissionLimits {
        max_bytes: u64::try_from(first.document().bytes().len() + observation.bytes().len())
            .unwrap(),
        max_work: admitted.work(),
    };
    assert_eq!(admitted.work(), 12);
    for _ in 0..2 {
        let receipt = read_observation_revision(
            first.document().bytes(),
            f.context(1, None),
            &f.selection(None),
            &observation,
            None,
            owner_limits(),
            exact,
        )
        .unwrap();
        assert_eq!(receipt.work(), 12);
    }
    for (bounded, limit, value, setting) in [
        (
            ObservationAdmissionLimits {
                max_bytes: exact.max_bytes - 1,
                ..exact
            },
            ObservationAdmissionLimit::Bytes,
            exact.max_bytes,
            exact.max_bytes - 1,
        ),
        (
            ObservationAdmissionLimits {
                max_work: exact.max_work - 1,
                ..exact
            },
            ObservationAdmissionLimit::Work,
            exact.max_work,
            exact.max_work - 1,
        ),
    ] {
        let error = read_observation_revision(
            first.document().bytes(),
            f.context(1, None),
            &f.selection(None),
            &observation,
            None,
            owner_limits(),
            bounded,
        )
        .unwrap_err();
        assert!(
            matches!(error,ObservationAdmissionError::LimitReached {limit: actual_limit,value: actual_value,setting:actual_setting} if actual_limit==limit && actual_value==value && actual_setting==setting)
        );
    }
    let error = read_observation_revision(
        b"{",
        f.context(1, None),
        &f.selection(None),
        &observation,
        None,
        owner_limits(),
        ObservationAdmissionLimits {
            max_bytes: 0,
            max_work: 0,
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ObservationAdmissionCode::LimitReached);
    for code in ObservationAdmissionCode::all() {
        assert_eq!(
            ObservationAdmissionCode::from_code(code.as_str()),
            Some(*code)
        );
    }
    assert_eq!(ObservationAdmissionCode::from_code("unknown"), None);
}
