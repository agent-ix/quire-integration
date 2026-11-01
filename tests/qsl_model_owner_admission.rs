// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-012: a package QSL emits for a unit that selects a domain-package model
//! is admitted by contract IR's checked-package reader, and its model nodes
//! are keyed by the content-only `ModelOwner`.
//!
//! The unit selects `acme/orders`, the domain package document QSL authors at
//! `tests/fixtures/spine-model.semantic-ir.json`; that file is read from the
//! `quire-spec-language` checkout Cargo resolved, never copied here.

mod support;

use std::{collections::BTreeMap, fs};

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{
    check::PackageDeclarations,
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedNodeOwner, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedTerm,
};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::Value;

/// The version the authored domain package document carries.
const AUTHORED_VERSION: &str = "1.0.0";

/// The domain package identity and its declarations the unit uses.
const IDENTITY: &str = "acme/orders";
const DECLARATIONS: [&str; 2] = ["Gadget", "Widget"];

/// QSL's authored domain package document with only `package.version` set to
/// `version`.
fn domain_package(version: &str) -> Vec<u8> {
    let path = support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let mut document: Value = serde_json::from_str(&text).expect("the fixture is JSON");
    document["package"]["version"] = Value::from(version);
    let bytes = serde_json::to_vec(&document).expect("the document serializes");
    let read_back: Value = serde_json::from_slice(&bytes).expect("the relabelled document is JSON");
    assert_eq!(read_back["package"]["version"], version);
    bytes
}

/// A unit selecting the domain package by its declared identity.
fn unit_selecting() -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model M = \"{IDENTITY}\";\n\
         function keep using v(g: M::Gadget): M::Gadget pure {{ g }}\n\
         function held using v(w: Reference<M::Widget>): Boolean pure {{ true }}\n\
         function step using v(x: Int[0, 9]): Int[1, 10] pure {{ x + 1 }}\n\
         function invert using v(flag: Boolean): Boolean pure {{ not flag }}\n"
    )
}

/// QSL's emitted package for `unit`: parse, form, model intake, assemble,
/// check, link and emit, through the crates' public APIs.
fn emit(unit: &str, document: &[u8]) -> Vec<u8> {
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "test:model-owner", "fixture", "fixture:1"),
        "program.native",
        unit.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 admits the unit");
    assert_eq!(parsed.diagnostics(), []);
    let raw = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the unit");
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document]).expect("domain package has its declared identity"),
        ModelNormalizationLimits::default(),
    )
    .expect("the domain package admits");
    let graph =
        PackageDeclarations::assemble(raw, "agent-ix/it-012".to_owned(), unit, models, Vec::new())
            .expect("the unit assembles")
            .check(CheckingLimits::default())
            .expect("the package checks");
    let emission = emit_checked(&CheckedPackage::link(graph)).expect("the package emits");
    assert_eq!(emission.omitted(), []);
    emission.package().bytes().to_vec()
}

/// Supply the selected domain document by identity and the required features.
fn evidence(bytes: &[u8], identity: &str, document: &[u8]) -> CheckedPackageEvidence {
    let wire: Value = serde_json::from_slice(bytes).expect("the emitted package is JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"].as_array().unwrap() {
        evidence.support_feature(feature.as_str().unwrap());
    }
    evidence
        .insert_domain_package_document(identity, document)
        .expect("the domain document is valid evidence");
    evidence
}

fn read(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, CheckedPackageReadLimits::bounded(), evidence)
}

/// The emitted package for a unit selecting the authored document at
/// `version`, admitted by IR's reader.
fn admitted(version: &str) -> CheckedPackageV2 {
    let document = domain_package(version);
    let bytes = emit(&unit_selecting(), &document);
    let evidence = evidence(&bytes, IDENTITY, &document);
    match read(&bytes, &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("expected AdmittedV2, got {other:?}"),
    }
}

/// Domain selections retained by the admitted model-root owners.
fn model_identities(package: &CheckedPackageV2) -> Vec<&str> {
    package
        .graph()
        .nodes
        .iter()
        .filter_map(|node| match &node.owner {
            Some(CheckedNodeOwner::ModelRoot { identity }) => Some(identity.as_ref()),
            _ => None,
        })
        .collect()
}

/// The admitted structural identities of declarations owned by the model.
fn model_node_ids(package: &CheckedPackageV2) -> BTreeMap<String, CheckedNodeId> {
    package
        .graph()
        .nodes
        .iter()
        .filter_map(|node| match (&*node.node_tag, &node.owner) {
            (
                "model",
                Some(CheckedNodeOwner::Model {
                    identity,
                    node: owner_node,
                }),
            ) => {
                assert_eq!(&**identity, IDENTITY);
                Some((owner_node.to_string(), node.node_id.clone()))
            }
            ("model", Some(CheckedNodeOwner::ModelRoot { .. })) => None,
            ("model", owner) => panic!("model node has unexpected owner: {owner:?}"),
            _ => None,
        })
        .collect()
}

fn operation_nodes(
    package: &CheckedPackageV2,
    operation: &str,
) -> Vec<quire_contract_ir::CheckedNodeId> {
    let node_ids = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            let Some(CheckedTerm::Application {
                operation: descriptor,
                ..
            }) = package.term(&node.node_id)
            else {
                return false;
            };
            descriptor.identity() == Some(operation)
        })
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(node_ids.len(), 1, "one checked {operation} node");
    node_ids
}

