// SPDX-License-Identifier: AGPL-3.0-or-later
//! The bridge's own position-set logic over plain typed inputs.
//!
//! These checks run on observation identities and stamps, not on a QObs bundle,
//! so they hold for position sets the owner's bundle does not admit today: a
//! QObs capture environment covers every qualified record at one anchor and a
//! bundle holds each role once, so a genuine bundle carries a single stamp.

use std::collections::BTreeMap;

use super::{
    TlMissingPremise as Missing, TlProjectionError as Error, TlProjectionPremise as Premise,
};

/// Key records by observation identity; two with one identity are a native premise.
pub(super) fn unique_identities<'a, T>(
    items: impl IntoIterator<Item = (&'a str, T)>,
) -> Result<BTreeMap<&'a str, T>, Error> {
    let mut keyed = BTreeMap::new();
    for (identity, item) in items {
        if keyed.insert(identity, item).is_some() {
            return Err(Error::NativePremise(Premise::Observation));
        }
    }
    Ok(keyed)
}

/// The stamp of the single position whose observation identity equals the opaque
/// anchor. None is incomplete anchor; several is a native premise.
pub(super) fn anchor_stamp(positions: &[(&str, u64)], anchor: &str) -> Result<u64, Error> {
    let mut matching = positions
        .iter()
        .filter(|(identity, _)| *identity == anchor)
        .map(|(_, stamp)| *stamp);
    match (matching.next(), matching.next()) {
        (None, _) => Err(Error::Incomplete(Missing::Anchor)),
        (Some(stamp), None) => Ok(stamp),
        (Some(_), Some(_)) => Err(Error::NativePremise(Premise::Anchor)),
    }
}

/// Request indices in ascending stamp order over the half-open range `[start, end)`.
///
/// A stamp outside the range or repeated is a native premise; a range whose length
/// differs from the position count, or a gap in the ascending stamps, is incomplete.
pub(super) fn order_by_stamp(stamps: &[u64], start: u64, end: u64) -> Result<Vec<usize>, Error> {
    let mut ordered = BTreeMap::new();
    for (index, &stamp) in stamps.iter().enumerate() {
        if stamp < start || stamp >= end || ordered.insert(stamp, index).is_some() {
            return Err(Error::NativePremise(Premise::Position));
        }
    }
    let length = u64::try_from(stamps.len()).unwrap_or(u64::MAX);
    if end.checked_sub(start) != Some(length) {
        return Err(Error::Incomplete(Missing::Position));
    }
    let mut positions = Vec::with_capacity(ordered.len());
    for (offset, (stamp, index)) in ordered.into_iter().enumerate() {
        if stamp.checked_sub(start) != u64::try_from(offset).ok() {
            return Err(Error::Incomplete(Missing::Position));
        }
        positions.push(index);
    }
    Ok(positions)
}

/// The numeric TL anchor: the anchor position's stamp minus the range start.
pub(super) const fn tl_anchor(stamp: u64, start: u64) -> u64 {
    stamp - start
}

/// The unique element of `matches`; none or several is `error`.
pub(super) fn exactly_one<T>(
    matches: impl IntoIterator<Item = T>,
    error: Error,
) -> Result<T, Error> {
    let mut matches = matches.into_iter();
    match (matches.next(), matches.next()) {
        (Some(only), None) => Ok(only),
        _ => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn premise(error: &Error, expected: Premise) -> bool {
        matches!(error, Error::NativePremise(actual) if *actual == expected)
    }

    fn missing(error: &Error, expected: Missing) -> bool {
        matches!(error, Error::Incomplete(actual) if *actual == expected)
    }

    #[test]
    fn duplicate_observation_identity_is_a_native_premise() {
        let error = unique_identities([("a", 0), ("b", 1), ("a", 2)]).unwrap_err();
        assert!(premise(&error, Premise::Observation), "{error:?}");
        let keyed = unique_identities([("a", 0), ("b", 1)]).unwrap();
        assert_eq!(keyed.len(), 2);
    }

    #[test]
    fn the_anchor_selects_exactly_one_position() {
        let positions = [("a", 4), ("b", 5), ("c", 6)];
        assert_eq!(anchor_stamp(&positions, "b").unwrap(), 5);
        let none = anchor_stamp(&positions, "z").unwrap_err();
        assert!(missing(&none, Missing::Anchor), "{none:?}");
        let several = anchor_stamp(&[("a", 4), ("a", 4)], "a").unwrap_err();
        assert!(premise(&several, Premise::Anchor), "{several:?}");
    }

    #[test]
    fn tl_order_is_ascending_stamp_order_whatever_the_request_order() {
        assert_eq!(order_by_stamp(&[2, 1, 0], 0, 3).unwrap(), vec![2, 1, 0]);
        assert_eq!(order_by_stamp(&[0, 1, 2], 0, 3).unwrap(), vec![0, 1, 2]);
        assert_eq!(order_by_stamp(&[5, 3, 4], 3, 6).unwrap(), vec![1, 2, 0]);
    }

    #[test]
    fn stamps_outside_or_repeated_are_a_native_premise() {
        for stamps in [&[0, 0, 1][..], &[0, 1, 3], &[1, 2, 3]] {
            let error = order_by_stamp(stamps, 0, 3).unwrap_err();
            assert!(premise(&error, Premise::Position), "{stamps:?}: {error:?}");
        }
    }

    #[test]
    fn a_missing_sample_or_range_mismatch_is_incomplete() {
        for (stamps, start, end) in [(&[0, 2][..], 0, 3), (&[0, 1], 0, 3), (&[0], 0, 2)] {
            let error = order_by_stamp(stamps, start, end).unwrap_err();
            assert!(missing(&error, Missing::Position), "{stamps:?}: {error:?}");
        }
    }

    #[test]
    fn the_tl_anchor_is_offset_by_the_range_start() {
        assert_eq!(tl_anchor(7, 5), 2);
        assert_eq!(tl_anchor(5, 5), 0);
    }

    #[test]
    fn exactly_one_refuses_none_and_several() {
        let one = exactly_one([7], Error::NativePremise(Premise::Source)).unwrap();
        assert_eq!(one, 7);
        for items in [vec![], vec![1, 2]] {
            let error = exactly_one(items, Error::NativePremise(Premise::Source)).unwrap_err();
            assert!(premise(&error, Premise::Source), "{error:?}");
        }
    }
}
