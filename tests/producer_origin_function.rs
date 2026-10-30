// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-016-SC-04 / TC-096: TC-194 function observations from QSL-admitted IDs.

mod support;

use std::{
    cell::{Cell, RefCell},
    fs,
    process::Command,
    rc::Rc,
    sync::OnceLock,
};

use ix_trace_rs::trace;
use qsl_foundation::node_wire::node_key_from_json;
use qsl_semantics::model::{intake::package_input, key::hex};
use quire_contract_ir::{
    read_checked_package, CheckedNodeOwner, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedSemanticNodeV2,
};
use quire_contract_runtime::exact::{
    plan_call, plan_evaluation, Body, BodyStep, CheckMode, CheckedPackage, CheckingLimits,
    EvaluationRefusal, Frame, FunctionDeclaration, InputRefusal, PackageDeclarations,
};
use quire_exact::{
    ChargePoint, CheckedInvariantCause, EffectiveId, Integer, LimitKind, Meter, NodeKey, ObjectId,
    ObjectReference, OriginalDeclarationKey, Outcome, Refusal, ScalarLimits, UnionMember,
    UnionValue, UniverseId, Value, ValueType,
};
use quire_semantic_value::{
    declaration::{
        CompositeDeclaration, CompositeShape, ObjectTypeDeclaration, TypeEnvironment,
        UnionMemberDeclaration,
    },
    object_closure::ObjectClosure,
};
use serde_json::Value as JsonValue;

const SOURCE_PATH: &str = "program.native";
const MODEL_PATH: &str = "tests/fixtures/spine-model.semantic-ir.json";
const LIMITS: ScalarLimits = ScalarLimits {
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
};

fn meter() -> Meter {
    Meter::new(LIMITS)
}

/// The model document remains in the QSL checkout selected by Cargo.lock.
/// Its digest is computed only for QSL's model-selection content identity.
fn admitted() -> &'static CheckedPackageV2 {
    static PACKAGE: OnceLock<CheckedPackageV2> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let model = support::qsl_root().join(MODEL_PATH);
        let document = fs::read(&model).expect("QSL-owned model document is present");
        let documents = package_input([document.as_slice()]);
        let [(digest, _)] = documents.iter().collect::<Vec<_>>()[..] else {
            panic!("the QSL producer has one model document");
        };
        let digest = hex(digest);
        let source = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n\
             model M = \"acme/orders\" version \"1.0.0\" digest \"sha256-jcs:{digest}\";\n\
             union RefChoice {{ Empty, WithRef(Reference<M::Widget>) }}\n\
             function identity using v(x: Boolean): Boolean pure {{ x }}\n\
             function held using v(w: Reference<M::Widget>): Boolean pure {{ true }}\n\
             function two using v(a: Boolean, b: Reference<M::Widget>): Boolean pure {{ a }}\n\
             function take_union using v(u: RefChoice): Boolean pure {{ true }}\n\
             function observe using v(): Boolean pure {{ true }}\n\
             function leaf using v(): Boolean pure {{ true }}\n\
             function parent using v(): Boolean pure {{ true }}\n\
             function borrow using v(): Boolean pure {{ true }}\n\
             function reenter using v(): Boolean pure {{ true }}\n"
        );
        let directory = tempfile::tempdir().expect("source directory");
        fs::write(directory.path().join(SOURCE_PATH), source).expect("write authored QSL source");
        let output = Command::new(env!("CARGO"))
            .current_dir(directory.path())
            .env("CARGO_TARGET_DIR", std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
            .arg("run").arg("--locked").arg("--quiet")
            .arg("--manifest-path").arg(support::qsl_root().join("Cargo.toml"))
            .args(["--example", "emit_checked_v2", "--", "--model"])
            .arg(&model)
            .args(["--package-name", "agent-ix/it-016-functions", SOURCE_PATH])
            .output().expect("locked QSL emitter starts");
        assert!(output.status.success(), "QSL emitter: {}", String::from_utf8_lossy(&output.stderr));
        let wire: JsonValue = serde_json::from_slice(&output.stdout).expect("QSL emits JSON");
        let mut evidence = CheckedPackageEvidence::new();
        for feature in wire["lock"]["required_features"].as_array().expect("feature list") {
            evidence.support_feature(feature.as_str().expect("feature name"));
        }
        evidence.insert_domain_package_document(&digest, &document)
            .expect("the selected model document is valid evidence");
        let package = match read_checked_package(
            &output.stdout, CheckedPackageReadLimits::bounded(), &evidence,
        ) {
            CheckedPackageDispatchResult::AdmittedV2(package) => *package,
            other => panic!("IR refused QSL output: {other:?}"),
        };
        eprintln!("producer=cargo run --locked --example emit_checked_v2; application={}; source-revision=Cargo.lock; outcome=AdmittedV2({})", env!("CARGO_PKG_VERSION"), package.name());
        package
    })
}

