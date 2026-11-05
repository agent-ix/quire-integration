// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-193 binary32 addition through checked QSL source and the exact runtime.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use qsl_semantics::value::{admit_ieee_profile, DefinitionRole};
use quire_eval::FamilyOutcome;
use quire_exact::{
    compare_ieee, convert_ieee_width, evaluate_ieee, exact_to_ieee, ieee_to_exact, ChargePoint,
    Decimal, ExactScalar, IeeeComparison, IeeeFlag, IeeeFlags, IeeeOperand, IeeeOperation,
    IeeeResult, IeeeValue, IeeeWidth, IllTypedCause, InjectedDenial, Integer, IntegerInterval,
    LimitKind, Meter, Outcome, Rational, RationalDomain, Refusal, RoundingMode, ScalarLimits,
    Undefined, Value,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::loss::ValueLoss;
use quire_specification::{vectors_of, QualificationVector, TestCase};
use std::num::NonZeroU64;

const LIMITS: ScalarLimits = ScalarLimits {
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

fn selected_vector(id: &str) -> &'static QualificationVector {
    // Branch-gating string comparison: only these explicit QSpec IDs own this slice.
    let rows: Vec<_> = vectors_of(TestCase::Tc193)
        .filter(|vector| vector.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-193 must contain exactly one {id} row; found {}",
            rows.len()
        );
    };
    row
}

fn hex_bits(text: &str) -> Vec<u32> {
    text.split("0x")
        .skip(1)
        .map(|tail| {
            let digits: String = tail
                .chars()
                .take_while(|ch| ch.is_ascii_hexdigit())
                .collect();
            assert!(!digits.is_empty(), "missing hexadecimal digits in {text:?}");
            u32::from_str_radix(&digits, 16).expect("QSpec binary32 bits fit u32")
        })
        .collect()
}

fn hex_bits64(text: &str) -> Vec<u64> {
    text.split("0x")
        .skip(1)
        .map(|tail| {
            let digits: String = tail
                .chars()
                .take_while(|ch| ch.is_ascii_hexdigit())
                .collect();
            assert!(!digits.is_empty(), "missing hexadecimal digits in {text:?}");
            u64::from_str_radix(&digits, 16).expect("QSpec IEEE bits fit u64")
        })
        .collect()
}

fn add_row(row: &QualificationVector) -> (u32, u32, u32, IeeeFlags) {
    let input_bits = hex_bits(row.inputs);
    let expected_bits = hex_bits(row.expected);
    // Branch-gating QSpec ID match: fail when a new row reaches this dispatcher.
    let (input_count, expected_count) = match row.id {
        "F03" | "F04" => (2, 1),
        "F04b" => (5, 2),
        other => panic!("unsupported QSpec TC-193 ADD vector {other}"),
    };
    assert_eq!(input_bits.len(), input_count, "{} input shape", row.id);
    assert_eq!(
        expected_bits.len(),
        expected_count,
        "{} expected shape",
        row.id
    );
    // Branch-gating QSpec expectation spelling: unknown flag prose fails closed.
    let flags = if row.expected.contains("no flag") {
        IeeeFlags::EMPTY
    } else if row.expected.contains("{invalid}") {
        [IeeeFlag::Invalid].into_iter().collect()
    } else {
        panic!("{} has an unrecognized ADD flag expectation", row.id);
    };
    (input_bits[0], input_bits[1], expected_bits[0], flags)
}

#[derive(Clone, Copy)]
enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOperation {
    fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        }
    }

    fn runtime(self, left: IeeeValue, right: IeeeValue) -> IeeeOperation {
        match self {
            Self::Add => IeeeOperation::Add(left, right),
            Self::Subtract => IeeeOperation::Subtract(left, right),
            Self::Multiply => IeeeOperation::Multiply(left, right),
            Self::Divide => IeeeOperation::Divide(left, right),
        }
    }
}

fn observed_binary(
    operation: BinaryOperation,
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    mut meter: Meter,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    assert_eq!(
        left.width(),
        right.width(),
        "the source selects one IEEE width"
    );
    let width = match left.width() {
        IeeeWidth::Binary32 => "Float32",
        IeeeWidth::Binary64 => "Float64",
    };
    let float_type = format!("{width}[{}]", mode.as_str());
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function add using v(a: {float_type}, b: {float_type}): \
         {float_type} pure {{ a {} b }}\n",
        operation.symbol()
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the IEEE source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the IEEE source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the IEEE source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("S4 checks the IEEE source"),
    );
    let evaluation = package
        .call(
            &QualifiedName::unqualified("add").expect("add is an identifier"),
            vec![Value::Float(left), Value::Float(right)],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the checked IEEE function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach IEEE evaluation: {:?}",
            evaluation.outcome
        );
    };
    let losses = evaluation
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect();
    (outcome, losses, meter)
}

fn observed_add(
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    limits: ScalarLimits,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    observed_binary(BinaryOperation::Add, left, right, mode, Meter::new(limits))
}

fn observed_sqrt(
    operand: IeeeValue,
    mode: RoundingMode,
    mut meter: Meter,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    let width = match operand.width() {
        IeeeWidth::Binary32 => "Float32",
        IeeeWidth::Binary64 => "Float64",
    };
    let float_type = format!("{width}[{}]", mode.as_str());
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function root using v(a: {float_type}): {float_type} pure \
         {{ quire::value::ieee::sqrt(a) }}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee-root.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the IEEE square-root source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the IEEE square-root source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the IEEE square-root source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("S4 checks the IEEE square-root source"),
    );
    let evaluation = package
        .call(
            &QualifiedName::unqualified("root").expect("root is an identifier"),
            vec![Value::Float(operand)],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the checked square-root function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach square-root evaluation: {:?}",
            evaluation.outcome
        );
    };
    let losses = evaluation
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect();
    (outcome, losses, meter)
}

fn observed_float_source(
    function: &str,
    arguments: Vec<Value>,
    mut meter: Meter,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    let package = checked_float_source(function);
    let evaluation = package
        .call(
            &QualifiedName::unqualified("f").expect("f is an identifier"),
            arguments,
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the checked IEEE callable");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach IEEE callable evaluation: {:?}",
            evaluation.outcome
        );
    };
    let losses = evaluation
        .losses
        .into_iter()
        .map(|loss| loss.loss)
        .collect();
    (outcome, losses, meter)
}

fn checked_float_source(function: &str) -> CheckedPackage {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{function}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee-callable.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the IEEE callable source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the IEEE callable source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the IEEE callable source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("S4 checks the IEEE callable source"),
    )
}

fn assert_ieee_carrier(function: &str, identity: &str, expected_count: usize) {
    let package = checked_float_source(function);
    let emission = emit_checked(&package).expect("admitted IEEE source emits a checked package");
    let wire: serde_json::Value =
        serde_json::from_slice(emission.package().bytes()).expect("checked package is JSON");
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("checked semantic nodes");
    let count = nodes
        .iter()
        .filter(|node| node["body"]["operation"]["identity"] == identity)
        .count();
    assert_eq!(count, expected_count, "{identity} checked-package carrier");
}

fn assert_scalar_charge_suffix(id: &str, qsl: &Meter, runtime: &Meter) {
    assert_eq!(
        qsl.admitted_charges()[0].point,
        ChargePoint::FunctionCall,
        "{id}"
    );
    assert_eq!(
        &qsl.admitted_charges()[1..],
        runtime.admitted_charges(),
        "{id}"
    );
    for counter in LimitKind::ALL {
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl.consumed(counter),
            runtime.consumed(counter) + envelope,
            "{id} {counter:?}"
        );
    }
}

fn asserted_limit(limits: ScalarLimits, kind: LimitKind) -> u64 {
    match kind {
        LimitKind::IntegerBits => limits.integer_bits,
        LimitKind::ValueOccurrences => limits.value_occurrences,
        LimitKind::WorkUnits => limits.work_units,
        other => panic!("unasserted IEEE conversion limit kind {other:?}"),
    }
}

#[derive(Clone, Copy, Debug)]
enum ConversionExpectation {
    Completed(u64, IeeeFlags),
    Refused(&'static str, &'static str),
    Incomplete(LimitKind, ChargePoint, u64, u64),
}

fn assert_float_conversion(
    id: &str,
    source_ty: &str,
    target_ty: &str,
    identity: &str,
    source_value: Value,
    limits: ScalarLimits,
    expected: ConversionExpectation,
    native: impl FnOnce(&mut Meter) -> Outcome<IeeeResult>,
) {
    let function = format!(
        "function f using v(a: {source_ty}): {target_ty} pure \
         {{ convert<{target_ty}>(a) }}"
    );
    assert_ieee_carrier(&function, identity, 1);
    let source_limits = ScalarLimits {
        work_units: limits.work_units.saturating_add(1),
        ..limits
    };
    let (qsl, losses, qsl_meter) =
        observed_float_source(&function, vec![source_value], Meter::new(source_limits));
    let mut native_meter = Meter::new(limits);
    let runtime = native(&mut native_meter);
    match (expected, qsl, runtime) {
        (
            ConversionExpectation::Completed(bits, flags),
            Outcome::Completed(Value::Float(qsl_value)),
            Outcome::Completed(native),
        ) => {
            assert_eq!(qsl_value, native.value(), "{id} result agreement");
            assert_eq!(
                format!("{qsl_value:?}"),
                format!("{:?}", native.value()),
                "{id} Debug rendering"
            );
            assert_eq!(qsl_value.bits(), bits, "{id} QSpec result");
            assert_eq!(native.flags(), flags, "{id} QSpec flags");
            if flags.is_empty() {
                assert!(losses.is_empty(), "{id} has no flags");
            } else {
                assert_eq!(losses, vec![ValueLoss::IeeeFlags(flags)], "{id} QSL flags");
            }
        }
        (
            ConversionExpectation::Refused(code, cause),
            Outcome::Refused(qsl_refusal),
            Outcome::Refused(native_refusal),
        ) => {
            assert_eq!(qsl_refusal, native_refusal, "{id} typed refusal");
            assert_eq!(qsl_refusal.code(), Some(code), "{id} refusal code");
            assert_eq!(qsl_refusal.cause(), Some(cause), "{id} refusal cause");
            assert!(losses.is_empty(), "{id} refused without flags");
        }
        (
            ConversionExpectation::Incomplete(kind, point, consumed, next),
            Outcome::Incomplete(qsl_shortage),
            Outcome::Incomplete(native_shortage),
        ) => {
            assert_eq!(qsl_shortage.limit_kind, kind, "{id} QSL limit kind");
            assert_eq!(native_shortage.limit_kind, kind, "{id} runtime limit kind");
            assert_eq!(qsl_shortage.charge_point, point, "{id} QSL charge point");
            assert_eq!(
                native_shortage.charge_point, point,
                "{id} runtime charge point"
            );
            assert_eq!(native_shortage.consumed, consumed, "{id} consumed");
            assert_eq!(
                native_shortage.limit,
                asserted_limit(limits, kind),
                "{id} runtime limit"
            );
            assert_eq!(
                native_shortage.next_charge,
                Integer::from(next),
                "{id} next charge"
            );
            let envelope = u64::from(kind == LimitKind::WorkUnits);
            assert_eq!(qsl_shortage.consumed, native_shortage.consumed + envelope);
            assert_eq!(
                qsl_shortage.limit,
                asserted_limit(source_limits, kind),
                "{id} QSL limit"
            );
            assert_eq!(
                qsl_shortage.next_charge,
                Integer::from(next),
                "{id} QSL next charge"
            );
            assert!(losses.is_empty(), "{id} incomplete without flags");
        }
        (expected, qsl, runtime) => {
            panic!("{id} expected {expected:?}; QSL {qsl:?}; runtime {runtime:?}")
        }
    }
    assert_scalar_charge_suffix(id, &qsl_meter, &native_meter);
}

fn rational_domain(
    numerator: (Integer, Integer),
    denominator: (Integer, Integer),
) -> RationalDomain {
    RationalDomain::new(
        IntegerInterval::new(numerator.0, numerator.1).expect("ordered numerator bounds"),
        IntegerInterval::new(denominator.0, denominator.1).expect("ordered denominator bounds"),
    )
    .expect("positive denominator bounds")
}

