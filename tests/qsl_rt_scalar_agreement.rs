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

fn assert_scalar_counters(qsl: &Meter, runtime: &Meter) {
    assert_eq!(
        qsl.admitted_charges()[0].point,
        quire_exact::ChargePoint::FunctionCall
    );
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl.consumed(counter),
            runtime.consumed(counter) + call_work,
            "{counter:?}"
        );
    }
}

fn assert_qspec_row(id: &str) {
    let rows: Vec<_> = quire_specification::vectors_of(quire_specification::TestCase::Tc192)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-192 must offer exactly one {id} row; found {}",
            rows.len()
        );
    };
    assert!(!row.inputs.is_empty(), "{id} needs authored inputs");
    assert!(!row.expected.is_empty(), "{id} needs authored expectation");
}

fn checked_package(source: &str, profile: Option<DivisionProfile>) -> CheckedPackage {
    let mut declarations = assembled_source(source, "quire.value.complete/v1");
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

fn assembled_source(source: &str, facet: &str) -> PackageDeclarations {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"{facet}\";\n{source}"
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
    PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the source unit")
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulo_through_qsl_matches_runtime() {
    assert_qspec_row("DIV-02");
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
    assert_qspec_row("DIV-01");
    assert_qspec_row("DIV-04");
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
            assert_scalar_counters(&qsl_meter, &runtime_meter);
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
            assert_scalar_counters(&qsl_meter, &runtime_meter);
        }
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn division_result_retain_denial_matches_runtime() {
    assert_qspec_row("DIV-08");
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
        assert_scalar_counters(&qsl_meter, &runtime_meter);

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
        assert_scalar_counters(&qsl_meter, &runtime_meter);
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulus_charge_prefix_and_denials_match_runtime() {
    assert_qspec_row("DIV-10");
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
        assert_scalar_counters(&qsl_meter, &runtime_meter);
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
    assert_scalar_counters(&qsl_meter, &runtime_meter);
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn zero_divisor_stops_after_operand_charge() {
    assert_qspec_row("DIV-11");
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
            assert_scalar_counters(&qsl_meter, &rt_meter);
        }
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn operand_size_denial_precedes_work_denial() {
    assert_qspec_row("DIV-13");
    let limits = ScalarLimits {
        integer_bits: 2,
        work_units: 0,
        ..LIMITS
    };
    for member in [DivisionMember::Quotient, DivisionMember::Remainder] {
        let (qsl, qsl_meter, runtime, runtime_meter) = checked_division_with_limits(
            DivisionProfile::Truncating,
            member,
            7,
            3,
            ScalarLimits {
                work_units: 1,
                ..limits
            },
            limits,
        );
        let Outcome::Incomplete(qsl_stop) = qsl else {
            panic!("DIV-13 QSL: {qsl:?}");
        };
        let Outcome::Incomplete(runtime_stop) = runtime else {
            panic!("DIV-13 RT: {runtime:?}");
        };
        assert_eq!(qsl_stop.limit_kind, LimitKind::IntegerBits);
        assert_eq!(qsl_stop, runtime_stop);
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );
        assert_scalar_counters(&qsl_meter, &runtime_meter);
    }
}

#[trace("TC-097", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn modulus_does_not_follow_selected_division_law() {
    assert_qspec_row("DIV-02");
    for profile in DivisionProfile::ALL {
        let package = checked_package(
            "function op using v(a: Integer, b: Integer): Integer pure { a mod b }",
            Some(profile),
        );
        let mut qsl_meter = Meter::new(LIMITS);
        let qsl = package
            .call(
                &QualifiedName::unqualified("op").expect("function name"),
                vec![
                    Value::Integer(Integer::from(-7_i64)),
                    Value::Integer(Integer::from(3_i64)),
                ],
                &ObjectEnvironment::default(),
                &mut qsl_meter,
            )
            .expect("checked mod evaluates");
        let FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value))) = qsl.outcome
        else {
            panic!("DIV-02 QSL result under {profile:?}: {:?}", qsl.outcome);
        };
        let mut rt_meter = Meter::new(LIMITS);
        let rt = quire_eval::modulo(
            &Integer::from(-7_i64),
            &Integer::from(3_i64),
            &IntegerDomain::Mathematical,
            &mut rt_meter,
        );
        assert_eq!(rt, Outcome::Completed(value.clone()));
        assert_eq!(value, Integer::from(2_i64));
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            rt_meter.admitted_charges()
        );
        assert_scalar_counters(&qsl_meter, &rt_meter);
    }
}