fn declaration_node(name: &str) -> &'static CheckedSemanticNodeV2 {
    let nodes: Vec<_> = admitted()
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .last()
                    .is_some_and(|part| &**part == name)
            })
        })
        .collect();
    let [node] = nodes.as_slice() else {
        panic!("one admitted declaration {name}, got {}", nodes.len())
    };
    node
}

fn declaration_key(name: &str) -> NodeKey {
    let node = declaration_node(name);
    node_key_from_json(&serde_json::to_value(&node.node_id).expect("node ID serializes"))
        .expect("admitted structural node ID decodes")
}

fn model_object() -> (EffectiveId, UniverseId) {
    let nodes: Vec<_> = admitted()
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .last()
                    .is_some_and(|part| &**part == "Widget")
            }) && matches!(&node.owner, Some(CheckedNodeOwner::Model { .. }))
        })
        .collect();
    let [node] = nodes.as_slice() else {
        panic!("one admitted model Widget, got {}", nodes.len())
    };
    let Some(CheckedNodeOwner::Model {
        identity,
        node: original,
    }) = &node.owner
    else {
        panic!("Widget has model owner");
    };
    let key = node_key_from_json(&serde_json::to_value(&node.node_id).expect("node ID serializes"))
        .expect("admitted model node ID decodes");
    (
        EffectiveId::new(
            None,
            OriginalDeclarationKey::new(identity.to_string(), original.to_string()),
        ),
        UniverseId::new(key),
    )
}

fn reference() -> ObjectReference {
    let (effective, universe) = model_object();
    ObjectReference::new(
        universe,
        effective,
        ObjectId::new("o").expect("object identifier"),
    )
}

fn types() -> TypeEnvironment {
    let (effective, _) = model_object();
    TypeEnvironment::new(
        [],
        [ObjectTypeDeclaration::new(effective, "Widget", Vec::new())],
    )
    .expect("admitted Widget type")
}

fn one_step<F>(run: F) -> Body
where
    F: for<'a, 'm> Fn(&Frame<'a, 'm>, &[Value]) -> Outcome<Value> + 'static,
{
    let run = Rc::new(run);
    Box::new(move |arguments| {
        let run = Rc::clone(&run);
        Box::new(move |frame: &Frame<'_, '_>, _: Option<Value>| {
            BodyStep::Return(run(frame, &arguments))
        })
    })
}

fn function(name: &str, parameters: Vec<(&str, ValueType)>, body: Body) -> FunctionDeclaration {
    // The name passed to RT is taken back from the IR-admitted declaration.
    let node = declaration_node(name);
    let _key = declaration_key(name);
    let admitted_name = node
        .declaration
        .as_ref()
        .expect("function declaration")
        .qualified_name
        .last()
        .expect("declared function name")
        .to_string();
    FunctionDeclaration {
        name: admitted_name,
        parameters: parameters
            .into_iter()
            .map(|(name, ty)| (name.to_owned(), ty))
            .collect(),
        result: ValueType::Boolean,
        ieee_requirements: Vec::new(),
        integer_division_consumers: Vec::new(),
        measure_discharged: true,
        body,
    }
}

fn checked(functions: Vec<FunctionDeclaration>, types: TypeEnvironment) -> CheckedPackage {
    PackageDeclarations { types, functions }
        .check(CheckMode::Linked, CheckingLimits::default())
        .unwrap_or_else(|refusals| panic!("admitted functions failed RT check: {refusals:?}"))
}

