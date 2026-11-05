// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-110: public TC-187 U34/U35 model and source namespace admission.

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{
    check::{AssemblyCause, AssemblyRefusal, PackageDeclarations},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const LOCAL: &str = "dimension Time; unit s : Time = rational(1, 1);";
const M: &str = "model M = \"acme/rpc\";";
const RF: &str = "model R = \"acme/rpc\"; model F = \"acme/flow\";";

fn public_row(id: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("one public {id}: {rows:?}")
    };
    assert!(!row.inputs.is_empty() && !row.expected.is_empty());
    match id {
        "U34" => {
            assert!(
                row.inputs.contains("unit tick : M::Rate") && row.inputs.contains("Length"),
                "{}",
                row.inputs
            );
            assert!(
                row.expected.contains("/semantic_graph/nodes/0/owner")
                    && row.expected.contains("scalar_type"),
                "{}",
                row.expected
            );
        }
        "U35" => {
            assert!(
                row.inputs.contains("dimension Speed = R::Time")
                    && row.inputs.contains("two dimensions of identity"),
                "{}",
                row.inputs
            );
            assert!(
                row.expected.contains("later `Dimension.identity`")
                    && row.expected.contains("three distinct dimensions"),
                "{}",
                row.expected
            );
        }
        other => panic!("unassigned public row {other}"),
    }
}

// Original native domain input, not an emitted package or owner receipt.
fn document(identity: &str, dimension: &str, root: &str, ms: bool, length: bool) -> Vec<u8> {
    let origin = json!({"generated": {"generatorIdentity": format!("ix://{identity}/namespace-input"), "generatorVersion": "1.0.0", "inputIdentities": [format!("ix://{identity}/namespace-input")]}});
    let dimension_id = format!("ix://{identity}/dimension/{dimension}");
    let root_id = format!("ix://{identity}/unit/{root}");
    let mut dimensions = vec![
        json!({"identity": dimension_id, "name": dimension, "base": true, "terms": [], "origin": origin}),
    ];
    if length {
        dimensions.push(json!({"identity": format!("ix://{identity}/dimension/Length"), "name": "Length", "base": true, "terms": [], "origin": origin}));
    }
    let mut units = vec![
        json!({"identity": root_id, "name": root, "dimension": dimension_id, "scale": "1", "offset": "0", "origin": origin}),
    ];
    if ms {
        units.push(json!({"identity": format!("ix://{identity}/unit/ms"), "name": "ms", "dimension": dimension_id, "target": root_id, "scale": "1000", "offset": "0", "origin": origin}));
    }
    serde_json::to_vec(&json!({
        "contractVersion": "2.0.0", "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": format!("ix://{identity}/namespace-input"), "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": identity, "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
        "occurrences": [], "extensions": [], "constructs": [], "types": [],
        "dimensions": dimensions, "units": units,
    })).expect("native domain input serializes")
}

fn source(body: &str) -> String {
    format!("language \"ix:native\" edition \"1-draft\"; profile v = \"quire.value.complete/v1\"; {body}")
}

fn assemble(
    body: &str,
    documents: &[Vec<u8>],
) -> (String, Result<PackageDeclarations, AssemblyRefusal>) {
    let source = source(body);
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc110-model-names"),
        "model-names.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("source forms");
    let input =
        package_input(documents.iter().map(Vec::as_slice)).expect("domain document identities");
    let models = admit_unit(
        &unit.selections().models,
        &input,
        ModelNormalizationLimits::default(),
    )
    .expect("genuine native model intake");
    let result = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc110-model-names".to_owned(),
        unit,
        models,
        Vec::new(),
    );
    (source, result)
}

fn emitted(body: &str, documents: &[Vec<u8>]) -> Value {
    let (_, assembled) = assemble(body, documents);
    let graph = assembled
        .expect("namespace assembly")
        .check(CheckingLimits::default())
        .expect("namespace checking");
    let emission = emit_checked(&CheckedPackage::link(graph)).expect("native checked emission");
    assert!(emission.omitted().is_empty());
    let bytes = emission.package().bytes();
    let wire: Value = serde_json::from_slice(bytes).expect("emitted JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature string"));
    }
    for document in documents {
        let value: Value = serde_json::from_slice(document).expect("domain JSON");
        evidence
            .insert_domain_package_document(
                value["package"]["identity"].as_str().expect("identity"),
                document,
            )
            .expect("original document evidence");
    }
    assert!(
        matches!(
            read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence),
            CheckedPackageDispatchResult::AdmittedV2(_)
        ),
        "strict IR admits original emission"
    );
    wire
}

