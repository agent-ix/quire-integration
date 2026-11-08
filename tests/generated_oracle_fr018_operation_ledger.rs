// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-018 composite equality cells: the generated CG oracle and QSL's checked
//! operation, on the same original source and typed arguments, agree on the
//! outcome, every ordered admitted per-charge request and every counter.
//!
//! The QSL side is the public `call_observing_applications` ledger. The
//! generated side is quire-exact's production scoped charge capture around the
//! generated oracle call. Neither side subtracts or precharges anything.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/family_scalar_limits.rs"]
mod family_scalar_limits;
#[allow(dead_code)]
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[allow(dead_code)]
#[path = "support/oracle_observation.rs"]
mod oracle_observation;

use family_function_selection::body;
use family_scalar_limits::limits;

use ix_trace_rs::trace;
use qsl_eval::value::{ApplicationObservationOptions, CheckedPackageEvaluation, QualifiedName};
use qsl_replay::{
    acquire_checked_closure, CheckedClosureCompilation, CompileUnit, DependencyInput, ReplayLimits,
    SourceIdentity, StageLimits,
};
use quire_contract_codegen::{
    generate_composite_equality_oracles, routed_contract_lowering_profile, ClaimDisposition,
    CompositeEqualityItem, CompositeEqualityRefusal, ContractDriverResultSelection,
    ContractRequestPosition, EqualityOperandDescriptor, EqualityOperatorKind,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, ContractDependencies, ContractPackageReadResult,
    ContractPackageWireV1,
};
use quire_eval::FamilyOutcome;
use quire_exact::{ChargePoint, LimitKind, Meter, Outcome, Value};
use quire_semantic_value::object_closure::ObjectClosure;

/// Original declarations, closed input constructors and one comparison per
/// operator for each selected shape.
const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "tuple Pair(Boolean, Boolean);\n",
    "union Choice { First, Second }\n",
    "union PayloadChoice { First(Integer), Second(Integer) }\n",
    "record Flag { flag: Boolean; }\n",
    "record Outer { inner: Flag; level: Int[0,9]; }\n",
    "function pairEq using v(left: Pair, right: Pair): Boolean pure { left = right }\n",
    "function pairNe using v(left: Pair, right: Pair): Boolean pure { left != right }\n",
    "function flagEq using v(left: Flag, right: Flag): Boolean pure { left = right }\n",
    "function flagNe using v(left: Flag, right: Flag): Boolean pure { left != right }\n",
    "function outerEq using v(left: Outer, right: Outer): Boolean pure { left = right }\n",
    "function outerNe using v(left: Outer, right: Outer): Boolean pure { left != right }\n",
    "function seqEq using v(left: Sequence<Int[0,9]>[0,4], right: Sequence<Int[0,9]>[0,4]): Boolean pure { left = right }\n",
    "function seqNe using v(left: Sequence<Int[0,9]>[0,4], right: Sequence<Int[0,9]>[0,4]): Boolean pure { left != right }\n",
    "function choiceEq using v(left: Choice, right: Choice): Boolean pure { left = right }\n",
    "function choiceNe using v(left: Choice, right: Choice): Boolean pure { left != right }\n",
    "function payloadChoiceEq using v(left: PayloadChoice, right: PayloadChoice): Boolean pure { left = right }\n",
    "function payloadChoiceNe using v(left: PayloadChoice, right: PayloadChoice): Boolean pure { left != right }\n",
    "function makeChoiceA using v(): Choice pure { Choice::First }\n",
    "function makeChoiceB using v(): Choice pure { Choice::Second }\n",
    "function makePayloadChoiceA using v(): PayloadChoice pure { PayloadChoice::First(1) }\n",
    "function makePayloadChoiceB using v(): PayloadChoice pure { PayloadChoice::First(2) }\n",
    "function makePayloadChoiceC using v(): PayloadChoice pure { PayloadChoice::Second(1) }\n",
    "function makePairA using v(): Pair pure { Pair(true,false) }\n",
    "function makePairB using v(): Pair pure { Pair(false,true) }\n",
    "function makeFlagA using v(): Flag pure { Flag { flag: true } }\n",
    "function makeFlagB using v(): Flag pure { Flag { flag: false } }\n",
    "function makeOuterA using v(): Outer pure { Outer { inner: Flag { flag: true }, level: 3 } }\n",
    "function makeOuterB using v(): Outer pure { Outer { inner: Flag { flag: true }, level: 4 } }\n",
    "function makeOuterC using v(): Outer pure { Outer { inner: Flag { flag: false }, level: 3 } }\n",
    "function makeSeqA using v(): Sequence<Int[0,9]>[0,4] pure { sequence[1,2,3] }\n",
    "function makeSeqB using v(): Sequence<Int[0,9]>[0,4] pure { sequence[1,2,4] }\n",
    "function makeSeqC using v(): Sequence<Int[0,9]>[0,4] pure { sequence[1,2] }\n",
);

