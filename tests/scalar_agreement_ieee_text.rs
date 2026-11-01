// SPDX-License-Identifier: AGPL-3.0-or-later
//! QSL and runtime IEEE and text agreement over checked scalar source.

use ix_trace_rs::trace;
use qsl_eval::value::QualifiedName;
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use qsl_semantics::value::{admit_ieee_profile, DefinitionRole};
use quire_eval::FamilyOutcome;
use quire_exact::{
    admit_text, compare_text, evaluate_ieee, ChargePoint, ComparisonOperator, IeeeFlag, IeeeFlags,
    IeeeOperation, IeeeValue, LimitKind, Meter, Outcome, RoundingMode, ScalarLimits, Text,
    TextPayload, TextProfile, TextType, Value,
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

fn package(function: &str, ieee: bool) -> CheckedPackage {
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
    if ieee {
        declarations.ieee_profile = Some(
            admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
                .expect("QSL admits the IEEE profile"),
        );
    }
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("QSL checks the scalar source"),
    )
}

fn scalar_charges_match(qsl: &Meter, runtime: &Meter) {
    let [ChargePoint::FunctionCall, scalar @ ..] = qsl.admitted_charges() else {
        panic!(
            "QSL first charges one function call: {:?}",
            qsl.admitted_charges()
        );
    };
    assert_eq!(scalar, runtime.admitted_charges());
    for counter in LimitKind::ALL {
        let qsl_value = qsl.consumed(counter);
        let runtime_value = runtime.consumed(counter);
        let envelope = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(qsl_value, runtime_value + envelope, "{counter:?}");
    }
}

fn text_value(source: &str, profile: TextProfile) -> Text {
    let payload = TextPayload::from_utf8(source.as_bytes()).expect("valid UTF-8 text");
    let ty = TextType::new(0, 16, profile).expect("bounded text type");
    let Outcome::Completed(text) = admit_text(&payload, &ty, &mut Meter::new(LIMITS)) else {
        panic!("the text operand admits under {profile:?}");
    };
    text
}