#[trace("TC-097", "IT-017-SC-04")]
#[test]
fn integer_division_definition_admission_precedes_evaluation() {
    assert_qspec_row("DIV-03");
    assert_qspec_row("DIV-09");
    use qsl_semantics::value::{admit_integer_division, DefinitionRole};
    use quire_semantic_value::definition::{PackageCause, PackageRefusal};

    let truncating = DefinitionRole::IntegerDivisionTruncating.reference();
    let floor = DefinitionRole::IntegerDivisionFloor.reference();
    let euclidean = DefinitionRole::IntegerDivisionEuclidean.reference();
    assert_eq!(
        admit_integer_division(&[], None).unwrap_err(),
        PackageRefusal::invalid_package(PackageCause::MissingMember)
    );
    assert_eq!(
        admit_integer_division(&[truncating.clone(), floor], None).unwrap_err(),
        PackageRefusal::invalid_package(PackageCause::ConflictingDefinition)
    );
    assert_eq!(
        admit_integer_division(&[truncating.clone()], Some(&truncating)).unwrap_err(),
        PackageRefusal::invalid_package(PackageCause::IncompatibleDefinition)
    );
    assert!(admit_integer_division(&[truncating], Some(&euclidean)).is_ok());
}

#[trace("TC-097", "IT-017-SC-02")]
#[test]
fn division_source_precedence_matches_selected_law() {
    assert_qspec_row("DIV-17");
    let package = checked_package(
        "function op using v(): Integer pure { 1 + 7 div 3 * 2 }",
        Some(DivisionProfile::Truncating),
    );
    let evaluation = package
        .call(
            &QualifiedName::unqualified("op").expect("function name"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut Meter::new(LIMITS),
        )
        .expect("complete-facet source evaluates");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value))) = evaluation.outcome
    else {
        panic!("DIV-17 complete-facet result: {:?}", evaluation.outcome);
    };
    assert_eq!(value, Integer::from(5_i64));

    let (qsl, qsl_meter, runtime, runtime_meter) =
        checked_division(DivisionProfile::Truncating, DivisionMember::Remainder, 7, 3);
    let Outcome::Completed(Value::Integer(remainder)) = qsl else {
        panic!("DIV-17 rem result: {qsl:?}");
    };
    assert_eq!(remainder, Integer::from(1_i64));
    assert_eq!(runtime, Outcome::Completed(remainder));
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    assert_scalar_counters(&qsl_meter, &runtime_meter);

    let literal = checked_package("function op using v(): Integer pure { -7 mod 3 }", None);
    let literal_outcome = literal
        .call(
            &QualifiedName::unqualified("op").expect("function name"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut Meter::new(LIMITS),
        )
        .expect("literal mod evaluates");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(modulus))) =
        literal_outcome.outcome
    else {
        panic!("DIV-17 literal mod result: {:?}", literal_outcome.outcome);
    };
    assert_eq!(modulus, Integer::from(2_i64));
}

#[trace("TC-097", "IT-017-SC-04")]
#[test]
fn integer_operator_source_spans_survive_form_building() {
    assert_qspec_row("DIV-17");
    for expression in ["7 rem 3", "-7 mod 3"] {
        let source = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n\
             function op using v(): Integer pure {{ {expression} }}"
        );
        let parsed = qsl_cst::parse(
            qsl_foundation::SourceIdentity::new("agent-ix", "ir-669-spans", "git", "1"),
            "scalar.native",
            source.as_bytes(),
            qsl_cst::Limits::default(),
        )
        .expect("operator source parses");
        assert!(parsed.is_admissible());
        let unit = qsl_forms::build_unit(&parsed).expect("operator forms build");
        let qsl_forms::DeclarationForm::Function(function) = unit.forms()[0].form() else {
            panic!("operator is a function body");
        };
        let spans = &function.spans().expect("source spans").body;
        let start = source.find(expression).expect("source expression");
        assert_eq!(
            spans.at(&[]),
            Some(qsl_foundation::Span {
                start,
                end: start + expression.len()
            })
        );
    }
}

