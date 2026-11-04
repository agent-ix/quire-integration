// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-185 decimal qualification, composed through QSL and the exact kernel.
//!
//! QSpec owns the vector identities. The assertions below execute each
//! vector; they do not derive expected behavior by parsing its prose.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::{
    check::{CheckCause, CheckRefusal, PackageDeclarations},
    model::{intake::SelectedModels, object_environment::ObjectEnvironment},
};
use quire_eval::{CallFailure, FamilyOutcome};
use quire_exact::{
    evaluate_decimal, order_numbers, planned_equality, ChargePoint, Decimal, DecimalOperation,
    DecimalResult, DecimalType, IllTypedCause, Integer, LimitKind, Meter, OrderedOperands,
    OrderingOperator, Outcome, Rational, Refusal, RoundingMode, ScalarLimits, Undefined, Value,
};
use quire_semantic_value::{call::InputRefusal, checking::CheckingLimits, loss::ValueLoss};
use quire_specification::{vectors_of, TestCase};

const UNLIMITED: ScalarLimits = ScalarLimits {
    integer_bits: u64::MAX,
    decimal_digits: u64::MAX,
    scale_expansion: u64::MAX,
    text_input_bytes: u64::MAX,
    text_scalars: u64::MAX,
    normalized_scalars: u64::MAX,
    unit_edges: u64::MAX,
    value_occurrences: u64::MAX,
    work_units: u64::MAX,
    result_units: u64::MAX,
};

fn decimal(coefficient: i64, scale: u32) -> Decimal {
    Decimal::new(Integer::from(coefficient), scale)
}

fn target(lower: i64, upper: i64, first: u64, last: u64, mode: RoundingMode) -> DecimalType {
    DecimalType::new(
        Integer::from(lower),
        Integer::from(upper),
        first,
        last,
        mode,
    )
    .expect("the target is well formed")
}

fn decimal_result(outcome: Outcome<DecimalResult>, coefficient: i64, scale: u32, has_loss: bool) {
    let Outcome::Completed(result) = outcome else {
        panic!("expected a decimal result, got {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(coefficient, scale).representation()
    );
    assert_eq!(result.loss().is_some(), has_loss);
}

fn denied<T: std::fmt::Debug>(
    outcome: Outcome<T>,
    kind: LimitKind,
    point: ChargePoint,
    consumed: u64,
    next: u64,
) {
    let Outcome::Incomplete(record) = outcome else {
        panic!("expected an incomplete outcome, got {outcome:?}");
    };
    assert_eq!(record.limit_kind, kind);
    assert_eq!(record.charge_point, point);
    assert_eq!(record.consumed, consumed);
    assert_eq!(record.next_charge, Integer::from(next));
}

fn rt(
    operation: DecimalOperation<'_>,
    ty: &DecimalType,
    limits: ScalarLimits,
) -> (Outcome<DecimalResult>, Meter) {
    let mut meter = Meter::new(limits);
    let outcome = evaluate_decimal(operation, ty, &mut meter);
    (outcome, meter)
}

/// A direct QSL function call. Parameters keep operand construction outside
/// the measured operation; QSL charges one extra `function.call` work unit.
fn qsl_binary(
    operator: &str,
    left_type: &str,
    right_type: &str,
    result_type: &str,
    left: Decimal,
    right: Decimal,
    limits: ScalarLimits,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function f using v(a: {left_type}, b: {right_type}): {result_type} pure {{ a {operator} b }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-decimal"),
        "decimal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the decimal function");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the decimal function");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the decimal function")
    .check(CheckingLimits::default())
    .expect("S4 checks the decimal function");
    let package = CheckedPackage::link(graph);
    let mut meter = Meter::new(limits);
    let evaluation = package
        .call(
            &QualifiedName::unqualified("f").expect("f is an identifier"),
            vec![Value::Decimal(left), Value::Decimal(right)],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the decimal function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!("expected a kernel outcome, got {:?}", evaluation.outcome);
    };
    let losses = evaluation
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect();
    (outcome, losses, meter)
}

fn qsl_convert(
    input_type: &str,
    result_type: &str,
    input: Value,
    limits: ScalarLimits,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function f using v(a: {input_type}): {result_type} pure {{ convert<{result_type}>(a) }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-decimal-convert"),
        "decimal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses decimal conversion");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds decimal conversion");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles decimal conversion")
    .check(CheckingLimits::default())
    .expect("S4 checks decimal conversion");
    let package = CheckedPackage::link(graph);
    let mut meter = Meter::new(limits);
    let evaluation = package
        .call(
            &QualifiedName::unqualified("f").expect("f is an identifier"),
            vec![input],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls decimal conversion");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!("expected a kernel outcome, got {:?}", evaluation.outcome);
    };
    let losses = evaluation
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect();
    (outcome, losses, meter)
}

// QSL-1394 / QSL-1619: the parser admits decimal(c,s), but the forms stage still
// refuses its ExactNumber expression. Keep this typed boundary executable so
// these source rows fail visibly when the producer becomes representable.
fn decimal_literal_source_pending(body: &str, result_type: &str) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function f using v(): {result_type} pure {{ {body} }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-decimal-literals"),
        "decimal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses Decimal literal source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let refusal = qsl_forms::build_unit(&parsed)
        .expect_err("QSL-1394 PENDING: ExactNumber must be represented before S4 agreement");
    assert_eq!(
        refusal.cause,
        qsl_forms::FormsCause::UnrepresentedConstruct {
            production: qsl_cst::Production::ExactNumber,
        },
        "QSL-1394 source body {body}"
    );
}

fn conversion_agreement(input: Decimal, input_type: &str, result_type: &str, target: &DecimalType) {
    conversion_agreement_limited(input, input_type, result_type, target, UNLIMITED);
}