fn assert_rational_conversion(
    id: &str,
    source_ty: &str,
    target_ty: &str,
    value: IeeeValue,
    domain: &RationalDomain,
    limits: ScalarLimits,
    expected: Option<Outcome<Value>>,
) {
    let function = format!(
        "function f using v(a: {source_ty}): {target_ty} pure {{ convert<{target_ty}>(a) }}"
    );
    assert_ieee_carrier(&function, "quire.op.ieee.to_rational", 1);
    let source_limits = ScalarLimits {
        work_units: limits.work_units.saturating_add(1),
        ..limits
    };
    let (qsl, losses, qsl_meter) = observed_float_source(
        &function,
        vec![Value::Float(value)],
        Meter::new(source_limits),
    );
    let mut native_meter = Meter::new(limits);
    let native = ieee_to_exact(value, domain, &mut native_meter);
    match (&qsl, &native) {
        (Outcome::Completed(Value::Rational(qsl_value)), Outcome::Completed(native)) => {
            assert_eq!(qsl_value, native.value(), "{id} exact value");
            assert_eq!(
                format!("{qsl_value:?}"),
                format!("{:?}", native.value()),
                "{id} Debug rendering"
            );
            if let Some(loss) = native.loss() {
                assert_eq!(
                    losses,
                    vec![ValueLoss::IeeeExact(loss)],
                    "{id} signed-zero loss"
                );
            } else {
                assert!(losses.is_empty(), "{id} lossless conversion");
            }
        }
        (Outcome::Undefined(qsl_value), Outcome::Undefined(native_value)) => {
            assert_eq!(qsl_value, native_value, "{id} undefined cause");
            assert!(losses.is_empty());
        }
        (Outcome::Refused(qsl_value), Outcome::Refused(native_value)) => {
            assert_eq!(qsl_value, native_value, "{id} typed refusal");
            assert!(losses.is_empty());
        }
        (Outcome::Incomplete(qsl_value), Outcome::Incomplete(native_value)) => {
            let (kind, point, consumed, next): (LimitKind, ChargePoint, u64, u64) = match id {
                "F27 tiny bit denial" => (
                    LimitKind::IntegerBits,
                    ChargePoint::IeeeExactIntermediate,
                    64,
                    1075,
                ),
                "F27 retain denial" => (LimitKind::WorkUnits, ChargePoint::IeeeResultRetain, 2, 1),
                "F27 infinity operand denial" => {
                    (LimitKind::WorkUnits, ChargePoint::IeeeOperands, 0, 1)
                }
                "F27 exact intermediate denial" => (
                    LimitKind::WorkUnits,
                    ChargePoint::IeeeExactIntermediate,
                    1,
                    1,
                ),
                other => panic!("unexpected rational conversion shortage {other}"),
            };
            assert_eq!(native_value.limit_kind, kind, "{id} QSpec limit kind");
            assert_eq!(native_value.charge_point, point, "{id} QSpec charge point");
            assert_eq!(native_value.consumed, consumed, "{id} QSpec consumed");
            assert_eq!(
                native_value.limit,
                asserted_limit(limits, kind),
                "{id} runtime limit"
            );
            assert_eq!(
                native_value.next_charge,
                Integer::from(next),
                "{id} QSpec next charge"
            );
            assert_eq!(
                qsl_value.limit_kind, native_value.limit_kind,
                "{id} limit kind"
            );
            assert_eq!(
                qsl_value.charge_point, native_value.charge_point,
                "{id} charge point"
            );
            assert_eq!(
                qsl_value.consumed,
                native_value.consumed + u64::from(native_value.limit_kind == LimitKind::WorkUnits)
            );
            assert_eq!(qsl_value.next_charge, native_value.next_charge);
            assert_eq!(
                qsl_value.limit,
                asserted_limit(source_limits, kind),
                "{id} QSL limit"
            );
            assert_eq!(
                qsl_value.next_charge,
                Integer::from(next),
                "{id} QSL next charge"
            );
            assert!(losses.is_empty());
        }
        _ => panic!("{id} QSL {qsl:?}; native {native:?}"),
    }
    if let Some(expected) = expected {
        match (qsl, expected) {
            (
                Outcome::Completed(Value::Rational(actual)),
                Outcome::Completed(Value::Rational(expected)),
            ) => assert_eq!(actual, expected, "{id} QSpec result"),
            (Outcome::Undefined(actual), Outcome::Undefined(expected)) => {
                assert_eq!(actual, expected, "{id} QSpec undefined")
            }
            (Outcome::Refused(actual), Outcome::Refused(expected)) => {
                assert_eq!(actual, expected, "{id} QSpec refusal")
            }
            (actual, expected) => panic!("{id} QSpec expected {expected:?}, got {actual:?}"),
        }
    }
    assert_scalar_charge_suffix(id, &qsl_meter, &native_meter);
}

fn assert_fma_agreement(id: &str, inputs: [u32; 3], expected_bits: u32, expected_flags: IeeeFlags) {
    let function = "function f using v(a: Float32[nearest-even], b: Float32[nearest-even], \
                    c: Float32[nearest-even]): Float32[nearest-even] pure \
                    { quire::value::ieee::fma(a, b, c) }";
    let operands = inputs.map(IeeeValue::binary32);
    let (qsl, losses, qsl_meter) = observed_float_source(
        function,
        operands.into_iter().map(Value::Float).collect(),
        Meter::new(LIMITS),
    );
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = evaluate_ieee(
        IeeeOperation::FusedMultiplyAdd(operands[0], operands[1], operands[2]),
        RoundingMode::NearestEven,
        &mut runtime_meter,
    )
    .expect("binary32 FMA operands type-check");
    let (Outcome::Completed(Value::Float(qsl_value)), Outcome::Completed(runtime_value)) =
        (qsl, runtime)
    else {
        panic!("{id} FMA must complete on both sides");
    };
    assert_eq!(qsl_value, runtime_value.value(), "{id} FMA bits agree");
    assert_eq!(
        qsl_value.bits(),
        u64::from(expected_bits),
        "{id} QSpec bits"
    );
    assert_eq!(runtime_value.flags(), expected_flags, "{id} QSpec flags");
    if expected_flags.is_empty() {
        assert!(losses.is_empty(), "{id} QSL flags");
    } else {
        assert_eq!(losses, vec![ValueLoss::IeeeFlags(expected_flags)]);
    }
    assert_scalar_charge_suffix(id, &qsl_meter, &runtime_meter);
}

fn float_type(width: IeeeWidth) -> &'static str {
    match width {
        IeeeWidth::Binary32 => "Float32[nearest-even]",
        IeeeWidth::Binary64 => "Float64[nearest-even]",
    }
}

fn assert_intrinsic_comparison(
    id: &str,
    left: IeeeValue,
    right: IeeeValue,
    comparison: IeeeComparison,
    expected: bool,
) {
    assert_eq!(left.width(), right.width(), "{id} selected width");
    let intrinsic = match comparison {
        IeeeComparison::NumericEqual => "numericEqual",
        IeeeComparison::TotalOrder => "totalOrder",
        IeeeComparison::BitIdentical => "bitIdentical",
    };
    let ty = float_type(left.width());
    let function = format!(
        "function f using v(a: {ty}, b: {ty}): Boolean pure \
         {{ quire::value::ieee::{intrinsic}(a, b) }}"
    );
    let (qsl, losses, qsl_meter) = observed_float_source(
        &function,
        vec![Value::Float(left), Value::Float(right)],
        Meter::new(LIMITS),
    );
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = compare_ieee(comparison, left, right, &mut runtime_meter)
        .expect("same-width IEEE comparison type-checks");
    let (Outcome::Completed(Value::Boolean(qsl_value)), Outcome::Completed(runtime_value)) =
        (qsl, runtime)
    else {
        panic!("{id} intrinsic comparison must complete on both sides");
    };
    assert_eq!(qsl_value, runtime_value, "{id} two-sided comparison");
    assert_eq!(qsl_value, expected, "{id} QSpec result");
    assert!(losses.is_empty(), "{id} comparison has no IEEE flag");
    assert_scalar_charge_suffix(id, &qsl_meter, &runtime_meter);
}

#[derive(Clone, Copy, Debug)]
enum FloatRelation {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl FloatRelation {
    fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessOrEqual => "<=",
            Self::Greater => ">",
            Self::GreaterOrEqual => ">=",
        }
    }
}

fn assert_float_relation(
    id: &str,
    left: IeeeValue,
    right: IeeeValue,
    relation: FloatRelation,
    expected: bool,
) {
    assert_eq!(left.width(), right.width(), "{id} selected width");
    let ty = float_type(left.width());
    let symbol = relation.symbol();
    let function = format!("function f using v(a: {ty}, b: {ty}): Boolean pure {{ a {symbol} b }}");
    let (qsl, losses, _qsl_meter) = observed_float_source(
        &function,
        vec![Value::Float(left), Value::Float(right)],
        Meter::new(LIMITS),
    );
    let Outcome::Completed(Value::Boolean(qsl_value)) = qsl else {
        panic!("{id} Float relation must complete: {qsl:?}");
    };
    let mut equality_meter = Meter::new(LIMITS);
    let equality = compare_ieee(
        IeeeComparison::NumericEqual,
        left,
        right,
        &mut equality_meter,
    )
    .expect("same-width IEEE equality type-checks");
    let Outcome::Completed(equal) = equality else {
        panic!("{id} runtime equality must complete: {equality:?}");
    };
    let mut order_meter = Meter::new(LIMITS);
    let forward = compare_ieee(IeeeComparison::TotalOrder, left, right, &mut order_meter)
        .expect("same-width IEEE order type-checks");
    let Outcome::Completed(forward) = forward else {
        panic!("{id} runtime order must complete: {forward:?}");
    };
    let mut reverse_meter = Meter::new(LIMITS);
    let reverse = compare_ieee(IeeeComparison::TotalOrder, right, left, &mut reverse_meter)
        .expect("same-width IEEE reverse order type-checks");
    let Outcome::Completed(reverse) = reverse else {
        panic!("{id} runtime reverse order must complete: {reverse:?}");
    };
    let ordinary = !left.is_nan() && !right.is_nan();
    let runtime_value = match relation {
        FloatRelation::Equal => equal,
        FloatRelation::NotEqual => !equal,
        FloatRelation::Less => ordinary && forward && !equal,
        FloatRelation::LessOrEqual => ordinary && (forward || equal),
        FloatRelation::Greater => ordinary && reverse && !equal,
        FloatRelation::GreaterOrEqual => ordinary && (reverse || equal),
    };
    assert_eq!(qsl_value, runtime_value, "{id} QSL versus RT relation");
    assert_eq!(qsl_value, expected, "{id} QSpec relation");
    assert!(losses.is_empty(), "{id} relation has no IEEE flag");
}

fn checked_add(left: u32, right: u32, mode: RoundingMode) -> (IeeeValue, IeeeFlags, Meter) {
    let (outcome, losses, meter) = observed_add(
        IeeeValue::binary32(left),
        IeeeValue::binary32(right),
        mode,
        LIMITS,
    );
    let Outcome::Completed(Value::Float(value)) = outcome else {
        panic!("QSL did not complete binary32 ADD: {outcome:?}");
    };
    let flags = match losses.as_slice() {
        [] => IeeeFlags::EMPTY,
        [loss] => match loss {
            ValueLoss::IeeeFlags(flags) => *flags,
            other => panic!("QSL returned a non-IEEE loss: {other:?}"),
        },
        other => panic!("QSL returned multiple loss records: {other:?}"),
    };
    (value, flags, meter)
}

fn assert_add_agreement(
    id: &str,
    left: u32,
    right: u32,
    mode: RoundingMode,
    expected_bits: u32,
    expected_flags: IeeeFlags,
) {
    let (qsl_value, qsl_flags, qsl_meter) = checked_add(left, right, mode);
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = evaluate_ieee(
        IeeeOperation::Add(IeeeValue::binary32(left), IeeeValue::binary32(right)),
        mode,
        &mut runtime_meter,
    )
    .expect("same-width IEEE operands type-check");
    let Outcome::Completed(runtime_result) = runtime else {
        panic!("runtime did not complete {id}: {runtime:?}");
    };
    assert_eq!(qsl_value, runtime_result.value(), "{id} bits disagree");
    assert_eq!(qsl_flags, runtime_result.flags(), "{id} flags disagree");
    assert_eq!(
        qsl_value.bits(),
        u64::from(expected_bits),
        "{id} QSpec bits"
    );
    assert_eq!(qsl_flags, expected_flags, "{id} QSpec flags");
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges(),
        "{id} charge sequence"
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + call_work,
            "{id} {counter:?}"
        );
    }
}

