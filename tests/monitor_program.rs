// SPDX-License-Identifier: AGPL-3.0-or-later
//! RT construction controls using identities from a live QSL producer and strict IR reader.
//! The controlled executable parts exercise construction, not temporal producer/executor qualification.

use ix_trace_rs::trace;
use quire_contract_runtime::monitor::{
    build, Buffer, BuildPlan, Entries, Expression, ExpressionId, Function, FunctionId, Layout,
    LocalId, MonitorBuildFailureCode, Operation, Program, ProgramParts, ProgramRefusalCode,
    StepBudget, StepResource, TypeBinding,
};
use quire_exact::{
    CollectionKind, CollectionType, FieldValue, Identifier, Integer, IntegerInterval, MemberId,
    MemberOwner, NodeKey, Presence, Rational, TextProfile, TextType, Value, ValueType,
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

fn admitted() -> &'static quire_contract_ir::CheckedPackageV2 {
    static PACKAGE: std::sync::OnceLock<quire_contract_ir::CheckedPackageV2> =
        std::sync::OnceLock::new();
    PACKAGE.get_or_init(|| {
        producer_origin_support::emit_and_read_checked_v2(
            producer_origin_support::CheckedProducerRequest {
                source_path: "monitor-construction.native",
                source: SOURCE,
                package_name: "agent-ix/monitor-construction",
                models: &[],
                execution_points: None,
            },
        )
    })
}

fn key(name: &str) -> NodeKey {
    static KEYS: std::sync::OnceLock<BTreeMap<String, NodeKey>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        let package = admitted();
        let mut keys: BTreeMap<String, NodeKey> = IDENTITIES.iter().map(|(label, declaration)| {
            let nodes: Vec<_> = package.graph().nodes.iter().filter(|node| {
                matches!(&node.owner, Some(quire_contract_ir::CheckedNodeOwner::Source {authority, identity}) if authority.as_ref() == "local" && identity.as_ref() == "source:monitor-construction.native")
                    && node.declaration.as_ref().is_some_and(|value| value.qualified_name.last().is_some_and(|part| part.as_ref() == *declaration))
            }).collect();
            let [node] = nodes.as_slice() else { panic!("one admitted declaration {declaration}, found {}", nodes.len()); };
            let expected_form = match *label {
                "Time" | "Length" => "dimension",
                "root-tick" | "tick" | "negative-tick" | "affine-tick" => "unit",
                "outcome" => "union",
                "state-pair-child" | "integer-pair" | "bounded-integer-pair" | "state-pair" | "bool-pair" | "step" | "step-buffered" | "step-missing-position" => "tuple",
                "init-missing-position" | "event-missing-position" | "watermark-missing-position" | "gap-missing-position" | "init-buffered" | "event-buffered" | "watermark-buffered" | "gap-buffered" | "record-init" | "tuple-init" | "boolean-init" | "lexical-init" | "lexical-helper" | "pass-second" | "pass-state" | "option-probe" | "init" | "event" | "watermark" | "gap" | "condition" | "unreached-cycle" | "foreign-argument" | "bad-if" | "clause" => "pure_function",
                _ => "record",
            };
            assert_eq!(node.semantic_form.as_ref(), expected_form, "retain declaration meaning for {declaration}");
            let wire = serde_json::to_value(&node.node_id).expect("admitted identity serializes");
            let key = qsl_foundation::node_wire::node_key_from_json(&wire).expect("carry admitted identity");
            ((*label).to_owned(), key)
        }).collect();
        let step_node = package.graph().nodes.iter().find(|node| {
            qsl_foundation::node_wire::node_key_from_json(&serde_json::to_value(&node.node_id).unwrap()).unwrap() == keys["step"]
        }).expect("Step declaration retains its admitted tuple body");
        let body = serde_json::to_value(&step_node.body).unwrap();
        assert_eq!(body["term"], "aggregate");
        let results = qsl_foundation::node_wire::node_key_from_json(&body["members"][1]["target"]).expect("Step's second position is the admitted result sequence");
        assert!(results.path().iter().any(|part| matches!(part, quire_exact::PathSegment::Node(_))));
        keys.insert("results".to_owned(), results.clone());
        // This diagnostic layout key tests storage, not temporal-plan admission.
        keys.insert("monitor_plan".to_owned(), results);
        keys
    }).get(name).unwrap_or_else(|| panic!("missing source identity {name}")).clone()
}

fn admitted_time_units() -> UnitTable {
    let dimensions = BTreeMap::from([(
        key("Time"),
        DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
    )]);
    let units = ["root-tick", "tick", "negative-tick", "affine-tick"]
        .into_iter()
        .map(|label| {
            let unit_key = key(label);
            let node = admitted()
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    qsl_foundation::node_wire::node_key_from_json(
                        &serde_json::to_value(&node.node_id).unwrap(),
                    )
                    .unwrap()
                        == unit_key
                })
                .expect("unit owner remains in admitted graph");
            let body = serde_json::to_value(&node.body).unwrap();
            assert_eq!(body["term"], "unit");
            let carry = |name: &str| {
                qsl_foundation::node_wire::node_key_from_json(&body[name])
                    .expect("admitted unit reference")
            };
            let ratio = |name: &str| {
                let part = |field: &str| {
                    body[name][field]
                        .as_str()
                        .unwrap()
                        .parse::<Integer>()
                        .unwrap()
                };
                Rational::new(part("numerator"), part("denominator")).unwrap()
            };
            let target =
                (!body["target_unit_node_id"].is_null()).then(|| carry("target_unit_node_id"));
            let unit = UnitNode::checked(
                carry("dimension_node_id"),
                target,
                ratio("scale"),
                ratio("offset"),
                NominalDeclaration::default(),
            )
            .unwrap();
            (unit_key, unit)
        })
        .collect();
    UnitTable::declared(&UnitGraph::from_checked_nodes(&dimensions, &units).unwrap())
}

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
dimension Time;
unit root_tick : Time = rational(1, 1);
unit tick : Time = rational(1, 1) * root_tick;
unit negative_tick : Time = rational(-1, 1) * root_tick;
unit affine_tick : Time = rational(1, 1) * root_tick + rational(7, 1);
dimension Length;
unit metre : Length = rational(1, 1);
record Activation { subformula: Integer; born: Integer; status: Boolean; }
record State { now: Integer; history_start: Integer; activations: Sequence<Activation>; event_position: Integer; }
record StateBuffered { now: Integer; history_start: Integer; buffer_0: Sequence<Activation>; buffer_1: Sequence<Activation>; activations: Sequence<Activation>; event_position: Integer; }
record StateMissingPosition { now: Integer; history_start: Integer; activations: Sequence<Activation>; }
record StateTextPosition { now: Integer; history_start: Integer; activations: Sequence<Activation>; event_position: Text[0, 8; nfc]; }
record StateBooleanPosition { now: Integer; history_start: Integer; activations: Sequence<Activation>; event_position: Boolean; }
record StateMisorderedPosition { now: Integer; history_start: Integer; event_position: Integer; activations: Sequence<Activation>; }
record StateExtraField { now: Integer; history_start: Integer; activations: Sequence<Activation>; event_position: Integer; spare: Boolean; }
record StateOptionalPosition { now: Integer; history_start: Integer; activations: Sequence<Activation>; event_position: Integer?; }
record StateBadBufferName { now: Integer; history_start: Integer; buffer_00: Sequence<Activation>; activations: Sequence<Activation>; event_position: Integer; }
record StateBadBufferType { now: Integer; history_start: Integer; buffer_0: Integer; activations: Sequence<Activation>; event_position: Integer; }
record Inputs { enabled: Boolean; }
record Scope { enabled: Boolean; }
record GapPayload { gap: Integer; max_reading_gap: Integer; }
record LimitPayload { limit: Integer; amount: Integer; setting: Integer; }
record Instance { subject: Integer; subformula: Integer; born: Integer; }
union Outcome { Failed(GapPayload), False, Incomplete(LimitPayload), Indeterminate, Pending, True }
record Result { instance: Instance; outcome: Outcome; }
type Results = Sequence<Result>;
record StateHolder { right_state: State; left_state: State; }
tuple StatePair(State, State);
tuple StatePairChild(State, State);
tuple IntegerPair(Integer, Boolean);
tuple BoundedIntegerPair(Int[0, 10], Boolean);
tuple BoolPair(Boolean, Boolean);
tuple Step(State, Results);
tuple StepBuffered(StateBuffered, Results);
tuple StepMissingPosition(StateMissingPosition, Results);
function missing_position_init using v(): StateMissingPosition pure { StateMissingPosition { now: 0, history_start: 0, activations: sequence[] } }
function missing_position_event using v(s: StateMissingPosition, delta: Integer, captures: Inputs): StepMissingPosition pure { StepMissingPosition(StateMissingPosition { now: 0, history_start: 0, activations: sequence[] }, sequence[]) }
function missing_position_watermark using v(s: StateMissingPosition, boundary: Integer, progress_scope: Scope): StepMissingPosition pure { StepMissingPosition(StateMissingPosition { now: 0, history_start: 0, activations: sequence[] }, sequence[]) }
function missing_position_gap using v(s: StateMissingPosition, delta: Integer): StepMissingPosition pure { StepMissingPosition(StateMissingPosition { now: 0, history_start: 0, activations: sequence[] }, sequence[]) }
function retain_text_position using v(s: StateTextPosition): StateTextPosition pure { s }
function buffered_init using v(): StateBuffered pure { StateBuffered { now: 0, history_start: 0, buffer_0: sequence[], buffer_1: sequence[], activations: sequence[], event_position: 0 } }
function buffered_event using v(s: StateBuffered, delta: Integer, captures: Inputs): StepBuffered pure { StepBuffered(StateBuffered { now: 0, history_start: 0, buffer_0: sequence[], buffer_1: sequence[], activations: sequence[], event_position: 0 }, sequence[]) }
function buffered_watermark using v(s: StateBuffered, boundary: Integer, progress_scope: Scope): StepBuffered pure { StepBuffered(StateBuffered { now: 0, history_start: 0, buffer_0: sequence[], buffer_1: sequence[], activations: sequence[], event_position: 0 }, sequence[]) }
function buffered_gap using v(s: StateBuffered, delta: Integer): StepBuffered pure { StepBuffered(StateBuffered { now: 0, history_start: 0, buffer_0: sequence[], buffer_1: sequence[], activations: sequence[], event_position: 0 }, sequence[]) }
function retain_state_0 using v(s: StateBuffered): StateBuffered pure { s }
function retain_state_1 using v(s: StateMissingPosition): StateMissingPosition pure { s }
function retain_state_2 using v(s: StateBooleanPosition): StateBooleanPosition pure { s }
function retain_state_3 using v(s: StateMisorderedPosition): StateMisorderedPosition pure { s }
function retain_state_4 using v(s: StateExtraField): StateExtraField pure { s }
function retain_state_5 using v(s: StateOptionalPosition): StateOptionalPosition pure { s }
function retain_state_6 using v(s: StateBadBufferName): StateBadBufferName pure { s }
function retain_state_7 using v(s: StateBadBufferType): StateBadBufferType pure { s }