fn authored(test_case: TestCase, id: &str) {
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
    let package = package(&source, true);
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

fn compare_text_pair(
    id: &str,
    profile: TextProfile,
    left_source: &str,
    right_source: &str,
    operator: ComparisonOperator,
    expected: bool,
) {
    authored(TestCase::Tc186, id);
    let symbol = match operator {
        ComparisonOperator::Equal => "=",
        ComparisonOperator::Less => "<",
        _ => panic!("unsupported text comparison in {id}: {operator:?}"),
    };
    let source = format!(
        "function f using v(a: Text[0, 16; {profile}], b: Text[0, 16; {profile}]): Boolean pure {{ a {symbol} b }}",
        profile = profile.as_str()
    );
    let package = package(&source, false);
    let left = text_value(left_source, profile);
    let right = text_value(right_source, profile);
    let mut qsl_meter = Meter::new(LIMITS);
    let qsl = package
        .call(
            &QualifiedName::unqualified("f").expect("function name"),
            vec![Value::Text(left.clone()), Value::Text(right.clone())],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("QSL evaluates the checked function");
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime =
        compare_text(operator, &left, &right, &mut runtime_meter).expect("the text profiles agree");
    let qsl_value = match qsl.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(value))) => value,
        other => panic!("QSL completes {id}, got {other:?}"),
    };
    assert_eq!(qsl_value, expected, "{id} QSpec result");
    assert_eq!(runtime, Outcome::Completed(expected), "{id} runtime result");
    scalar_charges_match(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f03_nan_addition_retains_the_left_payload() {
    authored(TestCase::Tc193, "F03");
    let source = "function f using v(a: Float32[nearest-even], b: Float32[nearest-even]): Float32[nearest-even] pure { a + b }";
    let package = package(source, true);
    let left = IeeeValue::binary32(0xffc0_0021);
    let right = IeeeValue::binary32(0x7fc0_0012);
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
    let runtime = evaluate_ieee(
        IeeeOperation::Add(left, right),
        RoundingMode::NearestEven,
        &mut runtime_meter,
    )
    .expect("same-width IEEE operands");
    let qsl_value = match qsl.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Float(value))) => value,
        other => panic!("QSL completes F03, got {other:?}"),
    };
    let runtime_value = match runtime {
        Outcome::Completed(value) => value,
        other => panic!("runtime completes F03, got {other:?}"),
    };
    assert_eq!(qsl_value, runtime_value.value());
    assert_eq!(qsl_value.bits(), 0xffc0_0021);
    assert!(qsl.losses.is_empty(), "F03 raises no IEEE flag");
    scalar_charges_match(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f04_signaling_nan_is_quieted_and_flags_invalid() {
    let invalid = [IeeeFlag::Invalid].into_iter().collect();
    binary_ieee32(
        "F04",
        "+",
        0x7f80_0021,
        0x7fc0_0012,
        RoundingMode::NearestEven,
        0x7fc0_0021,
        invalid,
    );
    binary_ieee32(
        "F04b",
        "+",
        0x7fc0_0012,
        0xff80_0021,
        RoundingMode::NearestEven,
        0x7fc0_0012,
        invalid,
    );
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn ieee_f07_f08_round_in_each_direction() {
    let inexact = [IeeeFlag::Inexact].into_iter().collect();
    for (mode, positive, negative) in [
        (RoundingMode::NearestEven, 0x3f80_0000, 0xbf80_0000),
        (RoundingMode::TowardZero, 0x3f80_0000, 0xbf80_0000),
        (RoundingMode::TowardNegative, 0x3f80_0000, 0xbf80_0001),
        (RoundingMode::NearestAway, 0x3f80_0001, 0xbf80_0001),
        (RoundingMode::TowardPositive, 0x3f80_0001, 0xbf80_0000),
    ] {
        binary_ieee32(
            "F07",
            "+",
            0x3f80_0000,
            0x3380_0000,
            mode,
            positive,
            inexact,
        );
        binary_ieee32(
            "F08",
            "+",
            0xbf80_0000,
            0xb380_0000,
            mode,
            negative,
            inexact,
        );
    }
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
fn text_t01_unicode_scalar_sequences_remain_distinct() {
    authored(TestCase::Tc186, "T01");
    let source = "function f using v(a: Text[0, 16; unicode-scalars], b: Text[0, 16; unicode-scalars]): Boolean pure { a = b }";
    let package = package(source, false);
    let left = text_value("\u{e9}", TextProfile::UnicodeScalars);
    let right = text_value("e\u{301}", TextProfile::UnicodeScalars);
    let mut qsl_meter = Meter::new(LIMITS);
    let qsl = package
        .call(
            &QualifiedName::unqualified("f").expect("function name"),
            vec![Value::Text(left.clone()), Value::Text(right.clone())],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("QSL evaluates the checked function");
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = compare_text(ComparisonOperator::Equal, &left, &right, &mut runtime_meter)
        .expect("the text profiles agree");
    let qsl_value = match qsl.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(value))) => value,
        other => panic!("QSL completes T01, got {other:?}"),
    };
    assert!(!qsl_value);
    assert_eq!(runtime, Outcome::Completed(qsl_value));
    scalar_charges_match(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn text_t02_t03_normalization_profiles_select_exact_equality() {
    for profile in [TextProfile::Nfc, TextProfile::Nfd] {
        compare_text_pair(
            "T02",
            profile,
            "\u{e9}",
            "e\u{301}",
            ComparisonOperator::Equal,
            true,
        );
    }
    for profile in [TextProfile::Nfkc, TextProfile::Nfkd] {
        compare_text_pair(
            "T03",
            profile,
            "\u{fb00}",
            "ff",
            ComparisonOperator::Equal,
            true,
        );
    }
    compare_text_pair(
        "T03",
        TextProfile::Nfc,
        "\u{fb00}",
        "ff",
        ComparisonOperator::Equal,
        false,
    );
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn text_t12_scalar_and_utf8_ordering_agree() {
    compare_text_pair(
        "T12",
        TextProfile::UnicodeScalars,
        "a",
        "b",
        ComparisonOperator::Less,
        true,
    );
    compare_text_pair(
        "T12",
        TextProfile::BinaryUtf8,
        "\u{7f}",
        "\u{80}",
        ComparisonOperator::Less,
        true,
    );
}
