// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U11r admission through QSL's selected model package.

mod support;

use std::fs;

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{
    check::{wire_id::structural_id, Owner, PackageDeclarations},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
    value::nominal_payload::NominalPayload,
};
use quire_exact::{Integer, Rational};
use quire_semantic_value::checking::CheckingLimits;
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const MODEL_IDENTITY: &str = "acme/orders";
const MODEL_PATH: &str = "tests/fixtures/spine-model.semantic-ir.json";
const DIMENSION_ID: &str = "ix://acme/orders/dimension/L";
const ROOT_ID: &str = "ix://acme/orders/unit/m";

fn selected_model_document() -> Vec<u8> {
    let path = support::qsl_root().join(MODEL_PATH);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let mut document: Value = serde_json::from_slice(&bytes).expect("the QSL model is JSON");
    assert_eq!(document["package"]["identity"], MODEL_IDENTITY);
    let origin = document["types"][0]["origin"].clone();
    document["dimensions"] = json!([{
        "identity": DIMENSION_ID,
        "name": "L",
        "base": true,
        "terms": [],
        "origin": origin,
    }]);
    document["units"] = json!([{
        "identity": ROOT_ID,
        "name": "m",
        "dimension": DIMENSION_ID,
        "scale": "1",
        "offset": "0",
        "origin": origin,
    }]);
    serde_json::to_vec(&document).expect("the selected model serializes")
}

fn public_u11r() {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == "U11r")
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public U11r vector: {matches:?}");
    };
    assert_eq!(
        row.inputs,
        "`dimension L;` imported from package P (with `unit m : L = 1;` there) and local `unit cm : L = 1/100 * m;`"
    );
    assert_eq!(
        row.expected,
        "admitted: the imported `m` is a terminal root in the same node; the imported dimension `L` takes its turn before `cm` (rule of the turn list), where `cm` is the only local unit and passes rows 1 to 6 (FR-142-AC-18)"
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-18. Imported root and local unit agree.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-18")]
#[test]
fn public_u11r_imported_root_checks_and_emits_with_exact_scale() {
    public_u11r();
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model P = \"{MODEL_IDENTITY}\";\n\
         unit cm : P::L = rational(1, 100) * P::m;\n\
         function use_cm using v(d: Quantity<cm>): Boolean pure {{ true }}\n"
    );
    assert!(source.contains("unit cm : P::L = rational(1, 100) * P::m;"));
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc187-u11-imported-positive"),
        "tc187-u11-imported-positive.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11r source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let raw = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("U11r source forms");
    let document = selected_model_document();
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document.as_slice()]).expect("selected model package identity"),
        ModelNormalizationLimits::default(),
    )
    .expect("the model dimension and unit admit");
    let assembled = PackageDeclarations::assemble(
        raw,
        "agent-ix/tc187-u11-imported-positive".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("the imported root and local cm assemble");
    let cm_keys = assembled
        .nominal_spans
        .iter()
        .filter(|(_, span)| &source[span.start..span.end] == "cm")
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let [cm_key] = cm_keys.as_slice() else {
        panic!("one local cm nominal key: {cm_keys:?}");
    };
    let cm = assembled
        .units
        .unit(cm_key.clone())
        .expect("the admitted local cm unit");
    let one_hundredth = Rational::new(Integer::from(1_i64), Integer::from(100_i64))
        .expect("a positive denominator");
    assert_eq!(cm.canonical().scale(), &one_hundredth);
    assert_eq!(
        cm.canonical().offset(),
        &Rational::from_integer(Integer::zero())
    );
    assert_eq!(cm.path().len(), 1, "one local edge to the imported root");
    let root_key = cm.root();
    let dimension_key = cm.dimension_node();

    let checked = assembled
        .check(CheckingLimits::default())
        .expect("the imported unit and its using function check");
    let graph_nodes = checked.semantic_graph().nodes().collect::<Vec<_>>();
    let model_node = |identity: &str| {
        graph_nodes
            .iter()
            .find(|node| {
                matches!(node.owner(), Some(Owner::Model(owner)) if owner.identity() == MODEL_IDENTITY && owner.node() == identity)
            })
            .unwrap_or_else(|| panic!("one checked model-owned node {identity}"))
    };
    let model_dimension = model_node(DIMENSION_ID);
    let model_root = model_node(ROOT_ID);
    assert_eq!(model_dimension.key(), dimension_key);
    assert_eq!(model_root.key(), root_key);
    assert_eq!(model_dimension.semantic_form(), "dimension");
    assert_eq!(model_root.semantic_form(), "unit");
    let Some(NominalPayload::Unit(root_payload)) = model_root.nominal_payload() else {
        panic!("checked imported m has a unit payload: {model_root:?}");
    };
    assert_eq!(root_payload.dimension(), &dimension_key);
    assert_eq!(root_payload.target(), None, "imported m stays terminal");
    let cm_node = graph_nodes
        .iter()
        .find(|node| node.key() == *cm_key)
        .expect("local cm remains in the checked graph");
    let Some(NominalPayload::Unit(cm_payload)) = cm_node.nominal_payload() else {
        panic!("checked cm has a unit payload: {cm_node:?}");
    };
    assert_eq!(cm_payload.dimension(), &dimension_key);
    assert_eq!(cm_payload.target(), Some(&root_key));
    assert_eq!(cm_payload.factor(), &one_hundredth);
    assert_eq!(
        cm_payload.offset(),
        &Rational::from_integer(Integer::zero())
    );

    let emission = emit_checked(
        &CheckedPackage::link(checked),
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("the checked imported unit emits");
    assert!(emission.omitted().is_empty());
    let wire: Value =
        serde_json::from_slice(emission.package().bytes()).expect("emitted checked package JSON");
    let wire_nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("emitted semantic graph nodes");
    for key in [cm_key.clone(), root_key, dimension_key] {
        let id = serde_json::to_value(structural_id(&key)).expect("structural id serializes");
        assert!(
            wire_nodes.iter().any(|node| node["node_id"] == id),
            "emission retains selected local or model-owned node {id}"
        );
    }
}
