// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-016-SC-02 / TC-096: remaining TC-201 quantity controls on admitted units.

#[path = "support/producer_origin.rs"]
mod producer_origin_support;
mod support;

use std::{collections::BTreeMap, sync::OnceLock};

use ix_trace_rs::trace;
use qsl_foundation::node_wire::node_key_from_json;
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm};
use quire_exact::{
    ChargePoint, CheckedInvariantCause, Decimal, DecimalType, IllTyped, IllTypedCause, Integer,
    IntegerInterval, LimitKind, Meter, NodeKey, Outcome, Quantity, Rational, Refusal, RoundingMode,
    ScalarLimits, UnitId, Value as ExactValue, ValueType,
};
use quire_semantic_value::{
    declaration::{operand_value, EqualityOperand, EqualityOperator, TypeEnvironment},
    enumeration::EnumMemberIndex,
    quantity::{
        convert_quantity, evaluate_quantity, ConvertedValue, HandoffRefusal, QuantityOperation,
        QuantityRefusal, QuantityTarget, QuantityUnit, UnitScope, UnitTable,
    },
    stop::Stop,
    unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode},
};
use serde_json::Value;

const SOURCE_PATH: &str = "program.native";
const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    dimension Length;\n\
    unit m : Length = rational(1, 1);\n\
    unit cm : Length = rational(1, 100) * m;\n\
    unit km : Length = rational(1000, 1) * m;\n\
    unit inch : Length = rational(127, 5000) * m;\n\
    dimension Area = Length ^ 2;\n\
    unit m2 : Area = rational(1, 1);\n\
    unit cm2 : Area = rational(1, 10000) * m2;\n\
    dimension Time;\n\
    unit s : Time = rational(1, 1);\n\
    dimension Mass;\n\
    unit kg : Mass = rational(1, 1);\n\
    dimension Torque = Mass * Length ^ 2 / Time ^ 2;\n\
    dimension Energy = Mass * Length ^ 2 / Time ^ 2;\n\
    unit N_m : Torque = rational(1, 1);\n\
    unit J : Energy = rational(1, 1);\n\
    dimension Temperature;\n\
    unit K : Temperature = rational(1, 1);\n\
    unit C : Temperature = rational(1, 1) * K + rational(5463, 20);\n\
    unit F : Temperature = rational(5, 9) * K + rational(45967, 180);\n\
    unit u1 : Temperature = rational(1, 1) * K + rational(10, 1);\n\
    unit u2 : Temperature = rational(1, 1) * u1 + rational(-10, 1);\n\
    unit u3 : Temperature = rational(1, 1) * C;\n\
    dimension Chain;\n\
    unit root : Chain = rational(1, 1);\n\
    unit source_mid : Chain = rational(2, 1) * root + rational(3, 1);\n\
    unit source_leaf : Chain = rational(-3, 1) * source_mid + rational(5, 1);\n\
    unit target_mid : Chain = rational(4, 1) * root + rational(-2, 1);\n\
    unit target_leaf : Chain = rational(3, 1) * target_mid + rational(7, 1);\n";

const UNIT_NAMES: [&str; 21] = [
    "m",
    "cm",
    "km",
    "inch",
    "m2",
    "cm2",
    "s",
    "kg",
    "N_m",
    "J",
    "K",
    "C",
    "F",
    "u1",
    "u2",
    "u3",
    "root",
    "source_mid",
    "source_leaf",
    "target_mid",
    "target_leaf",
];

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

fn integer(value: i64) -> Integer {
    Integer::from(value)
}

fn rational(numerator: i64, denominator: i64) -> Rational {
    Rational::new(integer(numerator), integer(denominator)).expect("nonzero denominator")
}

/// A package is emitted by the Cargo-lock-selected QSL binary and admitted
/// by IR in this run. Refusal at either seam is a test failure.
fn admitted() -> &'static CheckedPackageV2 {
    static PACKAGE: OnceLock<CheckedPackageV2> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let mut source = SOURCE.to_owned();
        for unit in UNIT_NAMES {
            source.push_str(&format!(
                "function touch_{unit} using v(x: Quantity<{unit}>): Quantity<{unit}> pure {{ x }}\n"
            ));
        }
        producer_origin_support::emit_and_read_checked_v2(producer_origin_support::CheckedProducerRequest {
            source_path: SOURCE_PATH,
            source: &source,
            package_name: "agent-ix/it-016-quantity",
            models: &[],
            execution_points: None,
        })
    })
}

