// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U12 compound quantity arithmetic values.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_eval::value::CheckedPackageEvaluation;
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{Integer, Meter, Outcome, ScalarLimits, UnitDomain, UnitId, Value};
use quire_semantic_value::checking::{CheckMode, CheckingLimits};
use quire_semantic_value::semantic_node::IDENTITY_LIMITS;
use quire_semantic_value::unit::compound_unit_id;
use quire_specification::{vectors_of, TestCase};

const U12_INPUTS: &str =
    "Multiply `2 m * 3 s`, divide `6 m / 2 s`, square `2 m`, and divide `5 m / 5 m`";
const U12_EXPECTED: &str = "exact values `6`, `3`, `4`, `1` under canonical compound units `m*s`, `m*s^-1`, `m^2`, and the empty dimensionless unit respectively";
const UNITS: &str = "dimension Length; unit m : Length = rational(1, 1); \
                     dimension Time; unit s : Time = rational(1, 1);";

fn source(declarations: &str) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    )
}

fn parsed_unit(
    declarations: &str,
) -> (
    qsl_foundation::source::provenance::RawSourceRef,
    qsl_forms::ParsedUnit,
) {
    let text = source(declarations);
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u12"),
        "tc187-u12.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U12 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    (
        parsed.source().reference().clone(),
        qsl_forms::build_unit(&parsed).expect("U12 source forms"),
    )
}

fn package() -> CheckedPackage {
    let (reference, unit) = parsed_unit(UNITS);
    let declarations = PackageDeclarations::assemble(
        reference,
        "agent-ix/tc187-u12".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("U12 unit declarations assemble");
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("U12 unit declarations check"),
    )
}

fn authored_expression(expression: &str) -> qsl_forms::Expression {
    let (_, unit) = parsed_unit(&format!(
        "{UNITS} function observe using v(): Boolean pure {{ \
         {expression} = {expression} }}"
    ));
    let body = unit
        .forms()
        .iter()
        .find_map(|form| match form.form() {
            qsl_forms::DeclarationForm::Function(function) if function.name == "observe" => {
                Some(function.body.clone())
            }
            _ => None,
        })
        .expect("authored observe expression");
    let children = body.root().children();
    let [left, _right] = children.as_slice() else {
        panic!("observe must be an authored equality");
    };
    body.cut(left.id(), &BTreeMap::new())
        .expect("the left authored arithmetic expression is complete")
}

fn generous_limits() -> ScalarLimits {
    ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: 1024,
        result_units: 1024,
    }
}

fn evaluate_quantity(package: &CheckedPackage, expression: &str) -> quire_exact::Quantity {
    let authored = authored_expression(expression);
    let checked = package
        .graph()
        .check_expression(
            Vec::new(),
            &authored,
            None,
            CheckMode::Kernel,
            CheckingLimits::default(),
        )
        .expect("authored quantity expression checks");
    let mut meter = Meter::new(generous_limits());
    let evaluation = package
        .evaluate(
            &checked,
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("checked quantity expression evaluates");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Quantity(quantity))) =
        evaluation.outcome
    else {
        panic!(
            "{expression} must complete as a quantity: {:?}",
            evaluation.outcome
        );
    };
    quantity
}

fn canonical_unit(terms: &[(UnitId, i64)]) -> UnitId {
    let mut terms: Vec<_> = terms
        .iter()
        .map(|(unit, power)| (*unit.as_bytes(), Integer::from(*power)))
        .collect();
    terms.sort_by_key(|(unit, _)| *unit);
    compound_unit_id(
        terms.iter().map(|(unit, power)| (*unit, power)),
        IDENTITY_LIMITS,
    )
    .expect("small canonical compound unit has an ID")
}

/// Trace: QSpec-TC-187 U12 and QSpec-FR-142. Four source operations retain exact values and canonical units.
#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn public_u12_arithmetic_values_have_their_canonical_compound_units() {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == "U12")
        .collect();
    let [row] = rows.as_slice() else {
        panic!("exactly one public TC-187 U12 row: {rows:?}");
    };
    assert_eq!(row.inputs, U12_INPUTS, "full authored U12 input");
    assert_eq!(
        row.expected, U12_EXPECTED,
        "full authored U12 expected cell"
    );

    let package = package();
    let meter = evaluate_quantity(&package, "1 m");
    let second = evaluate_quantity(&package, "1 s");
    let m = *meter.unit();
    let s = *second.unit();
    assert_eq!(m.domain(), UnitDomain::Declared);
    assert_eq!(s.domain(), UnitDomain::Declared);
    assert_ne!(m, s, "the two source dimensions have distinct roots");

    for (expression, magnitude, expected_unit) in [
        ("2 m * 3 s", "6/1", canonical_unit(&[(m, 1), (s, 1)])),
        ("6 m / 2 s", "3/1", canonical_unit(&[(m, 1), (s, -1)])),
        ("2 m ^ 2", "4/1", canonical_unit(&[(m, 2)])),
        ("5 m / 5 m", "1/1", canonical_unit(&[])),
    ] {
        let actual = evaluate_quantity(&package, expression);
        assert_eq!(
            actual.magnitude().to_string(),
            magnitude,
            "{expression} magnitude"
        );
        assert_eq!(actual.unit(), &expected_unit, "{expression} canonical unit");
    }
}
