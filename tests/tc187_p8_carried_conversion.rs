// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original carried conversion source and native conversion, without a call envelope.

use ix_trace_rs::trace;
use qsl_package::CheckedPackage;
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_eval::FamilyOutcome;
use quire_exact::{
    ChargePoint, Decimal, DecimalType, InexactTarget, Integer, LimitKind, Meter, Outcome, Quantity,
    QuantityCarrier, Rational, Refusal, RoundingMode, ScalarLimits, UnitId, Value,
};
use quire_semantic_value::{
    checking::CheckingLimits,
    loss::ValueLoss,
    quantity::{convert_quantity, Conversion, ConvertedValue, QuantityTarget},
};
use quire_specification::{vectors_of, TestCase};

fn bind(id: &str, inputs: &str, expected: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("one public {id}: {rows:?}")
    };
    assert_eq!(row.inputs, inputs);
    assert_eq!(row.expected, expected);
}

fn checked(carrier: &str, operand: &str) -> (CheckedPackage, UnitId, UnitId) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         dimension L; unit m : L = rational(1,1);\n\
         unit inch : L = rational(127,5000) * m;\n\
         function observe using v(): Quantity<m,{carrier}> pure {{ \
         convert<Quantity<m,{carrier}>>({operand}) }}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "carried-conversion"),
        "carried.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original conversion parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original conversion forms");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/carried-conversion".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("original conversion assembles");
    let declared = |name: &str| {
        let keys: Vec<_> = declarations
            .nominal_spans
            .iter()
            .filter(|(_, span)| &source[span.start..span.end] == name)
            .map(|(key, _)| key.clone())
            .collect();
        let [key] = keys.as_slice() else {
            panic!("one declared {name}: {keys:?}")
        };
        declarations
            .units
            .unit(key.clone())
            .expect("admitted source unit")
            .id()
            .clone()
    };
    let m = declared("m");
    let inch = declared("inch");
    let graph = declarations
        .check(CheckingLimits::default())
        .expect("original conversion checks");
    (CheckedPackage::link(graph), m, inch)
}

fn limits(bits: u64, digits: u64, edges: u64, work: u64, results: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: bits,
        decimal_digits: digits,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: edges,
        value_occurrences: 1,
        work_units: work,
        result_units: results,
    }
}

fn decimal_type(bound: i64, scale: u64, mode: RoundingMode) -> DecimalType {
    DecimalType::new(
        Integer::from(-bound),
        Integer::from(bound),
        scale,
        scale,
        mode,
    )
    .expect("well formed original target")
}