fn identity() -> CheckedPackage {
    checked(
        vec![function(
            "identity",
            vec![("x", ValueType::Boolean)],
            one_step(|_, args| Outcome::Completed(args[0].clone())),
        )],
        TypeEnvironment::default(),
    )
}

fn two() -> CheckedPackage {
    let (effective, _) = model_object();
    checked(
        vec![function(
            "two",
            vec![
                ("a", ValueType::Boolean),
                ("b", ValueType::Reference(effective)),
            ],
            one_step(|_, args| Outcome::Completed(args[0].clone())),
        )],
        types(),
    )
}

fn held() -> CheckedPackage {
    let (effective, _) = model_object();
    checked(
        vec![function(
            "held",
            vec![("w", ValueType::Reference(effective))],
            one_step(|_, _| Outcome::Completed(Value::Boolean(true))),
        )],
        types(),
    )
}

/// Arity wins over a malformed argument; parameter order wins over a later
/// dangling reference. Every refusal precedes the first meter charge.
#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-2", "FR-273-AC-3")]
fn admitted_function_arguments_keep_precedence_and_zero_charge() {
    let identity = identity();
    let mut identity_meter = meter();
    let success = identity
        .call(
            "identity",
            vec![Value::Boolean(true)],
            &ObjectClosure::default(),
            &mut identity_meter,
        )
        .expect("admitted identity function applies");
    assert!(matches!(
        success.outcome,
        Outcome::Completed(Value::Boolean(true))
    ));
    assert_eq!(
        identity_meter.admitted_charge_points(),
        &[ChargePoint::FunctionCall]
    );
    let mut wrong_meter = meter();
    assert_eq!(
        identity
            .call(
                "identity",
                vec![Value::Integer(Integer::one())],
                &ObjectClosure::default(),
                &mut wrong_meter,
            )
            .unwrap_err(),
        InputRefusal::WrongValueKind { parameter: 0 }
    );
    assert!(wrong_meter.admitted_charges().is_empty());

    let held = held();
    let mut held_meter = meter();
    let refusal = held
        .call(
            "held",
            vec![Value::Reference(reference())],
            &ObjectClosure::default(),
            &mut held_meter,
        )
        .expect_err("the direct model reference has no live object");
    assert_eq!(refusal, InputRefusal::DanglingReference { parameter: 0 });
    assert_eq!(refusal.code(), "dangling_reference");
    assert_eq!(refusal.cause(), "absent-target-in-complete-population");
    assert!(held_meter.admitted_charges().is_empty());

    let package = two();
    let objects = ObjectClosure::default();
    let mut charged = meter();
    let refusal = package
        .call(
            "two",
            vec![Value::Reference(reference())],
            &objects,
            &mut charged,
        )
        .expect_err("arity is checked before argument kind and references");
    assert_eq!(
        refusal,
        InputRefusal::Arity {
            declared: 2,
            supplied: 1
        }
    );
    assert_eq!(refusal.code(), "invalid_runtime_input");
    assert_eq!(refusal.cause(), "wrong-value-kind");
    assert!(charged.admitted_charges().is_empty());
    assert!(LimitKind::ALL
        .iter()
        .all(|&kind| charged.consumed(kind) == 0));

    let mut charged = meter();
    let refusal = package
        .call(
            "two",
            vec![
                Value::Integer(Integer::one()),
                Value::Reference(reference()),
            ],
            &objects,
            &mut charged,
        )
        .expect_err("first wrong kind wins over later dangling reference");
    assert_eq!(refusal, InputRefusal::WrongValueKind { parameter: 0 });
    assert_eq!(refusal.code(), "invalid_runtime_input");
    assert_eq!(refusal.cause(), "wrong-value-kind");
    assert!(charged.admitted_charges().is_empty());
    assert!(LimitKind::ALL
        .iter()
        .all(|&kind| charged.consumed(kind) == 0));

    assert_eq!(
        plan_call(
            &package,
            "two",
            &[Value::Boolean(true), Value::Reference(reference())],
            &objects
        )
        .unwrap_err(),
        InputRefusal::DanglingReference { parameter: 1 }
    );
    let mut charged = meter();
    assert_eq!(
        package
            .call(
                "two",
                vec![Value::Boolean(true), Value::Reference(reference())],
                &objects,
                &mut charged
            )
            .unwrap_err(),
        InputRefusal::DanglingReference { parameter: 1 }
    );
    assert!(charged.admitted_charges().is_empty());
    assert_eq!(
        plan_call(&package, "missing", &[], &objects).unwrap_err(),
        InputRefusal::UnknownFunction("missing".to_owned())
    );
    assert_eq!(
        plan_call(&package, "two", &[], &objects).unwrap_err(),
        InputRefusal::Arity {
            declared: 2,
            supplied: 0
        }
    );
    assert_eq!(
        plan_call(
            &package,
            "two",
            &[
                Value::Integer(Integer::one()),
                Value::Reference(reference())
            ],
            &objects
        )
        .unwrap_err(),
        InputRefusal::WrongValueKind { parameter: 0 }
    );
    let mut charged = meter();
    assert_eq!(
        package
            .call("missing", Vec::new(), &objects, &mut charged)
            .unwrap_err(),
        InputRefusal::UnknownFunction("missing".to_owned())
    );
    assert!(charged.admitted_charges().is_empty());
}

