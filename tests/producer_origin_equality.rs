// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-016-SC-03: checked equality uses a QSL-emitted composite declaration.

#[path = "support/producer_origin.rs"]
mod producer_origin_support;
mod support;

use std::{fs, num::NonZeroU64, sync::OnceLock};

use ix_trace_rs::trace;
use qsl_foundation::node_wire::node_key_from_json;
use quire_contract_ir::{CheckedNodeOwner, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm};
use quire_exact::{
    ChargePoint, CheckedInvariantCause, CollectionKind, CollectionType, EffectiveId, EnumMember,
    FieldValue, FloatType, IeeeWidth, IllTypedCause, InjectedDenial, Integer, LimitKind, Meter,
    NodeKey, ObjectId, ObjectReference, OriginalDeclarationKey, Outcome, Presence, Refusal,
    ScalarLimits, UniverseId, Value, ValueType, VariantId,
};
use quire_semantic_value::declaration::{
    CheckedEquality, CompositeDeclaration, CompositeShape, EqualityOperand, EqualityOperator,
    FieldDeclaration, TypeEnvironment,
};
use quire_semantic_value::enumeration::EnumMemberIndex;
use serde_json::Value as JsonValue;

const SOURCE_PATH: &str = "program.native";
const SOURCE_BODY: &str = "\
    enum Phase { OPEN, SHUT }\n\
    record Six {\n\
      field0: Integer; field1: Integer; field2: Integer;\n\
      field3: Integer; field4: Integer; field5: Integer?;\n\
    }\n\
    record Other {\n\
      field0: Integer; field1: Integer; field2: Integer;\n\
      field3: Integer; field4: Integer; field5: Integer?;\n\
    }\n\
    function keep_six using v(x: Six): Six pure { x }\n\
    function keep_other using v(x: Other): Other pure { x }\n";

const MODEL_IDENTITY: &str = "acme/orders";

fn model_document_path() -> std::path::PathBuf {
    support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json")
}

fn model_document() -> Vec<u8> {
    fs::read(model_document_path()).expect("read QSL's authoritative model document")
}

fn source() -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model M = \"{MODEL_IDENTITY}\";\n\
         {SOURCE_BODY}\n\
         function keep_widget using v(x: M::Widget): M::Widget pure {{ x }}\n\
         function keep_gadget using v(x: M::Gadget): M::Gadget pure {{ x }}\n\
         function keep_rock using v(x: M::Rock): M::Rock pure {{ x }}\n",
    )
}

const LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 100,
    decimal_digits: 100,
    scale_expansion: 100,
    text_input_bytes: 100,
    text_scalars: 100,
    normalized_scalars: 100,
    unit_edges: 100,
    value_occurrences: 100,
    work_units: 100,
    result_units: 100,
};

fn admitted() -> &'static CheckedPackageV2 {
    static PACKAGE: OnceLock<CheckedPackageV2> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let model_path = model_document_path();
        let source = source();
        producer_origin_support::emit_and_read_checked_v2(
            producer_origin_support::CheckedProducerRequest {
                source_path: SOURCE_PATH,
                source: &source,
                package_name: "agent-ix/it-016-equality",
                models: &[producer_origin_support::ModelInput {
                    identity: MODEL_IDENTITY,
                    path: &model_path,
                }],
                execution_points: None,
            },
        )
    })
}

fn admitted_declaration_key(name: &str) -> NodeKey {
    let node = admitted_declaration(name);
    let wire = serde_json::to_value(&node.node_id).expect("admitted node ID serializes");
    node_key_from_json(&wire).expect("admitted structural ID is a NodeKey")
}

fn admitted_declaration(name: &str) -> &CheckedSemanticNodeV2 {
    let matches: Vec<_> = admitted()
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
    let [node] = matches.as_slice() else {
        panic!(
            "expected one admitted declaration {name}, found {}",
            matches.len()
        );
    };
    node
}

