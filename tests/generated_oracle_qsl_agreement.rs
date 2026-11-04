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

fn boolean_outcome(
    outcome: quire_exact::Outcome<quire_exact::Value>,
) -> quire_exact::Outcome<bool> {
    use quire_exact::{Outcome, Value};
    match outcome {
        Outcome::Completed(Value::Boolean(value)) => Outcome::Completed(value),
        Outcome::Completed(other) => panic!("the authority must return Boolean: {other:?}"),
        Outcome::Undefined(cause) => Outcome::Undefined(cause),
        Outcome::Refused(cause) => Outcome::Refused(cause),
        Outcome::Incomplete(cause) => Outcome::Incomplete(cause),
    }
}

/// An initial family contribution; whole SC-01..06 remain Planned until the
/// complete scalar/composite/function coordinator executes their controls.
#[trace("TC-099")]
#[test]
fn generated_boolean_negation_matches_the_actual_qsl_body() {
    let admitted = admit("function negation using v(): Boolean pure { not false }");
    let nodes = operation_nodes(&admitted.ir, "quire.op.boolean.not");
    let items = derive_exact_scalar_items(&admitted.ir, &nodes)
        .into_iter()
        .map(|item| item.expect("admitted scalar derives"))
        .collect::<Vec<_>>();
    let output = generate_exact_scalar_oracles(&admitted.ir, &items)
        .expect("actual selected source generates");
    let [claim] = output.claim_map.items.as_slice() else {
        panic!("one negation claim")
    };
    let quire_contract_codegen::ClaimDisposition::Generated(generated) = &claim.result else {
        panic!("real negation must generate: {:?}", claim.result);
    };
    let identity = admitted
        .qsl
        .graph()
        .function_identity("negation")
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
    let expected = oracle_observation::snapshot(&boolean_outcome(qsl_outcome), &losses, &qsl_meter);
    let driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Meter, ScalarLimits}};
#[test]
fn observe() {{
    let mut meter = Meter::new({:?});
    let actual = quire_exact_scalar_oracles::{}(false, &mut meter)
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

/// A missing scalar bound remains CG's typed generation refusal, even when
/// QSL itself can evaluate the unbounded source. It is not agreement credit.
#[trace("TC-099")]
#[test]
fn unbounded_qsl_integer_source_retains_the_codegen_bound_refusal() {
    let admitted = admit("function unbounded using v(): Integer pure { 4 + 1 }");
    let nodes = operation_nodes(&admitted.ir, "quire.op.integer.add");
    let results = derive_exact_scalar_items(&admitted.ir, &nodes);
    let [Err(quire_contract_codegen::ExactScalarRefusal::RequiresBound { unbounded_type })] =
        results.as_slice()
    else {
        panic!("the unbounded source retains RequiresBound: {results:?}")
    };
    let declared = admitted
        .ir
        .graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == unbounded_type)
        .expect("the refused type is admitted");
    assert_eq!(declared.node_tag.as_ref(), "scalar_type");
    assert_eq!(declared.semantic_form.as_ref(), "integer");
}

/// The selected operation is observed from the real bounded-parameter call.
/// Full caller and generated-body ledgers remain distinct observations here;
/// this value/occurrence contribution does not claim SC-04 accounting parity.
#[trace("TC-099")]
#[test]
fn bounded_integer_source_executes_its_generated_oracle_and_observed_qsl_operation() {
    use qsl_eval::value::{ApplicationObservation, CheckedPackageEvaluation, QualifiedName};
    use qsl_semantics::model::object_environment::ObjectEnvironment;
    use quire_exact::{Integer, Meter, Outcome, Value};
    let admitted = admit("function step using v(x: Int[0, 9]): Integer pure { x + 1 }");
    let nodes = operation_nodes(&admitted.ir, "quire.op.integer.add");
    assert_eq!(
        nodes.len(),
        1,
        "one selected arithmetic operation, not a first-match selection"
    );
    let items = derive_exact_scalar_items(&admitted.ir, &nodes)
        .into_iter()
        .map(|item| item.expect("a genuine bounded-operand source must derive"))
        .collect::<Vec<_>>();
    let generated = generate_exact_scalar_oracles(&admitted.ir, &items)
        .expect("the genuine bounded source generates");
    let [claim] = generated.claim_map.items.as_slice() else {
        panic!("one selected claim")
    };
    assert_eq!(claim.node_id, nodes[0]);
    assert_eq!(
        claim.operation.provenance,
        quire_contract_codegen::OperationProvenance::IrConfirmed
    );
    let quire_contract_codegen::ClaimDisposition::Generated(oracle) = &claim.result else {
        panic!("the source claim must really generate: {:?}", claim.result);
    };
    assert!(
        !oracle.checked_bounds.is_empty(),
        "the source supplies actual checked bounds"
    );
    let key = qsl_foundation::node_wire::node_key_from_json(
        &serde_json::to_value(
            admitted
                .ir
                .structural_key(&nodes[0])
                .expect("the selected compact ID has its reader-verified structural key"),
        )
        .expect("serialize the admitted structural key"),
    )
    .expect("the owner decoder reads the admitted key, without synthesizing an identity");
    let locations = admitted
        .qsl
        .graph()
        .occurrences()
        .filter(|(node, _, _)| node == &key)
        .map(|(_, _, location)| location.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        locations.len(),
        1,
        "the selected emitted node has one actual source occurrence"
    );
    let mut observed = Vec::new();
    let mut unavailable_imported_call = false;
    let mut function_evaluations = 0;
    let mut function_case_limit_reached = false;
    let mut qsl_meter = Meter::new(limits());
    let evaluation = admitted
        .qsl
        .call_observing_applications(
            &QualifiedName::unqualified("step").expect("declared source name"),
            vec![Value::Integer(Integer::from(4_i64))],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
            ApplicationObservation {
                observed: &mut observed,
                unavailable_imported_call: &mut unavailable_imported_call,
                function_case_limit: 1,
                function_evaluations: &mut function_evaluations,
                function_case_limit_reached: &mut function_case_limit_reached,
            },
        )
        .expect("the real bounded QSL call executes");
    assert!(!unavailable_imported_call);
    assert!(!function_case_limit_reached);
    assert_eq!(function_evaluations, 1);
    let selected = observed
        .iter()
        .filter(|(location, _)| locations.contains(location))
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    let [Value::Integer(selected_value)] = selected.as_slice() else {
        panic!("the exact selected application completed an Integer: {selected:?}");
    };
    assert_eq!(selected_value, &Integer::from(5_i64));
    let quire_eval::FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(returned))) =
        &evaluation.outcome
    else {
        panic!("the typed call must complete: {:?}", evaluation.outcome)
    };
    assert_eq!(returned, selected_value);
    assert!(evaluation.location.is_none());
    assert!(evaluation.losses.is_empty());
    let driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Integer, Meter, ScalarLimits}};
