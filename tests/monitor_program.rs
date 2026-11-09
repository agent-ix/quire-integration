// SPDX-License-Identifier: AGPL-3.0-or-later
//! RT construction controls using identities from a live QSL producer and strict IR reader.
//! The controlled executable parts exercise construction, not temporal producer/executor qualification.

use ix_trace_rs::{string_edge, trace};
use qsv_failing_allocator::{arm, disarm, FailingAllocator};
use quire_contract_runtime::exact::CheckCause;
use quire_contract_runtime::monitor::{
    build, Buffer, BuildPlan, Entries, Expression, ExpressionId, Function, FunctionId, Layout,
    LocalId, MonitorFailureCode, Operation, Program, ProgramParts, ProgramRefusalCode, StepBudget,
    StepResource, TypeBinding,
};
use quire_exact::{
    CollectionKind, CollectionType, FieldValue, FloatType, Identifier, IeeeWidth, Integer,
    IntegerInterval, MemberId, MemberOwner, NodeKey, OrderingOperator, Presence, Rational,
    ScalarLimits, TextProfile, TextType, UnionMember, Value, ValueType,
};
use quire_semantic_value::{
    declaration::{
        CompositeDeclaration, CompositeShape, EnvironmentFailure, EqualityOperand,
        EqualityOperator, FieldDeclaration, StorageRequest, StorageUnit, TypeCheckError,
        TypeEnvironment, UnionMemberDeclaration,
    },
    enumeration::EnumMemberIndex,
    quantity::UnitTable,
    unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode},
};
use std::{collections::BTreeMap, rc::Rc};

#[global_allocator]
static ALLOCATOR: FailingAllocator = FailingAllocator;

fn admitted_key(id: &quire_contract_ir::CheckedNodeId) -> NodeKey {
    let structural = admitted()
        .structural_key(id)
        .expect("node ID belongs to the admitted package");
    let wire = serde_json::to_value(structural).expect("structural key serializes");
    qsl_foundation::node_wire::node_key_from_json(&wire).expect("structural key is a NodeKey")
}

fn admitted_key_from_wire(value: &serde_json::Value) -> NodeKey {
    let id: quire_contract_ir::CheckedNodeId =
        serde_json::from_value(value.clone()).expect("admitted node ID decodes");
    admitted_key(&id)
}

fn admitted() -> &'static quire_contract_ir::CheckedPackageV2 {
    static PACKAGE: std::sync::OnceLock<quire_contract_ir::CheckedPackageV2> =
        std::sync::OnceLock::new();
    PACKAGE.get_or_init(|| {
        let emitted = qsl_replay::compile_package(
            qsl_replay::CompileUnit {
                source: qsl_foundation::SourceIdentity::new(
                    "local",
                    "source:monitor-construction.native",
                ),
                package_name: "agent-ix/monitor-construction".to_owned(),
                path: "monitor-construction.native",
                bytes: SOURCE.as_bytes(),
            },
            std::iter::empty::<&[u8]>(),
            &qsl_replay::spine::DependencyInput::default(),
            qsl_replay::StageLimits::default(),
            qsl_replay::ReplayLimits::default(),
        )
        .expect("the lock-selected QSL public compiler emits the authored monitor source");
        let wire: serde_json::Value =
            serde_json::from_slice(emitted.bytes()).expect("QSL emits checked-package JSON");
        let mut evidence = quire_contract_ir::CheckedPackageEvidence::new();
        for feature in wire["lock"]["required_features"]
            .as_array()
            .expect("producer lock feature list")
        {
            evidence.support_feature(feature.as_str().expect("producer feature name"));
        }
        let read = quire_contract_ir::read_checked_package(
            emitted.bytes(),
            quire_contract_ir::CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let quire_contract_ir::CheckedPackageDispatchResult::AdmittedV2(package) = read else {
            panic!("IR refused the emitted monitor source: {read:?}");
        };
        *package
    })
}

fn key(name: &str) -> NodeKey {
    static KEYS: std::sync::OnceLock<BTreeMap<String, NodeKey>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        let package = admitted();
        let mut keys: BTreeMap<String, NodeKey> = IDENTITIES.iter().map(|(label, declaration)| {
            let nodes: Vec<_> = package.resolved_graph().nodes.iter().filter(|node| {
                matches!(&node.owner, Some(quire_contract_ir::CheckedNodeOwner::Source {authority, identity}) if authority.as_ref() == "local" && identity.as_ref() == "source:monitor-construction.native")
                    && node.declaration.as_ref().is_some_and(|value| value.qualified_name.last().is_some_and(|part| part.as_ref() == *declaration))
            }).collect();
            let [node] = nodes.as_slice() else { panic!("one admitted declaration {declaration}, found {}", nodes.len()); };
            let expected_form = match *label {
                "position-event" | "position-watermark" | "position-gap" => "pure_function",
                "Time" | "Length" => "dimension",
                "root-tick" | "tick" | "negative-tick" | "affine-tick" => "unit",
                "state-choice" | "outcome" => "union",
                "capture-mode" => "enum",
                "state-pair-child" | "integer-pair" | "bounded-integer-pair" | "state-pair" | "bool-pair" | "step" | "step-buffered" | "step-missing-position" => "tuple",
                "init-missing-position" | "event-missing-position" | "watermark-missing-position" | "gap-missing-position" | "init-buffered" | "event-buffered" | "watermark-buffered" | "gap-buffered" | "sequence-init" | "fold-init" | "fold-helper" | "union-init" | "equality-init" | "construct-init" | "scalar-init" | "record-init" | "tuple-init" | "boolean-init" | "lexical-init" | "lexical-helper" | "pass-second" | "pass-state" | "option-probe" | "init" | "event" | "watermark" | "gap" | "condition" | "unreached-cycle" | "foreign-argument" | "bad-if" | "clause" => "pure_function",
                _ => "record",
            };
            assert_eq!(node.semantic_form.as_ref(), expected_form, "retain declaration meaning for {declaration}");
            let key = admitted_key(&node.node_id);
            ((*label).to_owned(), key)
        }).collect();
        let step_node = package.resolved_graph().nodes.iter().find(|node| {
            admitted_key(&node.node_id) == keys["step"]
        }).expect("Step declaration retains its admitted tuple body");
        let body = serde_json::to_value(&step_node.body).unwrap();
        assert_eq!(body["term"], "aggregate");
        let results = admitted_key_from_wire(&body["members"][1]["target"]);
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
                .resolved_graph()
                .nodes
                .iter()
                .find(|node| admitted_key(&node.node_id) == unit_key)
                .expect("unit owner remains in admitted graph");
            let body = serde_json::to_value(&node.body).unwrap();
            assert_eq!(body["term"], "unit");
            let carry = |name: &str| admitted_key_from_wire(&body[name]);
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
ordered enum CaptureMode { Idle, Busy }
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
union StateChoice { Chosen(State), Empty, Pair(State, State) }
union Outcome { Failed(GapPayload), False, Incomplete(LimitPayload), Indeterminate, Pending, True }
record Result { instance: Instance; outcome: Outcome; }
type Results = Sequence<Result>;
record StateHolderChild { left_state: State; right_state: State; }
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
function position_event using v(s: State, delta: Integer, captures: Inputs): Step pure { Step(State { now: s.now + delta, history_start: s.history_start, activations: s.activations, event_position: s.event_position + 1 }, sequence[]) }
function position_watermark using v(s: State, boundary: Integer, progress_scope: Scope): Step pure { Step(s, sequence[]) }
function position_gap using v(s: State, delta: Integer): Step pure { Step(State { now: s.now + delta, history_start: s.now + delta, activations: sequence[], event_position: s.event_position }, sequence[]) }
type FoldTotal = Integer;
function typed_sequence using v(s: Sequence<Integer>): Sequence<Integer> pure { s }
function sequence_init using v(): State pure { let xs = typed_sequence(sequence[3, 1, 3]) in if size(xs) > 0 then State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } else State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function fold_init using v(): State pure { let n = fold<FoldTotal>(a, x in typed_sequence(sequence[1, 2, 3]): a * 10 + x, identity: 4) in if n > 0 then State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } else State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function fold_helper using v(flag: Boolean): Integer pure { fold<FoldTotal>(a, x in typed_sequence(sequence[1, 2]): a + x, identity: 10) }
function touch_mode using v(x: CaptureMode): CaptureMode pure { x }
function touch_state_choice using v(x: StateChoice): StateChoice pure { x }
function union_init using v(): State pure { State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } }
function equality_init using v(): State pure { State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } }
function construct_init using v(): State pure { State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } }
function scalar_init using v(): State pure { if 1 + 2 < 4 then State { now: 99, history_start: 0, activations: sequence[], event_position: 0 } else State { now: 0, history_start: 0, activations: sequence[], event_position: 0 } }
function record_init using v(): State pure { let holder = StateHolder { right_state: State { now: 99, history_start: 0, activations: sequence[], event_position: 0 }, left_state: State { now: 11, history_start: 0, activations: sequence[], event_position: 0 } } in holder.left_state }
function touch_holder_child using v(x: StateHolderChild): StateHolderChild pure { x }
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
    ("sequence-init", "sequence_init"),
    ("fold-init", "fold_init"),
    ("fold-helper", "fold_helper"),
    ("union-init", "union_init"),
    ("equality-init", "equality_init"),
    ("construct-init", "construct_init"),
    ("scalar-init", "scalar_init"),
    ("state-holder", "StateHolder"),
    ("state-holder-child", "StateHolderChild"),
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
    ("state-choice", "StateChoice"),
    ("capture-mode", "CaptureMode"),
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
    ("position-event", "position_event"),
    ("position-watermark", "position_watermark"),
    ("position-gap", "position_gap"),
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
                key("state-choice"),
                "state-choice",
                CompositeShape::Union(vec![
                    UnionMemberDeclaration::resolved(
                        Identifier::new("Chosen").unwrap(),
                        vec![ValueType::Composite(key("state"))],
                    ),
                    UnionMemberDeclaration::resolved(Identifier::new("Empty").unwrap(), vec![]),
                    UnionMemberDeclaration::resolved(
                        Identifier::new("Pair").unwrap(),
                        vec![
                            ValueType::Composite(key("state")),
                            ValueType::Composite(key("state")),
                        ],
                    ),
                ]),
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
            // Diagnostic native edge; the producer supplies the two keys and shapes only.
            record(
                "state-holder-child",
                vec![
                    ("left_state", ValueType::Composite(key("state"))),
                    ("right_state", ValueType::Composite(key("state"))),
                ],
            )
            .with_supertypes(vec![key("state-holder")]),
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
            clock_binding: Box::from(&b"diagnostic-clock-value"[..]),
            admitted_order: Box::from(&b"diagnostic-order-value"[..]),
            progress_context: Box::from(&b"diagnostic-progress-value"[..]),
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
        second.layout().clock_binding.as_ref(),
        b"diagnostic-clock-value"
    );
    assert_eq!(
        second.layout().admitted_order.as_ref(),
        b"diagnostic-order-value"
    );
    assert_eq!(
        second.layout().progress_context.as_ref(),
        b"diagnostic-progress-value"
    );
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
        scalar: ScalarLimits {
            integer_bits: 100_000,
            decimal_digits: 100_000,
            scale_expansion: 100_000,
            text_input_bytes: 100_000,
            text_scalars: 100_000,
            normalized_scalars: 100_000,
            unit_edges: 100_000,
            value_occurrences: 100_000,
            work_units: 100_000,
            result_units: 100_000,
        },
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
        assert_eq!(failure.code, MonitorFailureCode::Resource);
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
    assert_eq!(failure.code, MonitorFailureCode::Resource);
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
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
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
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_eq!(failure.resource.unwrap().resource, StepResource::Stack);
    assert!(failure.invariant.is_none());
}

fn membership_probe_parts() -> ProgramParts {
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
    input
}