fn admitted_model_type(name: &str) -> (NodeKey, EffectiveId) {
    let original = format!("ix://{MODEL_IDENTITY}/{name}");
    let matches: Vec<_> = admitted()
        .graph()
        .nodes
        .iter()
        .filter_map(|node| match &node.owner {
            Some(CheckedNodeOwner::Model {
                identity,
                node: owner_node,
            }) if &**identity == MODEL_IDENTITY && **owner_node == original => {
                Some((node, identity.as_ref(), owner_node.as_ref()))
            }
            _ => None,
        })
        .collect();
    let [(node, owner_identity, owner_node)] = matches.as_slice() else {
        panic!(
            "expected one IR-admitted {original} model type, found {}",
            matches.len()
        );
    };
    let wire = serde_json::to_value(&node.node_id).expect("model node ID serializes");
    let key = node_key_from_json(&wire).expect("IR-admitted model node is a NodeKey");
    let effective = EffectiveId::new(
        None,
        OriginalDeclarationKey::new(*owner_identity, *owner_node),
    );
    (key, effective)
}

fn model_supertypes(name: &str) -> Vec<String> {
    let document: JsonValue =
        serde_json::from_slice(&model_document()).expect("QSL model document JSON");
    let identity = format!("ix://{MODEL_IDENTITY}/{name}");
    let matches: Vec<_> = document["types"]
        .as_array()
        .expect("QSL model type list")
        .iter()
        .filter(|entry| entry["identity"].as_str() == Some(identity.as_str()))
        .collect();
    let [entry] = matches.as_slice() else {
        panic!(
            "expected one QSL model type {identity}, found {}",
            matches.len()
        );
    };
    entry["supertypes"]
        .as_array()
        .expect("QSL model supertypes")
        .iter()
        .map(|value| value.as_str().expect("supertype identity").to_owned())
        .collect()
}

fn reference(universe: UniverseId, object_type: EffectiveId) -> Value {
    Value::Reference(ObjectReference::new(
        universe,
        object_type,
        ObjectId::new("shared-object").expect("authored nonempty object identity"),
    ))
}

fn record_shape() -> CompositeShape {
    CompositeShape::Record(
        (0..6)
            .map(|index| {
                FieldDeclaration::new(
                    format!("field{index}"),
                    ValueType::Integer,
                    if index == 5 {
                        Presence::Optional
                    } else {
                        Presence::Required
                    },
                )
            })
            .collect(),
    )
}

fn checked_equality() -> (TypeEnvironment, CheckedEquality, NodeKey) {
    let key = admitted_declaration_key("Six");
    let nested = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            ValueType::Composite(key.clone()),
            None,
        )),
        None,
    ));
    let declaration = CompositeDeclaration::new(key.clone(), "Six", record_shape());
    let types = TypeEnvironment::new([declaration], []).expect("admitted record shape");
    let checked = types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(nested.clone()),
            EqualityOperand::typed(nested),
            &EnumMemberIndex::default(),
        )
        .expect("matching admitted static types");
    (types, checked, key)
}

fn operand(types: &TypeEnvironment, key: NodeKey, first: i128) -> Value {
    let inner_type = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key.clone()),
        None,
    );
    let outer_type = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::collection(inner_type.clone()),
        None,
    );
    let mut construction_meter = Meter::new(LIMITS);
    const FIELD_NAMES: [&str; 6] = ["field0", "field1", "field2", "field3", "field4", "field5"];
    let inner = (0..2)
        .map(|composite_index| {
            let fields = (0..6)
                .filter(|field_index| composite_index == 0 || *field_index != 5)
                .map(|field_index| {
                    let number = if composite_index == 0 && field_index == 0 {
                        first
                    } else {
                        i128::from(composite_index * 6 + field_index)
                    };
                    (
                        FIELD_NAMES[field_index as usize],
                        FieldValue::Present(Value::Integer(Integer::from(number))),
                    )
                })
                .collect::<Vec<_>>();
            let composite = types
                .record(key.clone(), fields)
                .expect("admitted record fields");
            quire_exact::form_collection(&inner_type, vec![composite], &mut construction_meter)
                .expect("inner collection shape")
                .completed()
                .expect("inner collection completes")
        })
        .collect();
    quire_exact::form_collection(&outer_type, inner, &mut construction_meter)
        .expect("outer collection shape")
        .completed()
        .expect("outer collection completes")
}