#[test]
fn observe() {{
    let mut meter = Meter::new({:?});
    let actual = quire_exact_scalar_oracles::{}(&Integer::from(4_i64), &Integer::from(1_i64), &mut meter)
        .expect("generated operand admission");
    println!("IT019_OBSERVATION {{:?}}", actual);
    let losses = Vec::<quire_semantic_value::loss::ValueLoss>::new();
    println!("IT019_OBSERVATION {{}}", oracle_observation::snapshot(&actual, &losses, &meter));
}}
"#,
        limits(),
        oracle.symbol
    );
    let native = generated_oracle::execute(&generated.artifacts, &driver);
    assert_eq!(
        native.len(),
        2,
        "actual typed outcome and complete body observation"
    );
    assert_eq!(
        native[0],
        format!("{:?}", Outcome::Completed(selected_value.clone()))
    );
    eprintln!(
        "QSL whole call: {:?}; raw meter {:?}; losses {:?}",
        evaluation.outcome, qsl_meter, evaluation.losses
    );
    eprintln!("Generated body whole observation: {}", native[1]);
}

/// This bounded-profile entry has no public per-call profile override. These
/// are real admitted source quantities, not manually assigned node labels.
/// A typed bound refusal is not quantity-oracle agreement credit.
#[trace("TC-099")]
#[test]
fn bounded_scalar_entry_retains_the_real_quantity_value_position_refusals() {
    let admitted = admit(
        "\
        dimension Length;\n\
        unit m : Length = rational(1, 1);\n\
        unit cm : Length = rational(1, 100) * m;\n\
        function qadd using v(a: Quantity<m>, b: Quantity<m>): Quantity<m> pure { a + b }\n\
        function qconvert using v(a: Quantity<cm>): Quantity<m> pure { convert<Quantity<m>>(a) }\n\
        function qless using v(a: Quantity<m>, b: Quantity<m>): Boolean pure { a < b }\n",
    );
    for identity in [
        "quire.op.quantity.add",
        "quire.op.quantity.convert",
        "quire.op.quantity.lt",
    ] {
        let nodes = operation_nodes(&admitted.ir, identity);
        assert_eq!(nodes.len(), 1, "one real {identity} source application");
        let results = derive_exact_scalar_items(&admitted.ir, &nodes);
        let [Err(quire_contract_codegen::ExactScalarRefusal::RequiresBound { unbounded_type })] =
            results.as_slice()
        else {
            panic!("current bounded-profile {identity}: {results:?}")
        };
        let refused_type = admitted
            .ir
            .graph()
            .nodes
            .iter()
            .find(|node| &node.node_id == unbounded_type)
            .expect("the refused type is in the source graph");
        assert_eq!(refused_type.node_tag.as_ref(), "scalar_type");
        assert!(
            matches!(
                refused_type.semantic_form.as_ref(),
                "unit" | "compound_unit" | "carried_quantity"
            ),
            "the actual quantity endpoint refuses, not a guessed scalar label: {refused_type:?}"
        );
    }
}

