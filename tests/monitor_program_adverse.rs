// SPDX-License-Identifier: AGPL-3.0-or-later
//! Producer preflight for AC3; driver-to-RT construction remains unqualified.

#[path = "support/monitor_program_adverse.rs"]
mod producer;
mod support;

use std::collections::BTreeSet;

use ix_trace_rs::trace;
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2, CheckedTerm};

fn reachable_modulo(package: &CheckedPackageV2, root: &CheckedNodeId) -> Vec<CheckedNodeId> {
    let mut pending = vec![root.clone()];
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let term = package.term(&id).expect("the admitted closure resolves");
        if matches!(term, CheckedTerm::Application { operation, .. } if operation.identity() == Some("quire.op.integer.mod"))
        {
            found.push(id.clone());
        }
        pending.extend(term.reference_targets().into_iter().cloned());
    }
    found
}

/// This asserts actual producer selection and mutation, not RT construction.
#[trace("TC-100")]
#[test]
fn producer_rekeys_a_genuinely_reachable_unsupported_initializer() {
    let healthy = producer::Produced::healthy();
    let (repeated, repeated_package) = healthy.entry_emission(false);
    assert_eq!(
        repeated.package().bytes(),
        healthy.root.emission.package().bytes(),
        "the unmodified public entry route reproduces the actual producer output"
    );
    let original = healthy.package.monitor_plans().next().unwrap();
    let same = repeated_package.monitor_plans().next().unwrap();
    assert_eq!(original.structural_key(), same.structural_key());
    assert!(reachable_modulo(&healthy.package, original.init().node_id()).is_empty());
    let (changed_emission, changed_package) = healthy.entry_emission(true);
    let changed = changed_package.monitor_plans().next().unwrap();
    assert_ne!(
        original.structural_key(),
        changed.structural_key(),
        "a changed entry has a new genuine content identity"
    );
    assert_ne!(
        original.init().structural_key(),
        changed.init().structural_key()
    );
    assert_eq!(
        healthy.package.structural_key(original.clause()),
        changed_package.structural_key(changed.clause()),
        "the original checked clause is retained"
    );
    let modulo = reachable_modulo(&changed_package, changed.init().node_id());
    assert_eq!(
        modulo.len(),
        1,
        "one actual modulo in the selected initializer closure"
    );
    assert!(
        !changed_emission
            .equality_producer_identity_rows()
            .is_empty(),
        "the changed emission retains its original producer witnesses"
    );
}
