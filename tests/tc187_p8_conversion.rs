// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original checked quantity operations and independently admitted native inputs.

use std::num::NonZeroU64;

use ix_trace_rs::trace;
use qsl_replay::{
    AdmittedContext, AdmittedUnit, CompileUnit, DependencyInput, ReplayLimits, StageLimits,
};
use quire_eval::FamilyOutcome;
use quire_exact::{
    AdmittedCharge, ChargePoint, Incomplete, InjectedDenial, Integer, LimitKind, Meter, Outcome,
    Quantity, Rational, ScalarLimits, Value,
};
use quire_semantic_value::quantity::{convert_quantity, QuantityTarget};
use quire_specification::{vectors_of, TestCase};

#[path = "support/family_checked_source.rs"]
mod family_checked_source;

const HEADER: &str =
    "language \"ix:native\" edition \"1-draft\"; profile v = \"quire.value.complete/v1\";";
const LENGTH: &str = "dimension L; unit m : L = rational(1, 1); unit cm : L = rational(1, 100) * m; unit metre_alias : L = rational(1, 1) * m;";
const TEMPERATURE: &str = "dimension Theta; unit K : Theta = rational(1, 1); unit degC : Theta = rational(1, 1) * K + rational(5463, 20); unit degF : Theta = rational(5, 9) * K + rational(45967, 180);";

fn bind(id: &str, input: &str, expected: &str) {
    let rows = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        panic!("one public vector {id}");
    };
    assert_eq!(row.inputs, input, "{id} entire input cell");
    assert_eq!(row.expected, expected, "{id} entire expected cell");
}

fn rational(n: i64, d: i64) -> Rational {
    Rational::new(Integer::from(n), Integer::from(d)).expect("nonzero denominator")
}

fn admitted(source: &str) -> AdmittedContext {
    qsl_replay::compile_admitted(
        CompileUnit {
            source: qsl_foundation::SourceIdentity::new("agent-ix", "scalar-operations"),
            package_name: "agent-ix/scalar-operations".to_owned(),
            path: "scalar-operations.native",
            bytes: source.as_bytes(),
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("identical original source admits")
    .into_parts()
    .1
}

fn unit<'a>(context: &'a AdmittedContext, name: &str) -> &'a AdmittedUnit {
    let matches = context
        .units()
        .filter(|unit| {
            unit.qualified_name()
                .iter()
                .eq([&qsl_replay::Identifier::new(name).expect("authored unit name")])
        })
        .collect::<Vec<_>>();
    let [unit] = matches.as_slice() else {
        panic!("one original declared unit {name}");
    };
    unit
}

fn limits() -> ScalarLimits {
    ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 16,
        value_occurrences: 16,
        work_units: 1024,
        result_units: 16,
    }
}

fn u10_limits() -> ScalarLimits {
    ScalarLimits {
        integer_bits: 8,
        unit_edges: 1,
        value_occurrences: 1,
        work_units: 6,
        result_units: 1,
        ..limits()
    }
}

fn lift<T>(outcome: Outcome<T>, completed: impl FnOnce(T) -> Value) -> Outcome<Value> {
    match outcome {
        Outcome::Completed(value) => Outcome::Completed(completed(value)),
        Outcome::Incomplete(stop) => Outcome::Incomplete(stop),
        Outcome::Undefined(stop) => Outcome::Undefined(stop),
        Outcome::Refused(stop) => Outcome::Refused(stop),
    }
}

fn counters(meter: &Meter) -> [u64; 10] {
    LimitKind::ALL.map(|kind| meter.consumed(kind))
}