fn assert_scalar_meter_agreement(qsl: &Meter, kernel: &Meter) {
    let Some(call) = qsl.admitted_charges().first() else {
        panic!("checked QSL call must charge function.call before scalar evaluation");
    };
    assert_eq!(call.point, ChargePoint::FunctionCall);
    assert_eq!(
        &qsl.admitted_charges()[1..],
        kernel.admitted_charges(),
        "ordered scalar charge point and amount sequence"
    );
    for kind in LimitKind::ALL {
        let offset = u64::from(kind == LimitKind::WorkUnits);
        assert_eq!(
            qsl.consumed(kind),
            kernel.consumed(kind) + offset,
            "{}",
            kind.as_str()
        );
    }
}

fn conversion_agreement_limited(
    input: Decimal,
    input_type: &str,
    result_type: &str,
    target: &DecimalType,
    limits: ScalarLimits,
) {
    let (kernel, kernel_meter) = rt(DecimalOperation::Round(&input), target, limits);
    let qsl_limits = ScalarLimits {
        work_units: limits.work_units.saturating_add(1),
        ..limits
    };
    let (qsl, losses, qsl_meter) =
        qsl_convert(input_type, result_type, Value::Decimal(input), qsl_limits);
    assert_decimal_agreement(kernel, &kernel_meter, qsl, &losses, &qsl_meter);
}

fn agreement(
    operator: &str,
    left_type: &str,
    right_type: &str,
    result_type: &str,
    left: Decimal,
    right: Decimal,
    target: &DecimalType,
    limits: ScalarLimits,
) {
    let operation = match operator {
        "+" => DecimalOperation::Add(&left, &right),
        "*" => DecimalOperation::Multiply(&left, &right),
        "/" => DecimalOperation::Divide(&left, &right),
        other => panic!("unsupported decimal operation {other}"),
    };
    let (kernel, kernel_meter) = rt(operation, target, limits);
    let mut qsl_limits = limits;
    qsl_limits.work_units = qsl_limits.work_units.saturating_add(1);
    let (qsl, losses, qsl_meter) = qsl_binary(
        operator,
        left_type,
        right_type,
        result_type,
        left,
        right,
        qsl_limits,
    );
    assert_decimal_agreement(kernel, &kernel_meter, qsl, &losses, &qsl_meter);
}

fn assert_decimal_agreement(
    kernel: Outcome<DecimalResult>,
    kernel_meter: &Meter,
    qsl: Outcome<Value>,
    losses: &[ValueLoss],
    qsl_meter: &Meter,
) {
    match (kernel, qsl) {
        (Outcome::Completed(expected), Outcome::Completed(Value::Decimal(actual))) => {
            assert_eq!(actual.representation(), expected.value().representation());
            match expected.loss() {
                Some(loss) => assert_eq!(losses, [ValueLoss::Decimal(loss.clone())]),
                None => assert!(losses.is_empty()),
            }
        }
        (Outcome::Undefined(expected), Outcome::Undefined(actual)) => assert_eq!(actual, expected),
        (Outcome::Refused(expected), Outcome::Refused(actual)) => assert_eq!(actual, expected),
        (Outcome::Incomplete(expected), Outcome::Incomplete(actual)) => {
            assert_eq!(actual.limit_kind, expected.limit_kind);
            assert_eq!(actual.charge_point, expected.charge_point);
            assert_eq!(actual.next_charge, expected.next_charge);
            let offset = u64::from(expected.limit_kind == LimitKind::WorkUnits);
            assert_eq!(actual.limit, expected.limit + offset);
            assert_eq!(actual.consumed, expected.consumed + offset);
        }
        (expected, actual) => {
            panic!("QSL/RT decimal disagreement: RT {expected:?}, QSL {actual:?}")
        }
    }
    assert_scalar_meter_agreement(qsl_meter, kernel_meter);
}

#[trace("TC-185", "FR-140")]
#[test]
fn tc185_decimal_vectors() {
    let mut seen = std::collections::BTreeSet::new();
    for vector in vectors_of(TestCase::Tc185) {
        assert!(
            seen.insert(vector.id),
            "duplicate QSpec vector {}",
            vector.id
        );
        match vector.id {
            "D01" => d01(),
            "D02" => d02(),
            "D03" => d03(),
            // QSL-1393 PENDING: linked S4 requires a nonzero divisor proof;
            // a source guard adds charges before the authored operation.
            "D04" => d04_rt_only(),
            "D05" => d05_rt_only(),
            // QSL-1393 PENDING: zero divisor requires kernel-mode source.
            "D06" => d06(),
            "D07" => d07(),
            "D08" => d08(),
            // QSL-1393 PENDING: a source guard changes the denial prefix.
            "D09" => d09_rt_only(),
            "D10" => d10(),
            "D11" => d11(),
            // QSL-1393 PENDING: linked S4 cannot check unguarded divide.
            "D12" => d12_rt_only(),
            // Evaluator-boundary row: 1.50 is outside the result type;
            // D26 is the source-admitted counterpart.
            "D13" => d13_rt_only(),
            // QSL-1393 PENDING: a source guard changes the work limit.
            "D14" => d14_rt_only(),
            // QSL-1393 PENDING: zero divisor requires kernel-mode source.
            "D15" => d15(),
            "D16" => d16(),
            // QSL-1393 PENDING: linked S4 cannot check unguarded divide.
            "D17" => d17_rt_only(),
            // Evaluator-boundary row: 1.234 exceeds the result scale;
            // D27 is the source-admitted counterpart.
            "D18" => d18_rt_only(),
            "D19" => d19(),
            "D20" => d20(),
            "D21" => d21(),
            "D22" => d22(),
            "D23" => d23(),
            "D24" => d24(),
            // QSL-1394 / QSL-1619 PENDING: literal arms stop at S2; mixed named
            // Decimal admission/refusal still executes at S4.
            "D25" => d25(),
            // QSL-1394 / QSL-1619 PENDING: source literal expression stops at S2;
            // retained-operand RT oracle remains runnable.
            "D26" => d26(),
            // QSL-1394 / QSL-1619 PENDING: source literal expression stops at S2;
            // RT division and denial remain runnable.
            "D27" => d27(),
            // QSL-1394 / QSL-1619 PENDING: S4 type-mismatch checks cannot run until
            // the literal expression passes S2.
            "D28" => d28(),
            other => panic!("unimplemented TC-185 vector {other}"),
        }
    }
    assert!(!seen.is_empty(), "QSpec TC-185 offers no vectors");
}

