// SPDX-License-Identifier: AGPL-3.0-or-later
//! QSL and runtime IEEE agreement over checked scalar source.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use qsl_semantics::value::{admit_ieee_profile, DefinitionRole};
use quire_eval::FamilyOutcome;
use quire_exact::{
    evaluate_ieee, ChargePoint, IeeeFlag, IeeeFlags, IeeeOperation, IeeeValue, LimitKind, Meter,
    Outcome, RoundingMode, ScalarLimits, Value,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::loss::ValueLoss;
use quire_specification::{vectors_of, TestCase};

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

fn package(function: &str) -> CheckedPackage {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{function}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "scalar.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses the scalar source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds the source unit");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the source unit");
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("QSL admits the IEEE profile"),
    );
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("QSL checks the scalar source"),
    )
}

fn scalar_charges_match(qsl: &Meter, runtime: &Meter) {
    let Some(first) = qsl.admitted_charges().first() else {
        panic!("QSL records a function-call charge");
    };
    assert_eq!(first.point, ChargePoint::FunctionCall);
    assert_eq!(&qsl.admitted_charges()[1..], runtime.admitted_charges());
    for counter in LimitKind::ALL {
        let qsl_value = qsl.consumed(counter);
        let runtime_value = runtime.consumed(counter);
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(qsl_value, runtime_value + envelope, "{counter:?}");
    }
}

fn authored(test_case: TestCase, id: &str) {
    // Branch-gating QSpec ID comparison: missing authored rows fail.
    assert!(
        vectors_of(test_case).any(|vector| vector.id == id),
        "QSpec no longer declares {id} in {}",
        test_case.id()
    );
}

fn binary_ieee32(
    id: &str,
    symbol: &str,
    left_bits: u32,
    right_bits: u32,
    mode: RoundingMode,
    expected_bits: u32,
    expected_flags: IeeeFlags,
) {
    authored(TestCase::Tc193, id);
    let ty = format!("Float32[{}]", mode.as_str());
    let source = format!("function f using v(a: {ty}, b: {ty}): {ty} pure {{ a {symbol} b }}");
    let package = package(&source);
    let left = IeeeValue::binary32(left_bits);
    let right = IeeeValue::binary32(right_bits);
    let mut qsl_meter = Meter::new(LIMITS);
    let qsl = package
        .call(
            &QualifiedName::unqualified("f").expect("function name"),
            vec![Value::Float(left), Value::Float(right)],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("QSL evaluates the checked function");
    let mut runtime_meter = Meter::new(LIMITS);
    // Branch-gating source operator spelling: unknown operators fail.
    let operation = match symbol {
        "+" => IeeeOperation::Add(left, right),
        "-" => IeeeOperation::Subtract(left, right),
        "*" => IeeeOperation::Multiply(left, right),
        "/" => IeeeOperation::Divide(left, right),
        _ => panic!("unsupported IEEE operation in {id}: {symbol}"),
    };
    let runtime =
        evaluate_ieee(operation, mode, &mut runtime_meter).expect("same-width IEEE operands");
    let qsl_value = match qsl.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Float(value))) => value,
        other => panic!("QSL completes {id}, got {other:?}"),
    };
    let runtime_value = match runtime {
        Outcome::Completed(value) => value,
        other => panic!("runtime completes {id}, got {other:?}"),
    };
    assert_eq!(qsl_value, runtime_value.value(), "{id} two-sided bits");
    assert_eq!(
        qsl_value.bits(),
        u64::from(expected_bits),
        "{id} QSpec bits"
    );
    assert_eq!(runtime_value.flags(), expected_flags, "{id} IEEE flags");
    if expected_flags == IeeeFlags::EMPTY {
        assert!(qsl.losses.is_empty(), "{id} has no QSL loss");
    } else {
        assert_eq!(qsl.losses.len(), 1, "{id} has one QSL flag record");
        assert_eq!(&qsl.losses[0].loss, &ValueLoss::IeeeFlags(expected_flags));
    }
    scalar_charges_match(&qsl_meter, &runtime_meter);
}

