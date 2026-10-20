// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-012: a package QSL emits for a unit that selects a domain-package model
//! is admitted by contract IR's checked-package reader, and its model nodes
//! are keyed by the content-only `ModelOwner`.
//!
//! The unit selects `acme/orders`, the domain package document QSL authors at
//! `tests/fixtures/spine-model.semantic-ir.json`; that file is read from the
//! `quire-spec-language` checkout Cargo resolved, never copied here.

// Shared with the other composed tests; this binary uses only `qsl_root`.
#[allow(dead_code)]
mod support;

use std::{collections::BTreeSet, fs};

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{
    check::PackageDeclarations,
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
        key::hex,
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

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

/// The `sha256-jcs` digest QSL keys `document` under.
fn document_digest(document: &[u8]) -> String {
    let packages = package_input([document]);
    let [(digest, _)] = packages.iter().collect::<Vec<_>>()[..] else {
        panic!("one supplied document");
    };
    hex(digest)
}

/// A unit whose `model M` selects `document` at `version`, using two of its
/// object types.
fn unit_selecting(version: &str, document: &[u8]) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model M = \"{IDENTITY}\" version \"{version}\" digest \"sha256-jcs:{}\";\n\
         function keep using v(g: M::Gadget): M::Gadget pure {{ g }}\n\
         function held using v(w: Reference<M::Widget>): Boolean pure {{ true }}\n",
        document_digest(document)
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
        &package_input([document]),
        ModelNormalizationLimits::default(),
    )
    .expect("the domain package admits");
    let graph = PackageDeclarations::assemble(raw, unit, models, Vec::new())
        .expect("the unit assembles")
        .check(CheckingLimits::default())
        .expect("the package checks");
    let emission = emit_checked(&CheckedPackage::link(graph)).expect("the package emits");
    assert_eq!(emission.omitted(), []);
    emission.package().bytes().to_vec()
}

/// What IR's reader holds about the domain package: its document under
/// `digest`, and every feature the emitted lock requires.
fn evidence(bytes: &[u8], digest: &str, document: &[u8]) -> CheckedPackageEvidence {
    let wire: Value = serde_json::from_slice(bytes).expect("the emitted package is JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"].as_array().unwrap() {
        evidence.support_feature(feature.as_str().unwrap());
    }
    evidence
        .insert_domain_package_document(digest, document)
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
    let bytes = emit(&unit_selecting(version, &document), &document);
    let evidence = evidence(&bytes, &document_digest(&document), &document);
    match read(&bytes, &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("expected AdmittedV2, got {other:?}"),
    }
}

/// The node id of `declaration`'s model node: the SHA-256 of its structural
/// node preimage, whose `owner` is exactly `{identity, kind, node}`.
fn model_node_id(declaration: &str) -> String {
    let preimage = format!(
        r#"{{"body":{{"members":[],"term":"aggregate"}},"declaration":null,"node_tag":"model","owner":{{"identity":"{IDENTITY}","kind":"model","node":"ix://{IDENTITY}/{declaration}"}},"recursion":null,"semantic_form":"object_type","semantic_type":null,"version":"quire.structural-node/v1"}}"#
    );
    format!("{:x}", Sha256::digest(preimage.as_bytes()))
}

/// The ids of the model nodes IR admitted.
fn model_node_ids(package: &CheckedPackageV2) -> BTreeSet<String> {
    package
        .graph()
        .nodes
        .iter()
        .filter(|node| &*node.node_tag == "model")
        .map(|node| node.node_id.digest.to_string())
        .collect()
}

/// IT-012-SC-01: IR's v2 reader admits the package QSL emitted for a unit
/// that selects a domain-package model, and the lock names that selection.
#[test]
#[trace("IT-012-SC-01")]
fn a_model_bearing_package_emitted_by_qsl_is_admitted_by_irs_reader() {
    let package = admitted(AUTHORED_VERSION);
    let [selection] = &package.lock().model_selections[..] else {
        panic!("one model selection: {:?}", package.lock().model_selections);
    };
    assert_eq!(&*selection.identity, IDENTITY);
    assert_eq!(&*selection.digest_domain, "sha256-jcs");
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

/// IT-012-SC-01: a domain package document supplied under another document's
/// digest is refused as a missing import at the selection's digest.
#[test]
#[trace("IT-012-SC-01")]
fn a_domain_package_supplied_under_another_digest_is_refused() {
    let document = domain_package(AUTHORED_VERSION);
    let bytes = emit(&unit_selecting(AUTHORED_VERSION, &document), &document);
    let evidence = evidence(&bytes, &"0".repeat(64), &document);
    let CheckedPackageDispatchResult::Refused(refusal) = read(&bytes, &evidence) else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        refusal.path.map(|path| path.to_string()).as_deref(),
        Some("/lock/model_selections/0/digest")
    );
}

/// IT-012-SC-02: every model node IR admitted is keyed by the content-only
/// `ModelOwner`, `{kind, identity, node}`. A `version` member in the owner
/// would change each node's id away from these.
#[test]
#[trace("IT-012-SC-02")]
fn model_nodes_are_owned_by_the_content_only_model_owner() {
    let package = admitted(AUTHORED_VERSION);
    let expected: BTreeSet<String> = DECLARATIONS.iter().map(|d| model_node_id(d)).collect();
    assert_eq!(model_node_ids(&package), expected);
}

/// IT-012-SC-03: a version-only change of the domain package leaves the
/// selection's version changed and every model node's id the same.
#[test]
#[trace("IT-012-SC-03")]
fn a_version_only_change_of_the_domain_package_keys_the_same_model_nodes() {
    let (first, second) = (admitted(AUTHORED_VERSION), admitted("2.0.0"));
    assert_ne!(
        second.lock().model_selections,
        first.lock().model_selections
    );
    assert_eq!(model_node_ids(&second), model_node_ids(&first));
    assert_eq!(model_node_ids(&second).len(), DECLARATIONS.len());
}
