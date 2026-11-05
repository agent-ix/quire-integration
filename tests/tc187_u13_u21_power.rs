// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 integer-power values and charged refusal boundaries.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    ChargePoint, Incomplete, Integer, LimitKind, Meter, Outcome, ScalarLimits, Undefined, Value,
};
use quire_semantic_value::checking::{CheckMode, CheckingLimits};
use quire_semantic_value::semantic_node::IDENTITY_LIMITS;
use quire_semantic_value::unit::compound_unit_id;
use quire_specification::{vectors_of, TestCase};

const U13_INPUTS: &str = "Raise `2 m` to integer power `18446744073709551616` with `ScalarLimitsV1 { integer_bits: 64, decimal_digits: 0, scale_expansion: 0, text_input_bytes: 0, text_scalars: 0, normalized_scalars: 0, unit_edges: 0, value_occurrences: 1, work_units: 5, result_units: 1 }`";
const U13_EXPECTED: &str = "`incomplete { limit_kind: integer_bits, limit: 64, consumed: 2, next_charge: 36893488147419103232, charge_point: unit.rational-arithmetic }` before the power is computed, since the amount is `abs(n) × maxparts(2) = 2^64 × 2`; `next_charge` exceeds `u64::MAX` and is reported exactly";
const U21_INPUTS: &str = "Raise `0 m` to integer power `0` under `L(1,0,0,1,4,1)`; raise `0 m` to `-1` under `L(1,0,0,1,1,0)`, then under `L(1,0,0,1,0,0)`";
const U21_EXPECTED: &str = "exact dimensionless `1` after read, power event, target admission and retention; undefined after one `unit.identity-read`; then `incomplete { limit_kind: work_units, limit: 0, consumed: 0, next_charge: 1, charge_point: unit.identity-read }`";

const UNITS: &str = "dimension Length; unit m : Length = rational(1, 1);";

fn bind_vector(id: &str, input: &str, expected: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {rows:?}");
    };
    assert_eq!(row.inputs, input, "{id} full authored input");
    assert_eq!(row.expected, expected, "{id} full authored expected cell");
}

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
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u13-u21"),
        "tc187-u13-u21.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("power source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    (
        parsed.source().reference().clone(),
        qsl_forms::build_unit(&parsed).expect("power source forms"),
    )
}

fn checked_package(declarations: &str) -> CheckedPackage {
    let (reference, unit) = parsed_unit(declarations);
    let declarations = PackageDeclarations::assemble(
        reference,
        "agent-ix/tc187-u13-u21".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("power declarations assemble");
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("power declarations check"),
    )
}

fn expression_body(declarations: &str) -> qsl_forms::Expression {
    let (_, unit) = parsed_unit(declarations);
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
        panic!("observe must be an authored equality over two power expressions");
    };
    body.cut(left.id(), &BTreeMap::new())
        .expect("the left authored power is a complete subtree")
}

fn limits(integer_bits: u64, work_units: u64, result_units: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 1,
        work_units,
        result_units,
    }
}

/// Trace: QSpec-TC-187 U13 and QSpec-FR-142. A larger-than-u64 exponent retains its exact denied amount.
#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn public_u13_power_reports_the_exact_over_u64_denied_charge() {
    bind_vector("U13", U13_INPUTS, U13_EXPECTED);
    let package = checked_package(&format!(
        "{UNITS} function observe using v(): Boolean pure {{ \
         2 m ^ 18446744073709551616 = 2 m ^ 18446744073709551616 }}"
    ));
    let mut meter = Meter::new(limits(64, 5, 1));
    let evaluation = package
        .call(
            &QualifiedName::unqualified("observe").expect("authored name"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("the checked source call runs");
    let FamilyOutcome::Evaluated(Outcome::Incomplete(actual)) = evaluation.outcome else {
        panic!("U13 must stop at the power event: {:?}", evaluation.outcome);
    };
    assert_eq!(
        actual,
        Incomplete {
            limit_kind: LimitKind::IntegerBits,
            limit: 64,
            consumed: 2,
            next_charge: "36893488147419103232"
                .parse::<Integer>()
                .expect("exact charge"),
            charge_point: ChargePoint::UnitRationalArithmetic,
        }
    );
    assert!(actual.next_charge > Integer::from(u64::MAX));
}

fn run_power(exponent: i64, work_units: u64, result_units: u64) -> (Outcome<Value>, Meter) {
    let package = checked_package(UNITS);
    // The expression is cut from the left side of a genuine source equality;
    // the enclosing function is not called, so its call charge is absent.
    let expression = expression_body(&format!(
        "{UNITS} function observe using v(): Boolean pure {{ \
         0 m ^ {exponent} = 0 m ^ {exponent} }}"
    ));
    let checked = package
        .graph()
        .check_expression(
            Vec::new(),
            &expression,
            None,
            CheckMode::Kernel,
            CheckingLimits::default(),
        )
        .expect("power expression checks without a return-type envelope");
    let mut meter = Meter::new(limits(1, work_units, result_units));
    let evaluation = package
        .evaluate(
            &checked,
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("checked power expression evaluates");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!("power must reach value authority: {:?}", evaluation.outcome);
    };
    (outcome, meter)
}

/// Trace: QSpec-TC-187 U21 and QSpec-FR-142. Zero powers and read denial retain distinct outcomes.
#[trace("QSpec-TC-187", "QSpec-FR-142")]
#[test]
fn public_u21_zero_powers_observe_value_undefined_and_first_read_denial() {
    bind_vector("U21", U21_INPUTS, U21_EXPECTED);

    let (completed, meter) = run_power(0, 4, 1);
    let Outcome::Completed(Value::Quantity(quantity)) = completed else {
        panic!("0 m ^ 0 must complete as a quantity: {completed:?}");
    };
    assert_eq!(quantity.magnitude().to_string(), "1/1");
    let dimensionless = compound_unit_id(std::iter::empty(), IDENTITY_LIMITS)
        .expect("the empty canonical compound unit has an ID");
    assert_eq!(quantity.unit(), &dimensionless, "the unit map is empty");
    let points: Vec<_> = meter
        .admitted_charges()
        .iter()
        .map(|charge| charge.point)
        .collect();
    assert_eq!(
        points,
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain,
        ]
    );

    let (undefined, meter) = run_power(-1, 1, 0);
    assert!(
        matches!(undefined, Outcome::Undefined(Undefined::DivisionByZero)),
        "0 m ^ -1 must be undefined after its read: {undefined:?}"
    );
    assert_eq!(
        meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect::<Vec<_>>(),
        [ChargePoint::UnitIdentityRead]
    );

    let (incomplete, meter) = run_power(-1, 0, 0);
    let Outcome::Incomplete(actual) = incomplete else {
        panic!("work-zero must stop before the read: {incomplete:?}");
    };
    assert_eq!(
        actual,
        Incomplete {
            limit_kind: LimitKind::WorkUnits,
            limit: 0,
            consumed: 0,
            next_charge: Integer::from(1_i64),
            charge_point: ChargePoint::UnitIdentityRead,
        }
    );
    assert!(meter.admitted_charges().is_empty());
}