function entry_init using v(): State pure { State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function entry_event using v(s: State, delta: Integer, captures: Inputs): Step pure { Step(State { now: 0, history_start: 0, activations: sequence[], event_position: 0 }, sequence[]) }
function entry_watermark using v(s: State, boundary: Integer, progress_scope: Scope): Step pure { Step(State { now: 0, history_start: 0, activations: sequence[], event_position: 0 }, sequence[]) }
function entry_gap using v(s: State, delta: Integer): Step pure { Step(State { now: 0, history_start: 0, activations: sequence[], event_position: 0 }, sequence[]) }
function record_init using v(): State pure { let holder = StateHolder { right_state: State { now: 99, history_start: 0, activations: sequence[], event_position: 0 }, left_state: State { now: 11, history_start: 0, activations: sequence[], event_position: 0 } } in holder.left_state }
function touch_holder using v(x: StateHolder): StateHolder pure { x }
function state_pair using v(a: State, b: State): StatePair pure { StatePair(a, b) }
function touch_pair_child using v(x: StatePairChild): StatePairChild pure { x }
function touch_integer_pair using v(x: IntegerPair): IntegerPair pure { x }
function touch_bounded_pair using v(x: BoundedIntegerPair): BoundedIntegerPair pure { x }
function bool_pair using v(): BoolPair pure { BoolPair(true, false) }
function tuple_init using v(): State pure { State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function boolean_init using v(): State pure { if not false and (true or false) then State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } else State { now: 22, history_start: 0, activations: sequence[], event_position: 0 } }
function lexical_init using v(): State pure { let saved = State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } in saved }
function lexical_helper using v(flag: Boolean): State pure { let saved = State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } in if flag then saved else saved }
function pass_second using v(s: State, t: State): State pure { t }
function pass_state using v(s: State): State pure { s }
function option_probe using v(s: Option<Boolean>): State pure { State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function condition using v(): Boolean pure { true }
function unreached_cycle using v(): Boolean pure { true }
function foreign_argument using v(): State pure { State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function bad_if using v(): Boolean pure { true }
function clause using v(): Boolean pure { true }
function touch_tick using v(x: Quantity<tick>): Quantity<tick> pure { x }
function touch_root using v(x: Quantity<root_tick>): Quantity<root_tick> pure { x }
function touch_negative using v(x: Quantity<negative_tick>): Quantity<negative_tick> pure { x }
function touch_affine using v(x: Quantity<affine_tick>): Quantity<affine_tick> pure { x }
function touch_metre using v(x: Quantity<metre>): Quantity<metre> pure { x }
function touch_activation using v(x: Activation): Activation pure { x }
function touch_gap_payload using v(x: GapPayload): GapPayload pure { x }
function touch_limit_payload using v(x: LimitPayload): LimitPayload pure { x }
function touch_instance using v(x: Instance): Instance pure { x }
function touch_outcome using v(x: Outcome): Outcome pure { x }
"#;
const IDENTITIES: &[(&str, &str)] = &[
    ("state-holder", "StateHolder"),
    ("record-init", "record_init"),
    ("state-pair", "StatePair"),
    ("state-pair-child", "StatePairChild"),
    ("integer-pair", "IntegerPair"),
    ("bounded-integer-pair", "BoundedIntegerPair"),
    ("bool-pair", "BoolPair"),
    ("tuple-init", "tuple_init"),
    ("boolean-init", "boolean_init"),
    ("lexical-init", "lexical_init"),
    ("lexical-helper", "lexical_helper"),
    ("pass-second", "pass_second"),
    ("pass-state", "pass_state"),
    ("option-probe", "option_probe"),
    ("state-text-position", "StateTextPosition"),
    ("step-missing-position", "StepMissingPosition"),
    ("init-missing-position", "missing_position_init"),
    ("event-missing-position", "missing_position_event"),
    ("watermark-missing-position", "missing_position_watermark"),
    ("gap-missing-position", "missing_position_gap"),
    ("state-buffered", "StateBuffered"),
    ("state-missing-position", "StateMissingPosition"),
    ("state-boolean-position", "StateBooleanPosition"),
    ("state-misordered-position", "StateMisorderedPosition"),
    ("state-extra-field", "StateExtraField"),
    ("state-optional-position", "StateOptionalPosition"),
    ("state-bad-buffer-name", "StateBadBufferName"),
    ("state-bad-buffer-type", "StateBadBufferType"),
    ("step-buffered", "StepBuffered"),
    ("init-buffered", "buffered_init"),
    ("event-buffered", "buffered_event"),
    ("watermark-buffered", "buffered_watermark"),
    ("gap-buffered", "buffered_gap"),
    ("state", "State"),
    ("activation", "Activation"),
    ("inputs", "Inputs"),
    ("scope", "Scope"),
    ("gap_payload", "GapPayload"),
    ("limit_payload", "LimitPayload"),
    ("instance", "Instance"),
    ("outcome", "Outcome"),
    ("result", "Result"),
    ("step", "Step"),
    ("Time", "Time"),
    ("Length", "Length"),
    ("tick", "tick"),
    ("root-tick", "root_tick"),
    ("negative-tick", "negative_tick"),
    ("affine-tick", "affine_tick"),
    ("init", "entry_init"),
    ("event", "entry_event"),
    ("watermark", "entry_watermark"),
    ("gap", "entry_gap"),
    ("condition", "condition"),
    ("unreached-cycle", "unreached_cycle"),
    ("foreign-argument", "foreign_argument"),
    ("bad-if", "bad_if"),
    ("clause", "clause"),
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
fn state_probe_declarations(
    activations: &ValueType,
    results: &ValueType,
) -> Vec<CompositeDeclaration> {
    let base = || {
        vec![
            ("now", ValueType::Integer),
            ("history_start", ValueType::Integer),
        ]
    };
    let suffix = || {
        vec![
            ("activations", activations.clone()),
            ("event_position", ValueType::Integer),
        ]
    };
    let mut declarations = Vec::new();
    for (name, extra, ending) in [
        (
            "state-buffered",
            vec![
                ("buffer_0", activations.clone()),
                ("buffer_1", activations.clone()),
            ],
            suffix(),
        ),
        (
            "state-missing-position",
            vec![],
            vec![("activations", activations.clone())],
        ),
        (
            "state-text-position",
            vec![],
            vec![
                ("activations", activations.clone()),
                (
                    "event_position",
                    ValueType::Text(TextType::new(0, 8, TextProfile::Nfc).unwrap()),
                ),
            ],
        ),
        (
            "state-boolean-position",
            vec![],
            vec![
                ("activations", activations.clone()),
                ("event_position", ValueType::Boolean),
            ],
        ),
        (
            "state-misordered-position",
            vec![],
            vec![
                ("event_position", ValueType::Integer),
                ("activations", activations.clone()),
            ],
        ),
        (
            "state-extra-field",
            vec![],
            vec![
                ("activations", activations.clone()),
                ("event_position", ValueType::Integer),
                ("spare", ValueType::Boolean),
            ],
        ),
        (
            "state-bad-buffer-name",
            vec![("buffer_00", activations.clone())],
            suffix(),
        ),
        (
            "state-bad-buffer-type",
            vec![("buffer_0", ValueType::Integer)],
            suffix(),
        ),
    ] {
        let mut fields = base();
        fields.extend(extra);
        fields.extend(ending);
        declarations.push(record(name, fields));
    }
    let fields = base()
        .into_iter()
        .chain(suffix())
        .map(|(name, ty)| {
            FieldDeclaration::new(
                name,
                ty,
                if name == "event_position" {
                    Presence::Optional
                } else {
                    Presence::Required
                },
            )
        })
        .collect();
    declarations.push(CompositeDeclaration::new(
        key("state-optional-position"),
        "state-optional-position",
        CompositeShape::Record(fields),
    ));
    declarations.push(CompositeDeclaration::new(
        key("step-buffered"),
        "step-buffered",
        CompositeShape::Tuple(vec![
            ValueType::Composite(key("state-buffered")),
            results.clone(),
        ]),
    ));
    declarations.push(CompositeDeclaration::new(
        key("step-missing-position"),
        "step-missing-position",
        CompositeShape::Tuple(vec![
            ValueType::Composite(key("state-missing-position")),
            results.clone(),
        ]),
    ));
    declarations
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FixtureState {
    Current,
    Buffered,
    MissingPosition,
}

fn parts() -> ProgramParts {
    parts_for(FixtureState::Current)
}
fn parts_for(profile: FixtureState) -> ProgramParts {
    let buffered = profile == FixtureState::Buffered;
    let state_name = match profile {
        FixtureState::Current => "state",
        FixtureState::Buffered => "state-buffered",
        FixtureState::MissingPosition => "state-missing-position",
    };
    let step_name = match profile {
        FixtureState::Current => "step",
        FixtureState::Buffered => "step-buffered",
        FixtureState::MissingPosition => "step-missing-position",
    };

    let state = ValueType::Composite(key(state_name));
    let inputs = ValueType::Composite(key("inputs"));
    let scope = ValueType::Composite(key("scope"));
    let results_type = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key("result")),
        None,
    );
    let results = ValueType::collection(results_type.clone());
    let step_result = ValueType::Composite(key(step_name));
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
                    ("activations", activations.clone()),
                    ("event_position", ValueType::Integer),
                ],
            ),
            record(
                "activation",
                vec![
                    ("subformula", ValueType::Integer),
                    ("born", ValueType::Integer),
                    ("status", ValueType::Boolean),
                ],
            ),
            record("inputs", vec![("enabled", ValueType::Boolean)]),
            record("scope", vec![("enabled", ValueType::Boolean)]),
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
                    ("limit", ValueType::Integer),
                    ("amount", ValueType::Integer),
                    ("setting", ValueType::Integer),
                ],
            ),
            record(
                "instance",
                vec![
                    ("subject", ValueType::Integer),
                    ("subformula", ValueType::Integer),
                    ("born", ValueType::Integer),
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
            record(
                "state-holder",
                vec![
                    ("right_state", ValueType::Composite(key("state"))),
                    ("left_state", ValueType::Composite(key("state"))),
                ],
            ),
            CompositeDeclaration::new(
                key("state-pair"),
                "state-pair",
                CompositeShape::Tuple(vec![
                    ValueType::Composite(key("state")),
                    ValueType::Composite(key("state")),
                ]),
            ),
            // This is a controlled native conformance edge, not an emitted QSL relationship.
            CompositeDeclaration::new(
                key("state-pair-child"),
                "state-pair-child",
                CompositeShape::Tuple(vec![
                    ValueType::Composite(key("state")),
                    ValueType::Composite(key("state")),
                ]),
            )
            .with_supertypes(vec![key("state-pair")]),
            CompositeDeclaration::new(
                key("integer-pair"),
                "integer-pair",
                CompositeShape::Tuple(vec![ValueType::Integer, ValueType::Boolean]),
            ),
            CompositeDeclaration::new(
                key("bounded-integer-pair"),
                "bounded-integer-pair",
                CompositeShape::Tuple(vec![
                    ValueType::Int(
                        IntegerInterval::new(Integer::zero(), Integer::from(10_u64)).unwrap(),
                    ),
                    ValueType::Boolean,
                ]),
            )
            .with_supertypes(vec![key("integer-pair")]),
            CompositeDeclaration::new(
                key("bool-pair"),
                "bool-pair",
                CompositeShape::Tuple(vec![ValueType::Boolean, ValueType::Boolean]),
            ),
            CompositeDeclaration::new(
                key("step"),
                "step",
                CompositeShape::Tuple(vec![ValueType::Composite(key("state")), results.clone()]),
            ),
        ]
        .into_iter()
        .chain(state_probe_declarations(&activations, &results)),
        [],
    )
    .expect("admit diagnostic nominal declarations");
    let zero = Rational::from_integer(Integer::zero());
    let one = Rational::from_integer(Integer::one());
    let types = types.with_units(admitted_time_units());
    let mut fields = vec![
        ("now", FieldValue::Present(Value::Integer(Integer::zero()))),
        (
            "history_start",
            FieldValue::Present(Value::Integer(Integer::zero())),
        ),
    ];
    if buffered {
        for name in ["buffer_0", "buffer_1"] {
            fields.push((
                name,
                FieldValue::Present(quire_exact::from_admitted(activation_type.clone(), vec![])),
            ));
        }
    }
    fields.push((
        "activations",
        FieldValue::Present(quire_exact::from_admitted(activation_type, vec![])),
    ));
    if profile != FixtureState::MissingPosition {
        fields.push((
            "event_position",
            FieldValue::Present(Value::Integer(Integer::zero())),
        ));
    }
    let state_value = types.record(key(state_name), fields).unwrap();
    let result_value = quire_exact::from_admitted(results_type, vec![]);
    let step_value = types
        .tuple(key(step_name), vec![state_value.clone(), result_value])
        .unwrap();
    let names = match profile {
        FixtureState::Current => ["init", "event", "watermark", "gap"],
        FixtureState::Buffered => [
            "init-buffered",
            "event-buffered",
            "watermark-buffered",
            "gap-buffered",
        ],
        FixtureState::MissingPosition => [
            "init-missing-position",
            "event-missing-position",
            "watermark-missing-position",
            "gap-missing-position",
        ],
    };
    let function_keys = names.map(key);
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
            state: binding(state_name, state),
            inputs: binding("inputs", inputs),
            scope: binding("scope", scope),
            results: binding("results", results),
            step_result: binding(step_name, step_result),
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
            buffers: if buffered {
                vec![
                    Buffer {
                        owner: key("clause"),
                        capacity: Integer::one(),
                    },
                    Buffer {
                        owner: key("condition"),
                        capacity: Integer::from(2_u64),
                    },
                ]
            } else {
                vec![]
            },
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
    let [FieldValue::Present(Value::Integer(now)), FieldValue::Present(Value::Integer(history_start)), FieldValue::Present(Value::Collection(activations)), FieldValue::Present(Value::Integer(event_position))] =
        state.slots()
    else {
        panic!("now/history_start/activations/event_position state contents");
    };
    assert_eq!(now, &Integer::zero());
    assert_eq!(history_start, &Integer::zero());
    assert_eq!(event_position, &Integer::zero());
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
    assert!(second
        .layout()
        .root
        .path()
        .iter()
        .any(|part| matches!(part, quire_exact::PathSegment::Node(_))));
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