#[trace("TC-097", "IT-017-SC-02")]
#[test]
fn rational_slash_never_becomes_integer_division() {
    assert_qspec_row("DIV-18");
    let package = checked_package(
        "function op using v(): Rational[0,7;1,3] pure { 7 / 3 }",
        None,
    );
    let mut qsl_meter = Meter::new(LIMITS);
    let evaluation = package
        .call(
            &QualifiedName::unqualified("op").expect("function name"),
            Vec::new(),
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("checked rational slash evaluates");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Rational(qsl_value))) =
        evaluation.outcome
    else {
        panic!("DIV-18 rational slash result: {:?}", evaluation.outcome);
    };
    let seven = quire_exact::Rational::from_integer(Integer::from(7_i64));
    let three = quire_exact::Rational::from_integer(Integer::from(3_i64));
    let mut runtime_meter = Meter::new(LIMITS);
    let runtime = quire_exact::evaluate_rational_arithmetic(
        quire_exact::RationalArithmetic::Divide(&seven, &three),
        None,
        &mut runtime_meter,
    );
    assert_eq!(runtime, Outcome::Completed(qsl_value.clone()));
    assert_eq!(
        qsl_value,
        quire_exact::Rational::new(Integer::from(7_i64), Integer::from(3_i64))
            .expect("nonzero denominator")
    );
    assert!(qsl_meter
        .admitted_charges()
        .iter()
        .any(|charge| charge.point == quire_exact::ChargePoint::RationalArithmeticOperands));
    assert_eq!(
        &qsl_meter.admitted_charges()[1..],
        runtime_meter.admitted_charges()
    );
    assert_scalar_counters(&qsl_meter, &runtime_meter);
}

fn independent_division(profile: DivisionProfile, a: i64, b: i64) -> (i64, i64) {
    match profile {
        DivisionProfile::Truncating => (a / b, a % b),
        DivisionProfile::Floor => {
            let quotient = a / b;
            let remainder = a % b;
            if remainder != 0 && (remainder > 0) != (b > 0) {
                (quotient - 1, remainder + b)
            } else {
                (quotient, remainder)
            }
        }
        DivisionProfile::Euclidean => (a.div_euclid(b), a.rem_euclid(b)),
    }
}

#[trace("TC-097", "IT-017-SC-03")]
#[test]
fn generated_division_sweep_has_independent_integer_oracle() {
    for a in [-7_i64, -3, -1, 0, 1, 3, 7] {
        for b in [-7_i64, -3, -1, 1, 3, 7] {
            for profile in DivisionProfile::ALL {
                let (quotient, remainder) = independent_division(profile, a, b);
                for (member, expected) in [
                    (DivisionMember::Quotient, quotient),
                    (DivisionMember::Remainder, remainder),
                ] {
                    let (qsl, qsl_meter, runtime, runtime_meter) =
                        checked_division(profile, member, a, b);
                    let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
                        panic!("sweep QSL {a} {profile:?} {member:?} {b}: {qsl:?}");
                    };
                    assert_eq!(qsl_value, Integer::from(expected));
                    assert_eq!(runtime, Outcome::Completed(qsl_value));
                    assert_eq!(
                        &qsl_meter.admitted_charges()[1..],
                        runtime_meter.admitted_charges()
                    );
                    assert_scalar_counters(&qsl_meter, &runtime_meter);
                }
            }
            let (qsl, qsl_meter, runtime, runtime_meter) =
                checked_modulo(a, b, LIMITS, LIMITS, IntegerDomain::Mathematical);
            let Outcome::Completed(Value::Integer(qsl_value)) = qsl else {
                panic!("sweep QSL {a} mod {b}: {qsl:?}");
            };
            assert_eq!(qsl_value, Integer::from(a.rem_euclid(b)));
            assert_eq!(runtime, Outcome::Completed(qsl_value));
            assert_eq!(
                &qsl_meter.admitted_charges()[1..],
                runtime_meter.admitted_charges()
            );
            assert_scalar_counters(&qsl_meter, &runtime_meter);
        }
    }
}