/// Compare mechanical representation transport, retaining the native typed result for assertions.
fn compare(
    package: &CheckedPackage,
    input: &Quantity,
    m: &UnitId,
    target: &DecimalType,
    limits: ScalarLimits,
    expected: Option<(i64, u32, bool)>,
) -> (Outcome<Conversion>, Meter) {
    let function = package
        .graph()
        .function_by_identity(
            package
                .graph()
                .function_identity("observe")
                .expect("original function identity"),
        )
        .expect("original function body");
    let quire_exact::ValueType::CarriedQuantity(source_target) = function.body.value_type() else {
        panic!("original checked conversion retains its carried target");
    };
    assert_eq!(source_target.unit(), m);
    assert_eq!(
        source_target.carrier(),
        &QuantityCarrier::Decimal(target.clone())
    );
    let units = package.graph().scope().types().units();
    let mut native_meter = Meter::new(limits);
    let native = convert_quantity(
        units.resolve(input).expect("original source unit context"),
        units.get(m.clone()).expect("original target unit"),
        &QuantityTarget::Carried(QuantityCarrier::Decimal(target.clone())),
        &mut native_meter,
    )
    .expect("native conversion is well typed");
    if let Some((coefficient, scale, rounded)) = expected {
        assert_decimal(&native, coefficient, scale);
        let Outcome::Completed(conversion) = &native else {
            unreachable!()
        };
        let ConvertedValue::Decimal(result) = conversion.value() else {
            unreachable!()
        };
        if rounded {
            let loss = result
                .loss()
                .expect("U07 retains the complete rounding loss");
            assert_eq!(loss.exact_numerator(), &Integer::from(127));
            assert_eq!(loss.exact_denominator(), Integer::from(5000));
            assert_eq!(loss.rounded_coefficient(), &Integer::from(3));
            assert_eq!(loss.rounded_scale(), 2);
            assert_eq!(loss.mode(), RoundingMode::NearestEven);
        } else {
            assert!(result.loss().is_none());
        }
    }
    let mut source_meter = Meter::new(limits);
    let evaluation =
        qsl_eval::value::evaluate_closed(package, function.body, function.slots, &mut source_meter)
            .expect("original closed conversion evaluates");
    let FamilyOutcome::Evaluated(source) = evaluation.outcome else {
        panic!("conversion must reach value evaluation");
    };
    let mut losses = Vec::new();
    let lifted = match &native {
        Outcome::Completed(conversion) => {
            assert_eq!(conversion.source(), input);
            assert_eq!(
                conversion.unit(),
                units.get(m.clone()).expect("original target unit")
            );
            let ConvertedValue::Decimal(result) = conversion.value() else {
                panic!("native carried decimal representation: {conversion:?}")
            };
            if let Some(loss) = result.loss() {
                losses.push(ValueLoss::Decimal(loss.clone()));
            }
            Outcome::Completed(Value::Quantity(Quantity::new(
                result.value().normalized().to_rational(),
                m.clone(),
            )))
        }
        Outcome::Refused(cause) => Outcome::Refused(cause.clone()),
        Outcome::Undefined(cause) => Outcome::Undefined(cause.clone()),
        Outcome::Incomplete(cause) => Outcome::Incomplete(cause.clone()),
    };
    assert_eq!(
        source, lifted,
        "full original source/native conversion outcome"
    );
    assert_eq!(
        evaluation
            .losses
            .iter()
            .map(|loss| loss.loss.clone())
            .collect::<Vec<_>>(),
        losses
    );
    for loss in &evaluation.losses {
        assert_eq!(&loss.location, function.body.location());
    }
    if matches!(native, Outcome::Completed(_)) {
        assert!(evaluation.location.is_none());
    } else {
        assert_eq!(evaluation.location.as_ref(), Some(function.body.location()));
    }
    assert_eq!(
        source_meter.admitted_charges(),
        native_meter.admitted_charges()
    );
    for kind in LimitKind::ALL {
        assert_eq!(
            source_meter.consumed(kind),
            native_meter.consumed(kind),
            "{kind:?}"
        );
    }
    (native, native_meter)
}

fn rational(n: i64, d: i64) -> Rational {
    Rational::new(Integer::from(n), Integer::from(d)).expect("nonzero denominator")
}

fn assert_decimal(outcome: &Outcome<Conversion>, coefficient: i64, scale: u32) {
    let Outcome::Completed(conversion) = outcome else {
        panic!("completed native decimal: {outcome:?}")
    };
    let ConvertedValue::Decimal(result) = conversion.value() else {
        panic!("decimal carrier")
    };
    assert_eq!(
        result.value().representation(),
        Decimal::new(Integer::from(coefficient), scale).representation()
    );
}

fn stop(outcome: &Outcome<Conversion>, point: ChargePoint, kind: LimitKind, consumed: u64) {
    let Outcome::Incomplete(record) = outcome else {
        panic!("expected denial: {outcome:?}")
    };
    assert_eq!(record.charge_point, point);
    assert_eq!(record.limit_kind, kind);
    assert_eq!(record.consumed, consumed);
    assert_eq!(record.next_charge, Integer::from(1));
}

