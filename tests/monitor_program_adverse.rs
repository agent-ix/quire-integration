// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real producer/IR/driver diagnostic boundaries; runtime construction remains unqualified.

#[path = "support/monitor_program_adverse.rs"]
mod producer;
mod support;

use std::collections::BTreeSet;

use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedTerm,
};
use quire_driver::monitor_refusal::{
    target_mismatch_at, unsupported_at, MonitorLocusErrorCode, TargetPremise,
};
use quire_exact::{Integer, Rational};

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
        healthy.root.emission().package().bytes(),
        "the unmodified public entry route reproduces the actual producer output"
    );
    let original = healthy.package.monitor_plans().next().unwrap();
    let same = repeated_package.monitor_plans().next().unwrap();
    assert_eq!(original.structural_key(), same.structural_key());
    assert!(reachable_modulo(&healthy.package, original.init().node_id()).is_empty());
    // This constructor locates the original premise; it does not compare a
    // target, refuse preparation, or decide runtime support.
    let independent_target = producer::target();
    assert_eq!(
        independent_target.uncertainty,
        Rational::from_integer(Integer::zero())
    );
    assert_eq!(original.uncertainty().numerator.as_ref(), "0");
    assert_eq!(original.uncertainty().denominator.as_ref(), "1");
    let root_node = healthy
        .package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == original.node_id())
        .unwrap();
    let [generated] = root_node.occurrences.as_slice() else {
        panic!("one genuine generated root occurrence");
    };
    assert_eq!(
        *generated,
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Generated,
            ordinal: 0
        }
    );
    let diagnostic = target_mismatch_at(
        &healthy.package,
        original.node_id(),
        TargetPremise::Uncertainty,
        None,
        Some(generated),
    )
    .unwrap();
    assert_eq!(diagnostic.premise, TargetPremise::Uncertainty);
    assert_eq!(diagnostic.locus.node, *original.node_id());
    assert_eq!(diagnostic.locus.structural_key, *original.structural_key());
    assert_eq!(diagnostic.locus.body_path, vec![14, 0]);
    assert_eq!(diagnostic.locus.occurrence.as_ref(), Some(generated));
    let CheckedTerm::Aggregate { members } = original.body() else {
        panic!("actual root bindings");
    };
    let CheckedTerm::Binding { name, value } = &members[14] else {
        panic!("actual uncertainty binding");
    };
    assert_eq!(name.as_ref(), "uncertainty");
    assert!(
        matches!(value.as_ref(), CheckedTerm::Literal { value, .. } if value.as_str() == Some("0"))
    );
    let unselected = target_mismatch_at(
        &healthy.package,
        original.node_id(),
        TargetPremise::Uncertainty,
        None,
        None,
    )
    .unwrap();
    assert_eq!(unselected.locus.occurrence, None);
    let foreign = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    assert_eq!(
        target_mismatch_at(
            &healthy.package,
            original.node_id(),
            TargetPremise::Uncertainty,
            None,
            Some(&foreign)
        )
        .unwrap_err()
        .code,
        MonitorLocusErrorCode::OccurrenceMismatch
    );
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
    let node = &modulo[0];
    let (source_node, kind) = changed_package
        .resolved_graph()
        .nodes
        .iter()
        .zip(changed_package.node_kinds())
        .find(|(original, _)| &original.node_id == node)
        .unwrap();
    let CheckedTerm::Application {
        operation,
        arguments,
        ..
    } = changed_package.term(node).unwrap()
    else {
        panic!("the actual reachable modulo is a root application");
    };
    assert_eq!(operation.identity(), Some("quire.op.integer.mod"));
    let [occurrence] = source_node.occurrences.as_slice() else {
        panic!("one exact source occurrence for the selected modulo");
    };
    let diagnostic = unsupported_at(&changed_package, node, &[], *kind, Some(occurrence)).unwrap();
    assert_eq!(diagnostic.locus.node, *node);
    assert_eq!(
        diagnostic.locus.structural_key,
        *changed_package.structural_key(node).unwrap()
    );
    assert_eq!(diagnostic.locus.body_path, Vec::<usize>::new());
    assert_eq!(diagnostic.locus.occurrence.as_ref(), Some(occurrence));
    assert_eq!(diagnostic.kind, *kind);
    assert_eq!(diagnostic.operation.as_ref(), Some(operation));
    assert_eq!(
        unsupported_at(
            &changed_package,
            node,
            &[arguments.len()],
            *kind,
            Some(occurrence)
        )
        .unwrap_err()
        .code,
        MonitorLocusErrorCode::InvalidBodyPath
    );
    assert!(
        !changed_emission
            .equality_producer_identity_rows()
            .is_empty(),
        "the changed emission retains its original producer witnesses"
    );
}

/// Missing entry bytes stop at strict admission, before any driver diagnostic or runtime call.
#[trace("TC-100")]
#[test]
fn missing_actual_initializer_body_is_a_native_ir_refusal() {
    let healthy = producer::Produced::healthy();
    let plan = healthy.package.monitor_plans().next().unwrap();
    let index = healthy
        .package
        .resolved_graph()
        .nodes
        .iter()
        .position(|node| &node.node_id == plan.init().node_id())
        .expect("the selected initializer has its original document holder");
    let mut wire: serde_json::Value =
        serde_json::from_slice(healthy.root.emission().package().bytes()).unwrap();
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    assert!(nodes[index]
        .as_object_mut()
        .unwrap()
        .remove("body")
        .is_some());
    // Keep every original identity, reference and other member unchanged.
    let bytes = serde_json::to_vec(&wire).unwrap();
    let CheckedPackageDispatchResult::Refused(refused) = healthy.read_bytes(&bytes) else {
        panic!("the missing selected entry body must refuse at IR ingress");
    };
    assert_eq!(refused.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(refused.cause, None);
    assert_eq!(refused.loci.len(), 1);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{index}")
    );
    assert_eq!(
        refused.locus, None,
        "closed decoding produces no admitted node identity"
    );
    assert!(
        matches!(
            healthy.read_bytes(healthy.root.emission().package().bytes()),
            CheckedPackageDispatchResult::AdmittedV2(_)
        ),
        "the original producer bytes remain healthy after the adverse read"
    );
}