#[trace("TC-097", "IT-017-SC-03")]
#[test]
fn integer_arithmetic_size_charge_is_exact_and_one_under() {
    for bits in [2, 3] {
        let limits = ScalarLimits {
            integer_bits: bits,
            ..LIMITS
        };
        let cases = [
            checked_division_with_limits(
                DivisionProfile::Truncating,
                DivisionMember::Quotient,
                7,
                3,
                limits,
                limits,
            ),
            checked_modulo(7, 3, limits, limits, IntegerDomain::Mathematical),
        ];
        for (qsl, qsl_meter, runtime, runtime_meter) in cases {
            if bits == 2 {
                let Outcome::Incomplete(qsl_stop) = qsl else {
                    panic!("QSL one-under: {qsl:?}");
                };
                let Outcome::Incomplete(runtime_stop) = runtime else {
                    panic!("RT one-under: {runtime:?}");
                };
                assert_eq!(qsl_stop, runtime_stop);
                assert_eq!(runtime_stop.limit_kind, LimitKind::IntegerBits);
            } else {
                assert!(matches!(qsl, Outcome::Completed(Value::Integer(_))));
                assert!(matches!(runtime, Outcome::Completed(_)));
                let arithmetic = runtime_meter
                    .admitted_charges()
                    .get(1)
                    .expect("arithmetic charge");
                assert_eq!(arithmetic.integer_bits, Some(3));
            }
            assert_eq!(
                &qsl_meter.admitted_charges()[1..],
                runtime_meter.admitted_charges()
            );
            assert_scalar_counters(&qsl_meter, &runtime_meter);
        }
    }
}

#[trace("TC-097", "IT-017-SC-03")]
#[test]
fn arithmetic_charge_injection_stops_before_result_allocation() {
    use std::num::NonZeroU64;
    for (operator, point) in [
        ("div", quire_exact::ChargePoint::IntegerDivisionArithmetic),
        ("mod", quire_exact::ChargePoint::IntegerModulusArithmetic),
    ] {
        let source = format!(
            "function op using v(a: Integer, b: Integer): Integer pure {{ a {operator} b }}"
        );
        let profile = (operator == "div").then_some(DivisionProfile::Truncating);
        let package = checked_package(&source, profile);
        let denial = quire_exact::InjectedDenial {
            point,
            occurrence: NonZeroU64::new(1).expect("one occurrence"),
        };
        let mut qsl_meter = Meter::new(LIMITS).with_injected_denial(denial);
        let evaluation = package
            .call(
                &QualifiedName::unqualified("op").expect("function name"),
                vec![
                    Value::Integer(Integer::from(7_i64)),
                    Value::Integer(Integer::from(3_i64)),
                ],
                &ObjectEnvironment::default(),
                &mut qsl_meter,
            )
            .expect("injected QSL operation evaluates to incomplete");
        let FamilyOutcome::Evaluated(Outcome::Incomplete(qsl_stop)) = evaluation.outcome else {
            panic!("injected QSL {operator}: {:?}", evaluation.outcome);
        };
        let mut runtime_meter = Meter::new(LIMITS).with_injected_denial(denial);
        let runtime = if operator == "div" {
            quire_exact::divide(
                DivisionProfile::Truncating,
                DivisionMember::Quotient,
                &Integer::from(7_i64),
                &Integer::from(3_i64),
                &IntegerDomain::Mathematical,
                &mut runtime_meter,
            )
        } else {
            quire_eval::modulo(
                &Integer::from(7_i64),
                &Integer::from(3_i64),
                &IntegerDomain::Mathematical,
                &mut runtime_meter,
            )
        };
        let Outcome::Incomplete(runtime_stop) = runtime else {
            panic!("injected RT {operator}: {runtime:?}");
        };
        assert_eq!(qsl_stop.charge_point, point);
        assert_eq!(qsl_stop.limit_kind, runtime_stop.limit_kind);
        assert_eq!(qsl_stop.next_charge, runtime_stop.next_charge);
        assert_eq!(qsl_stop.limit, runtime_stop.limit + 1);
        assert_eq!(qsl_stop.consumed, runtime_stop.consumed + 1);
        assert_eq!(
            &qsl_meter.admitted_charges()[1..],
            runtime_meter.admitted_charges()
        );
        assert_scalar_counters(&qsl_meter, &runtime_meter);
    }
}