fn conversion(
    declarations: &str,
    source_unit: &str,
    target: &str,
    n: i64,
    limits: ScalarLimits,
    deny: bool,
) -> (Outcome<Value>, Meter, AdmittedContext) {
    let source = format!("{HEADER} {declarations} function observe using v(): Quantity<{target}> pure {{ convert<Quantity<{target}>>({n} {source_unit}) }}");
    let package = family_checked_source::checked(&source);
    let context = admitted(&source);
    let name = "observe";
    let function = package
        .graph()
        .function_by_identity(
            package
                .graph()
                .function_identity(name)
                .expect("authored observe identity"),
        )
        .expect("original checked body");
    let new_meter = || {
        let meter = Meter::new(limits);
        if deny {
            meter.with_injected_denial(InjectedDenial {
                point: ChargePoint::UnitResultRetain,
                occurrence: NonZeroU64::new(1).expect("first occurrence"),
            })
        } else {
            meter
        }
    };
    let mut source_meter = new_meter();
    let evaluated = qsl_eval::value::evaluate_closed(
        &package,
        function.body,
        function.slots,
        &mut source_meter,
    )
    .expect("actual closed conversion evaluates");
    let FamilyOutcome::Evaluated(source_outcome) = evaluated.outcome else {
        panic!("quantity family: {:?}", evaluated.outcome);
    };
    assert!(
        evaluated.losses.is_empty(),
        "exact conversion has no loss: {:?}",
        evaluated.losses
    );
    match &source_outcome {
        Outcome::Completed(_) => assert!(evaluated.location.is_none()),
        Outcome::Incomplete(_) => {
            assert_eq!(evaluated.location, Some(function.body.location().clone()))
        }
        other => panic!("conversion completion or metered stop: {other:?}"),
    }
    let quantity = Quantity::new(rational(n, 1), unit(&context, source_unit).id());
    assert_ne!(
        quantity.unit(),
        &unit(&context, target).id(),
        "original source/target identities stay distinct"
    );
    let mut native_meter = new_meter();
    let native = convert_quantity(
        context
            .unit_table()
            .resolve(&quantity)
            .expect("real admitted source unit"),
        unit(&context, target).quantity_unit(),
        &QuantityTarget::Exact,
        &mut native_meter,
    )
    .expect("same admitted dimension");
    let native_outcome = lift(native, |conversion| {
        let (quantity, identified) = conversion
            .identified_quantity()
            .expect("exact native handoff");
        assert_eq!(identified.id(), unit(&context, target).id());
        Value::Quantity(quantity)
    });
    assert_eq!(
        format!("{source_outcome:?}"),
        format!("{native_outcome:?}"),
        "complete typed source/native outcome"
    );
    assert_eq!(
        source_meter.admitted_charges(),
        native_meter.admitted_charges(),
        "all ordered charge amounts"
    );
    assert_eq!(
        counters(&source_meter),
        counters(&native_meter),
        "all ten counters"
    );
    (source_outcome, source_meter, context)
}

fn charge(
    point: ChargePoint,
    bits: Option<u64>,
    edges: Option<u64>,
    occurrences: Option<u64>,
) -> AdmittedCharge {
    AdmittedCharge {
        point,
        integer_bits: bits,
        decimal_digits: None,
        scale_expansion: None,
        text_input_bytes: None,
        text_scalars: None,
        normalized_scalars: None,
        unit_edges: edges,
        value_occurrences: occurrences,
    }
}

fn conversion_stops(declarations: &str, from: &str, to: &str, n: i64) {
    for work in [0, 1] {
        let (outcome, meter, _) = conversion(
            declarations,
            from,
            to,
            n,
            ScalarLimits {
                work_units: work,
                ..limits()
            },
            false,
        );
        let Outcome::Incomplete(stop) = outcome else {
            panic!("operation stops without partial quantity: {outcome:?}");
        };
        assert_eq!(
            stop,
            Incomplete {
                limit_kind: LimitKind::WorkUnits,
                limit: work,
                consumed: work,
                next_charge: Integer::from(1_i64),
                charge_point: if work == 0 {
                    ChargePoint::UnitIdentityRead
                } else {
                    ChargePoint::UnitEdge
                },
            }
        );
        assert_eq!(
            meter.admitted_charges().len(),
            usize::try_from(work).expect("bounded work")
        );
        assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
    }
}