#[trace("TC-098", "IT-018-SC-05", "interface-003-AC-3")]
#[test]
fn executor_public_membership_work_stop_keeps_native_cause_and_arena() {
    let budget = StepBudget {
        work: 3,
        ..execution_budget()
    };
    let expected = StepResource::Work;
    let failure = match build(BuildPlan {
        program: Program::try_new(membership_probe_parts()).unwrap(),
        budget,
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("native argument admission must stop before invoking the helper"),
    };
    assert_eq!(failure.code, MonitorFailureCode::Resource);
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
            assert_eq!((stop.spent, stop.requested), (3, Integer::one()));
            assert!(native.arena.resolve(locus).is_some());
        }
        _ => panic!("native cause must match the operating stop, without an invariant"),
    }
    assert!(failure.invariant.is_none());
    assert!(failure.path_capacity.is_none());
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
    let input = one_lexical_binding();
    let local = ExpressionId(4);
    let let_body = input.functions[0].body;
    let original_local = (
        input.expressions[local.0].key.clone(),
        input.expressions[local.0].body_path.clone(),
    );
    let original_let = (
        input.expressions[let_body.0].key.clone(),
        input.expressions[let_body.0].body_path.clone(),
    );
    assert_eq!(original_local.0, key("lexical-init"));
    assert_eq!(original_let.0, key("lexical-init"));
    let program = Program::try_new(input).unwrap();
    let retained_local = program.expression(local).unwrap();
    assert_eq!(
        (&retained_local.key, &retained_local.body_path),
        (&original_local.0, &original_local.1)
    );
    assert!(matches!(
        &retained_local.operation,
        Operation::Local(LocalId {
            function: FunctionId(0),
            position: 0
        })
    ));
    let retained_let = program.expression(let_body).unwrap();
    assert_eq!(
        (&retained_let.key, &retained_let.body_path),
        (&original_let.0, &original_let.1)
    );
    assert!(
        matches!(&retained_let.operation, Operation::Let { binding: LocalId { function: FunctionId(0), position: 0 }, body, .. } if *body == local)
    );
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
            assert_eq!(failure.code, MonitorFailureCode::Invariant);
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
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
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
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
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
            assert_eq!(
                (stop.spent, stop.requested),
                (depth as u128, Integer::one())
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
    assert_eq!(
        (stop.limit, stop.spent, stop.requested),
        (Some(4), 4, Integer::one())
    );
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
            assert_eq!(failure.code, MonitorFailureCode::Resource);
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
        assert_eq!(failure.code, MonitorFailureCode::Resource);
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
    let (input, literal) = tuple_parts(1);
    let projection = input.functions[0].body;
    let source = (
        &input.expressions[projection.0].key,
        &input.expressions[projection.0].body_path,
    );
    let source = (source.0.clone(), source.1.clone());
    let program = Program::try_new(input).unwrap();
    let retained = program.expression(projection).unwrap();
    assert_eq!(retained.key, key("tuple-init"));
    assert_eq!((&retained.key, &retained.body_path), (&source.0, &source.1));
    assert!(
        matches!(&retained.operation, Operation::TupleElement { value, position: 1 } if *value == literal)
    );
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
    assert_eq!(
        (stop.limit, stop.spent, stop.requested),
        (Some(1), 1, Integer::one())
    );
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
    let (input, literal) = record_parts("left_state");
    let projection = input.functions[0].body;
    let source = (
        &input.expressions[projection.0].key,
        &input.expressions[projection.0].body_path,
    );
    let source = (source.0.clone(), source.1.clone());
    let program = Program::try_new(input).unwrap();
    let retained = program.expression(projection).unwrap();
    assert_eq!(retained.key, key("record-init"));
    assert_eq!((&retained.key, &retained.body_path), (&source.0, &source.1));
    assert!(
        matches!(&retained.operation, Operation::RecordField { value, member } if *value == literal && member == &MemberId::new(MemberOwner::Node(key("state-holder")), "left_state").unwrap())
    );
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
    assert_eq!((stop.spent, stop.requested), (1, Integer::one()));
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

#[trace("TC-098", "IT-018-SC-09", "interface-007-AC-1", "interface-007-AC-3")]
#[test]
fn record_dynamic_declaration_uses_its_own_order_and_charges_every_inspected_field() {
    for (member, expected, required_work) in [("left_state", 11, 5_u64), ("right_state", 99, 6_u64)]
    {
        let (mut input, literal) = record_parts(member);
        // Native edge is controlled; these are the original source-declared shapes.
        input.expressions[literal.0].operation = Operation::Literal(
            input
                .types
                .record(
                    key("state-holder-child"),
                    vec![
                        ("left_state", FieldValue::Present(state_at(&input, 11))),
                        ("right_state", FieldValue::Present(state_at(&input, 99))),
                    ],
                )
                .unwrap(),
        );
        let program = Program::try_new(input).unwrap();
        let mut budget = execution_budget();
        budget.work = required_work.checked_sub(1).unwrap();
        let failure = build(BuildPlan {
            program: Rc::clone(&program),
            budget,
        })
        .err()
        .expect("last scanned field or return is charged");
        assert!(failure.admission.is_none());
        assert!(failure.invariant.is_none());
        let stop = failure.resource.unwrap();
        assert_eq!(stop.resource, StepResource::Work);
        assert_eq!(
            (stop.spent, stop.requested),
            (
                u128::from(required_work.checked_sub(1).unwrap()),
                Integer::one()
            )
        );
        budget.work = required_work;
        let monitor = build(BuildPlan { program, budget }).unwrap();
        assert_now(&monitor, expected);
    }
    let (input, _) = record_parts("left_state");
    let mut budget = execution_budget();
    budget.work = 4;
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .unwrap();
    assert_now(&monitor, 11);
}

fn scalar_literal(input: &mut ProgramParts, value: &str) -> ExpressionId {
    lexical_expression(
        input,
        "scalar-init",
        ValueType::Integer,
        Operation::Literal(Value::Integer(value.parse().unwrap())),
    )
}

fn scalar_result_parts(case: usize, left: &str, right: &str, expected: &str) -> ProgramParts {
    let mut input = parts();
    input.functions[0].key = key("scalar-init");
    input.expressions[0].key = key("scalar-init");
    let left = scalar_literal(&mut input, left);
    let right = scalar_literal(&mut input, right);
    let operation = match case {
        0 => Operation::IntegerAdd { left, right },
        1 => Operation::IntegerSubtract { left, right },
        2 => Operation::IntegerMultiply { left, right },
        3 => Operation::IntegerNegate { value: left },
        _ => panic!("closed arithmetic cases"),
    };
    let arithmetic = lexical_expression(&mut input, "scalar-init", ValueType::Integer, operation);
    let local = LocalId {
        function: FunctionId(0),
        position: 0,
    };
    let read = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Integer,
        Operation::Local(local),
    );
    let expected = scalar_literal(&mut input, expected);
    let lower = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Boolean,
        Operation::IntegerOrder {
            operator: OrderingOperator::LessOrEqual,
            left: read,
            right: expected,
        },
    );
    let upper = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Boolean,
        Operation::IntegerOrder {
            operator: OrderingOperator::GreaterOrEqual,
            left: read,
            right: expected,
        },
    );
    let equal = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Boolean,
        Operation::BooleanAnd {
            left: lower,
            right: upper,
        },
    );
    let state = input.layout.state.value_type.clone();
    let good = state_at(&input, 99);
    let good = lexical_expression(
        &mut input,
        "scalar-init",
        state.clone(),
        Operation::Literal(good),
    );
    let result = lexical_expression(
        &mut input,
        "scalar-init",
        state.clone(),
        Operation::If {
            condition: equal,
            when_true: good,
            when_false: ExpressionId(0),
        },
    );
    let root = lexical_expression(
        &mut input,
        "scalar-init",
        state,
        Operation::Let {
            binding: local,
            value: arithmetic,
            body: result,
        },
    );
    input.functions[0].body = root;
    input
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-1")]
#[test]
fn integer_arithmetic_uses_exact_independent_values_beyond_host_width() {
    for (operation, left, right, expected) in [
        (
            0,
            "340282366920938463463374607431768211456",
            "1",
            "340282366920938463463374607431768211457",
        ),
        (
            1,
            "-340282366920938463463374607431768211456",
            "1",
            "-340282366920938463463374607431768211457",
        ),
        (
            2,
            "18446744073709551616",
            "18446744073709551616",
            "340282366920938463463374607431768211456",
        ),
        (
            3,
            "340282366920938463463374607431768211456",
            "0",
            "-340282366920938463463374607431768211456",
        ),
    ] {
        let monitor = build(BuildPlan {
            program: Program::try_new(scalar_result_parts(operation, left, right, expected))
                .unwrap(),
            budget: execution_budget(),
        })
        .unwrap();
        assert_now(&monitor, 99);
    }
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-2")]
#[test]
fn integer_native_shortage_retains_exact_charge_and_original_source_without_a_value() {
    let input = scalar_result_parts(0, "1", "2", "3");
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scalar.integer_bits = 0;
    let failure = build(BuildPlan { program, budget })
        .err()
        .expect("native operand charge must stop");
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert!(failure.invariant.is_none());
    let stop = failure.resource.unwrap();
    assert_eq!(
        stop.resource,
        StepResource::Scalar(quire_exact::LimitKind::IntegerBits)
    );
    assert_eq!(
        (stop.limit, stop.spent, stop.requested),
        (Some(0), 0, Integer::from(2_u64))
    );
    let native = failure.scalar.unwrap();
    assert_eq!(native.source_node(), Some(&key("scalar-init")));
    let quire_exact::Outcome::Incomplete(cause) = native.outcome else {
        panic!("original native shortage");
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::IntegerArithmeticOperands
    );
    assert_eq!(cause.next_charge, Integer::from(2_u64));
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-1")]
#[test]
fn integer_order_relations_follow_independent_three_way_truth_rows() {
    for (left, right, expected) in [
        ("-9", "7", [true, true, false, false]),
        (
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211456",
            [false, true, false, true],
        ),
        (
            "340282366920938463463374607431768211457",
            "340282366920938463463374607431768211456",
            [false, false, true, true],
        ),
    ] {
        for (operator, expected) in [
            OrderingOperator::Less,
            OrderingOperator::LessOrEqual,
            OrderingOperator::Greater,
            OrderingOperator::GreaterOrEqual,
        ]
        .into_iter()
        .zip(expected)
        {
            let mut input = scalar_result_parts(0, "0", "0", "0");
            let left = scalar_literal(&mut input, left);
            let right = scalar_literal(&mut input, right);
            let condition = lexical_expression(
                &mut input,
                "scalar-init",
                ValueType::Boolean,
                Operation::IntegerOrder {
                    operator,
                    left,
                    right,
                },
            );
            let state = input.layout.state.value_type.clone();
            let good = state_at(&input, 99);
            let good = lexical_expression(
                &mut input,
                "scalar-init",
                state.clone(),
                Operation::Literal(good),
            );
            let root = lexical_expression(
                &mut input,
                "scalar-init",
                state,
                Operation::If {
                    condition,
                    when_true: good,
                    when_false: ExpressionId(0),
                },
            );
            input.functions[0].body = root;
            let monitor = build(BuildPlan {
                program: Program::try_new(input).unwrap(),
                budget: execution_budget(),
            })
            .unwrap();
            assert_now(&monitor, if expected { 99 } else { 0 });
        }
    }
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-1")]
#[test]
fn integer_bounded_result_refusal_preserves_native_cause_as_an_invariant() {
    let mut input = scalar_result_parts(0, "1", "2", "3");
    let arithmetic = input
        .expressions
        .iter_mut()
        .find(|node| matches!(node.operation, Operation::IntegerAdd { .. }))
        .unwrap();
    arithmetic.value_type =
        ValueType::Int(IntegerInterval::new(Integer::zero(), Integer::from(2_u64)).unwrap());
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .err()
    .expect("3 is outside declared [0,2]");
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
    assert!(failure.resource.is_none());
    assert_eq!(
        failure.invariant.unwrap().code,
        quire_contract_runtime::monitor::ExecutorInvariantCode::Scalar
    );
    let quire_exact::Outcome::Refused(quire_exact::Refusal::IntegerOutOfDomain { target }) =
        failure.scalar.unwrap().outcome
    else {
        panic!("original native domain refusal");
    };
    assert_eq!(
        *target,
        IntegerInterval::new(Integer::zero(), Integer::from(2_u64)).unwrap()
    );
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-2")]
#[test]
fn integer_native_limits_are_cumulative_and_lazy_branches_do_not_charge() {
    let program = Program::try_new(scalar_result_parts(0, "1", "2", "3")).unwrap();
    let mut budget = execution_budget();
    budget.scalar.result_units = 2;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("arithmetic and two selected comparisons need three retained results");
    let stop = failure.resource.unwrap();
    assert_eq!(
        stop.resource,
        StepResource::Scalar(quire_exact::LimitKind::ResultUnits)
    );
    assert_eq!(
        (stop.limit, stop.spent, stop.requested),
        (Some(2), 2, Integer::one())
    );
    let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome else {
        panic!("native cumulative stop");
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::OrderingResultRetain
    );
    budget.scalar.result_units = 4;
    assert_now(&build(BuildPlan { program, budget }).unwrap(), 99);

    let mut input = scalar_result_parts(0, "1", "2", "3");
    let native_branch = input.functions[0].body;
    let condition = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Boolean,
        Operation::Literal(Value::Boolean(false)),
    );
    let state = input.layout.state.value_type.clone();
    let root = lexical_expression(
        &mut input,
        "scalar-init",
        state,
        Operation::If {
            condition,
            when_true: native_branch,
            when_false: ExpressionId(0),
        },
    );
    input.functions[0].body = root;
    let mut budget = execution_budget();
    budget.scalar.integer_bits = 0;
    budget.scalar.work_units = 0;
    budget.scalar.result_units = 0;
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget,
        })
        .unwrap(),
        0,
    );
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-3")]
#[test]
fn integer_computed_canonical_values_consume_explicit_scratch() {
    let program = Program::try_new(scalar_result_parts(3, "1", "0", "-1")).unwrap();
    let mut budget = execution_budget();
    budget.scratch_elements = 1;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("computed canonical integer and result address need separate slots");
    assert!(
        failure.scalar.is_none(),
        "native arithmetic completed; the address reservation stopped"
    );
    assert!(failure.invariant.is_none());
    let stop = failure.resource.unwrap();
    assert_eq!(stop.resource, StepResource::ScratchElements);
    assert_eq!((stop.spent, stop.requested), (1, Integer::one()));
    assert_now(
        &build(BuildPlan {
            program,
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-1")]
#[test]
fn integer_operations_refuse_noninteger_operands_and_wrong_result_annotations() {
    for case in 0..3 {
        let mut input = scalar_result_parts(0, "1", "2", "3");
        let invalid = lexical_expression(
            &mut input,
            "scalar-init",
            ValueType::Boolean,
            Operation::Literal(Value::Boolean(true)),
        );
        let arithmetic = input
            .expressions
            .iter_mut()
            .find(|node| matches!(node.operation, Operation::IntegerAdd { .. }))
            .unwrap();
        match case {
            0 => {
                let Operation::IntegerAdd { left, .. } = &mut arithmetic.operation else {
                    unreachable!()
                };
                *left = invalid;
            }
            1 => arithmetic.value_type = ValueType::Boolean,
            _ => {
                arithmetic.operation = Operation::IntegerOrder {
                    operator: OrderingOperator::Less,
                    left: invalid,
                    right: invalid,
                }
            }
        }
        assert_eq!(
            Program::try_new(input)
                .err()
                .expect("bad scalar annotation must refuse before execution")
                .code,
            ProgramRefusalCode::TypeMismatch
        );
    }
}

#[trace("TC-098", "IT-018-SC-10", "interface-008-AC-2")]
#[test]
fn integer_binary_operands_stop_on_the_left_native_operation_before_the_right() {
    let mut input = scalar_result_parts(0, "1", "2", "3");
    let one = scalar_literal(&mut input, "1");
    let large = scalar_literal(&mut input, "1024");
    let left = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Integer,
        Operation::IntegerNegate { value: one },
    );
    let right = lexical_expression(
        &mut input,
        "scalar-init",
        ValueType::Integer,
        Operation::IntegerNegate { value: large },
    );
    let arithmetic = input
        .expressions
        .iter_mut()
        .find(|node| matches!(node.operation, Operation::IntegerAdd { .. }))
        .unwrap();
    arithmetic.operation = Operation::IntegerAdd { left, right };
    let mut budget = execution_budget();
    budget.scalar.integer_bits = 0;
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .err()
    .expect("left operand's native operation must stop first");
    let native = failure.scalar.unwrap();
    assert_eq!(
        native.site,
        quire_contract_runtime::monitor::ExecutionSite::Expression(left)
    );
    let quire_exact::Outcome::Incomplete(cause) = native.outcome else {
        panic!("left native shortage");
    };
    assert_eq!(cause.next_charge, Integer::one());
}

fn constructed_state_parts() -> (ProgramParts, ExpressionId) {
    let mut input = parts();
    input.functions[0].key = key("construct-init");
    input.expressions[0].key = key("construct-init");
    let now = lexical_expression(
        &mut input,
        "construct-init",
        ValueType::Integer,
        Operation::Literal(Value::Integer(Integer::from(99_u64))),
    );
    let zero = lexical_expression(
        &mut input,
        "construct-init",
        ValueType::Integer,
        Operation::Literal(Value::Integer(Integer::zero())),
    );
    let Operation::Literal(Value::Composite(state)) = &input.expressions[0].operation else {
        panic!("original state literal");
    };
    let FieldValue::Present(activations) = state.slots().get(2).unwrap() else {
        panic!("original activations");
    };
    let activations = activations.clone();
    let Value::Collection(collection) = &activations else {
        panic!("activation sequence");
    };
    let activation_type = ValueType::collection(collection.collection_type().clone());
    let activations = lexical_expression(
        &mut input,
        "construct-init",
        activation_type,
        Operation::Literal(activations),
    );
    let member = |name| MemberId::new(MemberOwner::Node(key("state")), name).unwrap();
    let state = input.layout.state.value_type.clone();
    let root = lexical_expression(
        &mut input,
        "construct-init",
        state,
        Operation::Record {
            fields: vec![
                (member("event_position"), zero),
                (member("activations"), activations),
                (member("history_start"), zero),
                (member("now"), now),
            ],
        },
    );
    input.functions[0].body = root;
    (input, activations)
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-1")]
#[test]
fn constructed_records_and_tuples_keep_canonical_child_owners_and_declared_slots() {
    let (input, activations) = constructed_state_parts();
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 99);
    let Value::Composite(state) = monitor.state() else {
        panic!("canonical constructed state");
    };
    let Some(FieldValue::Present(Value::Collection(result))) = state.slots().get(2) else {
        panic!("canonical activations slot");
    };
    let Operation::Literal(Value::Collection(original)) =
        &monitor.program().expression(activations).unwrap().operation
    else {
        panic!("original sequence");
    };
    assert!(
        std::sync::Arc::ptr_eq(result, original),
        "constructing a record retains the original immutable child"
    );

    let (mut input, pair) = tuple_parts(1);
    let state = input.layout.state.value_type.clone();
    let first = state_at(&input, 11);
    let second = state_at(&input, 99);
    let first = lexical_expression(
        &mut input,
        "tuple-init",
        state.clone(),
        Operation::Literal(first),
    );
    let second = lexical_expression(&mut input, "tuple-init", state, Operation::Literal(second));
    input.expressions[pair.0].operation = Operation::Tuple {
        positions: vec![first, second],
    };
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 99);
    let Value::Composite(result) = monitor.state() else {
        panic!("selected canonical state");
    };
    let Operation::Literal(Value::Composite(original)) =
        &monitor.program().expression(second).unwrap().operation
    else {
        panic!("original state");
    };
    assert!(std::sync::Arc::ptr_eq(result, original));
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-2")]
#[test]
fn record_construction_refuses_missing_duplicate_foreign_unknown_optional_and_wrong_shapes() {
    for case in 0..6 {
        let (mut input, _) = constructed_state_parts();
        let root = input.functions[0].body;
        let Operation::Record { fields } = &mut input.expressions[root.0].operation else {
            panic!("record construction");
        };
        match case {
            0 => {
                fields.pop();
            }
            1 => fields[0].0 = fields[3].0.clone(),
            2 => {
                fields[0].0 =
                    MemberId::new(MemberOwner::Node(key("scope")), "event_position").unwrap()
            }
            3 => fields[0].0 = MemberId::new(MemberOwner::Node(key("state")), "missing").unwrap(),
            4 => {
                for (member, _) in fields.iter_mut() {
                    *member = MemberId::new(
                        MemberOwner::Node(key("state-optional-position")),
                        member.member(),
                    )
                    .unwrap();
                }
                input.expressions[root.0].value_type =
                    ValueType::Composite(key("state-optional-position"));
            }
            _ => input.expressions[root.0].value_type = ValueType::Composite(key("state-pair")),
        }
        let code = Program::try_new(input)
            .err()
            .expect("malformed constructor must not execute")
            .code;
        assert_eq!(
            code,
            if matches!(case, 1..=3) {
                ProgramRefusalCode::InvalidReference
            } else {
                ProgramRefusalCode::TypeMismatch
            }
        );
    }
    for wrong_shape in [false, true] {
        let (mut input, pair) = tuple_parts(0);
        input.expressions[pair.0].operation = Operation::Tuple { positions: vec![] };
        if wrong_shape {
            input.expressions[pair.0].value_type = input.layout.state.value_type.clone();
        }
        assert_eq!(
            Program::try_new(input)
                .err()
                .expect("tuple arity and result shape are checked")
                .code,
            ProgramRefusalCode::TypeMismatch
        );
    }
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-1", "interface-009-AC-3")]
#[test]
fn record_operands_execute_in_declaration_order_before_later_source_fields() {
    let (mut input, _) = constructed_state_parts();
    let one = scalar_literal(&mut input, "1");
    let large = scalar_literal(&mut input, "1024");
    let now = lexical_expression(
        &mut input,
        "construct-init",
        ValueType::Integer,
        Operation::IntegerNegate { value: one },
    );
    let history = lexical_expression(
        &mut input,
        "construct-init",
        ValueType::Integer,
        Operation::IntegerNegate { value: large },
    );
    let root = input.functions[0].body;
    let Operation::Record { fields } = &mut input.expressions[root.0].operation else {
        panic!("record construction");
    };
    for (member, value) in fields {
        match member.member() {
            "now" => *value = now,
            "history_start" => *value = history,
            _ => {}
        }
    }
    let mut budget = execution_budget();
    budget.scalar.integer_bits = 0;
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .err()
    .expect("first declared member stops before the earlier supplied history field");
    let native = failure.scalar.unwrap();
    assert_eq!(
        native.site,
        quire_contract_runtime::monitor::ExecutionSite::Expression(now)
    );
    let quire_exact::Outcome::Incomplete(cause) = native.outcome else {
        panic!("original native shortage");
    };
    assert_eq!(cause.next_charge, Integer::one());
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-3")]
#[test]
fn composite_slot_and_retention_stops_preserve_actual_units_without_a_partial_state() {
    let (input, _) = constructed_state_parts();
    let program = Program::try_new(input).unwrap();
    for case in 0..3 {
        let mut budget = execution_budget();
        match case {
            0 => budget.scratch_elements = 7,
            1 => budget.scratch_bytes = 0,
            _ => budget.scalar.result_units = 4,
        }
        let failure = build(BuildPlan {
            program: Rc::clone(&program),
            budget,
        })
        .err()
        .expect("new slots or native retained value must stop");
        assert_eq!(failure.code, MonitorFailureCode::Resource);
        assert!(failure.invariant.is_none());
        let stop = failure.resource.unwrap();
        match case {
            0 => {
                assert_eq!(stop.resource, StepResource::ScratchElements);
                assert_eq!((stop.spent, stop.requested), (4, Integer::from(4_u64)));
                assert!(failure.scalar.is_none());
            }
            1 => {
                assert_eq!(stop.resource, StepResource::ScratchBytes);
                assert_eq!(
                    stop.requested,
                    Integer::from(core::mem::size_of::<FieldValue>().checked_mul(4).unwrap())
                );
                assert!(failure.scalar.is_none());
            }
            _ => {
                assert_eq!(
                    stop.resource,
                    StepResource::Scalar(quire_exact::LimitKind::ResultUnits)
                );
                assert_eq!((stop.spent, stop.requested), (0, Integer::from(5_u64)));
                let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome
                else {
                    panic!("original composite retention shortage");
                };
                assert_eq!(
                    cause.charge_point,
                    quire_exact::ChargePoint::CompositeResultRetain
                );
            }
        }
    }
    let mut budget = execution_budget();
    budget.scratch_elements = 8;
    assert_now(&build(BuildPlan { program, budget }).unwrap(), 99);
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-3")]
#[test]
fn composite_checks_each_operand_before_evaluating_a_later_recursive_member() {
    for invalid in [false, true] {
        let (mut input, _) = constructed_state_parts();
        let now = if invalid {
            lexical_expression(
                &mut input,
                "construct-init",
                ValueType::Boolean,
                Operation::Literal(Value::Boolean(true)),
            )
        } else {
            lexical_expression(
                &mut input,
                "construct-init",
                ValueType::Int(
                    IntegerInterval::new(Integer::zero(), Integer::from(100_u64)).unwrap(),
                ),
                Operation::Literal(Value::Integer(Integer::from(99_u64))),
            )
        };
        let state = input.layout.state.value_type.clone();
        let later = lexical_expression(
            &mut input,
            "construct-init",
            state,
            Operation::Call {
                function: FunctionId(0),
                arguments: vec![],
            },
        );
        let root = input.functions[0].body;
        let Operation::Record { fields } = &mut input.expressions[root.0].operation else {
            panic!("record construction");
        };
        for (member, value) in fields {
            match member.member() {
                "now" => *value = now,
                "history_start" => *value = later,
                _ => {}
            }
        }
        let program = Program::try_new(input).unwrap();
        let mut budget = execution_budget();
        budget.stack_frames = 32;
        if !invalid {
            budget.work = 3;
        }
        let failure = build(BuildPlan {
            program: Rc::clone(&program),
            budget,
        })
        .err()
        .expect("first member conformance stops before recursion");
        let native = failure.admission.unwrap();
        assert_eq!(
            native.site,
            quire_contract_runtime::monitor::ExecutionSite::Expression(now)
        );
        if invalid {
            assert_eq!(failure.code, MonitorFailureCode::Invariant);
            assert!(failure.resource.is_none());
            assert!(matches!(
                native.failure,
                quire_semantic_value::supplied_membership::SuppliedAdmissionError::Invalid { .. }
            ));
        } else {
            assert_eq!(failure.code, MonitorFailureCode::Resource);
            let stop = failure.resource.unwrap();
            assert_eq!(stop.resource, StepResource::Work);
            assert_eq!((stop.spent, stop.requested), (3, Integer::one()));
            budget.work = 100_000;
            let later_failure = build(BuildPlan { program, budget })
                .err()
                .expect("successful first check reaches the recursive second member");
            assert_eq!(
                later_failure.resource.unwrap().resource,
                StepResource::Stack
            );
        }
    }
}

#[trace("TC-098", "IT-018-SC-11", "interface-009-AC-1", "interface-009-AC-3")]
#[test]
fn deep_composite_construction_uses_continuations_on_a_small_native_stack() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let (mut input, _) = tuple_parts(0);
            let depth = 2_000_usize;
            let state = input.layout.state.value_type.clone();
            let other = state_at(&input, 99);
            let other = lexical_expression(
                &mut input,
                "tuple-init",
                state.clone(),
                Operation::Literal(other),
            );
            let mut selected = ExpressionId(0);
            for _ in 0..depth {
                let pair = lexical_expression(
                    &mut input,
                    "tuple-init",
                    ValueType::Composite(key("state-pair")),
                    Operation::Tuple {
                        positions: vec![selected, other],
                    },
                );
                selected = lexical_expression(
                    &mut input,
                    "tuple-init",
                    state.clone(),
                    Operation::TupleElement {
                        value: pair,
                        position: 0,
                    },
                );
            }
            input.functions[0].body = selected;
            let mut budget = execution_budget();
            budget.stack_frames = depth.checked_mul(8).unwrap();
            budget.scratch_elements = depth.checked_mul(8).unwrap();
            let monitor = build(BuildPlan {
                program: Program::try_new(input).unwrap(),
                budget,
            })
            .unwrap();
            assert_now(&monitor, 0);
        })
        .unwrap()
        .join()
        .unwrap();
}

fn integer_sequence_type() -> ValueType {
    ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Integer,
        None,
    ))
}

fn standard_parts(name: &str) -> ProgramParts {
    let mut input = parts();
    input.functions[0].key = key(name);
    input.expressions[0].key = key(name);
    input
}

fn standard_literal(
    input: &mut ProgramParts,
    name: &str,
    ty: ValueType,
    value: Value,
) -> ExpressionId {
    lexical_expression(input, name, ty, Operation::Literal(value))
}

fn finish_integer_equal(
    input: &mut ProgramParts,
    name: &str,
    actual: ExpressionId,
    expected: &str,
) {
    let expected = standard_literal(
        input,
        name,
        ValueType::Integer,
        Value::Integer(expected.parse().unwrap()),
    );
    let checked = input
        .types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Integer),
            EqualityOperand::typed(ValueType::Integer),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let condition = lexical_expression(
        input,
        name,
        ValueType::Boolean,
        Operation::Equality {
            left: actual,
            right: expected,
            checked: Box::new(checked),
        },
    );
    let state = input.layout.state.value_type.clone();
    let good = state_at(input, 99);
    let good = standard_literal(input, name, state.clone(), good);
    let root = lexical_expression(
        input,
        name,
        state,
        Operation::If {
            condition,
            when_true: good,
            when_false: ExpressionId(0),
        },
    );
    input.functions[0].body = root;
}

fn sequence_literals(input: &mut ProgramParts, name: &str, numbers: &[i64]) -> ExpressionId {
    let elements = numbers
        .iter()
        .map(|number| {
            standard_literal(
                input,
                name,
                ValueType::Integer,
                Value::Integer(Integer::from(*number)),
            )
        })
        .collect();
    lexical_expression(
        input,
        name,
        integer_sequence_type(),
        Operation::Sequence { elements },
    )
}

fn digits_fold(
    input: &mut ProgramParts,
    name: &str,
    sequence: ExpressionId,
    seed: i64,
    slot: usize,
) -> ExpressionId {
    let initial = standard_literal(
        input,
        name,
        ValueType::Integer,
        Value::Integer(Integer::from(seed)),
    );
    let accumulator = LocalId {
        function: FunctionId(0),
        position: slot,
    };
    let item = LocalId {
        function: FunctionId(0),
        position: slot.checked_add(1).unwrap(),
    };
    let acc = lexical_expression(
        input,
        name,
        ValueType::Integer,
        Operation::Local(accumulator),
    );
    let element = lexical_expression(input, name, ValueType::Integer, Operation::Local(item));
    let ten = standard_literal(
        input,
        name,
        ValueType::Integer,
        Value::Integer(Integer::from(10_u64)),
    );
    let product = lexical_expression(
        input,
        name,
        ValueType::Integer,
        Operation::IntegerMultiply {
            left: acc,
            right: ten,
        },
    );
    let body = lexical_expression(
        input,
        name,
        ValueType::Integer,
        Operation::IntegerAdd {
            left: product,
            right: element,
        },
    );
    lexical_expression(
        input,
        name,
        ValueType::Integer,
        Operation::SequenceFold {
            sequence,
            initial,
            accumulator,
            item,
            body,
        },
    )
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-1", "interface-010-AC-2")]
#[test]
fn sequence_construction_size_flatten_and_fold_preserve_duplicate_occurrence_order() {
    for flatten in [false, true] {
        let mut input = standard_parts("sequence-init");
        let sequence = if flatten {
            let first = sequence_literals(&mut input, "sequence-init", &[3, 1]);
            let second = sequence_literals(&mut input, "sequence-init", &[3]);
            let outer = ValueType::collection(CollectionType::new(
                CollectionKind::Sequence,
                integer_sequence_type(),
                None,
            ));
            let outer = lexical_expression(
                &mut input,
                "sequence-init",
                outer,
                Operation::Sequence {
                    elements: vec![first, second],
                },
            );
            lexical_expression(
                &mut input,
                "sequence-init",
                integer_sequence_type(),
                Operation::SequenceFlatten { value: outer },
            )
        } else {
            sequence_literals(&mut input, "sequence-init", &[3, 1, 3])
        };
        let digits = digits_fold(&mut input, "sequence-init", sequence, 0, 0);
        finish_integer_equal(&mut input, "sequence-init", digits, "313");
        assert_now(
            &build(BuildPlan {
                program: Program::try_new(input).unwrap(),
                budget: execution_budget(),
            })
            .unwrap(),
            99,
        );
    }
    let mut input = standard_parts("sequence-init");
    let sequence = sequence_literals(&mut input, "sequence-init", &[3, 1, 3]);
    let size = lexical_expression(
        &mut input,
        "sequence-init",
        ValueType::Integer,
        Operation::SequenceSize { value: sequence },
    );
    finish_integer_equal(&mut input, "sequence-init", size, "3");
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-2")]
#[test]
fn sequence_fold_empty_returns_declared_identity_without_executing_its_body() {
    let mut input = standard_parts("fold-init");
    let sequence = sequence_literals(&mut input, "fold-init", &[]);
    let folded = digits_fold(&mut input, "fold-init", sequence, 47, 0);
    let body = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Local(LocalId {
            function: FunctionId(0),
            position: 1,
        }),
    );
    let Operation::SequenceFold { body: step, .. } = &mut input.expressions[folded.0].operation
    else {
        panic!("fold")
    };
    *step = body;
    finish_integer_equal(&mut input, "fold-init", folded, "47");
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-2")]
#[test]
fn sequence_fold_reads_lexical_captures_without_rebinding_outer_slots() {
    let mut input = standard_parts("fold-init");
    let sequence = sequence_literals(&mut input, "fold-init", &[1, 2, 3]);
    let ten = standard_literal(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Value::Integer(Integer::from(10_u64)),
    );
    let factor = LocalId {
        function: FunctionId(0),
        position: 0,
    };
    let folded = digits_fold(&mut input, "fold-init", sequence, 4, 1);
    let read = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Local(factor),
    );
    let product = input
        .expressions
        .iter_mut()
        .find(|node| matches!(node.operation, Operation::IntegerMultiply { .. }))
        .unwrap();
    let Operation::IntegerMultiply { right, .. } = &mut product.operation else {
        unreachable!()
    };
    *right = read;
    let wrapped = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Let {
            binding: factor,
            value: ten,
            body: folded,
        },
    );
    finish_integer_equal(&mut input, "fold-init", wrapped, "4123");
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

fn verified_union(input: &mut ProgramParts, name: &str) -> Vec<UnionMember> {
    let declaration = input.types.composite(key(name)).unwrap();
    let CompositeShape::Union(members) = declaration.shape() else {
        panic!("original union")
    };
    let bindings = members
        .iter()
        .map(|member| UnionMember::from_admitted(key(name), member.identifier()).unwrap())
        .collect::<Vec<_>>();
    input
        .types
        .seal_union_verified(key(name), key(name), bindings.clone())
        .unwrap();
    bindings
}

fn verified_outcome(input: &mut ProgramParts) -> Vec<UnionMember> {
    verified_union(input, "outcome")
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-1")]
#[test]
fn verified_union_case_and_payload_keep_original_member_and_child_owners() {
    let mut input = standard_parts("union-init");
    let bindings = verified_outcome(&mut input);
    let failed = bindings
        .iter()
        .find(|member| member.identifier() == "Failed")
        .unwrap()
        .variant()
        .clone();
    let payload = input
        .types
        .record(
            key("gap_payload"),
            vec![
                (
                    "gap",
                    FieldValue::Present(Value::Integer(Integer::from(99_u64))),
                ),
                (
                    "max_reading_gap",
                    FieldValue::Present(Value::Integer(Integer::from(100_u64))),
                ),
            ],
        )
        .unwrap();
    let payload = standard_literal(
        &mut input,
        "union-init",
        ValueType::Composite(key("gap_payload")),
        payload,
    );
    let union = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Composite(key("outcome")),
        Operation::Union {
            variant: failed.clone(),
            positions: vec![payload],
        },
    );
    let projected = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Composite(key("gap_payload")),
        Operation::UnionPayload {
            value: union,
            variant: failed.clone(),
            position: 0,
        },
    );
    let gap = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Integer,
        Operation::RecordField {
            value: projected,
            member: MemberId::new(MemberOwner::Node(key("gap_payload")), "gap").unwrap(),
        },
    );
    finish_integer_equal(&mut input, "union-init", gap, "99");
    let state = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&state, 99);

    for positive in [true, false] {
        let mut input = standard_parts("union-init");
        let bindings = verified_outcome(&mut input);
        let pending = bindings
            .iter()
            .find(|member| member.identifier() == "Pending")
            .unwrap()
            .variant()
            .clone();
        let other = bindings
            .iter()
            .find(|member| member.identifier() == "True")
            .unwrap()
            .variant()
            .clone();
        let value = lexical_expression(
            &mut input,
            "union-init",
            ValueType::Composite(key("outcome")),
            Operation::Union {
                variant: pending.clone(),
                positions: vec![],
            },
        );
        let condition = lexical_expression(
            &mut input,
            "union-init",
            ValueType::Boolean,
            Operation::UnionIs {
                value,
                variant: if positive { pending } else { other },
            },
        );
        let state = input.layout.state.value_type.clone();
        let good = state_at(&input, 99);
        let good = standard_literal(&mut input, "union-init", state.clone(), good);
        let root = lexical_expression(
            &mut input,
            "union-init",
            state,
            Operation::If {
                condition,
                when_true: good,
                when_false: ExpressionId(0),
            },
        );
        input.functions[0].body = root;
        let monitor = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap();
        assert_now(&monitor, if positive { 99 } else { 0 });
    }
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-1")]
#[test]
fn union_construction_refuses_unsealed_foreign_and_wrong_payload_bindings() {
    for case in 0..3 {
        let mut input = standard_parts("union-init");
        let bindings = if case == 0 {
            vec![UnionMember::from_admitted(key("outcome"), "Pending").unwrap()]
        } else {
            verified_outcome(&mut input)
        };
        let pending = bindings
            .iter()
            .find(|member| member.identifier() == "Pending")
            .unwrap()
            .variant()
            .clone();
        let variant = if case == 1 {
            quire_exact::VariantId::new(key("state"), "Pending").unwrap()
        } else {
            pending
        };
        let positions = if case == 2 {
            vec![ExpressionId(0)]
        } else {
            vec![]
        };
        lexical_expression(
            &mut input,
            "union-init",
            ValueType::Composite(key("outcome")),
            Operation::Union { variant, positions },
        );
        assert_eq!(
            Program::try_new(input)
                .err()
                .expect("unverified or malformed member cannot execute")
                .code,
            if case == 2 {
                ProgramRefusalCode::TypeMismatch
            } else {
                ProgramRefusalCode::InvalidReference
            }
        );
    }
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-1")]
#[test]
fn union_payload_shares_its_original_state_and_inactive_reads_do_not_invent_values() {
    for guarded in [false, true] {
        let mut input = standard_parts("union-init");
        let members = verified_union(&mut input, "state-choice");
        let state = input.layout.state.value_type.clone();
        let original = state_at(&input, 99);
        let original = standard_literal(&mut input, "union-init", state.clone(), original);
        let chosen = lexical_expression(
            &mut input,
            "union-init",
            ValueType::Composite(key("state-choice")),
            Operation::Union {
                variant: members[0].variant().clone(),
                positions: vec![original],
            },
        );
        let projected = lexical_expression(
            &mut input,
            "union-init",
            state.clone(),
            Operation::UnionPayload {
                value: chosen,
                variant: members[0].variant().clone(),
                position: 0,
            },
        );
        input.functions[0].body = projected;
        let monitor = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap();
        let Value::Composite(result) = monitor.state() else {
            panic!("projected State")
        };
        let Operation::Literal(Value::Composite(source)) =
            &monitor.program().expression(original).unwrap().operation
        else {
            panic!("original State")
        };
        assert!(std::sync::Arc::ptr_eq(result, source));
        assert_eq!(std::sync::Arc::strong_count(result), 2);

        let mut input = standard_parts("union-init");
        input
            .types
            .seal_union_verified(key("state-choice"), key("state-choice"), members.clone())
            .unwrap();
        let empty = lexical_expression(
            &mut input,
            "union-init",
            ValueType::Composite(key("state-choice")),
            Operation::Union {
                variant: members[1].variant().clone(),
                positions: vec![],
            },
        );
        let projected = lexical_expression(
            &mut input,
            "union-init",
            state.clone(),
            Operation::UnionPayload {
                value: empty,
                variant: members[0].variant().clone(),
                position: 0,
            },
        );
        input.functions[0].body = if guarded {
            let condition = lexical_expression(
                &mut input,
                "union-init",
                ValueType::Boolean,
                Operation::UnionIs {
                    value: empty,
                    variant: members[0].variant().clone(),
                },
            );
            lexical_expression(
                &mut input,
                "union-init",
                state.clone(),
                Operation::If {
                    condition,
                    when_true: projected,
                    when_false: ExpressionId(0),
                },
            )
        } else {
            projected
        };
        let result = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        });
        if guarded {
            assert_now(&result.unwrap(), 0)
        } else {
            let failure = result
                .err()
                .expect("inactive payload read is an invariant, never a default State");
            assert_eq!(failure.code, MonitorFailureCode::Invariant);
            assert_eq!(
                failure.invariant.unwrap().code,
                quire_contract_runtime::monitor::ExecutorInvariantCode::Projection
            );
        }
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-2")]
#[test]
fn recursive_sequence_fold_restores_the_callers_accumulator_and_item_slots() {
    let mut input = standard_parts("fold-init");
    let helper = FunctionId(input.functions.len());
    input.functions.push(Function {
        key: key("fold-helper"),
        parameters: vec![ValueType::Boolean],
        result: ValueType::Integer,
        body: ExpressionId(0),
    });
    let sequence = sequence_literals(&mut input, "fold-helper", &[1, 2]);
    let initial = standard_literal(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Value::Integer(Integer::from(10_u64)),
    );
    let accumulator = LocalId {
        function: helper,
        position: 0,
    };
    let item = LocalId {
        function: helper,
        position: 1,
    };
    let acc = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::Local(accumulator),
    );
    let element = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::Local(item),
    );
    let flag = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Boolean,
        Operation::Argument {
            function: helper,
            position: 0,
        },
    );
    let no = standard_literal(
        &mut input,
        "fold-helper",
        ValueType::Boolean,
        Value::Boolean(false),
    );
    let recurse = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::Call {
            function: helper,
            arguments: vec![no],
        },
    );
    let with_inner = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::IntegerAdd {
            left: recurse,
            right: acc,
        },
    );
    let outer = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::IntegerAdd {
            left: with_inner,
            right: element,
        },
    );
    let plain = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::IntegerAdd {
            left: acc,
            right: element,
        },
    );
    let body = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::If {
            condition: flag,
            when_true: outer,
            when_false: plain,
        },
    );
    let folded = lexical_expression(
        &mut input,
        "fold-helper",
        ValueType::Integer,
        Operation::SequenceFold {
            sequence,
            initial,
            accumulator,
            item,
            body,
        },
    );
    input.functions[helper.0].body = folded;
    let yes = standard_literal(
        &mut input,
        "fold-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let result = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Call {
            function: helper,
            arguments: vec![yes],
        },
    );
    finish_integer_equal(&mut input, "fold-init", result, "39");
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-1")]
#[test]
fn sequence_fold_refuses_duplicate_sparse_or_foreign_lexical_addresses() {
    for case in 0..3 {
        let mut input = standard_parts("fold-init");
        let sequence = sequence_literals(&mut input, "fold-init", &[1, 2]);
        let folded = digits_fold(&mut input, "fold-init", sequence, 4, 0);
        let Operation::SequenceFold {
            accumulator, item, ..
        } = &mut input.expressions[folded.0].operation
        else {
            panic!("fold")
        };
        match case {
            0 => *item = *accumulator,
            1 => item.position = 20,
            _ => item.function = FunctionId(1),
        }
        finish_integer_equal(&mut input, "fold-init", folded, "412");
        assert_eq!(
            Program::try_new(input)
                .err()
                .expect("invalid lexical declarations cannot execute")
                .code,
            ProgramRefusalCode::InvalidBinding
        );
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-3")]
#[test]
fn sequence_visit_and_formation_stops_retain_native_causes_without_a_partial_value() {
    let mut input = standard_parts("fold-init");
    let native_sequence = quire_exact::from_admitted(
        CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
        vec![Value::Integer(Integer::one())],
    );
    let sequence = standard_literal(
        &mut input,
        "fold-init",
        integer_sequence_type(),
        native_sequence,
    );
    let folded = digits_fold(&mut input, "fold-init", sequence, 4, 0);
    finish_integer_equal(&mut input, "fold-init", folded, "41");
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scalar.work_units = 0;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("first native visit must stop");
    let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome else {
        panic!("native visit stop")
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::CollectionVisit
    );
    assert_eq!(cause.limit_kind, quire_exact::LimitKind::WorkUnits);
    assert_eq!(cause.next_charge, Integer::one());
    assert_now(
        &build(BuildPlan {
            program,
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );

    let mut input = standard_parts("sequence-init");
    let sequence = sequence_literals(&mut input, "sequence-init", &[1, 2, 3]);
    let size = lexical_expression(
        &mut input,
        "sequence-init",
        ValueType::Integer,
        Operation::SequenceSize { value: sequence },
    );
    finish_integer_equal(&mut input, "sequence-init", size, "3");
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scalar.work_units = 0;
    let failure = build(BuildPlan { program, budget })
        .err()
        .expect("first element charge occurs before evaluating the element");
    let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome else {
        panic!("native element stop")
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::CollectionElement
    );
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-3")]
#[test]
fn collection_result_retention_denials_keep_the_native_charge_and_no_partial_state() {
    for limit_kind in [
        quire_exact::LimitKind::ValueOccurrences,
        quire_exact::LimitKind::ResultUnits,
    ] {
        let mut input = standard_parts("sequence-init");
        let sequence = sequence_literals(&mut input, "sequence-init", &[1]);
        let size = lexical_expression(
            &mut input,
            "sequence-init",
            ValueType::Integer,
            Operation::SequenceSize { value: sequence },
        );
        finish_integer_equal(&mut input, "sequence-init", size, "1");
        let program = Program::try_new(input).unwrap();
        let mut budget = execution_budget();
        match limit_kind {
            quire_exact::LimitKind::ValueOccurrences => budget.scalar.value_occurrences = 1,
            quire_exact::LimitKind::ResultUnits => budget.scalar.result_units = 0,
            _ => unreachable!("two exact retention counters"),
        }
        let failure = build(BuildPlan {
            program: Rc::clone(&program),
            budget,
        })
        .err()
        .expect("the collection retain charge must stop before an initialized State exists");
        assert_eq!(failure.code, MonitorFailureCode::Resource);
        assert!(failure.invariant.is_none());
        assert!(failure.admission.is_none());
        assert_eq!(
            failure.resource.as_ref().unwrap().resource,
            StepResource::Scalar(limit_kind)
        );
        let native = failure.scalar.expect("keep the native collection stop");
        assert_eq!(
            native.site,
            quire_contract_runtime::monitor::ExecutionSite::Expression(sequence)
        );
        let quire_exact::Outcome::Incomplete(cause) = native.outcome else {
            panic!("retain denial has no partial collection value");
        };
        assert_eq!(
            cause.charge_point,
            quire_exact::ChargePoint::CollectionResultRetain
        );
        assert_eq!(cause.limit_kind, limit_kind);
        assert_eq!(cause.next_charge, Integer::from(2_u64));
        assert_now(
            &build(BuildPlan {
                program,
                budget: execution_budget(),
            })
            .expect("later full-budget invocation starts clean"),
            99,
        );
    }
}

#[trace("TC-098", "IT-018-SC-07", "interface-005-AC-3", "interface-008-AC-2")]
#[test]
fn short_circuit_boolean_native_retention_stops_without_evaluating_the_skipped_recursion() {
    for selected in [false, true] {
        let (mut input, conjunction) = boolean_parts(BooleanProbe::And, selected, true);
        let helper = FunctionId(input.functions.len());
        let recursive = lexical_expression(
            &mut input,
            "condition",
            ValueType::Boolean,
            Operation::Call {
                function: helper,
                arguments: vec![],
            },
        );
        input.functions.push(Function {
            key: key("condition"),
            parameters: vec![],
            result: ValueType::Boolean,
            body: recursive,
        });
        let Operation::BooleanAnd { right, .. } = &mut input.expressions[conjunction.0].operation
        else {
            panic!("conjunction")
        };
        *right = recursive;
        let mut budget = execution_budget();
        budget.stack_frames = 32;
        budget.scalar.result_units = 0;
        let failure = build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget,
        })
        .err()
        .expect("selected operating stop");
        if selected {
            assert_eq!(failure.resource.unwrap().resource, StepResource::Stack)
        } else {
            assert_eq!(
                failure.resource.unwrap().resource,
                StepResource::Scalar(quire_exact::LimitKind::ResultUnits)
            );
            let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome else {
                panic!("native Boolean retain")
            };
            assert_eq!(
                cause.charge_point,
                quire_exact::ChargePoint::BooleanResultRetain
            );
        }
    }
}

fn finish_boolean_condition(input: &mut ProgramParts, name: &str, condition: ExpressionId) {
    let state = input.layout.state.value_type.clone();
    let good = state_at(input, 99);
    let good = standard_literal(input, name, state.clone(), good);
    let root = lexical_expression(
        input,
        name,
        state,
        Operation::If {
            condition,
            when_true: good,
            when_false: ExpressionId(0),
        },
    );
    input.functions[0].body = root;
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-2")]
#[test]
fn checked_equality_preserves_native_conversion_and_quantity_unit_facts() {
    for equal in [true, false] {
        let mut input = standard_parts("equality-init");
        let bounded =
            ValueType::Int(IntegerInterval::new(Integer::zero(), Integer::from(10_u64)).unwrap());
        let left = standard_literal(
            &mut input,
            "equality-init",
            bounded.clone(),
            Value::Integer(Integer::from(7_u64)),
        );
        let right = standard_literal(
            &mut input,
            "equality-init",
            ValueType::Integer,
            Value::Integer(Integer::from(if equal { 7_u64 } else { 8_u64 })),
        );
        let checked = input
            .types
            .check_equality(
                EqualityOperator::Equal,
                EqualityOperand::converted(bounded, ValueType::Integer),
                EqualityOperand::typed(ValueType::Integer),
                &EnumMemberIndex::default(),
            )
            .unwrap();
        let condition = lexical_expression(
            &mut input,
            "equality-init",
            ValueType::Boolean,
            Operation::Equality {
                left,
                right,
                checked: Box::new(checked),
            },
        );
        finish_boolean_condition(&mut input, "equality-init", condition);
        assert_now(
            &build(BuildPlan {
                program: Program::try_new(input).unwrap(),
                budget: execution_budget(),
            })
            .unwrap(),
            if equal { 99 } else { 0 },
        );
    }
    let mut input = standard_parts("equality-init");
    let tick = input
        .types
        .units()
        .ids()
        .find(|unit| unit.declared_key() == Some(&key("tick")))
        .unwrap()
        .clone();
    let root = input
        .types
        .units()
        .ids()
        .find(|unit| unit.declared_key() == Some(&key("root-tick")))
        .unwrap()
        .clone();
    let left_type = ValueType::Quantity(tick.clone());
    let right_type = ValueType::Quantity(root.clone());
    let left = standard_literal(
        &mut input,
        "equality-init",
        left_type.clone(),
        Value::Quantity(quire_exact::Quantity::new(
            Rational::from_integer(Integer::one()),
            tick,
        )),
    );
    let right = standard_literal(
        &mut input,
        "equality-init",
        right_type.clone(),
        Value::Quantity(quire_exact::Quantity::new(
            Rational::from_integer(Integer::one()),
            root,
        )),
    );
    let checked = input
        .types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::converted(left_type, right_type.clone()),
            EqualityOperand::typed(right_type),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    assert_eq!(
        checked.schedule(),
        quire_semantic_value::declaration::EqualitySchedule::Quantity
    );
    let condition = lexical_expression(
        &mut input,
        "equality-init",
        ValueType::Boolean,
        Operation::Equality {
            left,
            right,
            checked: Box::new(checked),
        },
    );
    finish_boolean_condition(&mut input, "equality-init", condition);
    assert_now(
        &build(BuildPlan {
            program: Program::try_new(input).unwrap(),
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-2")]
#[test]
fn checked_equality_keeps_original_enum_index_after_preparation_drops() {
    let declaration = quire_semantic_value::enumeration::EnumDeclaration::new(
        key("capture-mode"),
        true,
        vec!["Idle".to_owned(), "Busy".to_owned()],
    )
    .unwrap();
    let mut index = EnumMemberIndex::default();
    let mut values = Vec::new();
    for name in ["Idle", "Busy"] {
        let nodes = admitted()
            .resolved_graph()
            .nodes
            .iter()
            .filter(|node| {
                node.semantic_form.as_ref() == "enum_value"
                    && serde_json::to_value(&node.body).unwrap()["value"] == name
            })
            .collect::<Vec<_>>();
        let [node] = nodes.as_slice() else {
            panic!("one original enum member {name}")
        };
        assert_eq!(admitted_key(&node.semantic_type), key("capture-mode"));
        let member = admitted_key(&node.node_id);
        let value = declaration.member(name, member).unwrap();
        index.record(value.clone());
        values.push(value);
    }
    let shape = quire_exact::EnumShape::new(true, values.iter().map(|value| value.variant()));
    let ty = ValueType::Enum(shape.clone());
    let mut programs = Vec::new();
    for same in [false, true] {
        let mut input = standard_parts("equality-init");
        let variant = values[0].variant();
        let first = Value::Enum(quire_exact::EnumMember::new(
            variant.clone(),
            shape.rank(&variant).unwrap(),
        ));
        let variant = values[if same { 0 } else { 1 }].variant();
        let other = Value::Enum(quire_exact::EnumMember::new(
            variant.clone(),
            shape.rank(&variant).unwrap(),
        ));
        let left = standard_literal(&mut input, "equality-init", ty.clone(), first);
        let right = standard_literal(&mut input, "equality-init", ty.clone(), other);
        let checked = input
            .types
            .check_equality(
                EqualityOperator::Equal,
                EqualityOperand::typed(ty.clone()),
                EqualityOperand::typed(ty.clone()),
                &index,
            )
            .unwrap();
        assert_eq!(
            checked.schedule(),
            quire_semantic_value::declaration::EqualitySchedule::Enum
        );
        let condition = lexical_expression(
            &mut input,
            "equality-init",
            ValueType::Boolean,
            Operation::Equality {
                left,
                right,
                checked: Box::new(checked),
            },
        );
        finish_boolean_condition(&mut input, "equality-init", condition);
        programs.push((Program::try_new(input).unwrap(), same));
    }
    drop(index);
    drop(values);
    drop(declaration);
    for (program, same) in programs {
        assert_now(
            &build(BuildPlan {
                program,
                budget: execution_budget(),
            })
            .unwrap(),
            if same { 99 } else { 0 },
        );
    }
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-3")]
#[test]
#[string_edge]
fn checked_equality_and_union_retention_keep_native_operating_shortages() {
    let mut input = standard_parts("equality-init");
    let left = standard_literal(
        &mut input,
        "equality-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let right = standard_literal(
        &mut input,
        "equality-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let checked = input
        .types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Boolean),
            EqualityOperand::typed(ValueType::Boolean),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let equality = lexical_expression(
        &mut input,
        "equality-init",
        ValueType::Boolean,
        Operation::Equality {
            left,
            right,
            checked: Box::new(checked),
        },
    );
    finish_boolean_condition(&mut input, "equality-init", equality);
    let mut budget = execution_budget();
    budget.scalar.value_occurrences = 0;
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .err()
    .expect("native equality must stop before returning a relation");
    assert_eq!(
        failure.resource.unwrap().resource,
        StepResource::Scalar(quire_exact::LimitKind::ValueOccurrences)
    );
    let native = failure.scalar.unwrap();
    assert_eq!(
        native.site,
        quire_contract_runtime::monitor::ExecutionSite::Expression(equality)
    );
    let quire_exact::Outcome::Incomplete(cause) = native.outcome else {
        panic!("original equality shortage")
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::EqualityPlanForm
    );
    assert_eq!(cause.next_charge, Integer::one());

    let mut input = standard_parts("union-init");
    let members = verified_union(&mut input, "state-choice");
    let variant = members
        .iter()
        .find(|member| member.identifier() == "Empty")
        .unwrap()
        .variant()
        .clone();
    let empty = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Composite(key("state-choice")),
        Operation::Union {
            variant: variant.clone(),
            positions: vec![],
        },
    );
    let condition = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Boolean,
        Operation::UnionIs {
            value: empty,
            variant,
        },
    );
    finish_boolean_condition(&mut input, "union-init", condition);
    let mut budget = execution_budget();
    budget.scalar.result_units = 0;
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .err()
    .expect("nullary member still has one native retained occurrence");
    assert_eq!(
        failure.resource.unwrap().resource,
        StepResource::Scalar(quire_exact::LimitKind::ResultUnits)
    );
    let quire_exact::Outcome::Incomplete(cause) = failure.scalar.unwrap().outcome else {
        panic!("original union retention shortage")
    };
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::CompositeResultRetain
    );
    assert_eq!(cause.next_charge, Integer::one());
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-3")]
#[test]
#[string_edge]
fn equality_and_union_scratch_denials_leave_no_partial_initial_state() {
    let mut equality_input = standard_parts("equality-init");
    let left = standard_literal(
        &mut equality_input,
        "equality-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let right = standard_literal(
        &mut equality_input,
        "equality-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let checked = equality_input
        .types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Boolean),
            EqualityOperand::typed(ValueType::Boolean),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let equality = lexical_expression(
        &mut equality_input,
        "equality-init",
        ValueType::Boolean,
        Operation::Equality {
            left,
            right,
            checked: Box::new(checked),
        },
    );
    finish_boolean_condition(&mut equality_input, "equality-init", equality);
    let equality_program = Program::try_new(equality_input).unwrap();
    let failure = build(BuildPlan {
        program: Rc::clone(&equality_program),
        budget: StepBudget {
            scratch_elements: 1,
            ..execution_budget()
        },
    })
    .err()
    .expect("two equality operands cannot fit one scratch address");
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    let stop = failure.resource.expect("exact equality address shortage");
    assert_eq!(stop.resource, StepResource::ScratchElements);
    assert_eq!(
        (stop.limit, stop.spent, stop.requested),
        (Some(1), 1, Integer::one())
    );
    assert!(failure.scalar.is_none());
    assert!(failure.invariant.is_none());
    assert_now(
        &build(BuildPlan {
            program: equality_program,
            budget: execution_budget(),
        })
        .expect("later equality invocation starts clean"),
        99,
    );

    let mut union_input = standard_parts("union-init");
    let members = verified_union(&mut union_input, "state-choice");
    let variant = members
        .iter()
        .find(|member| member.identifier() == "Chosen")
        .unwrap()
        .variant()
        .clone();
    let state = union_input.layout.state.value_type.clone();
    let state_value = state_at(&union_input, 0);
    let payload = standard_literal(&mut union_input, "union-init", state, state_value);
    let union = lexical_expression(
        &mut union_input,
        "union-init",
        ValueType::Composite(key("state-choice")),
        Operation::Union {
            variant: variant.clone(),
            positions: vec![payload],
        },
    );
    let condition = lexical_expression(
        &mut union_input,
        "union-init",
        ValueType::Boolean,
        Operation::UnionIs {
            value: union,
            variant,
        },
    );
    finish_boolean_condition(&mut union_input, "union-init", condition);
    let union_program = Program::try_new(union_input).unwrap();
    let failure = build(BuildPlan {
        program: Rc::clone(&union_program),
        budget: StepBudget {
            scratch_bytes: 0,
            ..execution_budget()
        },
    })
    .err()
    .expect("one union payload cannot fit zero scratch bytes");
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    let stop = failure.resource.expect("exact union byte shortage");
    assert_eq!(stop.resource, StepResource::ScratchBytes);
    assert_eq!(stop.limit, Some(0));
    assert_eq!(stop.spent, 0);
    assert_eq!(
        stop.requested,
        Integer::from(core::mem::size_of::<FieldValue>())
    );
    assert!(failure.scalar.is_none());
    assert!(failure.invariant.is_none());
    assert_now(
        &build(BuildPlan {
            program: union_program,
            budget: execution_budget(),
        })
        .expect("later union invocation starts clean"),
        99,
    );
}

#[trace("TC-098", "IT-018-SC-13", "interface-011-AC-3")]
#[test]
fn union_payloads_stop_in_declaration_order_before_a_later_native_denial() {
    let mut input = standard_parts("union-init");
    let members = verified_union(&mut input, "state-choice");
    let variant = members
        .iter()
        .find(|member| member.identifier() == "Pair")
        .unwrap()
        .variant()
        .clone();
    let state = input.layout.state.value_type.clone();
    let first = lexical_expression(
        &mut input,
        "union-init",
        state.clone(),
        Operation::Call {
            function: FunctionId(0),
            arguments: vec![],
        },
    );
    let literal = standard_literal(
        &mut input,
        "union-init",
        ValueType::Boolean,
        Value::Boolean(true),
    );
    let later = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Boolean,
        Operation::BooleanNot { value: literal },
    );
    let second = lexical_expression(
        &mut input,
        "union-init",
        state.clone(),
        Operation::If {
            condition: later,
            when_true: ExpressionId(0),
            when_false: ExpressionId(0),
        },
    );
    let pair = lexical_expression(
        &mut input,
        "union-init",
        ValueType::Composite(key("state-choice")),
        Operation::Union {
            variant: variant.clone(),
            positions: vec![first, second],
        },
    );
    let root = lexical_expression(
        &mut input,
        "union-init",
        state,
        Operation::UnionPayload {
            value: pair,
            variant,
            position: 0,
        },
    );
    input.functions[0].body = root;
    let mut budget = execution_budget();
    budget.stack_frames = 32;
    budget.scalar.result_units = 0;
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget,
    })
    .err()
    .expect("first payload recursion must precede the later Boolean retention denial");
    assert_eq!(failure.resource.unwrap().resource, StepResource::Stack);
    assert!(failure.scalar.is_none());
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-1")]
#[test]
fn sequence_fold_returning_an_element_keeps_the_original_canonical_owner() {
    let mut input = standard_parts("fold-init");
    let state = input.layout.state.value_type.clone();
    let first = state_at(&input, 11);
    let last = state_at(&input, 99);
    let first = standard_literal(&mut input, "fold-init", state.clone(), first);
    let last = standard_literal(&mut input, "fold-init", state.clone(), last);
    let sequence_type = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        state.clone(),
        None,
    ));
    let sequence = lexical_expression(
        &mut input,
        "fold-init",
        sequence_type,
        Operation::Sequence {
            elements: vec![first, last],
        },
    );
    let accumulator = LocalId {
        function: FunctionId(0),
        position: 0,
    };
    let item = LocalId {
        function: FunctionId(0),
        position: 1,
    };
    let body = lexical_expression(
        &mut input,
        "fold-init",
        state.clone(),
        Operation::Local(item),
    );
    let root = lexical_expression(
        &mut input,
        "fold-init",
        state,
        Operation::SequenceFold {
            sequence,
            initial: ExpressionId(0),
            accumulator,
            item,
            body,
        },
    );
    input.functions[0].body = root;
    let monitor = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .unwrap();
    assert_now(&monitor, 99);
    let Value::Composite(result) = monitor.state() else {
        panic!("canonical State")
    };
    let Operation::Literal(Value::Composite(original)) =
        &monitor.program().expression(last).unwrap().operation
    else {
        panic!("original element")
    };
    assert!(std::sync::Arc::ptr_eq(result, original));
    assert_eq!(std::sync::Arc::strong_count(result), 2);
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-1", "interface-010-AC-3")]
#[test]
fn sequence_kind_and_authored_bound_are_not_reinterpreted_or_replaced_by_scratch_limits() {
    let mut input = standard_parts("sequence-init");
    let elements = vec![standard_literal(
        &mut input,
        "sequence-init",
        ValueType::Integer,
        Value::Integer(Integer::one()),
    )];
    lexical_expression(
        &mut input,
        "sequence-init",
        ValueType::collection(CollectionType::new(
            CollectionKind::OrderedSet,
            ValueType::Integer,
            None,
        )),
        Operation::Sequence { elements },
    );
    assert_eq!(
        Program::try_new(input)
            .err()
            .expect("sequence capability cannot stand for OrderedSet")
            .code,
        ProgramRefusalCode::TypeMismatch
    );
    let mut input = standard_parts("sequence-init");
    let sequence = sequence_literals(&mut input, "sequence-init", &[1, 2, 3]);
    input.expressions[sequence.0].value_type = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Integer,
        Some(quire_exact::CardinalityBound::new(0, 2).unwrap()),
    ));
    let size = lexical_expression(
        &mut input,
        "sequence-init",
        ValueType::Integer,
        Operation::SequenceSize { value: sequence },
    );
    finish_integer_equal(&mut input, "sequence-init", size, "3");
    let failure = build(BuildPlan {
        program: Program::try_new(input).unwrap(),
        budget: execution_budget(),
    })
    .err()
    .expect("native authored cardinality bound must refuse");
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
    assert!(failure.resource.is_none());
    assert!(matches!(
        failure.scalar.unwrap().outcome,
        quire_exact::Outcome::Refused(_)
    ));
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-3")]
#[test]
fn sequence_fold_temporary_addresses_are_bounded_by_explicit_scratch() {
    let mut input = standard_parts("fold-init");
    let source = quire_exact::from_admitted(
        CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
        (0..500).map(|_| Value::Integer(Integer::one())).collect(),
    );
    let sequence = standard_literal(&mut input, "fold-init", integer_sequence_type(), source);
    let initial = standard_literal(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Value::Integer(Integer::zero()),
    );
    let accumulator = LocalId {
        function: FunctionId(0),
        position: 0,
    };
    let item = LocalId {
        function: FunctionId(0),
        position: 1,
    };
    let acc = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Local(accumulator),
    );
    let element = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::Local(item),
    );
    let body = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::IntegerAdd {
            left: acc,
            right: element,
        },
    );
    let folded = lexical_expression(
        &mut input,
        "fold-init",
        ValueType::Integer,
        Operation::SequenceFold {
            sequence,
            initial,
            accumulator,
            item,
            body,
        },
    );
    finish_integer_equal(&mut input, "fold-init", folded, "500");
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scratch_elements = 64;
    let failure = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .err()
    .expect("temporary canonical operands cannot grow beyond the operating limit");
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_eq!(
        failure.resource.unwrap().resource,
        StepResource::ScratchElements
    );
    assert!(failure.invariant.is_none());
    assert_now(
        &build(BuildPlan {
            program,
            budget: execution_budget(),
        })
        .unwrap(),
        99,
    );
}