fn generated_scalar_claims(
    package: &CheckedPackageV2,
    operation: &str,
) -> Vec<quire_contract_codegen::ExactScalarClaim> {
    let node_ids = operation_nodes(package, operation);
    let derived = quire_contract_codegen::derive_exact_scalar_items(package, &node_ids);
    let [item] = derived.as_slice() else {
        panic!("one derived scalar item");
    };
    let item = item.clone().expect("checked operation has a scalar plan");
    let generated = quire_contract_codegen::generate_exact_scalar_oracles(package, &[item])
        .expect("CG renders the checked scalar operation");
    generated.claim_map.items
}

/// IT-012-SC-01: IR's v2 reader admits the package QSL emitted for a unit
/// that selects a domain-package model, and the lock names that selection.
#[test]
#[trace("IT-012-SC-01")]
fn a_model_bearing_package_emitted_by_qsl_is_admitted_by_irs_reader() {
    let package = admitted(AUTHORED_VERSION);
    assert_eq!(model_identities(&package), [IDENTITY]);
    let declared: Vec<&str> = package
        .graph()
        .nodes
        .iter()
        .filter_map(|node| node.declaration.as_ref())
        .map(|declaration| declaration.qualified_name.last().unwrap().as_ref())
        .collect();
    for function in ["keep", "held"] {
        assert!(declared.contains(&function), "{function}: {declared:?}");
    }
}

/// IT-012-SC-01: a document supplied under a foreign identity cannot satisfy
/// the selected model import.
#[test]
#[trace("IT-012-SC-01")]
fn a_domain_package_supplied_under_another_identity_is_refused() {
    let document = domain_package(AUTHORED_VERSION);
    let bytes = emit(&unit_selecting(), &document);
    let wire: Value = serde_json::from_slice(&bytes).unwrap();
    let root_position = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["owner"]["kind"] == "model_root")
        .expect("QSL emitted its selected model root");
    let expected_path = format!("/semantic_graph/nodes/{root_position}/owner/identity");
    let evidence = evidence(&bytes, "foreign/orders", &document);
    let CheckedPackageDispatchResult::Refused(refusal) = read(&bytes, &evidence) else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        refusal
            .loci
            .first()
            .and_then(|locus| locus.path.as_ref())
            .map(ToString::to_string)
            .as_deref(),
        Some(expected_path.as_str())
    );
}

/// IT-012-SC-02: each admitted model node has its selected model declaration
/// as owner, with no version member in that owner.
#[test]
#[trace("IT-012-SC-02")]
fn model_nodes_are_owned_by_the_content_only_model_owner() {
    let package = admitted(AUTHORED_VERSION);
    let nodes = model_node_ids(&package);
    let expected = DECLARATIONS
        .iter()
        .map(|declaration| format!("ix://{IDENTITY}/{declaration}"))
        .collect::<Vec<_>>();
    assert_eq!(nodes.keys().cloned().collect::<Vec<_>>(), expected);
}

/// IT-012-SC-03: a version-only change of the domain package leaves the
/// identity selection and every model node's id unchanged.
#[test]
#[trace("IT-012-SC-03")]
fn a_version_only_change_of_the_domain_package_keys_the_same_model_nodes() {
    let (first, second) = (admitted(AUTHORED_VERSION), admitted("2.0.0"));
    assert_eq!(model_identities(&second), model_identities(&first));
    assert_eq!(model_node_ids(&second), model_node_ids(&first));
    assert_eq!(model_node_ids(&second).len(), DECLARATIONS.len());
}

#[test]
#[trace("IT-013-SC-02")]
fn v2_checked_integer_operation_generates_a_cg_scalar_claim() {
    let package = admitted(AUTHORED_VERSION);
    let claims = generated_scalar_claims(&package, "quire.op.integer.add");
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].operation.identity, "quire.op.integer.add");
    assert_eq!(
        claims[0].operation.provenance,
        quire_contract_codegen::OperationProvenance::IrConfirmed
    );
    let quire_contract_codegen::ClaimDisposition::Generated(generated) = &claims[0].result else {
        panic!("integer addition did not generate")
    };
    assert!(!generated.source_map.is_empty());
    assert!(!generated.checked_bounds.is_empty());
}

#[test]
#[trace("IT-013-SC-03")]
fn v2_checked_boolean_operation_generates_a_cg_scalar_claim() {
    let package = admitted(AUTHORED_VERSION);
    let node_ids = operation_nodes(&package, "quire.op.boolean.not");
    assert!(package
        .source_map()
        .iter()
        .any(|entry| entry.node_id == node_ids[0] && !entry.regions.is_empty()));
    let claims = generated_scalar_claims(&package, "quire.op.boolean.not");
    let [claim] = claims.as_slice() else {
        panic!("one generated Boolean claim: {claims:?}");
    };
    assert_eq!(claim.operation.identity, "quire.op.boolean.not");
    assert_eq!(
        claim.operation.provenance,
        quire_contract_codegen::OperationProvenance::IrConfirmed
    );
    let quire_contract_codegen::ClaimDisposition::Generated(generated) = &claim.result else {
        panic!("the admitted Boolean node generates: {:?}", claim.result);
    };
    assert!(!generated.source_map.is_empty());
}
