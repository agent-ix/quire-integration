// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-187 exact unit conversion and addition through checked QSL source.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    IllTypedCause, Integer, Meter, Outcome, Quantity, Rational, ScalarLimits, Value,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::quantity::{
    convert_quantity, evaluate_quantity, QuantityOperation, QuantityTarget, UnitTable,
};
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

fn rational(numerator: i64, denominator: i64) -> Rational {
    Rational::new(Integer::from(numerator), Integer::from(denominator))
        .expect("nonzero denominator")
}

#[trace("TC-097", "IT-017-SC-02")]
#[test]
fn tc187_u01_conversion_then_addition_agrees_with_runtime() {
    // Branch-gating string comparison: only QSpec U01 owns this source case.
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|vector| vector.id == "U01")
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-187 must contain exactly one U01 row; found {}",
            rows.len()
        );
    };
    assert!(!row.inputs.is_empty(), "U01 has no authored inputs");
    assert!(!row.expected.is_empty(), "U01 has no authored expectation");

    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         dimension Length;\n\
         unit m : Length = rational(1, 1);\n\
         unit cm : Length = rational(1, 100) * m;\n\
         function converted using v(): Quantity<m> pure { convert<Quantity<m>>(250 cm) }\n\
         function sum using v(): Quantity<m> pure { convert<Quantity<m>>(250 cm) + 1 m }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "quantity.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the quantity source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the quantity source");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles the quantity source");
    let units = declarations.units.clone();
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("S4 checks the quantity source"),
    );
    let converted_evaluation = package
        .call(
            &QualifiedName::unqualified("converted").expect("converted is an identifier"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut Meter::new(LIMITS),
        )
        .expect("S6a calls the checked conversion function");
    let FamilyOutcome::Evaluated(converted_outcome) = converted_evaluation.outcome else {
        panic!(
            "QSL did not reach quantity conversion: {:?}",
            converted_evaluation.outcome
        );
    };
    let Outcome::Completed(Value::Quantity(qsl_converted)) = converted_outcome else {
        panic!("QSL did not complete quantity conversion: {converted_outcome:?}");
    };
    assert!(
        converted_evaluation.losses.is_empty(),
        "exact conversion has no loss"
    );
    let evaluation = package
        .call(
            &QualifiedName::unqualified("sum").expect("sum is an identifier"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut Meter::new(LIMITS),
        )
        .expect("S6a calls the checked quantity function");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!(
            "QSL did not reach quantity evaluation: {:?}",
            evaluation.outcome
        );
    };
    let Outcome::Completed(Value::Quantity(qsl_sum)) = outcome else {
        panic!("QSL did not complete quantity sum: {outcome:?}");
    };
    assert!(evaluation.losses.is_empty(), "exact conversion has no loss");

    let root_scale = rational(1, 1);
    let centimeter_scale = rational(1, 100);
    let mut root = None;
    let mut centimeter = None;
    for unit in units.units() {
        if unit.canonical().scale() == &root_scale {
            assert!(root.replace(unit).is_none(), "one canonical metre");
        } else if unit.canonical().scale() == &centimeter_scale {
            assert!(centimeter.replace(unit).is_none(), "one centimetre");
        }
    }
    let root = root.expect("the checked graph has a metre");
    let centimeter = centimeter.expect("the checked graph has a centimetre");
    let table = UnitTable::declared(&units);
    let source_quantity = Quantity::new(rational(250, 1), centimeter.id());
    let root_quantity = Quantity::new(rational(1, 1), root.id());
    let source = table.resolve(&source_quantity).expect("cm resolves");
    let conversion = convert_quantity(
        source,
        table.get(root.id()).expect("m resolves"),
        &QuantityTarget::Exact,
        &mut Meter::new(LIMITS),
    )
    .expect("same-dimension conversion type-checks");
    let Outcome::Completed(conversion) = conversion else {
        panic!("runtime did not complete conversion: {conversion:?}");
    };
    let (converted, _) = conversion
        .identified_quantity()
        .expect("exact conversion yields a quantity");
    assert_eq!(qsl_converted, converted, "source and runtime conversion");
    assert_eq!(format!("{qsl_converted:?}"), format!("{converted:?}"));
    assert_eq!(
        qsl_converted.magnitude(),
        &rational(5, 2),
        "QSpec U01 conversion"
    );
    let left = table.resolve(&converted).expect("converted m resolves");
    let right = table.resolve(&root_quantity).expect("literal m resolves");
    let runtime = evaluate_quantity(QuantityOperation::Add(left, right), &mut Meter::new(LIMITS))
        .expect("same-unit addition type-checks");
    let Outcome::Completed(runtime_sum) = runtime else {
        panic!("runtime did not complete addition: {runtime:?}");
    };
    assert_eq!(qsl_sum, runtime_sum, "source and runtime quantity sums");
    assert_eq!(format!("{qsl_sum:?}"), format!("{runtime_sum:?}"));
    assert_eq!(qsl_sum.magnitude(), &rational(7, 2), "QSpec U01 sum");
}

#[trace("TC-097", "IT-017-SC-04")]
#[test]
fn tc187_u03_incompatible_dimensions_refuse_at_source_check() {
    // Branch-gating string comparison: U03 owns this admission case.
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|vector| vector.id == "U03")
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-187 must contain exactly one U03 row; found {}",
            rows.len()
        );
    };
    assert!(!row.inputs.is_empty(), "U03 has no authored inputs");
    assert!(!row.expected.is_empty(), "U03 has no authored expectation");

    let source = "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         dimension Length;\n\
         dimension Time;\n\
         unit m : Length = rational(1, 1);\n\
         unit s : Time = rational(1, 1);\n\
         function bad using v(a: Quantity<m>, b: Quantity<s>): Quantity<m> pure { a + b }\n";
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669", "git", "1"),
        "quantity-incompatible.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("S1 parses the incompatible quantity source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("S2 builds the incompatible source");
    let refusal = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        Vec::new(),
        Vec::new(),
    )
    .expect("S3 assembles the incompatible source")
    .check(CheckingLimits::default())
    .expect_err("S4 rejects addition across distinct dimensions");
    assert_eq!(
        refusal.cause,
        CheckCause::IllTyped(IllTypedCause::TypeMismatch)
    );
}
