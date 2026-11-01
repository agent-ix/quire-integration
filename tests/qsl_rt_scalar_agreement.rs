// SPDX-License-Identifier: AGPL-3.0-or-later
//! Run checked QSL scalar source against the public runtime operation.

use ix_trace_rs::trace;
use qsl_eval::value::{CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    DivisionMember, DivisionProfile, Integer, IntegerDomain, LimitKind, Meter, Outcome,
    ScalarLimits, Value,
};
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

fn checked_package(source: &str, profile: Option<DivisionProfile>) -> CheckedPackage {
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
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the source unit");
    if let Some(profile) = profile {
        use qsl_semantics::value::{admit_integer_division, DefinitionRole};
        let role = match profile {
            DivisionProfile::Truncating => DefinitionRole::IntegerDivisionTruncating,
            DivisionProfile::Floor => DefinitionRole::IntegerDivisionFloor,
            DivisionProfile::Euclidean => DefinitionRole::IntegerDivisionEuclidean,
        };
        declarations.integer_division = Some(
            admit_integer_division(&[role.reference()], None)
                .expect("the selected division definition admits"),
        );
    }
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("QSL checks the scalar source"),
    )
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulo_through_qsl_matches_runtime() {
    let (qsl, qsl_meter, runtime, runtime_meter) =
        checked_modulo(-7, 3, LIMITS, LIMITS, IntegerDomain::Mathematical);
    let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
        panic!("QSL completes modulo, got {qsl:?}");
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

fn checked_modulo(
    a: i64,
    b: i64,
    qsl_limits: ScalarLimits,
    runtime_limits: ScalarLimits,
    domain: IntegerDomain,
) -> (Outcome<Value>, Meter, Outcome<Integer>, Meter) {
    let package = checked_package(
        "function modulo using v(a: Integer, b: Integer): Integer pure { a mod b }",
        None,
    );
    let mut qsl_meter = Meter::new(qsl_limits);
    let qsl = package
        .call(
            &QualifiedName::unqualified("modulo").expect("a function name"),
            vec![
                Value::Integer(Integer::from(a)),
                Value::Integer(Integer::from(b)),
            ],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("the checked function evaluates");

    let mut runtime_meter = Meter::new(runtime_limits);
    let runtime = quire_eval::modulo(
        &Integer::from(a),
        &Integer::from(b),
        &domain,
        &mut runtime_meter,
    );
    let FamilyOutcome::Evaluated(qsl_outcome) = qsl.outcome else {
        panic!("QSL modulo did not reach evaluation: {:?}", qsl.outcome);
    };
    (qsl_outcome, qsl_meter, runtime, runtime_meter)
}

fn checked_division(
    profile: DivisionProfile,
    member: DivisionMember,
    a: i64,
    b: i64,
) -> (Outcome<Value>, Meter, Outcome<Integer>, Meter) {
    checked_division_with_limits(profile, member, a, b, LIMITS, LIMITS)
}

fn checked_division_with_limits(
    profile: DivisionProfile,
    member: DivisionMember,
    a: i64,
    b: i64,
    qsl_limits: ScalarLimits,
    runtime_limits: ScalarLimits,
) -> (Outcome<Value>, Meter, Outcome<Integer>, Meter) {
    let operator = match member {
        DivisionMember::Quotient => "div",
        DivisionMember::Remainder => "rem",
    };
    let source = format!(
        "function operation using v(a: Integer, b: Integer): Integer pure {{ a {operator} b }}"
    );
    let package = checked_package(&source, Some(profile));
    let mut qsl_meter = Meter::new(qsl_limits);
    let qsl = package
        .call(
            &QualifiedName::unqualified("operation").expect("a function name"),
            vec![
                Value::Integer(Integer::from(a)),
                Value::Integer(Integer::from(b)),
            ],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("the checked division function evaluates");
    let FamilyOutcome::Evaluated(qsl_outcome) = qsl.outcome else {
        panic!("QSL division did not reach evaluation: {:?}", qsl.outcome);
    };
    let mut runtime_meter = Meter::new(runtime_limits);
    let runtime_outcome = quire_exact::divide(
        profile,
        member,
        &Integer::from(a),
        &Integer::from(b),
        &IntegerDomain::Mathematical,
        &mut runtime_meter,
    );
    (qsl_outcome, qsl_meter, runtime_outcome, runtime_meter)
}

#[trace("TC-097", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn integer_division_profile_rows_match_runtime() {
    let source_rows: Vec<_> = quire_specification::vectors_of(quire_specification::TestCase::Tc192)
        .map(|vector| vector.id)
        .collect();
    assert!(source_rows.contains(&"DIV-01"));
    assert!(source_rows.contains(&"DIV-04"));
    for profile in DivisionProfile::ALL {
        for member in [DivisionMember::Quotient, DivisionMember::Remainder] {
            let (qsl, qsl_meter, runtime, runtime_meter) = checked_division(profile, member, 7, 0);
            assert_eq!(
                format!("{qsl:?}"),
                format!("{runtime:?}"),
                "DIV-01 {profile:?} {member:?}"
            );
            assert_eq!(
                &qsl_meter.admitted_charges()[1..],
                runtime_meter.admitted_charges(),
                "DIV-01 {profile:?} {member:?}"
            );
        }
        for (member, expected) in [
            (
                DivisionMember::Quotient,
                Integer::from(9_223_372_036_854_775_808_u64),
            ),
            (DivisionMember::Remainder, Integer::from(0_i64)),
        ] {
            let (qsl, qsl_meter, runtime, runtime_meter) =
                checked_division(profile, member, i64::MIN, -1);
            let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
                panic!("DIV-04 QSL did not complete: {qsl:?}");
            };
            assert_eq!(qsl_value, expected, "DIV-04 {profile:?} {member:?}");
            assert_eq!(
                runtime,
                Outcome::Completed(expected),
                "DIV-04 {profile:?} {member:?}"
            );
            assert_eq!(
                &qsl_meter.admitted_charges()[1..],
                runtime_meter.admitted_charges()
            );
        }
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn division_result_retain_denial_matches_runtime() {
    let limits = ScalarLimits {
        integer_bits: 3,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 2,
        work_units: 4,
        result_units: 1,
    };
    for (member, expected) in [
        (DivisionMember::Quotient, Integer::from(2_i64)),
        (DivisionMember::Remainder, Integer::from(1_i64)),
    ] {
        let qsl_limits = ScalarLimits {
            work_units: 5,
            ..limits
        };
        let (qsl, qsl_meter, runtime, runtime_meter) = checked_division_with_limits(
            DivisionProfile::Truncating,
            member,
            7,
            3,
            qsl_limits,
            limits,
        );
        let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
            panic!("DIV-08 QSL did not complete: {qsl:?}");
        };
        assert_eq!(qsl_value, expected);
        assert_eq!(runtime, Outcome::Completed(expected));
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );

        let denied = ScalarLimits {
            work_units: 3,
            ..limits
        };
        let qsl_denied = ScalarLimits {
            work_units: 4,
            ..limits
        };
        let (qsl, qsl_meter, runtime, runtime_meter) = checked_division_with_limits(
            DivisionProfile::Truncating,
            member,
            7,
            3,
            qsl_denied,
            denied,
        );
        let Outcome::Incomplete(qsl_stop) = qsl else {
            panic!("DIV-08 QSL denial: {qsl:?}");
        };
        let Outcome::Incomplete(runtime_stop) = runtime else {
            panic!("DIV-08 RT denial: {runtime:?}");
        };
        assert_eq!(qsl_stop.limit_kind, runtime_stop.limit_kind);
        assert_eq!(qsl_stop.limit, runtime_stop.limit + 1);
        assert_eq!(qsl_stop.consumed, runtime_stop.consumed + 1);
        assert_eq!(qsl_stop.next_charge, runtime_stop.next_charge);
        assert_eq!(qsl_stop.charge_point, runtime_stop.charge_point);
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulus_charge_prefix_and_denials_match_runtime() {
    let base = ScalarLimits {
        integer_bits: 3,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 2,
        work_units: 4,
        result_units: 1,
    };
    for (work, expected_point) in [
        (0, quire_exact::ChargePoint::IntegerModulusOperands),
        (1, quire_exact::ChargePoint::IntegerModulusArithmetic),
        (2, quire_exact::ChargePoint::IntegerModulusDomain),
        (3, quire_exact::ChargePoint::IntegerModulusResultRetain),
    ] {
        let runtime_limits = ScalarLimits {
            work_units: work,
            ..base
        };
        let qsl_limits = ScalarLimits {
            work_units: work + 1,
            ..base
        };
        let (qsl, qsl_meter, runtime, runtime_meter) = checked_modulo(
            -7,
            3,
            qsl_limits,
            runtime_limits,
            IntegerDomain::Mathematical,
        );
        let Outcome::Incomplete(qsl_stop) = qsl else {
            panic!("DIV-10 QSL denial: {qsl:?}");
        };
        let Outcome::Incomplete(runtime_stop) = runtime else {
            panic!("DIV-10 RT denial: {runtime:?}");
        };
        assert_eq!(qsl_stop.charge_point, expected_point);
        assert_eq!(runtime_stop.charge_point, expected_point);
        assert_eq!(qsl_stop.limit_kind, LimitKind::WorkUnits);
        assert_eq!(runtime_stop.limit_kind, LimitKind::WorkUnits);
        assert_eq!(qsl_stop.limit, runtime_stop.limit + 1);
        assert_eq!(qsl_stop.consumed, runtime_stop.consumed + 1);
        assert_eq!(qsl_stop.next_charge, runtime_stop.next_charge);
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );
    }
    let (qsl, qsl_meter, runtime, runtime_meter) = checked_modulo(
        -7,
        3,
        ScalarLimits {
            work_units: 5,
            ..base
        },
        base,
        IntegerDomain::Mathematical,
    );
    let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
        panic!("DIV-10 QSL completion: {qsl:?}");
    };
    assert_eq!(qsl_value, Integer::from(2_i64));
    assert_eq!(runtime, Outcome::Completed(qsl_value));
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn zero_divisor_stops_after_operand_charge() {
    let base = ScalarLimits {
        integer_bits: 3,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 2,
        work_units: 1,
        result_units: 1,
    };
    for work in [0, 1] {
        let runtime_limits = ScalarLimits {
            work_units: work,
            ..base
        };
        let qsl_limits = ScalarLimits {
            work_units: work + 1,
            ..base
        };
        let (qsl_div, qsl_div_meter, rt_div, rt_div_meter) = checked_division_with_limits(
            DivisionProfile::Truncating,
            DivisionMember::Quotient,
            7,
            0,
            qsl_limits,
            runtime_limits,
        );
        let (qsl_mod, qsl_mod_meter, rt_mod, rt_mod_meter) = checked_modulo(
            7,
            0,
            qsl_limits,
            runtime_limits,
            IntegerDomain::Mathematical,
        );
        for (qsl, qsl_meter, rt, rt_meter) in [
            (qsl_div, qsl_div_meter, rt_div, rt_div_meter),
            (qsl_mod, qsl_mod_meter, rt_mod, rt_mod_meter),
        ] {
            if work == 1 {
                assert_eq!(format!("{qsl:?}"), format!("{rt:?}"));
                assert!(matches!(rt, Outcome::Undefined(_)));
            } else {
                let Outcome::Incomplete(qsl_stop) = qsl else {
                    panic!("DIV-11 QSL denial: {qsl:?}");
                };
                let Outcome::Incomplete(rt_stop) = rt else {
                    panic!("DIV-11 RT denial: {rt:?}");
                };
                assert_eq!(qsl_stop.charge_point, rt_stop.charge_point);
                assert_eq!(qsl_stop.limit_kind, rt_stop.limit_kind);
                assert_eq!(qsl_stop.next_charge, rt_stop.next_charge);
                assert_eq!(qsl_stop.limit, rt_stop.limit + 1);
                assert_eq!(qsl_stop.consumed, rt_stop.consumed + 1);
            }
            assert_eq!(
                &qsl_meter.admitted_charges()[1..],
                rt_meter.admitted_charges()
            );
        }
    }
}
