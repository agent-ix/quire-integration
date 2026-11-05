// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-019 SC-03: source occurrence and request identity are admitted separately.

#[path = "support/generated_oracle.rs"]
mod generated_oracle;

use ix_trace_rs::trace;
use qsl_replay::{
    acquire_checked_closure, CompileUnit, DependencyInput, ReplayLimits, SourceIdentity,
    StageLimits,
};
use quire_contract_codegen::oracle::operator_catalog::IntegerOperator;
use quire_contract_codegen::{
    generate_exact_function_oracles, routed_contract_lowering_profile, ClaimDisposition,
    ContractDriverResultSelection, ContractRequestPosition, ExactFunctionBody,
    ExactFunctionDeclaration, ExactFunctionItem, ExactFunctionRefusal, FunctionParameter,
    OracleGenerationError, PackageRequestError,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, ContractDependencies, ContractPackageReadResult,
    ContractPackageWireV1,
};

const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "function combine using v(left: Int[0,9], right: Int[0,9]): Integer pure { left + right }\n",
    "function exercise_a using v(left: Int[0,9], right: Int[0,9]): Integer pure { combine(left,right) }\n",
    "function exercise_b using v(x: Int[0,9], y: Int[0,9]): Integer pure { combine(x,y) }\n",
);

fn declared(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    let selected = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .iter()
                    .map(AsRef::as_ref)
                    .eq([name])
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1, "one exact authored declaration {name}");
    selected[0].node_id.clone()
}