/// The union declaration is QSL-authored; its member payload is a reference
/// to the admitted model Widget, so the nested target must still be checked.
#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-2", "FR-273-AC-3")]
fn admitted_union_argument_refuses_nested_dangling_reference() {
    let union = declaration_key("RefChoice");
    let member =
        UnionMember::from_admitted(union.clone(), "WithRef").expect("QSL-authored union member");
    let (effective, _) = model_object();
    let environment = TypeEnvironment::new(
        [CompositeDeclaration::new(
            union.clone(),
            "RefChoice",
            CompositeShape::Union(vec![UnionMemberDeclaration::from_verified(
                member.clone(),
                vec![ValueType::Reference(effective.clone())],
            )]),
        )],
        [ObjectTypeDeclaration::new(effective, "Widget", Vec::new())],
    )
    .expect("admitted union and object type");
    let package = checked(
        vec![function(
            "take_union",
            vec![("u", ValueType::Composite(union))],
            one_step(|_, _| Outcome::Completed(Value::Boolean(true))),
        )],
        environment,
    );
    let mut charged = meter();
    let refusal = package
        .call(
            "take_union",
            vec![UnionValue::from_admitted(
                member,
                vec![Value::Reference(reference())],
            )],
            &ObjectClosure::default(),
            &mut charged,
        )
        .expect_err("the union payload has a dangling reference");
    assert_eq!(refusal, InputRefusal::DanglingReference { parameter: 0 });
    assert!(charged.admitted_charges().is_empty());
}

/// The application charge is visible when the body starts; `evaluate`'s own
/// root has no application charge, while each reached child has one.
#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-2", "FR-273-AC-3")]
fn admitted_function_call_charges_before_body_once_per_application() {
    let package = checked(
        vec![function(
            "observe",
            Vec::new(),
            one_step(|frame, _| {
                match frame.meter(|meter| {
                    meter
                        .admitted_charge_points()
                        .contains(&ChargePoint::FunctionCall)
                }) {
                    Ok(charged) => Outcome::Completed(Value::Boolean(charged)),
                    Err(refusal) => Outcome::Refused(refusal),
                }
            }),
        )],
        TypeEnvironment::default(),
    );
    let mut charged = meter();
    let result = package
        .call(
            "observe",
            Vec::new(),
            &ObjectClosure::default(),
            &mut charged,
        )
        .expect("valid call");
    assert!(matches!(
        result.outcome,
        Outcome::Completed(Value::Boolean(true))
    ));
    assert_eq!(
        charged.admitted_charge_points(),
        &[ChargePoint::FunctionCall]
    );
    let plan = plan_call(&package, "observe", &[], &ObjectClosure::default()).expect("plan");
    assert_eq!(*plan.call_events(), Integer::one());

    let expression = package
        .check_expression(
            Vec::new(),
            ValueType::Boolean,
            one_step(|_, _| Outcome::Completed(Value::Boolean(true))),
        )
        .expect("checked expression");
    let plan =
        plan_evaluation(&package, &expression, &[], &ObjectClosure::default()).expect("plan");
    assert_eq!(*plan.call_events(), Integer::zero());
    let mut root_meter = meter();
    let result = package
        .evaluate(
            &expression,
            Vec::new(),
            &ObjectClosure::default(),
            &mut root_meter,
        )
        .expect("root expression");
    assert!(matches!(
        result.outcome,
        Outcome::Completed(Value::Boolean(true))
    ));
    assert!(root_meter.admitted_charges().is_empty());
}