/// Shape, constructor names (the first is compared with itself and with each other).
const SHAPES: [(&str, &[&str]); 6] = [
    ("pair", &["makePairA", "makePairB"]),
    ("flag", &["makeFlagA", "makeFlagB"]),
    ("outer", &["makeOuterA", "makeOuterB", "makeOuterC"]),
    ("seq", &["makeSeqA", "makeSeqB", "makeSeqC"]),
    ("choice", &["makeChoiceA", "makeChoiceB"]),
    (
        "payloadChoice",
        &[
            "makePayloadChoiceA",
            "makePayloadChoiceB",
            "makePayloadChoiceC",
        ],
    ),
];

/// Work allowances: ample, exhausted before the first charge, and exhausted
/// after one, two and three admitted charges.
const WORK: [u64; 5] = [1024, 0, 1, 2, 3];

fn acquire() -> (CheckedClosureCompilation, CheckedPackageV2) {
    let acquisition = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "fr018-ledger"),
            package_name: "fr018-ledger".to_owned(),
            path: "fr018-ledger.native",
            bytes: SOURCE.as_bytes(),
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("original FR-018 source checks and emits");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquisition.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact FR-018 source emission admits");
    };
    (acquisition, *package)
}

/// Evaluate a closed original constructor through the checked package.
fn construct(checked: &qsl_package::CheckedPackage, name: &str) -> Value {
    let function = checked
        .graph()
        .function_by_identity(
            checked
                .graph()
                .function_identity(name)
                .expect("original constructor identity"),
        )
        .expect("original constructor body");
    let mut meter = Meter::new(limits(1024));
    let evaluation = qsl_eval::value::evaluate_closed_with_frame(
        checked,
        function.body,
        function.slots,
        Vec::new(),
        function.formed_units,
        &mut meter,
    )
    .expect("original constructor evaluates");
    assert!(evaluation.location.is_none());
    assert!(evaluation.losses.is_empty());
    let FamilyOutcome::Evaluated(Outcome::Completed(value)) = evaluation.outcome else {
        panic!("original constructor completes");
    };
    value
}

