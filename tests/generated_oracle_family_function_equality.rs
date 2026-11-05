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
    let names = family_function_equality_source::NAMES;
    let exercise = family_function_equality_source::EXERCISES;
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
    let negate_body = body(&package, "negate");
    let negate_call = body(&package, "exerciseNegate");
    requested.extend([negate_body.clone(), negate_call.clone()]);
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
                    } else if index < 4 {
                        integer.clone()
                    } else {
                        let Some(CheckedTerm::Application { arguments, .. }) =
                            package.term(&bodies[index])
                        else {
                            panic!("actual nominal equality body");
                        };
                        let CheckedTerm::Reference { target } = &arguments[0] else {
                            panic!("actual nominal parameter reference");
                        };
                        let selected = package
                            .graph()
                            .nodes
                            .iter()
                            .filter(|node| &node.node_id == target)
                            .collect::<Vec<_>>();
                        let [node] = selected.as_slice() else {
                            panic!("one original nominal parameter");
                        };
                        node.semantic_type.clone()
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
    let declaration_positions = (0..names.len()).collect::<Vec<_>>();
    let item_positions = (names.len()..names.len() * 2).collect::<Vec<_>>();
    let output = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &declaration_positions,
        &functions,
        &item_positions,
        &items,
    )
    .expect("the selected original function rows generate");
    assert_eq!(output.claim_map.items.len(), names.len());
    // These are requests over original source nodes, not rewritten IR.
    use quire_contract_codegen::ExactFunctionRefusal;
    for boundary in ["result", "capability", "occurrence", "duplicate"] {
        let mut declarations = functions.clone();
        let mut requests = vec![items[0].clone(), items[9].clone()];
        let mut requested_positions = vec![item_positions[0], item_positions[9]];
        match boundary {
            "result" => declarations[0].result_type = integer.clone(),
            "capability" => {
                declarations[0].capability_requirements =
                    vec!["unsupported-equality-capability".to_owned()]
            }
            "occurrence" => {
                requests[0].call_node_id = calls[1].clone();
                requested_positions[0] = item_positions[1];
            }
            "duplicate" => {
                requests.push(items[0].clone());
                requested_positions.push(item_positions[0]);
            }
            _ => unreachable!(),
        }
        let measured = generate_exact_function_oracles(
            &package,
            &contract,
            &positions,
            &declaration_positions,
            &declarations,
            &requested_positions,
            &requests,
        )
        .expect("public source-backed boundary request");
        assert_eq!(
            measured.claim_map.items.len(),
            2,
            "duplicate occurrence collapses without swallowing sibling"
        );
        let refused = measured
            .claim_map
            .items
            .iter()
            .find(|claim| claim.node_id == requests[0].call_node_id)
            .expect("original refused call retained");
        let ClaimDisposition::Refused { refusal } = &refused.result else {
            panic!(
                "{boundary} must retain a typed refusal: {:?}",
                refused.result
            );
        };
        match boundary {
            "result" => assert!(matches!(
                refusal,
                ExactFunctionRefusal::SignatureMismatch { .. }
            )),
            "capability" => assert!(
                matches!(refusal, ExactFunctionRefusal::UnsupportedCapability { capability } if capability == "unsupported-equality-capability")
            ),
            "occurrence" => assert!(matches!(refusal, ExactFunctionRefusal::BodyMismatch)),
            "duplicate" => assert_eq!(refusal, &ExactFunctionRefusal::DuplicateRequest),
            _ => unreachable!(),
        }
        let sibling = measured
            .claim_map
            .items
            .iter()
            .find(|claim| claim.node_id == calls[9])
            .expect("healthy nominal sibling retained");
        assert!(
            matches!(sibling.result, ClaimDisposition::Generated(_)),
            "{boundary}: healthy sibling: {:?}",
            sibling.result
        );
    }
    let mut negate_functions = functions.clone();
    negate_functions.push(ExactFunctionDeclaration {
        node_id: negate_body,
        name: "negate".to_owned(),
        parameters: ["left", "right"]
            .into_iter()
            .map(|name| FunctionParameter {
                name: name.to_owned(),
                type_node_id: integer.clone(),
            })
            .collect(),
        result_type: integer,
        body: ExactFunctionBody::Scalar {
            operator: quire_contract_codegen::oracle::operator_catalog::IntegerOperator::Negate,
        },
        capability_requirements: Vec::new(),
    });
    let Some(CheckedTerm::Application { arguments, .. }) = package.term(&negate_call) else {
        panic!("original unary exercise call");
    };
    let negate_item = ExactFunctionItem {
        call_node_id: negate_call.clone(),
        function: "negate".to_owned(),
        argument_node_ids: arguments[1..]
            .iter()
            .map(|term| match term {
                CheckedTerm::Reference { target } => target.clone(),
                other => panic!("original negate argument: {other:?}"),
            })
            .collect(),
    };
    let mut negate_positions = declaration_positions.clone();
    negate_positions.push(names.len() * 2);
    let measured = generate_exact_function_oracles(
        &package,
        &contract,
        &positions,
        &negate_positions,
        &negate_functions,
        &[names.len() * 2 + 1, item_positions[9]],
        &[negate_item, items[9].clone()],
    )
    .expect("original unary body generation disposition");
    let refusal = measured
        .claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == negate_call)
        .expect("actual unary call retained");
    assert!(
        matches!(
            refusal.result,
            ClaimDisposition::Refused {
                refusal: ExactFunctionRefusal::UnsupportedOperator { .. }
            }
        ),
        "actual unary operator refusal: {:?}",
        refusal.result
    );
    assert!(matches!(
        measured
            .claim_map
            .items
            .iter()
            .find(|claim| claim.node_id == calls[9])
            .expect("healthy unary sibling retained")
            .result,
        ClaimDisposition::Generated(_)
    ));
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
    let driver = driver.replace(
        "include!(\"family_function_envelope.rs\");",
        include_str!("support/family_function_envelope.rs"),
    );
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        ["SC05 primitive function equality: 78 envelopes, 32 input refusals, 10 owner refusals"]
    );
}

