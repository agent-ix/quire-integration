// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual generated scalar/composite/function execution against checked QSL.

#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;

use ix_trace_rs::trace;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_contract_codegen::{derive_exact_scalar_items, generate_exact_scalar_oracles};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedTerm,
};
use quire_semantic_value::checking::CheckingLimits;

/// The actual producer package and its independently read emitted V2 package.
struct Admitted {
    qsl: CheckedPackage,
    ir: CheckedPackageV2,
}

fn admit(source: &str) -> Admitted {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{source}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "oracle-agreement"),
        "oracle-agreement.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("the source forms");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/oracle-agreement".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("the source assembles")
    .check(CheckingLimits::default())
    .expect("the source checks");
    let qsl = CheckedPackage::link(graph);
    let emission = emit_checked(&qsl).expect("the real checked source emits");
    assert!(
        emission.omitted().is_empty(),
        "no selected claim is omitted"
    );
    let bytes = emission.package().bytes();
    let wire: serde_json::Value = serde_json::from_slice(bytes).expect("emission JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("feature array")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    let ir = match read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("the emitted bytes must admit: {other:?}"),
    };
    Admitted { qsl, ir }
}

fn operation_nodes(package: &CheckedPackageV2, identity: &str) -> Vec<CheckedNodeId> {
    let nodes = package
        .graph()
        .nodes
        .iter()
        .filter_map(|node| match package.term(&node.node_id) {
            Some(CheckedTerm::Application { operation, .. })
                if operation.identity() == Some(identity) =>
            {
                Some(node.node_id.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!nodes.is_empty(), "the real source emits {identity}");
    nodes
}

fn limits() -> quire_exact::ScalarLimits {
    quire_exact::ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: 1024,
        result_units: 1024,
    }
}

fn integer_outcome(
    outcome: quire_exact::Outcome<quire_exact::Value>,
) -> quire_exact::Outcome<quire_exact::Integer> {
    use quire_exact::{Outcome, Value};
    match outcome {
        Outcome::Completed(Value::Integer(value)) => Outcome::Completed(value),
        Outcome::Completed(other) => panic!("the authority must return an Integer: {other:?}"),
        Outcome::Undefined(cause) => Outcome::Undefined(cause),
        Outcome::Refused(cause) => Outcome::Refused(cause),
        Outcome::Incomplete(cause) => Outcome::Incomplete(cause),
    }
}

/// An initial family contribution; whole SC-01..06 remain Planned until the
/// complete scalar/composite/function coordinator executes their controls.
#[trace("TC-099")]
#[test]
fn generated_integer_addition_matches_the_actual_qsl_body() {
    let admitted = admit("function addition using v(): Integer pure { 4 + 1 }");
    let nodes = operation_nodes(&admitted.ir, "quire.op.integer.add");
    let items = derive_exact_scalar_items(&admitted.ir, &nodes)
        .into_iter()
        .map(|item| item.expect("admitted scalar derives"))
        .collect::<Vec<_>>();
    let output = generate_exact_scalar_oracles(&admitted.ir, &items)
        .expect("actual selected source generates");
    let [claim] = output.claim_map.items.as_slice() else {
        panic!("one addition claim")
    };
    let quire_contract_codegen::ClaimDisposition::Generated(generated) = &claim.result else {
        panic!("real addition must generate: {:?}", claim.result);
    };
    let identity = admitted
        .qsl
        .graph()
        .function_identity("addition")
        .expect("the admitted function identity");
    let body = admitted
        .qsl
        .graph()
        .function_by_identity(identity)
        .expect("the admitted function body");
    let mut qsl_meter = quire_exact::Meter::new(limits());
    let qsl =
        qsl_eval::value::evaluate_closed(&admitted.qsl, body.body, body.slots, &mut qsl_meter)
            .expect("the actual closed QSL body evaluates");
    let quire_eval::FamilyOutcome::Evaluated(qsl_outcome) = qsl.outcome else {
        panic!("the actual QSL body must reach its value family");
    };
    assert!(
        qsl.location.is_none(),
        "completed value has no failure locus"
    );
    let losses = qsl
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect::<Vec<_>>();
    let expected = oracle_observation::snapshot(&integer_outcome(qsl_outcome), &losses, &qsl_meter);
    let driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Integer, Meter, ScalarLimits}};
#[test]
fn observe() {{
    let mut meter = Meter::new({:?});
    let actual = quire_exact_scalar_oracles::{}(&Integer::from(4_i64), &Integer::from(1_i64), &mut meter)
        .expect("generated operand admission");
    let losses = Vec::<quire_semantic_value::loss::ValueLoss>::new();
    println!("IT019_OBSERVATION {{}}", oracle_observation::snapshot(&actual, &losses, &meter));
}}
"#,
        limits(),
        generated.symbol
    );
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        vec![expected]
    );
}