fn admitted_points(meter: &Meter) -> Vec<ChargePoint> {
    meter
        .admitted_charges()
        .iter()
        .map(|charge| charge.point)
        .collect()
}

fn full_schedule() -> Vec<ChargePoint> {
    let mut points = vec![ChargePoint::EqualityPlanForm, ChargePoint::EqualityPlan];
    points.extend([ChargePoint::EqualityPair; 17]);
    points.push(ChargePoint::EqualityResultRetain);
    points
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_equality_pair_plan_and_mismatches_are_uncharged_and_typed() {
    let (types, _, key) = checked_equality();
    let value = operand(&types, key, 0);
    let plan = quire_exact::plan_equality(&value, &value).expect("well formed pair plan");
    assert_eq!(plan.pair_events(), &Integer::from(17_u64));
    let (_, widget_type) = admitted_model_type("Widget");
    let (widget_root, _) = admitted_model_type("Widget");
    let widget = reference(UniverseId::new(widget_root), widget_type.clone());
    let collection = |kind| {
        let ty = CollectionType::new(kind, ValueType::Reference(widget_type.clone()), None);
        quire_exact::form_collection(&ty, vec![], &mut Meter::new(LIMITS))
            .expect("empty typed collection")
            .completed()
            .expect("collection construction completes")
    };
    assert_eq!(
        quire_exact::plan_equality(
            &collection(CollectionKind::Sequence),
            &collection(CollectionKind::Set)
        ),
        Err(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::CollectionKindMismatch
        })
    );
    assert_eq!(
        quire_exact::plan_equality(&widget, &Value::Integer(Integer::one())),
        Err(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::ValueKindMismatch
        })
    );
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-025", "TC-026", "FR-008-AC-6")]
fn admitted_reference_identity_uses_exact_supplied_triple() {
    let (widget_root, widget_type) = admitted_model_type("Widget");
    let (rock_root, _) = admitted_model_type("Rock");
    let (_, gadget_type) = admitted_model_type("Gadget");
    let universe = UniverseId::new(widget_root);
    let object = ObjectId::new("shared-object").expect("authored object ID");
    let first = ObjectReference::new(universe.clone(), widget_type.clone(), object.clone());
    assert_eq!(first.universe(), &universe);
    assert_eq!(first.object_type(), &widget_type);
    assert_eq!(first.object(), &object);
    assert_eq!(
        first,
        ObjectReference::new(universe.clone(), widget_type.clone(), object.clone())
    );
    assert_ne!(
        first,
        ObjectReference::new(universe.clone(), gadget_type, object.clone())
    );
    assert_ne!(
        first,
        ObjectReference::new(
            universe.clone(),
            widget_type.clone(),
            ObjectId::new("other-object").expect("object ID")
        )
    );
    assert_ne!(
        first,
        ObjectReference::new(UniverseId::new(rock_root), widget_type, object)
    );
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_static_check_rejects_nested_ieee_before_equality_charge() {
    let key = admitted_declaration_key("Six");
    let types = TypeEnvironment::new([CompositeDeclaration::new(key, "Six", record_shape())], [])
        .expect("admitted record shape");
    let nested_ieee = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
        None,
    ));
    let refusal = types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(nested_ieee.clone()),
            EqualityOperand::typed(nested_ieee),
            &EnumMemberIndex::default(),
        )
        .expect_err("nested IEEE is equality-ineligible");
    assert_eq!(refusal.cause, IllTypedCause::OperatorIneligible);
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_equality_plan_form_denial_precedes_all_other_charges() {
    let (types, checked, key) = checked_equality();
    let value = operand(&types, key, 0);
    let mut meter = Meter::new(LIMITS).with_injected_denial(InjectedDenial {
        point: ChargePoint::EqualityPlanForm,
        occurrence: NonZeroU64::MIN,
    });
    let Outcome::Incomplete(record) = checked.evaluate(&types, &value, &value, &mut meter) else {
        panic!("plan formation denies before pair planning");
    };
    assert_eq!(record.charge_point, ChargePoint::EqualityPlanForm);
    assert!(admitted_points(&meter).is_empty());
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_checked_equality_rejects_wrong_operand_before_charging() {
    let (types, checked, key) = checked_equality();
    let left = operand(&types, key, 0);
    let wrong_right = Value::Integer(Integer::one());
    let mut meter = Meter::new(LIMITS);
    assert_eq!(
        checked.evaluate(&types, &left, &wrong_right, &mut meter),
        Outcome::Refused(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::EqualityOperandSourceNotAdmitted
        })
    );
    assert_eq!(meter.admission_count(), 0);
}

