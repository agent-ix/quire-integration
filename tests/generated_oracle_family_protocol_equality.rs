// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original protocol-stage equality candidate, without function-body reconstruction.
#[path = "support/family_scalar_limits.rs"]
mod family_scalar_limits;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;
#[path = "support/qsl_root.rs"]
mod support;
use generated_oracle::cargo_command;
use quire_contract_codegen::{
    ClaimDisposition, CompositeEqualityItem, ContractDriverResultSelection,
    ContractRequestPosition, EqualityOperandDescriptor, EqualityOperatorKind,
};
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm, ContractDependencies, ContractPackageReadResult, ContractPackageWireV1,
};
use quire_eval::FamilyOutcome;
use quire_exact::{ChargePoint, Integer, LimitKind, Meter, Outcome, Value};

use ix_trace_rs::trace;

#[trace("TC-105", "IT-019-SC-05")]
#[test]
fn original_protocol_finish_equality_retains_its_source_occurrence() {
    for (operator, syntax, answer) in [
        (EqualityOperatorKind::Equal, "=", true),
        (EqualityOperatorKind::NotEqual, "!=", false),
    ] {
        compare_protocol_finish(operator, syntax, answer);
    }
}

fn compare_protocol_finish(operator: EqualityOperatorKind, syntax: &str, answer: bool) {
    let source = concat!(
        "language \"ix:native\" edition \"1-draft\";\n",
        "profile v = \"quire.value.complete/v1\";\n",
        "profile p = \"quire.protocol.finite-global/v1\";\n",
        "model M = \"acme/orders\";\n",
        "tuple Pair(Boolean, Boolean);\n",
        "function makePair using v(): Pair pure { Pair(true,false) }\n",
        "protocol Inspect using p over (input: M::Gadget) on origin {\n",
        "role Viewer on M::Gadget;\n",
        "run sequence Main {}\n",
        "finish End as (outcome: Pair) { outcome = outcome };\n",
        "}\n",
    )
    .replace("outcome = outcome", &format!("outcome {syntax} outcome"));
    let model =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("lock-selected model input");
    let producer = checked(&source, &model);
    let emitted = qsl_package::emit_checked(
        &producer,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("original protocol equality emits");
    assert!(emitted.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    evidence.support_feature("quire.protocol.finite-global/v1");
    evidence
        .insert_domain_package_document("acme/orders", model.as_slice())
        .expect("original model evidence");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emitted.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("genuine protocol equality emission admits");
    };
    let equality = package
        .resolved_graph()
        .nodes
        .iter()
        .filter(|node| node.semantic_form.as_ref() == "binary")
        .collect::<Vec<_>>();
    let [equality] = equality.as_slice() else {
        panic!("one original protocol equality occurrence: {equality:?}");
    };
    let CheckedTerm::Application { arguments, .. } = &equality.body else {
        panic!("actual protocol equality application");
    };
    assert_eq!(arguments.len(), 2);
    let operand = |index: usize| {
        let CheckedTerm::Reference { target } = &arguments[index] else {
            panic!("original protocol binder reference");
        };
        let nodes = package
            .resolved_graph()
            .nodes
            .iter()
            .filter(|node| &node.node_id == target)
            .collect::<Vec<_>>();
        let [node] = nodes.as_slice() else {
            panic!("one actual protocol binder");
        };
        EqualityOperandDescriptor::typed(node.semantic_type.clone())
    };
    let item = CompositeEqualityItem {
        node_id: equality.node_id.clone(),
        operator,
        left: operand(0),
        right: operand(1),
    };
    let acquisition = qsl_replay::compile_admitted(
        qsl_replay::CompileUnit {
            source: qsl_replay::SourceIdentity::new("agent-ix", "protocol-equality"),
            package_name: "agent-ix/protocol-equality".to_owned(),
            path: "protocol-equality.native",
            bytes: source.as_bytes(),
        },
        [model.as_slice()],
        &qsl_replay::DependencyInput::default(),
        Default::default(),
        Default::default(),
    )
    .expect("actual protocol source context");
    let lowered = package.lower(
        std::slice::from_ref(&item.node_id),
        &quire_contract_codegen::routed_contract_lowering_profile(),
    );
    let contract = match ContractPackageWireV1::read_with_source(
        lowered
            .package
            .canonical_bytes()
            .expect("original protocol equality lowering"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &package,
    )
    .expect("authentic original protocol contract")
    {
        ContractPackageReadResult::Admitted(contract) => contract,
        other => panic!("actual protocol contract admits: {other:?}"),
    };
    let output = quire_contract_codegen::generate_composite_equality_oracles(
        &contract,
        Some(&package),
        Some(acquisition.context()),
        &[ContractRequestPosition::DriverItem {
            node_id: item.node_id.clone(),
            result_selection: ContractDriverResultSelection::NotApplicable,
        }],
        &[0],
        &[item],
        Default::default(),
    )
    .expect("actual protocol equality generation");
    let [claim] = output.claim_map.items.as_slice() else {
        panic!("one actual protocol equality claim");
    };
    let ClaimDisposition::Generated(generated) = &claim.result else {
        panic!("original protocol equality disposition: {:?}", claim.result);
    };
    let checked = checked(&source, &model);
    let protocol = checked
        .resolved_graph()
        .protocol("Inspect")
        .expect("original checked protocol");
    let [slot] = protocol.slots.bodies.as_slice() else {
        panic!("one original finish body");
    };
    let finish = protocol.finish.as_ref().expect("actual finish identity");
    assert_eq!(slot.node, finish.0);
    let correspondence = package
        .checked_term_data()
        .expect("original protocol term data")
        .protocol_locations
        .iter()
        .filter(|row| row.application == equality.node_id)
        .collect::<Vec<_>>();
    let [correspondence] = correspondence.as_slice() else {
        panic!("one original protocol equality location");
    };
    assert_eq!(
        package
            .protocol_location(
                &correspondence.application,
                &correspondence.occurrence,
                correspondence.term_ordinal,
                1024
            )
            .expect("actual retained occurrence"),
        *correspondence
    );
    assert!(correspondence.location.path.is_empty());
    match (
        &correspondence.location.origin,
        &slot.body.root().location().origin,
    ) {
        (
            quire_contract_ir::CheckedProtocolLocationOrigin::TypeDeclaration { name: emitted },
            quire_semantic_value::location::Origin::TypeDeclaration { name: checked },
        ) => assert_eq!(emitted.as_ref(), checked.as_str()),
        other => panic!("original protocol body location correspondence: {other:?}"),
    }
    let input_slot = protocol
        .slots
        .binders
        .iter()
        .position(|binder| binder.name == "input")
        .expect("original model input binder");
    assert!(
        protocol.slots.readers[input_slot].readers.is_empty(),
        "the compared finish does not read the unbound model input"
    );
    let outcome_slot = protocol
        .slots
        .binders
        .iter()
        .position(|binder| binder.name == "outcome")
        .expect("original outcome binder slot");
    let function = checked
        .resolved_graph()
        .function_by_identity(
            checked
                .resolved_graph()
                .function_identity("makePair")
                .expect("actual constructor"),
        )
        .expect("actual constructor body");
    let mut input_meter = Meter::new(family_scalar_limits::limits(1024));
    let input = qsl_eval::value::evaluate_closed_with_frame(
        &checked,
        function.body,
        function.slots,
        Vec::new(),
        function.formed_units,
        &mut input_meter,
    )
    .expect("original source constructor");
    assert!(input.location.is_none());
    assert!(input.losses.is_empty());
    let FamilyOutcome::Evaluated(Outcome::Completed(input)) = input.outcome else {
        panic!("original source constructor completes");
    };
    let Value::Composite(composite) = &input else {
        panic!("original constructor returns its tuple");
    };
    assert!(matches!(
        composite.slots(),
        [
            quire_exact::FieldValue::Present(Value::Boolean(true)),
            quire_exact::FieldValue::Present(Value::Boolean(false))
        ]
    ));
    let quire_exact::ValueType::Composite(owner_key) =
        &protocol.slots.binders[outcome_slot].value_type
    else {
        panic!("original finish binder nominal owner");
    };
    assert_eq!(composite.declaration(), owner_key);
    assert!(protocol.slots.binders[outcome_slot]
        .value_type
        .admits(&input));
    assert_eq!(slot.slot_names[outcome_slot], "outcome");
    let mut expected = vec![format!("source input: {input:?}")];
    let mut driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Value,Meter,Outcome,ScalarLimits}};
fn lift(outcome: Outcome<bool>)->Outcome<Value> {{ match outcome {{ Outcome::Completed(v)=>Outcome::Completed(Value::Boolean(v)), Outcome::Undefined(v)=>Outcome::Undefined(v), Outcome::Refused(v)=>Outcome::Refused(v), Outcome::Incomplete(v)=>Outcome::Incomplete(v) }} }}
#[test]
fn observe() {{
let model: &[u8]=&{model:?};
let source={source:?};
let acquired=qsl_replay::acquire_checked_closure(qsl_replay::CompileUnit {{source:qsl_replay::SourceIdentity::new("agent-ix","protocol-equality"), package_name:"agent-ix/protocol-equality".to_owned(),path:"protocol-equality.native",bytes:source.as_bytes()}},[model],&qsl_replay::DependencyInput::default(),Default::default(),Default::default()).expect("original source constructor acquisition");
"#
    );
    // The native input is the same original source constructor result transported
    // through the public checked-call interface, never a reconstructed owner key.
    driver.push_str(r#"
let mut evidence=quire_contract_model::CheckedPackageEvidence::new();
evidence.support_feature("quire.value.complete/v1");
evidence.support_feature("quire.protocol.finite-global/v1");
evidence.insert_domain_package_document("acme/orders",model).unwrap();
let quire_contract_model::CheckedPackageV2ReadResult::Admitted(package)=quire_contract_model::CheckedPackageV2::read(acquired.package().bytes(),quire_contract_model::CheckedPackageReadLimits::bounded(),&evidence) else {panic!("source package");};
let declarations=package.resolved_graph().nodes.iter().filter(|node|node.declaration.as_ref().is_some_and(|decl|decl.qualified_name.iter().map(AsRef::as_ref).eq(["makePair"]))).collect::<Vec<_>>();
let [declaration]=declarations.as_slice() else {panic!("original constructor declaration");};
let key=acquired.context().wire_graph().key(&declaration.node_id).expect("actual source constructor key");
let handle=acquired.handle();
let owner=qsl_replay::LibraryName::new(package.name().to_owned()).unwrap();
let selection=handle.lookup_callable(&owner,&key).unwrap();
let quire_exact::ValueType::Composite(declaration_key)=selection.signature().result() else {panic!("actual constructor result nominal type");};
let declaration=handle.nominal_declaration(&selection,declaration_key).expect("actual source declaration metadata");
assert_eq!(declaration.form(),qsl_replay::NominalDeclarationForm::Tuple);
let shape=declaration.members().iter().enumerate().map(|(index,member)| match member {
qsl_replay::NominalDeclarationMember::TuplePosition{position,value_type} => {assert_eq!(*position,index);value_type.clone()},
_=>panic!("source tuple position") }).collect::<Vec<_>>();
let input=quire_exact::tuple(&shape,declaration.key().clone(),vec![Value::Boolean(true),Value::Boolean(false)]).expect("public construction from actual owner metadata");
println!("IT019_OBSERVATION source input: {:?}",input);
"#);
    for work in [1024, 0, 1] {
        let mut frame = vec![None; protocol.slots.binders.len()];
        frame[outcome_slot] = Some(input.clone());
        let mut meter = Meter::new(family_scalar_limits::limits(work));
        let current = qsl_semantics::model::object_environment::ObjectEnvironment::new(
            quire_semantic_value::object_closure::ObjectClosure::default(),
        );
        let evaluation = qsl_eval::value::evaluate_protocol_slot(
            &checked, &slot.body, frame, &current, &mut meter,
        )
        .expect("original public finish slot evaluation");
        let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
            panic!("original nominal equality value family");
        };
        assert!(evaluation.losses.is_empty());
        if work == 1024 {
            assert!(
                matches!(outcome, Outcome::Completed(Value::Boolean(actual)) if actual == answer)
            );
            assert!(evaluation.location.is_none());
        } else {
            assert!(
                matches!(&outcome, Outcome::Incomplete(stop) if stop.limit_kind == LimitKind::WorkUnits && stop.limit == work && stop.consumed == 0 && stop.next_charge == Integer::from(6_u64) && stop.charge_point == ChargePoint::EqualityPlanForm),
                "actual original slot stop: {outcome:?}"
            );
            assert_eq!(
                evaluation.location.as_ref(),
                Some(slot.body.root().location())
            );
            assert!(meter.admitted_charges().is_empty());
            for kind in LimitKind::ALL {
                assert_eq!(meter.consumed(kind), 0);
            }
        }
        expected.push(oracle_observation::snapshot(
            &outcome,
            &Vec::<quire_semantic_value::loss::ValueLoss>::new(),
            &meter,
        ));
        driver.push_str(&format!(r#"{{ let prepared=quire_composite_equality_oracles::{}(Some(acquired.context())).unwrap(); let mut meter=Meter::new({:?}); let actual=lift(quire_composite_equality_oracles::{}(&prepared,&input,&input,&mut meter)); println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&Vec::<quire_semantic_value::loss::ValueLoss>::new(),&meter)); }}"#,generated.environment_symbol,family_scalar_limits::limits(work),generated.oracle_symbol));
    }
    driver.push('}');
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        expected
    );
}

fn checked(source: &str, model: &[u8]) -> qsl_package::CheckedPackage {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "protocol-equality"),
        "protocol-equality.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original protocol source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original protocol forms");
    let models = qsl_semantics::model::intake::admit_unit(
        &unit.selections().models,
        &qsl_semantics::model::intake::package_input([model]).expect("actual model input"),
        Default::default(),
    )
    .expect("original model selection");
    qsl_package::CheckedPackage::link(
        qsl_semantics::check::PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "agent-ix/protocol-equality".to_owned(),
            unit,
            models,
            Vec::new(),
        )
        .expect("original protocol assembly")
        .check(Default::default())
        .expect("original protocol checks"),
    )
}