fn declaration(name: &str) -> &'static CheckedSemanticNodeV2 {
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
        panic!("expected one admitted {name}, got {}", nodes.len())
    };
    node
}

fn key(id: &CheckedNodeId) -> NodeKey {
    node_key_from_json(&serde_json::to_value(id).expect("admitted ID serializes"))
        .expect("admitted structural ID decodes")
}

fn named_key(name: &str) -> NodeKey {
    key(&declaration(name).node_id)
}

fn body(name: &str) -> Value {
    let node = declaration(name);
    assert!(
        matches!(&node.body, CheckedTerm::NominalBody(_)),
        "{name} is nominal"
    );
    serde_json::to_value(&node.body).expect("admitted nominal body serializes")
}

fn body_key(body: &Value, field: &str) -> NodeKey {
    let id: CheckedNodeId = serde_json::from_value(body[field].clone())
        .unwrap_or_else(|error| panic!("admitted {field}: {error}"));
    key(&id)
}

fn body_rational(body: &Value, field: &str) -> Rational {
    let number = &body[field];
    let parse = |part: &str| {
        number[part]
            .as_str()
            .unwrap_or_else(|| panic!("{field}.{part} is an integer"))
            .parse::<Integer>()
            .unwrap_or_else(|error| panic!("{field}.{part}: {error}"))
    };
    Rational::new(parse("numerator"), parse("denominator"))
        .expect("admitted rational denominator is nonzero")
}

fn graph() -> UnitGraph {
    let dimensions = [
        "Length",
        "Area",
        "Time",
        "Mass",
        "Torque",
        "Energy",
        "Temperature",
        "Chain",
    ]
    .into_iter()
    .map(|name| {
        let record = body(name);
        assert_eq!(record["term"], "dimension");
        let terms = record["terms"]
            .as_array()
            .expect("admitted dimension has ordered terms")
            .iter()
            .map(|term| {
                let exponent = term["exponent"]
                    .as_str()
                    .expect("canonical exponent")
                    .parse::<Integer>()
                    .expect("admitted integer exponent");
                (body_key(term, "dimension_node_id"), exponent)
            })
            .collect();
        (
            named_key(name),
            DimensionNode::checked(terms, NominalDeclaration::default())
                .expect("admitted dimension terms"),
        )
    })
    .collect::<BTreeMap<_, _>>();
    let units = UNIT_NAMES
        .into_iter()
        .map(|name| {
            let record = body(name);
            assert_eq!(record["term"], "unit");
            let dimension = body_key(&record, "dimension_node_id");
            let target = if record["target_unit_node_id"].is_null() {
                None
            } else {
                Some(body_key(&record, "target_unit_node_id"))
            };
            let unit = UnitNode::checked(
                dimension,
                target,
                body_rational(&record, "scale"),
                body_rational(&record, "offset"),
                NominalDeclaration::default(),
            )
            .expect("admitted unit edge");
            (named_key(name), unit)
        })
        .collect::<BTreeMap<_, _>>();
    UnitGraph::from_checked_nodes(&dimensions, &units).expect("QSL's unit topology admits")
}

fn quantity(unit: &str, numerator: i64, denominator: i64) -> Quantity {
    Quantity::new(
        rational(numerator, denominator),
        UnitId::declared(named_key(unit)),
    )
}