fn limits(
    bits: u64,
    digits: u64,
    scale: u64,
    occurrences: u64,
    work: u64,
    results: u64,
) -> ScalarLimits {
    ScalarLimits {
        integer_bits: bits,
        decimal_digits: digits,
        scale_expansion: scale,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: occurrences,
        work_units: work,
        result_units: results,
    }
}

fn d01() {
    let left = decimal(10, 1);
    let right = decimal(100, 2);
    assert_eq!(left.normalized(), decimal(1, 0).representation());
    assert_eq!(right.normalized(), left.normalized());
    assert_ne!(left.representation(), right.representation());
    let mut kernel_meter = Meter::new(UNLIMITED);
    assert_eq!(
        planned_equality(
            &Value::Decimal(left.clone()),
            &Value::Decimal(right.clone()),
            &mut kernel_meter,
        ),
        Outcome::Completed(true)
    );
    let (outcome, losses, qsl_meter) = qsl_binary(
        "=",
        "Decimal[0,1000;0,2;exact]",
        "Decimal[0,1000;0,2;exact]",
        "Boolean",
        left,
        right,
        UNLIMITED,
    );
    assert!(matches!(outcome, Outcome::Completed(Value::Boolean(true))));
    assert!(losses.is_empty());
    assert_scalar_meter_agreement(&qsl_meter, &kernel_meter);
}

