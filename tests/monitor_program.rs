// SPDX-License-Identifier: AGPL-3.0-or-later
//! RT construction controls using identities from a live QSL producer and strict IR reader.
//! The controlled executable parts exercise construction, not temporal producer/executor qualification.

use ix_trace_rs::trace;
use quire_contract_runtime::monitor::{
    Entries, Expression, ExpressionId, Function, FunctionId, Layout, Operation, Program,
    ProgramParts, ProgramRefusalCode, TypeBinding,
};
use quire_exact::{
    CollectionKind, CollectionType, FieldValue, Identifier, Integer, NodeKey, Presence, Rational,
    Value, ValueType,
};
use quire_semantic_value::{
    declaration::{
        CompositeDeclaration, CompositeShape, FieldDeclaration, TypeEnvironment,
        UnionMemberDeclaration,
    },
    quantity::UnitTable,
    unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode},
};
use std::{collections::BTreeMap, rc::Rc};

#[path = "support/producer_origin.rs"]
mod producer_origin_support;
mod support;

fn key(name: &str) -> NodeKey {
    static KEYS: std::sync::OnceLock<BTreeMap<String, NodeKey>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        let package = producer_origin_support::emit_and_read_checked_v2(
            producer_origin_support::CheckedProducerRequest {
                source_path: "monitor-construction.native",
                source: SOURCE,
                package_name: "agent-ix/monitor-construction",
                models: &[],
                execution_points: None,
            },
        );
        IDENTITIES
            .iter()
            .map(|(label, declaration)| {
                let nodes: Vec<_> = package
                    .graph()
                    .nodes
                    .iter()
                    .filter(|node| {
                        matches!(&node.owner, Some(quire_contract_ir::CheckedNodeOwner::Source {authority, identity}) if authority.as_ref() == "local" && identity.as_ref() == "source:monitor-construction.native") && node.declaration.as_ref().is_some_and(|value| {
                            value
                                .qualified_name
                                .last()
                                .is_some_and(|part| part.as_ref() == *declaration)
                        })
                    })
                    .collect();
                let [node] = nodes.as_slice() else {
                    panic!("one admitted declaration {declaration}, found {}; candidates {:?}", nodes.len(), package.graph().nodes.iter().filter(|n| n.declaration.as_ref().is_some_and(|d| d.qualified_name.last().is_some_and(|p| p.as_ref() == *declaration))).map(|n| &n.owner).collect::<Vec<_>>());
                };
                let wire =
                    serde_json::to_value(&node.node_id).expect("admitted identity serializes");
                let key = qsl_foundation::node_wire::node_key_from_json(&wire)
                    .expect("carry admitted identity");
                ((*label).to_owned(), key)
            })
            .collect()
    })
    .get(name)
    .unwrap_or_else(|| panic!("missing source identity {name}"))
    .clone()
}

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
dimension Time;
unit root_tick : Time = rational(1, 1);
unit tick : Time = rational(1, 1) * root_tick;
enum Decl0 { CASE0 }
enum Decl1 { CASE1 }
enum Decl2 { CASE2 }
enum Decl3 { CASE3 }
enum Decl4 { CASE4 }
enum Decl5 { CASE5 }
enum Decl6 { CASE6 }
enum Decl8 { CASE8 }
enum Decl9 { CASE9 }
enum Decl10 { CASE10 }
enum Decl11 { CASE11 }
enum Decl12 { CASE12 }
enum Decl13 { CASE13 }
enum Decl14 { CASE14 }
enum Decl15 { CASE15 }
enum Decl16 { CASE16 }
enum Decl17 { CASE17 }
enum Decl18 { CASE18 }
enum Decl19 { CASE19 }
enum Decl20 { CASE20 }
enum Decl21 { CASE21 }
enum Decl22 { CASE22 }
enum Decl23 { CASE23 }
enum Decl24 { CASE24 }
enum Decl25 { CASE25 }
enum Decl26 { CASE26 }
enum Decl27 { CASE27 }
enum Decl28 { CASE28 }
enum Decl29 { CASE29 }
enum Decl30 { CASE30 }
enum Decl31 { CASE31 }
enum Decl32 { CASE32 }
enum Decl33 { CASE33 }
enum Decl34 { CASE34 }
enum Decl35 { CASE35 }
enum Decl36 { CASE36 }
enum Decl37 { CASE37 }
enum Decl38 { CASE38 }
enum Decl40 { CASE40 }
enum Decl41 { CASE41 }
enum Decl42 { CASE42 }
enum Decl43 { CASE43 }
enum Decl44 { CASE44 }
enum Decl45 { CASE45 }
enum Decl46 { CASE46 }
enum Decl47 { CASE47 }
enum Decl48 { CASE48 }
enum Decl50 { CASE50 }
enum Decl51 { CASE51 }
enum Decl52 { CASE52 }
enum Decl53 { CASE53 }