fn body(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    let id = declared(package, name);
    let Some(CheckedTerm::Aggregate { members }) = package.term(&id) else {
        panic!("the selected declaration has a typed function header");
    };
    let targets = members
        .iter()
        .filter_map(|member| match member {
            CheckedTerm::Binding { name, value } if name.as_ref() == "body" => match &**value {
                CheckedTerm::Reference { target } => Some(target.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 1);
    targets[0].clone()
}

fn call_arguments(package: &CheckedPackageV2, call: &CheckedNodeId) -> Vec<CheckedNodeId> {
    let Some(CheckedTerm::Application { arguments, .. }) = package.term(call) else {
        panic!("the real source call has an application term");
    };
    assert!(
        matches!(&arguments[0], CheckedTerm::Reference { target } if target == &declared(package, "combine"))
    );
    arguments[1..]
        .iter()
        .map(|argument| match argument {
            CheckedTerm::Reference { target } => target.clone(),
            other => panic!("actual call argument must reference a checked node: {other:?}"),
        })
        .collect()
}

fn emitted_source(artifacts: &[quire_contract_codegen::Artifact]) -> &str {
    &artifacts
        .iter()
        .find(|artifact| artifact.path == "src/lib.rs")
        .expect("actual generated library")
        .contents
}

#[trace("TC-099", "IT-019-SC-03")]
#[test]
fn wrong_occurrence_refuses_without_changing_the_healthy_native_call() {
    let acquisition = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "oracle-function"),
            package_name: "oracle-function".to_owned(),
            path: "oracle-function.native",
            bytes: SOURCE.as_bytes(),
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("independent public source acquisition");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquisition.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("the exact public producer bytes admit");
    };
    let integer = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "integer"
        })
        .collect::<Vec<_>>();
    assert_eq!(integer.len(), 1);
    let integer = integer[0].node_id.clone();
    let function_body = body(&package, "combine");
    let call_a = body(&package, "exercise_a");
    let call_b = body(&package, "exercise_b");
    assert_ne!(call_a, call_b, "distinct real source occurrences");
    let args_a = call_arguments(&package, &call_a);
    let args_b = call_arguments(&package, &call_b);
    assert_ne!(
        args_a, args_b,
        "the occurrences have different checked operands"
    );

    let requested = [function_body.clone(), call_a.clone(), call_b.clone()];
    let lowered = package.lower(&requested, &routed_contract_lowering_profile());
    let contract = match ContractPackageWireV1::read_with_source(
        lowered
            .package
            .canonical_bytes()
            .expect("canonical lowering"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("source-authenticated lowering")
    {
        ContractPackageReadResult::Admitted(package) => package,
        other => panic!("the actual lowered contract admits: {other:?}"),
    };
    let positions = requested
        .iter()
        .cloned()
        .map(|node_id| ContractRequestPosition::DriverItem {
            node_id,
            result_selection: ContractDriverResultSelection::NotApplicable,
        })
        .collect::<Vec<_>>();
    let declaration = ExactFunctionDeclaration {
        node_id: function_body,
        name: "combine".to_owned(),
        parameters: ["left", "right"]
            .into_iter()
            .map(|name| FunctionParameter {
                name: name.to_owned(),
                type_node_id: integer.clone(),
            })
            .collect(),
        result_type: integer,
        body: ExactFunctionBody::Scalar {
            operator: IntegerOperator::Add,
        },
        capability_requirements: Vec::new(),
    };
    let healthy = ExactFunctionItem {
        call_node_id: call_b.clone(),
        function: "combine".to_owned(),
        argument_node_ids: args_b.clone(),
    };
    let wrong_occurrence = ExactFunctionItem {
        call_node_id: call_a.clone(),
        function: "combine".to_owned(),
        argument_node_ids: args_b,
    };
    let mixed = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &[0],
        &[declaration.clone()],
        &[1, 2],
        &[wrong_occurrence, healthy.clone()],
    )
    .expect("one malformed item cannot fail healthy generation");
    assert_eq!(mixed.claim_map.items.len(), 2);
    let refused = mixed
        .claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == call_a)
        .expect("wrong occurrence retains its original node identity");
    assert!(matches!(
        refused.result,
        ClaimDisposition::Refused {
            refusal: ExactFunctionRefusal::BodyMismatch
        }
    ));
    let mixed_healthy = mixed
        .claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == call_b)
        .expect("healthy sibling keeps its own claim");
    let ClaimDisposition::Generated(mixed_generated) = &mixed_healthy.result else {
        panic!("healthy sibling generates: {:?}", mixed_healthy.result);
    };
    let standalone = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &[0],
        &[declaration.clone()],
        &[2],
        &[healthy],
    )
    .expect("healthy item generates alone");
    let [standalone_claim] = standalone.claim_map.items.as_slice() else {
        panic!("one standalone healthy claim");
    };
    assert_eq!(
        mixed_healthy, standalone_claim,
        "same disposition and symbol"
    );
    assert_eq!(mixed.location_map, standalone.location_map);
    assert_eq!(
        emitted_source(&mixed.artifacts),
        emitted_source(&standalone.artifacts),
        "refused occurrence emits no function and cannot alter the healthy one"
    );

    let mut wrong_positions = positions.clone();
    wrong_positions[1] = ContractRequestPosition::DriverItem {
        node_id: call_b.clone(),
        result_selection: ContractDriverResultSelection::NotApplicable,
    };
    assert!(matches!(
        generate_exact_function_oracles(
            &package,
            &contract,
            &wrong_positions,
            &[0],
            &[declaration],
            &[2],
            &[ExactFunctionItem {
                call_node_id: call_b.clone(),
                function: "combine".to_owned(),
                argument_node_ids: args_a,
            }],
        ),
        Err(OracleGenerationError::PackageRequest(PackageRequestError::Identity {
            request_index: 1,
            requested,
            accounted,
        })) if requested == call_b && accounted == call_a
    ));

    let driver = include_str!("support/function_observation.rs")
        .replace("__AUTHORED_SOURCE__", &format!("{SOURCE:?}"))
        .replace("__GENERATED_SYMBOL__", &mixed_generated.oracle_symbol);
    let observations = generated_oracle::execute(&mixed.artifacts, &driver);
    assert_eq!(
        observations,
        ["whole-function-envelope: 3 exact comparisons"]
    );
}