/// `plan_evaluation` has one additional typed refusal for an expression
/// checked under another package, and runtime evaluation carries that cause.
#[test]
#[trace(
    "TC-096",
    "IT-016-SC-04",
    "TC-194",
    "FR-273-AC-2",
    "FR-273-AC-3",
    "FR-273-AC-5",
    "FR-273-AC-8"
)]
fn admitted_expression_keeps_plan_and_evaluation_refusal_distinctions() {
    let package = two();
    let (effective, _) = model_object();
    let expression = package
        .check_expression(
            vec![
                ("a".to_owned(), ValueType::Boolean),
                ("b".to_owned(), ValueType::Reference(effective)),
            ],
            ValueType::Boolean,
            one_step(|_, args| Outcome::Completed(args[0].clone())),
        )
        .expect("producer-owned expression types check");
    let objects = ObjectClosure::default();
    let live_objects = ObjectClosure::new(&types(), [(reference(), Vec::new())], &[])
        .expect("the admitted Widget has one live reference");
    let valid = [Value::Boolean(true), Value::Reference(reference())];
    assert!(plan_evaluation(&package, &expression, &valid, &live_objects).is_ok());
    let mut success_meter = meter();
    let success = package
        .evaluate(
            &expression,
            valid.to_vec(),
            &live_objects,
            &mut success_meter,
        )
        .expect("valid expression arguments");
    assert!(matches!(
        success.outcome,
        Outcome::Completed(Value::Boolean(true))
    ));
    assert!(success_meter.admitted_charges().is_empty());
    assert_eq!(
        plan_evaluation(&package, &expression, &[], &objects).unwrap_err(),
        EvaluationRefusal::Input(InputRefusal::Arity {
            declared: 2,
            supplied: 0
        })
    );
    assert_eq!(
        plan_evaluation(
            &package,
            &expression,
            &[
                Value::Integer(Integer::one()),
                Value::Reference(reference())
            ],
            &objects
        )
        .unwrap_err(),
        EvaluationRefusal::Input(InputRefusal::WrongValueKind { parameter: 0 })
    );
    assert_eq!(
        plan_evaluation(
            &package,
            &expression,
            &[Value::Boolean(true), Value::Reference(reference())],
            &objects
        )
        .unwrap_err(),
        EvaluationRefusal::Input(InputRefusal::DanglingReference { parameter: 1 })
    );
    let mut charged = meter();
    assert_eq!(
        package
            .evaluate(&expression, Vec::new(), &objects, &mut charged)
            .unwrap_err(),
        InputRefusal::Arity {
            declared: 2,
            supplied: 0
        }
    );
    assert!(charged.admitted_charges().is_empty());
    let mut charged = meter();
    assert_eq!(
        package
            .evaluate(
                &expression,
                vec![
                    Value::Integer(Integer::one()),
                    Value::Reference(reference())
                ],
                &objects,
                &mut charged
            )
            .unwrap_err(),
        InputRefusal::WrongValueKind { parameter: 0 }
    );
    assert!(charged.admitted_charges().is_empty());
    let mut charged = meter();
    assert_eq!(
        package
            .evaluate(
                &expression,
                vec![Value::Boolean(true), Value::Reference(reference())],
                &objects,
                &mut charged
            )
            .unwrap_err(),
        InputRefusal::DanglingReference { parameter: 1 }
    );
    assert!(charged.admitted_charges().is_empty());

    let other = two();
    // Arguments are valid for both packages, isolating the foreign owner.
    assert_eq!(
        plan_evaluation(&other, &expression, &valid, &live_objects).unwrap_err(),
        EvaluationRefusal::ForeignExpression
    );
    let mut charged = meter();
    let result = other
        .evaluate(&expression, valid.to_vec(), &live_objects, &mut charged)
        .expect("foreign expression is an evaluation outcome");
    assert!(matches!(
        result.outcome,
        Outcome::Refused(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::ForeignCheckedExpression
        })
    ));
    assert!(charged.admitted_charges().is_empty());
}

