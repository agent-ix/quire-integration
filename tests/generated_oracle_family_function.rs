// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected FR-021 operation rows; other SC-05 operations remain unqualified.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;

use family_function_selection::{body, declared};

use ix_trace_rs::trace;
use qsl_replay::{
    acquire_checked_closure, CompileUnit, DependencyInput, ReplayLimits, SourceIdentity,
    StageLimits,
};
use quire_contract_codegen::oracle::operator_catalog::IntegerOperator;
use quire_contract_codegen::{
    generate_exact_function_oracles, routed_contract_lowering_profile, ClaimDisposition,
    ContractDriverResultSelection, ContractRequestPosition, ExactFunctionBody,
    ExactFunctionDeclaration, ExactFunctionItem, FunctionParameter,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, ContractDependencies, ContractPackageReadResult,
    ContractPackageWireV1,
};

const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "function difference using v(left: Int[0,9], right: Int[0,9]): Integer pure { left - right }\n",
    "function product using v(left: Int[0,9], right: Int[0,9]): Integer pure { left * right }\n",
    "function relay using v(left: Int[0,9], right: Int[0,9]): Integer pure { difference(left,right) }\n",
    "function exerciseDifference using v(x: Int[0,9], y: Int[0,9]): Integer pure { difference(x,y) }\n",
    "function exerciseProduct using v(x: Int[0,9], y: Int[0,9]): Integer pure { product(x,y) }\n",
    "function exerciseRelay using v(x: Int[0,9], y: Int[0,9]): Integer pure { relay(x,y) }\n",
);

#[trace("TC-103", "IT-019-SC-05")]
#[test]
fn subtraction_multiplication_and_nested_call_match_fresh_qsl_sessions() {
    let acquisition = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "family-functions"),
            package_name: "family-functions".to_owned(),
            path: "family-functions.native",
            bytes: SOURCE.as_bytes(),
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("the original source is checked and emitted by QSL");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquisition.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("the exact original emission admits");
    };
    let integers = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "integer"
        })
        .collect::<Vec<_>>();
    let [integer] = integers.as_slice() else {
        panic!("one canonical scalar Integer type");
    };
    let integer = integer.node_id.clone();
    let names = ["difference", "product", "relay"];
    let exercise = ["exerciseDifference", "exerciseProduct", "exerciseRelay"];
    let declarations = names
        .iter()
        .map(|name| declared(&package, name))
        .collect::<Vec<_>>();
    let bodies = names
        .iter()
        .map(|name| body(&package, name))
        .collect::<Vec<_>>();
    let calls = exercise
        .iter()
        .map(|name| body(&package, name))
        .collect::<Vec<_>>();
    assert_ne!(
        bodies[2], calls[0],
        "nested and exercise calls are distinct real occurrences"
    );
    let mut requested = bodies.clone();
    requested.extend(calls.iter().cloned());
    let lowered = package.lower(&requested, &routed_contract_lowering_profile());
    let contract = match ContractPackageWireV1::read_with_source(
        lowered
            .package
            .canonical_bytes()
            .expect("original canonical lowered contract"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("the source authenticates its contract")
    {
        ContractPackageReadResult::Admitted(contract) => contract,
        other => panic!("the actual lowered contract admits: {other:?}"),
    };
    let positions = requested
        .into_iter()
        .map(|node_id| ContractRequestPosition::DriverItem {
            node_id,
            result_selection: ContractDriverResultSelection::NotApplicable,
        })
        .collect::<Vec<_>>();
    let functions = names
        .iter()
        .enumerate()
        .map(|(index, name)| ExactFunctionDeclaration {
            node_id: bodies[index].clone(),
            name: (*name).to_owned(),
            parameters: ["left", "right"]
                .into_iter()
                .map(|name| FunctionParameter {
                    name: name.to_owned(),
                    type_node_id: integer.clone(),
                })
                .collect(),
            result_type: integer.clone(),
            body: match index {
                0 => ExactFunctionBody::Scalar {
                    operator: IntegerOperator::Subtract,
                },
                1 => ExactFunctionBody::Scalar {
                    operator: IntegerOperator::Multiply,
                },
                2 => ExactFunctionBody::Call {
                    callee: "difference".to_owned(),
                },
                _ => unreachable!(),
            },
            capability_requirements: Vec::new(),
        })
        .collect::<Vec<_>>();
    let items = calls.iter().enumerate().map(|(index, call)| {
        let Some(CheckedTerm::Application { arguments, .. }) = package.term(call) else { panic!("original exercise call"); };
        assert!(matches!(&arguments[0], CheckedTerm::Reference { target } if target == &declarations[index]));
        let argument_node_ids = arguments[1..].iter().map(|argument| match argument {
            CheckedTerm::Reference { target } => target.clone(),
            other => panic!("original call argument reference: {other:?}"),
        }).collect();
        ExactFunctionItem { call_node_id: call.clone(), function: names[index].to_owned(), argument_node_ids }
    }).collect::<Vec<_>>();
    let output = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &[0, 1, 2],
        &functions,
        &[3, 4, 5],
        &items,
    )
    .expect("the selected original function rows generate");
    assert_eq!(output.claim_map.items.len(), names.len());
    let mut driver = include_str!("support/family_function_observation.rs")
        .replace("__AUTHORED_SOURCE__", &format!("{SOURCE:?}"));
    for index in 0..names.len() {
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == calls[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one exact requested call claim");
        };
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} must generate: {:?}", names[index], claim.result);
        };
        assert_eq!(generated.function_node_id, declarations[index]);
        driver = driver.replace(&format!("__SYMBOL_{index}__"), &generated.oracle_symbol);
    }
    let driver = driver.replace(
        "include!(\"family_function_selection.rs\");",
        include_str!("support/family_function_selection.rs"),
    );
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        ["SC05 function rows: 9 envelopes, 12 input refusals, 3 owner refusals"]
    );
}