/// Trace: IT-016-SC-03, TC-096, TC-203, FR-277-AC-1, FR-277-AC-2.
#[test]
#[trace(
    "IT-016-SC-05",
    "IT-016-SC-03",
    "TC-096",
    "TC-203",
    "FR-277-AC-1",
    "FR-277-AC-2"
)]
fn admitted_record_equality_keeps_the_complete_seventeen_pair_schedule() {
    let (types, checked, key) = checked_equality();
    let left = operand(&types, key.clone(), 0);
    let equal_right = operand(&types, key.clone(), 0);
    let unequal_right = operand(&types, key, 99);
    assert_eq!(left.occ(), Integer::from(16_u64));
    for (right, expected) in [(&equal_right, true), (&unequal_right, false)] {
        let mut meter = Meter::new(LIMITS);
        assert!(matches!(
            checked.evaluate(&types, &left, right, &mut meter),
            Outcome::Completed(value) if value == expected
        ));
        assert_eq!(admitted_points(&meter), full_schedule());
    }
}

/// Trace: IT-016-SC-03, TC-096, TC-203, FR-277-AC-3.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-203", "FR-277-AC-3")]
fn denied_seventeenth_pair_and_result_retention_keep_the_admitted_prefix() {
    let (types, checked, key) = checked_equality();
    let value = operand(&types, key, 0);
    let schedule = full_schedule();
    let mut meter = Meter::new(LIMITS).with_injected_denial(InjectedDenial {
        point: ChargePoint::EqualityPair,
        occurrence: NonZeroU64::new(17).expect("nonzero occurrence"),
    });
    let Outcome::Incomplete(refusal) = checked.evaluate(&types, &value, &value, &mut meter) else {
        panic!("the seventeenth pair must be denied");
    };
    assert_eq!(refusal.charge_point, ChargePoint::EqualityPair);
    assert_eq!(admitted_points(&meter), schedule[..18]);

    let mut meter = Meter::new(LIMITS).with_injected_denial(InjectedDenial {
        point: ChargePoint::EqualityResultRetain,
        occurrence: NonZeroU64::MIN,
    });
    let Outcome::Incomplete(refusal) = checked.evaluate(&types, &value, &value, &mut meter) else {
        panic!("result retention must be denied");
    };
    assert_eq!(refusal.charge_point, ChargePoint::EqualityResultRetain);
    assert_eq!(admitted_points(&meter), schedule[..19]);
}

/// Trace: IT-016-SC-03, TC-096, TC-203, FR-277-AC-4.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-203", "FR-277-AC-4")]
fn distinct_admitted_static_composite_types_refuse_before_evaluation() {
    let first = admitted_declaration_key("Six");
    let second = admitted_declaration_key("Other");
    assert_ne!(first, second);
    let shape = record_shape();
    let types = TypeEnvironment::new(
        [
            CompositeDeclaration::new(first.clone(), "First", shape.clone()),
            CompositeDeclaration::new(second.clone(), "Second", shape),
        ],
        [],
    )
    .expect("distinct admitted declarations");
    let refusal = types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Composite(first)),
            EqualityOperand::typed(ValueType::Composite(second)),
            &EnumMemberIndex::default(),
        )
        .expect_err("distinct static composite types refuse");
    assert_eq!(refusal.cause, IllTypedCause::TypeMismatch);
}