function touch_tick using v(x: Quantity<tick>): Quantity<tick> pure { x }
function touch_root using v(x: Quantity<root_tick>): Quantity<root_tick> pure { x }
function touch_Decl0 using v(x: Decl0): Decl0 pure { x }
function touch_Decl1 using v(x: Decl1): Decl1 pure { x }
function touch_Decl2 using v(x: Decl2): Decl2 pure { x }
function touch_Decl3 using v(x: Decl3): Decl3 pure { x }
function touch_Decl4 using v(x: Decl4): Decl4 pure { x }
function touch_Decl5 using v(x: Decl5): Decl5 pure { x }
function touch_Decl6 using v(x: Decl6): Decl6 pure { x }
function touch_Decl8 using v(x: Decl8): Decl8 pure { x }
function touch_Decl9 using v(x: Decl9): Decl9 pure { x }
function touch_Decl10 using v(x: Decl10): Decl10 pure { x }
function touch_Decl11 using v(x: Decl11): Decl11 pure { x }
function touch_Decl12 using v(x: Decl12): Decl12 pure { x }
function touch_Decl13 using v(x: Decl13): Decl13 pure { x }
function touch_Decl14 using v(x: Decl14): Decl14 pure { x }
function touch_Decl15 using v(x: Decl15): Decl15 pure { x }
function touch_Decl16 using v(x: Decl16): Decl16 pure { x }
function touch_Decl17 using v(x: Decl17): Decl17 pure { x }
function touch_Decl18 using v(x: Decl18): Decl18 pure { x }
function touch_Decl19 using v(x: Decl19): Decl19 pure { x }
function touch_Decl20 using v(x: Decl20): Decl20 pure { x }
function touch_Decl21 using v(x: Decl21): Decl21 pure { x }
function touch_Decl22 using v(x: Decl22): Decl22 pure { x }
function touch_Decl23 using v(x: Decl23): Decl23 pure { x }
function touch_Decl24 using v(x: Decl24): Decl24 pure { x }
function touch_Decl25 using v(x: Decl25): Decl25 pure { x }
function touch_Decl26 using v(x: Decl26): Decl26 pure { x }
function touch_Decl27 using v(x: Decl27): Decl27 pure { x }
function touch_Decl28 using v(x: Decl28): Decl28 pure { x }
function touch_Decl29 using v(x: Decl29): Decl29 pure { x }
function touch_Decl30 using v(x: Decl30): Decl30 pure { x }
function touch_Decl31 using v(x: Decl31): Decl31 pure { x }
function touch_Decl32 using v(x: Decl32): Decl32 pure { x }
function touch_Decl33 using v(x: Decl33): Decl33 pure { x }
function touch_Decl34 using v(x: Decl34): Decl34 pure { x }
function touch_Decl35 using v(x: Decl35): Decl35 pure { x }
function touch_Decl36 using v(x: Decl36): Decl36 pure { x }
function touch_Decl37 using v(x: Decl37): Decl37 pure { x }
function touch_Decl38 using v(x: Decl38): Decl38 pure { x }
function touch_Decl40 using v(x: Decl40): Decl40 pure { x }
function touch_Decl41 using v(x: Decl41): Decl41 pure { x }
function touch_Decl42 using v(x: Decl42): Decl42 pure { x }
function touch_Decl43 using v(x: Decl43): Decl43 pure { x }
function touch_Decl44 using v(x: Decl44): Decl44 pure { x }
function touch_Decl45 using v(x: Decl45): Decl45 pure { x }
function touch_Decl46 using v(x: Decl46): Decl46 pure { x }
function touch_Decl47 using v(x: Decl47): Decl47 pure { x }
function touch_Decl48 using v(x: Decl48): Decl48 pure { x }
function touch_Decl50 using v(x: Decl50): Decl50 pure { x }
function touch_Decl51 using v(x: Decl51): Decl51 pure { x }
function touch_Decl52 using v(x: Decl52): Decl52 pure { x }
function touch_Decl53 using v(x: Decl53): Decl53 pure { x }
"#;
const IDENTITIES: &[(&str, &str)] = &[
    ("Failed", "Decl0"),
    ("False", "Decl1"),
    ("Incomplete", "Decl2"),
    ("Indeterminate", "Decl3"),
    ("Length", "Decl4"),
    ("Pending", "Decl5"),
    ("TC-209", "Decl6"),
    ("Time", "Time"),
    ("True", "Decl8"),
    ("activation", "Decl9"),
    ("activations", "Decl10"),
    ("bad-if", "Decl11"),
    ("clause", "Decl12"),
    ("condition", "Decl13"),
    ("counter-a", "Decl14"),
    ("event", "Decl15"),
    ("exact", "Decl16"),
    ("foreign-argument", "Decl17"),
    ("gap", "Decl18"),
    ("gap_payload", "Decl19"),
    ("history_start", "Decl20"),
    ("init", "Decl21"),
    ("inputs", "Decl22"),
    ("instance", "Decl23"),
    ("interface-002-AC-1", "Decl24"),
    ("interface-002-AC-2", "Decl25"),
    ("interface-002-AC-3", "Decl26"),
    ("interface-002-AC-4", "Decl27"),
    ("limit", "Decl28"),
    ("limit_payload", "Decl29"),
    ("max_reading_gap", "Decl30"),
    ("monitor-test", "Decl31"),
    ("monitor_plan", "Decl32"),
    ("now", "Decl33"),
    ("occurrence", "Decl34"),
    ("origin", "Decl35"),
    ("outcome", "Decl36"),
    ("result", "Decl37"),
    ("results", "Decl38"),
    ("root-tick", "root_tick"),
    ("scope", "Decl40"),
    ("selected-lock", "Decl41"),
    ("selected-subject", "Decl42"),
    ("setting", "Decl43"),
    ("state", "Decl44"),
    ("status", "Decl45"),
    ("step", "Decl46"),
    ("subformula", "Decl47"),
    ("subject", "Decl48"),
    ("tick", "tick"),
    ("timestamped-event", "Decl50"),
    ("unreached-cycle", "Decl51"),
    ("value", "Decl52"),
    ("watermark", "Decl53"),
];

