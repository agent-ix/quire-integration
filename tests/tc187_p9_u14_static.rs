// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original U14 source refusals; admission evidence without runtime/native credit.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{
    IllTyped, IllTypedCause, Integer, LimitKind, Meter, Quantity, Rational, ScalarLimits,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::quantity::{
    evaluate_quantity, QuantityOperation, QuantityRefusal, UnitTable,
};
use quire_specification::{vectors_of, TestCase};

const DECLARATIONS: &str = "\
    language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    dimension Distance; unit m : Distance = rational(1, 1);\n\
    unit cm : Distance = rational(1, 100) * m;\n\
    dimension Duration; unit s : Duration = rational(1, 1);\n\
    dimension Temperature; unit K : Temperature = rational(1, 1);\n\
    unit degC : Temperature = rational(1, 1) * K + rational(5463, 20);\n";

/// Runtime L is irrelevant until successful checking publishes executable source.
/// Ordinary compiler CheckingLimits keep the static cause observable.
#[trace(
    "IT-019-SC-04",
    "TC-106",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-2",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn original_u14_operations_refuse_at_their_source_occurrence() {
    let mut rows = vectors_of(TestCase::Tc187).filter(|row| row.id == "U14");
    let row = rows.next().expect("public U14 vector");
    assert!(rows.next().is_none(), "U14 is unique");
    assert!(row.inputs.contains("L(0,0,0,0,0,0)"), "{}", row.inputs);
    assert!(row.expected.contains("ill_typed"), "{}", row.expected);
    assert!(row.expected.contains("no charge"), "{}", row.expected);
    assert!(row
        .expected
        .contains("rather than reaching the runtime zero divisor"));

    for (operation, left_unit, right_unit, right_value, divide, cause) in [
        (
            "1 m + 1 s",
            "m",
            "s",
            1_i64,
            false,
            IllTypedCause::IncompatibleDimensions,
        ),
        (
            "1 degC + 1 s",
            "degC",
            "s",
            1,
            false,
            IllTypedCause::IncompatibleDimensions,
        ),
        (
            "1 degC + 1 degC",
            "degC",
            "degC",
            1,
            false,
            IllTypedCause::AffineUnitArithmetic,
        ),
        (
            "1 degC / 0 degC",
            "degC",
            "degC",
            0,
            true,
            IllTypedCause::AffineUnitArithmetic,
        ),
        (
            "1 cm + 1 m",
            "cm",
            "m",
            1,
            false,
            IllTypedCause::DistinctUnits,
        ),
    ] {
        assert!(row.inputs.contains(operation), "{}", row.inputs);
        let source = format!(
            "{DECLARATIONS}function candidate using v(): Quantity<m> pure {{ {operation} }}\n"
        );
        let parsed = qsl_cst::parse(
            qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u14-static"),
            "tc187-u14-static.native",
            source.as_bytes(),
            qsl_cst::Limits::default(),
        )
        .expect("original U14 source parses");
        assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
        let source_ref = parsed.source().reference().clone();
        let forms = qsl_forms::build_unit(&parsed).expect("original U14 source forms");
        let declarations = PackageDeclarations::assemble(
            source_ref.clone(),
            "agent-ix/tc187-u14-static".to_owned(),
            forms,
            SelectedModels::default(),
            Vec::new(),
        )
        .expect("original U14 units assemble");
        let keys: BTreeMap<_, _> = declarations
            .nominal_spans
            .iter()
            .map(|(key, span)| (&source[span.start..span.end], key.clone()))
            .collect();
        let unit = |name| {
            declarations
                .units
                .unit(keys[name].clone())
                .expect("declared unit")
        };
        assert_ne!(keys["cm"], keys["m"]);
        assert_eq!(unit("cm").dimension(), unit("m").dimension());
        assert_ne!(unit("m").dimension(), unit("s").dimension());
        assert_ne!(unit("degC").dimension(), unit("s").dimension());
        assert_eq!(unit("degC").root(), keys["K"]);
        assert!(unit("degC").is_affine());
        assert_eq!(
            unit("degC").canonical().offset(),
            &Rational::new(Integer::from(5463_i64), Integer::from(20_i64)).expect("Celsius offset")
        );
        assert_eq!(
            unit("cm").canonical().scale(),
            &Rational::new(Integer::from(1_i64), Integer::from(100_i64)).expect("centimetre scale")
        );
        let table = UnitTable::declared(&declarations.units);
        let quantity = |name, value| {
            Quantity::new(
                Rational::from_integer(Integer::from(value)),
                declarations
                    .units
                    .declared_unit_id(keys[name].clone())
                    .expect("original unit id"),
            )
        };
        let left = quantity(left_unit, 1_i64);
        let right = quantity(right_unit, right_value);
        let left = table.resolve(&left).expect("original left unit");
        let right = table.resolve(&right).expect("original right unit");
        let quantity_operation = if divide {
            QuantityOperation::Divide(left, right)
        } else {
            QuantityOperation::Add(left, right)
        };
        let mut meter = Meter::new(ScalarLimits {
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
        });
        assert_eq!(
            evaluate_quantity(quantity_operation, &mut meter),
            Err(QuantityRefusal::IllTyped(IllTyped { cause })),
            "{operation}"
        );
        assert!(meter.admitted_charges().is_empty(), "{operation}");
        for kind in LimitKind::ALL {
            assert_eq!(meter.consumed(kind), 0, "{operation}: {kind:?}");
        }
        let regions = declarations.regions();
        let refusals = declarations
            .check(CheckingLimits::default())
            .expect_err("static U14 refusal prevents checked-package publication");
        let [refusal] = refusals.as_slice() else {
            panic!("{operation}: one static refusal: {refusals:?}");
        };
        assert_eq!(
            refusal.cause,
            CheckCause::IllTyped(IllTypedCause::TypeMismatch),
            "{operation}"
        );
        assert_eq!(refusal.cause.code().as_str(), "ill_typed", "{operation}");
        assert_eq!(
            refusal
                .cause
                .catalog_code()
                .expect("catalog code")
                .to_string(),
            "ill_typed/type-mismatch",
            "{operation}"
        );
        let region = regions
            .refusal_region(refusal)
            .expect("original refusal region");
        assert_eq!(region.source(), &source_ref, "{operation}");
        let start = usize::try_from(region.start()).expect("source start");
        let end = usize::try_from(region.end()).expect("source end");
        assert_eq!(&source[start..end], operation, "{refusal:?}");
    }
}
