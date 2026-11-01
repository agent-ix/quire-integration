// SPDX-License-Identifier: AGPL-3.0-or-later
//! Run checked QSL scalar source against the public runtime operation.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{Integer, IntegerDomain, LimitKind, Meter, Outcome, ScalarLimits, Value};
use quire_semantic_value::checking::CheckingLimits;

const LIMITS: ScalarLimits = ScalarLimits {
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
};

fn checked_package(source: &str) -> CheckedPackage {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{source}"
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
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the source unit");
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("QSL checks the scalar source"),
    )
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulo_through_qsl_matches_runtime() {
    let package = checked_package(
        "function modulo using v(a: Integer, b: Integer): Integer pure { a mod b }",
    );
    let mut qsl_meter = Meter::new(LIMITS);
    let qsl = package
        .call(
            &QualifiedName::unqualified("modulo").expect("a function name"),
            vec![
                Value::Integer(Integer::from(-7_i64)),
                Value::Integer(Integer::from(3_i64)),
            ],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("the checked function evaluates");

    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = quire_eval::modulo(
        &Integer::from(-7_i64),
        &Integer::from(3_i64),
        &IntegerDomain::Mathematical,
        &mut runtime_meter,
    );
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(qsl_value))) = qsl.outcome
    else {
        panic!("QSL completes modulo, got {:?}", qsl.outcome);
    };
    let Outcome::Completed(runtime_value) = runtime else {
        panic!("runtime completes modulo, got {runtime:?}");
    };
    assert_eq!(qsl_value, runtime_value);
    assert_eq!(runtime_value, Integer::from(2_i64));
    assert_eq!(
        qsl_meter.admitted_charges()[0].point,
        quire_exact::ChargePoint::FunctionCall
    );
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl_meter.consumed(counter),
            runtime_meter.consumed(counter) + call_work,
            "{counter:?}"
        );
    }
}
