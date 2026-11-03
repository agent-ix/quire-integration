// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-186 text comparison through checked QSL source and the exact runtime.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    admit_text, compare_text, ChargePoint, ComparisonOperator, IllTyped, IllTypedCause, Incomplete,
    InjectedDenial, Integer, LimitKind, Meter, Outcome, Refusal, ScalarLimits, Text, TextPayload,
    TextProfile, TextProvenance, TextType, Value,
};
use quire_semantic_value::checking::CheckingLimits;
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
    let rows: Vec<_> = vectors_of(TestCase::Tc186)
        .filter(|vector| vector.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-186 must contain exactly one {id} row; found {}",
            rows.len()
        );
    };
    assert!(!row.inputs.is_empty(), "{id} has no authored inputs");
    assert!(!row.expected.is_empty(), "{id} has no authored expectation");
    row
}

fn codepoint_sequence(text: &str) -> String {
    let scalars: Vec<_> = text
        .split("U+")
        .skip(1)
        .map(|tail| {
            let digits: String = tail
                .chars()
                .take_while(|ch| ch.is_ascii_hexdigit())
                .collect();
            let codepoint =
                u32::from_str_radix(&digits, 16).expect("QSpec codepoint is hexadecimal");
            char::from_u32(codepoint).expect("QSpec codepoint is a Unicode scalar")
        })
        .collect();
    assert!(!scalars.is_empty(), "QSpec needs codepoint inputs: {text}");
    scalars.into_iter().collect()
}

fn text(payload: &str, profile: TextProfile) -> Text {
    admit_text(
        &TextPayload::from_utf8(payload.as_bytes()).expect("valid UTF-8"),
        &TextType::new(0, 16, profile).expect("valid text bounds"),
        &mut Meter::new(LIMITS),
    )
    .completed()
    .expect("both operands admit into the selected text type")
}

fn admit(
    payload: &str,
    bounds: (u64, u64),
    profile: TextProfile,
    meter: &mut Meter,
) -> Outcome<Text> {
    admit_text(
        &TextPayload::from_utf8(payload.as_bytes()).expect("valid UTF-8"),
        &TextType::new(bounds.0, bounds.1, profile).expect("valid text bounds"),
        meter,
    )
}

fn t01_operands() -> (String, String) {
    let row = selected_vector("T01");
    let (left, right) = row
        .inputs
        .split_once(" and ")
        .expect("T01 has two operands");
    (codepoint_sequence(left), codepoint_sequence(right))
}

fn source_literal(value: &str) -> String {
    serde_json::to_string(value).expect("QSpec Unicode scalar sequence is a source string")
}

fn checked_text_literal(
    bounds: (u64, u64),
    profile: TextProfile,
    literal: &str,
) -> Result<Text, CheckCause> {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function literal using v(): Text[{}, {}; {}] pure {{ Text({literal}) }}\n",
        bounds.0,
        bounds.1,
        profile.as_str(),
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "text-type.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses bounded Text");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds bounded Text");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles bounded Text");
    let checked = declarations
        .check(CheckingLimits::default())
        .map_err(|mut refusals| {
            assert_eq!(refusals.len(), 1, "one Text literal admission refusal");
            refusals.remove(0).cause
        })?;
    let package = CheckedPackage::link(checked);
    let mut meter = Meter::new(LIMITS);
    let evaluation = package
        .call(
            &QualifiedName::unqualified("literal").expect("literal is an identifier"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls checked Text literal function");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Text(value))) = evaluation.outcome
    else {
        panic!(
            "checked Text literal did not complete: {:?}",
            evaluation.outcome
        );
    };
    assert!(evaluation.losses.is_empty(), "Text literal has no loss");
    assert_eq!(meter.admitted_charges()[0].point, ChargePoint::FunctionCall);
    Ok(value)
}

fn assert_checked_text_literal_refusal(
    bounds: (u64, u64),
    profile: TextProfile,
    value: &str,
    retained_length: u64,
) {
    let literal = source_literal(value);
    let refusal = checked_text_literal(bounds, profile, &literal)
        .expect_err("S4 refuses a Text literal outside its retained bounds");
    assert_eq!(
        refusal,
        CheckCause::TextLiteralOutsideBounds {
            argument: value.to_owned(),
            min: bounds.0,
            max: bounds.1,
            profile,
            retained_length,
        }
    );
}

fn checked_compare_with_limits(
    operator: ComparisonOperator,
    profile: TextProfile,
    left: &Text,
    right: &Text,
    limits: ScalarLimits,
    denial: Option<InjectedDenial>,
) -> (Outcome<Value>, Meter) {
    let symbol = match operator {
        ComparisonOperator::Equal => "=",
        ComparisonOperator::Less => "<",
        other => panic!("unsupported TC-186 checked comparison: {other:?}"),
    };
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function equal using v(a: Text[0, 16; {profile}], b: Text[0, 16; {profile}]): \
         Boolean pure {{ a {symbol} b }}\n",
        profile = profile.as_str(),
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "text.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the text source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the text source");
    let package = CheckedPackage::link(
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "ir-669".to_owned(),
            unit,
            Vec::new(),
            Vec::new(),
        )
        .expect("S3 assembles the text source")
        .check(CheckingLimits::default())
        .expect("S4 checks the text source"),
    );
    let mut meter = Meter::new(limits);
    if let Some(denial) = denial {
        meter = meter.with_injected_denial(denial);
    }
    let evaluation = package
        .call(
            &QualifiedName::unqualified("equal").expect("equal is an identifier"),
            vec![Value::Text(left.clone()), Value::Text(right.clone())],
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("S6a calls the checked text function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach text evaluation: {:?}",
            evaluation.outcome
        );
    };
    assert!(evaluation.losses.is_empty(), "text equality has no loss");
    (outcome, meter)
}