fn d02() {
    let left = decimal(125, 2);
    let right = decimal(75, 2);
    let ty = target(0, 1000, 0, 2, RoundingMode::Exact);
    let (outcome, _) = rt(DecimalOperation::Add(&left, &right), &ty, UNLIMITED);
    let Outcome::Completed(result) = outcome else {
        panic!("D02 should complete: {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(200, 2).representation()
    );
    assert_eq!(result.value().normalized(), decimal(2, 0).representation());
    assert!(result.loss().is_none());
    agreement(
        "+",
        "Decimal[0,1000;0,2;exact]",
        "Decimal[0,1000;0,2;exact]",
        "Decimal[0,1000;0,2;exact]",
        left,
        right,
        &ty,
        UNLIMITED,
    );
}

fn d03() {
    let expectations = [
        (RoundingMode::TowardZero, 2, -2),
        (RoundingMode::TowardPositive, 3, -2),
        (RoundingMode::TowardNegative, 2, -3),
        (RoundingMode::NearestEven, 2, -2),
        (RoundingMode::NearestAway, 3, -3),
    ];
    for (mode, positive, negative) in expectations {
        let ty = target(-9, 9, 0, 0, mode);
        for (operand, coefficient) in [(decimal(25, 1), positive), (decimal(-25, 1), negative)] {
            let (outcome, meter) = rt(DecimalOperation::Round(&operand), &ty, UNLIMITED);
            let Outcome::Completed(result) = outcome else {
                panic!("rounding should complete: {outcome:?}");
            };
            assert_eq!(
                result.value().representation(),
                decimal(coefficient, 0).representation()
            );
            let loss = result.loss().expect("nonzero discarded digit records loss");
            assert_eq!(
                loss.exact_numerator(),
                &Integer::from(if coefficient < 0 { -5_i64 } else { 5_i64 })
            );
            assert_eq!(loss.exact_denominator(), Integer::from(2_i64));
            assert_eq!(loss.rounded_coefficient(), &Integer::from(coefficient));
            assert_eq!(loss.rounded_scale(), 0);
            assert_eq!(loss.mode(), mode);
            assert_eq!(meter.consumed(LimitKind::WorkUnits), 5);
            conversion_agreement(
                operand,
                "Decimal[-25,25;1,1;exact]",
                &format!("Decimal[-9,9;0,0;{}]", mode.as_str()),
                &ty,
            );
        }
    }
    let ty = target(-9, 9, 0, 0, RoundingMode::Exact);
    for operand in [decimal(25, 1), decimal(-25, 1)] {
        assert!(matches!(
            rt(DecimalOperation::Round(&operand), &ty, UNLIMITED).0,
            Outcome::Refused(Refusal::InexactDecimal { .. })
        ));
    }
    d03_exact_qsl_agreement();
}

fn d04_rt_only() {
    let left = decimal(1, 0);
    let right = decimal(8, 0);
    let ty = target(0, 1000, 3, 3, RoundingMode::Exact);
    decimal_result(
        rt(DecimalOperation::Divide(&left, &right), &ty, UNLIMITED).0,
        125,
        3,
        false,
    );
}

fn d05_rt_only() {
    let left = decimal(1, 0);
    let right = decimal(3, 0);
    let exact = target(-1000, 1000, 0, 2, RoundingMode::Exact);
    assert!(matches!(
        rt(DecimalOperation::Divide(&left, &right), &exact, UNLIMITED).0,
        Outcome::Refused(Refusal::InexactDecimal { .. })
    ));
    let rounded = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    let (outcome, _) = rt(DecimalOperation::Divide(&left, &right), &rounded, UNLIMITED);
    let Outcome::Completed(result) = outcome else {
        panic!("third rounds: {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(33, 2).representation()
    );
    let loss = result.loss().expect("third records loss");
    assert_eq!(loss.exact_numerator(), &Integer::from(1_i64));
    assert_eq!(loss.exact_denominator(), Integer::from(3_i64));
    assert_eq!(loss.rounded_coefficient(), &Integer::from(33_i64));
    assert_eq!(loss.rounded_scale(), 2);
    assert_eq!(loss.mode(), RoundingMode::NearestEven);
}

fn d06() {
    let left = decimal(1, 0);
    let right = decimal(0, 3);
    let ty = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    assert_eq!(
        rt(DecimalOperation::Divide(&left, &right), &ty, UNLIMITED).0,
        Outcome::Undefined(Undefined::DivisionByZero)
    );
}

fn d07() {
    let operand = decimal(25, 1);
    let ty = target(-2, 2, 0, 0, RoundingMode::NearestAway);
    let (outcome, meter) = rt(DecimalOperation::Round(&operand), &ty, UNLIMITED);
    assert!(matches!(
        outcome,
        Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
    ));
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
    assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
    d07_qsl_agreement();
}

fn d08() {
    let ty = target(-2, 2, 0, 0, RoundingMode::Exact);
    for coefficient in [-2, 2] {
        let value = decimal(coefficient, 0);
        decimal_result(
            rt(DecimalOperation::Round(&value), &ty, UNLIMITED).0,
            coefficient,
            0,
            false,
        );
    }
    for coefficient in [-3, 3] {
        let value = decimal(coefficient, 0);
        assert!(matches!(
            rt(DecimalOperation::Round(&value), &ty, UNLIMITED).0,
            Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
        ));
    }
    d08_qsl_agreement();
}

fn d09_rt_only() {
    let left = decimal(1, 0);
    let right = decimal(3, 0);
    let ty = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    let exact_limits = limits(8, 3, 2, 2, 5, 1);
    decimal_result(
        rt(DecimalOperation::Divide(&left, &right), &ty, exact_limits).0,
        33,
        2,
        true,
    );

    let mut meter = Meter::new(exact_limits).with_injected_denial(quire_exact::InjectedDenial {
        point: ChargePoint::DecimalResultRetain,
        occurrence: std::num::NonZeroU64::new(1).unwrap(),
    });
    let denied_outcome = evaluate_decimal(DecimalOperation::Divide(&left, &right), &ty, &mut meter);
    denied(
        denied_outcome,
        LimitKind::WorkUnits,
        ChargePoint::DecimalResultRetain,
        4,
        1,
    );
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
}

fn d10() {
    for (lo, hi, first, last) in [(3_i64, 2_i64, 0, 0), (0, 9, 2, 1), (0, 9, 0, 4_294_967_296)] {
        assert!(DecimalType::new(
            Integer::from(lo),
            Integer::from(hi),
            first,
            last,
            RoundingMode::Exact
        )
        .is_err());
        assert!(!qsl_type_admitted(lo, hi, first, last));
    }
    for (lo, hi, first, last) in [(2_i64, 2_i64, 1, 1), (0, 9, 0, 4_294_967_295)] {
        assert!(DecimalType::new(
            Integer::from(lo),
            Integer::from(hi),
            first,
            last,
            RoundingMode::Exact
        )
        .is_ok());
        assert!(qsl_type_admitted(lo, hi, first, last));
    }
}

fn qsl_type_admitted(lo: i64, hi: i64, first: u64, last: u64) -> bool {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         type T = Decimal[{lo},{hi};{first},{last};exact];\n\
         function f using v(a: T): T pure {{ a }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-d10"),
        "decimal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses D10 type declaration");
    assert!(
        parsed.is_admissible(),
        "D10 syntax: {:?}",
        parsed.diagnostics()
    );
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds D10 type declaration");
    let declarations = match PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    ) {
        Ok(declarations) => declarations,
        Err(refusal) => {
            assert!(
                refusal
                    .errors
                    .iter()
                    .all(|error| error.cause.code() == qsl_foundation::diagnostic::Code::IllTyped),
                "D10 S3 must refuse as ill_typed: {refusal:?}"
            );
            return false;
        }
    };
    match declarations.check(CheckingLimits::default()) {
        Ok(_) => true,
        Err(refusals) => {
            assert!(!refusals.is_empty());
            assert!(
                refusals
                    .iter()
                    .all(|refusal| refusal.cause.code()
                        == qsl_foundation::diagnostic::Code::IllTyped),
                "D10 must refuse as ill_typed: {refusals:?}"
            );
            false
        }
    }
}

fn qsl_membership(type_spelling: &str, value: Decimal) -> bool {
    qsl_membership_limited(type_spelling, value, UNLIMITED)
}

fn qsl_membership_limited(type_spelling: &str, value: Decimal, limits: ScalarLimits) -> bool {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function f using v(a: {type_spelling}): {type_spelling} pure {{ a }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-decimal-membership"),
        "decimal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses membership function");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds membership function");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles membership function")
    .check(CheckingLimits::default())
    .expect("S4 checks membership function");
    let package = CheckedPackage::link(graph);
    let mut meter = Meter::new(limits);
    match package.call(
        &QualifiedName::unqualified("f").expect("f is an identifier"),
        vec![Value::Decimal(value.clone())],
        &ObjectEnvironment::default(),
        &mut meter,
    ) {
        Ok(evaluation) => {
            if let FamilyOutcome::Evaluated(Outcome::Incomplete(refusal)) = &evaluation.outcome {
                assert_eq!(limits.work_units, 0);
                assert_eq!(refusal.limit_kind, LimitKind::WorkUnits);
                assert_eq!(refusal.charge_point, ChargePoint::FunctionCall);
                assert_eq!(refusal.limit, 0);
                assert_eq!(refusal.consumed, 0);
                assert_eq!(refusal.next_charge, Integer::from(1_i64));
                assert_eq!(meter.admission_count(), 0);
                return true;
            }
            let FamilyOutcome::Evaluated(Outcome::Completed(Value::Decimal(actual))) =
                evaluation.outcome
            else {
                panic!("member did not retain a decimal: {:?}", evaluation.outcome);
            };
            assert_eq!(actual.representation(), value.representation());
            assert_eq!(meter.consumed(LimitKind::WorkUnits), 1);
            true
        }
        Err(CallFailure::Input(InputRefusal::WrongValueKind { parameter: 0 })) => {
            assert_eq!(meter.admission_count(), 0);
            false
        }
        other => panic!("unexpected decimal membership result: {other:?}"),
    }
}