#[trace("TC-116", "IT-019-SC-05")]
#[test]
fn composite_equality_ledgers_match_the_generated_oracle() {
    let (acquisition, package) = acquire();
    let checked = family_checked(SOURCE);
    let mut items = Vec::new();
    let mut nodes = Vec::new();
    let mut names = Vec::new();
    for (shape, _) in SHAPES {
        for (suffix, operator) in [
            ("Eq", EqualityOperatorKind::Equal),
            ("Ne", EqualityOperatorKind::NotEqual),
        ] {
            let name = format!("{shape}{suffix}");
            let node_id = body(&package, &name);
            let Some(CheckedTerm::Application { arguments, .. }) = package.term(&node_id) else {
                panic!("original equality application");
            };
            let operand = |index: usize| {
                let CheckedTerm::Reference { target } = &arguments[index] else {
                    panic!("original parameter reference");
                };
                let selected = package
                    .resolved_graph()
                    .nodes
                    .iter()
                    .filter(|node| &node.node_id == target)
                    .collect::<Vec<_>>();
                let [node] = selected.as_slice() else {
                    panic!("one original parameter");
                };
                EqualityOperandDescriptor::typed(node.semantic_type.clone())
            };
            items.push(CompositeEqualityItem {
                node_id: node_id.clone(),
                operator,
                left: operand(0),
                right: operand(1),
            });
            nodes.push(node_id);
            names.push(name);
        }
    }
    let lowered = package.lower(&nodes, &routed_contract_lowering_profile());
    let contract = match ContractPackageWireV1::read_with_source(
        lowered.package.canonical_bytes().expect("lowered contract"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("the source authenticates its contract")
    {
        ContractPackageReadResult::Admitted(contract) => contract,
        other => panic!("the lowered contract admits: {other:?}"),
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
        Some(acquisition.context()),
        &positions,
        &indices,
        &items,
        Default::default(),
    )
    .expect("FR-018 composite generation");
    assert_eq!(output.claim_map.items.len(), items.len());

    let mut expected: Vec<String> = Vec::new();
    let mut driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Meter, Outcome, ScalarLimits, Value}};
fn lift(outcome: Outcome<bool>) -> Outcome<Value> {{
    match outcome {{ Outcome::Completed(v) => Outcome::Completed(Value::Boolean(v)), Outcome::Undefined(v) => Outcome::Undefined(v), Outcome::Refused(v) => Outcome::Refused(v), Outcome::Incomplete(v) => Outcome::Incomplete(v) }}
}}
#[test]
fn observe() {{
let source={SOURCE:?};
let acquired=qsl_replay::acquire_checked_closure(qsl_replay::CompileUnit {{source:qsl_replay::SourceIdentity::new("agent-ix","fr018-ledger"),package_name:"fr018-ledger".to_owned(),path:"fr018-ledger.native",bytes:source.as_bytes()}},[],&qsl_replay::DependencyInput::default(),Default::default(),Default::default()).expect("original source acquisition");
let mut evidence=quire_contract_model::CheckedPackageEvidence::new();
evidence.support_feature("quire.value.complete/v1");
let quire_contract_model::CheckedPackageV2ReadResult::Admitted(package)=quire_contract_model::CheckedPackageV2::read(acquired.package().bytes(),quire_contract_model::CheckedPackageReadLimits::bounded(),&evidence) else {{panic!("source package");}};
let handle=acquired.handle();
let owner=qsl_replay::LibraryName::new(package.name().to_owned()).unwrap();
let construct=|name:&str| -> Value {{
    let declarations=package.resolved_graph().nodes.iter().filter(|node|node.declaration.as_ref().is_some_and(|decl|decl.qualified_name.iter().map(AsRef::as_ref).eq([name]))).collect::<Vec<_>>();
    let [declaration]=declarations.as_slice() else {{panic!("original constructor declaration");}};
    let key=acquired.context().wire_graph().key(&declaration.node_id).expect("constructor key");
    let selected=handle.lookup_callable(&owner,&key).expect("constructor selection");
    let environment=qsl_replay::ApplicationEnvironment::new(quire_semantic_value::object_closure::ObjectClosure::default());
    let mut registry=handle.registry();
    let mut meter=Meter::new({ample:?});
    let evaluation=registry.begin_call(&selected,Vec::new(),&environment,&mut meter).expect("constructor admission").run().expect("constructor evaluation");
    match evaluation.outcome {{ quire_eval::FamilyOutcome::Evaluated(Outcome::Completed(value))=>value, _=>panic!("constructor completes") }}
}};
"#,
        ample = limits(1024),
    );

    let mut refused = 0;
    for (index, name) in names.iter().enumerate() {
        let shape_index = index / 2;
        let shape = SHAPES[shape_index].0;
        let (_, constructors) = SHAPES[shape_index];
        let equal_operator = index % 2 == 0;
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one claim for {name}");
        };
        let ClaimDisposition::Generated(generated) = &claim.result else {
            // The one typed owner refusal that is allowed: a payload-carrying union
            // operand. It is asserted, so any other refusal fails the test.
            assert!(
                shape == "payloadChoice"
                    && matches!(
                        &claim.result,
                        ClaimDisposition::Refused {
                            refusal: CompositeEqualityRefusal::CompositeContextMismatch { .. }
                        }
                    ),
                "{name} unexpected disposition: {:?}",
                claim.result
            );
            refused += 1;
            continue;
        };
        // Each input with itself, then the first with every other.
        let mut pairs = vec![(0, 0)];
        pairs.extend((1..constructors.len()).map(|other| (0, other)));
        for (left_index, right_index) in pairs {
            let left = construct(&checked, constructors[left_index]);
            let right = construct(&checked, constructors[right_index]);
            let equal_inputs = left_index == right_index;
            let answer = equal_inputs == equal_operator;
            expected.push(format!("input {left:?} {right:?}"));
            driver.push_str(&format!(
                r#"
{{ let prepared=quire_composite_equality_oracles::{environment}(Some(acquired.context())).expect("prepared environment");
let left=construct({left_name:?}); let right=construct({right_name:?});
println!("IT019_OBSERVATION input {{:?}} {{:?}}",left,right);
"#,
                environment = generated.environment_symbol,
                left_name = constructors[left_index],
                right_name = constructors[right_index],
            ));
            for work in WORK {
                // Independent QSL authority: the same checked original body on the
                // same typed arguments, with the production scoped capture.
                let function = checked
                    .graph()
                    .function_by_identity(checked.graph().function_identity(name).unwrap())
                    .expect("original comparison body");
                let mut meter = Meter::new(limits(work));
                meter.enable_charge_capture(64, 256);
                let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                    &checked,
                    function.body,
                    function.slots,
                    vec![left.clone(), right.clone()],
                    function.formed_units,
                    &mut meter,
                )
                .expect("independent checked-body evaluation");
                assert!(evaluation.losses.is_empty());
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("QSL value family");
                };
                let capture = meter
                    .take_charge_capture()
                    .expect("capture enabled")
                    .expect("complete capture");
                let (admitted, denied) = oracle_observation::split(capture.events());
                if work == 1024 {
                    assert!(
                        matches!(&outcome, Outcome::Completed(Value::Boolean(value)) if *value == answer),
                        "{name} {left_index}/{right_index}: {outcome:?}"
                    );
                    assert!(denied.is_empty());
                    assert!(!admitted.is_empty());
                    // The QSL call ledger carries the same operation requests after
                    // its own leading FunctionCall request.
                    let mut call_meter = Meter::new(limits(work));
                    let observation = checked.call_observing_applications(
                        &QualifiedName::unqualified(name).expect("declared name"),
                        vec![left.clone(), right.clone()],
                        &qsl_semantics::model::object_environment::ObjectEnvironment::new(
                            ObjectClosure::default(),
                        ),
                        &mut call_meter,
                        ApplicationObservationOptions {
                            function_case_limit: 1,
                            max_events: 64,
                            max_size_requests: 256,
                        },
                    );
                    let ledger = observation.ledger.as_ref().expect("complete QSL ledger");
                    let [call, operation @ ..] = ledger.entries() else {
                        panic!("QSL ledger has the call request");
                    };
                    assert_eq!(call.request.point, ChargePoint::FunctionCall);
                    assert_eq!(
                        operation
                            .iter()
                            .map(|entry| &entry.request)
                            .collect::<Vec<_>>(),
                        admitted,
                        "{name} QSL call ledger and operation capture"
                    );
                } else {
                    let Outcome::Incomplete(stop) = &outcome else {
                        panic!("{name} work={work} stops: {outcome:?}");
                    };
                    assert_eq!(stop.limit_kind, LimitKind::WorkUnits);
                    assert_eq!(denied, vec![stop.charge_point]);
                    assert!(admitted.len() as u64 <= work);
                }
                expected.push(oracle_observation::ledger_snapshot(
                    &outcome, &admitted, &denied, &meter,
                ));
                driver.push_str(&format!(
                    r#"
{{ let mut meter=Meter::new({allowance:?}); meter.enable_charge_capture(64,256);
let actual=lift(quire_composite_equality_oracles::{oracle}(&prepared,&left,&right,&mut meter));
let capture=meter.take_charge_capture().expect("capture enabled").expect("complete capture");
let (admitted,denied)=oracle_observation::split(capture.events());
println!("IT019_OBSERVATION {{}}",oracle_observation::ledger_snapshot(&actual,&admitted,&denied,&meter)); }}
"#,
                    allowance = limits(work),
                    oracle = generated.oracle_symbol,
                ));
            }
            driver.push_str("}\n");
        }
    }
    driver.push_str("}\n");
    assert_eq!(
        refused, 2,
        "payloadChoice Equal and NotEqual are typed owner refusals"
    );
    assert_eq!(
        expected.len(),
        144,
        "twelve credited input pairs (payload-carrying unions excluded), each with its inputs and five allowances, under both operators"
    );
    assert_eq!(
        generated_oracle::execute_with(
            &output.artifacts,
            &driver,
            "quire-eval = { git = \"https://github.com/agent-ix/quire-eval\", branch = \"main\", default-features = false }\n",
        ),
        expected
    );
}