fn assert_binary_observation(
    id: &str,
    operation: BinaryOperation,
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    expected: Result<(u64, IeeeFlags), IeeeFlags>,
) {
    let (qsl, losses, qsl_meter) =
        observed_binary(operation, left, right, mode, Meter::new(LIMITS));
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = evaluate_ieee(operation.runtime(left, right), mode, &mut runtime_meter)
        .expect("same-width IEEE operands type-check");
    match (expected, qsl, runtime) {
        (
            Ok((bits, flags)),
            Outcome::Completed(Value::Float(qsl_value)),
            Outcome::Completed(runtime_value),
        ) => {
            assert_eq!(qsl_value, runtime_value.value(), "{id} bits disagree");
            assert_eq!(qsl_value.bits(), bits, "{id} QSpec bits");
            assert_eq!(runtime_value.flags(), flags, "{id} QSpec flags");
            if flags.is_empty() {
                assert!(losses.is_empty(), "{id} QSL flags");
            } else {
                assert_eq!(losses, vec![ValueLoss::IeeeFlags(flags)], "{id} QSL flags");
            }
        }
        (
            Err(would_be),
            Outcome::Refused(Refusal::IeeeNotExact {
                target: qsl_width,
                would_be: qsl_flags,
            }),
            Outcome::Refused(Refusal::IeeeNotExact {
                target: runtime_width,
                would_be: runtime_flags,
            }),
        ) => {
            assert_eq!(qsl_width, left.width(), "{id} QSL target width");
            assert_eq!(runtime_width, left.width(), "{id} runtime target width");
            assert_eq!(qsl_flags, would_be, "{id} QSL would-be flags");
            assert_eq!(runtime_flags, would_be, "{id} runtime would-be flags");
            assert!(losses.is_empty(), "{id} refusal carries no result loss");
        }
        (expected, qsl, runtime) => {
            panic!("{id} expected {expected:?}; QSL {qsl:?}; runtime {runtime:?}")
        }
    }
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall,
        "{id} function envelope"
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges(),
        "{id} charge sequence"
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + call_work,
            "{id} {counter:?}"
        );
    }
}

fn assert_add_observation(
    id: &str,
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    expected: Result<(u64, IeeeFlags), IeeeFlags>,
) {
    assert_binary_observation(id, BinaryOperation::Add, left, right, mode, expected);
}

fn assert_sqrt_observation(
    id: &str,
    operand: u32,
    mode: RoundingMode,
    expected_bits: u32,
    expected_flags: IeeeFlags,
) {
    let operand = IeeeValue::binary32(operand);
    let (qsl, losses, qsl_meter) = observed_sqrt(operand, mode, Meter::new(LIMITS));
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = evaluate_ieee(IeeeOperation::SquareRoot(operand), mode, &mut runtime_meter)
        .expect("binary32 square root type-checks");
    let (Outcome::Completed(Value::Float(qsl_value)), Outcome::Completed(runtime_value)) =
        (qsl, runtime)
    else {
        panic!("{id} square root must complete on both sides");
    };
    assert_eq!(qsl_value, runtime_value.value(), "{id} two-sided bits");
    assert_eq!(
        qsl_value.bits(),
        u64::from(expected_bits),
        "{id} QSpec bits"
    );
    assert_eq!(runtime_value.flags(), expected_flags, "{id} QSpec flags");
    if expected_flags.is_empty() {
        assert!(losses.is_empty(), "{id} has no QSL flags");
    } else {
        assert_eq!(losses, vec![ValueLoss::IeeeFlags(expected_flags)]);
    }
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    for counter in LimitKind::ALL {
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + envelope,
            "{id} {counter:?}"
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum BoundedExpectation {
    Completed(u64, IeeeFlags),
    Refused(IeeeFlags),
    Incomplete(ChargePoint),
}

fn ieee_limits(integer_bits: u64, occurrences: u64, work: u64, results: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: occurrences,
        work_units: work,
        result_units: results,
    }
}

fn assert_bounded_binary(
    id: &str,
    operation: BinaryOperation,
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    limits: ScalarLimits,
    denial: Option<ChargePoint>,
    expected: BoundedExpectation,
) {
    let source_limits = ScalarLimits {
        work_units: limits.work_units + 1,
        ..limits
    };
    let mut source_meter = Meter::new(source_limits);
    let mut runtime_meter = Meter::new(limits);
    if let Some(point) = denial {
        let injected = InjectedDenial {
            point,
            occurrence: NonZeroU64::new(1).expect("one is nonzero"),
        };
        source_meter = source_meter.with_injected_denial(injected);
        runtime_meter = runtime_meter.with_injected_denial(injected);
    }
    let (qsl, losses, qsl_meter) = observed_binary(operation, left, right, mode, source_meter);
    let runtime = evaluate_ieee(operation.runtime(left, right), mode, &mut runtime_meter)
        .expect("same-width IEEE operands type-check");
    match (expected, qsl, runtime) {
        (
            BoundedExpectation::Completed(bits, flags),
            Outcome::Completed(Value::Float(qsl_value)),
            Outcome::Completed(runtime_value),
        ) => {
            assert_eq!(qsl_value, runtime_value.value(), "{id} bits agree");
            assert_eq!(qsl_value.bits(), bits, "{id} QSpec bits");
            assert_eq!(runtime_value.flags(), flags, "{id} QSpec flags");
            if flags.is_empty() {
                assert!(losses.is_empty(), "{id} QSL flags");
            } else {
                assert_eq!(losses, vec![ValueLoss::IeeeFlags(flags)]);
            }
        }
        (
            BoundedExpectation::Refused(would_be),
            Outcome::Refused(Refusal::IeeeNotExact {
                would_be: qsl_flags,
                ..
            }),
            Outcome::Refused(Refusal::IeeeNotExact {
                would_be: runtime_flags,
                ..
            }),
        ) => {
            assert_eq!(qsl_flags, would_be, "{id} QSL would-be flags");
            assert_eq!(runtime_flags, would_be, "{id} runtime would-be flags");
            assert!(losses.is_empty(), "{id} refusal has no result loss");
        }
        (
            BoundedExpectation::Incomplete(point),
            Outcome::Incomplete(qsl_shortage),
            Outcome::Incomplete(runtime_shortage),
        ) => {
            assert_eq!(qsl_shortage.charge_point, point, "{id} QSL denial");
            assert_eq!(runtime_shortage.charge_point, point, "{id} runtime denial");
            assert_eq!(qsl_shortage.limit_kind, LimitKind::WorkUnits);
            assert_eq!(runtime_shortage.limit_kind, LimitKind::WorkUnits);
            assert_eq!(qsl_shortage.consumed, runtime_shortage.consumed + 1);
            assert_eq!(qsl_shortage.limit, runtime_shortage.limit + 1);
            assert_eq!(qsl_shortage.next_charge, runtime_shortage.next_charge);
            assert!(losses.is_empty(), "{id} incomplete has no result loss");
        }
        (expected, qsl, runtime) => {
            panic!("{id} expected bounded {expected:?}; QSL {qsl:?}; runtime {runtime:?}")
        }
    }
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    for counter in LimitKind::ALL {
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + envelope,
            "{id} {counter:?}"
        );
    }
}

fn assert_bounded_sqrt(
    id: &str,
    operand_bits: u32,
    limits: ScalarLimits,
    denial: Option<ChargePoint>,
    expected: Result<(u64, IeeeFlags), ChargePoint>,
) {
    let operand = IeeeValue::binary32(operand_bits);
    let source_limits = ScalarLimits {
        work_units: limits.work_units + 1,
        ..limits
    };
    let denial = denial.map(|point| InjectedDenial {
        point,
        occurrence: NonZeroU64::new(1).expect("one is nonzero"),
    });
    let mut source_meter = Meter::new(source_limits);
    let mut runtime_meter = Meter::new(limits);
    if let Some(denial) = denial {
        source_meter = source_meter.with_injected_denial(denial);
        runtime_meter = runtime_meter.with_injected_denial(denial);
    }
    let (qsl, losses, qsl_meter) = observed_sqrt(operand, RoundingMode::NearestEven, source_meter);
    let runtime = evaluate_ieee(
        IeeeOperation::SquareRoot(operand),
        RoundingMode::NearestEven,
        &mut runtime_meter,
    )
    .expect("binary32 square root type-checks");
    match (expected, qsl, runtime) {
        (
            Ok((bits, flags)),
            Outcome::Completed(Value::Float(qsl_value)),
            Outcome::Completed(runtime_value),
        ) => {
            assert_eq!(qsl_value, runtime_value.value(), "{id} bits agree");
            assert_eq!(qsl_value.bits(), bits, "{id} QSpec bits");
            assert_eq!(runtime_value.flags(), flags, "{id} QSpec flags");
            if flags.is_empty() {
                assert!(losses.is_empty(), "{id} QSL flags");
            } else {
                assert_eq!(losses, vec![ValueLoss::IeeeFlags(flags)]);
            }
        }
        (Err(point), Outcome::Incomplete(qsl_shortage), Outcome::Incomplete(runtime_shortage)) => {
            assert_eq!(qsl_shortage.charge_point, point, "{id} QSL denial");
            assert_eq!(runtime_shortage.charge_point, point, "{id} runtime denial");
            assert_eq!(qsl_shortage.limit_kind, LimitKind::WorkUnits);
            assert_eq!(runtime_shortage.limit_kind, LimitKind::WorkUnits);
            assert_eq!(qsl_shortage.consumed, runtime_shortage.consumed + 1);
            assert_eq!(qsl_shortage.limit, runtime_shortage.limit + 1);
            assert_eq!(qsl_shortage.next_charge, runtime_shortage.next_charge);
            assert!(losses.is_empty(), "{id} denial has no result loss");
        }
        (expected, qsl, runtime) => {
            panic!("{id} expected bounded {expected:?}; QSL {qsl:?}; runtime {runtime:?}")
        }
    }
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    for counter in LimitKind::ALL {
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + envelope,
            "{id} {counter:?}"
        );
    }
}

fn observed_numeric_equal(left: u32, right: u32, mut meter: Meter) -> (Outcome<Value>, Meter) {
    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function equal using v(a: Float32[nearest-even], b: Float32[nearest-even]): \
         Boolean pure { a = b }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee-equal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the IEEE comparison source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the IEEE comparison source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the IEEE comparison source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("S4 checks the IEEE comparison source"),
    );
    let evaluation = package
        .call(
            &QualifiedName::unqualified("equal").expect("equal is an identifier"),
            vec![
                Value::Float(IeeeValue::binary32(left)),
                Value::Float(IeeeValue::binary32(right)),
            ],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the checked IEEE comparison function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach IEEE comparison: {:?}",
            evaluation.outcome
        );
    };
    assert!(evaluation.losses.is_empty(), "numeric equality has no loss");
    (outcome, meter)
}

fn checked_numeric_equal(left: u32, right: u32) -> (bool, Meter) {
    let (outcome, meter) = observed_numeric_equal(left, right, Meter::new(LIMITS));
    let Outcome::Completed(Value::Boolean(equal)) = outcome else {
        panic!("QSL did not complete IEEE comparison: {outcome:?}");
    };
    (equal, meter)
}