fn d11() {
    let wide = target(0, 10_000, 2, 2, RoundingMode::Exact);
    for value in [
        decimal(1, 0),
        decimal(11, 1),
        decimal(1000, 3),
        decimal(9999, 2),
    ] {
        assert!(wide.contains(&value));
        assert!(qsl_membership("Decimal[0,10000;2,2;exact]", value));
    }
    let narrow = target(0, 9999, 2, 2, RoundingMode::Exact);
    assert!(!narrow.contains(&decimal(100, 0)));
    assert!(narrow.contains(&decimal(9999, 2)));
    assert!(!qsl_membership(
        "Decimal[0,9999;2,2;exact]",
        decimal(100, 0)
    ));
    assert!(qsl_membership(
        "Decimal[0,9999;2,2;exact]",
        decimal(9999, 2)
    ));
    assert!(!target(-9, 9, 0, 2, RoundingMode::Exact).contains(&decimal(1, 3)));
    assert!(!qsl_membership("Decimal[-9,9;0,2;exact]", decimal(1, 3)));
    assert_eq!(
        wide.contains(&decimal(1, 0)),
        wide.contains(&decimal(1000, 3))
    );
}

fn d12_rt_only() {
    let left = decimal(1, 0);
    let right = decimal(3, 0);
    let wide = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    decimal_result(
        rt(DecimalOperation::Divide(&left, &right), &wide, UNLIMITED).0,
        33,
        2,
        true,
    );

    let narrow = target(-10, 10, 0, 2, RoundingMode::NearestEven);
    assert!(matches!(
        rt(DecimalOperation::Divide(&left, &right), &narrow, UNLIMITED).0,
        Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
    ));
}

fn d13_rt_only() {
    let left = decimal(150, 2);
    let right = decimal(2, 0);
    for mode in [RoundingMode::TowardZero, RoundingMode::Exact] {
        let ty = target(-9, 9, 0, 0, mode);
        let base = limits(10, 4, 0, 2, 4, 1);
        decimal_result(
            rt(DecimalOperation::Multiply(&left, &right), &ty, base).0,
            3,
            0,
            false,
        );

        let short_work = ScalarLimits {
            work_units: 3,
            ..base
        };
        denied(
            rt(DecimalOperation::Multiply(&left, &right), &ty, short_work).0,
            LimitKind::WorkUnits,
            ChargePoint::DecimalResultRetain,
            3,
            1,
        );

        let short_bits = ScalarLimits {
            integer_bits: 9,
            ..base
        };
        denied(
            rt(DecimalOperation::Multiply(&left, &right), &ty, short_bits).0,
            LimitKind::IntegerBits,
            ChargePoint::DecimalArithmetic,
            8,
            10,
        );
    }
}

fn d14_rt_only() {
    let left = decimal(1, 0);
    let right = decimal(3, 0);
    let ty = target(-1000, 1000, 0, 2, RoundingMode::Exact);
    let base = limits(8, 3, 2, 2, 3, 1);
    let (outcome, meter) = rt(DecimalOperation::Divide(&left, &right), &ty, base);
    assert!(matches!(
        outcome,
        Outcome::Refused(Refusal::InexactDecimal { .. })
    ));
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 3);

    denied(
        rt(
            DecimalOperation::Divide(&left, &right),
            &ty,
            ScalarLimits {
                work_units: 2,
                ..base
            },
        )
        .0,
        LimitKind::WorkUnits,
        ChargePoint::DecimalArithmetic,
        2,
        1,
    );
}

fn d15() {
    let left = decimal(1, 0);
    let zero = decimal(0, 3);
    let ty = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    let base = limits(8, 3, 2, 2, 1, 1);
    let (outcome, meter) = rt(DecimalOperation::Divide(&left, &zero), &ty, base);
    assert_eq!(outcome, Outcome::Undefined(Undefined::DivisionByZero));
    assert_eq!(meter.consumed(LimitKind::ScaleExpansion), 0);
    denied(
        rt(
            DecimalOperation::Divide(&left, &zero),
            &ty,
            ScalarLimits {
                work_units: 0,
                ..base
            },
        )
        .0,
        LimitKind::WorkUnits,
        ChargePoint::DecimalOperands,
        0,
        1,
    );
}

fn d16() {
    let value = decimal(25, 1);
    let ty = target(-2, 2, 0, 0, RoundingMode::NearestAway);
    let base = limits(5, 2, 0, 1, 4, 0);
    let (outcome, meter) = rt(DecimalOperation::Round(&value), &ty, base);
    assert!(matches!(
        outcome,
        Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
    ));
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
    assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
    denied(
        rt(
            DecimalOperation::Round(&value),
            &ty,
            ScalarLimits {
                work_units: 3,
                ..base
            },
        )
        .0,
        LimitKind::WorkUnits,
        ChargePoint::DecimalRounding,
        3,
        1,
    );
    d16_qsl_agreement();
}

fn d17_rt_only() {
    let left = decimal(100, 2);
    let right = decimal(1, 0);
    let ty = target(-1000, 1000, 0, 2, RoundingMode::Exact);
    let base = limits(7, 3, 1, 2, 4, 1);
    decimal_result(
        rt(DecimalOperation::Divide(&left, &right), &ty, base).0,
        100,
        2,
        false,
    );
}