fn record(name: &str, fields: Vec<(&str, ValueType)>) -> CompositeDeclaration {
    CompositeDeclaration::new(
        key(name),
        name,
        CompositeShape::Record(
            fields
                .into_iter()
                .map(|(name, ty)| FieldDeclaration::new(name, ty, Presence::Required))
                .collect(),
        ),
    )
}
fn parts() -> ProgramParts {
    let state = ValueType::Composite(key("state"));
    let inputs = ValueType::Composite(key("inputs"));
    let scope = ValueType::Composite(key("scope"));
    let results_type = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key("result")),
        None,
    );
    let results = ValueType::collection(results_type.clone());
    let step_result = ValueType::Composite(key("step"));
    let activation_type = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key("activation")),
        None,
    );
    let activations = ValueType::collection(activation_type.clone());
    let types = TypeEnvironment::new(
        [
            record(
                "state",
                vec![
                    ("now", ValueType::Integer),
                    ("history_start", ValueType::Integer),
                    ("activations", activations),
                ],
            ),
            record(
                "activation",
                vec![
                    ("subformula", ValueType::Integer),
                    ("origin", ValueType::Integer),
                    ("status", ValueType::Boolean),
                ],
            ),
            record("inputs", vec![]),
            record("scope", vec![]),
            record(
                "gap_payload",
                vec![
                    ("gap", ValueType::Integer),
                    ("max_reading_gap", ValueType::Integer),
                ],
            ),
            record(
                "limit_payload",
                vec![
                    ("limit", ValueType::String),
                    ("value", ValueType::Integer),
                    ("setting", ValueType::String),
                ],
            ),
            record(
                "instance",
                vec![
                    ("subject", ValueType::String),
                    ("subformula", ValueType::Integer),
                    ("origin", ValueType::Integer),
                ],
            ),
            CompositeDeclaration::new(
                key("outcome"),
                "outcome",
                CompositeShape::Union(
                    [
                        ("Failed", vec![ValueType::Composite(key("gap_payload"))]),
                        ("False", vec![]),
                        (
                            "Incomplete",
                            vec![ValueType::Composite(key("limit_payload"))],
                        ),
                        ("Indeterminate", vec![]),
                        ("Pending", vec![]),
                        ("True", vec![]),
                    ]
                    .into_iter()
                    .map(|(name, positions)| {
                        UnionMemberDeclaration::resolved(Identifier::new(name).unwrap(), positions)
                    })
                    .collect(),
                ),
            ),
            record(
                "result",
                vec![
                    ("instance", ValueType::Composite(key("instance"))),
                    ("outcome", ValueType::Composite(key("outcome"))),
                ],
            ),
            CompositeDeclaration::new(
                key("step"),
                "step",
                CompositeShape::Tuple(vec![state.clone(), results.clone()]),
            ),
        ],
        [],
    )
    .expect("admit diagnostic nominal declarations");
    let zero = Rational::from_integer(Integer::zero());
    let one = Rational::from_integer(Integer::one());
    let dimensions = BTreeMap::from([(
        key("Time"),
        DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
    )]);
    let units = BTreeMap::from([(
        key("tick"),
        UnitNode::checked(
            key("Time"),
            None,
            one.clone(),
            zero.clone(),
            NominalDeclaration::default(),
        )
        .unwrap(),
    )]);
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
    let types = types.with_units(UnitTable::declared(&graph));
    let state_value = types
        .record(
            key("state"),
            vec![
                ("now", FieldValue::Present(Value::Integer(Integer::zero()))),
                (
                    "history_start",
                    FieldValue::Present(Value::Integer(Integer::zero())),
                ),
                (
                    "activations",
                    FieldValue::Present(quire_exact::from_admitted(activation_type, vec![])),
                ),
            ],
        )
        .unwrap();
    let result_value = quire_exact::from_admitted(results_type, vec![]);
    let step_value = types
        .tuple(key("step"), vec![state_value.clone(), result_value])
        .unwrap();
    let function_keys = [key("init"), key("event"), key("watermark"), key("gap")];
    let signatures = [
        vec![],
        vec![state.clone(), ValueType::Integer, inputs.clone()],
        vec![state.clone(), ValueType::Integer, scope.clone()],
        vec![state.clone(), ValueType::Integer],
    ];
    let mut expressions = Vec::new();
    let mut functions = Vec::new();
    for (index, (source, parameters)) in function_keys.into_iter().zip(signatures).enumerate() {
        let (value_type, value) = if index == 0 {
            (state.clone(), state_value.clone())
        } else {
            (step_result.clone(), step_value.clone())
        };
        expressions.push(Expression {
            key: source.clone(),
            body_path: vec![0],
            value_type: value_type.clone(),
            operation: Operation::Literal(value),
        });
        functions.push(Function {
            key: source,
            parameters,
            result: value_type,
            body: ExpressionId(index),
        });
    }
    let binding = |name, value_type| TypeBinding {
        key: key(name),
        value_type,
    };
    ProgramParts {
        layout: Layout {
            root: key("monitor_plan"),
            clause: key("clause"),
            package_lock: Box::from(&b"selected-lock"[..]),
            subject: "selected-subject".into(),
            occurrence: Box::from(&b"occurrence"[..]),
            profile: "timestamped-event".into(),
            state: binding("state", state),
            inputs: binding("inputs", inputs),
            scope: binding("scope", scope),
            results: binding("results", results),
            step_result: binding("step", step_result),
            entries: Entries {
                init: FunctionId(0),
                on_event: FunctionId(1),
                on_watermark: FunctionId(2),
                on_gap: FunctionId(3),
            },
            counter: "counter-a".into(),
            tick_unit: key("tick"),
            time_dimension: key("Time"),
            counter_width_bits: 16,
            max_reading_gap: 100,
            max_event_rate: one,
            uncertainty: zero,
            buffers: vec![],
            activation_capacity: Integer::zero(),
        },
        types,
        functions,
        expressions,
    }
}
fn refused(parts: ProgramParts, expected: ProgramRefusalCode) {
    match Program::try_new(parts) {
        Err(error) => assert_eq!(error.code, expected),
        Ok(_) => panic!("invalid construction must refuse"),
    }
}