#[path = "support/function_agreement.rs"]
mod function_agreement;

/// Complete truth-table and short-circuit native scalar contribution. Source
/// locations remain a separate source-map axis; this does not qualify SC-04.
#[trace("TC-099")]
#[test]
fn generated_boolean_truth_tables_preserve_typed_values_and_ordered_accounting() {
    for (syntax, identity) in [
        ("and", "quire.op.boolean.and"),
        ("or", "quire.op.boolean.or"),
        ("implies", "quire.op.boolean.implies"),
        ("=", "quire.op.boolean.eq"),
        ("!=", "quire.op.boolean.ne"),
    ] {
        for left in [false, true] {
            for right in [false, true] {
                let admitted = admit(&format!(
                    "function truth using v(): Boolean pure {{ {left} {syntax} {right} }}"
                ));
                let nodes = operation_nodes(&admitted.ir, identity);
                assert_eq!(nodes.len(), 1, "one genuine selected operation");
                let items = derive_exact_scalar_items(&admitted.ir, &nodes)
                    .into_iter()
                    .map(|item| item.expect("Boolean derives from original source"))
                    .collect::<Vec<_>>();
                let output = generate_exact_scalar_oracles(&admitted.ir, &items).unwrap();
                let [claim] = output.claim_map.items.as_slice() else {
                    panic!("one truth claim");
                };
                let quire_contract_codegen::ClaimDisposition::Generated(generated) = &claim.result
                else {
                    panic!("actual Boolean family must generate: {:?}", claim.result);
                };
                let function = admitted
                    .qsl
                    .graph()
                    .function_by_identity(admitted.qsl.graph().function_identity("truth").unwrap())
                    .unwrap();
                let mut expected = Vec::new();
                for work in [1024, 0, 1] {
                    let mut allowance = limits();
                    allowance.work_units = work;
                    let mut meter = quire_exact::Meter::new(allowance);
                    let result = qsl_eval::value::evaluate_closed(
                        &admitted.qsl,
                        function.body,
                        function.slots,
                        &mut meter,
                    )
                    .expect("genuine closed Boolean body");
                    let quire_eval::FamilyOutcome::Evaluated(outcome) = result.outcome else {
                        panic!("actual Boolean value family");
                    };
                    let losses = result
                        .losses
                        .into_iter()
                        .map(|loss| loss.loss)
                        .collect::<Vec<_>>();
                    expected.push(oracle_observation::snapshot(
                        &boolean_outcome(outcome),
                        &losses,
                        &meter,
                    ));
                }
                let lazy = matches!(
                    items[0].operation,
                    quire_contract_codegen::ExactScalarOperation::Boolean {
                        right_may_stop: true,
                        ..
                    }
                );
                let arguments = if lazy {
                    format!("{left}, || quire_exact::Outcome::Completed({right})")
                } else {
                    format!("{left}, {right}")
                };
                let driver = format!(
                    r#"
mod oracle_observation;
use quire_exact::{{Meter,ScalarLimits}};
#[test]
fn observe() {{
    for work in [1024,0,1] {{
        let mut allowance = {:?}; allowance.work_units = work;
        let mut meter = Meter::new(allowance);
        let actual = quire_exact_scalar_oracles::{}({},&mut meter).expect("actual admitted Boolean arguments");
        let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
        println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter));
    }}
}}
"#,
                    limits(),
                    generated.symbol,
                    arguments
                );
                assert_eq!(
                    generated_oracle::execute(&output.artifacts, &driver),
                    expected,
                    "{left} {syntax} {right}"
                );
            }
        }
    }
}