#[test]
#[trace("IT-016-SC-02", "TC-096", "TC-201", "FR-275-AC-23")]
fn admitted_energy_converts_through_first_compound_target_and_back_to_joule() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let source = quantity("N_m", 1, 1);
    let mut terms = [
        (named_key("kg"), integer(1)),
        (named_key("m"), integer(2)),
        (named_key("s"), integer(-2)),
    ];
    terms.sort_by(|left, right| left.0.cmp(&right.0));
    let compound = QuantityUnit::Compound(graph.compound_unit(&terms).expect("admitted roots"));
    let limits = ScalarLimits {
        integer_bits: 1,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 1,
        work_units: 3,
        result_units: 1,
    };
    let mut first = Meter::new(limits);
    let Outcome::Completed(converted) = convert_quantity(
        table.resolve(&source).expect("admitted N_m"),
        &compound,
        &QuantityTarget::Exact,
        &mut first,
    )
    .expect("equal admitted base dimensions") else {
        panic!("first compound target conversion completes");
    };
    assert_eq!(converted.value(), &ConvertedValue::Exact(rational(1, 1)));
    assert_eq!(converted.source(), &source);
    assert_eq!(converted.unit(), &compound);
    let before_handoff = first.admission_count();
    let (carried, identified) = converted.identified_quantity().expect("compound identity");
    assert_eq!(first.admission_count(), before_handoff);
    assert_eq!(carried.magnitude(), &rational(1, 1));
    assert_eq!(carried.unit(), &identified.id());
    assert_ne!(carried.unit(), &UnitId::declared(named_key("J")));
    let mut scope = UnitScope::new(&table);
    assert_eq!(scope.form(identified), carried.unit().clone());
    let mut second = Meter::new(limits);
    let Outcome::Completed(back) = convert_quantity(
        scope
            .resolve(&carried)
            .expect("formed compound is readable"),
        table
            .get(UnitId::declared(named_key("J")))
            .expect("admitted J"),
        &QuantityTarget::Exact,
        &mut second,
    )
    .expect("equal admitted base dimensions") else {
        panic!("second declared target conversion completes");
    };
    let (joules, _) = back.identified_quantity().expect("declared identity");
    assert_eq!(joules.magnitude(), &rational(1, 1));
    assert_eq!(joules.unit(), &UnitId::declared(named_key("J")));
    for meter in [&first, &second] {
        assert_eq!(
            meter
                .admitted_charges()
                .iter()
                .map(|charge| charge.point)
                .collect::<Vec<_>>(),
            [
                ChargePoint::UnitIdentityRead,
                ChargePoint::UnitTargetDomain,
                ChargePoint::UnitResultRetain
            ]
        );
        assert_eq!(meter.consumed(LimitKind::UnitEdges), 0);
        assert_eq!(meter.consumed(LimitKind::WorkUnits), 3);
    }
}

