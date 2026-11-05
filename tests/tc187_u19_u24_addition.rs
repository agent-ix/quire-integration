// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 same-unit addition values and charged boundaries through QSL.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_eval::value::CheckedPackageEvaluation;
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    ChargePoint, Incomplete, Integer, LimitKind, Meter, NodeKey, Outcome, ScalarLimits, Value,
};
use quire_semantic_value::checking::{CheckMode, CheckingLimits};
use quire_specification::{vectors_of, TestCase};

const U19_INPUTS: &str = "Add `2 m + 3 m` under `L(4,0,0,2,5,1)`, then under `L(4,0,0,2,4,1)`";
const U19_EXPECTED: &str = "`5 m` after the add event with `max(bits(2) + bits(1), bits(3) + bits(1)) + 1 = 4`; then `incomplete { limit_kind: work_units, limit: 4, consumed: 4, next_charge: 1, charge_point: unit.result-retain }`: the rational target charges `unit.target-domain` with one work unit and no size amount";
const U24_INPUTS: &str = "Add `1 cm + 2 cm` under `L(4,0,0,2,5,1)`";
const U24_EXPECTED: &str = "exact `3 cm` with no `unit.edge`: two reads, one addition event in `cm`, target admission and retention";

fn bind_vector(id: &str, inputs: &str, expected: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {rows:?}");
    };
    assert_eq!(row.inputs, inputs, "{id} full authored input");
    assert_eq!(row.expected, expected, "{id} full authored expected cell");
}

fn limits(work_units: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: 4,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 2,
        work_units,
        result_units: 1,
    }
}

fn evaluate_add(
    unit_declarations: &str,
    unit_name: &str,
    addition: &str,
    work_units: u64,
) -> (Outcome<Value>, Meter, NodeKey) {
    let text = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         {unit_declarations}\n\
         function observe using v(): Boolean pure {{ {addition} = {addition} }}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u19-u24"),
        "tc187-u19-u24.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("quantity addition source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("quantity addition source forms");
    let body = unit
        .forms()
        .iter()
        .find_map(|form| match form.form() {
            qsl_forms::DeclarationForm::Function(function) if function.name == "observe" => {
                Some(function.body.clone())
            }
            _ => None,
        })
        .expect("authored observe function");
    let children = body.root().children();
    let [left, _right] = children.as_slice() else {
        panic!("observe must compare two source additions");
    };
    let expression = body
        .cut(left.id(), &BTreeMap::new())
        .expect("the authored left addition is a complete subtree");

    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u19-u24".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("quantity units assemble");
    let unit_keys: Vec<_> = assembled
        .nominal_spans
        .iter()
        .filter(|(key, span)| {
            &text[span.start..span.end] == unit_name
                && assembled.units.unit((*key).clone()).is_some()
        })
        .map(|(key, _)| key.clone())
        .collect();
    let [unit_key] = unit_keys.as_slice() else {
        panic!("one declared {unit_name} unit: {unit_keys:?}");
    };
    let package = CheckedPackage::link(
        assembled
            .check(CheckingLimits::default())
            .expect("the source addition checks"),
    );
    let checked = package
        .graph()
        .check_expression(
            Vec::new(),
            &expression,
            None,
            CheckMode::Linked,
            CheckingLimits::default(),
        )
        .expect("the original addition subtree checks");
    let mut meter = Meter::new(limits(work_units));
    let evaluation = package
        .evaluate(
            &checked,
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("the checked addition evaluates");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!("the value evaluator must run: {:?}", evaluation.outcome);
    };
    (outcome, meter, unit_key.clone())
}

fn points(meter: &Meter) -> Vec<ChargePoint> {
    meter
        .admitted_charges()
        .iter()
        .map(|charge| charge.point)
        .collect()
}

/// Trace: QSpec-TC-187 U19, QSpec-FR-142-AC-1 and QSpec-FR-142-AC-7.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-1", "QSpec-FR-142-AC-7")]
#[test]
fn public_u19_addition_reaches_exact_value_and_first_unpaid_retention() {
    bind_vector("U19", U19_INPUTS, U19_EXPECTED);
    let declarations = "dimension L; unit m : L = rational(1, 1);";
    let (completed, meter, metre) = evaluate_add(declarations, "m", "2 m + 3 m", 5);
    let Outcome::Completed(Value::Quantity(quantity)) = completed else {
        panic!("U19 must complete with a quantity: {completed:?}");
    };
    assert_eq!(quantity.magnitude().to_string(), "5/1");
    assert_eq!(quantity.unit(), &metre);
    assert_eq!(
        points(&meter),
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain,
        ]
    );
    assert_eq!(meter.consumed(LimitKind::IntegerBits), 4);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 5);
    assert_eq!(meter.consumed(LimitKind::UnitEdges), 0);

    let (denied, meter, _) = evaluate_add(declarations, "m", "2 m + 3 m", 4);
    let Outcome::Incomplete(actual) = denied else {
        panic!("U19 must stop before result retention: {denied:?}");
    };
    assert_eq!(
        actual,
        Incomplete {
            limit_kind: LimitKind::WorkUnits,
            limit: 4,
            consumed: 4,
            next_charge: Integer::from(1_i64),
            charge_point: ChargePoint::UnitResultRetain,
        }
    );
    assert_eq!(
        points(&meter),
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
        ]
    );
    assert_eq!(meter.consumed(LimitKind::IntegerBits), 4);
}

/// Trace: QSpec-TC-187 U24, QSpec-FR-142-AC-1 and QSpec-FR-142-AC-7.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-1", "QSpec-FR-142-AC-7")]
#[test]
fn public_u24_same_centimetre_operands_do_not_visit_a_unit_edge() {
    bind_vector("U24", U24_INPUTS, U24_EXPECTED);
    let declarations =
        "dimension L; unit m : L = rational(1, 1); unit cm : L = rational(1, 100) * m;";
    let (completed, meter, centimetre) = evaluate_add(declarations, "cm", "1 cm + 2 cm", 5);
    let Outcome::Completed(Value::Quantity(quantity)) = completed else {
        panic!("U24 must complete with a quantity: {completed:?}");
    };
    assert_eq!(quantity.magnitude().to_string(), "3/1");
    assert_eq!(quantity.unit(), &centimetre);
    assert_eq!(
        points(&meter),
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain,
        ]
    );
    assert_eq!(meter.consumed(LimitKind::UnitEdges), 0);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 5);
}