/// The generated child's missing name is distinct from a public unknown
/// function refusal, and the parent charge is the only admitted charge.
#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-8")]
fn admitted_parent_missing_child_has_typed_cause_and_no_child_charge() {
    let resumed = Rc::new(Cell::new(false));
    let body_resumed = Rc::clone(&resumed);
    let package = checked(
        vec![function(
            "parent",
            Vec::new(),
            Box::new(move |_| {
                let resumed = Rc::clone(&body_resumed);
                let mut first = true;
                Box::new(move |_: &Frame<'_, '_>, _: Option<Value>| {
                    if first {
                        first = false;
                        BodyStep::Call {
                            function: "absent".to_owned(),
                            arguments: Vec::new(),
                        }
                    } else {
                        resumed.set(true);
                        BodyStep::Return(Outcome::Completed(Value::Boolean(true)))
                    }
                })
            }),
        )],
        TypeEnvironment::default(),
    );
    let mut charged = meter();
    let result = package
        .call(
            "parent",
            Vec::new(),
            &ObjectClosure::default(),
            &mut charged,
        )
        .expect("valid parent call");
    assert!(matches!(
        result.outcome,
        Outcome::Refused(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::UnknownCheckedFunction
        })
    ));
    assert!(!resumed.get());
    assert_eq!(
        charged.admitted_charge_points(),
        &[ChargePoint::FunctionCall]
    );
}

#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-8")]
fn admitted_function_meter_borrow_conflict_has_typed_cause() {
    let package = checked(
        vec![function(
            "borrow",
            Vec::new(),
            one_step(|frame, _| {
                let inner = frame.meter(|_| frame.meter(|_| true));
                Outcome::Completed(Value::Boolean(matches!(
                    inner,
                    Ok(Err(Refusal::CheckedInvariant {
                        cause: CheckedInvariantCause::MeterBorrowConflict
                    }))
                )))
            }),
        )],
        TypeEnvironment::default(),
    );
    let mut charged = meter();
    let result = package
        .call(
            "borrow",
            Vec::new(),
            &ObjectClosure::default(),
            &mut charged,
        )
        .expect("valid call");
    assert!(matches!(
        result.outcome,
        Outcome::Completed(Value::Boolean(true))
    ));
    assert_eq!(charged.consumed(LimitKind::WorkUnits), 1);
}

#[test]
#[trace("TC-096", "IT-016-SC-04", "TC-194", "FR-273-AC-8")]
fn admitted_function_reentry_refuses_before_nested_charge() {
    let holder: Rc<RefCell<Option<Rc<CheckedPackage>>>> = Rc::new(RefCell::new(None));
    let body_holder = Rc::clone(&holder);
    let package = checked(
        vec![function(
            "reenter",
            Vec::new(),
            Box::new(move |_| {
                let holder = Rc::clone(&body_holder);
                Box::new(move |_: &Frame<'_, '_>, _: Option<Value>| {
                    let package = holder
                        .borrow()
                        .as_ref()
                        .expect("checked package installed")
                        .clone();
                    let mut nested_meter = meter();
                    let nested = package
                        .call(
                            "reenter",
                            Vec::new(),
                            &ObjectClosure::default(),
                            &mut nested_meter,
                        )
                        .expect("reentry is an outcome");
                    assert!(nested_meter.admitted_charges().is_empty());
                    BodyStep::Return(nested.outcome)
                })
            }),
        )],
        TypeEnvironment::default(),
    );
    let package = Rc::new(package);
    *holder.borrow_mut() = Some(Rc::clone(&package));
    let mut charged = meter();
    let result = package
        .call(
            "reenter",
            Vec::new(),
            &ObjectClosure::default(),
            &mut charged,
        )
        .expect("valid outer call");
    assert!(matches!(
        result.outcome,
        Outcome::Refused(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::PackageReentry
        })
    ));
    assert_eq!(
        charged.admitted_charge_points(),
        &[ChargePoint::FunctionCall]
    );
}
