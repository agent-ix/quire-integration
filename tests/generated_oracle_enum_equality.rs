// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original model-owned enum equality: admitted by QSL and IR, refused by CG derivation.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;

use ix_trace_rs::trace;
use qsl_semantics::{
    check::PackageDeclarations,
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_codegen::{derive_exact_scalar_items, ExactScalarRefusal};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_specification::{ACME_COLORS_COMPARISON_SOURCE, ACME_COLORS_DOMAIN_PACKAGE};

#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn model_owned_enum_equality_is_admitted_and_refused_by_cg_derivation() {
    let document = ACME_COLORS_DOMAIN_PACKAGE.as_bytes();
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "enum-comparison"),
        "enum-comparison.native",
        ACME_COLORS_COMPARISON_SOURCE.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the QSpec native source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let raw = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("the native source forms");
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document]).expect("the colors model has its declared identity"),
        ModelNormalizationLimits::default(),
    )
    .expect("the QSpec colors model admits");
    let graph = PackageDeclarations::assemble(
        raw,
        "agent-ix/enum-comparison".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("the unit assembles")
    .check(CheckingLimits::default())
    .expect("the unit checks");
    let package = qsl_package::CheckedPackage::link(graph);
    let emission = qsl_package::emit_checked(
        &package,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("the package emits");
    assert!(emission.omitted().is_empty());
    let wire: serde_json::Value =
        serde_json::from_slice(emission.package().bytes()).expect("emitted JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    evidence
        .insert_domain_package_document("acme/colors", document)
        .expect("the colors model is valid evidence");
    let CheckedPackageV2ReadResult::Admitted(admitted) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("the emitted enum package admits");
    };
    // QSL and IR admit the standalone QSpec colors model and all four equality and inequality
    // truth arms; CG derivation refuses each model-owned variant comparison, so none reaches a
    // generated native oracle and none is credited as agreement.
    for name in [
        "redEqualsRed",
        "redEqualsGreen",
        "redDiffersFromRed",
        "redDiffersFromGreen",
    ] {
        let body = family_function_selection::body(&admitted, name);
        let derived = derive_exact_scalar_items(&admitted, std::slice::from_ref(&body));
        let [Err(ExactScalarRefusal::BlockedOnUpstream {
            unsupported_node_id,
            node_tag,
            ..
        })] = derived.as_slice()
        else {
            panic!("{name}: CG derivation of enum comparison: {derived:?}");
        };
        assert_eq!(&**node_tag, "model", "{name}");
        let refused = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| &node.node_id == unsupported_node_id)
            .expect("the refused node is in the admitted graph");
        assert_eq!(&*refused.semantic_form, "variant_type", "{name}");
    }
}