// These associations and bodies are diagnostic controls, not a driver admission claim.
fn dispatch_context(program: &Program) -> quire_contract_runtime::monitor::InputContext<'_> {
    let layout = program.layout();
    quire_contract_runtime::monitor::InputContext {
        package_lock: &layout.package_lock,
        clause: &layout.clause,
        subject: &layout.subject,
        occurrence: &layout.occurrence,
        clock_binding: &layout.clock_binding,
        counter: &layout.counter,
        tick_unit: &layout.tick_unit,
        counter_width_bits: layout.counter_width_bits,
        profile: &layout.profile,
        admitted_order: &layout.admitted_order,
    }
}

fn dispatch_parts() -> ProgramParts {
    let mut input = parts();
    for (index, owner) in [(1, "event"), (2, "watermark"), (3, "gap")] {
        let now = lexical_expression(
            &mut input,
            owner,
            ValueType::Integer,
            Operation::Argument {
                function: FunctionId(index),
                position: 1,
            },
        );
        let now = if index == 3 {
            let marker = lexical_expression(
                &mut input,
                owner,
                ValueType::Integer,
                Operation::Literal(Value::Integer(Integer::from(1000_u64))),
            );
            lexical_expression(
                &mut input,
                owner,
                ValueType::Integer,
                Operation::IntegerAdd {
                    left: now,
                    right: marker,
                },
            )
        } else {
            now
        };
        let Value::Composite(state) = state_at(&input, 0) else {
            panic!("state");
        };
        let mut fields = Vec::new();
        for (position, name) in ["now", "history_start", "activations", "event_position"]
            .iter()
            .enumerate()
        {
            let value = if position == 0 {
                now
            } else {
                let FieldValue::Present(value) = &state.slots()[position] else {
                    panic!("slot");
                };
                let ty = if position == 2 {
                    let Value::Collection(collection) = value else {
                        panic!("activations");
                    };
                    ValueType::collection(collection.collection_type().clone())
                } else {
                    ValueType::Integer
                };
                lexical_expression(&mut input, owner, ty, Operation::Literal(value.clone()))
            };
            fields.push((
                MemberId::new(MemberOwner::Node(key("state")), *name).unwrap(),
                value,
            ));
        }
        let state_type = input.layout.state.value_type.clone();
        let state = lexical_expression(&mut input, owner, state_type, Operation::Record { fields });
        let Operation::Literal(Value::Composite(step)) = &input.expressions[index].operation else {
            panic!("step");
        };
        let FieldValue::Present(results) = &step.slots()[1] else {
            panic!("results");
        };
        let results = results.clone();
        let results_type = input.layout.results.value_type.clone();
        let results =
            lexical_expression(&mut input, owner, results_type, Operation::Literal(results));
        let step_type = input.layout.step_result.value_type.clone();
        let body = lexical_expression(
            &mut input,
            owner,
            step_type,
            Operation::Tuple {
                positions: vec![state, results],
            },
        );
        input.functions[index].body = body;
    }
    input
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-1", "interface-012-AC-2")]
fn dispatch_poll_and_watermark_do_not_move_event_baseline() {
    use quire_contract_runtime::monitor::{Event, Reading, Watermark};
    let input = dispatch_parts();
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let scope = input
        .types
        .record(
            key("scope"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let context = dispatch_context(&program);
    monitor
        .poll(Reading {
            context,
            value: 65000,
        })
        .unwrap();
    assert_eq!(monitor.baseline(), None);
    monitor
        .event(Event {
            reading: Reading {
                context,
                value: 65534,
            },
            inputs: &captures,
        })
        .unwrap();
    assert_now(&monitor, 0);
    assert_eq!(monitor.baseline(), Some(65534));
    let boundary = Integer::from(31_u64);
    let step = monitor
        .watermark(Watermark {
            context,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .unwrap();
    assert!(Rc::ptr_eq(step.program(), &program));
    assert_now(&monitor, 31);
    assert_eq!(monitor.baseline(), Some(65534));
    monitor
        .event(Event {
            reading: Reading { context, value: 1 },
            inputs: &captures,
        })
        .unwrap();
    assert_now(&monitor, 3);
    assert_eq!(monitor.baseline(), Some(1));
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-1", "interface-012-AC-2")]
fn dispatch_refuses_foreign_associations_ranges_and_regressing_progress_atomically() {
    use quire_contract_runtime::monitor::{Event, InputRefusal, Reading, Watermark};
    let input = dispatch_parts();
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let scope = input
        .types
        .record(
            key("scope"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let context = dispatch_context(&program);
    let mut foreign = context;
    foreign.clock_binding = b"foreign clock";
    let failure = monitor
        .event(Event {
            reading: Reading {
                context: foreign,
                value: 9,
            },
            inputs: &captures,
        })
        .err()
        .unwrap();
    assert_eq!(failure.input, Some(InputRefusal::ClockBinding));
    let invalid = Value::Integer(Integer::zero());
    let failure = monitor
        .event(Event {
            reading: Reading { context, value: 9 },
            inputs: &invalid,
        })
        .err()
        .unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Input);
    assert_eq!(failure.input, Some(InputRefusal::Values));
    let failure = monitor
        .event(Event {
            reading: Reading {
                context,
                value: 65536,
            },
            inputs: &captures,
        })
        .err()
        .unwrap();
    assert_eq!(failure.input, Some(InputRefusal::ReadingRange));
    assert_eq!(monitor.baseline(), None);
    assert_now(&monitor, 0);
    let boundary = Integer::from(31_u64);
    monitor
        .watermark(Watermark {
            context,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .unwrap();
    let previous = Integer::from(30_u64);
    let failure = monitor
        .watermark(Watermark {
            context,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &previous,
            scope: &scope,
        })
        .err()
        .unwrap();
    assert_eq!(failure.input, Some(InputRefusal::ProgressRegression));
    assert_now(&monitor, 31);
    assert_eq!(monitor.baseline(), None);
}

#[test]
#[trace(
    "IT-018-SC-14",
    "interface-012-AC-3",
    "interface-004-AC-4",
    "interface-008-AC-3"
)]
fn dispatch_resource_stop_reuses_the_same_machine_and_preserves_unconsumed_baseline() {
    use quire_contract_runtime::monitor::{Event, Reading};
    let mut input = dispatch_parts();
    let state_type = input.layout.state.value_type.clone();
    let input_type = input.layout.inputs.value_type.clone();
    let step_type = input.layout.step_result.value_type.clone();
    let argument = |input: &mut ProgramParts, position, ty| {
        lexical_expression(
            input,
            "event",
            ty,
            Operation::Argument {
                function: FunctionId(1),
                position,
            },
        )
    };
    let state = argument(&mut input, 0, state_type.clone());
    let delta = argument(&mut input, 1, ValueType::Integer);
    let captures = argument(&mut input, 2, input_type);
    let enabled = lexical_expression(
        &mut input,
        "event",
        ValueType::Boolean,
        Operation::RecordField {
            value: captures,
            member: MemberId::new(MemberOwner::Node(key("inputs")), "enabled").unwrap(),
        },
    );
    let negated = lexical_expression(
        &mut input,
        "event",
        ValueType::Boolean,
        Operation::BooleanNot { value: enabled },
    );
    let enabled = lexical_expression(
        &mut input,
        "event",
        ValueType::Boolean,
        Operation::BooleanNot { value: negated },
    );
    let binding = LocalId {
        function: FunctionId(1),
        position: 0,
    };
    let local = lexical_expression(&mut input, "event", state_type, Operation::Local(binding));
    let recursive = lexical_expression(
        &mut input,
        "event",
        step_type.clone(),
        Operation::Call {
            function: FunctionId(1),
            arguments: vec![local, delta, captures],
        },
    );
    let completed = input.functions[1].body;
    let branch = lexical_expression(
        &mut input,
        "event",
        step_type.clone(),
        Operation::If {
            condition: enabled,
            when_true: recursive,
            when_false: completed,
        },
    );
    let body = lexical_expression(
        &mut input,
        "event",
        step_type,
        Operation::Let {
            binding,
            value: state,
            body: branch,
        },
    );
    input.functions[1].body = body;
    let captures = |enabled| {
        input
            .types
            .record(
                key("inputs"),
                vec![("enabled", FieldValue::Present(Value::Boolean(enabled)))],
            )
            .unwrap()
    };
    let recurse = captures(true);
    let complete = captures(false);
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scalar.result_units = 20;
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .unwrap();
    let context = dispatch_context(&program);
    let failure = monitor
        .event(Event {
            reading: Reading {
                context,
                value: 500,
            },
            inputs: &recurse,
        })
        .err()
        .unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_eq!(
        failure.resource.unwrap().resource,
        StepResource::Scalar(quire_exact::LimitKind::ResultUnits)
    );
    assert_eq!(monitor.baseline(), None);
    assert_now(&monitor, 0);
    // Neither monitor nor machine nor limits is replaced between the stop and retry.
    monitor
        .event(Event {
            reading: Reading {
                context,
                value: 700,
            },
            inputs: &complete,
        })
        .unwrap();
    assert_eq!(monitor.baseline(), Some(700));
    assert_now(&monitor, 0);
    monitor
        .event(Event {
            reading: Reading {
                context,
                value: 703,
            },
            inputs: &complete,
        })
        .unwrap();
    assert_now(&monitor, 3);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-4")]
fn dispatch_gap_selects_the_gap_entry_and_commits_its_reading_as_the_new_baseline() {
    use quire_contract_runtime::monitor::{Event, Reading};
    let mut input = dispatch_parts();
    input.layout.counter_width_bits = 64;
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let context = dispatch_context(&program);
    let event = |value| Event {
        reading: Reading { context, value },
        inputs: &captures,
    };
    monitor.event(event(u64::MAX - 2)).unwrap();
    assert_now(&monitor, 0);
    monitor.event(event(1)).unwrap();
    assert_now(&monitor, 4);
    monitor.event(event(200)).unwrap();
    // Only the diagnostic gap body adds 1000; the unchanged modular gap is 199.
    assert_now(&monitor, 1199);
    assert_eq!(monitor.baseline(), Some(200));
    monitor.event(event(201)).unwrap();
    assert_now(&monitor, 1);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-3", "interface-012-AC-4")]
fn dispatch_gap_resource_stop_keeps_the_previous_reading_and_machine_reusable() {
    use quire_contract_runtime::monitor::{Event, Reading};
    let mut input = dispatch_parts();
    let state_type = input.layout.state.value_type.clone();
    let state = lexical_expression(
        &mut input,
        "gap",
        state_type,
        Operation::Argument {
            function: FunctionId(3),
            position: 0,
        },
    );
    let delta = lexical_expression(
        &mut input,
        "gap",
        ValueType::Integer,
        Operation::Argument {
            function: FunctionId(3),
            position: 1,
        },
    );
    let step_type = input.layout.step_result.value_type.clone();
    let body = lexical_expression(
        &mut input,
        "gap",
        step_type,
        Operation::Call {
            function: FunctionId(3),
            arguments: vec![state, delta],
        },
    );
    input.functions[3].body = body;
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.work = 300;
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .unwrap();
    let context = dispatch_context(&program);
    let event = |value| Event {
        reading: Reading { context, value },
        inputs: &captures,
    };
    monitor.event(event(100)).unwrap();
    let failure = monitor.event(event(500)).err().unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_eq!(monitor.baseline(), Some(100));
    assert_now(&monitor, 0);
    monitor.event(event(105)).unwrap();
    assert_eq!(monitor.baseline(), Some(105));
    assert_now(&monitor, 5);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-3")]
fn dispatch_body_invariant_terminates_without_consuming_the_event() {
    use quire_contract_runtime::monitor::{Event, InputRefusal, Reading};
    let mut input = dispatch_parts();
    let members = verified_union(&mut input, "state-choice");
    let empty = lexical_expression(
        &mut input,
        "event",
        ValueType::Composite(key("state-choice")),
        Operation::Union {
            variant: members[1].variant().clone(),
            positions: vec![],
        },
    );
    let state_type = input.layout.state.value_type.clone();
    let impossible = lexical_expression(
        &mut input,
        "event",
        state_type,
        Operation::UnionPayload {
            value: empty,
            variant: members[0].variant().clone(),
            position: 0,
        },
    );
    let Operation::Literal(Value::Composite(step)) = &input.expressions[1].operation else {
        panic!("step");
    };
    let FieldValue::Present(results) = &step.slots()[1] else {
        panic!("results");
    };
    let results = results.clone();
    let results_type = input.layout.results.value_type.clone();
    let results = lexical_expression(
        &mut input,
        "event",
        results_type,
        Operation::Literal(results),
    );
    let step_type = input.layout.step_result.value_type.clone();
    let body = lexical_expression(
        &mut input,
        "event",
        step_type,
        Operation::Tuple {
            positions: vec![impossible, results],
        },
    );
    input.functions[1].body = body;
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let context = dispatch_context(&program);
    let failure = monitor
        .event(Event {
            reading: Reading { context, value: 10 },
            inputs: &captures,
        })
        .err()
        .unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Invariant);
    assert_eq!(monitor.baseline(), None);
    assert_now(&monitor, 0);
    let failure = monitor
        .event(Event {
            reading: Reading { context, value: 11 },
            inputs: &captures,
        })
        .err()
        .unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Terminated);
    assert_eq!(failure.input, Some(InputRefusal::Terminated));
    let failure = monitor.poll(Reading { context, value: 11 }).err().unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Terminated);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-3")]
fn dispatch_stopped_watermark_preserves_progress_for_same_machine_retry() {
    use quire_contract_runtime::monitor::{InputRefusal, Watermark};
    let mut input = dispatch_parts();
    let scope_type = input.layout.scope.value_type.clone();
    let scope = lexical_expression(
        &mut input,
        "watermark",
        scope_type,
        Operation::Argument {
            function: FunctionId(2),
            position: 2,
        },
    );
    let enabled = lexical_expression(
        &mut input,
        "watermark",
        ValueType::Boolean,
        Operation::RecordField {
            value: scope,
            member: MemberId::new(MemberOwner::Node(key("scope")), "enabled").unwrap(),
        },
    );
    let state_type = input.layout.state.value_type.clone();
    let state = lexical_expression(
        &mut input,
        "watermark",
        state_type,
        Operation::Argument {
            function: FunctionId(2),
            position: 0,
        },
    );
    let boundary = lexical_expression(
        &mut input,
        "watermark",
        ValueType::Integer,
        Operation::Argument {
            function: FunctionId(2),
            position: 1,
        },
    );
    let step_type = input.layout.step_result.value_type.clone();
    let recursive = lexical_expression(
        &mut input,
        "watermark",
        step_type.clone(),
        Operation::Call {
            function: FunctionId(2),
            arguments: vec![state, boundary, scope],
        },
    );
    let completed = input.functions[2].body;
    let body = lexical_expression(
        &mut input,
        "watermark",
        step_type,
        Operation::If {
            condition: enabled,
            when_true: recursive,
            when_false: completed,
        },
    );
    input.functions[2].body = body;
    let scope = |enabled| {
        input
            .types
            .record(
                key("scope"),
                vec![("enabled", FieldValue::Present(Value::Boolean(enabled)))],
            )
            .unwrap()
    };
    let recurse = scope(true);
    let complete = scope(false);
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.work = 300;
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .unwrap();
    let context = dispatch_context(&program);
    let watermark = |boundary, scope| Watermark {
        context,
        progress_context: &program.layout().progress_context,
        assertion: b"diagnostic admitted assertion",
        inclusive_boundary: boundary,
        scope,
    };
    let ten = Integer::from(10_u64);
    monitor.watermark(watermark(&ten, &complete)).unwrap();
    let thirty = Integer::from(30_u64);
    let failure = monitor
        .watermark(watermark(&thirty, &recurse))
        .err()
        .unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_now(&monitor, 10);
    assert_eq!(monitor.baseline(), None);
    let twenty = Integer::from(20_u64);
    monitor.watermark(watermark(&twenty, &complete)).unwrap();
    assert_now(&monitor, 20);
    let nineteen = Integer::from(19_u64);
    let failure = monitor
        .watermark(watermark(&nineteen, &complete))
        .err()
        .unwrap();
    assert_eq!(failure.input, Some(InputRefusal::ProgressRegression));
    assert_now(&monitor, 20);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-3", "interface-010-AC-3")]
fn dispatch_stopped_fold_restores_slots_before_same_machine_retry() {
    use quire_contract_runtime::monitor::{Event, Reading};
    let mut input = dispatch_parts();
    let input_type = input.layout.inputs.value_type.clone();
    let captures = lexical_expression(
        &mut input,
        "event",
        input_type,
        Operation::Argument {
            function: FunctionId(1),
            position: 2,
        },
    );
    let enabled = lexical_expression(
        &mut input,
        "event",
        ValueType::Boolean,
        Operation::RecordField {
            value: captures,
            member: MemberId::new(MemberOwner::Node(key("inputs")), "enabled").unwrap(),
        },
    );
    let long = quire_exact::from_admitted(
        CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
        (0..500).map(|_| Value::Integer(Integer::one())).collect(),
    );
    let short = quire_exact::from_admitted(
        CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
        [1_u64, 2, 3]
            .into_iter()
            .map(|n| Value::Integer(Integer::from(n)))
            .collect(),
    );
    let long = lexical_expression(
        &mut input,
        "event",
        integer_sequence_type(),
        Operation::Literal(long),
    );
    let short = lexical_expression(
        &mut input,
        "event",
        integer_sequence_type(),
        Operation::Literal(short),
    );
    let sequence = lexical_expression(
        &mut input,
        "event",
        integer_sequence_type(),
        Operation::If {
            condition: enabled,
            when_true: long,
            when_false: short,
        },
    );
    let initial = lexical_expression(
        &mut input,
        "event",
        ValueType::Integer,
        Operation::Literal(Value::Integer(Integer::zero())),
    );
    let accumulator = LocalId {
        function: FunctionId(1),
        position: 0,
    };
    let item = LocalId {
        function: FunctionId(1),
        position: 1,
    };
    let acc = lexical_expression(
        &mut input,
        "event",
        ValueType::Integer,
        Operation::Local(accumulator),
    );
    let element = lexical_expression(
        &mut input,
        "event",
        ValueType::Integer,
        Operation::Local(item),
    );
    let body = lexical_expression(
        &mut input,
        "event",
        ValueType::Integer,
        Operation::IntegerAdd {
            left: acc,
            right: element,
        },
    );
    // Replace the diagnostic event's delta expression used as State.now.
    let delta = input
        .expressions
        .iter()
        .position(|expression| {
            matches!(
                expression.operation,
                Operation::Argument {
                    function: FunctionId(1),
                    position: 1
                }
            )
        })
        .unwrap();
    input.expressions[delta].operation = Operation::SequenceFold {
        sequence,
        initial,
        accumulator,
        item,
        body,
    };
    let captures = |enabled| {
        input
            .types
            .record(
                key("inputs"),
                vec![("enabled", FieldValue::Present(Value::Boolean(enabled)))],
            )
            .unwrap()
    };
    let long = captures(true);
    let short = captures(false);
    let program = Program::try_new(input).unwrap();
    let mut budget = execution_budget();
    budget.scratch_elements = 64;
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget,
    })
    .unwrap();
    let context = dispatch_context(&program);
    let event = |value, inputs| Event {
        reading: Reading { context, value },
        inputs,
    };
    let failure = monitor.event(event(10, &long)).err().unwrap();
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    assert_eq!(
        failure.resource.unwrap().resource,
        StepResource::ScratchElements
    );
    assert_eq!(monitor.baseline(), None);
    assert_now(&monitor, 0);
    monitor.event(event(20, &short)).unwrap();
    assert_eq!(monitor.baseline(), Some(20));
    assert_now(&monitor, 6);
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-1", "interface-012-AC-2")]
fn dispatch_checks_each_independent_context_component_before_executing() {
    use quire_contract_runtime::monitor::{Event, InputRefusal, Reading, Watermark};
    let input = dispatch_parts();
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let scope = input
        .types
        .record(
            key("scope"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let original = dispatch_context(&program);
    let foreign_key = key("condition");
    for expected in [
        InputRefusal::Package,
        InputRefusal::Clause,
        InputRefusal::Subject,
        InputRefusal::Occurrence,
        InputRefusal::ClockBinding,
        InputRefusal::Counter,
        InputRefusal::Unit,
        InputRefusal::Width,
        InputRefusal::Profile,
        InputRefusal::Order,
    ] {
        let mut context = original;
        match expected {
            InputRefusal::Package => context.package_lock = b"foreign package",
            InputRefusal::Clause => context.clause = &foreign_key,
            InputRefusal::Subject => context.subject = "foreign subject",
            InputRefusal::Occurrence => context.occurrence = b"foreign occurrence",
            InputRefusal::ClockBinding => context.clock_binding = b"foreign clock",
            InputRefusal::Counter => context.counter = "foreign counter",
            InputRefusal::Unit => context.tick_unit = &foreign_key,
            InputRefusal::Width => context.counter_width_bits = 8,
            InputRefusal::Profile => context.profile = "foreign profile",
            InputRefusal::Order => context.admitted_order = b"foreign order",
            _ => unreachable!("explicit context cases"),
        }
        let failure = monitor
            .event(Event {
                reading: Reading { context, value: 10 },
                inputs: &captures,
            })
            .err()
            .unwrap();
        assert_eq!(failure.input, Some(expected));
        assert_eq!(monitor.baseline(), None);
        assert_now(&monitor, 0);
    }
    let boundary = Integer::from(10_u64);
    let failure = monitor
        .watermark(Watermark {
            context: original,
            progress_context: b"foreign progress",
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .err()
        .unwrap();
    assert_eq!(failure.input, Some(InputRefusal::ProgressContext));
    assert_now(&monitor, 0);
    let invalid_scope = Value::Integer(Integer::zero());
    let failure = monitor
        .watermark(Watermark {
            context: original,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &boundary,
            scope: &invalid_scope,
        })
        .err()
        .expect("a wrong scope value refuses before progress commits");
    assert_eq!(failure.input, Some(InputRefusal::Values));
    assert_now(&monitor, 0);
    let later = Integer::from(5_u64);
    monitor
        .watermark(Watermark {
            context: original,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted assertion",
            inclusive_boundary: &later,
            scope: &scope,
        })
        .expect("the lower valid watermark proves rejected progress was not committed");
    assert_now(&monitor, 5);
    assert_eq!(monitor.baseline(), None);
    monitor
        .event(Event {
            reading: Reading {
                context: original,
                value: 10,
            },
            inputs: &captures,
        })
        .unwrap();
    assert_eq!(monitor.baseline(), Some(10));
}

#[test]
#[trace("IT-018-SC-14", "interface-012-AC-4")]
fn dispatch_results_retain_original_value_and_distinguish_foreign_program_association() {
    use quire_contract_runtime::monitor::{Event, Reading};
    let first = parts();
    let second = parts();
    let captures = first
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let Operation::Literal(Value::Composite(step)) = &first.expressions[1].operation else {
        panic!("step");
    };
    let FieldValue::Present(Value::Collection(original)) = &step.slots()[1] else {
        panic!("Results");
    };
    let original = std::sync::Arc::downgrade(original);
    let first = Program::try_new(first).unwrap();
    let mut second_parts = second;
    second_parts.layout.subject = "foreign selected subject".into();
    let second = Program::try_new(second_parts).unwrap();
    let first_owner = Rc::downgrade(&first);
    let mut first_monitor = build(BuildPlan {
        program: Rc::clone(&first),
        budget: execution_budget(),
    })
    .unwrap();
    let mut second_monitor = build(BuildPlan {
        program: Rc::clone(&second),
        budget: execution_budget(),
    })
    .unwrap();
    let first_step = first_monitor
        .event(Event {
            reading: Reading {
                context: dispatch_context(&first),
                value: 10,
            },
            inputs: &captures,
        })
        .unwrap();
    let second_step = second_monitor
        .event(Event {
            reading: Reading {
                context: dispatch_context(&second),
                value: 10,
            },
            inputs: &captures,
        })
        .unwrap();
    assert!(!Rc::ptr_eq(first_step.program(), second_step.program()));
    assert_eq!(first_step.program().layout().subject, "selected-subject");
    assert_eq!(
        second_step.program().layout().subject,
        "foreign selected subject"
    );
    drop(first_monitor);
    drop(second_monitor);
    drop(first);
    drop(second);
    drop(captures);
    let Value::Collection(retained) = first_step.results() else {
        panic!("Results");
    };
    assert!(std::sync::Arc::ptr_eq(
        retained,
        &original.upgrade().unwrap()
    ));
    assert!(Rc::ptr_eq(
        first_step.program(),
        &first_owner.upgrade().unwrap()
    ));
    drop(first_step);
    assert!(first_owner.upgrade().is_none());
    assert!(original.upgrade().is_none());
}

fn event_position_parts() -> ProgramParts {
    let mut input = parts();
    let Operation::Literal(Value::Composite(initial)) = &input.expressions[0].operation else {
        panic!("source initializer");
    };
    let FieldValue::Present(Value::Collection(activations)) = &initial.slots()[2] else {
        panic!("source activations");
    };
    let activations_type = ValueType::collection(activations.collection_type().clone());
    let empty_activations = Value::Collection(activations.clone());
    let Operation::Literal(Value::Composite(step)) = &input.expressions[1].operation else {
        panic!("source Step");
    };
    let FieldValue::Present(results) = &step.slots()[1] else {
        panic!("source Results");
    };
    let empty_results = results.clone();
    for (index, owner) in [
        (1, "position-event"),
        (2, "position-watermark"),
        (3, "position-gap"),
    ] {
        input.functions[index].key = key(owner);
        let state_type = input.layout.state.value_type.clone();
        let state = lexical_expression(
            &mut input,
            owner,
            state_type.clone(),
            Operation::Argument {
                function: FunctionId(index),
                position: 0,
            },
        );
        let next_state = if index == 2 {
            state
        } else {
            let field = |input: &mut ProgramParts, name| {
                lexical_expression(
                    input,
                    owner,
                    ValueType::Integer,
                    Operation::RecordField {
                        value: state,
                        member: MemberId::new(MemberOwner::Node(key("state")), name).unwrap(),
                    },
                )
            };
            let now = field(&mut input, "now");
            let delta = lexical_expression(
                &mut input,
                owner,
                ValueType::Integer,
                Operation::Argument {
                    function: FunctionId(index),
                    position: 1,
                },
            );
            let now = lexical_expression(
                &mut input,
                owner,
                ValueType::Integer,
                Operation::IntegerAdd {
                    left: now,
                    right: delta,
                },
            );
            let position = field(&mut input, "event_position");
            let (history, activations, position) = if index == 1 {
                let history = field(&mut input, "history_start");
                let activations = lexical_expression(
                    &mut input,
                    owner,
                    activations_type.clone(),
                    Operation::RecordField {
                        value: state,
                        member: MemberId::new(MemberOwner::Node(key("state")), "activations")
                            .unwrap(),
                    },
                );
                let one = lexical_expression(
                    &mut input,
                    owner,
                    ValueType::Integer,
                    Operation::Literal(Value::Integer(Integer::one())),
                );
                let position = lexical_expression(
                    &mut input,
                    owner,
                    ValueType::Integer,
                    Operation::IntegerAdd {
                        left: position,
                        right: one,
                    },
                );
                (history, activations, position)
            } else {
                let activations = lexical_expression(
                    &mut input,
                    owner,
                    activations_type.clone(),
                    Operation::Literal(empty_activations.clone()),
                );
                (now, activations, position)
            };
            let fields = [
                ("now", now),
                ("history_start", history),
                ("activations", activations),
                ("event_position", position),
            ]
            .into_iter()
            .map(|(name, value)| {
                (
                    MemberId::new(MemberOwner::Node(key("state")), name).unwrap(),
                    value,
                )
            })
            .collect();
            lexical_expression(&mut input, owner, state_type, Operation::Record { fields })
        };
        let results_type = input.layout.results.value_type.clone();
        let results = lexical_expression(
            &mut input,
            owner,
            results_type,
            Operation::Literal(empty_results.clone()),
        );
        let step_type = input.layout.step_result.value_type.clone();
        let body = lexical_expression(
            &mut input,
            owner,
            step_type,
            Operation::Tuple {
                positions: vec![next_state, results],
            },
        );
        input.functions[index].body = body;
    }
    input
}

/// Root locations of the checked State argument, its `now` read, the delta
/// argument, and the addition in the authored `position_event` source.
#[string_edge]
fn admitted_position_sources() -> [NodeKey; 4] {
    use quire_contract_ir::CheckedTerm;

    let package = admitted();
    let function_key = key("position-event");
    let function = package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| admitted_key(&node.node_id) == function_key)
        .expect("admitted position_event function");
    let CheckedTerm::Aggregate { members } = &function.body else {
        panic!("admitted function has an aggregate body");
    };
    let parameters = members
        .iter()
        .find_map(|member| match member {
            CheckedTerm::Binding { name, value } if name.as_ref() == "parameters" => {
                Some(value.as_ref())
            }
            _ => None,
        })
        .expect("admitted function has parameters");
    let CheckedTerm::Aggregate { members } = parameters else {
        panic!("admitted parameters are ordered references");
    };
    let [CheckedTerm::Reference { target: state }, CheckedTerm::Reference { target: delta }, ..] =
        members.as_slice()
    else {
        panic!("position_event begins with State and delta parameters");
    };
    assert!(package.term(state).is_some());
    assert!(package.term(delta).is_some());

    let additions: Vec<_> = package
        .resolved_graph()
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            CheckedTerm::Application {
                operation,
                arguments,
                ..
            } if operation.identity() == Some("quire.op.integer.add")
                && matches!(arguments.as_slice(), [_, CheckedTerm::Reference { target }] if target == delta) =>
            {
                Some((node, &arguments[0]))
            }
            _ => None,
        })
        .collect();
    let [(addition, CheckedTerm::Reference { target: now })] = additions.as_slice() else {
        panic!("one admitted s.now + delta node, found {}", additions.len());
    };
    let Some(CheckedTerm::Application {
        operation,
        arguments,
        ..
    }) = package.term(now)
    else {
        panic!("s.now is an admitted field projection");
    };
    let (_, field) = operation
        .projection_field()
        .expect("admitted operation views")
        .expect("field projection");
    assert_eq!(field, "now");
    assert!(matches!(arguments.as_slice(), [CheckedTerm::Reference { target }] if target == state));
    [
        admitted_key(state),
        admitted_key(now),
        admitted_key(delta),
        admitted_key(&addition.node_id),
    ]
}

fn position_expression_ids(input: &ProgramParts, function: FunctionId) -> [ExpressionId; 4] {
    let argument = |position| {
        let found: Vec<_> = input
            .expressions
            .iter()
            .enumerate()
            .filter_map(|(index, expression)| {
                matches!(&expression.operation, Operation::Argument { function: owner, position: slot } if *owner == function && *slot == position)
                    .then_some(ExpressionId(index))
            })
            .collect();
        let [id] = found.as_slice() else {
            panic!("one argument {position} for function {}", function.0);
        };
        *id
    };
    let state = argument(0);
    let delta = argument(1);
    let now_member = MemberId::new(MemberOwner::Node(key("state")), "now").unwrap();
    let projections: Vec<_> = input
        .expressions
        .iter()
        .enumerate()
        .filter_map(|(index, expression)| match &expression.operation {
            Operation::RecordField { value, member }
                if *value == state && member == &now_member =>
            {
                Some(ExpressionId(index))
            }
            _ => None,
        })
        .collect();
    let [now] = projections.as_slice() else {
        panic!("one State.now projection for function {}", function.0);
    };
    let additions: Vec<_> = input
        .expressions
        .iter()
        .enumerate()
        .filter_map(|(index, expression)| match &expression.operation {
            Operation::IntegerAdd { left, right } if *left == *now && *right == delta => {
                Some(ExpressionId(index))
            }
            _ => None,
        })
        .collect();
    let [addition] = additions.as_slice() else {
        panic!("one State.now + delta for function {}", function.0);
    };
    [state, *now, delta, *addition]
}

fn shared_position_parts() -> (ProgramParts, [[ExpressionId; 4]; 2], [NodeKey; 4]) {
    let source = admitted_position_sources();
    let mut input = event_position_parts();
    let ids = [FunctionId(1), FunctionId(3)].map(|function| {
        let ids = position_expression_ids(&input, function);
        for (id, key) in ids.into_iter().zip(&source) {
            let expression = &mut input.expressions[id.0];
            expression.key = key.clone();
            // Each selected location is the root of its admitted graph node.
            expression.body_path.clear();
        }
        ids
    });
    (input, ids, source)
}

#[trace("TC-209", "IT-018-SC-03", "interface-002-AC-5")]
#[test]
fn admitted_position_nodes_keep_equal_locations_across_distinct_function_owners() {
    let (input, ids, source) = shared_position_parts();
    let function_keys = [key("position-event"), key("position-gap")];
    let program =
        Program::try_new(input).expect("distinct derived owners admit shared source nodes");
    for (branch, function) in [FunctionId(1), FunctionId(3)].into_iter().enumerate() {
        assert_eq!(
            program.function(function).unwrap().key,
            function_keys[branch]
        );
        for (id, expected_key) in ids[branch].into_iter().zip(&source) {
            let retained = program.expression(id).unwrap();
            assert_eq!(&retained.key, expected_key);
            assert!(
                retained.body_path.is_empty(),
                "admitted node-body root path"
            );
        }
        assert!(matches!(
            &program.expression(ids[branch][0]).unwrap().operation,
            Operation::Argument { function: owner, position: 0 } if *owner == function
        ));
        assert!(matches!(
            &program.expression(ids[branch][2]).unwrap().operation,
            Operation::Argument { function: owner, position: 1 } if *owner == function
        ));
        assert!(matches!(
            &program.expression(ids[branch][3]).unwrap().operation,
            Operation::IntegerAdd { left, right }
                if *left == ids[branch][1] && *right == ids[branch][2]
        ));
    }
    assert_ne!(
        ids[0], ids[1],
        "separate arena addresses retain equal source locations"
    );
}

#[trace("TC-209", "IT-018-SC-03", "interface-002-AC-6")]
#[test]
fn admitted_position_locations_refuse_duplicate_and_foreign_bindings() {
    let (mut same, ids, source) = shared_position_parts();
    same.expressions.push(Expression {
        key: source[0].clone(),
        body_path: vec![],
        value_type: same.expressions[ids[0][0].0].value_type.clone(),
        operation: Operation::Argument {
            function: FunctionId(1),
            position: 0,
        },
    });
    refused(same, ProgramRefusalCode::DuplicateKey);

    let (mut neutral, _, _) = shared_position_parts();
    for _ in 0..2 {
        neutral.expressions.push(Expression {
            key: key("condition"),
            body_path: vec![],
            value_type: ValueType::Boolean,
            operation: Operation::Literal(Value::Boolean(true)),
        });
    }
    refused(neutral, ProgramRefusalCode::DuplicateKey);

    let (mut neutral_bound, _, _) = shared_position_parts();
    for operation in [
        Operation::Literal(Value::Integer(Integer::zero())),
        Operation::Argument {
            function: FunctionId(1),
            position: 1,
        },
    ] {
        neutral_bound.expressions.push(Expression {
            key: key("condition"),
            body_path: vec![],
            value_type: ValueType::Integer,
            operation,
        });
    }
    refused(neutral_bound, ProgramRefusalCode::DuplicateKey);

    let (mut mixed, ids, _) = shared_position_parts();
    mixed.expressions[ids[0][3].0].operation = Operation::IntegerAdd {
        left: ids[0][1],
        right: ids[1][2],
    };
    let mixed_refusal = Program::try_new(mixed)
        .err()
        .expect("a mixed owner cannot reuse the admitted addition location");
    assert_eq!(mixed_refusal.code, ProgramRefusalCode::DuplicateKey);
    assert_eq!(mixed_refusal.node.as_ref(), Some(&source[3]));

    let mut foreign_argument = event_position_parts();
    let ids = position_expression_ids(&foreign_argument, FunctionId(1));
    foreign_argument.expressions[ids[2].0].key = source[2].clone();
    foreign_argument.expressions[ids[2].0].body_path.clear();
    foreign_argument.expressions[ids[2].0].operation = Operation::Argument {
        function: FunctionId(3),
        position: 1,
    };
    refused(foreign_argument, ProgramRefusalCode::ForeignArgument);

    for mixed_owner in [false, true] {
        let mut foreign_local = event_position_parts();
        let ids = position_expression_ids(&foreign_local, FunctionId(1));
        foreign_local.expressions[ids[2].0].key = source[2].clone();
        foreign_local.expressions[ids[2].0].body_path.clear();
        foreign_local.expressions[ids[2].0].operation = Operation::Local(LocalId {
            function: FunctionId(3),
            position: 1,
        });
        if !mixed_owner {
            foreign_local.expressions[ids[0].0].operation = Operation::Argument {
                function: FunctionId(3),
                position: 0,
            };
        }
        refused(foreign_local, ProgramRefusalCode::InvalidBinding);
    }
}

#[test]
#[trace("IT-018-SC-15", "FR-276-AC-11")]
fn event_position_counts_ingested_events_and_not_watermarks_polls_or_gap_faults() {
    use quire_contract_runtime::monitor::{Event, Reading, Watermark};
    let input = event_position_parts();
    let captures = input
        .types
        .record(
            key("inputs"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let scope = input
        .types
        .record(
            key("scope"),
            vec![("enabled", FieldValue::Present(Value::Boolean(true)))],
        )
        .unwrap();
    let program = Program::try_new(input).unwrap();
    let mut monitor = build(BuildPlan {
        program: Rc::clone(&program),
        budget: execution_budget(),
    })
    .unwrap();
    let context = dispatch_context(&program);
    let event = |value| Event {
        reading: Reading { context, value },
        inputs: &captures,
    };
    let assert_position = |monitor: &quire_contract_runtime::monitor::Monitor, expected: u64| {
        let Value::Composite(state) = monitor.state() else {
            panic!("canonical State");
        };
        assert_eq!(state.slots().len(), 4, "no extra host counter field");
        let FieldValue::Present(Value::Integer(position)) = &state.slots()[3] else {
            panic!("final required exact Integer event_position");
        };
        assert_eq!(position, &Integer::from(expected));
    };
    assert_position(&monitor, 0);
    monitor.event(event(65530)).unwrap();
    assert_position(&monitor, 1);
    assert_now(&monitor, 0);
    let boundary = Integer::from(40_u64);
    monitor
        .watermark(Watermark {
            context,
            progress_context: &program.layout().progress_context,
            assertion: b"diagnostic admitted progress",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .unwrap();
    assert_position(&monitor, 1);
    monitor.poll(Reading { context, value: 7 }).unwrap();
    assert_position(&monitor, 1);
    assert_eq!(monitor.baseline(), Some(65530));
    monitor.event(event(65500)).unwrap();
    assert_position(&monitor, 1);
    assert_now(&monitor, 65506);
    assert_eq!(monitor.baseline(), Some(65500));
    monitor.event(event(65510)).unwrap();
    assert_position(&monitor, 2);
    assert_now(&monitor, 65516);
    assert_eq!(monitor.baseline(), Some(65510));
}

fn denied_owner_type_walk(input: &ProgramParts) -> (StorageRequest, usize) {
    arm(0);
    let result = input.types.try_check_type(&input.layout.state.value_type);
    let denied = disarm();
    let Err(TypeCheckError::Allocation(request)) = result else {
        panic!("the direct owner worklist must refuse its first allocation: {result:?}");
    };
    let bytes = denied.expect("the owner's real worklist allocation was denied");
    assert!(bytes > 0);
    assert_eq!(request.unit(), StorageUnit::AdditionalElements);
    assert_eq!(request.amount(), 1);
    (request, bytes)
}

#[trace("TC-098", "IT-018-SC-16")]
#[test]
fn first_layout_type_allocation_returns_native_membership_then_retries() {
    let input = parts();
    let state_key = input.layout.state.key.clone();
    let (expected, expected_bytes) = denied_owner_type_walk(&input);

    arm(0);
    let result = Program::try_new(input);
    let denied = disarm();
    let Err(refusal) = result else {
        panic!("a denied first Layout type walk must not publish a Program");
    };
    assert_eq!(denied, Some(expected_bytes));
    assert_eq!(refusal.code, ProgramRefusalCode::Membership);
    assert_eq!(refusal.node, Some(state_key.clone()));
    assert_eq!(
        refusal.membership_failure,
        Some(EnvironmentFailure::Allocation(expected))
    );

    let program = Program::try_new(parts()).expect("allocation-enabled reconstruction succeeds");
    assert_eq!(program.layout().state.key, state_key);
}

#[trace("TC-098", "IT-018-SC-16")]
#[test]
fn later_declared_type_walk_denials_keep_native_membership_and_source() {
    // Five Layout owner walks precede the RT scratch reservation. The four
    // function signatures contain twelve owner walks before expression types.
    for (successful, source) in [(6, "init"), (7, "event"), (18, "init")] {
        let input = parts();
        let expected_node = key(source);
        let (expected, expected_bytes) = denied_owner_type_walk(&input);
        arm(successful);
        let result = Program::try_new(input);
        let denied = disarm();
        let Err(refusal) = result else {
            panic!("denied declared-type walk must not publish a Program");
        };
        assert_eq!(
            denied,
            Some(expected_bytes),
            "allocation index {successful}"
        );
        assert_eq!(refusal.code, ProgramRefusalCode::Membership);
        assert_eq!(refusal.node, Some(expected_node));
        assert_eq!(
            refusal.membership_failure,
            Some(EnvironmentFailure::Allocation(expected))
        );
        let program =
            Program::try_new(parts()).expect("allocation-enabled reconstruction succeeds");
        assert_eq!(program.layout().state.key, key("state"));
    }
}

#[trace("TC-098", "IT-018-SC-16")]
#[test]
fn rt_validation_scratch_denial_stays_storage_after_five_layout_walks() {
    let input = parts();
    let root = input.layout.root.clone();
    // Each of the five Layout bindings completes one owner worklist reservation;
    // the next reservation belongs to RT's function-key validation scratch.
    arm(5);
    let result = Program::try_new(input);
    let denied = disarm();
    let Err(refusal) = result else {
        panic!("denied RT validation scratch must not publish a Program");
    };
    assert!(denied.is_some_and(|bytes| bytes > 0));
    assert_eq!(refusal.code, ProgramRefusalCode::Storage);
    assert_eq!(refusal.membership_failure, None);
    assert_eq!(refusal.node, None);

    let program = Program::try_new(parts()).expect("allocation-enabled reconstruction succeeds");
    assert_eq!(program.layout().root, root);
}

fn ieee_type_collection(kind: CollectionKind) -> ValueType {
    ValueType::collection(CollectionType::new(
        kind,
        ValueType::option(ValueType::Float(FloatType::exact(IeeeWidth::Binary64))),
        None,
    ))
}

#[trace("TC-098", "IT-018-SC-16")]
#[test]
fn declared_type_semantics_keep_original_locus_and_layout_precedence() {
    // Time is a genuine admitted dimension identity, not a composite declaration
    // in this environment. The negative control never constructs a fresh key.
    let not_a_composite = ValueType::Composite(key("Time"));
    for (which, value_type) in [
        (0, not_a_composite.clone()),
        (0, ValueType::Population(None)),
        (0, ieee_type_collection(CollectionKind::Set)),
        (1, ieee_type_collection(CollectionKind::Bag)),
        (1, ValueType::Population(None)),
        (2, not_a_composite),
    ] {
        let mut input = parts();
        let expected = match which {
            0 => {
                input.layout.state.value_type = value_type;
                input.layout.state.key.clone()
            }
            1 => {
                input.expressions[0].value_type = value_type;
                input.expressions[0].key.clone()
            }
            2 => {
                let source = key("unreached-cycle");
                input.functions.push(Function {
                    key: source.clone(),
                    parameters: vec![value_type],
                    result: ValueType::Boolean,
                    body: ExpressionId(0),
                });
                source
            }
            _ => unreachable!("the authored cases cover three consumer locations"),
        };
        let Err(refusal) = Program::try_new(input) else {
            panic!("an invalid declared type must not publish a Program");
        };
        assert_eq!(refusal.code, ProgramRefusalCode::TypeMismatch);
        assert_eq!(refusal.node, Some(expected));
        assert_eq!(refusal.membership_failure, None);
    }

    let mut input = parts();
    let root = input.layout.root.clone();
    input.layout.package_lock = Box::new([]);
    input.layout.state.value_type = ValueType::Population(None);
    let Err(refusal) = Program::try_new(input) else {
        panic!("invalid layout premises must precede later declared-type failures");
    };
    assert_eq!(refusal.code, ProgramRefusalCode::InvalidLayout);
    assert_eq!(refusal.node, Some(root));
    assert_eq!(refusal.membership_failure, None);
}

#[trace("TC-098", "IT-018-SC-16")]
#[test]
fn native_type_projection_keeps_allocation_and_capacity_requests_distinct() {
    // These actual owner error values test projection only. They are not
    // observations of allocator denial or impossible capacity exhaustion.
    for request in [
        StorageRequest::new(7, StorageUnit::Bytes),
        StorageRequest::new(19, StorageUnit::AdditionalElements),
    ] {
        for error in [
            TypeCheckError::Allocation(request),
            TypeCheckError::Capacity(request),
        ] {
            let expected = CheckCause::TypeCheck(error.clone());
            arm(0);
            let actual = CheckCause::from(error);
            let denied = disarm();
            assert_eq!(
                denied, None,
                "native request projection allocates no storage"
            );
            assert_eq!(actual, expected);
        }
    }
}

// ---- interface-010 SequenceMap / SequenceFilter over a controlled arena whose
// identities are the QSL-admitted keys above (RT's own tests may not use NodeKey fixtures).

const SEQ_HELPER: FunctionId = FunctionId(4);

fn seq_booleans() -> ValueType {
    ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Boolean,
        None,
    ))
}

fn seq_integers() -> ValueType {
    ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Integer,
        None,
    ))
}

fn seq_budget() -> StepBudget {
    StepBudget {
        work: 100_000,
        scalar: ScalarLimits {
            integer_bits: 100_000,
            decimal_digits: 100_000,
            scale_expansion: 100_000,
            text_input_bytes: 100_000,
            text_scalars: 100_000,
            normalized_scalars: 100_000,
            unit_edges: 100_000,
            value_occurrences: 100_000,
            work_units: 100_000,
            result_units: 100_000,
        },
        stack_frames: 10_000,
        scratch_elements: 10_000,
        scratch_bytes: 1_000_000,
    }
}

struct SeqFx {
    parts: ProgramParts,
    slots: [usize; 5],
}

impl SeqFx {
    fn new() -> Self {
        let mut parts = lexical_base();
        // Execution-only arena: every function reuses the admitted fixture's literals.
        parts.layout.max_reading_gap = 100;
        Self {
            parts,
            slots: [0; 5],
        }
    }

    fn state_with(parts: &ProgramParts, now: u64) -> Value {
        state_at(parts, now)
    }

    fn add(&mut self, value_type: ValueType, operation: Operation) -> ExpressionId {
        let id = ExpressionId(self.parts.expressions.len());
        self.parts.expressions.push(Expression {
            key: key("condition"),
            body_path: vec![id.0 + 100],
            value_type,
            operation,
        });
        id
    }

    fn local(&mut self, function: FunctionId) -> LocalId {
        let position = self.slots[function.0];
        self.slots[function.0] += 1;
        LocalId { function, position }
    }

    fn int(&mut self, value: i128) -> ExpressionId {
        self.add(
            ValueType::Integer,
            Operation::Literal(Value::Integer(Integer::from(value))),
        )
    }

    fn seq(&mut self, values: &[i128]) -> ExpressionId {
        let elements = values.iter().map(|value| self.int(*value)).collect();
        self.add(seq_integers(), Operation::Sequence { elements })
    }

    fn read(&mut self, local: LocalId) -> ExpressionId {
        self.add(ValueType::Integer, Operation::Local(local))
    }

    fn binary(
        &mut self,
        make: fn(ExpressionId, ExpressionId) -> Operation,
        l: ExpressionId,
        r: ExpressionId,
    ) -> ExpressionId {
        self.add(ValueType::Integer, make(l, r))
    }

    fn greater(&mut self, left: ExpressionId, right: ExpressionId) -> ExpressionId {
        self.add(
            ValueType::Boolean,
            Operation::IntegerOrder {
                operator: OrderingOperator::Greater,
                left,
                right,
            },
        )
    }

    /// `item * item + 1`
    fn square_plus_one(&mut self, item: LocalId) -> ExpressionId {
        let (a, b) = (self.read(item), self.read(item));
        let square = self.binary(
            |left, right| Operation::IntegerMultiply { left, right },
            a,
            b,
        );
        let one = self.int(1);
        self.binary(
            |left, right| Operation::IntegerAdd { left, right },
            square,
            one,
        )
    }

    fn map(&mut self, sequence: ExpressionId, item: LocalId, body: ExpressionId) -> ExpressionId {
        self.add(
            seq_integers(),
            Operation::SequenceMap {
                sequence,
                item,
                body,
            },
        )
    }

    fn filter(
        &mut self,
        sequence: ExpressionId,
        item: LocalId,
        predicate: ExpressionId,
    ) -> ExpressionId {
        self.add(
            seq_integers(),
            Operation::SequenceFilter {
                sequence,
                item,
                predicate,
            },
        )
    }

    fn fold(
        &mut self,
        function: FunctionId,
        sequence: ExpressionId,
        seed: i128,
        step: impl FnOnce(&mut Self, LocalId, LocalId) -> ExpressionId,
    ) -> ExpressionId {
        let initial = self.int(seed);
        let accumulator = self.local(function);
        let item = self.local(function);
        let body = step(self, accumulator, item);
        self.add(
            ValueType::Integer,
            Operation::SequenceFold {
                sequence,
                initial,
                accumulator,
                item,
                body,
            },
        )
    }

    /// Left fold into decimal digits: `acc * 10 + item`.
    fn digits(&mut self, sequence: ExpressionId) -> ExpressionId {
        self.fold(FunctionId(0), sequence, 0, |fx, acc, item| {
            let (acc, item) = (fx.read(acc), fx.read(item));
            let ten = fx.int(10);
            let shifted = fx.binary(
                |l, r| Operation::IntegerMultiply { left: l, right: r },
                acc,
                ten,
            );
            fx.binary(
                |l, r| Operation::IntegerAdd { left: l, right: r },
                shifted,
                item,
            )
        })
    }

    /// Install a root that yields `now == 99` only when every pair is equal.
    fn finish(&mut self, pairs: &[(ExpressionId, ExpressionId)]) {
        let state = self.parts.layout.state.value_type.clone();
        let good = Self::state_with(&self.parts, 99);
        let mut root = self.add(state.clone(), Operation::Literal(good));
        for (left, right) in pairs.iter().rev() {
            let checked = self
                .parts
                .types
                .check_equality(
                    EqualityOperator::Equal,
                    EqualityOperand::typed(ValueType::Integer),
                    EqualityOperand::typed(ValueType::Integer),
                    &EnumMemberIndex::default(),
                )
                .unwrap();
            let condition = self.add(
                ValueType::Boolean,
                Operation::Equality {
                    left: *left,
                    right: *right,
                    checked: Box::new(checked),
                },
            );
            root = self.add(
                state.clone(),
                Operation::If {
                    condition,
                    when_true: root,
                    when_false: ExpressionId(0),
                },
            );
        }
        self.parts.functions[0].body = root;
    }

    fn expect(&mut self, actual: ExpressionId, expected: i128) -> (ExpressionId, ExpressionId) {
        (actual, self.int(expected))
    }
}

fn seq_now(monitor: &quire_contract_runtime::monitor::Monitor) -> Integer {
    let Value::Composite(state) = monitor.state() else {
        panic!("state");
    };
    let FieldValue::Present(Value::Integer(now)) = &state.slots()[0] else {
        panic!("now");
    };
    now.clone()
}

fn seq_run(fx: SeqFx) -> Integer {
    seq_now(
        &build(BuildPlan {
            program: Program::try_new(fx.parts).unwrap(),
            budget: seq_budget(),
        })
        .unwrap(),
    )
}

fn seq_refused(fx: SeqFx) -> ProgramRefusalCode {
    Program::try_new(fx.parts)
        .err()
        .expect("construction must refuse")
        .code
}

fn digits_of(values: impl IntoIterator<Item = i128>) -> i128 {
    values.into_iter().fold(0, |acc, value| acc * 10 + value)
}

const SAMPLES: [&[i128]; 5] = [&[], &[7], &[3, 1, 3], &[1, 2, 3, 4, 5, 2, 9], &[2, 2, 2]];

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-4")]
#[test]
fn map_preserves_order_length_and_duplicates_and_equals_the_equivalent_fold() {
    for values in SAMPLES {
        let mut fx = SeqFx::new();
        let sequence = fx.seq(values);
        let item = fx.local(FunctionId(0));
        let body = fx.square_plus_one(item);
        let mapped = fx.map(sequence, item, body);
        let via_map = fx.digits(mapped);

        let source = fx.seq(values);
        let via_fold = fx.fold(FunctionId(0), source, 0, |fx, acc, item| {
            let acc = fx.read(acc);
            let ten = fx.int(10);
            let shifted = fx.binary(
                |left, right| Operation::IntegerMultiply { left, right },
                acc,
                ten,
            );
            let (a, b) = (fx.read(item), fx.read(item));
            let square = fx.binary(
                |left, right| Operation::IntegerMultiply { left, right },
                a,
                b,
            );
            let one = fx.int(1);
            let mapped = fx.binary(
                |left, right| Operation::IntegerAdd { left, right },
                square,
                one,
            );
            fx.binary(
                |left, right| Operation::IntegerAdd { left, right },
                shifted,
                mapped,
            )
        });
        let expected = digits_of(values.iter().map(|value| value * value + 1));
        let literal = fx.expect(via_map, expected);
        fx.finish(&[literal, (via_map, via_fold)]);
        assert_eq!(seq_run(fx), Integer::from(99_u64), "map over {values:?}");
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-4")]
#[test]
fn filter_keeps_true_occurrences_in_order_and_equals_the_equivalent_fold() {
    for values in SAMPLES {
        let mut fx = SeqFx::new();
        let sequence = fx.seq(values);
        let item = fx.local(FunctionId(0));
        let (read, two) = (fx.read(item), fx.int(2));
        let predicate = fx.greater(read, two);
        let kept = fx.filter(sequence, item, predicate);
        let via_filter = fx.digits(kept);

        let source = fx.seq(values);
        let via_fold = fx.fold(FunctionId(0), source, 0, |fx, acc, item| {
            let (read, two) = (fx.read(item), fx.int(2));
            let condition = fx.greater(read, two);
            let (acc_a, item_a, acc_b) = (fx.read(acc), fx.read(item), fx.read(acc));
            let ten = fx.int(10);
            let shifted = fx.binary(
                |left, right| Operation::IntegerMultiply { left, right },
                acc_a,
                ten,
            );
            let appended = fx.binary(
                |left, right| Operation::IntegerAdd { left, right },
                shifted,
                item_a,
            );
            fx.add(
                ValueType::Integer,
                Operation::If {
                    condition,
                    when_true: appended,
                    when_false: acc_b,
                },
            )
        });
        let expected = digits_of(values.iter().copied().filter(|value| *value > 2));
        let literal = fx.expect(via_filter, expected);
        fx.finish(&[literal, (via_filter, via_fold)]);
        assert_eq!(seq_run(fx), Integer::from(99_u64), "filter over {values:?}");
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-4")]
#[test]
fn empty_input_never_runs_the_body_or_predicate() {
    // The body would be a divide-free but ill-addressed read; an empty input must not reach it.
    let mut fx = SeqFx::new();
    let sequence = fx.seq(&[]);
    let item = fx.local(FunctionId(0));
    let body = fx.square_plus_one(item);
    let mapped = fx.map(sequence, item, body);
    let size = fx.add(
        ValueType::Integer,
        Operation::SequenceSize { value: mapped },
    );
    let literal = fx.expect(size, 0);
    fx.finish(&[literal]);
    assert_eq!(seq_run(fx), Integer::from(99_u64));
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-4")]
#[test]
fn nested_item_scopes_capture_and_keep_the_outer_item_after_the_inner_traversal() {
    // outer = map [1, 2] a -> sum(map [3, 4] b -> a + b) + a ; the right operand is read after
    // the inner traversal completes, so a lost outer binding changes the digits.
    let mut fx = SeqFx::new();
    let outer_source = fx.seq(&[1, 2]);
    let a = fx.local(FunctionId(0));
    let inner_source = fx.seq(&[3, 4]);
    let b = fx.local(FunctionId(0));
    let (read_a, read_b) = (fx.read(a), fx.read(b));
    let inner_body = fx.binary(
        |left, right| Operation::IntegerAdd { left, right },
        read_a,
        read_b,
    );
    let inner = fx.map(inner_source, b, inner_body);
    let sum = fx.fold(FunctionId(0), inner, 0, |fx, acc, item| {
        let (acc, item) = (fx.read(acc), fx.read(item));
        fx.binary(
            |left, right| Operation::IntegerAdd { left, right },
            acc,
            item,
        )
    });
    let after = fx.read(a);
    let outer_body = fx.binary(
        |left, right| Operation::IntegerAdd { left, right },
        sum,
        after,
    );
    let outer = fx.map(outer_source, a, outer_body);
    let digits = fx.digits(outer);
    let expected = digits_of([(1 + 3) + (1 + 4) + 1, (2 + 3) + (2 + 4) + 2]);
    let literal = fx.expect(digits, expected);
    fx.finish(&[literal]);
    assert_eq!(seq_run(fx), Integer::from(99_u64));
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-4")]
#[test]
fn recursive_invocation_restores_the_callers_item_binding() {
    // h(d) = sum(map [1, 2] x -> (if d > 0 then h(d - 1) else 0) + x); x is read after the call.
    fn helper(fx: &mut SeqFx) {
        let state_free = fx.parts.layout.state.value_type.clone();
        let _ = state_free;
        let source = fx.seq(&[1, 2]);
        let x = fx.local(SEQ_HELPER);
        let d = fx.add(
            ValueType::Integer,
            Operation::Argument {
                function: SEQ_HELPER,
                position: 0,
            },
        );
        let zero = fx.int(0);
        let condition = fx.greater(d, zero);
        let d_again = fx.add(
            ValueType::Integer,
            Operation::Argument {
                function: SEQ_HELPER,
                position: 0,
            },
        );
        let one = fx.int(1);
        let decremented = fx.binary(
            |left, right| Operation::IntegerSubtract { left, right },
            d_again,
            one,
        );
        let call = fx.add(
            ValueType::Integer,
            Operation::Call {
                function: SEQ_HELPER,
                arguments: vec![decremented],
            },
        );
        let nothing = fx.int(0);
        let below = fx.add(
            ValueType::Integer,
            Operation::If {
                condition,
                when_true: call,
                when_false: nothing,
            },
        );
        let read = fx.read(x);
        let body = fx.binary(
            |left, right| Operation::IntegerAdd { left, right },
            below,
            read,
        );
        let mapped = fx.map(source, x, body);
        let total = fx.fold(SEQ_HELPER, mapped, 0, |fx, acc, item| {
            let (acc, item) = (fx.read(acc), fx.read(item));
            fx.binary(
                |left, right| Operation::IntegerAdd { left, right },
                acc,
                item,
            )
        });
        fx.parts.functions.push(Function {
            key: key("fold-helper"),
            parameters: vec![ValueType::Integer],
            result: ValueType::Integer,
            body: total,
        });
    }
    for (depth, expected) in [(0, 3), (1, 9), (2, 21)] {
        let mut fx = SeqFx::new();
        helper(&mut fx);
        let argument = fx.int(depth);
        let call = fx.add(
            ValueType::Integer,
            Operation::Call {
                function: SEQ_HELPER,
                arguments: vec![argument],
            },
        );
        let literal = fx.expect(call, expected);
        fx.finish(&[literal]);
        assert_eq!(seq_run(fx), Integer::from(99_u64), "depth {depth}");
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn non_sequence_operand_refuses_for_map_and_filter() {
    for filter in [false, true] {
        let mut fx = SeqFx::new();
        let not_a_sequence = fx.int(5);
        let item = fx.local(FunctionId(0));
        let operand = fx.read(item);
        let predicate = fx.greater(operand, not_a_sequence);
        let node = if filter {
            fx.filter(not_a_sequence, item, predicate)
        } else {
            fx.map(not_a_sequence, item, operand)
        };
        let digits = fx.digits(node);
        let literal = fx.expect(digits, 0);
        fx.finish(&[literal]);
        assert_eq!(seq_refused(fx), ProgramRefusalCode::InvalidBinding);
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn non_boolean_predicate_refuses() {
    let mut fx = SeqFx::new();
    let sequence = fx.seq(&[1, 2]);
    let item = fx.local(FunctionId(0));
    let integer_predicate = fx.read(item);
    let kept = fx.filter(sequence, item, integer_predicate);
    let digits = fx.digits(kept);
    let literal = fx.expect(digits, 12);
    fx.finish(&[literal]);
    assert_eq!(seq_refused(fx), ProgramRefusalCode::TypeMismatch);
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn mistyped_result_annotations_refuse() {
    // Map result element type must be the body's type.
    let mut fx = SeqFx::new();
    let sequence = fx.seq(&[1, 2]);
    let item = fx.local(FunctionId(0));
    let body = fx.read(item);
    let mapped = fx.add(
        seq_booleans(),
        Operation::SequenceMap {
            sequence,
            item,
            body,
        },
    );
    let size = fx.add(
        ValueType::Integer,
        Operation::SequenceSize { value: mapped },
    );
    let literal = fx.expect(size, 2);
    fx.finish(&[literal]);
    assert_eq!(seq_refused(fx), ProgramRefusalCode::TypeMismatch);

    // Filter result element type must be the operand's element type.
    let mut fx = SeqFx::new();
    let sequence = fx.seq(&[1, 2]);
    let item = fx.local(FunctionId(0));
    let (read, one) = (fx.read(item), fx.int(1));
    let predicate = fx.greater(read, one);
    let kept = fx.add(
        seq_booleans(),
        Operation::SequenceFilter {
            sequence,
            item,
            predicate,
        },
    );
    let size = fx.add(ValueType::Integer, Operation::SequenceSize { value: kept });
    let literal = fx.expect(size, 1);
    fx.finish(&[literal]);
    assert_eq!(seq_refused(fx), ProgramRefusalCode::TypeMismatch);
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn duplicate_sparse_and_foreign_item_addresses_refuse() {
    for case in 0..3 {
        let mut fx = SeqFx::new();
        let first_source = fx.seq(&[1, 2]);
        let first_item = fx.local(FunctionId(0));
        let first_body = fx.read(first_item);
        let first = fx.map(first_source, first_item, first_body);
        let second_source = fx.seq(&[3]);
        let mut second_item = fx.local(FunctionId(0));
        let second_body = fx.read(second_item);
        match case {
            0 => second_item = first_item,
            1 => second_item.position = 20,
            _ => second_item.function = FunctionId(9),
        }
        let second = fx.map(second_source, second_item, second_body);
        let (a, b) = (fx.digits(first), fx.digits(second));
        fx.finish(&[(a, b)]);
        assert_eq!(
            seq_refused(fx),
            ProgramRefusalCode::InvalidBinding,
            "case {case}"
        );
    }
}

fn map_program(values: &[i128]) -> SeqFx {
    let mut fx = SeqFx::new();
    let sequence = fx.seq(values);
    let item = fx.local(FunctionId(0));
    let body = fx.square_plus_one(item);
    let mapped = fx.map(sequence, item, body);
    let digits = fx.digits(mapped);
    let literal = fx.expect(digits, digits_of(values.iter().map(|v| v * v + 1)));
    fx.finish(&[literal]);
    fx
}

fn filter_program(values: &[i128]) -> SeqFx {
    let mut fx = SeqFx::new();
    let sequence = fx.seq(values);
    let item = fx.local(FunctionId(0));
    let (read, two) = (fx.read(item), fx.int(2));
    let predicate = fx.greater(read, two);
    let kept = fx.filter(sequence, item, predicate);
    let digits = fx.digits(kept);
    let literal = fx.expect(digits, digits_of(values.iter().copied().filter(|v| *v > 2)));
    fx.finish(&[literal]);
    fx
}

fn fold_program(values: &[i128]) -> SeqFx {
    let mut fx = SeqFx::new();
    let sequence = fx.seq(values);
    let digits = fx.digits(sequence);
    let literal = fx.expect(digits, digits_of(values.iter().copied()));
    fx.finish(&[literal]);
    fx
}

fn seq_stop(
    fx: SeqFx,
    limits: StepBudget,
) -> Result<(), (MonitorFailureCode, Option<StepResource>)> {
    build(BuildPlan {
        program: Program::try_new(fx.parts).unwrap(),
        budget: limits,
    })
    .map(|_| ())
    .map_err(|failure| (failure.code, failure.resource.map(|stop| stop.resource)))
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn exhausting_work_stack_or_scratch_stops_map_and_filter_as_it_stops_a_fold() {
    let values = [3, 1, 4, 1, 5, 9, 2, 6];
    let tight = [
        (
            StepBudget {
                work: 40,
                ..seq_budget()
            },
            StepResource::Work,
        ),
        (
            StepBudget {
                stack_frames: 4,
                ..seq_budget()
            },
            StepResource::Stack,
        ),
        (
            StepBudget {
                scratch_elements: 6,
                ..seq_budget()
            },
            StepResource::ScratchElements,
        ),
    ];
    for (limits, resource) in tight {
        for (name, make) in [
            ("fold", fold_program as fn(&[i128]) -> SeqFx),
            ("map", map_program),
            ("filter", filter_program),
        ] {
            assert_eq!(
                seq_stop(make(&values), limits),
                Err((MonitorFailureCode::Resource, Some(resource))),
                "{name} under {resource:?}"
            );
        }
    }
    for make in [fold_program, map_program, filter_program] {
        seq_stop(make(&values), seq_budget())
            .expect("the same programs complete when the budget allows");
    }
}

#[trace("TC-098", "IT-018-SC-12", "interface-010-AC-5")]
#[test]
fn result_storage_is_reserved_from_the_scratch_byte_budget_before_the_first_visit() {
    // A native sequence literal needs no construction storage, so only the map result does.
    let build_fx = || {
        let mut fx = SeqFx::new();
        let native = quire_exact::from_admitted(
            CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
            vec![Value::Integer(Integer::from(2_u64))],
        );
        let sequence = fx.add(seq_integers(), Operation::Literal(native));
        let item = fx.local(FunctionId(0));
        let body = fx.read(item);
        let mapped = fx.map(sequence, item, body);
        let size = fx.add(
            ValueType::Integer,
            Operation::SequenceSize { value: mapped },
        );
        let literal = fx.expect(size, 1);
        fx.finish(&[literal]);
        fx
    };
    let starved = StepBudget {
        scratch_bytes: 0,
        ..seq_budget()
    };
    assert_eq!(
        seq_stop(build_fx(), starved),
        Err((
            MonitorFailureCode::Resource,
            Some(StepResource::ScratchBytes)
        ))
    );
    seq_stop(build_fx(), seq_budget()).expect("the same map completes with byte storage");
}
