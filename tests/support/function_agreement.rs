// SPDX-License-Identifier: AGPL-3.0-or-later
//! Genuine whole-function composition; no replacement function factory.

#[path = "family_function_selection.rs"]
mod family_function_selection;

use self::family_function_selection::{body, declared};

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
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm, ContractDependencies, ContractPackageReadResult, ContractPackageWireV1,
};

#[trace("TC-099", "IT-019-SC-01", "IT-019-SC-02")]
#[test]
fn actual_generated_function_preserves_the_whole_qsl_call_envelope() {
    // Both operands are genuinely bounded in the source. The result is plain
    // Integer, so this control does not conflate an implicit narrow with add.
    const SOURCE: &str = concat!(
        "language \"ix:native\" edition \"1-draft\";\n",
        "profile v = \"quire.value.complete/v1\";\n",
        "function combine using v(left: Int[0,9], right: Int[0,9]): Integer pure { left + right }\n",
        "function exercise using v(left: Int[0,9], right: Int[0,9]): Integer pure { combine(left,right) }\n",
    );
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
    let call = body(&package, "exercise");
    let Some(CheckedTerm::Application { arguments, .. }) = package.term(&call) else {
        panic!("the actual source probe emits a call application");
    };
    assert!(
        matches!(&arguments[0], CheckedTerm::Reference { target } if target == &declared(&package,"combine"))
    );
    let argument_node_ids = arguments[1..]
        .iter()
        .map(|argument| match argument {
            CheckedTerm::Reference { target } => target.clone(),
            other => panic!("actual probe argument must be a reference: {other:?}"),
        })
        .collect();
    let requested = vec![function_body.clone(), call.clone()];
    let lower = package.lower(&requested, &routed_contract_lowering_profile());
    let contract = match ContractPackageWireV1::read_with_source(
        lower
            .package
            .canonical_bytes()
            .expect("actual canonical lowering"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("source-authenticated lowering")
    {
        ContractPackageReadResult::Admitted(package) => package,
        other => panic!("actual lowered contract admits: {other:?}"),
    };
    let positions = requested
        .into_iter()
        .map(|node_id| ContractRequestPosition::DriverItem {
            node_id,
            result_selection: ContractDriverResultSelection::NotApplicable,
        })
        .collect::<Vec<_>>();
    let output = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &[0],
        &[ExactFunctionDeclaration {
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
        }],
        &[1],
        &[ExactFunctionItem {
            call_node_id: call,
            function: "combine".to_owned(),
            argument_node_ids,
        }],
    )
    .expect("public authentic function generation");
    let [claim] = output.claim_map.items.as_slice() else {
        panic!("one requested call claim");
    };
    let ClaimDisposition::Generated(generated) = &claim.result else {
        panic!("genuine bounded source generates: {:?}", claim.result);
    };
    assert_eq!(generated.function_node_id, declared(&package, "combine"));
    let driver = include_str!("function_observation.rs")
        .replace("__AUTHORED_SOURCE__", &format!("{SOURCE:?}"))
        .replace("__GENERATED_SYMBOL__", &generated.oracle_symbol);
    let driver = driver.replace(
        "include!(\"family_function_selection.rs\");",
        include_str!("family_function_selection.rs"),
    );
    let observations = super::generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(
        observations,
        ["whole-function-envelope: 3 exact comparisons"]
    );
}