fn completed(outcome: Outcome<Value>, context: &AdmittedContext, target: &str, expected: Rational) {
    let Outcome::Completed(Value::Quantity(quantity)) = outcome else {
        panic!("required exact quantity: {outcome:?}");
    };
    assert_eq!(quantity.magnitude(), &expected);
    assert_eq!(quantity.unit(), &unit(context, target).id());
}

#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn original_affine_conversions_and_scale_one_alias_match_native_exactly() {
    bind(
        "U04",
        "Convert `0 degC` and `32 degF` to `K`",
        "both equal exact `5463/20 K`",
    );
    for (name, n) in [("degC", 0), ("degF", 32)] {
        let (outcome, _, context) = conversion(TEMPERATURE, name, "K", n, limits(), false);
        completed(outcome, &context, "K", rational(5463, 20));
        conversion_stops(TEMPERATURE, name, "K", n);
    }
    bind(
        "U09b",
        "Declare a distinct unit key under `L` with scale one and convert it to `m`",
        "conversion succeeds exactly while source and target unit identities remain distinct",
    );
    let (outcome, _, context) = conversion(LENGTH, "metre_alias", "m", 7, limits(), false);
    assert_eq!(
        unit(&context, "metre_alias").quantity_unit().dimension(),
        unit(&context, "m").quantity_unit().dimension()
    );
    let (
        quire_semantic_value::quantity::QuantityUnit::Declared(alias),
        quire_semantic_value::quantity::QuantityUnit::Declared(metre),
    ) = (
        unit(&context, "metre_alias").quantity_unit(),
        unit(&context, "m").quantity_unit(),
    )
    else {
        panic!("both original units are declared");
    };
    assert_ne!(alias.key(), metre.key());
    assert_eq!(alias.dimension_node(), metre.dimension_node());
    completed(outcome, &context, "m", rational(7, 1));
    conversion_stops(LENGTH, "metre_alias", "m", 7);
}

#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn original_u10_exact_tuple_and_named_final_denial_keep_full_prefix() {
    bind("U10", "Convert `100 cm` to `m` with `ScalarLimitsV1 { integer_bits: 8, decimal_digits: 0, scale_expansion: 0, text_input_bytes: 0, text_scalars: 0, normalized_scalars: 0, unit_edges: 1, value_occurrences: 1, work_units: 6, result_units: 1 }`; then deny `unit.result-retain`", "the exact tuple returns `1 m`, with the largest amount `max(bits(100) + bits(1), bits(1) + bits(100)) = 8` at the multiply event; the denied named final charge returns incomplete without a partial quantity");
    let (healthy, meter, context) = conversion(LENGTH, "cm", "m", 100, u10_limits(), false);
    completed(healthy, &context, "m", rational(1, 1));
    assert_eq!(
        meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect::<Vec<_>>(),
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitEdge,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain
        ]
    );
    assert_eq!(
        meter.admitted_charges(),
        &[
            charge(ChargePoint::UnitIdentityRead, Some(7), None, Some(1)),
            charge(ChargePoint::UnitEdge, None, Some(1), None),
            charge(ChargePoint::UnitRationalArithmetic, Some(8), None, None),
            charge(ChargePoint::UnitRationalArithmetic, Some(3), None, None),
            charge(ChargePoint::UnitTargetDomain, None, None, None),
            charge(ChargePoint::UnitResultRetain, None, None, Some(1)),
        ]
    );
    assert_eq!(counters(&meter), [8, 0, 0, 0, 0, 0, 1, 1, 6, 1]);
    assert_eq!(meter.admitted_charges()[2].integer_bits, Some(8));
    let (denied, stopped, _) = conversion(LENGTH, "cm", "m", 100, u10_limits(), true);
    let Outcome::Incomplete(denied) = denied else {
        panic!("named denial yields no partial quantity: {denied:?}");
    };
    assert_eq!(
        denied,
        Incomplete {
            limit_kind: LimitKind::WorkUnits,
            limit: 5,
            consumed: 5,
            next_charge: Integer::from(1_i64),
            charge_point: ChargePoint::UnitResultRetain
        }
    );
    assert_eq!(stopped.admitted_charges(), &meter.admitted_charges()[..5]);
    assert_eq!(counters(&stopped), [8, 0, 0, 0, 0, 0, 1, 1, 5, 0]);
    let (sibling, sibling_meter, sibling_context) =
        conversion(LENGTH, "cm", "m", 100, u10_limits(), false);
    completed(sibling, &sibling_context, "m", rational(1, 1));
    assert_eq!(sibling_meter.admitted_charges(), meter.admitted_charges());
    assert_eq!(counters(&sibling_meter), counters(&meter));
}