#[trace("TC-098", "IT-018-SC-03", "interface-002-AC-2")]
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
    for (label, scale, affine) in [("negative-tick", -1_i64, false), ("affine-tick", 1, true)] {
        let mut input = parts();
        input.layout.tick_unit = key(label);
        let program =
            Program::try_new(input).expect("retain admitted Time unit without extra restrictions");
        let id = program
            .types()
            .units()
            .ids()
            .find(|id| id.declared_key() == Some(&key(label)))
            .unwrap();
        let retained = program.types().units().get(id).unwrap();
        let quire_semantic_value::quantity::QuantityUnit::Declared(retained) = retained else {
            panic!("declared unit was replaced");
        };
        assert_eq!(retained.dimension_node(), key("Time"));
        assert_eq!(
            retained.canonical().scale(),
            &Rational::from_integer(Integer::from(scale))
        );
        assert_eq!(retained.is_affine(), affine);
    }
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn closed_state_retains_event_position_after_ordered_buffers() {
    let program = Program::try_new(parts_for(FixtureState::Buffered)).unwrap();
    let CompositeShape::Record(fields) = program
        .types()
        .composite(key("state-buffered"))
        .unwrap()
        .shape()
    else {
        panic!("State record");
    };
    assert_eq!(
        fields.iter().map(|field| field.name()).collect::<Vec<_>>(),
        [
            "now",
            "history_start",
            "buffer_0",
            "buffer_1",
            "activations",
            "event_position"
        ]
    );
    let body = program
        .expression(
            program
                .function(program.layout().entries.init)
                .unwrap()
                .body,
        )
        .unwrap();
    let Operation::Literal(Value::Composite(value)) = &body.operation else {
        panic!("retained initializer value");
    };
    let Some(FieldValue::Present(Value::Integer(position))) = value.slots().last() else {
        panic!("last value is the event position");
    };
    assert_eq!(position, &Integer::zero());
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn closed_state_refuses_missing_optional_mistyped_extra_and_misordered_fields() {
    for (name, buffers) in [
        ("state-missing-position", 0),
        ("state-boolean-position", 0),
        ("state-text-position", 0),
        ("state-misordered-position", 0),
        ("state-extra-field", 0),
        ("state-optional-position", 0),
        ("state-buffered", 0),
        ("state-bad-buffer-name", 1),
        ("state-bad-buffer-type", 1),
    ] {
        let mut input = parts();
        input.layout.state = TypeBinding {
            key: key(name),
            value_type: ValueType::Composite(key(name)),
        };
        if buffers == 1 {
            input.layout.buffers = vec![Buffer {
                owner: key("clause"),
                capacity: Integer::one(),
            }];
        }
        let Err(error) = Program::try_new(input) else {
            panic!("malformed State must refuse: {name}");
        };
        assert_eq!(error.code, ProgramRefusalCode::TypeMismatch, "{name}");
        assert_eq!(
            error.node,
            Some(key(name)),
            "State must fail before later entry-signature checks: {name}"
        );
    }
}

#[trace("TC-098", "IT-018-SC-02", "interface-002-AC-2")]
#[test]
fn state_without_event_position_has_no_compatibility_path() {
    let Err(error) = Program::try_new(parts_for(FixtureState::MissingPosition)) else {
        panic!("a fully typed closure without event_position must refuse");
    };
    assert_eq!(error.code, ProgramRefusalCode::TypeMismatch);
    assert_eq!(error.node, Some(key("state-missing-position")));
}

fn execution_budget() -> StepBudget {
    StepBudget {
        work: 100_000,
        stack_frames: 10_000,
        scratch_elements: 10_000,
        scratch_bytes: 1_000_000,
    }
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-1")]
#[test]
fn executor_build_retains_owner_and_initializes_independent_state_slots() {
    let program = Program::try_new(parts()).unwrap();
    let first = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let second = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    assert!(Rc::ptr_eq(first.program(), second.program()));
    drop(program);
    assert_state_contents(first.state());
    assert_state_contents(second.state());
    assert!(
        !std::ptr::eq(first.state(), second.state()),
        "each build owns its replacement slot"
    );
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-2", "interface-003-AC-4")]
#[test]
fn executor_operating_stops_and_budget_replacement_are_typed_and_atomic() {
    for (budget, resource) in [
        (
            StepBudget {
                work: 0,
                ..execution_budget()
            },
            StepResource::Work,
        ),
        (
            StepBudget {
                stack_frames: 0,
                ..execution_budget()
            },
            StepResource::Stack,
        ),
        (
            StepBudget {
                scratch_elements: 0,
                ..execution_budget()
            },
            StepResource::ScratchElements,
        ),
    ] {
        let failure = match build(BuildPlan {
            program: Program::try_new(parts()).unwrap(),
            budget,
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("zero operating resource must stop init"),
        };
        assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
        assert_eq!(failure.resource.unwrap().resource, resource);
    }
    let mut monitor = build(BuildPlan {
        program: Program::try_new(parts()).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    let state = monitor.state() as *const Value;
    let owner = Rc::as_ptr(monitor.program());
    let impossible = StepBudget {
        stack_frames: usize::MAX,
        ..execution_budget()
    };
    let failure = monitor.set_step_budget(impossible).unwrap_err();
    assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
    assert_eq!(failure.resource.unwrap().resource, StepResource::Storage);
    assert_eq!(monitor.step_budget(), execution_budget());
    assert_eq!(monitor.state() as *const Value, state);
    assert_eq!(Rc::as_ptr(monitor.program()), owner);
    let replacement = StepBudget {
        work: 200_000,
        ..execution_budget()
    };
    monitor.set_step_budget(replacement).unwrap();
    assert_eq!(monitor.step_budget(), replacement);
    assert_state_contents(monitor.state());
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-2")]
#[test]
fn executor_lazy_branch_does_not_enter_an_unselected_recursive_call() {
    let mut input = parts();
    let recursive = ExpressionId(input.expressions.len());
    input.expressions.push(Expression {
        key: key("init"),
        body_path: vec![1],
        value_type: input.layout.state.value_type.clone(),
        operation: Operation::Call {
            function: FunctionId(0),
            arguments: vec![],
        },
    });
    let condition = ExpressionId(input.expressions.len());
    input.expressions.push(Expression {
        key: key("condition"),
        body_path: vec![1],
        value_type: ValueType::Boolean,
        operation: Operation::Literal(Value::Boolean(true)),
    });
    let root = ExpressionId(input.expressions.len());
    input.expressions.push(Expression {
        key: key("init"),
        body_path: vec![2],
        value_type: input.layout.state.value_type.clone(),
        operation: Operation::If {
            condition,
            when_true: ExpressionId(0),
            when_false: recursive,
        },
    });
    input.functions[0].body = root;
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_state_contents(monitor.state());
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-3")]
#[test]
fn executor_wrong_initializer_result_preserves_native_membership_failure() {
    let mut input = parts();
    input.expressions[0].value_type = ValueType::Boolean;
    input.expressions[0].operation = Operation::Literal(Value::Boolean(true));
    let failure = match build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("a Boolean body cannot initialize nominal State"),
    };
    assert_eq!(failure.code, MonitorBuildFailureCode::Invariant);
    let invariant = failure.invariant.unwrap();
    assert_eq!(
        invariant.code,
        quire_contract_runtime::monitor::ExecutorInvariantCode::Membership
    );
    assert_eq!(invariant.source_node(), Some(&key("init")));
    let native = failure
        .admission
        .expect("keep the native invalid-membership cause and its arena");
    assert!(matches!(
        native.failure,
        quire_semantic_value::supplied_membership::SuppliedAdmissionError::Invalid { .. }
    ));
    assert_eq!(native.source_node(), Some(&key("init")));
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-2")]
#[test]
fn executor_deep_lazy_chain_uses_bounded_explicit_stack() {
    let _ = key("init");
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut input = parts();
            let condition = ExpressionId(input.expressions.len());
            input.expressions.push(Expression {
                key: key("condition"),
                body_path: vec![1],
                value_type: ValueType::Boolean,
                operation: Operation::Literal(Value::Boolean(true)),
            });
            let mut previous = ExpressionId(0);
            for position in 0..2_000 {
                let next = ExpressionId(input.expressions.len());
                input.expressions.push(Expression {
                    key: key("init"),
                    body_path: vec![position + 1],
                    value_type: input.layout.state.value_type.clone(),
                    operation: Operation::If {
                        condition,
                        when_true: previous,
                        when_false: previous,
                    },
                });
                previous = next;
            }
            input.functions[0].body = previous;
            let program = Program::try_new(input).unwrap();
            let failure = match build(BuildPlan {
                program: Rc::clone(&program),
                budget: StepBudget {
                    stack_frames: 32,
                    ..execution_budget()
                },
            }) {
                Err(failure) => failure,
                Ok(_) => panic!("the explicit continuation stack must stop the deep path"),
            };
            assert_eq!(failure.resource.unwrap().resource, StepResource::Stack);
            let monitor = build(BuildPlan {
                program,
                budget: execution_budget(),
            })
            .unwrap();
            assert_state_contents(monitor.state());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-2")]
#[test]
fn executor_nested_calls_keep_argument_order_and_owning_scopes() {
    let mut input = parts();
    let state = input.layout.state.value_type.clone();
    let Value::Composite(value) = (match &input.expressions[0].operation {
        Operation::Literal(value) => value,
        _ => panic!("initializer literal"),
    }) else {
        panic!("State");
    };
    let alternate = input
        .types
        .record(
            key("state"),
            vec![
                (
                    "now",
                    FieldValue::Present(Value::Integer(Integer::from(99_u64))),
                ),
                ("history_start", value.slots()[1].clone()),
                ("activations", value.slots()[2].clone()),
                ("event_position", value.slots()[3].clone()),
            ],
        )
        .unwrap();
    input.expressions.extend([
        Expression {
            key: key("pass-second"),
            body_path: vec![0],
            value_type: state.clone(),
            operation: Operation::Argument {
                function: FunctionId(4),
                position: 1,
            },
        },
        Expression {
            key: key("pass-state"),
            body_path: vec![0],
            value_type: state.clone(),
            operation: Operation::Argument {
                function: FunctionId(5),
                position: 0,
            },
        },
        Expression {
            key: key("init"),
            body_path: vec![1],
            value_type: state.clone(),
            operation: Operation::Literal(alternate),
        },
        Expression {
            key: key("init"),
            body_path: vec![2],
            value_type: state.clone(),
            operation: Operation::Call {
                function: FunctionId(5),
                arguments: vec![ExpressionId(6)],
            },
        },
        Expression {
            key: key("init"),
            body_path: vec![3],
            value_type: state.clone(),
            operation: Operation::Call {
                function: FunctionId(4),
                arguments: vec![ExpressionId(0), ExpressionId(7)],
            },
        },
    ]);
    input.functions.extend([
        Function {
            key: key("pass-second"),
            parameters: vec![state.clone(), state.clone()],
            result: state.clone(),
            body: ExpressionId(4),
        },
        Function {
            key: key("pass-state"),
            parameters: vec![state.clone()],
            result: state,
            body: ExpressionId(5),
        },
    ]);
    input.functions[0].body = ExpressionId(8);
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    let Value::Composite(value) = monitor.state() else {
        panic!("State after nested helper calls");
    };
    let FieldValue::Present(Value::Integer(now)) = &value.slots()[0] else {
        panic!("integer now");
    };
    assert_eq!(
        now,
        &Integer::from(99_u64),
        "second argument survives nested invocation and scope rollback"
    );
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-2")]
#[test]
fn executor_selected_recursion_stops_on_the_explicit_stack() {
    let mut input = parts();
    input.expressions[0].operation = Operation::Call {
        function: FunctionId(0),
        arguments: vec![],
    };
    let failure = match build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: StepBudget {
            stack_frames: 32,
            ..execution_budget()
        },
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("selected recursive call cannot finish before its stack stop"),
    };
    assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
    assert_eq!(failure.resource.unwrap().resource, StepResource::Stack);
    assert!(failure.invariant.is_none());
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-3")]
#[test]
fn executor_public_membership_work_stop_keeps_native_cause_and_arena() {
    let budget = StepBudget {
        work: 3,
        ..execution_budget()
    };
    let expected = StepResource::Work;
    let mut input = parts();
    let option_integer = ValueType::option(ValueType::Integer);
    input.expressions.push(Expression {
        key: key("init"),
        body_path: vec![1],
        value_type: option_integer,
        operation: Operation::Literal(quire_exact::OptionValue::from_admitted(
            ValueType::Integer,
            Some(Value::Integer(Integer::one())),
        )),
    });
    input.expressions.push(Expression {
        key: key("init"),
        body_path: vec![2],
        value_type: input.layout.state.value_type.clone(),
        operation: Operation::Call {
            function: FunctionId(4),
            arguments: vec![ExpressionId(4)],
        },
    });
    input.functions.push(Function {
        key: key("option-probe"),
        parameters: vec![ValueType::option(ValueType::Boolean)],
        result: input.layout.state.value_type.clone(),
        body: ExpressionId(0),
    });
    input.functions[0].body = ExpressionId(5);
    let failure = match build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("native argument admission must stop before invoking the helper"),
    };
    assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
    let stop = failure.resource.unwrap();
    assert_eq!(stop.resource, expected);
    let native = failure
        .admission
        .expect("original native membership stop retained");
    match (expected, native.failure) {
        (
            StepResource::Work,
            quire_semantic_value::supplied_membership::SuppliedAdmissionError::Limit {
                locus, ..
            },
        ) => {
            assert_eq!((stop.spent, stop.requested), (3, 1));
            assert!(native.arena.resolve(locus).is_some());
        }
        _ => panic!("native cause must match the operating stop, without an invariant"),
    }
    assert!(failure.invariant.is_none());
}

fn lexical_base() -> ProgramParts {
    let mut input = parts();
    input.functions[0].key = key("lexical-init");
    input.expressions[0].key = key("lexical-init");
    input
}

fn lexical_expression(
    input: &mut ProgramParts,
    owner: &str,
    value_type: ValueType,
    operation: Operation,
) -> ExpressionId {
    let id = ExpressionId(input.expressions.len());
    input.expressions.push(Expression {
        key: key(owner),
        body_path: vec![id.0 + 100],
        value_type,
        operation,
    });
    id
}

fn state_at(input: &ProgramParts, now: u64) -> Value {
    let Operation::Literal(Value::Composite(state)) = &input.expressions[0].operation else {
        panic!("state literal");
    };
    input
        .types
        .record(
            key("state"),
            vec![
                (
                    "now",
                    FieldValue::Present(Value::Integer(Integer::from(now))),
                ),
                ("history_start", state.slots()[1].clone()),
                ("activations", state.slots()[2].clone()),
                ("event_position", state.slots()[3].clone()),
            ],
        )
        .unwrap()
}

fn assert_now(monitor: &quire_contract_runtime::monitor::Monitor, expected: u64) {
    let Value::Composite(state) = monitor.state() else {
        panic!("state");
    };
    let FieldValue::Present(Value::Integer(now)) = &state.slots()[0] else {
        panic!("now");
    };
    assert_eq!(now, &Integer::from(expected));
}

fn one_lexical_binding() -> ProgramParts {
    let mut input = lexical_base();
    let value = state_at(&input, 99);
    input.expressions[0].operation = Operation::Literal(value);
    let state = input.layout.state.value_type.clone();
    let binding = LocalId {
        function: FunctionId(0),
        position: 0,
    };
    let read = lexical_expression(
        &mut input,
        "lexical-init",
        state.clone(),
        Operation::Local(binding),
    );
    let body = lexical_expression(
        &mut input,
        "lexical-init",
        state,
        Operation::Let {
            binding,
            value: ExpressionId(0),
            body: read,
        },
    );
    input.functions[0].body = body;
    input
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-2")]
#[test]
fn lexical_value_is_evaluated_once_within_its_scoped_body() {
    let program = Program::try_new(one_lexical_binding()).unwrap();
    let monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: StepBudget {
            work: 7,
            ..execution_budget()
        },
    })
    .unwrap();
    assert_now(&monitor, 99);
    let failure = match build(BuildPlan {
        program,
        budget: StepBudget {
            work: 6,
            ..execution_budget()
        },
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("seven required continuations cannot fit six units"),
    };
    assert_eq!(failure.resource.unwrap().resource, StepResource::Work);
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-1")]
#[test]
fn lexical_missing_duplicate_sparse_and_foreign_declarations_refuse() {
    for case in 0..4 {
        let mut input = one_lexical_binding();
        match case {
            0 => {
                input.expressions.pop();
                input.functions[0].body = ExpressionId(4);
            }
            1 => {
                let state = input.layout.state.value_type.clone();
                lexical_expression(
                    &mut input,
                    "lexical-init",
                    state,
                    Operation::Let {
                        binding: LocalId {
                            function: FunctionId(0),
                            position: 0,
                        },
                        value: ExpressionId(0),
                        body: ExpressionId(4),
                    },
                );
            }
            2 | 3 => {
                let binding = LocalId {
                    function: FunctionId(if case == 3 { 1 } else { 0 }),
                    position: if case == 2 { usize::MAX } else { 0 },
                };
                input.expressions[4].operation = Operation::Local(binding);
                input.expressions[5].operation = Operation::Let {
                    binding,
                    value: ExpressionId(0),
                    body: ExpressionId(4),
                };
            }
            _ => unreachable!(),
        }
        refused(input, ProgramRefusalCode::InvalidBinding);
    }
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-2", "interface-004-AC-3")]
#[test]
fn lexical_unselected_branch_does_not_activate_a_local() {
    for selected in [true, false] {
        let mut input = one_lexical_binding();
        let state = input.layout.state.value_type.clone();
        let condition = lexical_expression(
            &mut input,
            "lexical-init",
            ValueType::Boolean,
            Operation::Literal(Value::Boolean(selected)),
        );
        let read = lexical_expression(
            &mut input,
            "lexical-init",
            state.clone(),
            Operation::Local(LocalId {
                function: FunctionId(0),
                position: 0,
            }),
        );
        let root = lexical_expression(
            &mut input,
            "lexical-init",
            state,
            Operation::If {
                condition,
                when_true: ExpressionId(5),
                when_false: read,
            },
        );
        input.functions[0].body = root;
        let result = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        });
        if selected {
            assert_now(&result.unwrap(), 99);
        } else {
            let failure = match result {
                Err(failure) => failure,
                Ok(_) => panic!("an inactive branch cannot supply its local"),
            };
            assert_eq!(failure.code, MonitorBuildFailureCode::Invariant);
            assert_eq!(
                failure.invariant.unwrap().code,
                quire_contract_runtime::monitor::ExecutorInvariantCode::Binding
            );
        }
    }
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-2")]
#[test]
fn lexical_different_annotation_preserves_native_membership_failure() {
    let mut input = one_lexical_binding();
    input.expressions[0].value_type = ValueType::Boolean;
    input.expressions[0].operation = Operation::Literal(Value::Boolean(true));
    let failure = match build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("Boolean local cannot become State by annotation"),
    };
    assert_eq!(failure.code, MonitorBuildFailureCode::Invariant);
    assert_eq!(
        failure.invariant.unwrap().code,
        quire_contract_runtime::monitor::ExecutorInvariantCode::Membership
    );
    let native = failure.admission.unwrap();
    assert!(matches!(
        native.failure,
        quire_semantic_value::supplied_membership::SuppliedAdmissionError::Invalid { .. }
    ));
    assert_eq!(native.source_node(), Some(&key("lexical-init")));
}

fn recursive_local_parts(isolation: bool) -> ProgramParts {
    let mut input = lexical_base();
    let state = input.layout.state.value_type.clone();
    let outer_value = state_at(&input, 99);
    let inner_value = state_at(&input, 22);
    let owner = "lexical-helper";
    let binding = LocalId {
        function: FunctionId(4),
        position: 0,
    };
    let argument = lexical_expression(
        &mut input,
        owner,
        ValueType::Boolean,
        Operation::Argument {
            function: FunctionId(4),
            position: 0,
        },
    );
    let outer = lexical_expression(
        &mut input,
        owner,
        state.clone(),
        Operation::Literal(outer_value),
    );
    let inner = lexical_expression(
        &mut input,
        owner,
        state.clone(),
        Operation::Literal(inner_value),
    );
    let selected = lexical_expression(
        &mut input,
        owner,
        state.clone(),
        Operation::If {
            condition: argument,
            when_true: outer,
            when_false: inner,
        },
    );
    let read = lexical_expression(&mut input, owner, state.clone(), Operation::Local(binding));
    let no = lexical_expression(
        &mut input,
        owner,
        ValueType::Boolean,
        Operation::Literal(Value::Boolean(false)),
    );
    let recursive = lexical_expression(
        &mut input,
        owner,
        state.clone(),
        Operation::Call {
            function: FunctionId(4),
            arguments: vec![no],
        },
    );
    let body = if isolation {
        let bound = lexical_expression(
            &mut input,
            owner,
            state.clone(),
            Operation::Let {
                binding,
                value: selected,
                body: recursive,
            },
        );
        lexical_expression(
            &mut input,
            owner,
            state.clone(),
            Operation::If {
                condition: argument,
                when_true: bound,
                when_false: read,
            },
        )
    } else {
        let ignored = lexical_expression(
            &mut input,
            owner,
            state.clone(),
            Operation::Let {
                binding: LocalId {
                    function: FunctionId(4),
                    position: 1,
                },
                value: recursive,
                body: read,
            },
        );
        let continuation = lexical_expression(
            &mut input,
            owner,
            state.clone(),
            Operation::If {
                condition: argument,
                when_true: ignored,
                when_false: read,
            },
        );
        lexical_expression(
            &mut input,
            owner,
            state.clone(),
            Operation::Let {
                binding,
                value: selected,
                body: continuation,
            },
        )
    };
    input.functions.push(Function {
        key: key(owner),
        parameters: vec![ValueType::Boolean],
        result: state.clone(),
        body,
    });
    let yes = lexical_expression(
        &mut input,
        "lexical-init",
        ValueType::Boolean,
        Operation::Literal(Value::Boolean(true)),
    );
    let root = lexical_expression(
        &mut input,
        "lexical-init",
        state,
        Operation::Call {
            function: FunctionId(4),
            arguments: vec![yes],
        },
    );
    input.functions[0].body = root;
    input
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-3")]
#[test]
fn lexical_recursive_return_restores_the_outer_binding() {
    let monitor = build(BuildPlan {
        program: Program::try_new(recursive_local_parts(false)).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 99);
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-3")]
#[test]
fn lexical_recursive_invocation_cannot_capture_a_callers_active_local() {
    let failure = match build(BuildPlan {
        program: Program::try_new(recursive_local_parts(true)).unwrap(),
        budget: execution_budget(),
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("callee without its own activation cannot read caller's local"),
    };
    assert_eq!(failure.code, MonitorBuildFailureCode::Invariant);
    let invariant = failure.invariant.unwrap();
    assert_eq!(
        invariant.code,
        quire_contract_runtime::monitor::ExecutorInvariantCode::Binding
    );
    assert_eq!(invariant.source_node(), Some(&key("lexical-helper")));
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-4")]
#[test]
fn lexical_deep_chain_charges_live_bindings_and_uses_explicit_stack() {
    let _ = key("lexical-init");
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let depth = 2_000;
            let mut input = lexical_base();
            let state = input.layout.state.value_type.clone();
            let mut previous = ExpressionId(0);
            for position in 0..depth {
                previous = lexical_expression(
                    &mut input,
                    "lexical-init",
                    state.clone(),
                    Operation::Let {
                        binding: LocalId {
                            function: FunctionId(0),
                            position,
                        },
                        value: ExpressionId(0),
                        body: previous,
                    },
                );
            }
            input.functions[0].body = previous;
            let program = Program::try_new(input).unwrap();
            let stopped = match build(BuildPlan {
                program: Rc::clone(&program),
                budget: StepBudget {
                    scratch_elements: depth,
                    ..execution_budget()
                },
            }) {
                Err(failure) => failure,
                Ok(_) => panic!("the body result needs a slot beside every active binding"),
            };
            let stop = stopped.resource.unwrap();
            assert_eq!(stop.resource, StepResource::ScratchElements);
            assert_eq!((stop.spent, stop.requested), (depth as u128, 1));
            let monitor = build(BuildPlan {
                program,
                budget: StepBudget {
                    scratch_elements: depth + 1,
                    ..execution_budget()
                },
            })
            .unwrap();
            assert_now(&monitor, 0);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[trace("TC-098", "IT-018-SC-06", "interface-004-AC-4")]
#[test]
fn lexical_recursive_shadow_counts_the_retained_caller_operand() {
    let program = Program::try_new(recursive_local_parts(false)).unwrap();
    let stopped = match build(BuildPlan {
        program: Rc::clone(&program),
        budget: StepBudget {
            scratch_elements: 4,
            ..execution_budget()
        },
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("recursive shadow retains the outer operand beside its own"),
    };
    let stop = stopped.resource.unwrap();
    assert_eq!(stop.resource, StepResource::ScratchElements);
    assert_eq!((stop.limit, stop.spent, stop.requested), (Some(4), 4, 1));
    let monitor = build(BuildPlan {
        program,
        budget: StepBudget {
            scratch_elements: 5,
            ..execution_budget()
        },
    })
    .unwrap();
    assert_now(&monitor, 99);
}

#[derive(Clone, Copy)]
enum BooleanProbe {
    Not,
    And,
    Or,
}

fn boolean_parts(which: BooleanProbe, left: bool, right: bool) -> (ProgramParts, ExpressionId) {
    let mut input = parts();
    input.functions[0].key = key("boolean-init");
    input.expressions[0].key = key("boolean-init");
    let state = input.layout.state.value_type.clone();
    let yes = state_at(&input, 99);
    let no = state_at(&input, 22);
    let owner = "boolean-init";
    let left = lexical_expression(
        &mut input,
        owner,
        ValueType::Boolean,
        Operation::Literal(Value::Boolean(left)),
    );
    let right = lexical_expression(
        &mut input,
        owner,
        ValueType::Boolean,
        Operation::Literal(Value::Boolean(right)),
    );
    let operation = match which {
        BooleanProbe::Not => Operation::BooleanNot { value: left },
        BooleanProbe::And => Operation::BooleanAnd { left, right },
        BooleanProbe::Or => Operation::BooleanOr { left, right },
    };
    let result = lexical_expression(&mut input, owner, ValueType::Boolean, operation);
    let yes = lexical_expression(&mut input, owner, state.clone(), Operation::Literal(yes));
    let no = lexical_expression(&mut input, owner, state.clone(), Operation::Literal(no));
    let root = lexical_expression(
        &mut input,
        owner,
        state,
        Operation::If {
            condition: result,
            when_true: yes,
            when_false: no,
        },
    );
    input.functions[0].body = root;
    (input, result)
}

#[trace("TC-098", "IT-018-SC-07", "interface-005-AC-1")]
#[test]
fn boolean_native_values_follow_independent_truth_tables() {
    for (which, left, right, expected) in [
        (BooleanProbe::Not, false, false, true),
        (BooleanProbe::Not, true, false, false),
        (BooleanProbe::And, false, false, false),
        (BooleanProbe::And, false, true, false),
        (BooleanProbe::And, true, false, false),
        (BooleanProbe::And, true, true, true),
        (BooleanProbe::Or, false, false, false),
        (BooleanProbe::Or, false, true, true),
        (BooleanProbe::Or, true, false, true),
        (BooleanProbe::Or, true, true, true),
    ] {
        let (input, _) = boolean_parts(which, left, right);
        let monitor = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap();
        assert_now(&monitor, if expected { 99 } else { 22 });
    }
}

#[trace("TC-098", "IT-018-SC-07", "interface-005-AC-2")]
#[test]
fn boolean_right_recursion_is_observable_only_when_selected() {
    for (which, left, selected, expected) in [
        (BooleanProbe::And, false, false, 22),
        (BooleanProbe::And, true, true, 0),
        (BooleanProbe::Or, true, false, 99),
        (BooleanProbe::Or, false, true, 0),
    ] {
        let (mut input, result) = boolean_parts(which, left, false);
        let recursive = lexical_expression(
            &mut input,
            "condition",
            ValueType::Boolean,
            Operation::Call {
                function: FunctionId(4),
                arguments: vec![],
            },
        );
        input.functions.push(Function {
            key: key("condition"),
            parameters: vec![],
            result: ValueType::Boolean,
            body: recursive,
        });
        match &mut input.expressions[result.0].operation {
            Operation::BooleanAnd { right, .. } | Operation::BooleanOr { right, .. } => {
                *right = recursive
            }
            _ => panic!("binary probe"),
        }
        let outcome = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: StepBudget {
                stack_frames: 32,
                ..execution_budget()
            },
        });
        if selected {
            let failure = match outcome {
                Err(failure) => failure,
                Ok(_) => panic!("selected recursion must hit its explicit ceiling"),
            };
            assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
            assert_eq!(failure.resource.unwrap().resource, StepResource::Stack);
        } else {
            assert_now(&outcome.unwrap(), expected);
        }
    }
}

#[trace("TC-098", "IT-018-SC-07", "interface-005-AC-3")]
#[test]
fn boolean_non_boolean_operands_and_result_refuse_before_execution() {
    for case in 0..3 {
        let (mut input, result) = boolean_parts(BooleanProbe::And, true, true);
        match case {
            0 => {
                input.expressions[result.0].operation = Operation::BooleanNot {
                    value: ExpressionId(0),
                }
            }
            1 => {
                let Operation::BooleanAnd { right, .. } =
                    &mut input.expressions[result.0].operation
                else {
                    panic!("and");
                };
                *right = ExpressionId(0);
            }
            2 => {
                input.expressions[result.0].value_type = input.layout.state.value_type.clone();
                input.functions[0].body = result;
            }
            _ => unreachable!(),
        }
        refused(input, ProgramRefusalCode::TypeMismatch);
    }
}

#[trace("TC-098", "IT-018-SC-07", "interface-005-AC-3")]
#[test]
fn boolean_operating_stops_are_never_boolean_results() {
    for (budget, expected) in [
        (
            StepBudget {
                work: 5,
                ..execution_budget()
            },
            StepResource::Work,
        ),
        (
            StepBudget {
                stack_frames: 3,
                ..execution_budget()
            },
            StepResource::Stack,
        ),
        (
            StepBudget {
                scratch_elements: 0,
                ..execution_budget()
            },
            StepResource::ScratchElements,
        ),
    ] {
        let (input, _) = boolean_parts(BooleanProbe::Not, true, false);
        let failure = match build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget,
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("insufficient operating resources cannot become a false result"),
        };
        assert_eq!(failure.code, MonitorBuildFailureCode::Resource);
        assert_eq!(failure.resource.unwrap().resource, expected);
        assert!(failure.invariant.is_none());
    }
}

fn tuple_parts(position: usize) -> (ProgramParts, ExpressionId) {
    let mut input = parts();
    input.functions[0].key = key("tuple-init");
    input.expressions[0].key = key("tuple-init");
    let first = state_at(&input, 11);
    let second = state_at(&input, 99);
    let tuple = input
        .types
        .tuple(key("state-pair"), vec![first, second])
        .unwrap();
    let literal = lexical_expression(
        &mut input,
        "tuple-init",
        ValueType::Composite(key("state-pair")),
        Operation::Literal(tuple),
    );
    let state = input.layout.state.value_type.clone();
    let root = lexical_expression(
        &mut input,
        "tuple-init",
        state,
        Operation::TupleElement {
            value: literal,
            position,
        },
    );
    input.functions[0].body = root;
    (input, literal)
}

#[trace("TC-098", "IT-018-SC-08", "interface-006-AC-1")]
#[test]
fn tuple_projected_state_outlives_the_temporary_owner_in_declared_position_order() {
    for (position, expected) in [(0, 11), (1, 99)] {
        let (input, literal) = tuple_parts(position);
        let program = Program::try_new(input).unwrap();
        let monitor = build(BuildPlan {
            program,
            budget: execution_budget(),
        })
        .unwrap();
        assert_now(&monitor, expected);
        let Operation::Literal(Value::Composite(owner)) =
            &monitor.program().expression(literal).unwrap().operation
        else {
            panic!("source tuple remains retained");
        };
        assert_eq!(
            std::sync::Arc::strong_count(owner),
            1,
            "temporary projection owner was released before build returned"
        );
        let Value::Composite(state) = monitor.state() else {
            panic!("projected State");
        };
        assert_eq!(
            std::sync::Arc::strong_count(state),
            2,
            "one original tuple slot and one monitor state owner"
        );
    }
}

#[trace("TC-098", "IT-018-SC-08", "interface-006-AC-2")]
#[test]
fn tuple_projection_rejects_record_receivers_missing_positions_and_wrong_types() {
    for case in 0..3 {
        let (mut input, _) = tuple_parts(0);
        let root = input.functions[0].body;
        match case {
            0 => {
                input.expressions[root.0].operation = Operation::TupleElement {
                    value: ExpressionId(0),
                    position: 0,
                }
            }
            1 => {
                let Operation::TupleElement { position, .. } =
                    &mut input.expressions[root.0].operation
                else {
                    panic!("tuple projection");
                };
                *position = 2;
            }
            2 => input.expressions[root.0].value_type = ValueType::Boolean,
            _ => unreachable!(),
        }
        refused(
            input,
            if case == 1 {
                ProgramRefusalCode::InvalidReference
            } else {
                ProgramRefusalCode::TypeMismatch
            },
        );
    }
}

#[trace("TC-098", "IT-018-SC-08", "interface-006-AC-3")]
#[test]
fn tuple_projection_retained_addresses_respect_scratch_budget() {
    let (input, _) = tuple_parts(1);
    let program = Program::try_new(input).unwrap();
    let failure = match build(BuildPlan {
        program: Rc::clone(&program),
        budget: StepBudget {
            scratch_elements: 1,
            ..execution_budget()
        },
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("tuple owner address and result operand need separate slots"),
    };
    let stop = failure.resource.unwrap();
    assert_eq!(stop.resource, StepResource::ScratchElements);
    assert_eq!((stop.limit, stop.spent, stop.requested), (Some(1), 1, 1));
    let monitor = build(BuildPlan {
        program,
        budget: StepBudget {
            scratch_elements: 2,
            ..execution_budget()
        },
    })
    .unwrap();
    assert_now(&monitor, 99);
}

#[trace("TC-098", "IT-018-SC-08", "interface-006-AC-3")]
#[test]
fn tuple_deep_projections_use_explicit_continuations_without_subtree_copies() {
    let _ = key("tuple-init");
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let depth = 2_000;
            let (mut input, _) = tuple_parts(0);
            let tuple_type = ValueType::Composite(key("bool-pair"));
            let tuple = input
                .types
                .tuple(
                    key("bool-pair"),
                    vec![Value::Boolean(true), Value::Boolean(false)],
                )
                .unwrap();
            let literal = lexical_expression(
                &mut input,
                "tuple-init",
                tuple_type.clone(),
                Operation::Literal(tuple),
            );
            let mut previous = lexical_expression(
                &mut input,
                "tuple-init",
                ValueType::Boolean,
                Operation::Literal(Value::Boolean(true)),
            );
            for _ in 0..depth {
                let selected = lexical_expression(
                    &mut input,
                    "tuple-init",
                    tuple_type.clone(),
                    Operation::If {
                        condition: previous,
                        when_true: literal,
                        when_false: literal,
                    },
                );
                previous = lexical_expression(
                    &mut input,
                    "tuple-init",
                    ValueType::Boolean,
                    Operation::TupleElement {
                        value: selected,
                        position: 0,
                    },
                );
            }
            let state = input.layout.state.value_type.clone();
            let root = lexical_expression(
                &mut input,
                "tuple-init",
                state,
                Operation::If {
                    condition: previous,
                    when_true: ExpressionId(0),
                    when_false: ExpressionId(0),
                },
            );
            input.functions[0].body = root;
            let program = Program::try_new(input).unwrap();
            let stopped = match build(BuildPlan {
                program: Rc::clone(&program),
                budget: StepBudget {
                    scratch_elements: depth,
                    ..execution_budget()
                },
            }) {
                Err(failure) => failure,
                Ok(_) => panic!("each retained projection plus its result must be charged"),
            };
            assert_eq!(
                stopped.resource.unwrap().resource,
                StepResource::ScratchElements
            );
            let monitor = build(BuildPlan {
                program,
                budget: StepBudget {
                    scratch_elements: depth + 1,
                    ..execution_budget()
                },
            })
            .unwrap();
            assert_now(&monitor, 0);
            let Operation::Literal(Value::Composite(owner)) =
                &monitor.program().expression(literal).unwrap().operation
            else {
                panic!("tuple literal");
            };
            assert_eq!(
                std::sync::Arc::strong_count(owner),
                1,
                "deep temporary projection owners all released"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

fn record_parts(member: &str) -> (ProgramParts, ExpressionId) {
    let mut input = parts();
    input.functions[0].key = key("record-init");
    input.expressions[0].key = key("record-init");
    let holder = input
        .types
        .record(
            key("state-holder"),
            vec![
                ("right_state", FieldValue::Present(state_at(&input, 99))),
                ("left_state", FieldValue::Present(state_at(&input, 11))),
            ],
        )
        .unwrap();
    let literal = lexical_expression(
        &mut input,
        "record-init",
        ValueType::Composite(key("state-holder")),
        Operation::Literal(holder),
    );
    let state = input.layout.state.value_type.clone();
    let root = lexical_expression(
        &mut input,
        "record-init",
        state,
        Operation::RecordField {
            value: literal,
            member: MemberId::new(MemberOwner::Node(key("state-holder")), member).unwrap(),
        },
    );
    input.functions[0].body = root;
    (input, literal)
}

#[trace("TC-098", "IT-018-SC-09", "interface-007-AC-1")]
#[test]
fn record_members_retain_declared_identity_and_original_value_owners() {
    for (member, expected) in [("left_state", 11), ("right_state", 99)] {
        let (input, literal) = record_parts(member);
        let monitor = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap();
        assert_now(&monitor, expected);
        let Operation::Literal(Value::Composite(owner)) =
            &monitor.program().expression(literal).unwrap().operation
        else {
            panic!("retained holder");
        };
        assert_eq!(std::sync::Arc::strong_count(owner), 1);
        let Value::Composite(state) = monitor.state() else {
            panic!("selected state");
        };
        assert_eq!(std::sync::Arc::strong_count(state), 2);
    }
}

#[trace("TC-098", "IT-018-SC-09", "interface-007-AC-2")]
#[test]
fn record_members_refuse_wrong_owner_missing_member_tuple_and_wrong_result() {
    for case in 0..4 {
        let (mut input, literal) = record_parts("left_state");
        let root = input.functions[0].body;
        match case {
            0 => {
                input.expressions[root.0].operation = Operation::RecordField {
                    value: literal,
                    member: MemberId::new(MemberOwner::Node(key("state")), "left_state").unwrap(),
                }
            }
            1 => {
                input.expressions[root.0].operation = Operation::RecordField {
                    value: literal,
                    member: MemberId::new(MemberOwner::Node(key("state-holder")), "missing")
                        .unwrap(),
                }
            }
            2 => {
                input.expressions[literal.0].value_type = ValueType::Composite(key("state-pair"));
                input.expressions[literal.0].operation = Operation::Literal(
                    input
                        .types
                        .tuple(
                            key("state-pair"),
                            vec![state_at(&input, 0), state_at(&input, 1)],
                        )
                        .unwrap(),
                );
                input.expressions[root.0].operation = Operation::RecordField {
                    value: literal,
                    member: MemberId::new(MemberOwner::Node(key("state-pair")), "left_state")
                        .unwrap(),
                };
            }
            _ => input.expressions[root.0].value_type = ValueType::Boolean,
        }
        let failure = Program::try_new(input)
            .err()
            .expect("invalid member must refuse");
        assert_eq!(
            failure.code,
            if case < 2 {
                ProgramRefusalCode::InvalidReference
            } else {
                ProgramRefusalCode::TypeMismatch
            }
        );
    }
}

#[trace("TC-098", "IT-018-SC-09", "interface-007-AC-3")]
#[test]
fn record_member_addresses_consume_scratch_before_returning_state() {
    let (input, _) = record_parts("left_state");
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scratch_elements = 1;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("projection and result require separate slots");
    let stop = failure.resource.unwrap();
    assert_eq!(stop.resource, StepResource::ScratchElements);
    assert_eq!((stop.spent, stop.requested), (1, 1));
    budget.scratch_elements = 2;
    let monitor = build(BuildPlan { program, budget }).unwrap();
    assert_now(&monitor, 11);
}

#[trace("TC-098", "IT-018-SC-09", "interface-007-AC-2")]
#[test]
fn record_required_read_refuses_optional_presence_without_inventing_an_option_value() {
    let mut input = parts();
    let source = state_at(&input, 0);
    let Value::Composite(state) = source else {
        panic!("state fixture");
    };
    let activations = state.slots().get(2).unwrap().clone();
    let optional = input
        .types
        .record(
            key("state-optional-position"),
            vec![
                ("now", FieldValue::Present(Value::Integer(Integer::zero()))),
                (
                    "history_start",
                    FieldValue::Present(Value::Integer(Integer::zero())),
                ),
                ("activations", activations),
                ("event_position", FieldValue::Absent),
            ],
        )
        .unwrap();
    let literal = lexical_expression(
        &mut input,
        "record-init",
        ValueType::Composite(key("state-optional-position")),
        Operation::Literal(optional),
    );
    lexical_expression(
        &mut input,
        "record-init",
        ValueType::Integer,
        Operation::RecordField {
            value: literal,
            member: MemberId::new(
                MemberOwner::Node(key("state-optional-position")),
                "event_position",
            )
            .unwrap(),
        },
    );
    assert_eq!(
        Program::try_new(input)
            .err()
            .expect("required projection refuses optional field")
            .code,
        ProgramRefusalCode::TypeMismatch
    );
}

#[trace("TC-098", "IT-018-SC-08", "interface-006-AC-1")]
#[test]
fn tuple_dynamic_native_conformance_preserves_position_and_checks_nonidentical_scalar_slots() {
    // Declaration keys/shapes come from QSL/IR. The conformance edges above are
    // controlled native diagnostics, not real-producer temporal qualification.
    let (mut input, literal) = tuple_parts(1);
    input.expressions[literal.0].operation = Operation::Literal(
        input
            .types
            .tuple(
                key("state-pair-child"),
                vec![state_at(&input, 11), state_at(&input, 99)],
            )
            .unwrap(),
    );
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 99);

    let mut input = parts();
    input.functions[0].key = key("tuple-init");
    input.expressions[0].key = key("tuple-init");
    let pair = input
        .types
        .tuple(
            key("bounded-integer-pair"),
            vec![Value::Integer(Integer::from(7_u64)), Value::Boolean(true)],
        )
        .unwrap();
    let literal = lexical_expression(
        &mut input,
        "tuple-init",
        ValueType::Composite(key("integer-pair")),
        Operation::Literal(pair),
    );
    let projected = lexical_expression(
        &mut input,
        "tuple-init",
        ValueType::Integer,
        Operation::TupleElement {
            value: literal,
            position: 0,
        },
    );
    let state = input.layout.state.value_type.clone();
    let root = lexical_expression(
        &mut input,
        "tuple-init",
        state,
        Operation::Let {
            binding: LocalId {
                function: FunctionId(0),
                position: 0,
            },
            value: projected,
            body: ExpressionId(0),
        },
    );
    input.functions[0].body = root;
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.work = 4;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("native scalar slot membership must request work");
    assert!(
        failure.admission.is_some(),
        "original native membership cause is retained"
    );
    assert_eq!(failure.resource.unwrap().resource, StepResource::Work);
    let monitor = build(BuildPlan {
        program,
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 0);
}