fn assert_numeric_equal(id: &str, left: u32, right: u32, expected: bool) {
    let (qsl_equal, qsl_meter) = checked_numeric_equal(left, right);
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = compare_ieee(
        IeeeComparison::NumericEqual,
        IeeeValue::binary32(left),
        IeeeValue::binary32(right),
        &mut runtime_meter,
    )
    .expect("same-width IEEE comparison type-checks");
    let Outcome::Completed(runtime_equal) = runtime else {
        panic!("runtime did not complete {id} numeric equality: {runtime:?}");
    };
    assert_eq!(
        qsl_equal, runtime_equal,
        "{id} numeric equality disagreement"
    );
    assert_eq!(qsl_equal, expected, "{id} QSpec numeric equality");
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges(),
        "{id} charge sequence"
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + call_work,
            "{id} {counter:?}"
        );
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f01_f02_numeric_equal_charges_agree() {
    let f01 = selected_vector("F01");
    let f01_bits = hex_bits(f01.inputs);
    assert_eq!(f01_bits.len(), 2, "F01 has two binary32 operands");
    assert_numeric_equal("F01", f01_bits[0], f01_bits[1], true);

    let f02 = selected_vector("F02");
    let f02_bits = hex_bits(f02.inputs);
    assert_eq!(f02_bits.len(), 2, "F02 has two binary32 operands");
    for bits in f02_bits {
        assert_numeric_equal("F02", bits, bits, false);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f17_numeric_equality_accounts_for_named_charges() {
    let f17 = selected_vector("F17");
    let f17_bits = hex_bits(f17.inputs);
    assert_eq!(f17_bits.len(), 2, "F17 has two binary32 operands");
    let runtime_limits = ieee_limits(32, 2, 2, 1);
    let source_limits = ScalarLimits {
        work_units: 3,
        ..runtime_limits
    };
    for denial in [
        None,
        Some(ChargePoint::IeeeOperands),
        Some(ChargePoint::IeeeResultRetain),
    ] {
        let mut qsl_meter = Meter::new(source_limits);
        let mut runtime_meter = Meter::new(runtime_limits);
        if let Some(point) = denial {
            let injected = InjectedDenial {
                point,
                occurrence: NonZeroU64::new(1).expect("one is nonzero"),
            };
            qsl_meter = qsl_meter.with_injected_denial(injected);
            runtime_meter = runtime_meter.with_injected_denial(injected);
        }
        let (qsl, qsl_meter) = observed_numeric_equal(f17_bits[0], f17_bits[1], qsl_meter);
        let runtime = compare_ieee(
            IeeeComparison::NumericEqual,
            IeeeValue::binary32(f17_bits[0]),
            IeeeValue::binary32(f17_bits[1]),
            &mut runtime_meter,
        )
        .expect("same-width IEEE comparison type-checks");
        match (denial, qsl, runtime) {
            (
                None,
                Outcome::Completed(Value::Boolean(qsl_equal)),
                Outcome::Completed(runtime_equal),
            ) => {
                assert!(!qsl_equal, "F17 QSpec false");
                assert_eq!(qsl_equal, runtime_equal, "F17 two-sided Boolean");
            }
            (
                Some(point),
                Outcome::Incomplete(qsl_shortage),
                Outcome::Incomplete(runtime_shortage),
            ) => {
                assert_eq!(qsl_shortage.charge_point, point, "F17 QSL named denial");
                assert_eq!(
                    runtime_shortage.charge_point, point,
                    "F17 runtime named denial"
                );
                assert_eq!(qsl_shortage.consumed, runtime_shortage.consumed + 1);
                assert_eq!(qsl_shortage.limit_kind, runtime_shortage.limit_kind);
                assert_eq!(runtime_shortage.limit_kind, LimitKind::WorkUnits);
                assert_eq!(runtime_shortage.limit, runtime_shortage.consumed);
                assert_eq!(runtime_shortage.next_charge, Integer::from(1_i64));
                let expected_consumed = match point {
                    ChargePoint::IeeeOperands => 0,
                    ChargePoint::IeeeResultRetain => 1,
                    other => panic!("F17 has no authored denial at {other:?}"),
                };
                assert_eq!(runtime_shortage.consumed, expected_consumed);
                assert_eq!(qsl_shortage.limit, runtime_shortage.limit + 1);
                assert_eq!(qsl_shortage.next_charge, runtime_shortage.next_charge);
            }
            (denial, qsl, runtime) => {
                panic!("F17 denial {denial:?}: QSL {qsl:?}, runtime {runtime:?}")
            }
        }
        assert_eq!(
            qsl_meter.admitted_charges()[0].point,
            ChargePoint::FunctionCall
        );
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );
        for counter in LimitKind::ALL {
            let envelope = u64::from(counter == LimitKind::WorkUnits);
            assert_eq!(
                qsl_meter.consumed(counter),
                runtime_meter.consumed(counter) + envelope,
                "F17 {counter:?} after {denial:?}"
            );
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_selected_add_rows_agree_from_checked_source_to_runtime() {
    for id in ["F03", "F04", "F04b"] {
        let row = selected_vector(id);
        let (left, right, expected_bits, expected_flags) = add_row(row);
        assert_add_agreement(
            id,
            left,
            right,
            RoundingMode::NearestEven,
            expected_bits,
            expected_flags,
        );
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f07_f08_rounding_modes_agree_from_checked_source_to_runtime() {
    let inexact: IeeeFlags = [IeeeFlag::Inexact].into_iter().collect();
    for id in ["F07", "F08"] {
        let row = selected_vector(id);
        let inputs = hex_bits(row.inputs);
        let expected = hex_bits(row.expected);
        assert_eq!(inputs.len(), 2, "{id} has two binary32 inputs");
        assert_eq!(expected.len(), 2, "{id} has two expected bit patterns");
        // Branch-gating QSpec expectation spelling: every mode raises inexact.
        assert!(row.expected.contains("{inexact}"), "{id} flags inexact");
        // Branch-gating QSpec ID match: the rounded-up direction differs by sign.
        let up_mode = match id {
            "F07" => RoundingMode::TowardPositive,
            "F08" => RoundingMode::TowardNegative,
            other => panic!("unsupported rounding vector {other}"),
        };
        for mode in [
            RoundingMode::NearestEven,
            RoundingMode::NearestAway,
            RoundingMode::TowardZero,
            RoundingMode::TowardPositive,
            RoundingMode::TowardNegative,
        ] {
            let bits = if mode == RoundingMode::NearestAway || mode == up_mode {
                expected[1]
            } else {
                expected[0]
            };
            assert_add_agreement(id, inputs[0], inputs[1], mode, bits, inexact);
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc193_f06_cross_width_comparison_refuses_before_evaluation() {
    let row = selected_vector("F06");
    let operands: Vec<_> = row
        .inputs
        .split("0x")
        .skip(1)
        .map(|tail| {
            let digits: String = tail
                .chars()
                .take_while(|ch| ch.is_ascii_hexdigit())
                .collect();
            u64::from_str_radix(&digits, 16).expect("QSpec IEEE bits are hexadecimal")
        })
        .collect();
    let [left, right] = operands.as_slice() else {
        panic!("F06 has two authored operands, found {}", operands.len());
    };
    let left = u32::try_from(*left).expect("F06 left operand is binary32");
    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function mixed using v(a: Float32[nearest-even], b: Float64[nearest-even]): \
         Boolean pure { a = b }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee-mixed.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the mixed-width source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the mixed-width source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the mixed-width source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let refusals = declarations
        .check(CheckingLimits::default())
        .expect_err("S4 refuses mixed IEEE widths");
    let [refusal] = refusals.as_slice() else {
        panic!("S4 must report one mixed-width refusal: {refusals:?}");
    };
    assert_eq!(
        refusal.cause,
        CheckCause::IllTyped(IllTypedCause::TypeMismatch)
    );
    let mut meter = Meter::new(LIMITS);
    let runtime = compare_ieee(
        IeeeComparison::NumericEqual,
        IeeeValue::binary32(left),
        IeeeValue::binary64(*right),
        &mut meter,
    )
    .expect_err("runtime also refuses mixed IEEE widths before evaluation");
    assert_eq!(runtime.cause, IllTypedCause::DistinctIeeeWidths);
    assert!(meter.admitted_charges().is_empty(), "no runtime charge");
}

fn assert_f20_source_admission_refusal(function: &str) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{function}\n"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669"),
        "ieee-f20.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the F20 admission source");
    assert!(
        parsed.is_admissible(),
        "{source:?}: {:?}",
        parsed.diagnostics()
    );
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the F20 admission source");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("S3 assembles the F20 admission source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let refusals = declarations
        .check(CheckingLimits::default())
        .expect_err("S4 refuses F20 before evaluation");
    let [refusal] = refusals.as_slice() else {
        panic!("S4 must report one F20 refusal: {refusals:?}");
    };
    assert_eq!(
        refusal.cause,
        CheckCause::IllTyped(IllTypedCause::TypeMismatch),
        "F20 typed admission refusal for {function}"
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc193_f20_admission_refusals_have_no_charge() {
    let row = selected_vector("F20");
    let operands = hex_bits64(row.inputs);
    assert!(operands.len() >= 2, "F20 names both IEEE widths");
    let left = u32::try_from(operands[0]).expect("F20 first operand is binary32");
    let right = operands[1];
    for function in [
        "function f using v(a: Float32[nearest-even], b: Float64[nearest-even]): Boolean pure { a = b }",
        "function f using v(a: Float32[nearest-even], b: Float64[nearest-even]): Boolean pure { quire::value::ieee::totalOrder(a, b) }",
        "function f using v(a: Float32[nearest-even], b: Float64[nearest-even]): Boolean pure { quire::value::ieee::bitIdentical(a, b) }",
        "function f using v(a: Float32[nearest-even], b: Float64[nearest-even]): Float32[nearest-even] pure { a + b }",
        "function f using v(a: Float32[nearest-even], b: Int[0,1]): Float32[nearest-even] pure { a + b }",
        "function f using v(a: Float32[nearest-even]): Decimal[0,10;0,0;nearest-even] pure { convert<Decimal[0,10;0,0;nearest-even]>(a) }",
        "function f using v(a: Float32[nearest-even]): Int[0,1] pure { convert<Int[0,1]>(a) }",
    ] {
        assert_f20_source_admission_refusal(function);
    }
    let mut meter = Meter::new(ieee_limits(0, 0, 0, 0));
    for comparison in IeeeComparison::ALL {
        let refusal = compare_ieee(
            comparison,
            IeeeValue::binary32(left),
            IeeeValue::binary64(right),
            &mut meter,
        )
        .expect_err("mixed-width IEEE comparison refuses before evaluation");
        assert_eq!(refusal.cause, IllTypedCause::DistinctIeeeWidths);
    }
    let runtime = evaluate_ieee(
        IeeeOperation::Add(IeeeValue::binary32(left), IeeeValue::binary64(right)),
        RoundingMode::NearestEven,
        &mut meter,
    )
    .expect_err("runtime also refuses mixed IEEE widths before evaluation");
    assert_eq!(runtime.cause, IllTypedCause::DistinctIeeeWidths);
    let one = Integer::from(1_i64);
    let mixed = evaluate_ieee(
        IeeeOperation::Add(
            IeeeOperand::Ieee(IeeeValue::binary32(left)),
            IeeeOperand::Exact(ExactScalar::Integer(&one)),
        ),
        RoundingMode::NearestEven,
        &mut meter,
    )
    .expect_err("IEEE plus exact integer refuses before evaluation");
    assert_eq!(mixed.cause, IllTypedCause::IeeeWithExactOperand);
    assert!(
        meter.admitted_charges().is_empty(),
        "F20 refusal has no charge"
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f09_f14_f23_f29_exact_refusal_and_rounding_agree() {
    let inexact: IeeeFlags = [IeeeFlag::Inexact].into_iter().collect();
    let overflow_inexact: IeeeFlags = [IeeeFlag::Overflow, IeeeFlag::Inexact]
        .into_iter()
        .collect();

    let f07 = selected_vector("F07");
    let f07_bits = hex_bits(f07.inputs);
    let [one, half_ulp] = f07_bits.as_slice() else {
        panic!("F07 supplies the two F09 exact-refusal operands");
    };
    let f09 = selected_vector("F09");
    let f09_bits = hex_bits(f09.inputs);
    assert!(f09_bits.contains(&0x3400_0000), "F09 supplies one full ulp");
    // Branch-gating QSpec expectation: F09's first branch must remain a refusal.
    assert!(f09.expected.contains("refused"));
    assert!(hex_bits(f09.expected).contains(&0x3f80_0001));
    assert_add_observation(
        "F09 refusal",
        IeeeValue::binary32(*one),
        IeeeValue::binary32(*half_ulp),
        RoundingMode::Exact,
        Err(inexact),
    );
    assert_add_observation(
        "F09 exact completion",
        IeeeValue::binary32(*one),
        IeeeValue::binary32(0x3400_0000),
        RoundingMode::Exact,
        Ok((0x3f80_0001, IeeeFlags::EMPTY)),
    );

    let f14 = selected_vector("F14");
    let f14_bits = hex_bits(f14.inputs);
    assert_eq!(hex_bits(f14.expected), [0x7f80_0000]);
    let [maximum_left, maximum_right] = f14_bits.as_slice() else {
        panic!("F14 supplies two maximum finite operands");
    };
    for (mode, expectation) in [
        (
            RoundingMode::NearestEven,
            Ok((0x7f80_0000, overflow_inexact)),
        ),
        (RoundingMode::Exact, Err(overflow_inexact)),
    ] {
        assert_add_observation(
            "F14",
            IeeeValue::binary32(*maximum_left),
            IeeeValue::binary32(*maximum_right),
            mode,
            expectation,
        );
    }

    let f23 = selected_vector("F23");
    let f23_bits = hex_bits(f23.inputs);
    assert_eq!(hex_bits(f23.expected), [0x7f80_0000, 0x7f7f_ffff]);
    let [maximum, half_max_ulp] = f23_bits.as_slice() else {
        panic!("F23 supplies two binary32 operands");
    };
    for (mode, expectation) in [
        (
            RoundingMode::NearestEven,
            Ok((0x7f80_0000, overflow_inexact)),
        ),
        (RoundingMode::TowardZero, Ok((0x7f7f_ffff, inexact))),
        (RoundingMode::Exact, Err(overflow_inexact)),
    ] {
        assert_add_observation(
            "F23",
            IeeeValue::binary32(*maximum),
            IeeeValue::binary32(*half_max_ulp),
            mode,
            expectation,
        );
    }

    let f29 = selected_vector("F29");
    let f29_bits = hex_bits(f29.inputs);
    assert_eq!(hex_bits(f29.expected), [0x7f7f_ffff, 0x0080_0000]);
    assert_eq!(f29_bits.len(), 4, "F29 has two binary operations");
    for (mode, expectation) in [
        (RoundingMode::NearestEven, Ok((0x7f7f_ffff, inexact))),
        (RoundingMode::Exact, Err(inexact)),
    ] {
        assert_add_observation(
            "F29 addition",
            IeeeValue::binary32(f29_bits[0]),
            IeeeValue::binary32(f29_bits[1]),
            mode,
            expectation,
        );
    }
    for (mode, expectation) in [
        (RoundingMode::NearestEven, Ok((0x0080_0000, inexact))),
        (RoundingMode::Exact, Err(inexact)),
    ] {
        assert_binary_observation(
            "F29 multiplication",
            BinaryOperation::Multiply,
            IeeeValue::binary32(f29_bits[2]),
            IeeeValue::binary32(f29_bits[3]),
            mode,
            expectation,
        );
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f26_signed_zero_arithmetic_agrees_in_every_mode() {
    let f26 = selected_vector("F26");
    // Branch-gating QSpec input and expectation spelling: the source row must
    // retain the signed-zero cases that this selected test evaluates.
    for operand in ["0x80000000", "0x00000000", "0x3f800000", "0xbf800000"] {
        assert!(f26.inputs.contains(operand), "F26 lacks {operand}");
    }
    assert!(f26.expected.contains("toward-negative"));
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::TowardPositive,
        RoundingMode::TowardNegative,
        RoundingMode::Exact,
    ] {
        let cancellation_bits = if mode == RoundingMode::TowardNegative {
            0x8000_0000
        } else {
            0x0000_0000
        };
        for (operation, left, right, bits) in [
            (BinaryOperation::Add, 0x8000_0000, 0x8000_0000, 0x8000_0000),
            (
                BinaryOperation::Add,
                0x3f80_0000,
                0xbf80_0000,
                cancellation_bits,
            ),
            (
                BinaryOperation::Subtract,
                0x8000_0000,
                0x0000_0000,
                0x8000_0000,
            ),
            (
                BinaryOperation::Subtract,
                0x0000_0000,
                0x0000_0000,
                cancellation_bits,
            ),
            (
                BinaryOperation::Subtract,
                0x3f80_0000,
                0x3f80_0000,
                cancellation_bits,
            ),
        ] {
            assert_binary_observation(
                "F26 arithmetic",
                operation,
                IeeeValue::binary32(left),
                IeeeValue::binary32(right),
                mode,
                Ok((bits, IeeeFlags::EMPTY)),
            );
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f32_ieee_negation_preserves_bits_and_rounding_independence() {
    let row = selected_vector("F32");
    assert!(row.inputs.contains("0x7f800001"));
    assert!(row.inputs.contains("0x7ff0000000000001"));
    let binary32: [(u32, u32); 6] = [
        (0x3f80_0000, 0xbf80_0000),
        (0x0000_0000, 0x8000_0000),
        (0x8000_0000, 0x0000_0000),
        (0x7f80_0000, 0xff80_0000),
        (0x7f80_0001, 0xff80_0001),
        (0xffc0_0005, 0x7fc0_0005),
    ];
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::TowardZero,
        RoundingMode::Exact,
    ] {
        let ty = format!("Float32[{}]", mode.as_str());
        let source = format!("function f using v(a: {ty}): {ty} pure {{ -a }}");
        assert_ieee_carrier(&source, "quire.op.ieee.float32.negate", 1);
        for &(input_bits, expected_bits) in &binary32 {
            let input = IeeeValue::binary32(input_bits);
            let (qsl, losses, qsl_meter) =
                observed_float_source(&source, vec![Value::Float(input)], Meter::new(LIMITS));
            let mut runtime_meter = Meter::new(LIMITS);
            let runtime = evaluate_ieee(IeeeOperation::Negate(input), mode, &mut runtime_meter)
                .expect("IEEE negation admits a selected width");
            let (Outcome::Completed(Value::Float(qsl_value)), Outcome::Completed(native)) =
                (qsl, runtime)
            else {
                panic!("F32 negation must complete for {input_bits:#010x} in {mode:?}");
            };
            assert_eq!(qsl_value, native.value());
            assert_eq!(format!("{qsl_value:?}"), format!("{:?}", native.value()));
            assert_eq!(qsl_value.bits(), u64::from(expected_bits));
            assert!(native.flags().is_empty());
            assert!(losses.is_empty());
            assert_scalar_charge_suffix("F32", &qsl_meter, &runtime_meter);
        }
        let doubled = format!("function f using v(a: {ty}): {ty} pure {{ -(-a) }}");
        assert_ieee_carrier(&doubled, "quire.op.ieee.float32.negate", 2);
        for &(input_bits, _) in &binary32 {
            let input = IeeeValue::binary32(input_bits);
            let (qsl, losses, qsl_meter) =
                observed_float_source(&doubled, vec![Value::Float(input)], Meter::new(LIMITS));
            let mut native_meter = Meter::new(LIMITS);
            let Outcome::Completed(first) =
                evaluate_ieee(IeeeOperation::Negate(input), mode, &mut native_meter)
                    .expect("first negation admits")
            else {
                panic!("F32 first native negation must complete");
            };
            let Outcome::Completed(second) = evaluate_ieee(
                IeeeOperation::Negate(first.value()),
                mode,
                &mut native_meter,
            )
            .expect("second negation admits") else {
                panic!("F32 second native negation must complete");
            };
            let Outcome::Completed(Value::Float(value)) = qsl else {
                panic!("F32 double negation must complete for {input_bits:#010x}");
            };
            assert_eq!(value, second.value());
            assert_eq!(format!("{value:?}"), format!("{:?}", second.value()));
            assert_eq!(value.bits(), u64::from(input_bits));
            assert!(first.flags().is_empty() && second.flags().is_empty());
            assert!(losses.is_empty());
            assert_scalar_charge_suffix("F32 double negation", &qsl_meter, &native_meter);
        }
        let wide_ty = format!("Float64[{}]", mode.as_str());
        let wide_source = format!("function f using v(a: {wide_ty}): {wide_ty} pure {{ -a }}");
        assert_ieee_carrier(&wide_source, "quire.op.ieee.float64.negate", 1);
        let wide_input = IeeeValue::binary64(0x7ff0_0000_0000_0001);
        let (qsl, losses, qsl_meter) = observed_float_source(
            &wide_source,
            vec![Value::Float(wide_input)],
            Meter::new(LIMITS),
        );
        let mut runtime_meter = Meter::new(LIMITS);
        let runtime = evaluate_ieee(IeeeOperation::Negate(wide_input), mode, &mut runtime_meter)
            .expect("binary64 negation admits");
        let (Outcome::Completed(Value::Float(qsl_value)), Outcome::Completed(native)) =
            (qsl, runtime)
        else {
            panic!("F32 binary64 negation must complete");
        };
        assert_eq!(qsl_value, native.value());
        assert_eq!(format!("{qsl_value:?}"), format!("{:?}", native.value()));
        assert_eq!(qsl_value.bits(), 0xfff0_0000_0000_0001);
        assert!(native.flags().is_empty());
        assert!(losses.is_empty());
        assert_scalar_charge_suffix("F32 binary64", &qsl_meter, &runtime_meter);
        let wide_doubled = format!("function f using v(a: {wide_ty}): {wide_ty} pure {{ -(-a) }}");
        assert_ieee_carrier(&wide_doubled, "quire.op.ieee.float64.negate", 2);
        let (qsl, losses, qsl_meter) = observed_float_source(
            &wide_doubled,
            vec![Value::Float(wide_input)],
            Meter::new(LIMITS),
        );
        let mut native_meter = Meter::new(LIMITS);
        let Outcome::Completed(first) =
            evaluate_ieee(IeeeOperation::Negate(wide_input), mode, &mut native_meter)
                .expect("first binary64 negation admits")
        else {
            panic!("F32 first native binary64 negation must complete");
        };
        let Outcome::Completed(second) = evaluate_ieee(
            IeeeOperation::Negate(first.value()),
            mode,
            &mut native_meter,
        )
        .expect("second binary64 negation admits") else {
            panic!("F32 second native binary64 negation must complete");
        };
        let Outcome::Completed(Value::Float(value)) = qsl else {
            panic!("F32 binary64 double negation must complete");
        };
        assert_eq!(value, second.value());
        assert_eq!(format!("{value:?}"), format!("{:?}", second.value()));
        assert_eq!(value.bits(), wide_input.bits());
        assert!(first.flags().is_empty() && second.flags().is_empty());
        assert!(losses.is_empty());
        assert_scalar_charge_suffix("F32 binary64 double negation", &qsl_meter, &native_meter);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f33_negation_meter_agrees() {
    assert!(!selected_vector("F33").inputs.is_empty());
    let input = IeeeValue::binary32(0x3f80_0000);
    let function =
        "function f using v(a: Float32[nearest-even]): Float32[nearest-even] pure { -a }";
    assert_ieee_carrier(function, "quire.op.ieee.float32.negate", 1);
    for (id, limits, expected) in [
        (
            "F33 complete",
            ieee_limits(32, 1, 2, 1),
            ConversionExpectation::Completed(0xbf80_0000, IeeeFlags::EMPTY),
        ),
        (
            "F33 retain denial",
            ieee_limits(32, 1, 1, 1),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeResultRetain,
                1,
                1,
            ),
        ),
        (
            "F33 operand denial",
            ieee_limits(32, 0, 2, 1),
            ConversionExpectation::Incomplete(
                LimitKind::ValueOccurrences,
                ChargePoint::IeeeOperands,
                0,
                1,
            ),
        ),
    ] {
        let source_limits = ScalarLimits {
            work_units: limits.work_units + 1,
            ..limits
        };
        let (qsl, losses, qsl_meter) = observed_float_source(
            function,
            vec![Value::Float(input)],
            Meter::new(source_limits),
        );
        let mut runtime_meter = Meter::new(limits);
        let native = evaluate_ieee(
            IeeeOperation::Negate(input),
            RoundingMode::NearestEven,
            &mut runtime_meter,
        )
        .expect("negation admits");
        match (expected, qsl, native) {
            (
                ConversionExpectation::Completed(bits, flags),
                Outcome::Completed(Value::Float(qsl_value)),
                Outcome::Completed(native),
            ) => {
                assert_eq!(qsl_value, native.value(), "{id} two-sided value");
                assert_eq!(
                    format!("{qsl_value:?}"),
                    format!("{:?}", native.value()),
                    "{id} Debug rendering"
                );
                assert_eq!(qsl_value.bits(), bits, "{id} QSpec bits");
                assert_eq!(native.flags(), flags);
                assert!(losses.is_empty());
            }
            (
                ConversionExpectation::Incomplete(kind, point, consumed, next),
                Outcome::Incomplete(qsl_shortage),
                Outcome::Incomplete(native_shortage),
            ) => {
                assert_eq!(native_shortage.limit_kind, kind, "{id}");
                assert_eq!(native_shortage.charge_point, point, "{id}");
                assert_eq!(native_shortage.consumed, consumed, "{id}");
                assert_eq!(
                    native_shortage.limit,
                    asserted_limit(limits, kind),
                    "{id} runtime limit"
                );
                assert_eq!(native_shortage.next_charge, Integer::from(next), "{id}");
                assert_eq!(qsl_shortage.limit_kind, kind, "{id}");
                assert_eq!(qsl_shortage.charge_point, point, "{id}");
                assert_eq!(
                    qsl_shortage.consumed,
                    consumed + u64::from(kind == LimitKind::WorkUnits),
                    "{id}"
                );
                assert_eq!(
                    qsl_shortage.limit,
                    asserted_limit(source_limits, kind),
                    "{id} QSL limit"
                );
                assert_eq!(
                    qsl_shortage.next_charge,
                    Integer::from(next),
                    "{id} QSL next charge"
                );
                assert!(losses.is_empty());
            }
            (expected, qsl, native) => {
                panic!("{id} expected {expected:?}; QSL {qsl:?}; native {native:?}")
            }
        }
        assert_scalar_charge_suffix(id, &qsl_meter, &runtime_meter);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f24_f25_f26_f31_width_conversions_agree() {
    for id in ["F24", "F25", "F26", "F31"] {
        assert!(
            !selected_vector(id).inputs.is_empty(),
            "{id} remains published"
        );
    }
    let f32_one = IeeeValue::binary32(0x3f80_0000);
    for (id, limits, expected) in [
        (
            "F24 widen one",
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(0x3ff0_0000_0000_0000, IeeeFlags::EMPTY),
        ),
        (
            "F24 widen bit limit",
            ieee_limits(32, 1, 4, 1),
            ConversionExpectation::Incomplete(
                LimitKind::IntegerBits,
                ChargePoint::IeeeExactIntermediate,
                32,
                64,
            ),
        ),
    ] {
        assert_float_conversion(
            id,
            "Float32[nearest-even]",
            "Float64[nearest-even]",
            "quire.op.ieee.to_float64",
            Value::Float(f32_one),
            limits,
            expected,
            |meter| {
                convert_ieee_width(
                    f32_one,
                    IeeeWidth::Binary64,
                    RoundingMode::NearestEven,
                    meter,
                )
            },
        );
    }
    let f64_one = IeeeValue::binary64(0x3ff0_0000_0000_0000);
    assert_float_conversion(
        "F24 narrow input bit limit",
        "Float64[nearest-even]",
        "Float32[nearest-even]",
        "quire.op.ieee.to_float32",
        Value::Float(f64_one),
        ieee_limits(32, 1, 4, 1),
        ConversionExpectation::Incomplete(LimitKind::IntegerBits, ChargePoint::IeeeOperands, 0, 64),
        |meter| {
            convert_ieee_width(
                f64_one,
                IeeeWidth::Binary32,
                RoundingMode::NearestEven,
                meter,
            )
        },
    );
    let negative_zero = IeeeValue::binary32(0x8000_0000);
    for (id, limits, expected) in [
        (
            "F26 signed zero widen",
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(0x8000_0000_0000_0000, IeeeFlags::EMPTY),
        ),
        (
            "F26 signed zero retain limit",
            ieee_limits(64, 1, 3, 1),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeResultRetain,
                3,
                1,
            ),
        ),
    ] {
        assert_float_conversion(
            id,
            "Float32[nearest-even]",
            "Float64[nearest-even]",
            "quire.op.ieee.to_float64",
            Value::Float(negative_zero),
            limits,
            expected,
            |meter| {
                convert_ieee_width(
                    negative_zero,
                    IeeeWidth::Binary64,
                    RoundingMode::NearestEven,
                    meter,
                )
            },
        );
    }
    for (id, bits, mode, limits, expected) in [
        (
            "F25 signaling payload",
            0x7ff0_0000_0000_0001,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 2, 1),
            ConversionExpectation::Completed(
                0x7fc0_0001,
                [IeeeFlag::Invalid].into_iter().collect(),
            ),
        ),
        (
            "F25 negative quiet payload",
            0xfff8_0000_0000_0003,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 2, 1),
            ConversionExpectation::Completed(0xffc0_0003, IeeeFlags::EMPTY),
        ),
        (
            "F25 oversized payload",
            0x7ff8_0000_0040_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 1, 0),
            ConversionExpectation::Refused(
                "ieee_nan_payload_not_representable",
                "payload-exceeds-target",
            ),
        ),
        (
            "F25 signaling oversized payload",
            0x7ff0_0000_0040_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 1, 0),
            ConversionExpectation::Refused(
                "ieee_nan_payload_not_representable",
                "payload-exceeds-target",
            ),
        ),
        (
            "F25 operand denial",
            0x7ff8_0000_0040_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 0, 0),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeOperands,
                0,
                1,
            ),
        ),
        (
            "F31 negative infinity",
            0xfff0_0000_0000_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 2, 1),
            ConversionExpectation::Completed(0xff80_0000, IeeeFlags::EMPTY),
        ),
        (
            "F31 negative infinity retention",
            0xfff0_0000_0000_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 1, 1),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeResultRetain,
                1,
                1,
            ),
        ),
        (
            "F31 nearest tenth",
            0x3fb9_9999_9999_999a,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(
                0x3dcc_cccd,
                [IeeeFlag::Inexact].into_iter().collect(),
            ),
        ),
        (
            "F31 exact tenth",
            0x3fb9_9999_9999_999a,
            RoundingMode::Exact,
            ieee_limits(64, 1, 3, 1),
            ConversionExpectation::Refused("ieee_not_exact", "rounding-required"),
        ),
        (
            "F31 exact tenth round denial",
            0x3fb9_9999_9999_999a,
            RoundingMode::Exact,
            ieee_limits(64, 1, 2, 1),
            ConversionExpectation::Incomplete(LimitKind::WorkUnits, ChargePoint::IeeeRound, 2, 1),
        ),
        (
            "F31 exact maximum",
            0x47ef_ffff_e000_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(0x7f7f_ffff, IeeeFlags::EMPTY),
        ),
        (
            "F31 overflow tie",
            0x47ef_ffff_f000_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(
                0x7f80_0000,
                [IeeeFlag::Overflow, IeeeFlag::Inexact]
                    .into_iter()
                    .collect(),
            ),
        ),
        (
            "F31 overflow retention denial",
            0x47ef_ffff_f000_0000,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 3, 1),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeResultRetain,
                3,
                1,
            ),
        ),
        (
            "F31 toward-zero maximum",
            0x47ef_ffff_f000_0000,
            RoundingMode::TowardZero,
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(
                0x7f7f_ffff,
                [IeeeFlag::Inexact].into_iter().collect(),
            ),
        ),
        (
            "F31 underflow",
            0x0000_0000_0000_0001,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 4, 1),
            ConversionExpectation::Completed(
                0,
                [IeeeFlag::Underflow, IeeeFlag::Inexact]
                    .into_iter()
                    .collect(),
            ),
        ),
        (
            "F31 underflow retention denial",
            0x0000_0000_0000_0001,
            RoundingMode::NearestEven,
            ieee_limits(64, 1, 3, 1),
            ConversionExpectation::Incomplete(
                LimitKind::WorkUnits,
                ChargePoint::IeeeResultRetain,
                3,
                1,
            ),
        ),
    ] {
        let input = IeeeValue::binary64(bits);
        let target = format!("Float32[{}]", mode.as_str());
        assert_float_conversion(
            id,
            "Float64[nearest-even]",
            &target,
            "quire.op.ieee.to_float32",
            Value::Float(input),
            limits,
            expected,
            |meter| convert_ieee_width(input, IeeeWidth::Binary32, mode, meter),
        );
    }
    let nan32 = IeeeValue::binary32(0x7fc0_0001);
    assert_float_conversion(
        "F25 quiet payload widening",
        "Float32[nearest-even]",
        "Float64[nearest-even]",
        "quire.op.ieee.to_float64",
        Value::Float(nan32),
        ieee_limits(32, 1, 2, 1),
        ConversionExpectation::Completed(0x7ff8_0000_0000_0001, IeeeFlags::EMPTY),
        |meter| convert_ieee_width(nan32, IeeeWidth::Binary64, RoundingMode::NearestEven, meter),
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f24_f27_f28_exact_conversions_agree() {
    for id in ["F24", "F27", "F28"] {
        assert!(
            !selected_vector(id).inputs.is_empty(),
            "{id} remains published"
        );
    }
    let third = Rational::new(Integer::from(1_i64), Integer::from(3_i64)).expect("one third");
    for (id, limits, expected) in [
        (
            "F24 one third",
            ieee_limits(32, 1, 4, 1),
            ConversionExpectation::Completed(
                0x3eaa_aaab,
                [IeeeFlag::Inexact].into_iter().collect(),
            ),
        ),
        (
            "F24 exact intermediate denial",
            ieee_limits(2, 1, 4, 1),
            ConversionExpectation::Incomplete(
                LimitKind::IntegerBits,
                ChargePoint::IeeeExactIntermediate,
                2,
                32,
            ),
        ),
    ] {
        assert_float_conversion(
            id,
            "Rational[0, 1; 1, 3]",
            "Float32[nearest-even]",
            "quire.op.ieee.from_exact",
            Value::Rational(third.clone()),
            limits,
            expected,
            |meter| {
                exact_to_ieee(
                    &third,
                    IeeeWidth::Binary32,
                    RoundingMode::NearestEven,
                    meter,
                )
            },
        );
    }
    let decimal = Decimal::new(Integer::from(100_i64), 2);
    for (id, limits, expected) in [
        (
            "F28 decimal one",
            ieee_limits(32, 1, 4, 1),
            ConversionExpectation::Completed(0x3f80_0000, IeeeFlags::EMPTY),
        ),
        (
            "F28 decimal retained bits",
            ieee_limits(6, 1, 4, 1),
            ConversionExpectation::Incomplete(
                LimitKind::IntegerBits,
                ChargePoint::IeeeOperands,
                0,
                7,
            ),
        ),
    ] {
        assert_float_conversion(
            id,
            "Decimal[0, 100; 0, 2; nearest-even]",
            "Float32[nearest-even]",
            "quire.op.ieee.from_exact",
            Value::Decimal(decimal.clone()),
            limits,
            expected,
            |meter| {
                exact_to_ieee(
                    &decimal,
                    IeeeWidth::Binary32,
                    RoundingMode::NearestEven,
                    meter,
                )
            },
        );
    }
    let small = rational_domain((0_i64.into(), 1_i64.into()), (1_i64.into(), 2_i64.into()));
    let half = Rational::new(1_i64.into(), 2_i64.into()).expect("half");
    assert_rational_conversion(
        "F24 IEEE half",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 2]",
        IeeeValue::binary32(0x3f00_0000),
        &small,
        ieee_limits(32, 1, 3, 1),
        Some(Outcome::Completed(Value::Rational(half.clone()))),
    );
    assert_rational_conversion(
        "F24 NaN undefined",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 2]",
        IeeeValue::binary32(0x7fc0_0000),
        &small,
        ieee_limits(32, 1, 2, 1),
        Some(Outcome::Undefined(Undefined::IeeeNotFinite)),
    );
    let tiny_denominator = Integer::from(2_i64).pow(&Integer::from(1074_i64));
    let large = rational_domain(
        (0_i64.into(), 1_i64.into()),
        (1_i64.into(), tiny_denominator.clone()),
    );
    let large_ty = format!("Rational[0, 1; 1, {tiny_denominator}]");
    let tiny =
        Rational::new(1_i64.into(), tiny_denominator).expect("smallest subnormal exact value");
    assert_rational_conversion(
        "F27 tiny exact",
        "Float64[nearest-even]",
        &large_ty,
        IeeeValue::binary64(1),
        &large,
        ieee_limits(1075, 1, 3, 1),
        Some(Outcome::Completed(Value::Rational(tiny))),
    );
    assert_rational_conversion(
        "F27 tiny bit denial",
        "Float64[nearest-even]",
        &large_ty,
        IeeeValue::binary64(1),
        &large,
        ieee_limits(1074, 1, 3, 1),
        None,
    );
    let zero = rational_domain((0_i64.into(), 0_i64.into()), (1_i64.into(), 1_i64.into()));
    assert_rational_conversion(
        "F27 negative zero",
        "Float32[nearest-even]",
        "Rational[0, 0; 1, 1]",
        IeeeValue::binary32(0x8000_0000),
        &zero,
        ieee_limits(32, 1, 3, 1),
        Some(Outcome::Completed(Value::Rational(Rational::from_integer(
            0_i64.into(),
        )))),
    );
    assert_rational_conversion(
        "F27 retain denial",
        "Float32[nearest-even]",
        "Rational[0, 0; 1, 1]",
        IeeeValue::binary32(0x8000_0000),
        &zero,
        ieee_limits(32, 1, 2, 1),
        None,
    );
    assert_rational_conversion(
        "F27 infinity undefined",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 2]",
        IeeeValue::binary32(0x7f80_0000),
        &small,
        ieee_limits(32, 1, 1, 0),
        Some(Outcome::Undefined(Undefined::IeeeNotFinite)),
    );
    assert_rational_conversion(
        "F27 infinity operand denial",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 2]",
        IeeeValue::binary32(0x7f80_0000),
        &small,
        ieee_limits(32, 1, 0, 0),
        None,
    );
    let integral = rational_domain((0_i64.into(), 1_i64.into()), (1_i64.into(), 1_i64.into()));
    assert_rational_conversion(
        "F27 outside domain",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 1]",
        IeeeValue::binary32(0x3f00_0000),
        &integral,
        ieee_limits(32, 1, 2, 0),
        Some(Outcome::Refused(Refusal::IeeeRationalOutOfDomain {
            target: Box::new(integral.clone()),
        })),
    );
    assert_rational_conversion(
        "F27 exact intermediate denial",
        "Float32[nearest-even]",
        "Rational[0, 1; 1, 1]",
        IeeeValue::binary32(0x3f00_0000),
        &integral,
        ieee_limits(32, 1, 1, 0),
        None,
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f10_binary64_rounding_agrees() {
    let f10 = selected_vector("F10");
    let f10_bits = hex_bits64(f10.inputs);
    assert_eq!(
        hex_bits64(f10.expected),
        [0x3ff0_0000_0000_0000, 0x3ff0_0000_0000_0001]
    );
    let [one, half_ulp] = f10_bits.as_slice() else {
        panic!("F10 supplies two binary64 operands");
    };
    let inexact: IeeeFlags = [IeeeFlag::Inexact].into_iter().collect();
    for (mode, expected_bits) in [
        (RoundingMode::NearestEven, 0x3ff0_0000_0000_0000),
        (RoundingMode::NearestAway, 0x3ff0_0000_0000_0001),
        (RoundingMode::TowardZero, 0x3ff0_0000_0000_0000),
        (RoundingMode::TowardPositive, 0x3ff0_0000_0000_0001),
        (RoundingMode::TowardNegative, 0x3ff0_0000_0000_0000),
    ] {
        assert_add_observation(
            "F10",
            IeeeValue::binary64(*one),
            IeeeValue::binary64(*half_ulp),
            mode,
            Ok((expected_bits, inexact)),
        );
    }
    let limits = ieee_limits(64, 2, 4, 1);
    assert_bounded_binary(
        "F10 exact tuple",
        BinaryOperation::Add,
        IeeeValue::binary64(*one),
        IeeeValue::binary64(*half_ulp),
        RoundingMode::NearestEven,
        limits,
        None,
        BoundedExpectation::Completed(0x3ff0_0000_0000_0000, inexact),
    );
    assert_bounded_binary(
        "F10 denied result retention",
        BinaryOperation::Add,
        IeeeValue::binary64(*one),
        IeeeValue::binary64(*half_ulp),
        RoundingMode::NearestEven,
        limits,
        Some(ChargePoint::IeeeResultRetain),
        BoundedExpectation::Incomplete(ChargePoint::IeeeResultRetain),
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f18_f21_f22_metered_arithmetic_agrees() {
    let f18 = selected_vector("F18");
    let f18_bits = hex_bits(f18.inputs);
    assert_eq!(hex_bits(f18.expected), [0x7fc0_0000]);
    assert_eq!(f18_bits.len(), 2, "F18 has two binary32 operands");
    assert_bounded_binary(
        "F18",
        BinaryOperation::Multiply,
        IeeeValue::binary32(f18_bits[0]),
        IeeeValue::binary32(f18_bits[1]),
        RoundingMode::NearestEven,
        ieee_limits(32, 2, 2, 1),
        None,
        BoundedExpectation::Completed(0x7fc0_0000, [IeeeFlag::Invalid].into_iter().collect()),
    );
    assert_bounded_binary(
        "F18 denied result retention",
        BinaryOperation::Multiply,
        IeeeValue::binary32(f18_bits[0]),
        IeeeValue::binary32(f18_bits[1]),
        RoundingMode::NearestEven,
        ieee_limits(32, 2, 1, 1),
        None,
        BoundedExpectation::Incomplete(ChargePoint::IeeeResultRetain),
    );

    let f21 = selected_vector("F21");
    let f21_bits = hex_bits(f21.inputs);
    assert_eq!(hex_bits(f21.expected), [0x0000_0000, 0x8000_0000]);
    assert_eq!(f21_bits.len(), 4, "F21 has two binary operations");
    for (operation, left, right, expected_bits) in [
        (BinaryOperation::Add, f21_bits[0], f21_bits[1], 0x0000_0000),
        (
            BinaryOperation::Multiply,
            f21_bits[2],
            f21_bits[3],
            0x8000_0000,
        ),
    ] {
        assert_bounded_binary(
            "F21 admitted",
            operation,
            IeeeValue::binary32(left),
            IeeeValue::binary32(right),
            RoundingMode::NearestEven,
            ieee_limits(32, 2, 4, 1),
            None,
            BoundedExpectation::Completed(expected_bits, IeeeFlags::EMPTY),
        );
        assert_bounded_binary(
            "F21 denied result retention",
            operation,
            IeeeValue::binary32(left),
            IeeeValue::binary32(right),
            RoundingMode::NearestEven,
            ieee_limits(32, 2, 3, 1),
            None,
            BoundedExpectation::Incomplete(ChargePoint::IeeeResultRetain),
        );
    }

    let f22 = selected_vector("F22");
    // Branch-gating QSpec cross-row reference: F22 must remain the F09 first sum.
    assert!(f22.inputs.contains("F09"));
    let f07_bits = hex_bits(selected_vector("F07").inputs);
    assert_eq!(f07_bits.len(), 2);
    let inexact: IeeeFlags = [IeeeFlag::Inexact].into_iter().collect();
    for (mode, work, expected) in [
        (RoundingMode::Exact, 3, BoundedExpectation::Refused(inexact)),
        (
            RoundingMode::NearestEven,
            3,
            BoundedExpectation::Incomplete(ChargePoint::IeeeResultRetain),
        ),
        (
            RoundingMode::Exact,
            2,
            BoundedExpectation::Incomplete(ChargePoint::IeeeRound),
        ),
    ] {
        assert_bounded_binary(
            "F22",
            BinaryOperation::Add,
            IeeeValue::binary32(f07_bits[0]),
            IeeeValue::binary32(f07_bits[1]),
            mode,
            ieee_limits(32, 2, work, 1),
            None,
            expected,
        );
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f19_f30_square_root_named_charges_agree() {
    let f19 = selected_vector("F19");
    let f19_bits = hex_bits(f19.inputs);
    assert_eq!(hex_bits(f19.expected), [0x3fb5_04f3]);
    assert_eq!(f19_bits.len(), 1, "F19 has one binary32 operand");
    let inexact: IeeeFlags = [IeeeFlag::Inexact].into_iter().collect();
    let f19_limits = ieee_limits(32, 1, 4, 1);
    assert_bounded_sqrt(
        "F19",
        f19_bits[0],
        f19_limits,
        None,
        Ok((0x3fb5_04f3, inexact)),
    );
    for point in [
        ChargePoint::IeeeExactIntermediate,
        ChargePoint::IeeeRound,
        ChargePoint::IeeeResultRetain,
    ] {
        assert_bounded_sqrt(
            "F19 denial",
            f19_bits[0],
            f19_limits,
            Some(point),
            Err(point),
        );
    }

    let f30 = selected_vector("F30");
    let f30_bits = hex_bits(f30.inputs);
    assert_eq!(hex_bits(f30.expected), [0x8000_0000]);
    assert_eq!(f30_bits.len(), 1, "F30 has one signed-zero operand");
    assert_bounded_sqrt(
        "F30 admitted",
        f30_bits[0],
        ieee_limits(32, 1, 4, 1),
        None,
        Ok((0x8000_0000, IeeeFlags::EMPTY)),
    );
    assert_bounded_sqrt(
        "F30 denied result retention",
        f30_bits[0],
        ieee_limits(32, 1, 3, 1),
        None,
        Err(ChargePoint::IeeeResultRetain),
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f12_invalid_arithmetic_agrees_in_every_mode() {
    let row = selected_vector("F12");
    let inputs = hex_bits(row.inputs);
    assert_eq!(inputs, [0x0000_0000, 0x7f80_0000, 0xbf80_0000]);
    assert_eq!(hex_bits(row.expected), [0x7fc0_0000]);
    let invalid: IeeeFlags = [IeeeFlag::Invalid].into_iter().collect();
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::TowardPositive,
        RoundingMode::TowardNegative,
        RoundingMode::Exact,
    ] {
        assert_binary_observation(
            "F12 multiply",
            BinaryOperation::Multiply,
            IeeeValue::binary32(inputs[0]),
            IeeeValue::binary32(inputs[1]),
            mode,
            Ok((0x7fc0_0000, invalid)),
        );
        assert_sqrt_observation("F12 square root", inputs[2], mode, 0x7fc0_0000, invalid);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f13_division_flags_are_operation_local() {
    let row = selected_vector("F13");
    // Branch-gating QSpec expectation: the exceptional flag remains authored.
    assert!(
        row.expected.contains("divide_by_zero"),
        "F13 flag expectation"
    );
    let divide_by_zero: IeeeFlags = [IeeeFlag::DivideByZero].into_iter().collect();
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::TowardPositive,
        RoundingMode::TowardNegative,
        RoundingMode::Exact,
    ] {
        for (left, right, bits) in [
            (0x3f80_0000, 0x0000_0000, 0x7f80_0000),
            (0xbf80_0000, 0x0000_0000, 0xff80_0000),
            (0x3f80_0000, 0x8000_0000, 0xff80_0000),
            (0xbf80_0000, 0x8000_0000, 0x7f80_0000),
        ] {
            assert_binary_observation(
                "F13",
                BinaryOperation::Divide,
                IeeeValue::binary32(left),
                IeeeValue::binary32(right),
                mode,
                Ok((bits, divide_by_zero)),
            );
        }
    }
    let f09 = selected_vector("F09");
    assert!(hex_bits(f09.expected).contains(&0x3f80_0001));
    assert_add_observation(
        "F13 following exact no-flag operation",
        IeeeValue::binary32(0x3f80_0000),
        IeeeValue::binary32(0x3400_0000),
        RoundingMode::Exact,
        Ok((0x3f80_0001, IeeeFlags::EMPTY)),
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f15_f16_subnormal_multiplication_agrees() {
    let f15 = selected_vector("F15");
    let f15_bits = hex_bits(f15.inputs);
    assert_eq!(f15_bits, [0x0080_0000, 0x3f00_0000]);
    assert_eq!(hex_bits(f15.expected), [0x0040_0000]);
    assert_binary_observation(
        "F15",
        BinaryOperation::Multiply,
        IeeeValue::binary32(f15_bits[0]),
        IeeeValue::binary32(f15_bits[1]),
        RoundingMode::NearestEven,
        Ok((0x0040_0000, IeeeFlags::EMPTY)),
    );

    let f16 = selected_vector("F16");
    let f16_bits = hex_bits(f16.inputs);
    assert_eq!(f16_bits, [0x0000_0001, 0x3f00_0000]);
    assert_eq!(hex_bits(f16.expected), [0x0000_0000]);
    assert_binary_observation(
        "F16",
        BinaryOperation::Multiply,
        IeeeValue::binary32(f16_bits[0]),
        IeeeValue::binary32(f16_bits[1]),
        RoundingMode::NearestEven,
        Ok((
            0x0000_0000,
            [IeeeFlag::Underflow, IeeeFlag::Inexact]
                .into_iter()
                .collect(),
        )),
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f04b_f11_fma_agrees_with_runtime_and_separate_rounding() {
    let f04b = selected_vector("F04b");
    let f04b_inputs = hex_bits(f04b.inputs);
    let [_, _, fma_a, fma_b, fma_c] = f04b_inputs.as_slice() else {
        panic!("F04b supplies ADD then three FMA operands");
    };
    assert_eq!(hex_bits(f04b.expected), [0x7fc0_0012, 0x7fc0_0012]);
    assert_fma_agreement(
        "F04b FMA",
        [*fma_a, *fma_b, *fma_c],
        0x7fc0_0012,
        [IeeeFlag::Invalid].into_iter().collect(),
    );

    let f11 = selected_vector("F11");
    let inputs = hex_bits(f11.inputs);
    let [a, b, c] = inputs.as_slice() else {
        panic!("F11 supplies three FMA operands");
    };
    assert_eq!(hex_bits(f11.expected), [0xa880_0000, 0x0000_0000]);
    assert_fma_agreement("F11 fused", [*a, *b, *c], 0xa880_0000, IeeeFlags::EMPTY);

    let function = "function f using v(a: Float32[nearest-even], b: Float32[nearest-even], \
                    c: Float32[nearest-even]): Float32[nearest-even] pure { a * b + c }";
    let a = IeeeValue::binary32(*a);
    let b = IeeeValue::binary32(*b);
    let c = IeeeValue::binary32(*c);
    let (qsl, losses, qsl_meter) = observed_float_source(
        function,
        vec![Value::Float(a), Value::Float(b), Value::Float(c)],
        Meter::new(LIMITS),
    );
    let mut runtime_meter = Meter::new(LIMITS);
    let product = evaluate_ieee(
        IeeeOperation::Multiply(a, b),
        RoundingMode::NearestEven,
        &mut runtime_meter,
    )
    .expect("binary32 product type-checks");
    let Outcome::Completed(product) = product else {
        panic!("F11 separate product must complete");
    };
    let sum = evaluate_ieee(
        IeeeOperation::Add(product.value(), c),
        RoundingMode::NearestEven,
        &mut runtime_meter,
    )
    .expect("binary32 sum type-checks");
    let (Outcome::Completed(Value::Float(qsl_value)), Outcome::Completed(sum)) = (qsl, sum) else {
        panic!("F11 separate path must complete on both sides");
    };
    assert_eq!(qsl_value, sum.value(), "F11 separate two-sided bits");
    assert_eq!(qsl_value.bits(), 0, "F11 separate QSpec bits");
    let runtime_losses: Vec<_> = [product.flags(), sum.flags()]
        .into_iter()
        .filter(|flags| !flags.is_empty())
        .map(ValueLoss::IeeeFlags)
        .collect();
    assert_eq!(losses, runtime_losses, "F11 separate flags");
    assert_scalar_charge_suffix("F11 separate", &qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f05_infinities_and_finite_extrema_agree() {
    let row = selected_vector("F05");
    // Branch-gating QSpec prose: the selected comparison class must stay authored.
    assert!(row.inputs.contains("infinity"));
    assert!(row.inputs.contains("finite extrema"));
    for (low, high) in [
        (0xff80_0000, 0xff7f_ffff),
        (0xff7f_ffff, 0x7f7f_ffff),
        (0x7f7f_ffff, 0x7f80_0000),
    ] {
        let low = IeeeValue::binary32(low);
        let high = IeeeValue::binary32(high);
        assert_float_relation("F05 less", low, high, FloatRelation::Less, true);
        assert_float_relation(
            "F05 less-or-equal",
            low,
            high,
            FloatRelation::LessOrEqual,
            true,
        );
        assert_float_relation("F05 reverse less", high, low, FloatRelation::Less, false);
        assert_float_relation(
            "F05 reverse less-or-equal",
            high,
            low,
            FloatRelation::LessOrEqual,
            false,
        );
        assert_intrinsic_comparison(
            "F05 total order",
            low,
            high,
            IeeeComparison::TotalOrder,
            true,
        );
        assert_intrinsic_comparison(
            "F05 numeric inequality",
            low,
            high,
            IeeeComparison::NumericEqual,
            false,
        );
    }
    let positive_infinity = IeeeValue::binary32(0x7f80_0000);
    let negative_infinity = IeeeValue::binary32(0xff80_0000);
    assert_intrinsic_comparison(
        "F05 infinity self identity",
        positive_infinity,
        positive_infinity,
        IeeeComparison::BitIdentical,
        true,
    );
    assert_intrinsic_comparison(
        "F05 infinity sign identity",
        positive_infinity,
        negative_infinity,
        IeeeComparison::BitIdentical,
        false,
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02")]
#[test]
fn tc193_f34_float_relations_agree_at_both_widths() {
    let row = selected_vector("F34");
    let binary64 = hex_bits64(row.inputs);
    assert_eq!(binary64.len(), 6, "F34 names six binary64 roles");
    for (p0, n0, one, pinf, qnan, snan) in [
        (
            IeeeValue::binary64(binary64[0]),
            IeeeValue::binary64(binary64[1]),
            IeeeValue::binary64(binary64[2]),
            IeeeValue::binary64(binary64[3]),
            IeeeValue::binary64(binary64[4]),
            IeeeValue::binary64(binary64[5]),
        ),
        (
            IeeeValue::binary32(0x0000_0000),
            IeeeValue::binary32(0x8000_0000),
            IeeeValue::binary32(0x3f80_0000),
            IeeeValue::binary32(0x7f80_0000),
            IeeeValue::binary32(0x7fc0_0000),
            IeeeValue::binary32(0x7f80_0001),
        ),
    ] {
        for (left, relation, right, expected) in [
            (n0, FloatRelation::Equal, p0, true),
            (n0, FloatRelation::Less, p0, false),
            (n0, FloatRelation::LessOrEqual, p0, true),
            (p0, FloatRelation::GreaterOrEqual, n0, true),
            (p0, FloatRelation::Greater, n0, false),
            (one, FloatRelation::Less, pinf, true),
            (qnan, FloatRelation::Equal, qnan, false),
            (qnan, FloatRelation::NotEqual, qnan, true),
            (qnan, FloatRelation::NotEqual, one, true),
            (qnan, FloatRelation::Less, one, false),
            (qnan, FloatRelation::LessOrEqual, one, false),
            (one, FloatRelation::Greater, qnan, false),
            (one, FloatRelation::GreaterOrEqual, qnan, false),
            (snan, FloatRelation::Less, one, false),
            (snan, FloatRelation::Equal, snan, false),
            (pinf, FloatRelation::Equal, pinf, true),
            (pinf, FloatRelation::LessOrEqual, pinf, true),
        ] {
            assert_float_relation("F34", left, right, relation, expected);
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc193_f01_f02_f02b_order_and_identity_agree() {
    let f01 = selected_vector("F01");
    let f01_bits = hex_bits(f01.inputs);
    let [positive_zero, negative_zero] = f01_bits.as_slice() else {
        panic!("F01 supplies two binary32 zeros");
    };
    let positive_zero = IeeeValue::binary32(*positive_zero);
    let negative_zero = IeeeValue::binary32(*negative_zero);
    assert_intrinsic_comparison(
        "F01 signed-zero identity",
        negative_zero,
        positive_zero,
        IeeeComparison::BitIdentical,
        false,
    );
    assert_intrinsic_comparison(
        "F01 signed-zero order",
        negative_zero,
        positive_zero,
        IeeeComparison::TotalOrder,
        true,
    );
    assert_intrinsic_comparison(
        "F01 reverse signed-zero order",
        positive_zero,
        negative_zero,
        IeeeComparison::TotalOrder,
        false,
    );

    let f02 = selected_vector("F02");
    let f02_bits = hex_bits(f02.inputs);
    let [first_nan, second_nan] = f02_bits.as_slice() else {
        panic!("F02 supplies two qNaNs");
    };
    let first_nan = IeeeValue::binary32(*first_nan);
    let second_nan = IeeeValue::binary32(*second_nan);
    assert_intrinsic_comparison(
        "F02 self bit identity",
        first_nan,
        first_nan,
        IeeeComparison::BitIdentical,
        true,
    );
    assert_intrinsic_comparison(
        "F02 second self bit identity",
        second_nan,
        second_nan,
        IeeeComparison::BitIdentical,
        true,
    );
    assert_intrinsic_comparison(
        "F02 payload order",
        first_nan,
        second_nan,
        IeeeComparison::TotalOrder,
        true,
    );
    assert_intrinsic_comparison(
        "F02 reverse payload order",
        second_nan,
        first_nan,
        IeeeComparison::TotalOrder,
        false,
    );

    let f02b = selected_vector("F02b");
    let f02b_bits = hex_bits(f02b.inputs);
    let [negative_two, negative_one, signaling, quiet] = f02b_bits.as_slice() else {
        panic!("F02b supplies two negative qNaNs and a signaling/quiet pair");
    };
    for (low, high) in [(*negative_two, *negative_one), (*signaling, *quiet)] {
        assert_intrinsic_comparison(
            "F02b authored NaN order",
            IeeeValue::binary32(low),
            IeeeValue::binary32(high),
            IeeeComparison::TotalOrder,
            true,
        );
        assert_intrinsic_comparison(
            "F02b reverse NaN order",
            IeeeValue::binary32(high),
            IeeeValue::binary32(low),
            IeeeComparison::TotalOrder,
            false,
        );
    }
}

fn run_f01_f02_available() {
    tc193_f01_f02_numeric_equal_charges_agree();
    tc193_f01_f02_f02b_order_and_identity_agree();
}

fn run_f04b_add_and_fma() {
    tc193_selected_add_rows_agree_from_checked_source_to_runtime();
    tc193_f04b_f11_fma_agrees_with_runtime_and_separate_rounding();
}

fn run_f24_conversion_agreement() {
    tc193_f24_f25_f26_f31_width_conversions_agree();
    tc193_f24_f27_f28_exact_conversions_agree();
}

fn run_f26_signed_zero_agreement() {
    tc193_f26_signed_zero_arithmetic_agrees_in_every_mode();
    tc193_f24_f25_f26_f31_width_conversions_agree();
}

fn f_case(id: &str) -> fn() {
    // Branch-gating QSpec F-ID dispatch: an unknown or newly authored ID fails closed.
    match id {
        "F01" | "F02" => run_f01_f02_available,
        "F02b" => tc193_f01_f02_f02b_order_and_identity_agree,
        "F03" | "F04" => tc193_selected_add_rows_agree_from_checked_source_to_runtime,
        "F04b" => run_f04b_add_and_fma,
        "F05" => tc193_f05_infinities_and_finite_extrema_agree,
        "F06" => tc193_f06_cross_width_comparison_refuses_before_evaluation,
        "F07" | "F08" => tc193_f07_f08_rounding_modes_agree_from_checked_source_to_runtime,
        "F09" | "F14" | "F23" | "F29" => tc193_f09_f14_f23_f29_exact_refusal_and_rounding_agree,
        "F10" => tc193_f10_binary64_rounding_agrees,
        "F11" => tc193_f04b_f11_fma_agrees_with_runtime_and_separate_rounding,
        "F12" => tc193_f12_invalid_arithmetic_agrees_in_every_mode,
        "F13" => tc193_f13_division_flags_are_operation_local,
        "F15" | "F16" => tc193_f15_f16_subnormal_multiplication_agrees,
        "F17" => tc193_f17_numeric_equality_accounts_for_named_charges,
        "F18" | "F21" | "F22" => tc193_f18_f21_f22_metered_arithmetic_agrees,
        "F19" | "F30" => tc193_f19_f30_square_root_named_charges_agree,
        "F20" => tc193_f20_admission_refusals_have_no_charge,
        "F24" => run_f24_conversion_agreement,
        "F25" | "F31" => tc193_f24_f25_f26_f31_width_conversions_agree,
        "F26" => run_f26_signed_zero_agreement,
        "F27" | "F28" => tc193_f24_f27_f28_exact_conversions_agree,
        "F32" => tc193_f32_ieee_negation_preserves_bits_and_rounding_independence,
        "F33" => tc193_f33_negation_meter_agrees,
        "F34" => tc193_f34_float_relations_agree_at_both_widths,
        other => panic!("unrouted QSpec TC-193 F vector {other}"),
    }
}

#[trace("TC-097", "IT-017-SC-01")]
#[test]
fn tc193_every_published_f_row_has_a_runnable_or_ticket_backed_route() {
    for row in vectors_of(TestCase::Tc193) {
        f_case(row.id)();
    }
}