fn assert_state_contents(value: &Value) {
    let Value::Composite(state) = value else {
        panic!("retained State record");
    };
    assert_eq!(state.declaration(), &key("state"));
    let [FieldValue::Present(Value::Integer(now)), FieldValue::Present(Value::Integer(history_start)), FieldValue::Present(Value::Collection(activations))] =
        state.slots()
    else {
        panic!("now/history_start/activations state contents");
    };
    assert_eq!(now, &Integer::zero());
    assert_eq!(history_start, &Integer::zero());
    assert_eq!(
        activations.collection_type().kind(),
        CollectionKind::Sequence
    );
    assert_eq!(
        activations.collection_type().element(),
        &ValueType::Composite(key("activation"))
    );
    assert!(activations.elements().is_empty());
}

#[trace("TC-098", "IT-018-SC-01", "interface-002-AC-1")]
#[test]
fn retained_program_outlives_preparation_and_keeps_four_entry_keys() {
    let program = {
        let input = parts();
        Program::try_new(input).unwrap()
    };
    let second = Rc::clone(&program);
    drop(program);
    assert_eq!(second.layout().root, key("monitor_plan"));
    assert!(second.function(FunctionId(100)).is_none());
    assert!(second.expression(ExpressionId(100)).is_none());
    assert_eq!(second.layout().package_lock.as_ref(), b"selected-lock");
    assert_eq!(second.layout().occurrence.as_ref(), b"occurrence");
    assert_eq!(
        second.layout().entries,
        Entries {
            init: FunctionId(0),
            on_event: FunctionId(1),
            on_watermark: FunctionId(2),
            on_gap: FunctionId(3)
        }
    );
    for (entry, name, parameters) in [
        (second.layout().entries.init, "init", vec![]),
        (
            second.layout().entries.on_event,
            "event",
            vec![
                ValueType::Composite(key("state")),
                ValueType::Integer,
                ValueType::Composite(key("inputs")),
            ],
        ),
        (
            second.layout().entries.on_watermark,
            "watermark",
            vec![
                ValueType::Composite(key("state")),
                ValueType::Integer,
                ValueType::Composite(key("scope")),
            ],
        ),
        (
            second.layout().entries.on_gap,
            "gap",
            vec![ValueType::Composite(key("state")), ValueType::Integer],
        ),
    ] {
        let function = second.function(entry).unwrap();
        assert_eq!(function.key, key(name));
        assert_eq!(function.parameters, parameters);
        assert_eq!(function.body, ExpressionId(entry.0));
        let body = second.expression(function.body).unwrap();
        assert_eq!(body.key, key(name));
        assert_eq!(body.body_path, [0]);
        let Operation::Literal(value) = &body.operation else {
            panic!("retained literal body");
        };
        if name == "init" {
            assert_eq!(function.result, ValueType::Composite(key("state")));
            assert_state_contents(value);
        } else {
            assert_eq!(function.result, ValueType::Composite(key("step")));
            let Value::Composite(step) = value else {
                panic!("retained tuple");
            };
            assert_eq!(step.declaration(), &key("step"));
            let [FieldValue::Present(state), FieldValue::Present(Value::Collection(results))] =
                step.slots()
            else {
                panic!("State/Results tuple contents");
            };
            assert_state_contents(state);
            assert_eq!(results.collection_type().kind(), CollectionKind::Sequence);
            assert_eq!(
                results.collection_type().element(),
                &ValueType::Composite(key("result"))
            );
            assert!(results.elements().is_empty());
        }
        assert_eq!(body.value_type, function.result);
    }
    assert!(second.types().composite(key("state")).is_some());
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn missing_and_mistyped_fourth_entry_refuse_before_any_execution() {
    let mut missing = parts();
    missing.layout.entries.on_gap = FunctionId(100);
    refused(missing, ProgramRefusalCode::InvalidLayout);
    let mut wrong = parts();
    wrong.functions[3].parameters.pop();
    refused(wrong, ProgramRefusalCode::TypeMismatch);
}

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-3")]
#[test]
fn duplicate_source_keys_invalid_edges_and_cycles_refuse() {
    let mut duplicate = parts();
    duplicate.functions[1].key = duplicate.functions[0].key.clone();
    refused(duplicate, ProgramRefusalCode::DuplicateKey);
    let mut invalid = parts();
    invalid.functions[0].body = ExpressionId(100);
    refused(invalid, ProgramRefusalCode::InvalidReference);
    let mut cycle = parts();
    cycle.expressions[0].operation = Operation::If {
        condition: ExpressionId(0),
        when_true: ExpressionId(0),
        when_false: ExpressionId(0),
    };
    // Type mismatch at the condition has precedence over graph traversal.
    refused(cycle, ProgramRefusalCode::TypeMismatch);
    let mut cycle = parts();
    cycle.expressions.push(Expression {
        key: key("unreached-cycle"),
        body_path: vec![],
        value_type: ValueType::Boolean,
        operation: Operation::If {
            condition: ExpressionId(4),
            when_true: ExpressionId(4),
            when_false: ExpressionId(4),
        },
    });
    refused(cycle, ProgramRefusalCode::ExpressionCycle);
}

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-3")]
#[test]
fn argument_ownership_is_checked_through_the_expression_closure() {
    let mut wrong = parts();
    wrong.expressions[0] = Expression {
        key: key("foreign-argument"),
        body_path: vec![],
        value_type: wrong.layout.state.value_type.clone(),
        operation: Operation::Argument {
            function: FunctionId(1),
            position: 0,
        },
    };
    refused(wrong, ProgramRefusalCode::ForeignArgument);
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn counter_premises_keep_width_64_and_reject_out_of_width_gap() {
    let mut full_width = parts();
    full_width.layout.counter_width_bits = 64;
    full_width.layout.max_reading_gap = u64::MAX;
    assert!(Program::try_new(full_width).is_ok());
    for width in [0, 65] {
        let mut wrong = parts();
        wrong.layout.counter_width_bits = width;
        refused(wrong, ProgramRefusalCode::InvalidLayout);
    }
    let mut wrong = parts();
    wrong.layout.max_reading_gap = 65536;
    refused(wrong, ProgramRefusalCode::InvalidLayout);
    let mut wrong = parts();
    wrong.layout.uncertainty = Rational::from_integer(Integer::from(-1_i64));
    refused(wrong, ProgramRefusalCode::InvalidLayout);
    let mut wrong = parts();
    wrong.layout.time_dimension = key("Length");
    refused(wrong, ProgramRefusalCode::InvalidLayout);
}

#[trace("TC-098", "IT-018-SC-04", "interface-002-AC-4")]
#[test]
fn compatible_entry_roles_may_share_one_function() {
    let mut input = parts();
    input.layout.scope = input.layout.inputs.clone();
    input.layout.entries.on_watermark = input.layout.entries.on_event;
    let program = Program::try_new(input).unwrap();
    assert_eq!(
        program.layout().entries.on_event,
        program.layout().entries.on_watermark
    );
}

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-3")]
#[test]
fn shared_expression_nodes_and_distinct_inline_paths_are_not_cycles_or_duplicate_keys() {
    let mut input = parts();
    let original = input.expressions[0].key.clone();
    input.expressions.push(Expression {
        key: original.clone(),
        body_path: vec![1],
        value_type: ValueType::Boolean,
        operation: Operation::Literal(Value::Boolean(true)),
    });
    input.expressions.push(Expression {
        key: original.clone(),
        body_path: vec![2],
        value_type: input.layout.state.value_type.clone(),
        operation: Operation::If {
            condition: ExpressionId(4),
            when_true: ExpressionId(0),
            when_false: ExpressionId(0),
        },
    });
    input.functions[0].body = ExpressionId(5);
    let program = Program::try_new(input).unwrap();
    assert_eq!(program.expression(ExpressionId(0)).unwrap().key, original);
    assert_eq!(program.expression(ExpressionId(5)).unwrap().body_path, [2]);
}

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-3")]
#[test]
fn deeply_shared_inline_arena_validates_on_a_small_native_stack() {
    let _ = key("init");
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut input = parts();
            input.expressions.push(Expression {
                key: key("condition"),
                body_path: vec![],
                value_type: ValueType::Boolean,
                operation: Operation::Literal(Value::Boolean(true)),
            });
            let mut previous = ExpressionId(0);
            for position in 0..20_000 {
                let next = ExpressionId(input.expressions.len());
                input.expressions.push(Expression {
                    key: key("init"),
                    body_path: vec![position + 1],
                    value_type: input.layout.state.value_type.clone(),
                    operation: Operation::If {
                        condition: ExpressionId(4),
                        when_true: previous,
                        when_false: previous,
                    },
                });
                previous = next;
            }
            input.functions[0].body = previous;
            let program = Program::try_new(input).unwrap();
            assert_eq!(program.function(FunctionId(0)).unwrap().body, previous);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn exact_activation_capacity_includes_zero_and_values_above_host_width() {
    let zero = Program::try_new(parts()).unwrap();
    assert_eq!(zero.layout().activation_capacity, Integer::zero());
    let mut large = parts();
    let capacity = Integer::from(u64::MAX).add(&Integer::one());
    large.layout.activation_capacity = capacity.clone();
    let retained = Program::try_new(large).unwrap();
    assert_eq!(retained.layout().activation_capacity, capacity);
    let mut invalid = parts();
    invalid.layout.activation_capacity = Integer::from(-1_i64);
    refused(invalid, ProgramRefusalCode::InvalidLayout);
}

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-3")]
#[test]
fn duplicate_expression_locations_and_each_call_or_if_child_refuse() {
    let mut duplicate = parts();
    duplicate.expressions[1].key = duplicate.expressions[0].key.clone();
    duplicate.expressions[1].body_path = duplicate.expressions[0].body_path.clone();
    refused(duplicate, ProgramRefusalCode::DuplicateKey);
    for (function, arguments) in [
        (FunctionId(100), vec![]),
        (FunctionId(3), vec![ExpressionId(0), ExpressionId(100)]),
    ] {
        let mut invalid = parts();
        invalid.expressions[1].operation = Operation::Call {
            function,
            arguments,
        };
        refused(invalid, ProgramRefusalCode::InvalidReference);
    }
    for (condition, when_true, when_false) in [
        (ExpressionId(100), ExpressionId(1), ExpressionId(1)),
        (ExpressionId(4), ExpressionId(100), ExpressionId(1)),
        (ExpressionId(4), ExpressionId(1), ExpressionId(100)),
    ] {
        let mut invalid = parts();
        invalid.expressions.push(Expression {
            key: key("condition"),
            body_path: vec![],
            value_type: ValueType::Boolean,
            operation: Operation::Literal(Value::Boolean(true)),
        });
        invalid.expressions.push(Expression {
            key: key("bad-if"),
            body_path: vec![],
            value_type: invalid.layout.step_result.value_type.clone(),
            operation: Operation::If {
                condition,
                when_true,
                when_false,
            },
        });
        refused(invalid, ProgramRefusalCode::InvalidReference);
    }
}
#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn declared_time_units_retain_nonzero_scales_and_affine_offsets() {
    for (scale, offset) in [(-1_i64, 0_i64), (1, 7)] {
        let mut input = parts();
        let dimensions = BTreeMap::from([(
            key("Time"),
            DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
        )]);
        let units = BTreeMap::from([
            (
                key("tick"),
                UnitNode::checked(
                    key("Time"),
                    Some(key("root-tick")),
                    Rational::from_integer(Integer::from(scale)),
                    Rational::from_integer(Integer::from(offset)),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
            (
                key("root-tick"),
                UnitNode::checked(
                    key("Time"),
                    None,
                    Rational::from_integer(Integer::one()),
                    Rational::from_integer(Integer::zero()),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
        ]);
        let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
        input.types = input.types.with_units(UnitTable::declared(&graph));
        let program =
            Program::try_new(input).expect("retain admitted Time unit without extra restrictions");
        let retained = program
            .types()
            .units()
            .get(quire_exact::UnitId::declared(key("tick")))
            .unwrap();
        let quire_semantic_value::quantity::QuantityUnit::Declared(retained) = retained else {
            panic!("declared unit was replaced");
        };
        assert_eq!(retained.dimension_node(), key("Time"));
        assert_eq!(
            retained.canonical().scale(),
            &Rational::from_integer(Integer::from(scale))
        );
        assert_eq!(retained.is_affine(), offset != 0);
    }
}