/// Trace: IT-016-SC-03, TC-096, TC-026, FR-008-AC-5.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_composites_with_different_declarations_compare_unequal() {
    let first = admitted_declaration_key("Six");
    let second = admitted_declaration_key("Other");
    let types = TypeEnvironment::new(
        [
            CompositeDeclaration::new(first.clone(), "Six", record_shape()),
            CompositeDeclaration::new(second.clone(), "Other", record_shape()),
        ],
        [],
    )
    .expect("the two QSL-admitted record declarations are distinct");
    let fields = || {
        (0..6)
            .map(|index| {
                (
                    ["field0", "field1", "field2", "field3", "field4", "field5"][index],
                    FieldValue::Present(Value::Integer(Integer::from(index))),
                )
            })
            .collect()
    };
    let left = types.record(first, fields()).expect("first record");
    let right = types.record(second, fields()).expect("second record");
    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::planned_equality(&left, &right, &mut meter),
        Outcome::Completed(false)
    ));
    assert_eq!(
        admitted_points(&meter),
        [
            ChargePoint::EqualityPlanForm,
            ChargePoint::EqualityPlan,
            ChargePoint::EqualityPair,
            ChargePoint::EqualityResultRetain,
        ]
    );
}

/// Trace: IT-016-SC-03, TC-096, TC-026, FR-008-AC-5.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_enum_cases_compare_by_variant_identity() {
    let phase = admitted_declaration("Phase");
    let CheckedTerm::NominalBody(body) = &phase.body else {
        panic!("admitted Phase has an enum body: {:?}", phase.body);
    };
    let (ordered, cases) = body.enum_details().expect("Phase is an admitted enum");
    assert!(!ordered);
    assert_eq!(
        cases
            .iter()
            .map(|case| case.as_ref())
            .collect::<Vec<&str>>(),
        ["OPEN", "SHUT"]
    );
    let owner = admitted_declaration_key("Phase");
    let open = VariantId::new(owner.clone(), cases[0].to_string())
        .expect("admitted OPEN case is nonempty");
    let shut = VariantId::new(owner, cases[1].to_string()).expect("admitted SHUT case is nonempty");
    assert_ne!(open, shut);
    let first = Value::Enum(EnumMember::new(open.clone(), 0));
    let same_variant = Value::Enum(EnumMember::new(open, 1));
    let different_variant = Value::Enum(EnumMember::new(shut, 1));
    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::planned_equality(&first, &same_variant, &mut meter),
        Outcome::Completed(true)
    ));
    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::planned_equality(&first, &different_variant, &mut meter),
        Outcome::Completed(false)
    ));
}