fn d18_rt_only() {
    let left = decimal(1234, 3);
    let right = decimal(2, 0);
    let ty = target(-1000, 1000, 0, 1, RoundingMode::NearestEven);
    let base = limits(11, 4, 2, 2, 5, 1);
    let (outcome, _) = rt(DecimalOperation::Divide(&left, &right), &ty, base);
    let Outcome::Completed(result) = outcome else {
        panic!("D18 should complete: {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(6, 1).representation()
    );
    let loss = result.loss().expect("D18 rounds");
    assert_eq!(loss.exact_numerator(), &Integer::from(617_i64));
    assert_eq!(loss.exact_denominator(), Integer::from(1000_i64));

    denied(
        rt(
            DecimalOperation::Divide(&left, &right),
            &ty,
            ScalarLimits {
                scale_expansion: 1,
                ..base
            },
        )
        .0,
        LimitKind::ScaleExpansion,
        ChargePoint::DecimalScaleExpansion,
        0,
        2,
    );
}

fn d19() {
    let ty = target(
        0,
        10,
        u64::from(u32::MAX),
        u64::from(u32::MAX),
        RoundingMode::Exact,
    );
    assert!(ty.contains(&decimal(0, 0)));
    assert!(!ty.contains(&decimal(1, 0)));
    let qsl_type = "Decimal[0,10;4294967295,4294967295;exact]";
    let zero = limits(0, 0, 0, 0, 0, 0);
    assert!(qsl_membership_limited(qsl_type, decimal(0, 0), zero));
    assert!(!qsl_membership_limited(qsl_type, decimal(1, 0), zero));
}

fn ordering(
    left: Decimal,
    right: Decimal,
    qsl_type: &str,
    base: ScalarLimits,
    short_bits: u64,
    expected_bits: u64,
    consumed_bits: u64,
) {
    let mut meter = Meter::new(base);
    assert_eq!(
        order_numbers(
            OrderingOperator::Less,
            OrderedOperands::Decimals(&left, &right),
            &mut meter
        ),
        Outcome::Completed(true)
    );
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 3);
    assert_eq!(meter.consumed(LimitKind::ResultUnits), 1);
    assert_eq!(meter.consumed(LimitKind::IntegerBits), expected_bits);
    let mut qsl_limits = base;
    qsl_limits.work_units += 1;
    let (qsl, losses, qsl_meter) = qsl_binary(
        "<",
        qsl_type,
        qsl_type,
        "Boolean",
        left.clone(),
        right.clone(),
        qsl_limits,
    );
    assert!(matches!(qsl, Outcome::Completed(Value::Boolean(true))));
    assert!(losses.is_empty());
    assert_scalar_meter_agreement(&qsl_meter, &meter);
    let mut short_meter = Meter::new(ScalarLimits {
        integer_bits: short_bits,
        ..base
    });
    let kernel_short = order_numbers(
        OrderingOperator::Less,
        OrderedOperands::Decimals(&left, &right),
        &mut short_meter,
    );
    let Outcome::Incomplete(kernel_stop) = &kernel_short else {
        panic!("ordering runtime must stop at arithmetic: {kernel_short:?}");
    };
    let kernel_stop = kernel_stop.clone();
    denied(
        kernel_short,
        LimitKind::IntegerBits,
        ChargePoint::OrderingArithmetic,
        consumed_bits,
        expected_bits,
    );
    let qsl_short_limits = ScalarLimits {
        integer_bits: short_bits,
        work_units: base.work_units + 1,
        ..base
    };
    let (qsl_short, losses, qsl_meter) = qsl_binary(
        "<",
        qsl_type,
        qsl_type,
        "Boolean",
        left,
        right,
        qsl_short_limits,
    );
    let Outcome::Incomplete(qsl_stop) = &qsl_short else {
        panic!("ordering QSL must stop at arithmetic: {qsl_short:?}");
    };
    assert_eq!(*qsl_stop, kernel_stop);
    denied(
        qsl_short,
        LimitKind::IntegerBits,
        ChargePoint::OrderingArithmetic,
        consumed_bits,
        expected_bits,
    );
    assert!(losses.is_empty());
    assert_scalar_meter_agreement(&qsl_meter, &short_meter);
}

fn d20() {
    ordering(
        decimal(15, 1),
        decimal(2, 0),
        "Decimal[0,100;0,1;exact]",
        limits(6, 2, 1, 2, 3, 1),
        5,
        6,
        4,
    );
}

fn d21() {
    ordering(
        decimal(100, 2),
        decimal(2, 0),
        "Decimal[0,100;0,2;exact]",
        limits(9, 3, 2, 2, 3, 1),
        8,
        9,
        7,
    );
}

fn d22() {
    let one = decimal(1, 0);
    let ty = target(0, 1, 0, 1000, RoundingMode::Exact);
    let base = limits(3323, 1001, 1000, 2, 4, 1);
    let (outcome, meter) = rt(DecimalOperation::Multiply(&one, &one), &ty, base);
    let Outcome::Completed(value) = outcome else {
        panic!("D22 should complete: {outcome:?}");
    };
    assert_eq!(value.value().representation().scale(), 1000);
    assert_eq!(value.value().normalized(), one.normalized());
    assert!(value.loss().is_none());
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
    agreement(
        "*",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        one.clone(),
        one.clone(),
        &ty,
        base,
    );
    denied(
        rt(
            DecimalOperation::Multiply(&one, &one),
            &ty,
            ScalarLimits {
                integer_bits: 3322,
                ..base
            },
        )
        .0,
        LimitKind::IntegerBits,
        ChargePoint::DecimalResultRetain,
        2,
        3323,
    );
    agreement(
        "*",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        one.clone(),
        one.clone(),
        &ty,
        ScalarLimits {
            integer_bits: 3322,
            ..base
        },
    );
    denied(
        rt(
            DecimalOperation::Multiply(&one, &one),
            &ty,
            ScalarLimits {
                scale_expansion: 999,
                ..base
            },
        )
        .0,
        LimitKind::ScaleExpansion,
        ChargePoint::DecimalResultRetain,
        0,
        1000,
    );
    agreement(
        "*",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        "Decimal[0,1;0,1000;exact]",
        one.clone(),
        one,
        &ty,
        ScalarLimits {
            scale_expansion: 999,
            ..base
        },
    );
}