/// A changed consumer descriptor keeps the owner's typed refusal and leaves the
/// healthy sibling claim, with its binding, unchanged.
#[trace("TC-116", "IT-019-SC-05")]
#[test]
fn changed_operand_descriptor_refuses_without_disturbing_a_sibling() {
    let (acquisition, package) = acquire();
    let nodes = [body(&package, "pairEq"), body(&package, "flagEq")];
    let items = nodes
        .iter()
        .map(|node_id| {
            let Some(CheckedTerm::Application { arguments, .. }) = package.term(node_id) else {
                panic!("original equality application");
            };
            let operand = |index: usize| {
                let CheckedTerm::Reference { target } = &arguments[index] else {
                    panic!("original parameter reference");
                };
                let node = package
                    .resolved_graph()
                    .nodes
                    .iter()
                    .find(|node| &node.node_id == target)
                    .expect("original parameter node");
                EqualityOperandDescriptor::typed(node.semantic_type.clone())
            };
            CompositeEqualityItem {
                node_id: node_id.clone(),
                operator: EqualityOperatorKind::Equal,
                left: operand(0),
                right: operand(1),
            }
        })
        .collect::<Vec<_>>();
    let lowered = package.lower(&nodes, &routed_contract_lowering_profile());
    let ContractPackageReadResult::Admitted(contract) = ContractPackageWireV1::read_with_source(
        lowered.package.canonical_bytes().expect("lowered contract"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("the source authenticates its contract") else {
        panic!("the lowered contract admits");
    };
    let positions = nodes
        .iter()
        .cloned()
        .map(|node_id| ContractRequestPosition::DriverItem {
            node_id,
            result_selection: ContractDriverResultSelection::NotApplicable,
        })
        .collect::<Vec<_>>();
    let generate = |items: &[CompositeEqualityItem]| {
        generate_composite_equality_oracles(
            &contract,
            Some(&package),
            Some(acquisition.context()),
            &positions,
            &[0, 1],
            items,
            Default::default(),
        )
        .expect("composite generation")
    };
    let healthy = generate(&items);
    let mut changed = items.clone();
    let pair_type = items[0].left.source_type.clone();
    let flag_type = items[1].left.source_type.clone();
    changed[1].left = EqualityOperandDescriptor::typed(pair_type.clone());
    let refused = generate(&changed);
    let claim = |output: &quire_contract_codegen::CompositeEqualityOracles,
                 node: &CheckedNodeId| {
        output
            .claim_map
            .items
            .iter()
            .find(|claim| &claim.node_id == node)
            .cloned()
            .expect("one claim per original node")
    };
    assert!(matches!(
        claim(&healthy, &nodes[1]).result,
        ClaimDisposition::Generated(_)
    ));
    assert!(
        matches!(
            claim(&refused, &nodes[1]).result,
            ClaimDisposition::Refused {
                refusal: CompositeEqualityRefusal::OperandTypeMismatch {
                    position: 0,
                    expected,
                    found: Some(found),
                }
            } if expected == pair_type && found == flag_type
        ),
        "changed descriptor refusal: {:?}",
        claim(&refused, &nodes[1]).result
    );
    // Symbols are numbered by claim order; every other field must be unchanged.
    let mut sibling = claim(&refused, &nodes[0]);
    let mut original = claim(&healthy, &nodes[0]);
    for claim in [&mut sibling, &mut original] {
        let ClaimDisposition::Generated(generated) = &mut claim.result else {
            panic!("the healthy sibling generates");
        };
        generated.environment_symbol.clear();
        generated.oracle_symbol.clear();
    }
    assert_eq!(sibling, original);
}

fn family_checked(source: &str) -> qsl_package::CheckedPackage {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "fr018-ledger"),
        "fr018-ledger.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original source forms");
    qsl_package::CheckedPackage::link(
        qsl_semantics::check::PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "fr018-ledger".to_owned(),
            unit,
            qsl_semantics::model::intake::SelectedModels::default(),
            Vec::new(),
        )
        .expect("original source assembles")
        .check(quire_semantic_value::checking::CheckingLimits::default())
        .expect("original source checks"),
    )
}