/// Trace: IT-016-SC-03, TC-096, TC-025, TC-026, FR-008-AC-5, FR-008-AC-6.
#[test]
#[trace(
    "IT-016-SC-05",
    "IT-016-SC-03",
    "TC-096",
    "TC-025",
    "TC-026",
    "FR-008-AC-5",
    "FR-008-AC-6"
)]
fn admitted_model_roots_set_the_reference_universe_and_distinct_owner_result() {
    assert!(model_supertypes("Widget").is_empty());
    assert!(model_supertypes("Rock").is_empty());
    assert_eq!(
        model_supertypes("Gadget"),
        [format!("ix://{MODEL_IDENTITY}/Widget")]
    );
    let (widget_root, widget_type) = admitted_model_type("Widget");
    let (rock_root, _) = admitted_model_type("Rock");
    let (_, gadget_type) = admitted_model_type("Gadget");
    assert_ne!(widget_root, rock_root);
    assert_ne!(widget_type, gadget_type);
    let widget_universe = UniverseId::new(widget_root);
    let rock_universe = UniverseId::new(rock_root);
    let retained = reference(widget_universe.clone(), widget_type.clone());
    let same = reference(widget_universe.clone(), widget_type.clone());
    let distinct_owner = reference(widget_universe.clone(), gadget_type);
    // The operation supplies Rock's admitted universe for Widget's admitted
    // type. The malformed triple is made after both identities are admitted.
    let foreign = reference(rock_universe.clone(), widget_type.clone());

    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::planned_equality(&retained, &same, &mut meter),
        Outcome::Completed(true)
    ));
    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::planned_equality(&retained, &distinct_owner, &mut meter),
        Outcome::Completed(false)
    ));

    assert!(matches!(
        quire_exact::plan_equality(&retained, &foreign),
        Err(Refusal::ForeignReference { required, supplied })
            if required == widget_universe && supplied == rock_universe
    ));
    let collection_type =
        CollectionType::new(CollectionKind::Set, ValueType::Reference(widget_type), None);
    let mut meter = Meter::new(LIMITS);
    assert!(matches!(
        quire_exact::form_collection(&collection_type, vec![retained, foreign], &mut meter),
        Ok(Outcome::Refused(Refusal::ForeignReference { required, supplied }))
            if required == widget_universe && supplied == rock_universe
    ));
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-025", "TC-026", "FR-008-AC-5")]
fn admitted_membership_foreign_reference_names_retained_member_first() {
    let (widget_root, widget_type) = admitted_model_type("Widget");
    let (rock_root, _) = admitted_model_type("Rock");
    let retained_universe = UniverseId::new(widget_root);
    let candidate_universe = UniverseId::new(rock_root);
    let retained = reference(retained_universe.clone(), widget_type.clone());
    let candidate = reference(candidate_universe.clone(), widget_type);
    let mut charged = Meter::new(LIMITS);
    assert_eq!(
        quire_exact::member_equal(&candidate, &retained, &mut charged),
        Outcome::Refused(Refusal::ForeignReference {
            required: retained_universe,
            supplied: candidate_universe,
        })
    );
    assert_eq!(
        admitted_points(&charged),
        [ChargePoint::CollectionMemberWalk]
    );
}

/// Trace: IT-016-SC-03, TC-096, TC-203, FR-277-AC-5.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-203", "FR-277-AC-5")]
fn admitted_equality_plan_refuses_atomically_at_fifty_work_units() {
    let (types, checked, key) = checked_equality();
    let value = operand(&types, key, 0);
    let mut meter = Meter::new(ScalarLimits {
        work_units: 50,
        ..LIMITS
    });
    let Outcome::Incomplete(refusal) = checked.evaluate(&types, &value, &value, &mut meter) else {
        panic!("the plan reservation must be denied");
    };
    assert_eq!(refusal.charge_point, ChargePoint::EqualityPlan);
    assert_eq!(refusal.limit_kind, LimitKind::WorkUnits);
    assert_eq!(refusal.limit, 50);
    assert_eq!(refusal.consumed, 32);
    assert_eq!(refusal.next_charge, Integer::from(19_u64));
    assert_eq!(admitted_points(&meter), [ChargePoint::EqualityPlanForm]);
}

/// Trace: IT-016-SC-03, TC-096, TC-203, FR-277-AC-6.
#[test]
#[trace("IT-016-SC-05", "IT-016-SC-03", "TC-096", "TC-203", "FR-277-AC-6")]
fn admitted_equality_plan_checks_result_capacity_before_pair_charges() {
    let (types, checked, key) = checked_equality();
    let value = operand(&types, key, 0);
    let mut meter = Meter::new(ScalarLimits {
        result_units: 0,
        ..LIMITS
    });
    let Outcome::Incomplete(refusal) = checked.evaluate(&types, &value, &value, &mut meter) else {
        panic!("the result reservation must be denied");
    };
    assert_eq!(refusal.charge_point, ChargePoint::EqualityPlan);
    assert_eq!(refusal.limit_kind, LimitKind::ResultUnits);
    assert_eq!(refusal.limit, 0);
    assert_eq!(refusal.consumed, 0);
    assert_eq!(refusal.next_charge, Integer::one());
    assert_eq!(admitted_points(&meter), [ChargePoint::EqualityPlanForm]);
}