fn d23() {
    let one = decimal(1, 0);
    let ty = target(0, 1, 0, u64::from(u32::MAX), RoundingMode::Exact);
    let base = limits(u64::MAX, 64, u64::from(u32::MAX), 2, 4, 1);
    denied(
        rt(DecimalOperation::Multiply(&one, &one), &ty, base).0,
        LimitKind::DecimalDigits,
        ChargePoint::DecimalResultRetain,
        2,
        4_294_967_296,
    );
    agreement(
        "*",
        "Decimal[0,1;0,4294967295;exact]",
        "Decimal[0,1;0,4294967295;exact]",
        "Decimal[0,1;0,4294967295;exact]",
        one.clone(),
        one,
        &ty,
        base,
    );
}

fn d24() {
    let two = decimal(2, 0);
    let three = decimal(3, 0);
    let rounded = target(0, 100, 2, 2, RoundingMode::NearestEven);
    let base = limits(9, 3, 2, 2, 5, 1);
    let (outcome, meter) = rt(DecimalOperation::Divide(&two, &three), &rounded, base);
    let Outcome::Completed(result) = outcome else {
        panic!("D24 should complete: {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(67, 2).representation()
    );
    let loss = result.loss().expect("D24 rounds");
    assert_eq!(loss.exact_numerator(), &Integer::from(2_i64));
    assert_eq!(loss.exact_denominator(), Integer::from(3_i64));
    assert_eq!(loss.rounded_coefficient(), &Integer::from(67_i64));
    assert_eq!(loss.rounded_scale(), 2);
    assert_eq!(loss.mode(), RoundingMode::NearestEven);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 5);

    denied(
        rt(
            DecimalOperation::Divide(&two, &three),
            &rounded,
            ScalarLimits {
                scale_expansion: 1,
                ..base
            },
        )
        .0,
        LimitKind::ScaleExpansion,
        ChargePoint::DecimalScaleExpansion,
        0,
        2,
    );

    let exact = target(0, 100, 2, 2, RoundingMode::Exact);
    let (outcome, meter) = rt(DecimalOperation::Divide(&two, &three), &exact, base);
    assert!(matches!(
        outcome,
        Outcome::Refused(Refusal::InexactDecimal { .. })
    ));
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 3);

    let rational = Rational::new(Integer::from(2_i64), Integer::from(3_i64))
        .expect("2/3 has a nonzero denominator");
    for (result_type, target, run_limits) in [
        ("Decimal[0,100;2,2;nearest-even]", &rounded, base),
        (
            "Decimal[0,100;2,2;nearest-even]",
            &rounded,
            ScalarLimits {
                scale_expansion: 1,
                ..base
            },
        ),
        ("Decimal[0,100;2,2;exact]", &exact, base),
    ] {
        let (kernel, kernel_meter) = rt(DecimalOperation::Divide(&two, &three), target, run_limits);
        let qsl_limits = ScalarLimits {
            work_units: run_limits.work_units + 1,
            ..run_limits
        };
        let (qsl, losses, qsl_meter) = qsl_convert(
            "Rational[0,2;1,3]",
            result_type,
            Value::Rational(rational.clone()),
            qsl_limits,
        );
        assert_decimal_agreement(kernel, &kernel_meter, qsl, &losses, &qsl_meter);
    }
}

fn d25_source(body: &str, result: &str) -> Result<(), Vec<CheckRefusal>> {
    let text = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         type P = Decimal[0,1000;2,2;exact];\n\
         type R = Decimal[0,1000;1,1;exact];\n\
         function f using v(p: P, r: R): {result} pure {{ {body} }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir669-d25"),
        "decimal.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses D25 source");
    assert!(
        parsed.is_admissible(),
        "D25 syntax: {:?}",
        parsed.diagnostics()
    );
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds D25 source");
    PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles D25 source")
    .check(CheckingLimits::default())
    .map(|_| ())
}

fn d25() {
    d25_source("p + convert<P>(r)", "P").expect("S4 admits explicit conversion");
    let refusals = d25_source("p + r", "P").expect_err("S4 refuses mixed Decimal profiles");
    let [refusal] = refusals.as_slice() else {
        panic!("S4 must report one mixed-profile refusal: {refusals:?}");
    };
    assert_eq!(
        refusal.cause,
        CheckCause::IllTyped(IllTypedCause::TypeMismatch)
    );
    // QSL-1394 / QSL-1619 PENDING: ExactNumber expressions do not reach S4 yet.
    for (body, result_type) in [
        ("let v = decimal(15,1) + decimal(225,2) in v", "P"),
        ("decimal(15,1) = decimal(150,2)", "Boolean"),
        ("p + decimal(15,1)", "P"),
        ("decimal(15,1) + decimal(225,2)", "P"),
    ] {
        decimal_literal_source_pending(body, result_type);
    }
}