#[test]
#[trace("IT-016-SC-02", "TC-096", "TC-201", "FR-275-AC-23")]
fn admitted_integer_conversion_refuses_exact_quantity_handoff_without_charge() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let source = quantity("N_m", 1, 1);
    let mut charged = meter();
    let Outcome::Completed(converted) = convert_quantity(
        table.resolve(&source).expect("admitted N_m"),
        table
            .get(UnitId::declared(named_key("N_m")))
            .expect("admitted N_m"),
        &QuantityTarget::Integer {
            domain: IntegerInterval::spanning(integer(0), integer(10)),
            rounding: RoundingMode::Exact,
        },
        &mut charged,
    )
    .expect("same admitted dimension") else {
        panic!("integer target conversion completes");
    };
    let before_handoff = charged.admission_count();
    assert_eq!(
        converted.identified_quantity(),
        Err(HandoffRefusal::NotExact)
    );
    assert_eq!(charged.admission_count(), before_handoff);
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-034", "FR-007-AC-12")]
fn admitted_quantity_operations_report_the_first_typed_cause_without_charge() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let metre = quantity("m", 1, 1);
    let kilometre = quantity("km", 1, 1);
    let kelvin = quantity("K", 1, 1);
    let celsius = quantity("C", 1, 1);
    let m = table.resolve(&metre).expect("admitted metre");
    let km = table.resolve(&kilometre).expect("admitted kilometre");
    let k = table.resolve(&kelvin).expect("admitted kelvin");
    let c = table.resolve(&celsius).expect("admitted celsius");
    for (operation, expected) in [
        (
            QuantityOperation::Add(m, c),
            IllTypedCause::IncompatibleDimensions,
        ),
        (
            QuantityOperation::Subtract(m, c),
            IllTypedCause::IncompatibleDimensions,
        ),
        (
            QuantityOperation::Add(c, k),
            IllTypedCause::AffineUnitArithmetic,
        ),
        (QuantityOperation::Add(m, km), IllTypedCause::DistinctUnits),
        (
            QuantityOperation::Subtract(m, km),
            IllTypedCause::DistinctUnits,
        ),
        (
            QuantityOperation::Multiply(c, k),
            IllTypedCause::AffineUnitArithmetic,
        ),
        (
            QuantityOperation::Divide(k, c),
            IllTypedCause::AffineUnitArithmetic,
        ),
        (
            QuantityOperation::Power(c, &integer(2)),
            IllTypedCause::AffineUnitArithmetic,
        ),
    ] {
        let mut charged = meter();
        assert_eq!(
            evaluate_quantity(operation, &mut charged).unwrap_err(),
            QuantityRefusal::IllTyped(IllTyped { cause: expected })
        );
        assert_eq!(charged.admission_count(), 0);
    }
    for operation in [
        QuantityOperation::Multiply(km, k),
        QuantityOperation::Divide(km, k),
        QuantityOperation::Power(km, &integer(2)),
    ] {
        assert!(matches!(
            evaluate_quantity(operation, &mut meter()),
            Ok(Outcome::Completed(_))
        ));
    }
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-032", "FR-011-AC-6")]
fn admitted_quantity_huge_power_denies_before_computation_with_exact_over_u64_amount() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let base = quantity("m", 2, 1);
    let exponent = Integer::from(18_446_744_073_709_551_616_i128);
    let mut charged = Meter::new(ScalarLimits {
        integer_bits: 64,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 1,
        work_units: 5,
        result_units: 1,
    });
    let Outcome::Incomplete(record) = evaluate_quantity(
        QuantityOperation::Power(table.resolve(&base).expect("admitted metre"), &exponent),
        &mut charged,
    )
    .expect("power operation has a runtime outcome") else {
        panic!("the derived amount must be denied before powering");
    };
    assert_eq!(record.limit_kind, LimitKind::IntegerBits);
    assert_eq!(record.limit, 64);
    assert_eq!(record.consumed, 2);
    assert_eq!(record.charge_point, ChargePoint::UnitRationalArithmetic);
    assert_eq!(record.next_charge.to_string(), "36893488147419103232");
    assert_eq!(
        charged
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect::<Vec<_>>(),
        [ChargePoint::UnitIdentityRead]
    );
}