fn nodes(wire: &Value) -> &[Value] {
    wire["semantic_graph"]["nodes"].as_array().expect("nodes")
}
fn model_node<'a>(wire: &'a Value, identity: &str) -> &'a Value {
    let found: Vec<_> = nodes(wire)
        .iter()
        .filter(|node| {
            node["node_id"]["path"] == json!(["scalar_type", "unit", identity])
                || node["node_id"]["path"] == json!(["scalar_type", "dimension", identity])
        })
        .collect();
    let [node] = found.as_slice() else {
        panic!("one native model node {identity}: {wire}")
    };
    node
}
fn refusal(body: &str, documents: &[Vec<u8>], name: &str) {
    let (source, result) = assemble(body, documents);
    let refused = result.expect_err("missing dimension refuses");
    let [error] = refused.errors.as_slice() else {
        panic!("one first refusal: {refused:?}")
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnresolvedQuantityName {
            name: name.to_owned()
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "missing_declaration/missing-name"
    );
    assert_eq!(&source[error.span.start..error.span.end], name);
}

#[trace("TC-110", "QSpec-TC-187", "QSpec-FR-142-AC-19")]
#[test]
fn source_and_model_ms_are_independent_and_tick_uses_model_rate() {
    public_row("U34");
    let docs = [document("acme/rpc", "Rate", "us", true, false)];
    let wire = emitted(&format!("{M} {LOCAL} unit ms : Time = rational(1, 1000) * s; unit tick : M::Rate = rational(1, 2) * M::us; function f using v(d: Quantity<ms>): Boolean pure {{ true }} function g using v(d: Quantity<M::ms>): Boolean pure {{ true }}"), &docs);
    let ms = model_node(&wire, "ix://acme/rpc/unit/ms");
    assert!(
        ms.get("declaration").is_none(),
        "model unit carries no source declaration: {ms}"
    );
    println!("U34 wire {wire}");
    refusal(
        &format!("{M} {LOCAL} unit tick : Rate = rational(1, 2) * M::us;"),
        &docs,
        "Rate",
    );
}

#[trace("TC-110", "QSpec-TC-187", "QSpec-FR-142-AC-19")]
#[test]
fn unitless_imported_length_retains_original_first_refusal() {
    public_row("U34");
    let docs = [document("acme/rpc", "Rate", "us", true, true)];
    let (source, result) = assemble(
        &format!("{M} {LOCAL} dimension Area = M::Length ^ 2; unit m2 : Area = rational(1, 1);"),
        &docs,
    );
    let refused = result.expect_err("unitless model Length refuses");
    let [error] = refused.errors.as_slice() else {
        panic!("one first refusal: {refused:?}")
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnanchoredBaseDimension {
            dimension: "ix://acme/rpc/dimension/Length".to_owned(),
            unit: "m2".to_owned()
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "missing_declaration/missing-name"
    );
    assert_eq!(&source[error.span.start..error.span.end], M);
}

#[trace("TC-110", "QSpec-TC-187", "QSpec-FR-142-AC-20")]
#[test]
fn three_time_owners_compose_and_missing_names_remain_missing() {
    public_row("U35");
    let docs = [
        document("acme/rpc", "Time", "s", false, false),
        document("acme/flow", "Time", "s", false, false),
    ];
    let wire = emitted(&format!("{RF} {LOCAL} unit tenth : R::Time = rational(1, 10) * R::s; dimension Speed = R::Time ^ -1 * F::Time; function local using v(d: Quantity<s>): Boolean pure {{ true }} function rpc using v(d: Quantity<R::s>): Boolean pure {{ true }} function flow using v(d: Quantity<F::s>): Boolean pure {{ true }}"), &docs);
    println!("U35 wire {wire}");
    assert_ne!(
        model_node(&wire, "ix://acme/rpc/dimension/Time")["node_id"],
        model_node(&wire, "ix://acme/flow/dimension/Time")["node_id"]
    );
    for (body, name) in [
        (format!("{RF} unit x : Time = rational(1, 1);"), "Time"),
        (
            format!("{RF} {LOCAL} unit x : Z::Time = rational(1, 1);"),
            "Z::Time",
        ),
        (
            format!("{RF} {LOCAL} unit x : R::Speed = rational(1, 1);"),
            "R::Speed",
        ),
    ] {
        refusal(&body, &docs, name);
    }
    let (source, result) = assemble(&format!("{RF} {LOCAL} dimension Time;"), &docs);
    let refused = result.expect_err("duplicate source Time refuses");
    let [error] = refused.errors.as_slice() else {
        panic!("one ambiguity: {refused:?}")
    };
    let AssemblyCause::DuplicateQuantityName { name, spans } = &error.cause else {
        panic!("native ambiguity: {error:?}")
    };
    assert_eq!(name, "Time");
    assert_eq!(spans.len(), 2);
    for span in spans {
        assert_eq!(&source[span.start..span.end], "Time");
    }
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "ambiguous_declaration/ambiguous-name"
    );
}

#[trace("TC-110", "QSpec-TC-187", "QSpec-FR-142-AC-20")]
#[test]
fn duplicate_model_dimension_identity_refuses_at_intake() {
    public_row("U35");
    let mut native: Value =
        serde_json::from_slice(&document("acme/rpc", "Time", "s", false, false))
            .expect("native JSON");
    let duplicate = native["dimensions"][0].clone();
    native["dimensions"]
        .as_array_mut()
        .expect("dimensions")
        .push(duplicate);
    let bytes = serde_json::to_vec(&native).expect("duplicate input JSON");
    let text = source(M);
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc110-model-names"),
        "model-names.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("source parse");
    let unit = qsl_forms::build_unit(&parsed).expect("source forms");
    let input = package_input([bytes.as_slice()]).expect("package input");
    let refused = admit_unit(
        &unit.selections().models,
        &input,
        ModelNormalizationLimits::default(),
    )
    .expect_err("duplicate model identity refuses before assembly");
    println!("U35 duplicate {refused:?}");
    assert_eq!(
        refused
            .cause
            .catalog_code()
            .expect("catalog cause")
            .to_string(),
        "invalid_model_binding/conflicting-binding"
    );
}
