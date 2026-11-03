// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-193 binary32 addition through checked QSL source and the exact runtime.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use qsl_semantics::value::{admit_ieee_profile, DefinitionRole};
use quire_eval::FamilyOutcome;
use quire_exact::{
    compare_ieee, evaluate_ieee, ChargePoint, IeeeComparison, IeeeFlag, IeeeFlags, IeeeOperation,
    IeeeValue, IeeeWidth, IllTypedCause, InjectedDenial, LimitKind, Meter, Outcome, Refusal,
    RoundingMode, ScalarLimits, Value,
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
}

impl BinaryOperation {
    fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
        }
    }

    fn runtime(self, left: IeeeValue, right: IeeeValue) -> IeeeOperation {
        match self {
            Self::Add => IeeeOperation::Add(left, right),
            Self::Subtract => IeeeOperation::Subtract(left, right),
            Self::Multiply => IeeeOperation::Multiply(left, right),
        }
    }
}

fn observed_binary(
    operation: BinaryOperation,
    left: IeeeValue,
    right: IeeeValue,
    mode: RoundingMode,
    limits: ScalarLimits,
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
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
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
        Vec::new(),
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
    let mut meter = Meter::new(limits);
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
    observed_binary(BinaryOperation::Add, left, right, mode, limits)
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
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
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
        Vec::new(),
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
    let (qsl, losses, qsl_meter) = observed_binary(operation, left, right, mode, LIMITS);
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
    left: u32,
    right: u32,
    mode: RoundingMode,
    limits: ScalarLimits,
    expected: BoundedExpectation,
) {
    let left = IeeeValue::binary32(left);
    let right = IeeeValue::binary32(right);
    let source_limits = ScalarLimits {
        work_units: limits.work_units + 1,
        ..limits
    };
    let (qsl, losses, qsl_meter) = observed_binary(operation, left, right, mode, source_limits);
    let mut runtime_meter = Meter::new(limits);
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
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
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
        Vec::new(),
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
fn tc193_f01_f02_numeric_equal_portions_agree() {
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
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
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
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles the mixed-width source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let refusal = declarations
        .check(CheckingLimits::default())
        .expect_err("S4 refuses mixed IEEE widths");
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

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc193_f20_cross_width_addition_refuses_without_charge() {
    let row = selected_vector("F20");
    let operands = hex_bits64(row.inputs);
    assert!(operands.len() >= 2, "F20 names both IEEE widths");
    let left = u32::try_from(operands[0]).expect("F20 first operand is binary32");
    let right = operands[1];
    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function mixed using v(a: Float32[nearest-even], b: Float64[nearest-even]): \
         Float32[nearest-even] pure { a + b }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "ieee-mixed-add.native",
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
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles the mixed-width source");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    let refusal = declarations
        .check(CheckingLimits::default())
        .expect_err("S4 refuses mixed IEEE widths before evaluation");
    assert_eq!(
        refusal.cause,
        CheckCause::IllTyped(IllTypedCause::TypeMismatch)
    );
    let mut meter = Meter::new(ieee_limits(0, 0, 0, 0));
    let runtime = evaluate_ieee(
        IeeeOperation::Add(IeeeValue::binary32(left), IeeeValue::binary64(right)),
        RoundingMode::NearestEven,
        &mut meter,
    )
    .expect_err("runtime also refuses mixed IEEE widths before evaluation");
    assert_eq!(runtime.cause, IllTypedCause::DistinctIeeeWidths);
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
        f18_bits[0],
        f18_bits[1],
        RoundingMode::NearestEven,
        ieee_limits(32, 2, 2, 1),
        BoundedExpectation::Completed(0x7fc0_0000, [IeeeFlag::Invalid].into_iter().collect()),
    );
    assert_bounded_binary(
        "F18 denied result retention",
        BinaryOperation::Multiply,
        f18_bits[0],
        f18_bits[1],
        RoundingMode::NearestEven,
        ieee_limits(32, 2, 1, 1),
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
            left,
            right,
            RoundingMode::NearestEven,
            ieee_limits(32, 2, 4, 1),
            BoundedExpectation::Completed(expected_bits, IeeeFlags::EMPTY),
        );
        assert_bounded_binary(
            "F21 denied result retention",
            operation,
            left,
            right,
            RoundingMode::NearestEven,
            ieee_limits(32, 2, 3, 1),
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
            f07_bits[0],
            f07_bits[1],
            mode,
            ieee_limits(32, 2, work, 1),
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