fn sqrt_ieee32(
    id: &str,
    operand_bits: u32,
    mode: RoundingMode,
    expected_bits: u32,
    expected_flags: IeeeFlags,
) {
    authored(TestCase::Tc193, id);
    let ty = format!("Float32[{}]", mode.as_str());
    let source =
        format!("function f using v(a: {ty}): {ty} pure {{ quire::value::ieee::sqrt(a) }}");
    let package = package(&source);
    let operand = IeeeValue::binary32(operand_bits);
    let mut qsl_meter = Meter::new(LIMITS);
    let qsl = package
        .call(
            &QualifiedName::unqualified("f").expect("function name"),
            vec![Value::Float(operand)],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("QSL evaluates the checked square root");
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = evaluate_ieee(IeeeOperation::SquareRoot(operand), mode, &mut runtime_meter)
        .expect("IEEE operand width");
    let qsl_value = match qsl.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Float(value))) => value,
        other => panic!("QSL completes {id}, got {other:?}"),
    };
    let runtime_value = match runtime {
        Outcome::Completed(value) => value,
        other => panic!("runtime completes {id}, got {other:?}"),
    };
    assert_eq!(qsl_value, runtime_value.value(), "{id} two-sided bits");
    assert_eq!(
        qsl_value.bits(),
        u64::from(expected_bits),
        "{id} QSpec bits"
    );
    assert_eq!(runtime_value.flags(), expected_flags, "{id} IEEE flags");
    if expected_flags.is_empty() {
        assert!(qsl.losses.is_empty(), "{id} has no QSL loss");
    } else {
        assert_eq!(qsl.losses.len(), 1, "{id} has one QSL flag record");
        assert_eq!(&qsl.losses[0].loss, &ValueLoss::IeeeFlags(expected_flags));
    }
    scalar_charges_match(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f15_f16_subnormal_multiplication_preserves_flags() {
    let underflow_inexact = [IeeeFlag::Underflow, IeeeFlag::Inexact]
        .into_iter()
        .collect();
    binary_ieee32(
        "F15",
        "*",
        0x0080_0000,
        0x3f00_0000,
        RoundingMode::NearestEven,
        0x0040_0000,
        IeeeFlags::EMPTY,
    );
    binary_ieee32(
        "F16",
        "*",
        0x0000_0001,
        0x3f00_0000,
        RoundingMode::NearestEven,
        0x0000_0000,
        underflow_inexact,
    );
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f13_division_by_signed_zero_agrees_in_every_mode() {
    let divide_by_zero = [IeeeFlag::DivideByZero].into_iter().collect();
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::TowardPositive,
        RoundingMode::TowardNegative,
        RoundingMode::Exact,
    ] {
        for (left, right, expected) in [
            (0x3f80_0000, 0x0000_0000, 0x7f80_0000),
            (0xbf80_0000, 0x0000_0000, 0xff80_0000),
            (0x3f80_0000, 0x8000_0000, 0xff80_0000),
            (0xbf80_0000, 0x8000_0000, 0x7f80_0000),
        ] {
            binary_ieee32("F13", "/", left, right, mode, expected, divide_by_zero);
        }
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f12_invalid_operations_keep_the_canonical_nan_and_flag() {
    let invalid = [IeeeFlag::Invalid].into_iter().collect();
    for mode in [
        RoundingMode::NearestEven,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::TowardPositive,
        RoundingMode::TowardNegative,
        RoundingMode::Exact,
    ] {
        binary_ieee32(
            "F12",
            "*",
            0x0000_0000,
            0x7f80_0000,
            mode,
            0x7fc0_0000,
            invalid,
        );
        sqrt_ieee32("F12", 0xbf80_0000, mode, 0x7fc0_0000, invalid);
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f19_f30_square_roots_agree_on_rounded_and_signed_zero() {
    sqrt_ieee32(
        "F19",
        0x4000_0000,
        RoundingMode::NearestEven,
        0x3fb5_04f3,
        [IeeeFlag::Inexact].into_iter().collect(),
    );
    sqrt_ieee32(
        "F30",
        0x8000_0000,
        RoundingMode::NearestEven,
        0x8000_0000,
        IeeeFlags::EMPTY,
    );
}