#[trace("TC-112", "QSpec-TC-187")]
#[test]
fn u07_original_carried_exact_and_nearest_even_preserve_loss_and_precedence() {
    bind("U07", "Convert `1 in` to decimal metres at scale two", "`exact`: refused; `nearest-even`: `(3,2) m` plus `{ exact_numerator: 127, exact_denominator: 5000, rounded_coefficient: 3, rounded_scale: 2, mode: nearest-even }`");
    for (mode, spelling) in [
        (RoundingMode::Exact, "exact"),
        (RoundingMode::NearestEven, "nearest-even"),
    ] {
        let target = decimal_type(100, 2, mode);
        let (package, m, inch) = checked(&format!("Decimal[-100,100;2,2;{spelling}]"), "1 inch");
        let input = Quantity::new(rational(1, 1), inch);
        for allowance in [
            limits(15, 5, 1, 6, 1),
            limits(15, 5, 1, 4, 1),
            limits(15, 5, 1, 6, 0),
        ] {
            let (outcome, meter) = compare(
                &package,
                &input,
                &m,
                &target,
                allowance,
                (mode == RoundingMode::NearestEven
                    && allowance.work_units == 6
                    && allowance.result_units == 1)
                    .then_some((3, 2, true)),
            );
            if mode == RoundingMode::Exact {
                assert_eq!(
                    outcome,
                    Outcome::Refused(Refusal::InexactConversion {
                        target: InexactTarget::Decimal(Box::new(target.clone()))
                    })
                );
                assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
            } else if allowance.work_units == 4 {
                stop(
                    &outcome,
                    ChargePoint::UnitTargetDomain,
                    LimitKind::WorkUnits,
                    4,
                );
            } else if allowance.result_units == 0 {
                stop(
                    &outcome,
                    ChargePoint::UnitResultRetain,
                    LimitKind::ResultUnits,
                    0,
                );
            }
            let points: Vec<_> = meter
                .admitted_charges()
                .iter()
                .map(|charge| charge.point)
                .collect();
            assert_eq!(
                &points[..4],
                &[
                    ChargePoint::UnitIdentityRead,
                    ChargePoint::UnitEdge,
                    ChargePoint::UnitRationalArithmetic,
                    ChargePoint::UnitRationalArithmetic
                ]
            );
            if mode == RoundingMode::Exact {
                assert_eq!(points.len(), 4);
            }
        }
    }
}

#[trace("TC-112", "QSpec-TC-187")]
#[test]
fn u08_original_carried_boundaries_refuse_atomically_after_target_domain() {
    bind("U08", "Convert `-2 m`, `2 m`, `-3 m`, `3 m` into target Decimal coefficient domain `[-2,2]`, scale zero", "`-2` and `2` succeed; one-below/one-over refuse atomically");
    let target = decimal_type(2, 0, RoundingMode::Exact);
    for (n, operand) in [
        (-2, "rational(-2,1) m"),
        (2, "2 m"),
        (-3, "rational(-3,1) m"),
        (3, "3 m"),
    ] {
        let (package, m, _) = checked("Decimal[-2,2;0,0;exact]", operand);
        let input = Quantity::new(rational(n, 1), m.clone());
        for work in [3, 1] {
            let (outcome, meter) = compare(
                &package,
                &input,
                &m,
                &target,
                limits(2, 1, 0, work, 1),
                (work == 3 && n.abs() == 2).then_some((n, 0, false)),
            );
            if work == 1 {
                stop(
                    &outcome,
                    ChargePoint::UnitTargetDomain,
                    LimitKind::WorkUnits,
                    1,
                );
            } else if n.abs() == 2 {
                assert_decimal(&outcome, n, 0);
                let Outcome::Completed(conversion) = &outcome else {
                    unreachable!()
                };
                let ConvertedValue::Decimal(result) = conversion.value() else {
                    unreachable!()
                };
                assert!(result.loss().is_none());
            } else {
                assert_eq!(
                    outcome,
                    Outcome::Refused(Refusal::DecimalOutOfDomain {
                        target: Box::new(target.clone())
                    })
                );
                assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
                assert_eq!(
                    meter
                        .admitted_charges()
                        .iter()
                        .map(|c| c.point)
                        .collect::<Vec<_>>(),
                    [ChargePoint::UnitIdentityRead, ChargePoint::UnitTargetDomain]
                );
            }
        }
    }
}