fn checked_equal_with_limits(
    profile: TextProfile,
    left: &Text,
    right: &Text,
    limits: ScalarLimits,
    denial: Option<InjectedDenial>,
) -> (Outcome<Value>, Meter) {
    checked_compare_with_limits(
        ComparisonOperator::Equal,
        profile,
        left,
        right,
        limits,
        denial,
    )
}

fn checked_equal(profile: TextProfile, left: &Text, right: &Text) -> (bool, Meter) {
    let (outcome, meter) = checked_equal_with_limits(profile, left, right, LIMITS, None);
    let Outcome::Completed(Value::Boolean(equal)) = outcome else {
        panic!("QSL did not complete text equality: {outcome:?}");
    };
    (equal, meter)
}

fn assert_agreement(profile: TextProfile, left: &str, right: &str, expected: bool) {
    let left = text(left, profile);
    let right = text(right, profile);
    let (qsl_equal, qsl_meter) = checked_equal(profile, &left, &right);
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = compare_text(ComparisonOperator::Equal, &left, &right, &mut runtime_meter)
        .expect("same-profile text operands type-check");
    let Outcome::Completed(runtime_equal) = runtime else {
        panic!("runtime did not complete text equality: {runtime:?}");
    };
    assert_eq!(
        qsl_equal, runtime_equal,
        "{profile:?} source/runtime equality"
    );
    assert_eq!(format!("{qsl_equal:?}"), format!("{runtime_equal:?}"));
    assert_eq!(qsl_equal, expected, "{profile:?} QSpec equality");
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges(),
        "{profile:?} charge sequence"
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + call_work,
            "{profile:?} {counter:?}"
        );
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc186_t01_t02_text_profiles_agree_from_checked_source_to_runtime() {
    let t01 = selected_vector("T01");
    let t02 = selected_vector("T02");
    // Branch-gating QSpec prose split: T01 authors the two scalar sequences.
    let (left, right) = t01
        .inputs
        .split_once(" and ")
        .expect("T01 has two operands");
    let composed = codepoint_sequence(left);
    let decomposed = codepoint_sequence(right);
    // Branch-gating QSpec reference: T02 must use T01's operands.
    assert!(t02.inputs.contains("T01"), "T02 must refer to T01 operands");
    assert_agreement(TextProfile::UnicodeScalars, &composed, &decomposed, false);
    for (profile, retained) in [
        (TextProfile::Nfc, composed.as_str()),
        (TextProfile::Nfd, decomposed.as_str()),
    ] {
        assert_eq!(text(&composed, profile).retained(), retained);
        assert_eq!(text(&decomposed, profile).retained(), retained);
        assert_agreement(profile, &composed, &decomposed, true);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc186_t03_compatibility_profiles_agree_from_checked_source_to_runtime() {
    let row = selected_vector("T03");
    // Branch-gating QSpec input spelling: T03 selects the ligature and ff.
    let (ligature, letters) = row
        .inputs
        .split_once(" and ")
        .expect("T03 has two operands");
    let ligature = codepoint_sequence(ligature);
    assert!(letters.contains("`ff`"), "T03's second operand is ff");
    // Branch-gating QSpec expected spelling: compatibility profiles agree.
    assert!(
        row.expected.contains("equal under compatibility profiles"),
        "T03 expects compatibility equality"
    );
    assert_agreement(TextProfile::Nfkc, &ligature, "ff", true);
    assert_agreement(TextProfile::Nfkd, &ligature, "ff", true);
    assert_agreement(TextProfile::Nfc, &ligature, "ff", false);
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t05b_bare_text_type_refuses_at_source_recognition() {
    selected_vector("T05b");
    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function invalid using v(a: Text): Boolean pure { a = a }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "bare-text.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 recognizes the source envelope");
    assert!(!parsed.is_admissible(), "bare Text has no required bounds");
    assert!(
        parsed
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == qsl_foundation::Code::InvalidSyntax),
        "S1 must report invalid syntax: {:?}",
        parsed.diagnostics()
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t06_invalid_utf8_refuses_at_both_reader_boundaries() {
    let row = selected_vector("T06");
    // Branch-gating QSpec prose split: T06 authors two invalid UTF-8 bytes.
    let bytes = row
        .inputs
        .split_once("bytes ")
        .and_then(|(_, tail)| tail.split_once(" presented"))
        .map(|(hex, _)| {
            hex.trim_matches('`')
                .split_whitespace()
                .map(|byte| u8::from_str_radix(byte, 16).expect("QSpec byte is hexadecimal"))
                .collect::<Vec<_>>()
        })
        .expect("T06 names reader bytes");
    assert_eq!(bytes.len(), 2, "T06 has two reader bytes");
    let source_refusal = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "invalid-text.native",
        &bytes,
        qsl_cst::Limits::default(),
    )
    .expect_err("S1 rejects invalid UTF-8 before source parsing");
    assert_eq!(source_refusal.code, qsl_foundation::Code::InvalidUtf8);
    let runtime_refusal = TextPayload::from_utf8(&bytes)
        .expect_err("runtime payload reader rejects invalid UTF-8 before text admission");
    assert_eq!(runtime_refusal.valid_up_to, 0);
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t04_t05_text_bounds_admit_and_refuse() {
    let t04 = selected_vector("T04");
    let t05 = selected_vector("T05");
    assert!(t04.inputs.contains("Text[1,1;nfc]"));
    assert!(t05.inputs.contains("Text[0,0;nfc]"));
    let composed = codepoint_sequence(&t04.inputs);
    assert_eq!(composed, "é");
    let composed_literal = source_literal(&composed);
    let qsl_nfc = checked_text_literal((1, 1), TextProfile::Nfc, &composed_literal)
        .expect("S4 and S6a admit one normalized scalar");
    assert_eq!(qsl_nfc.retained(), composed);
    assert_eq!(qsl_nfc.length(), 1);
    assert_checked_text_literal_refusal((1, 1), TextProfile::BinaryUtf8, &composed, 2);
    assert_eq!(
        checked_text_literal((0, 0), TextProfile::Nfc, "\"\"")
            .expect("S4 and S6a admit empty Text")
            .length(),
        0
    );
    assert_checked_text_literal_refusal((0, 0), TextProfile::Nfc, "a", 1);
    assert_eq!(
        checked_text_literal((1, 1), TextProfile::Nfc, "\"a\"")
            .expect("S4 and S6a admit exact-bound Text")
            .retained(),
        "a"
    );
    let admitted = admit(&composed, (1, 1), TextProfile::Nfc, &mut Meter::new(LIMITS))
        .completed()
        .expect("one normalized scalar admits");
    assert_eq!(admitted.length(), 1);
    assert!(matches!(
        admit(
            &composed,
            (1, 1),
            TextProfile::BinaryUtf8,
            &mut Meter::new(LIMITS)
        ),
        Outcome::Refused(Refusal::TextLengthOutOfDomain { .. })
    ));
    assert_eq!(
        admit("", (0, 0), TextProfile::Nfc, &mut Meter::new(LIMITS))
            .completed()
            .expect("empty text admits")
            .length(),
        0
    );
    assert!(matches!(
        admit("a", (0, 0), TextProfile::Nfc, &mut Meter::new(LIMITS)),
        Outcome::Refused(Refusal::TextLengthOutOfDomain { .. })
    ));
    assert_eq!(
        admit("a", (1, 1), TextProfile::Nfc, &mut Meter::new(LIMITS))
            .completed()
            .expect("exact bound admits")
            .length(),
        1
    );
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t06b_checked_text_values_agree_with_runtime_bytes() {
    let row = selected_vector("T06b");
    // Source-spelling provenance remains PENDING IR-1286: QSL constructs it
    // from decoded text, so the raw and escaped spellings are not distinguished.
    assert!(row.inputs.contains("\\u00e9"));
    assert!(row.inputs.contains("binary-utf8"));
    let raw_spelling = "\"é\"";
    let escaped_spelling = "\"\\u00e9\"";
    let raw = checked_text_literal((2, 2), TextProfile::BinaryUtf8, raw_spelling)
        .expect("S4 and S6a admit raw source Text");
    let escaped = checked_text_literal((2, 2), TextProfile::BinaryUtf8, escaped_spelling)
        .expect("S4 and S6a admit escaped source Text");
    let runtime_payload = TextPayload::from_utf8(&[0xc3, 0xa9]).expect("valid runtime bytes");
    let ty = TextType::new(2, 2, TextProfile::BinaryUtf8).expect("valid bounds");
    let runtime = admit_text(&runtime_payload, &ty, &mut Meter::new(LIMITS))
        .completed()
        .expect("runtime bytes admit");
    assert!(matches!(
        raw.payload().provenance(),
        TextProvenance::SourceLiteral(_)
    ));
    assert!(matches!(
        escaped.payload().provenance(),
        TextProvenance::SourceLiteral(_)
    ));
    assert_eq!(runtime.payload().provenance(), &TextProvenance::Runtime);
    for left in [&raw, &escaped, &runtime] {
        assert_eq!(left.retained().as_bytes(), [0xc3, 0xa9]);
        for right in [&raw, &escaped, &runtime] {
            assert_eq!(
                compare_text(
                    ComparisonOperator::Equal,
                    left,
                    right,
                    &mut Meter::new(LIMITS)
                )
                .expect("same profile")
                .completed(),
                Some(true)
            );
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t10_distinct_text_profiles_refuse_before_charge() {
    let row = selected_vector("T10");
    assert!(row.inputs.contains("binary-utf8"));
    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         function wrong using v(a: Text[0, 16; nfc], b: Text[0, 16; binary-utf8]): \
         Boolean pure { a = b }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "mixed-profiles.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses mixed profiles");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds mixed profiles");
    let refusals = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles mixed profiles")
    .check(CheckingLimits::default())
    .expect_err("S4 refuses distinct text profiles");
    assert!(refusals
        .iter()
        .any(|refusal| refusal.cause.code() == qsl_foundation::Code::IllTyped));
    let left = text("é", TextProfile::Nfc);
    let right = text("é", TextProfile::BinaryUtf8);
    let mut meter = Meter::new(LIMITS);
    assert_eq!(
        compare_text(ComparisonOperator::Equal, &left, &right, &mut meter),
        Err(IllTyped {
            cause: IllTypedCause::DistinctTextProfiles
        })
    );
    assert!(meter.admitted_charges().is_empty());
}

const T11_LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 0,
    decimal_digits: 0,
    scale_expansion: 0,
    text_input_bytes: 5,
    text_scalars: 3,
    normalized_scalars: 2,
    unit_edges: 0,
    value_occurrences: 2,
    work_units: 6,
    result_units: 1,
};

const T14_LIMITS: ScalarLimits = ScalarLimits {
    normalized_scalars: 0,
    work_units: 3,
    ..T11_LIMITS
};

fn assert_meter_agreement(qsl: &Meter, runtime: &Meter) {
    assert_eq!(qsl.admitted_charges()[0].point, ChargePoint::FunctionCall);
    assert_eq!(&qsl.admitted_charges()[1..], runtime.admitted_charges());
    for kind in LimitKind::ALL {
        assert_eq!(
            qsl.consumed(kind),
            runtime.consumed(kind) + u64::from(kind == LimitKind::WorkUnits),
            "{kind:?}"
        );
    }
}

fn assert_order_agreement(id: &str, profile: TextProfile, left: &str, right: &str) {
    let left = text(left, profile);
    let right = text(right, profile);
    let (qsl, qsl_meter) = checked_compare_with_limits(
        ComparisonOperator::Less,
        profile,
        &left,
        &right,
        LIMITS,
        None,
    );
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = compare_text(ComparisonOperator::Less, &left, &right, &mut runtime_meter)
        .expect("same-profile T12 operands type-check");
    assert_eq!(
        qsl,
        Outcome::Completed(Value::Boolean(true)),
        "{id} QSL order"
    );
    assert_eq!(runtime, Outcome::Completed(true), "{id} runtime order");
    assert_meter_agreement(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc186_t12_scalar_and_unsigned_utf8_order_agree() {
    selected_vector("T12");
    assert_order_agreement("T12 scalar", TextProfile::UnicodeScalars, "a", "b");
    assert_order_agreement("T12 binary", TextProfile::BinaryUtf8, "\u{7f}", "\u{80}");
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc186_t11_nfc_exact_limits_and_named_denials_agree() {
    let row = selected_vector("T11");
    assert!(row.inputs.contains("text.normalize-output"));
    let (composed, decomposed) = t01_operands();
    let (left, right) = (
        text(&composed, TextProfile::Nfc),
        text(&decomposed, TextProfile::Nfc),
    );
    let qsl_limits = ScalarLimits {
        work_units: 7,
        ..T11_LIMITS
    };
    let (qsl, qsl_meter) =
        checked_equal_with_limits(TextProfile::Nfc, &left, &right, qsl_limits, None);
    let mut runtime_meter = Meter::new(T11_LIMITS);
    let runtime = compare_text(ComparisonOperator::Equal, &left, &right, &mut runtime_meter)
        .expect("same profile");
    assert!(matches!(qsl, Outcome::Completed(Value::Boolean(true))));
    assert_eq!(runtime, Outcome::Completed(true));
    assert_meter_agreement(&qsl_meter, &runtime_meter);
    let points: Vec<_> = runtime_meter
        .admitted_charges()
        .iter()
        .map(|charge| charge.point)
        .collect();
    assert_eq!(
        points,
        [
            ChargePoint::TextInputBytes,
            ChargePoint::TextDecodeScalars,
            ChargePoint::TextNormalizeInput,
            ChargePoint::TextNormalizeOutput,
            ChargePoint::TextNormalizeOutput,
            ChargePoint::TextResultRetain,
        ]
    );
    for (kind, count) in [
        (LimitKind::TextInputBytes, 5),
        (LimitKind::TextScalars, 3),
        (LimitKind::NormalizedScalars, 2),
        (LimitKind::ValueOccurrences, 2),
        (LimitKind::WorkUnits, 6),
        (LimitKind::ResultUnits, 1),
    ] {
        assert_eq!(runtime_meter.consumed(kind), count, "{kind:?}");
    }
    for (point, occurrence) in [
        (ChargePoint::TextNormalizeOutput, 2),
        (ChargePoint::TextResultRetain, 1),
    ] {
        let denial = InjectedDenial {
            point,
            occurrence: NonZeroU64::new(occurrence).expect("nonzero"),
        };
        let (qsl, qsl_meter) =
            checked_equal_with_limits(TextProfile::Nfc, &left, &right, qsl_limits, Some(denial));
        let mut runtime_meter = Meter::new(T11_LIMITS).with_injected_denial(denial);
        let runtime = compare_text(ComparisonOperator::Equal, &left, &right, &mut runtime_meter)
            .expect("same profile");
        assert!(
            matches!(qsl, Outcome::Incomplete(Incomplete { charge_point, .. }) if charge_point == point)
        );
        assert!(
            matches!(runtime, Outcome::Incomplete(Incomplete { charge_point, .. }) if charge_point == point)
        );
        assert_meter_agreement(&qsl_meter, &runtime_meter);
        assert_eq!(runtime_meter.consumed(LimitKind::ResultUnits), 0);
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn tc186_t14_non_normalizing_comparison_and_one_under_agree() {
    let row = selected_vector("T14");
    assert!(row.inputs.contains("work_units: 2"));
    let (composed, decomposed) = t01_operands();
    for profile in [TextProfile::UnicodeScalars, TextProfile::BinaryUtf8] {
        let (left, right) = (text(&composed, profile), text(&decomposed, profile));
        for work in [3, 2] {
            let runtime_limits = ScalarLimits {
                work_units: work,
                ..T14_LIMITS
            };
            let qsl_limits = ScalarLimits {
                work_units: work + 1,
                ..T14_LIMITS
            };
            let (qsl, qsl_meter) =
                checked_equal_with_limits(profile, &left, &right, qsl_limits, None);
            let mut runtime_meter = Meter::new(runtime_limits);
            let runtime =
                compare_text(ComparisonOperator::Equal, &left, &right, &mut runtime_meter)
                    .expect("same profile");
            if work == 3 {
                assert!(matches!(qsl, Outcome::Completed(Value::Boolean(false))));
                assert_eq!(runtime, Outcome::Completed(false));
            } else {
                assert!(matches!(qsl, Outcome::Incomplete(Incomplete {
                    limit_kind: LimitKind::WorkUnits, limit: 3, consumed: 3,
                    next_charge, charge_point: ChargePoint::TextResultRetain,
                }) if next_charge == Integer::from(1_u64)));
                assert_eq!(
                    runtime,
                    Outcome::Incomplete(Incomplete {
                        limit_kind: LimitKind::WorkUnits,
                        limit: 2,
                        consumed: 2,
                        next_charge: Integer::from(1_u64),
                        charge_point: ChargePoint::TextResultRetain,
                    })
                );
            }
            assert_meter_agreement(&qsl_meter, &runtime_meter);
            assert_eq!(runtime_meter.consumed(LimitKind::NormalizedScalars), 0);
            let points: Vec<_> = runtime_meter
                .admitted_charges()
                .iter()
                .map(|charge| charge.point)
                .collect();
            assert_eq!(
                points,
                [ChargePoint::TextInputBytes, ChargePoint::TextDecodeScalars]
                    .into_iter()
                    .chain((work == 3).then_some(ChargePoint::TextResultRetain))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t13_normalized_length_controls_admission() {
    let row = selected_vector("T13");
    assert!(row.inputs.contains("Text[1,1;nfd]"));
    let (composed, decomposed) = t01_operands();
    let decomposed_literal = source_literal(&decomposed);
    let composed_literal = source_literal(&composed);
    let qsl_nfc = checked_text_literal((1, 1), TextProfile::Nfc, &decomposed_literal)
        .expect("S4 and S6a normalize decomposed source to one scalar");
    assert_eq!(qsl_nfc.retained(), composed);
    assert_checked_text_literal_refusal((1, 1), TextProfile::Nfd, &decomposed, 2);
    assert_checked_text_literal_refusal((1, 1), TextProfile::UnicodeScalars, &decomposed, 2);
    let qsl_nfd = checked_text_literal((2, 2), TextProfile::Nfd, &composed_literal)
        .expect("S4 and S6a normalize composed source to two scalars");
    assert_eq!(qsl_nfd.retained(), decomposed);
    assert_eq!(
        admit(
            &decomposed,
            (1, 1),
            TextProfile::Nfc,
            &mut Meter::new(LIMITS)
        )
        .completed()
        .expect("NFC normalizes to one scalar")
        .length(),
        1
    );
    for profile in [TextProfile::Nfd, TextProfile::UnicodeScalars] {
        assert!(matches!(
            admit(&decomposed, (1, 1), profile, &mut Meter::new(LIMITS)),
            Outcome::Refused(Refusal::TextLengthOutOfDomain { .. })
        ));
    }
    let nfd = admit(&composed, (2, 2), TextProfile::Nfd, &mut Meter::new(LIMITS))
        .completed()
        .expect("NFD normalizes to two scalars");
    assert_eq!(nfd.length(), 2);
    assert_eq!(nfd.retained(), decomposed);
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn tc186_t16_admission_limit_precedence_is_exact() {
    let row = selected_vector("T16");
    assert!(row.inputs.contains("normalized_scalars: 1"));
    let (composed, decomposed) = t01_operands();
    assert_checked_text_literal_refusal((1, 1), TextProfile::Nfd, &decomposed, 2);
    assert_checked_text_literal_refusal((1, 1), TextProfile::UnicodeScalars, &decomposed, 2);
    assert_checked_text_literal_refusal((1, 1), TextProfile::BinaryUtf8, &composed, 2);
    let base = ScalarLimits {
        integer_bits: 0,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 3,
        text_scalars: 2,
        normalized_scalars: 2,
        unit_edges: 0,
        value_occurrences: 1,
        work_units: 5,
        result_units: 1,
    };
    let scalar = ScalarLimits {
        normalized_scalars: 0,
        work_units: 2,
        ..base
    };
    let binary = ScalarLimits {
        text_input_bytes: 2,
        text_scalars: 0,
        normalized_scalars: 0,
        work_units: 1,
        ..base
    };
    for (payload, profile, limits, expected_points, denied, expected) in [
        (
            &decomposed,
            TextProfile::Nfd,
            base,
            &[
                ChargePoint::TextInputBytes,
                ChargePoint::TextDecodeScalars,
                ChargePoint::TextNormalizeInput,
                ChargePoint::TextNormalizeOutput,
                ChargePoint::TextNormalizeOutput,
            ][..],
            ScalarLimits {
                normalized_scalars: 1,
                ..base
            },
            Incomplete {
                limit_kind: LimitKind::NormalizedScalars,
                limit: 1,
                consumed: 1,
                next_charge: Integer::from(2_u64),
                charge_point: ChargePoint::TextNormalizeOutput,
            },
        ),
        (
            &decomposed,
            TextProfile::UnicodeScalars,
            scalar,
            &[ChargePoint::TextInputBytes, ChargePoint::TextDecodeScalars][..],
            ScalarLimits {
                text_scalars: 1,
                ..scalar
            },
            Incomplete {
                limit_kind: LimitKind::TextScalars,
                limit: 1,
                consumed: 0,
                next_charge: Integer::from(2_u64),
                charge_point: ChargePoint::TextDecodeScalars,
            },
        ),
        (
            &composed,
            TextProfile::BinaryUtf8,
            binary,
            &[ChargePoint::TextInputBytes][..],
            ScalarLimits {
                text_input_bytes: 1,
                ..binary
            },
            Incomplete {
                limit_kind: LimitKind::TextInputBytes,
                limit: 1,
                consumed: 0,
                next_charge: Integer::from(2_u64),
                charge_point: ChargePoint::TextInputBytes,
            },
        ),
    ] {
        let mut meter = Meter::new(limits);
        assert!(matches!(
            admit(payload, (1, 1), profile, &mut meter),
            Outcome::Refused(Refusal::TextLengthOutOfDomain { .. })
        ));
        let points: Vec<_> = meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect();
        assert_eq!(points.as_slice(), expected_points);
        assert!(matches!(
            admit(payload, (1, 1), profile, &mut Meter::new(denied)),
            Outcome::Incomplete(record) if record == expected
        ));
    }
}

mod enum_rows {
    use ix_trace_rs::trace;
    use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
    use qsl_package::CheckedPackage;
    use qsl_semantics::check::{EnumBinding, PackageDeclarations};
    use qsl_semantics::model::object_environment::ObjectEnvironment;
    use quire_eval::FamilyOutcome;
    use quire_exact::{
        ChargePoint, ComparisonOperator, IllTypedCause, LimitKind, Meter, Outcome, ScalarLimits,
        Value,
    };
    use quire_semantic_value::checking::CheckingLimits;
    use quire_semantic_value::enumeration::{compare_enum, EnumValue};
    use quire_specification::{vectors_of, TestCase};

    const UNBOUNDED: ScalarLimits = ScalarLimits {
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

    const ZERO: ScalarLimits = ScalarLimits {
        integer_bits: 0,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 0,
        work_units: 0,
        result_units: 0,
    };

    fn selected_vector(id: &str) {
        let rows: Vec<_> = vectors_of(TestCase::Tc186)
            .filter(|vector| vector.id == id)
            .collect();
        let [row] = rows.as_slice() else {
            panic!(
                "QSpec TC-186 must contain exactly one {id} row; found {}",
                rows.len()
            );
        };
        assert!(!row.inputs.is_empty(), "{id} has no authored inputs");
        assert!(!row.expected.is_empty(), "{id} has no authored expectation");
    }

    fn assemble_with_package(body: &str, package_name: &str) -> PackageDeclarations {
        let source = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n{body}\n"
        );
        let parsed = qsl_cst::parse(
            qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
            "enum.native",
            source.as_bytes(),
            qsl_cst::Limits::default(),
        )
        .expect("S1 parses enum source");
        assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
        let unit = qsl_forms::build_unit(&parsed).expect("S2 builds enum source");
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            package_name.to_owned(),
            unit,
            Vec::new(),
            Vec::new(),
        )
        .expect("S3 assembles enum source")
    }

    fn assemble(body: &str) -> PackageDeclarations {
        assemble_with_package(body, "ir-669")
    }

    fn member<'a>(binding: &'a EnumBinding, case: &str) -> &'a EnumValue {
        binding
            .members
            .iter()
            .find(|value| value.case() == case)
            .unwrap_or_else(|| panic!("admitted {} lacks {case}", binding.name))
    }

    fn checked_true(body: &str) -> Meter {
        let package = CheckedPackage::link(
            assemble(body)
                .check(CheckingLimits::default())
                .expect("S4 checks enum comparison"),
        );
        let mut meter = Meter::new(UNBOUNDED);
        let evaluation = package
            .call(
                &QualifiedName::unqualified("compare").expect("compare is an identifier"),
                Vec::new(),
                &ObjectEnvironment::default(),
                &mut meter,
            )
            .expect("S6a calls checked enum function");
        let FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(true))) = evaluation.outcome
        else {
            panic!(
                "checked enum comparison did not complete true: {:?}",
                evaluation.outcome
            );
        };
        assert!(evaluation.losses.is_empty(), "enum comparison has no loss");
        meter
    }

    fn checked_ill_typed(body: &str) {
        let refusals = assemble(body)
            .check(CheckingLimits::default())
            .expect_err("S4 refuses an ill-typed enum comparison");
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.cause.code() == qsl_foundation::Code::IllTyped),
            "{refusals:?}"
        );
    }

    #[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
    #[test]
    pub(super) fn tc186_t07_cross_declaration_is_ill_typed_at_source_and_runtime() {
        selected_vector("T07");
        let body = "enum A { READY }\nenum B { READY }\n\
                    function compare using v(): Boolean pure { A::READY = B::READY }";
        let declarations = assemble(body);
        let [left, right] = declarations.enums() else {
            panic!("T07 admits exactly two enum declarations");
        };
        assert_ne!(left.declaration.key(), right.declaration.key());
        let mut meter = Meter::new(UNBOUNDED);
        let error = compare_enum(
            ComparisonOperator::Equal,
            member(left, "READY"),
            member(right, "READY"),
            &mut meter,
        )
        .expect_err("distinct admitted declarations cannot compare");
        assert_eq!(error.cause, IllTypedCause::DistinctEnumDeclarations);
        assert!(meter.admitted_charges().is_empty());
        checked_ill_typed(body);
    }

    #[trace(
        "TC-097",
        "IT-017-SC-01",
        "IT-017-SC-02",
        "IT-017-SC-03",
        "IT-017-SC-04"
    )]
    #[test]
    pub(super) fn tc186_t08_ordered_members_agree_and_unordered_order_refuses() {
        selected_vector("T08");
        let ordered = "ordered enum A { READY, DONE }\n\
                       function compare using v(): Boolean pure { A::READY < A::DONE }";
        let declarations = assemble(ordered);
        let [binding] = declarations.enums() else {
            panic!("T08 admits one enum declaration");
        };
        let mut meter = Meter::new(UNBOUNDED);
        let result = compare_enum(
            ComparisonOperator::Less,
            member(binding, "READY"),
            member(binding, "DONE"),
            &mut meter,
        )
        .expect("same ordered declaration compares");
        assert_eq!(result, Outcome::Completed(true));
        let qsl_meter = checked_true(ordered);
        assert_eq!(
            qsl_meter.admitted_charges()[0].point,
            ChargePoint::FunctionCall
        );
        assert_eq!(&qsl_meter.admitted_charges()[1..], meter.admitted_charges());
        for counter in LimitKind::ALL {
            let call_work = u64::from(counter == LimitKind::WorkUnits);
            assert_eq!(
                qsl_meter.consumed(counter),
                meter.consumed(counter) + call_work
            );
        }

        let unordered = "enum A { READY, DONE }\n\
                         function compare using v(): Boolean pure { A::READY < A::DONE }";
        let declarations = assemble(unordered);
        let [binding] = declarations.enums() else {
            panic!("T08 admits one unordered declaration");
        };
        let mut meter = Meter::new(UNBOUNDED);
        let error = compare_enum(
            ComparisonOperator::Less,
            member(binding, "READY"),
            member(binding, "DONE"),
            &mut meter,
        )
        .expect_err("unordered enum cannot be ordered");
        assert_eq!(error.cause, IllTypedCause::UnorderedEnumOrdering);
        assert!(meter.admitted_charges().is_empty());
        checked_ill_typed(unordered);
    }

    #[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
    #[test]
    pub(super) fn tc186_t09_changed_owner_case_or_order_has_distinct_identity() {
        selected_vector("T09");
        let old = assemble("ordered enum A { READY, DONE }");
        let changed_case = assemble("ordered enum A { READY, CLOSED }");
        let changed_order = assemble("ordered enum A { DONE, READY }");
        let changed_owner =
            assemble_with_package("ordered enum A { READY, DONE }", "ir-669-other-owner");
        let old = &old.enums()[0];
        for changed in [
            &changed_owner.enums()[0],
            &changed_case.enums()[0],
            &changed_order.enums()[0],
        ] {
            assert_ne!(old.declaration.key(), changed.declaration.key());
            let mut meter = Meter::new(UNBOUNDED);
            let error = compare_enum(
                ComparisonOperator::Equal,
                member(old, "READY"),
                member(changed, "READY"),
                &mut meter,
            )
            .expect_err("a changed admitted declaration has a new identity");
            assert_eq!(error.cause, IllTypedCause::DistinctEnumDeclarations);
            assert!(meter.admitted_charges().is_empty());
        }
        checked_ill_typed(
            "ordered enum A { READY, DONE }\nordered enum B { READY, CLOSED }\n\
             function compare using v(): Boolean pure { A::READY = B::READY }",
        );
    }

    #[trace("TC-097", "IT-017-SC-01", "IT-017-SC-04")]
    #[test]
    pub(super) fn tc186_t15_precedence_is_ill_typed_with_zero_limits_and_no_charge() {
        selected_vector("T15");
        let declarations = assemble("enum A { DONE, READY }\nenum B { READY }");
        let [a, b] = declarations.enums() else {
            panic!("T15 admits two declarations");
        };
        for (operator, left, right, cause) in [
            (
                ComparisonOperator::Less,
                member(a, "READY"),
                member(a, "DONE"),
                IllTypedCause::UnorderedEnumOrdering,
            ),
            (
                ComparisonOperator::Equal,
                member(a, "READY"),
                member(b, "READY"),
                IllTypedCause::DistinctEnumDeclarations,
            ),
        ] {
            let mut meter = Meter::new(ZERO);
            let error = compare_enum(operator, left, right, &mut meter)
                .expect_err("type refusal precedes every zero limit");
            assert_eq!(error.cause, cause);
            assert!(meter.admitted_charges().is_empty());
        }
        checked_ill_typed(
            "enum A { DONE, READY }\nfunction compare using v(): Boolean pure { A::READY < A::DONE }",
        );
        checked_ill_typed(
            "enum A { READY }\nenum B { READY }\n\
             function compare using v(): Boolean pure { A::READY = B::READY }",
        );
    }
}

enum TRoute {
    Runnable(fn()),
    Partial {
        run: fn(),
        ticket: &'static str,
        missing: &'static str,
    },
}

fn t_route(id: &str) -> TRoute {
    // Branch-gating QSpec ID match: a new row must have a runnable or ticket-backed route.
    match id {
        "T01" | "T02" => {
            TRoute::Runnable(tc186_t01_t02_text_profiles_agree_from_checked_source_to_runtime)
        }
        "T03" => {
            TRoute::Runnable(tc186_t03_compatibility_profiles_agree_from_checked_source_to_runtime)
        }
        "T04" | "T05" => TRoute::Runnable(tc186_t04_t05_text_bounds_admit_and_refuse),
        "T05b" => TRoute::Runnable(tc186_t05b_bare_text_type_refuses_at_source_recognition),
        "T06" => TRoute::Runnable(tc186_t06_invalid_utf8_refuses_at_both_reader_boundaries),
        "T06b" => TRoute::Partial {
            run: tc186_t06b_checked_text_values_agree_with_runtime_bytes,
            ticket: "IR-1286",
            missing: "QSL must retain distinct raw and escaped Text source spellings",
        },
        "T07" => TRoute::Runnable(
            enum_rows::tc186_t07_cross_declaration_is_ill_typed_at_source_and_runtime,
        ),
        "T08" => {
            TRoute::Runnable(enum_rows::tc186_t08_ordered_members_agree_and_unordered_order_refuses)
        }
        "T09" => {
            TRoute::Runnable(enum_rows::tc186_t09_changed_owner_case_or_order_has_distinct_identity)
        }
        "T10" => TRoute::Runnable(tc186_t10_distinct_text_profiles_refuse_before_charge),
        "T11" => TRoute::Runnable(tc186_t11_nfc_exact_limits_and_named_denials_agree),
        "T12" => TRoute::Runnable(tc186_t12_scalar_and_unsigned_utf8_order_agree),
        "T13" => TRoute::Runnable(tc186_t13_normalized_length_controls_admission),
        "T14" => TRoute::Runnable(tc186_t14_non_normalizing_comparison_and_one_under_agree),
        "T15" => TRoute::Runnable(
            enum_rows::tc186_t15_precedence_is_ill_typed_with_zero_limits_and_no_charge,
        ),
        "T16" => TRoute::Runnable(tc186_t16_admission_limit_precedence_is_exact),
        other => panic!("unrouted QSpec TC-186 T vector {other}"),
    }
}

#[trace("TC-097", "IT-017-SC-01")]
#[test]
fn tc186_every_published_t_row_has_a_runnable_or_ticket_backed_route() {
    let mut seen = 0usize;
    for row in vectors_of(TestCase::Tc186) {
        seen += 1;
        match t_route(row.id) {
            TRoute::Runnable(run) => run(),
            TRoute::Partial {
                run,
                ticket,
                missing,
            } => {
                assert!(
                    !ticket.is_empty() && !missing.is_empty(),
                    "{id} has a precise pending owner",
                    id = row.id
                );
                run();
            }
        }
    }
    assert!(seen > 0, "QSpec TC-186 offers no vectors");
}
