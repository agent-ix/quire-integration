// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original source lifecycle helpers for the namespace and emission tests.

use qsl_foundation::source::provenance::RawSourceRef;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::check::CheckedGraph;
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use quire_specification::{vectors_of, TestCase};
use serde_json::Value;

/// Complete source language and value profile selections.
pub const HEADER: &str =
    "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n";
/// Original time dimension and millisecond unit declarations.
pub const UNITS: &str =
    "dimension T;\nunit s : T = rational(1, 1);\nunit ms : T = rational(1, 1000) * s;\n";

/// Bind one complete public vector cell without importing a fixture corpus.
pub fn vector(id: &str, input: &str, expected: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("one public TC-187 {id} row: {rows:?}");
    };
    assert_eq!(row.inputs, input, "{id} complete input");
    assert_eq!(row.expected, expected, "{id} complete expectation");
}

/// Parse and form an original source document, retaining its source identity.
pub fn parsed(name: &str, text: &str) -> (RawSourceRef, qsl_forms::ParsedUnit) {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", name),
        "tc187-p8.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("source intake succeeds");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    (
        parsed.source().reference().clone(),
        qsl_forms::build_unit(&parsed).expect("source forms"),
    )
}

/// Link and emit the original checked graph, then strictly admit its bytes.
pub fn emitted(graph: CheckedGraph, model: Option<(&str, &[u8])>) -> CheckedPackageV2 {
    let emission = emit_checked(
        &CheckedPackage::link(graph),
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("checked source emits");
    assert!(emission.omitted().is_empty(), "{:?}", emission.omitted());
    let bytes = emission.package().bytes();
    let wire: Value = serde_json::from_slice(bytes).expect("original emitted JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("emitted feature inventory")
    {
        evidence.support_feature(feature.as_str().expect("feature string"));
    }
    if let Some((identity, document)) = model {
        evidence
            .insert_domain_package_document(identity, document)
            .expect("original model evidence");
    }
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("strict IR admission of original emission: {other:?}"),
    }
}
