// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected FR-021 operation rows; other SC-05 operations remain unqualified.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;

use family_function_selection::{body, declared};

use ix_trace_rs::trace;
use quire_contract_codegen::{
    generate_exact_function_oracles, routed_contract_lowering_profile, ClaimDisposition,
    ContractDriverResultSelection, ContractRequestPosition, EqualityOperatorKind,
    ExactFunctionBody, ExactFunctionDeclaration, ExactFunctionItem, FunctionParameter,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, ContractDependencies, ContractPackageReadResult,
    ContractPackageWireV1,
};

#[path = "support/family_function_equality_source.rs"]
mod family_function_equality_source;

#[trace("TC-104", "IT-019-SC-05")]
#[test]
fn primitive_function_equality_matches_fresh_qsl_sessions() {
    let (_, package) = family_function_equality_source::acquire(
        family_function_equality_source::SOURCE.as_bytes(),
    );
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
    let booleans = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "boolean"
        })
        .collect::<Vec<_>>();
    let [boolean] = booleans.as_slice() else {
        panic!("one canonical scalar Boolean type");
    };
    let boolean = boolean.node_id.clone();
    let names = ["boolEqual", "boolNotEqual", "intEqual", "intNotEqual"];
    let exercise = [
        "exerciseBoolEqual",
        "exerciseBoolNotEqual",
        "exerciseIntEqual",
        "exerciseIntNotEqual",
    ];
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
                    type_node_id: if index < 2 {
                        boolean.clone()
                    } else {
                        integer.clone()
                    },
                })
                .collect(),
            result_type: boolean.clone(),
            body: ExactFunctionBody::CompositeEquality {
                operator: if index % 2 == 0 {
                    EqualityOperatorKind::Equal
                } else {
                    EqualityOperatorKind::NotEqual
                },
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
        &[0, 1, 2, 3],
        &functions,
        &[4, 5, 6, 7],
        &items,
    )
    .expect("the selected original function rows generate");
    assert_eq!(output.claim_map.items.len(), names.len());
    let mut driver = include_str!("support/family_function_equality_observation.rs").to_owned();
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
    let driver = driver
        .replace(
            "include!(\"family_function_selection.rs\");",
            include_str!("support/family_function_selection.rs"),
        )
        .replace(
            "include!(\"family_scalar_limits.rs\");",
            include_str!("support/family_scalar_limits.rs"),
        );
    let driver = driver.replace(
        "include!(\"family_function_equality_source.rs\");",
        include_str!("support/family_function_equality_source.rs"),
    );
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        ["SC05 primitive function equality: 42 envelopes, 14 input refusals, 4 owner refusals"]
    );
}