#[path = "support/family_checked_source.rs"]
mod family_checked_source;
#[path = "support/family_scalar_limits.rs"]
mod family_scalar_limits;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;

#[trace("TC-104", "IT-019-SC-05")]
#[test]
fn primitive_composite_generator_matches_independent_checked_bodies() {
    use family_scalar_limits::limits;
    use quire_contract_codegen::{
        generate_composite_equality_oracles, CompositeEqualityItem, EqualityOperandDescriptor,
    };
    use quire_eval::FamilyOutcome;
    use quire_exact::{ChargePoint, Integer, LimitKind, Meter, Outcome, Value};
    // Answers are independent authored truth data, not a local evaluator.
    let mut rows = Vec::new();
    for (operator, syntax, answers) in [
        (EqualityOperatorKind::Equal, "=", [true, false, false, true]),
        (
            EqualityOperatorKind::NotEqual,
            "!=",
            [false, true, true, false],
        ),
    ] {
        for ((left, right), answer) in [
            ("false", "false"),
            ("false", "true"),
            ("true", "false"),
            ("true", "true"),
        ]
        .into_iter()
        .zip(answers)
        {
            rows.push((operator, syntax, left, right, answer, true));
        }
    }
    for (operator, syntax, answers) in [
        (EqualityOperatorKind::Equal, "=", [true, false, false]),
        (EqualityOperatorKind::NotEqual, "!=", [false, true, true]),
    ] {
        for ((left, right), answer) in [("3", "3"), ("2", "3"), ("4", "3")]
            .into_iter()
            .zip(answers)
        {
            rows.push((operator, syntax, left, right, answer, false));
        }
    }
    let mut source =
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n"
            .to_owned();
    for (index, (_, syntax, left, right, _, _)) in rows.iter().enumerate() {
        source.push_str(&format!(
            "function equality{index} using v(): Boolean pure {{ {left} {syntax} {right} }}\n"
        ));
    }
    let producer = family_checked_source::checked(&source);
    let emission = qsl_package::emit_checked(&producer).expect("actual equality source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("actual equality emission admits");
    };
    let nodes = (0..rows.len())
        .map(|index| body(&package, &format!("equality{index}")))
        .collect::<Vec<_>>();
    let items = nodes
        .iter()
        .zip(&rows)
        .map(|(node_id, (operator, ..))| {
            let Some(CheckedTerm::Application { arguments, .. }) = package.term(node_id) else {
                panic!("actual original equality application");
            };
            assert_eq!(arguments.len(), 2);
            let operand = |index: usize| {
                let target = match &arguments[index] {
                    CheckedTerm::Reference { target } => target,
                    CheckedTerm::Literal { type_id, .. } => {
                        return EqualityOperandDescriptor::typed(type_id.clone())
                    }
                    other => panic!("original typed equality operand: {other:?}"),
                };
                let selected = package
                    .graph()
                    .nodes
                    .iter()
                    .filter(|node| &node.node_id == target)
                    .collect::<Vec<_>>();
                let [node] = selected.as_slice() else {
                    panic!("one original operand node");
                };
                EqualityOperandDescriptor::typed(node.semantic_type.clone())
            };
            CompositeEqualityItem {
                node_id: node_id.clone(),
                operator: *operator,
                left: operand(0),
                right: operand(1),
            }
        })
        .collect::<Vec<_>>();
    let profile = quire_contract_ir::CompleteLoweringProfileV2 {
        reference: quire_contract_ir::LoweringProfileRef {
            authority: "agent-ix".into(),
            identity: "quire.codegen.composite-equality/v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: [
            quire_contract_ir::CheckedNodeTag::ScalarType,
            quire_contract_ir::CheckedNodeTag::CompositeType,
            quire_contract_ir::CheckedNodeTag::BoundedDomain,
            quire_contract_ir::CheckedNodeTag::Value,
            quire_contract_ir::CheckedNodeTag::Expression,
            quire_contract_ir::CheckedNodeTag::Claim,
            quire_contract_ir::CheckedNodeTag::Correspondence,
        ]
        .into(),
        require_bounds: false,
        work_limit: 65_536,
    };
    let lowered = package.lower(&nodes, &profile);
    let contract = match ContractPackageWireV1::read_with_source(
        lowered
            .package
            .canonical_bytes()
            .expect("actual lowered contract"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("source authentication")
    {
        ContractPackageReadResult::Admitted(contract) => contract,
        other => panic!("authentic equality contract admits: {other:?}"),
    };
    let positions = nodes
        .iter()
        .cloned()
        .map(|node_id| ContractRequestPosition::DriverItem {
            node_id,
            result_selection: ContractDriverResultSelection::NotApplicable,
        })
        .collect::<Vec<_>>();
    let indices = (0..nodes.len()).collect::<Vec<_>>();
    let output = generate_composite_equality_oracles(
        &contract,
        Some(&package),
        None,
        &positions,
        &indices,
        &items,
        Default::default(),
    )
    .expect("primitive composite API generation");
    assert_eq!(output.claim_map.items.len(), rows.len());
    let authority = family_checked_source::checked(&source);
    let mut expected = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{Meter,Outcome,ScalarLimits,Value};
fn lift(outcome: Outcome<bool>)->Outcome<Value> {
    match outcome { Outcome::Completed(v)=>Outcome::Completed(Value::Boolean(v)), Outcome::Undefined(v)=>Outcome::Undefined(v), Outcome::Refused(v)=>Outcome::Refused(v), Outcome::Incomplete(v)=>Outcome::Incomplete(v) }
}
#[test]
fn observe() {
"#,
    );
    for (index, (_, _, left, right, answer, boolean)) in rows.iter().enumerate() {
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one original equality claim");
        };
        if !boolean {
            let ClaimDisposition::Refused {
                refusal:
                    quire_contract_codegen::CompositeEqualityRefusal::OperandTypeMismatch {
                        position: 0,
                        expected,
                        found: None,
                    },
            } = &claim.result
            else {
                panic!(
                    "original inline Integer literal retains its typed generation refusal: {:?}",
                    claim.result
                );
            };
            assert_eq!(expected, &items[index].left.source_type);
            continue;
        }
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("healthy Boolean sibling generates: {:?}", claim.result);
        };
        let name = format!("equality{index}");
        let function = authority
            .graph()
            .function_by_identity(
                authority
                    .graph()
                    .function_identity(&name)
                    .expect("original checked function identity"),
            )
            .expect("original checked body");
        for work in [1024, 0, 1] {
            let mut meter = Meter::new(limits(work));
            let evaluation = qsl_eval::value::evaluate_closed(
                &authority,
                function.body,
                function.slots,
                &mut meter,
            )
            .expect("independent public checked-body evaluation");
            let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                panic!("primitive value outcome");
            };
            assert!(evaluation.losses.is_empty());
            if work == 1024 {
                assert!(
                    matches!(&outcome, Outcome::Completed(Value::Boolean(actual)) if actual == answer),
                    "required independent Boolean: {outcome:?}"
                );
                assert!(evaluation.location.is_none());
            } else {
                assert!(
                    matches!(&outcome, Outcome::Incomplete(stop) if stop.limit_kind == LimitKind::WorkUnits && stop.limit == work && stop.consumed == 0 && stop.next_charge == Integer::from(2_u64) && stop.charge_point == ChargePoint::EqualityPlanForm),
                    "required exact equality stop: {outcome:?}"
                );
                assert!(evaluation.location.is_some());
                assert!(meter.admitted_charges().is_empty());
                for kind in LimitKind::ALL {
                    assert_eq!(meter.consumed(kind), 0);
                }
            }
            let losses = evaluation
                .losses
                .into_iter()
                .map(|loss| loss.loss)
                .collect::<Vec<_>>();
            expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
            let argument = |literal: &str| {
                if *boolean {
                    format!("Value::Boolean({literal})")
                } else {
                    format!("Value::Integer(quire_exact::Integer::from({literal}_i64))")
                }
            };
            driver.push_str(&format!(
                r#"
{{ let prepared=quire_composite_equality_oracles::{}(None).expect("original primitive environment");
let mut meter=Meter::new({:?});
let actual=lift(quire_composite_equality_oracles::{}(&prepared,&{},&{},&mut meter));
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#,
                generated.environment_symbol,
                limits(work),
                generated.oracle_symbol,
                argument(left),
                argument(right)
            ));
        }
    }
    driver.push_str("}\n");
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        expected
    );
}