#[test]
#[trace(
    "IT-016-SC-05",
    "TC-096",
    "IT-016-SC-02",
    "TC-201",
    "FR-007-AC-20",
    "FR-007-AC-23"
)]
fn admitted_noncommuting_multiedge_conversion_keeps_order_and_edge_count() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    for (source, target, expected) in [
        (
            quantity("source_leaf", 2, 1),
            "target_leaf",
            rational(-25, 12),
        ),
        (
            quantity("target_leaf", -25, 12),
            "source_leaf",
            rational(2, 1),
        ),
    ] {
        let mut charged = meter();
        let Outcome::Completed(result) = convert_quantity(
            table.resolve(&source).expect("admitted source"),
            table
                .get(UnitId::declared(named_key(target)))
                .expect("admitted target"),
            &QuantityTarget::Exact,
            &mut charged,
        )
        .expect("same dimension") else {
            panic!("two-edge conversion completes");
        };
        assert_eq!(result.value(), &ConvertedValue::Exact(expected));
        assert_eq!(charged.consumed(LimitKind::UnitEdges), 4);
    }
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-23")]
fn admitted_quantity_multiedge_and_affine_charge_order_is_exact() {
    let graph = graph();
    let mut table = UnitTable::declared(&graph);
    let square_id = table
        .insert(QuantityUnit::Compound(
            graph
                .compound_unit(&[(named_key("m"), integer(2))])
                .expect("the admitted metre is a canonical root"),
        ))
        .expect("compound area can be resolved");
    for (source, target, expected, edges, arithmetic, bits, work) in [
        (quantity("inch", 1, 1), "cm", rational(127, 50), 2, 4, 15, 9),
        (
            Quantity::new(rational(4, 1), square_id.clone()),
            "cm2",
            rational(40000, 1),
            1,
            2,
            17,
            6,
        ),
        (
            Quantity::new(rational(4, 1), square_id.clone()),
            "m2",
            rational(4, 1),
            0,
            0,
            3,
            3,
        ),
        (quantity("u2", 1, 1), "u1", rational(-9, 1), 3, 6, 6, 12),
    ] {
        let mut charged = meter();
        let Outcome::Completed(result) = convert_quantity(
            table.resolve(&source).expect("admitted source"),
            table
                .get(UnitId::declared(named_key(target)))
                .expect("admitted target"),
            &QuantityTarget::Exact,
            &mut charged,
        )
        .expect("same dimension") else {
            panic!("listed conversion completes");
        };
        assert_eq!(result.value(), &ConvertedValue::Exact(expected));
        let mut schedule = vec![ChargePoint::UnitIdentityRead];
        schedule.extend(std::iter::repeat_n(ChargePoint::UnitEdge, edges));
        schedule.extend(std::iter::repeat_n(
            ChargePoint::UnitRationalArithmetic,
            arithmetic,
        ));
        schedule.extend([ChargePoint::UnitTargetDomain, ChargePoint::UnitResultRetain]);
        assert_eq!(
            charged
                .admitted_charges()
                .iter()
                .map(|charge| charge.point)
                .collect::<Vec<_>>(),
            schedule
        );
        assert_eq!(charged.consumed(LimitKind::IntegerBits), bits);
        assert_eq!(charged.consumed(LimitKind::UnitEdges), edges as u64);
        assert_eq!(charged.consumed(LimitKind::WorkUnits), work);
        assert_eq!(charged.consumed(LimitKind::ResultUnits), 1);
    }
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-23")]
fn admitted_inch_conversion_reports_first_two_rational_charge_amounts() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let source = quantity("inch", 1, 1);
    for (budget, consumed, next) in [(13, 1, 14), (14, 14, 15)] {
        let mut limits = *meter().limits();
        limits.integer_bits = budget;
        let Outcome::Incomplete(record) = convert_quantity(
            table.resolve(&source).expect("admitted inch"),
            table
                .get(UnitId::declared(named_key("cm")))
                .expect("admitted centimetre"),
            &QuantityTarget::Exact,
            &mut Meter::new(limits),
        )
        .expect("same dimension") else {
            panic!("the selected arithmetic charge must deny");
        };
        assert_eq!(record.limit_kind, LimitKind::IntegerBits);
        assert_eq!(record.consumed, consumed);
        assert_eq!(record.next_charge, integer(next));
        assert_eq!(record.charge_point, ChargePoint::UnitRationalArithmetic);
    }
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-24")]
fn admitted_affine_units_refuse_arithmetic_without_charging() {
    let graph = graph();
    let table = UnitTable::declared(&graph);
    let c1 = quantity("C", 1, 1);
    let c2 = quantity("C", 2, 1);
    let u1 = quantity("u1", 1, 1);
    let u3 = quantity("u3", 1, 1);
    let metre = quantity("m", 1, 1);
    let c1 = table.resolve(&c1).expect("admitted celsius");
    let c2 = table.resolve(&c2).expect("admitted celsius");
    for operation in [
        QuantityOperation::Add(c1, c2),
        QuantityOperation::Subtract(c1, c2),
        QuantityOperation::Multiply(c1, c2),
        QuantityOperation::Divide(c1, c2),
        QuantityOperation::Power(c1, &integer(2)),
        QuantityOperation::Add(
            table.resolve(&u1).expect("u1"),
            table.resolve(&u1).expect("u1"),
        ),
        QuantityOperation::Multiply(
            table.resolve(&u3).expect("u3"),
            table.resolve(&metre).expect("metre"),
        ),
    ] {
        let mut charged = meter();
        assert_eq!(
            evaluate_quantity(operation, &mut charged).unwrap_err(),
            QuantityRefusal::IllTyped(IllTyped {
                cause: IllTypedCause::AffineUnitArithmetic
            })
        );
        assert_eq!(charged.admission_count(), 0);
    }
    let u2_one = quantity("u2", 1, 1);
    let u2_two = quantity("u2", 2, 1);
    let Outcome::Completed(sum) = evaluate_quantity(
        QuantityOperation::Add(
            table.resolve(&u2_one).expect("u2"),
            table.resolve(&u2_two).expect("u2"),
        ),
        &mut meter(),
    )
    .expect("u2 is non-affine") else {
        panic!("u2 addition completes");
    };
    assert_eq!(sum.magnitude(), &rational(3, 1));
    assert_eq!(sum.unit(), &UnitId::declared(named_key("u2")));
}

#[test]
#[trace("IT-016-SC-05", "TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-25")]
fn admitted_conversion_denials_preserve_charge_point_prefix_and_amounts() {
    let graph = graph();
    let mut table = UnitTable::declared(&graph);
    let square_id = table
        .insert(QuantityUnit::Compound(
            graph
                .compound_unit(&[(named_key("m"), integer(2))])
                .expect("the admitted metre is a canonical root"),
        ))
        .expect("compound area can be resolved");
    let source = quantity("cm", 100, 1);
    let mut charged = Meter::new(ScalarLimits {
        result_units: 0,
        ..LIMITS
    });
    let Outcome::Incomplete(record) = convert_quantity(
        table.resolve(&source).expect("admitted centimetre"),
        table
            .get(UnitId::declared(named_key("m")))
            .expect("admitted metre"),
        &QuantityTarget::Exact,
        &mut charged,
    )
    .expect("same dimension") else {
        panic!("retention denies after conversion work");
    };
    assert_eq!(record.limit_kind, LimitKind::ResultUnits);
    assert_eq!(record.limit, 0);
    assert_eq!(record.consumed, 0);
    assert_eq!(record.next_charge, integer(1));
    assert_eq!(record.charge_point, ChargePoint::UnitResultRetain);

    for (source, target, kind, limit, consumed, next, point) in [
        (
            quantity("inch", 1, 1),
            "cm",
            LimitKind::UnitEdges,
            1,
            1,
            2,
            ChargePoint::UnitEdge,
        ),
        (
            quantity("u2", 1, 1),
            "u1",
            LimitKind::UnitEdges,
            2,
            2,
            3,
            ChargePoint::UnitEdge,
        ),
        (
            Quantity::new(rational(4, 1), square_id),
            "cm2",
            LimitKind::IntegerBits,
            16,
            5,
            17,
            ChargePoint::UnitRationalArithmetic,
        ),
        (
            quantity("inch", 1, 1),
            "cm",
            LimitKind::IntegerBits,
            13,
            1,
            14,
            ChargePoint::UnitRationalArithmetic,
        ),
    ] {
        let mut limits = LIMITS;
        match kind {
            LimitKind::UnitEdges => limits.unit_edges = limit,
            LimitKind::IntegerBits => limits.integer_bits = limit,
            _ => panic!("listed denial uses edge or arithmetic limit"),
        }
        let mut charged = Meter::new(limits);
        let Outcome::Incomplete(record) = convert_quantity(
            table.resolve(&source).expect("admitted source"),
            table
                .get(UnitId::declared(named_key(target)))
                .expect("admitted target"),
            &QuantityTarget::Exact,
            &mut charged,
        )
        .expect("same dimension") else {
            panic!("selected charge must deny");
        };
        assert_eq!(record.limit_kind, kind);
        assert_eq!(record.limit, limit);
        assert_eq!(record.consumed, consumed);
        assert_eq!(record.next_charge, integer(next));
        assert_eq!(record.charge_point, point);
    }
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_quantity_equality_converts_left_operand_before_comparison() {
    let units = UnitTable::declared(&graph());
    let metre = UnitId::declared(named_key("m"));
    let centimetre = UnitId::declared(named_key("cm"));
    let types = TypeEnvironment::default().with_units(units);
    let checked = types
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::converted(
                ValueType::Quantity(metre.clone()),
                ValueType::Quantity(centimetre.clone()),
            ),
            EqualityOperand::typed(ValueType::Quantity(centimetre.clone())),
            &EnumMemberIndex::default(),
        )
        .expect("admitted metre converts to centimetre for equality");
    let left = ExactValue::Quantity(Quantity::new(rational(1, 1), metre));
    let equal = ExactValue::Quantity(Quantity::new(rational(100, 1), centimetre.clone()));
    let unequal = ExactValue::Quantity(Quantity::new(rational(99, 1), centimetre));
    for (right, expected) in [(&equal, true), (&unequal, false)] {
        let mut charged = meter();
        assert_eq!(
            checked.evaluate(&types, &left, right, &mut charged),
            Outcome::Completed(expected)
        );
        let points: Vec<_> = charged
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect();
        assert_eq!(points.first(), Some(&ChargePoint::UnitIdentityRead));
        assert!(points.contains(&ChargePoint::UnitResultRetain));
        assert!(points.contains(&ChargePoint::OrderingResultRetain));
        let conversion_end = points
            .iter()
            .position(|point| *point == ChargePoint::UnitResultRetain)
            .expect("left operand conversion retains its result");
        let comparison_start = points
            .iter()
            .position(|point| *point == ChargePoint::OrderingOperands)
            .expect("converted quantities are compared");
        assert!(conversion_end < comparison_start, "{points:?}");
    }
    assert_eq!(
        types
            .check_equality(
                EqualityOperator::Equal,
                EqualityOperand::typed(ValueType::Quantity(UnitId::declared(named_key("m")))),
                EqualityOperand::typed(ValueType::Quantity(UnitId::declared(named_key("cm")))),
                &EnumMemberIndex::default(),
            )
            .unwrap_err()
            .cause,
        IllTypedCause::DistinctUnits
    );
    assert_eq!(
        types
            .check_equality(
                EqualityOperator::Equal,
                EqualityOperand::typed(ValueType::Quantity(UnitId::declared(named_key("m")))),
                EqualityOperand::typed(ValueType::Quantity(UnitId::declared(named_key("s")))),
                &EnumMemberIndex::default(),
            )
            .unwrap_err()
            .cause,
        IllTypedCause::IncompatibleDimensions
    );
}

#[test]
#[trace("IT-016-SC-03", "TC-096", "TC-026", "FR-008-AC-5")]
fn admitted_equality_operand_failures_keep_distinct_typed_causes() {
    let units = UnitTable::declared(&graph());
    let scope = UnitScope::new(&units);
    let types = TypeEnvironment::default().with_units(units.clone());
    let failure = |operand: EqualityOperand, value: ExactValue, units: &UnitScope<'_>| {
        let mut charged = meter();
        let result = operand_value(&types, &operand, &value, units, &mut charged);
        assert_eq!(charged.admission_count(), 0);
        result
    };
    let refusal = |result, cause| {
        assert!(matches!(
            result,
            Err(Stop::Refused(Refusal::CheckedInvariant { cause: actual })) if actual == cause
        ));
    };
    refusal(
        failure(
            EqualityOperand::converted(ValueType::Boolean, ValueType::Integer),
            ExactValue::Boolean(true),
            &scope,
        ),
        CheckedInvariantCause::EqualityConversionShapeMismatch,
    );
    refusal(
        failure(
            EqualityOperand::converted(
                ValueType::Integer,
                ValueType::Int(IntegerInterval::spanning(integer(0), integer(0))),
            ),
            ExactValue::Integer(integer(1)),
            &scope,
        ),
        CheckedInvariantCause::EqualityOperandTargetNotAdmitted,
    );
    let decimal_type = ValueType::Decimal(
        DecimalType::new(integer(0), integer(10), 1, 1, RoundingMode::Exact)
            .expect("admitted decimal bounds"),
    );
    refusal(
        failure(
            EqualityOperand::converted(decimal_type, ValueType::Integer),
            ExactValue::Decimal(Decimal::new(integer(5), 1)),
            &scope,
        ),
        CheckedInvariantCause::EqualityOperandNonIntegralDecimal,
    );
    let metre = UnitId::declared(named_key("m"));
    let second = UnitId::declared(named_key("s"));
    let one_metre = ExactValue::Quantity(Quantity::new(rational(1, 1), metre.clone()));
    let missing = UnitTable::default();
    refusal(
        failure(
            EqualityOperand::converted(
                ValueType::Quantity(metre.clone()),
                ValueType::Quantity(second.clone()),
            ),
            one_metre.clone(),
            &UnitScope::new(&missing),
        ),
        CheckedInvariantCause::EqualityUnitUnresolved,
    );
    refusal(
        failure(
            EqualityOperand::converted(ValueType::Quantity(metre), ValueType::Quantity(second)),
            one_metre,
            &scope,
        ),
        CheckedInvariantCause::EqualityQuantityConversionRejected {
            cause: IllTypedCause::IncompatibleDimensions,
        },
    );
}