fn d26() {
    let left = decimal(100, 2);
    let right = decimal(3, 0);
    let base = limits(9, 4, 0, 2, 4, 1);
    for (result_type, mode) in [
        ("Decimal[-9,9;0,0;toward-zero]", RoundingMode::TowardZero),
        ("Decimal[-9,9;0,0;exact]", RoundingMode::Exact),
    ] {
        let ty = target(-9, 9, 0, 0, mode);
        let (outcome, meter) = rt(DecimalOperation::Multiply(&left, &right), &ty, base);
        decimal_result(outcome, 3, 0, false);
        assert_eq!(meter.consumed(LimitKind::WorkUnits), 4);
        assert!(meter
            .admitted_charges()
            .iter()
            .all(|charge| charge.point != ChargePoint::DecimalRounding));
        denied(
            rt(
                DecimalOperation::Multiply(&left, &right),
                &ty,
                ScalarLimits {
                    work_units: 3,
                    ..base
                },
            )
            .0,
            LimitKind::WorkUnits,
            ChargePoint::DecimalResultRetain,
            3,
            1,
        );
        denied(
            rt(
                DecimalOperation::Multiply(&left, &right),
                &ty,
                ScalarLimits {
                    integer_bits: 8,
                    ..base
                },
            )
            .0,
            LimitKind::IntegerBits,
            ChargePoint::DecimalArithmetic,
            7,
            9,
        );
        // QSL-1394 / QSL-1619 PENDING: authored literal body cannot reach S4/S6a.
        decimal_literal_source_pending("decimal(100,2) * decimal(3,0)", result_type);
    }
}

fn d27() {
    let left = decimal(1200, 3);
    let right = decimal(7, 0);
    let ty = target(-1000, 1000, 0, 1, RoundingMode::NearestEven);
    let base = limits(11, 4, 2, 2, 5, 1);
    let (outcome, meter) = rt(DecimalOperation::Divide(&left, &right), &ty, base);
    let Outcome::Completed(result) = outcome else {
        panic!("D27 should complete: {outcome:?}");
    };
    assert_eq!(
        result.value().representation(),
        decimal(2, 1).representation()
    );
    let loss = result.loss().expect("D27 rounds");
    assert_eq!(loss.exact_numerator(), &Integer::from(6_i64));
    assert_eq!(loss.exact_denominator(), Integer::from(35_i64));
    assert_eq!(loss.rounded_coefficient(), &Integer::from(2_i64));
    assert_eq!(loss.rounded_scale(), 1);
    assert_eq!(loss.mode(), RoundingMode::NearestEven);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 5);
    denied(
        rt(
            DecimalOperation::Divide(&left, &right),
            &ty,
            ScalarLimits {
                scale_expansion: 1,
                ..base
            },
        )
        .0,
        LimitKind::ScaleExpansion,
        ChargePoint::DecimalScaleExpansion,
        0,
        2,
    );
    // QSL-1394 / QSL-1619 PENDING: authored literal body cannot reach S4/S6a.
    decimal_literal_source_pending(
        "decimal(1200,3) / decimal(7,0)",
        "Decimal[-1000,1000;0,1;nearest-even]",
    );
}

fn d28() {
    // QSL-1394 / QSL-1619 PENDING: type-mismatch refusals require S4, which ExactNumber
    // cannot reach. Assert the precise S2 producer blocker for all three arms.
    for (body, result_type) in [
        (
            "decimal(150,2) * decimal(2,0)",
            "Decimal[-9,9;0,0;toward-zero]",
        ),
        (
            "decimal(1234,3) / decimal(2,0)",
            "Decimal[-1000,1000;0,1;nearest-even]",
        ),
        (
            "decimal(100,2) * decimal(3,0)",
            "Decimal[-9,9;0,0;toward-zero]",
        ),
    ] {
        decimal_literal_source_pending(body, result_type);
    }
}

fn d03_exact_qsl_agreement() {
    let ty = target(-9, 9, 0, 0, RoundingMode::Exact);
    for value in [decimal(25, 1), decimal(-25, 1)] {
        conversion_agreement(
            value,
            "Decimal[-25,25;1,1;exact]",
            "Decimal[-9,9;0,0;exact]",
            &ty,
        );
    }
}

#[test]
#[ignore = "QSL-1393: S4 CheckCause::Unproved(Nonzero) for a zero divisor"]
fn d06_qsl_probe() {
    let ty = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    agreement(
        "/",
        "Decimal[-1000,1000;0,2;nearest-even]",
        "Decimal[-1000,1000;0,2;nearest-even]",
        "Decimal[-1000,1000;0,2;nearest-even]",
        decimal(1, 0),
        decimal(0, 3),
        &ty,
        UNLIMITED,
    );
}

fn d07_qsl_agreement() {
    let ty = target(-2, 2, 0, 0, RoundingMode::NearestAway);
    conversion_agreement(
        decimal(25, 1),
        "Decimal[25,25;1,1;exact]",
        "Decimal[-2,2;0,0;nearest-away]",
        &ty,
    );
}

fn d08_qsl_agreement() {
    let ty = target(-2, 2, 0, 0, RoundingMode::Exact);
    for coefficient in [-2, 2, -3, 3] {
        conversion_agreement(
            decimal(coefficient, 0),
            "Decimal[-3,3;0,0;exact]",
            "Decimal[-2,2;0,0;exact]",
            &ty,
        );
    }
}

#[test]
#[ignore = "QSL-1393: S4 CheckCause::Unproved(Nonzero) for a zero divisor"]
fn d15_qsl_probe() {
    let ty = target(-1000, 1000, 0, 2, RoundingMode::NearestEven);
    agreement(
        "/",
        "Decimal[-1000,1000;0,2;nearest-even]",
        "Decimal[-1000,1000;0,2;nearest-even]",
        "Decimal[-1000,1000;0,2;nearest-even]",
        decimal(1, 0),
        decimal(0, 3),
        &ty,
        limits(8, 3, 2, 2, 1, 1),
    );
}

fn d16_qsl_agreement() {
    let ty = target(-2, 2, 0, 0, RoundingMode::NearestAway);
    let base = limits(5, 2, 0, 1, 4, 0);
    for limit in [
        base,
        ScalarLimits {
            work_units: 3,
            ..base
        },
    ] {
        conversion_agreement_limited(
            decimal(25, 1),
            "Decimal[25,25;1,1;exact]",
            "Decimal[-2,2;0,0;nearest-away]",
            &ty,
            limit,
        );
    }
}