#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn native_normalization_uses_original_admitted_dimension_maps() {
    bind("U02", "Normalize `(L*T)/T` and `L^0`", "respectively `L` and the empty dimension; zero exponents are absent from each retained sorted map");
    let source = format!("{HEADER} {LENGTH} dimension T; unit s : T = rational(1, 1); function observe using v(a: Quantity<m>, b: Quantity<s>): Boolean pure {{ true }}");
    let context = admitted(&source);
    let l = unit(&context, "m").quantity_unit().dimension();
    let t = unit(&context, "s").quantity_unit().dimension();
    let normalized = l.multiply(t).divide(t);
    assert_eq!(&normalized, l);
    assert_eq!(
        normalized.exponents().collect::<Vec<_>>(),
        l.exponents().collect::<Vec<_>>()
    );
    assert!(normalized
        .exponents()
        .all(|(_, exponent)| exponent != &Integer::from(0_i64)));
    let empty = l.power(&Integer::from(0_i64));
    assert!(empty.is_dimensionless());
    assert_eq!(empty.exponents().count(), 0);
}

#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn original_quantity_subnodes_preserve_native_normalized_compound_identities() {
    use quire_checked::node::NodeKind;
    use quire_exact::{ArithmeticOperator, ValueType};
    use quire_semantic_value::quantity::{evaluate_quantity_unit, QuantityOperation, UnitScope};

    bind("U02", "Normalize `(L*T)/T` and `L^0`", "respectively `L` and the empty dimension; zero exponents are absent from each retained sorted map");
    for expression in ["(1 m * 1 s) / 1 s", "1 m ^ 0"] {
        let source = format!("{HEADER} {LENGTH} dimension T; unit s : T = rational(1, 1); function observe using v(): Boolean pure {{ {expression} = {expression} }}");
        let package = family_checked_source::checked(&source);
        let context = admitted(&source);
        let name = "observe";
        let function = package
            .graph()
            .function_by_identity(
                package
                    .graph()
                    .function_identity(name)
                    .expect("authored identity"),
            )
            .expect("original checked equality body");
        let NodeKind::Equality(_, _, left, _) = function.body.kind() else {
            panic!("authored quantity equality: {:?}", function.body.kind());
        };
        let selected = function.body.at(*left);
        let ValueType::Quantity(original_unit) = selected.value_type() else {
            panic!("original operand has a quantity type");
        };
        let is_power = match selected.kind() {
            NodeKind::QuantityPower(_, _) => true,
            NodeKind::Quantity(ArithmeticOperator::Divide, product, _) => {
                assert!(matches!(
                    selected.at(*product).kind(),
                    NodeKind::Quantity(ArithmeticOperator::Multiply, _, _)
                ));
                false
            }
            other => panic!("actual original quantity operation: {other:?}"),
        };
        let metre = Quantity::new(rational(1, 1), unit(&context, "m").id());
        let second = Quantity::new(rational(1, 1), unit(&context, "s").id());
        for work in [0, 1, 1024] {
            let operation_limits = ScalarLimits {
                work_units: work,
                ..limits()
            };
            let mut source_meter = Meter::new(operation_limits);
            let evaluation = qsl_eval::value::evaluate_closed(
                &package,
                selected,
                function.slots,
                &mut source_meter,
            )
            .expect("original selected quantity subtree evaluates");
            let FamilyOutcome::Evaluated(actual) = evaluation.outcome else {
                panic!("original quantity family");
            };
            assert!(evaluation.losses.is_empty());
            let mut native_meter = Meter::new(operation_limits);
            let mut scope = UnitScope::new(context.unit_table());
            let (native, identified) = if is_power {
                evaluate_quantity_unit(
                    QuantityOperation::Power(
                        scope.resolve(&metre).expect("admitted metre"),
                        &Integer::from(0_i64),
                    ),
                    &mut native_meter,
                )
                .expect("native power")
            } else {
                let (product, product_unit) = evaluate_quantity_unit(
                    QuantityOperation::Multiply(
                        scope.resolve(&metre).expect("admitted metre"),
                        scope.resolve(&second).expect("admitted second"),
                    ),
                    &mut native_meter,
                )
                .expect("native product");
                scope.form(product_unit.clone());
                match product {
                    Outcome::Completed(quantity) => evaluate_quantity_unit(
                        QuantityOperation::Divide(
                            scope
                                .resolve(&quantity)
                                .expect("genuine returned product unit"),
                            scope.resolve(&second).expect("admitted second"),
                        ),
                        &mut native_meter,
                    )
                    .expect("native quotient"),
                    Outcome::Incomplete(stop) => (Outcome::Incomplete(stop), product_unit),
                    other => panic!("product completion or stop: {other:?}"),
                }
            };
            let native = lift(native, Value::Quantity);
            assert_eq!(
                format!("{actual:?}"),
                format!("{native:?}"),
                "{expression} full source/native outcome at work {work}"
            );
            assert_eq!(
                source_meter.admitted_charges(),
                native_meter.admitted_charges(),
                "{expression} complete ledger at work {work}"
            );
            assert_eq!(counters(&source_meter), counters(&native_meter));
            if work == 1024 {
                let Outcome::Completed(Value::Quantity(quantity)) = actual else {
                    panic!("healthy subtree completes");
                };
                assert!(evaluation.location.is_none());
                assert_eq!(quantity.magnitude(), &rational(1, 1));
                assert_eq!(
                    quantity.unit(),
                    original_unit,
                    "genuine original static compound identity"
                );
                assert_eq!(quantity.unit(), &identified.id());
                let terms = quantity
                    .unit()
                    .compound_terms()
                    .expect("compound result identity");
                if is_power {
                    assert!(terms.is_empty());
                    assert_eq!(identified.unit().dimension().exponents().count(), 0);
                } else {
                    let quire_semantic_value::quantity::QuantityUnit::Declared(m) =
                        unit(&context, "m").quantity_unit()
                    else {
                        panic!("original declared m");
                    };
                    assert_eq!(terms, &[(m.root(), Integer::from(1_i64))]);
                    assert_eq!(identified.unit().dimension(), m.dimension());
                }
                assert!(identified
                    .unit()
                    .dimension()
                    .exponents()
                    .all(|(_, exponent)| exponent != &Integer::from(0_i64)));
            } else {
                let Outcome::Incomplete(stop) = actual else {
                    panic!("bounded subtree stops");
                };
                assert_eq!(stop.limit_kind, LimitKind::WorkUnits);
                assert_eq!(
                    stop.charge_point,
                    if is_power && work == 1 {
                        ChargePoint::UnitRationalArithmetic
                    } else {
                        ChargePoint::UnitIdentityRead
                    }
                );
                assert_eq!(stop.limit, work);
                assert_eq!(stop.consumed, work);
                assert_eq!(stop.next_charge, Integer::from(1_i64));
                let stop_node = if is_power {
                    selected
                } else {
                    selected.children()[0]
                };
                assert_eq!(evaluation.location, Some(stop_node.location().clone()));
            }
        }
    }
}
