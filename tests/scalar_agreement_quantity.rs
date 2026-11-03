// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-187 quantity agreement through QSL admission and the public value runtime.

use ix_trace_rs::trace;
use qsl_eval::value::{evaluate_closed, CheckedPackageEvaluation, QualifiedName};
use qsl_package::CheckedPackage;
use qsl_semantics::check::{CheckCause, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_eval::FamilyOutcome;
use quire_exact::{
    ChargePoint, ComparisonOperator, IllTypedCause, InjectedDenial, Integer, LimitKind, Meter,
    Outcome, Quantity, Rational, Refusal, RoundingMode, ScalarLimits, Value, ValueType,
};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::loss::ValueLoss;
use quire_semantic_value::quantity::{
    compare_quantity, convert_quantity, evaluate_quantity, evaluate_quantity_unit, ConvertedValue,
    QuantityOperation, QuantityTarget, UnitTable,
};
use quire_specification::{vectors_of, TestCase};

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

const ZERO_LIMITS: ScalarLimits = ScalarLimits {
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

const U10_LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 8,
    decimal_digits: 0,
    scale_expansion: 0,
    text_input_bytes: 0,
    text_scalars: 0,
    normalized_scalars: 0,
    unit_edges: 1,
    value_occurrences: 1,
    work_units: 6,
    result_units: 1,
};

const U19_LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 4,
    decimal_digits: 0,
    scale_expansion: 0,
    text_input_bytes: 0,
    text_scalars: 0,
    normalized_scalars: 0,
    unit_edges: 0,
    value_occurrences: 2,
    work_units: 5,
    result_units: 1,
};

const U15_LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 15,
    decimal_digits: 0,
    scale_expansion: 0,
    text_input_bytes: 0,
    text_scalars: 0,
    normalized_scalars: 0,
    unit_edges: 2,
    value_occurrences: 1,
    work_units: 9,
    result_units: 1,
};

const U23_LIMITS: ScalarLimits = ScalarLimits {
    integer_bits: 15,
    decimal_digits: 0,
    scale_expansion: 0,
    text_input_bytes: 0,
    text_scalars: 0,
    normalized_scalars: 0,
    unit_edges: 2,
    value_occurrences: 2,
    work_units: 9,
    result_units: 1,
};

const UNITS: &str = r#"
dimension Length;
unit m : Length = rational(1,1);
unit cm : Length = rational(1,100) * m;
unit inch : Length = rational(127,5000) * m;
dimension Area = Length ^ 2;
unit m2 : Area = rational(1,1);
unit cm2 : Area = rational(1,10000) * m2;
dimension Torque = Length * Length;
unit N_m : Torque = rational(1,1);
dimension Energy = Length ^ 2;
unit J : Energy = rational(1,1);
dimension Time;
unit s : Time = rational(1,1);
dimension Temperature;
unit K : Temperature = rational(1,1);
unit degC : Temperature = rational(1,1) * K + rational(5463,20);
unit degF : Temperature = rational(5,9) * K + rational(45967,180);
unit rev : Length = rational(-1,1) * m;
function touch_m using v(x: Quantity<m>): Quantity<m> pure { x }
function touch_cm using v(x: Quantity<cm>): Quantity<cm> pure { x }
function touch_inch using v(x: Quantity<inch>): Quantity<inch> pure { x }
function touch_m2 using v(x: Quantity<m2>): Quantity<m2> pure { x }
function touch_cm2 using v(x: Quantity<cm2>): Quantity<cm2> pure { x }
function touch_N_m using v(x: Quantity<N_m>): Quantity<N_m> pure { x }
function touch_J using v(x: Quantity<J>): Quantity<J> pure { x }
function touch_s using v(x: Quantity<s>): Quantity<s> pure { x }
function touch_K using v(x: Quantity<K>): Quantity<K> pure { x }
function touch_degC using v(x: Quantity<degC>): Quantity<degC> pure { x }
function touch_degF using v(x: Quantity<degF>): Quantity<degF> pure { x }
function touch_rev using v(x: Quantity<rev>): Quantity<rev> pure { x }
"#;

struct CheckedQuantityFixture {
    package: CheckedPackage,
    units: UnitTable,
}

fn checked_fixture(body: &str) -> CheckedQuantityFixture {
    checked_fixture_as("ir-669-quantity", body)
}

fn checked_fixture_as(identity: &str, body: &str) -> CheckedQuantityFixture {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{UNITS}\n{body}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", identity, "git", "1"),
        "quantity.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses TC-187 source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds TC-187 source");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        identity.to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles TC-187 declarations");
    let units = UnitTable::declared(&declarations.units);
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("QSL checks TC-187 source"),
    );
    CheckedQuantityFixture { package, units }
}

fn qsl_check_refuses(body: &str) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{UNITS}\n{body}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669-quantity-refusal", "git", "1"),
        "refusal.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses the refusal source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds the refusal source");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669-quantity-refusal".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the refusal declarations");
    let refusals = declarations
        .check(CheckingLimits::default())
        .expect_err("the quantity source must refuse at S4");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.cause == CheckCause::IllTyped(IllTypedCause::TypeMismatch)),
        "the quantity source must refuse as ill-typed: {refusals:?}"
    );
}

fn qsl_check_accepts(declarations: &str) -> bool {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669-checking", "git", "1"),
        "checking.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses the checking source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds the checking source");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669-checking".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL assembles the checking declarations");
    declarations.check(CheckingLimits::default()).is_ok()
}

fn qsl_assembly_codes(declarations: &str) -> Vec<String> {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669-topology", "git", "1"),
        "topology.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses the topology source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds the topology source");
    let refusal = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669-topology".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect_err("the topology must be refused before value evaluation");
    refusal
        .errors
        .iter()
        .map(|error| error.cause.catalog_code().to_string())
        .collect()
}

fn qsl_assembly_admits(declarations: &str) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "ir-669-admitted", "git", "1"),
        "admitted.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("QSL parses the admitted source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("QSL builds the admitted source");
    PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir-669-admitted".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("QSL admits the source topology");
}

fn unit_id(fixture: &CheckedQuantityFixture, name: &str) -> quire_exact::UnitId {
    let signature = fixture
        .package
        .graph()
        .callable(&format!("touch_{name}"))
        .expect("QSL checked the unit parameter");
    let [(_, ValueType::Quantity(id))] = signature.parameters else {
        panic!("{name} must have one quantity parameter")
    };
    *id
}

fn quantity(
    fixture: &CheckedQuantityFixture,
    name: &str,
    numerator: i64,
    denominator: i64,
) -> Quantity {
    Quantity::new(
        Rational::new(Integer::from(numerator), Integer::from(denominator))
            .expect("nonzero denominator"),
        unit_id(fixture, name),
    )
}

fn qsl_quantity(fixture: &CheckedQuantityFixture, function: &str) -> Quantity {
    let identity = fixture
        .package
        .graph()
        .function_identity(function)
        .expect("QSL checks the function");
    let state = fixture
        .package
        .graph()
        .function_by_identity(identity)
        .expect("QSL retains the function");
    let result = evaluate_closed(
        &fixture.package,
        state.body,
        state.slots,
        &mut Meter::new(LIMITS),
    )
    .expect("QSL evaluates the checked quantity expression");
    let FamilyOutcome::Evaluated(Outcome::Completed(Value::Quantity(value))) = result.outcome
    else {
        panic!(
            "{function} must complete as a quantity: {:?}",
            result.outcome
        );
    };
    value
}

fn qsl_value_call(
    fixture: &CheckedQuantityFixture,
    function: &str,
    arguments: Vec<Quantity>,
) -> (Outcome<Value>, Meter) {
    let mut meter = Meter::new(LIMITS);
    let result = fixture
        .package
        .call(
            &QualifiedName::unqualified(function).expect("QSL function name"),
            arguments.into_iter().map(Value::Quantity).collect(),
            &ObjectEnvironment::default(),
            &mut meter,
        )
        .expect("checked quantity function call");
    let FamilyOutcome::Evaluated(outcome) = result.outcome else {
        panic!("{function} must reach QSL evaluation: {:?}", result.outcome)
    };
    (outcome, meter)
}

fn assert_call_charge_suffix(qsl: &Meter, runtime: &Meter) {
    let [call, rest @ ..] = qsl.admitted_charges() else {
        panic!("QSL must admit the function call charge")
    };
    assert_eq!(call.point, ChargePoint::FunctionCall);
    assert_eq!(rest, runtime.admitted_charges());
    for counter in LimitKind::ALL {
        let call_work = u64::from(counter == LimitKind::WorkUnits);
        assert_eq!(
            qsl.consumed(counter),
            runtime.consumed(counter) + call_work,
            "{counter:?}"
        );
    }
}

fn selected_quantity_row(id: &str) {
    // Branch-gating QSpec ID comparison: a selected row must exist exactly once.
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|vector| vector.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!(
            "QSpec TC-187 must contain exactly one {id} row; found {}",
            rows.len()
        );
    };
    assert!(!row.inputs.is_empty(), "{id} has no authored inputs");
    assert!(!row.expected.is_empty(), "{id} has no authored expectation");
}

fn carried_conversion_agreement(
    fixture: &CheckedQuantityFixture,
    function: &str,
    source: &Quantity,
    runtime_limits: ScalarLimits,
) -> (Outcome<Value>, Vec<ValueLoss>, Meter) {
    let signature = fixture
        .package
        .graph()
        .callable(function)
        .expect("QSL checked the carried conversion function");
    let ValueType::CarriedQuantity(target) = signature.result else {
        panic!("{function} must have a checked carried quantity result type")
    };
    let target_unit = fixture
        .units
        .get(target.unit().clone())
        .expect("QSL admitted the target unit");
    let source_unit = fixture
        .units
        .resolve(source)
        .expect("QSL admitted the source unit");
    let mut runtime_meter = Meter::new(runtime_limits);
    let runtime = convert_quantity(
        source_unit,
        target_unit,
        &QuantityTarget::Carried(target.carrier().clone()),
        &mut runtime_meter,
    )
    .expect("the checked source has a compatible quantity conversion");
    let (runtime, runtime_losses) = match runtime {
        Outcome::Completed(conversion) => {
            let (magnitude, loss) = match conversion.value() {
                ConvertedValue::Exact(value) => (value.clone(), None),
                ConvertedValue::Decimal(result) => (
                    result.value().normalized().to_rational(),
                    result.loss().cloned(),
                ),
                ConvertedValue::Integer { value, loss } => {
                    (Rational::from_integer(value.value().clone()), loss.clone())
                }
            };
            (
                Outcome::Completed(Value::Quantity(Quantity::new(
                    magnitude,
                    target.unit().clone(),
                ))),
                loss.into_iter().map(ValueLoss::Decimal).collect(),
            )
        }
        Outcome::Undefined(reason) => (Outcome::Undefined(reason), Vec::new()),
        Outcome::Refused(reason) => (Outcome::Refused(reason), Vec::new()),
        Outcome::Incomplete(reason) => (Outcome::Incomplete(reason), Vec::new()),
    };

    // A checked call adds exactly one function.call work unit before the
    // scalar conversion. The scalar charge suffix is otherwise identical.
    let qsl_limits = ScalarLimits {
        work_units: runtime_limits
            .work_units
            .checked_add(1)
            .expect("the QSL call needs one additional work unit"),
        ..runtime_limits
    };
    let mut qsl_meter = Meter::new(qsl_limits);
    let result = fixture
        .package
        .call(
            &QualifiedName::unqualified(function).expect("QSL checked the function name"),
            vec![Value::Quantity(source.clone())],
            &ObjectEnvironment::default(),
            &mut qsl_meter,
        )
        .expect("the checked carried conversion call runs");
    let FamilyOutcome::Evaluated(qsl) = result.outcome else {
        panic!(
            "{function} must reach quantity evaluation: {:?}",
            result.outcome
        )
    };
    let qsl_losses: Vec<_> = result.losses.into_iter().map(|loss| loss.loss).collect();
    match (&qsl, &runtime) {
        (Outcome::Incomplete(qsl_stop), Outcome::Incomplete(runtime_stop)) => {
            assert_eq!(
                qsl_stop.limit_kind, runtime_stop.limit_kind,
                "{function} limit kind"
            );
            let call_work = u64::from(runtime_stop.limit_kind == LimitKind::WorkUnits);
            assert_eq!(
                qsl_stop.limit,
                runtime_stop.limit + call_work,
                "{function} limit"
            );
            assert_eq!(
                qsl_stop.consumed,
                runtime_stop.consumed + call_work,
                "{function} consumed"
            );
            assert_eq!(qsl_stop.next_charge, runtime_stop.next_charge);
            assert_eq!(qsl_stop.charge_point, runtime_stop.charge_point);
        }
        _ => assert_eq!(qsl, runtime, "{function} QSL/runtime outcome"),
    }
    assert_eq!(qsl_losses, runtime_losses, "{function} QSL/runtime losses");
    assert_call_charge_suffix(&qsl_meter, &runtime_meter);
    (runtime, runtime_losses, runtime_meter)
}

const CARRIED_FUNCTIONS: &str = r#"
function u07_exact using v(x: Quantity<inch>): Quantity<m, Decimal[-100, 100; 2, 2; exact]> pure { convert<Quantity<m, Decimal[-100, 100; 2, 2; exact]>>(x) }
function u07_nearest using v(x: Quantity<inch>): Quantity<m, Decimal[-100, 100; 2, 2; nearest-even]> pure { convert<Quantity<m, Decimal[-100, 100; 2, 2; nearest-even]>>(x) }
function u08_exact using v(x: Quantity<m>): Quantity<m, Decimal[-2, 2; 0, 0; exact]> pure { convert<Quantity<m, Decimal[-2, 2; 0, 0; exact]>>(x) }
function u18_large_scale using v(x: Quantity<m>): Quantity<m, Decimal[0, 1; 0, 4294967295; exact]> pure { convert<Quantity<m, Decimal[0, 1; 0, 4294967295; exact]>>(x) }
function u29_exact using v(x: Quantity<inch>): Quantity<m, Int[-2, 2; exact]> pure { convert<Quantity<m, Int[-2, 2; exact]>>(x) }
function u29_nearest using v(x: Quantity<inch>): Quantity<m, Int[-2, 2; nearest-even]> pure { convert<Quantity<m, Int[-2, 2; nearest-even]>>(x) }
function u29_outside using v(x: Quantity<m>): Quantity<m, Int[-2, 2; exact]> pure { convert<Quantity<m, Int[-2, 2; exact]>>(x) }
"#;

fn carried_fixture() -> CheckedQuantityFixture {
    checked_fixture(CARRIED_FUNCTIONS)
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn qspec_u07_u08_carried_conversion_values_and_refusals_agree() {
    for id in ["U07", "U08"] {
        selected_quantity_row(id);
    }
    let fixture = carried_fixture();
    let inch = quantity(&fixture, "inch", 1, 1);
    let (exact, losses, _) = carried_conversion_agreement(&fixture, "u07_exact", &inch, LIMITS);
    assert!(matches!(
        exact,
        Outcome::Refused(Refusal::InexactConversion { .. })
    ));
    assert!(losses.is_empty(), "U07 exact refusal has no loss record");

    let (rounded, losses, _) = carried_conversion_agreement(&fixture, "u07_nearest", &inch, LIMITS);
    let Outcome::Completed(Value::Quantity(rounded)) = rounded else {
        panic!("U07 nearest-even must complete with a quantity: {rounded:?}")
    };
    assert_eq!(
        rounded.magnitude(),
        &Rational::new(3_i64.into(), 100_i64.into()).unwrap()
    );
    let [ValueLoss::Decimal(loss)] = losses.as_slice() else {
        panic!("U07 nearest-even must retain one decimal loss: {losses:?}")
    };
    assert_eq!(loss.exact_numerator(), &Integer::from(127_i64));
    assert_eq!(loss.exact_denominator(), Integer::from(5000_i64));
    assert_eq!(loss.rounded_coefficient(), &Integer::from(3_i64));
    assert_eq!(loss.rounded_scale(), 2);
    assert_eq!(loss.mode(), RoundingMode::NearestEven);

    for value in [-2_i64, 2, -3, 3] {
        let source = quantity(&fixture, "m", value, 1);
        let (result, losses, _) =
            carried_conversion_agreement(&fixture, "u08_exact", &source, LIMITS);
        assert!(losses.is_empty(), "U08 exact conversion has no loss");
        if (-2..=2).contains(&value) {
            assert_eq!(result, Outcome::Completed(Value::Quantity(source)));
        } else {
            assert!(matches!(
                result,
                Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
            ));
        }
    }
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn qspec_u16_u17_u18_u29_carried_limits_and_stop_points_agree() {
    for id in ["U16", "U17", "U18", "U29"] {
        selected_quantity_row(id);
    }
    let fixture = carried_fixture();
    let inch = quantity(&fixture, "inch", 1, 1);
    let u16 = ScalarLimits {
        integer_bits: 15,
        decimal_digits: 5,
        unit_edges: 1,
        value_occurrences: 1,
        work_units: 6,
        result_units: 1,
        ..ZERO_LIMITS
    };
    let (exact, _, exact_meter) = carried_conversion_agreement(&fixture, "u07_exact", &inch, u16);
    assert!(matches!(
        exact,
        Outcome::Refused(Refusal::InexactConversion { .. })
    ));
    assert_eq!(exact_meter.consumed(LimitKind::WorkUnits), 4);
    let (rounded, _, _) = carried_conversion_agreement(&fixture, "u07_nearest", &inch, u16);
    assert!(matches!(rounded, Outcome::Completed(Value::Quantity(_))));
    let short_u16 = ScalarLimits {
        work_units: 4,
        ..u16
    };
    let (exact, _, _) = carried_conversion_agreement(&fixture, "u07_exact", &inch, short_u16);
    assert!(matches!(
        exact,
        Outcome::Refused(Refusal::InexactConversion { .. })
    ));
    let (rounded, _, _) = carried_conversion_agreement(&fixture, "u07_nearest", &inch, short_u16);
    let Outcome::Incomplete(stop) = rounded else {
        panic!("U16 nearest-even must stop at target-domain: {rounded:?}")
    };
    assert_eq!(
        (stop.limit_kind, stop.limit, stop.consumed),
        (LimitKind::WorkUnits, 4, 4)
    );
    assert_eq!(stop.next_charge, Integer::from(1_i64));
    assert_eq!(stop.charge_point, ChargePoint::UnitTargetDomain);

    let three = quantity(&fixture, "m", 3, 1);
    let u17 = ScalarLimits {
        integer_bits: 2,
        decimal_digits: 1,
        value_occurrences: 1,
        work_units: 2,
        ..ZERO_LIMITS
    };
    let (outside, _, _) = carried_conversion_agreement(&fixture, "u08_exact", &three, u17);
    assert!(matches!(
        outside,
        Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
    ));
    let (denied, _, _) = carried_conversion_agreement(
        &fixture,
        "u08_exact",
        &three,
        ScalarLimits {
            work_units: 1,
            ..u17
        },
    );
    let Outcome::Incomplete(stop) = denied else {
        panic!("U17 must stop at target-domain: {denied:?}")
    };
    assert_eq!(
        (stop.limit_kind, stop.limit, stop.consumed),
        (LimitKind::WorkUnits, 1, 1)
    );
    assert_eq!(stop.next_charge, Integer::from(1_i64));
    assert_eq!(stop.charge_point, ChargePoint::UnitTargetDomain);

    let one = quantity(&fixture, "m", 1, 1);
    let u18 = ScalarLimits {
        integer_bits: u64::MAX,
        decimal_digits: 64,
        value_occurrences: 1,
        work_units: 2,
        result_units: 1,
        ..ZERO_LIMITS
    };
    let (denied, _, _) = carried_conversion_agreement(&fixture, "u18_large_scale", &one, u18);
    let Outcome::Incomplete(stop) = denied else {
        panic!("U18 must stop before materializing the coefficient: {denied:?}")
    };
    assert_eq!(
        (stop.limit_kind, stop.limit, stop.consumed),
        (LimitKind::DecimalDigits, 64, 0)
    );
    assert_eq!(stop.next_charge, Integer::from(4_294_967_296_i64));
    assert_eq!(stop.charge_point, ChargePoint::UnitTargetDomain);

    let u29 = ScalarLimits {
        integer_bits: 15,
        unit_edges: 1,
        value_occurrences: 1,
        work_units: 4,
        result_units: 1,
        ..ZERO_LIMITS
    };
    let (exact, _, _) = carried_conversion_agreement(&fixture, "u29_exact", &inch, u29);
    assert!(matches!(
        exact,
        Outcome::Refused(Refusal::InexactConversion { .. })
    ));
    let (rounded, _, _) = carried_conversion_agreement(&fixture, "u29_nearest", &inch, u29);
    let Outcome::Incomplete(stop) = rounded else {
        panic!("U29 nearest-even must stop at target-domain: {rounded:?}")
    };
    assert_eq!(
        (stop.limit_kind, stop.limit, stop.consumed),
        (LimitKind::WorkUnits, 4, 4)
    );
    assert_eq!(stop.next_charge, Integer::from(1_i64));
    assert_eq!(stop.charge_point, ChargePoint::UnitTargetDomain);
    let outside_limits = ScalarLimits {
        integer_bits: 2,
        value_occurrences: 1,
        work_units: 2,
        ..ZERO_LIMITS
    };
    let (outside, _, _) =
        carried_conversion_agreement(&fixture, "u29_outside", &three, outside_limits);
    assert!(matches!(
        outside,
        Outcome::Refused(Refusal::IntegerOutOfDomain { .. })
    ));
    let (denied, _, _) = carried_conversion_agreement(
        &fixture,
        "u29_outside",
        &three,
        ScalarLimits {
            work_units: 1,
            ..outside_limits
        },
    );
    let Outcome::Incomplete(stop) = denied else {
        panic!("U29 outside bound must stop at target-domain: {denied:?}")
    };
    assert_eq!(
        (stop.limit_kind, stop.limit, stop.consumed),
        (LimitKind::WorkUnits, 1, 1)
    );
    assert_eq!(stop.next_charge, Integer::from(1_i64));
    assert_eq!(stop.charge_point, ChargePoint::UnitTargetDomain);
}

fn exact_conversion(fixture: &CheckedQuantityFixture, source: &Quantity, target: &str) -> Quantity {
    let target_id = unit_id(fixture, target);
    let target_unit = fixture.units.get(target_id).expect("admitted target unit");
    let source_unit = fixture.units.resolve(source).expect("admitted source unit");
    let result = convert_quantity(
        source_unit,
        target_unit,
        &QuantityTarget::Exact,
        &mut Meter::new(LIMITS),
    )
    .expect("compatible dimensions");
    let Outcome::Completed(converted) = result else {
        panic!("exact conversion must complete: {result:?}")
    };
    let ConvertedValue::Exact(value) = converted.value() else {
        panic!("exact target must return a rational")
    };
    Quantity::new(value.clone(), target_id)
}

fn completed_operation(operation: QuantityOperation<'_>) -> Quantity {
    let result =
        evaluate_quantity(operation, &mut Meter::new(LIMITS)).expect("typed quantity operation");
    let Outcome::Completed(value) = result else {
        panic!("quantity operation must complete: {result:?}")
    };
    value
}

fn assert_ill_typed_without_charge(operation: QuantityOperation<'_>, limits: ScalarLimits) {
    let mut meter = Meter::new(limits);
    assert!(evaluate_quantity(operation, &mut meter).is_err());
    assert!(meter.admitted_charges().is_empty());
}

#[trace("TC-187", "IT-017-SC-02", "IT-017-SC-03")]
#[test]
fn qspec_quantity_examples_agree_with_checked_qsl_and_runtime() {
    let fixture = checked_fixture(
        r#"
function u04_c using v(): Quantity<K> pure { convert<Quantity<K>>(0 degC) }
function u04_f using v(): Quantity<K> pure { convert<Quantity<K>>(32 degF) }
function u10 using v(): Quantity<m> pure { convert<Quantity<m>>(100 cm) }
function u10_param using v(x: Quantity<cm>): Quantity<m> pure { convert<Quantity<m>>(x) }
function u15_param using v(x: Quantity<inch>): Quantity<cm> pure { convert<Quantity<cm>>(x) }
function u19 using v(): Quantity<m> pure { 2 m + 3 m }
function u19_param using v(a: Quantity<m>, b: Quantity<m>): Quantity<m> pure { a + b }
function u22_m2 using v(): Quantity<m2> pure { convert<Quantity<m2>>(4 m * 1 m) }
function u22_cm2 using v(): Quantity<cm2> pure { convert<Quantity<cm2>>(4 m * 1 m) }
function u23_less using v(a: Quantity<degC>, b: Quantity<degC>): Boolean pure { a < b }
function u24 using v(): Quantity<cm> pure { 1 cm + 2 cm }
function u24_param using v(a: Quantity<cm>, b: Quantity<cm>): Quantity<cm> pure { a + b }
"#,
    );
    // Branch-gating QSpec IDs: this family slice names each executable row explicitly.
    for vector in vectors_of(TestCase::Tc187).filter(|vector| {
        matches!(
            vector.id,
            "U04" | "U05" | "U09b" | "U10" | "U14" | "U15" | "U19" | "U22" | "U23" | "U24"
        )
    }) {
        // Branch-gating QSpec ID dispatch: an unexpected selected row fails.
        match vector.id {
            "U04" => {
                let c = exact_conversion(&fixture, &quantity(&fixture, "degC", 0, 1), "K");
                let f = exact_conversion(&fixture, &quantity(&fixture, "degF", 32, 1), "K");
                assert_eq!(qsl_quantity(&fixture, "u04_c"), c);
                assert_eq!(qsl_quantity(&fixture, "u04_f"), f);
                assert_eq!(c, f);
                assert_eq!(
                    c.magnitude(),
                    &Rational::new(5463_i64.into(), 20_i64.into()).unwrap()
                );
            }
            "U05" => {
                qsl_check_refuses(
                    "function bad using v(): Quantity<degC> pure { 1 degC + 2 degC }",
                );
                let a = quantity(&fixture, "degC", 1, 1);
                let b = quantity(&fixture, "degC", 2, 1);
                let f = quantity(&fixture, "degF", 1, 1);
                let one = Integer::from(1_i64);
                let a = fixture.units.resolve(&a).unwrap();
                let b = fixture.units.resolve(&b).unwrap();
                let f = fixture.units.resolve(&f).unwrap();
                for operation in [
                    QuantityOperation::Add(a, b),
                    QuantityOperation::Subtract(a, b),
                    QuantityOperation::Multiply(a, b),
                    QuantityOperation::Divide(a, b),
                    QuantityOperation::Power(f, &one),
                ] {
                    assert_ill_typed_without_charge(operation, LIMITS);
                }
            }
            "U09b" => {
                let distinct = checked_fixture(
                    "unit m_alt : Length = rational(1,1) * m;\n\
                     function touch_m_alt using v(x: Quantity<m_alt>): Quantity<m_alt> pure { x }\n\
                     function u09b using v(): Quantity<m> pure { convert<Quantity<m>>(1 m_alt) }",
                );
                let source = quantity(&distinct, "m_alt", 1, 1);
                assert_ne!(source.unit(), unit_id(&distinct, "m"));
                assert_eq!(
                    qsl_quantity(&distinct, "u09b"),
                    exact_conversion(&distinct, &source, "m")
                );
            }
            "U10" => {
                let source = quantity(&fixture, "cm", 100, 1);
                let target = fixture.units.get(unit_id(&fixture, "m")).unwrap();
                let mut exact_meter = Meter::new(U10_LIMITS);
                let exact = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut exact_meter,
                )
                .unwrap();
                assert!(matches!(exact, Outcome::Completed(_)));
                assert_eq!(exact_meter.consumed(LimitKind::IntegerBits), 8);
                let mut denied_meter =
                    Meter::new(U10_LIMITS).with_injected_denial(InjectedDenial {
                        point: ChargePoint::UnitResultRetain,
                        occurrence: 1,
                    });
                let denied = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut denied_meter,
                )
                .unwrap();
                let Outcome::Incomplete(incomplete) = denied else {
                    panic!("U10 denied final retention must be incomplete: {denied:?}")
                };
                assert_eq!(incomplete.charge_point, ChargePoint::UnitResultRetain);
                let (qsl_result, qsl_meter) =
                    qsl_value_call(&fixture, "u10_param", vec![source.clone()]);
                let mut runtime_meter = Meter::new(LIMITS);
                let runtime_result = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut runtime_meter,
                )
                .unwrap();
                let Outcome::Completed(runtime_conversion) = runtime_result else {
                    panic!("U10 unbounded runtime conversion must complete")
                };
                let ConvertedValue::Exact(runtime_value) = runtime_conversion.value() else {
                    panic!("U10 exact conversion must return a rational")
                };
                assert_eq!(
                    qsl_result,
                    Outcome::Completed(Value::Quantity(Quantity::new(
                        runtime_value.clone(),
                        unit_id(&fixture, "m"),
                    )))
                );
                assert_call_charge_suffix(&qsl_meter, &runtime_meter);
                assert_eq!(
                    qsl_quantity(&fixture, "u10"),
                    exact_conversion(&fixture, &source, "m")
                );
            }
            "U14" => {
                qsl_check_refuses("function bad using v(): Quantity<m> pure { 1 m + 1 s }");
                let zero = ScalarLimits {
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
                let m = quantity(&fixture, "m", 1, 1);
                let s = quantity(&fixture, "s", 1, 1);
                let c = quantity(&fixture, "degC", 1, 1);
                let zero_c = quantity(&fixture, "degC", 0, 1);
                let cm = quantity(&fixture, "cm", 1, 1);
                let m = fixture.units.resolve(&m).unwrap();
                let s = fixture.units.resolve(&s).unwrap();
                let c = fixture.units.resolve(&c).unwrap();
                let zero_c = fixture.units.resolve(&zero_c).unwrap();
                let cm = fixture.units.resolve(&cm).unwrap();
                for operation in [
                    QuantityOperation::Add(m, s),
                    QuantityOperation::Add(c, s),
                    QuantityOperation::Add(c, c),
                    QuantityOperation::Divide(c, zero_c),
                    QuantityOperation::Add(cm, m),
                ] {
                    assert_ill_typed_without_charge(operation, zero);
                }
            }
            "U15" => {
                let source = quantity(&fixture, "inch", 1, 1);
                let target = fixture.units.get(unit_id(&fixture, "cm")).unwrap();
                let mut exact_meter = Meter::new(U15_LIMITS);
                let exact = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut exact_meter,
                )
                .unwrap();
                let Outcome::Completed(conversion) = exact else {
                    panic!("U15 two-edge conversion must complete")
                };
                assert_eq!(
                    conversion.value(),
                    &ConvertedValue::Exact(Rational::new(127_i64.into(), 50_i64.into()).unwrap())
                );
                assert_eq!(exact_meter.consumed(LimitKind::IntegerBits), 15);
                assert_eq!(exact_meter.consumed(LimitKind::UnitEdges), 2);
                let mut short_meter = Meter::new(ScalarLimits {
                    integer_bits: 7,
                    unit_edges: 1,
                    ..U15_LIMITS
                });
                let Outcome::Incomplete(incomplete) = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut short_meter,
                )
                .unwrap() else {
                    panic!("U15 one-edge bound must stop before arithmetic")
                };
                assert_eq!(incomplete.limit_kind, LimitKind::UnitEdges);
                assert_eq!(incomplete.consumed, 1);
                assert_eq!(incomplete.next_charge, Integer::from(2_i64));
                assert_eq!(incomplete.charge_point, ChargePoint::UnitEdge);
                let (qsl_result, qsl_meter) =
                    qsl_value_call(&fixture, "u15_param", vec![source.clone()]);
                let mut runtime_meter = Meter::new(LIMITS);
                let runtime_result = convert_quantity(
                    fixture.units.resolve(&source).unwrap(),
                    target,
                    &QuantityTarget::Exact,
                    &mut runtime_meter,
                )
                .unwrap();
                let Outcome::Completed(runtime_conversion) = runtime_result else {
                    panic!("U15 unbounded conversion must complete")
                };
                let ConvertedValue::Exact(runtime_value) = runtime_conversion.value() else {
                    panic!("U15 exact conversion must return a rational")
                };
                assert_eq!(
                    qsl_result,
                    Outcome::Completed(Value::Quantity(Quantity::new(
                        runtime_value.clone(),
                        unit_id(&fixture, "cm"),
                    )))
                );
                assert_call_charge_suffix(&qsl_meter, &runtime_meter);
            }
            "U19" => {
                let a = quantity(&fixture, "m", 2, 1);
                let b = quantity(&fixture, "m", 3, 1);
                let result = completed_operation(QuantityOperation::Add(
                    fixture.units.resolve(&a).unwrap(),
                    fixture.units.resolve(&b).unwrap(),
                ));
                assert_eq!(qsl_quantity(&fixture, "u19"), result);
                assert_eq!(
                    result.magnitude(),
                    &Rational::new(5_i64.into(), 1_i64.into()).unwrap()
                );
                let operation = QuantityOperation::Add(
                    fixture.units.resolve(&a).unwrap(),
                    fixture.units.resolve(&b).unwrap(),
                );
                let (qsl_result, qsl_meter) =
                    qsl_value_call(&fixture, "u19_param", vec![a.clone(), b.clone()]);
                let mut runtime_meter = Meter::new(LIMITS);
                let runtime_result = evaluate_quantity(operation, &mut runtime_meter).unwrap();
                let Outcome::Completed(runtime_value) = runtime_result else {
                    panic!("U19 unbounded runtime addition must complete")
                };
                assert_eq!(
                    qsl_result,
                    Outcome::Completed(Value::Quantity(runtime_value))
                );
                assert_call_charge_suffix(&qsl_meter, &runtime_meter);
                let mut exact_meter = Meter::new(U19_LIMITS);
                assert!(matches!(
                    evaluate_quantity(operation, &mut exact_meter).unwrap(),
                    Outcome::Completed(_)
                ));
                assert_eq!(exact_meter.consumed(LimitKind::IntegerBits), 4);
                let mut short_meter = Meter::new(ScalarLimits {
                    work_units: 4,
                    ..U19_LIMITS
                });
                let Outcome::Incomplete(incomplete) =
                    evaluate_quantity(operation, &mut short_meter).unwrap()
                else {
                    panic!("U19 must deny final retention under four work units")
                };
                assert_eq!(incomplete.limit_kind, LimitKind::WorkUnits);
                assert_eq!(incomplete.consumed, 4);
                assert_eq!(incomplete.next_charge, Integer::from(1_i64));
                assert_eq!(incomplete.charge_point, ChargePoint::UnitResultRetain);
            }
            "U22" => {
                qsl_check_refuses(
                    "function bad using v(): Quantity<J> pure { convert<Quantity<J>>(1 N_m) }",
                );
                let torque = quantity(&fixture, "N_m", 1, 1);
                let joule = fixture.units.get(unit_id(&fixture, "J")).unwrap();
                let mut zero_meter = Meter::new(ScalarLimits {
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
                assert!(convert_quantity(
                    fixture.units.resolve(&torque).unwrap(),
                    joule,
                    &QuantityTarget::Exact,
                    &mut zero_meter,
                )
                .is_err());
                assert!(zero_meter.admitted_charges().is_empty());

                let four_m = quantity(&fixture, "m", 4, 1);
                let one_m = quantity(&fixture, "m", 1, 1);
                let (compound_outcome, compound_unit) = evaluate_quantity_unit(
                    QuantityOperation::Multiply(
                        fixture.units.resolve(&four_m).unwrap(),
                        fixture.units.resolve(&one_m).unwrap(),
                    ),
                    &mut Meter::new(LIMITS),
                )
                .unwrap();
                let Outcome::Completed(compound) = compound_outcome else {
                    panic!("U22 compound source must complete")
                };
                let mut units = fixture.units.clone();
                assert_eq!(
                    units.insert(compound_unit.unit().clone()).unwrap(),
                    compound.unit()
                );
                let m2_id = unit_id(&fixture, "m2");
                let cm2_id = unit_id(&fixture, "cm2");
                let mut m2_meter = Meter::new(ScalarLimits {
                    integer_bits: 3,
                    decimal_digits: 0,
                    scale_expansion: 0,
                    text_input_bytes: 0,
                    text_scalars: 0,
                    normalized_scalars: 0,
                    unit_edges: 0,
                    value_occurrences: 1,
                    work_units: 3,
                    result_units: 1,
                });
                let m2 = convert_quantity(
                    units.resolve(&compound).unwrap(),
                    units.get(m2_id).unwrap(),
                    &QuantityTarget::Exact,
                    &mut m2_meter,
                )
                .unwrap();
                let Outcome::Completed(m2_conversion) = m2 else {
                    panic!("U22 m2 conversion must complete")
                };
                assert_eq!(
                    m2_conversion.value(),
                    &ConvertedValue::Exact(Rational::new(4_i64.into(), 1_i64.into()).unwrap())
                );
                assert_eq!(
                    qsl_quantity(&fixture, "u22_m2"),
                    Quantity::new(Rational::new(4_i64.into(), 1_i64.into()).unwrap(), m2_id)
                );
                let cm2_limits = ScalarLimits {
                    integer_bits: 17,
                    decimal_digits: 0,
                    scale_expansion: 0,
                    text_input_bytes: 0,
                    text_scalars: 0,
                    normalized_scalars: 0,
                    unit_edges: 1,
                    value_occurrences: 1,
                    work_units: 6,
                    result_units: 1,
                };
                let cm2 = convert_quantity(
                    units.resolve(&compound).unwrap(),
                    units.get(cm2_id).unwrap(),
                    &QuantityTarget::Exact,
                    &mut Meter::new(cm2_limits),
                )
                .unwrap();
                let Outcome::Completed(cm2_conversion) = cm2 else {
                    panic!("U22 cm2 conversion must complete")
                };
                assert_eq!(
                    cm2_conversion.value(),
                    &ConvertedValue::Exact(Rational::new(40000_i64.into(), 1_i64.into()).unwrap())
                );
                assert_eq!(
                    qsl_quantity(&fixture, "u22_cm2"),
                    Quantity::new(
                        Rational::new(40000_i64.into(), 1_i64.into()).unwrap(),
                        cm2_id
                    )
                );
                let mut short_meter = Meter::new(ScalarLimits {
                    integer_bits: 16,
                    ..cm2_limits
                });
                let Outcome::Incomplete(incomplete) = convert_quantity(
                    units.resolve(&compound).unwrap(),
                    units.get(cm2_id).unwrap(),
                    &QuantityTarget::Exact,
                    &mut short_meter,
                )
                .unwrap() else {
                    panic!("U22 bit ceiling must stop before cm2 retention")
                };
                assert_eq!(incomplete.limit_kind, LimitKind::IntegerBits);
                assert_eq!(incomplete.consumed, 5);
                assert_eq!(incomplete.next_charge, Integer::from(17_i64));
                assert_eq!(incomplete.charge_point, ChargePoint::UnitRationalArithmetic);
            }
            "U23" => {
                let c1 = quantity(&fixture, "degC", 1, 1);
                let c2 = quantity(&fixture, "degC", 2, 1);
                let mut meter = Meter::new(U23_LIMITS);
                let outcome = compare_quantity(
                    ComparisonOperator::Less,
                    fixture.units.resolve(&c1).unwrap(),
                    fixture.units.resolve(&c2).unwrap(),
                    &mut meter,
                )
                .unwrap();
                assert_eq!(outcome, Outcome::Completed(true));
                let mut short_meter = Meter::new(ScalarLimits {
                    work_units: 8,
                    ..U23_LIMITS
                });
                let Outcome::Incomplete(incomplete) = compare_quantity(
                    ComparisonOperator::Less,
                    fixture.units.resolve(&c1).unwrap(),
                    fixture.units.resolve(&c2).unwrap(),
                    &mut short_meter,
                )
                .unwrap() else {
                    panic!("U23 retention must stop at eight work units")
                };
                assert_eq!(incomplete.charge_point, ChargePoint::UnitResultRetain);
                assert_eq!(incomplete.consumed, 8);
                let (qsl_result, qsl_meter) =
                    qsl_value_call(&fixture, "u23_less", vec![c1.clone(), c2.clone()]);
                let mut runtime_meter = Meter::new(LIMITS);
                let runtime_result = compare_quantity(
                    ComparisonOperator::Less,
                    fixture.units.resolve(&c1).unwrap(),
                    fixture.units.resolve(&c2).unwrap(),
                    &mut runtime_meter,
                )
                .unwrap();
                let Outcome::Completed(runtime_value) = runtime_result else {
                    panic!("U23 unbounded comparison must complete")
                };
                assert_eq!(
                    qsl_result,
                    Outcome::Completed(Value::Boolean(runtime_value))
                );
                assert_call_charge_suffix(&qsl_meter, &runtime_meter);
                let rev1 = quantity(&fixture, "rev", 1, 1);
                let rev2 = quantity(&fixture, "rev", 2, 1);
                assert_eq!(
                    compare_quantity(
                        ComparisonOperator::Less,
                        fixture.units.resolve(&rev1).unwrap(),
                        fixture.units.resolve(&rev2).unwrap(),
                        &mut Meter::new(LIMITS),
                    )
                    .unwrap(),
                    Outcome::Completed(false)
                );
                assert_eq!(
                    compare_quantity(
                        ComparisonOperator::Equal,
                        fixture.units.resolve(&c1).unwrap(),
                        fixture.units.resolve(&c1).unwrap(),
                        &mut Meter::new(LIMITS),
                    )
                    .unwrap(),
                    Outcome::Completed(true)
                );
                qsl_check_refuses("function bad using v(): Boolean pure { 0 degC < 32 degF }");
                let c0 = quantity(&fixture, "degC", 0, 1);
                let f32 = quantity(&fixture, "degF", 32, 1);
                for operator in [ComparisonOperator::Less, ComparisonOperator::Equal] {
                    let mut zero_meter = Meter::new(ScalarLimits {
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
                    assert!(compare_quantity(
                        operator,
                        fixture.units.resolve(&c0).unwrap(),
                        fixture.units.resolve(&f32).unwrap(),
                        &mut zero_meter,
                    )
                    .is_err());
                    assert!(zero_meter.admitted_charges().is_empty());
                }
            }
            "U24" => {
                let a = quantity(&fixture, "cm", 1, 1);
                let b = quantity(&fixture, "cm", 2, 1);
                let operation = QuantityOperation::Add(
                    fixture.units.resolve(&a).unwrap(),
                    fixture.units.resolve(&b).unwrap(),
                );
                let mut meter = Meter::new(U19_LIMITS);
                let Outcome::Completed(result) = evaluate_quantity(operation, &mut meter).unwrap()
                else {
                    panic!("U24 same-unit addition must complete")
                };
                assert_eq!(qsl_quantity(&fixture, "u24"), result);
                assert_eq!(
                    result.magnitude(),
                    &Rational::new(3_i64.into(), 1_i64.into()).unwrap()
                );
                assert!(meter
                    .admitted_charges()
                    .iter()
                    .all(|charge| charge.point != ChargePoint::UnitEdge));
                let (qsl_result, qsl_meter) =
                    qsl_value_call(&fixture, "u24_param", vec![a.clone(), b.clone()]);
                let mut runtime_meter = Meter::new(LIMITS);
                let runtime_result = evaluate_quantity(operation, &mut runtime_meter).unwrap();
                let Outcome::Completed(runtime_value) = runtime_result else {
                    panic!("U24 unbounded runtime addition must complete")
                };
                assert_eq!(
                    qsl_result,
                    Outcome::Completed(Value::Quantity(runtime_value))
                );
                assert_call_charge_suffix(&qsl_meter, &runtime_meter);
            }
            other => panic!("unknown selected TC-187 quantity vector: {other}"),
        }
    }
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn qspec_admission_only_quantity_sources_refuse_at_assembly() {
    // Branch-gating QSpec IDs: each selected row has an S3 admission observation.
    for vector in vectors_of(TestCase::Tc187).filter(|vector| {
        matches!(
            vector.id,
            "U06"
                | "U11a"
                | "U11b"
                | "U11c"
                | "U11d"
                | "U11e"
                | "U11f"
                | "U11g1"
                | "U11g2"
                | "U11g4"
                | "U11i"
                | "U11j"
                | "U11k"
                | "U11l"
                | "U11n"
                | "U11o"
                | "U11p"
                | "U11w"
        )
    }) {
        // Branch-gating QSpec ID dispatch: unknown selected rows fail.
        let (source, expected) = match vector.id {
            "U06" => (
                "dimension L; unit m : L = rational(1,1); unit zero : L = rational(0,1) * m;",
                "invalid_package/unit-graph-topology",
            ),
            "U11a" => (
                "dimension L; unit m : L = rational(1,1); unit m_alt : L = rational(1,1);",
                "invalid_package/unit-graph-topology",
            ),
            "U11b" => (
                "dimension L; unit m : L = rational(1,1) * cm; unit cm : L = rational(1,100) * m;",
                "invalid_package/definition-cycle",
            ),
            "U11c" => (
                "dimension L; unit m : L = rational(1,1); unit x : L = rational(1,1) * nothing;",
                "missing_declaration/missing-name",
            ),
            "U11d" => (
                "dimension L; dimension T; unit m : L = rational(1,1); unit s : T = rational(1,1); unit cm : L = rational(1,100) * s;",
                "invalid_package/unit-graph-topology",
            ),
            "U11e" => (
                "dimension L; unit m : L = rational(2,1);",
                "invalid_package/unit-graph-topology",
            ),
            "U11f" => (
                "dimension L; unit m : L = rational(1,1); unit u_self : L = rational(1,1) * u_self;",
                "invalid_package/definition-cycle",
            ),
            "U11g1" => (
                "dimension L; dimension Area = L ^ 2; unit m2 : Area = rational(1,1);",
                "missing_declaration/missing-name",
            ),
            "U11g2" => (
                "dimension L; dimension Area = L ^ 2; dimension Vol = Area * L; unit m3 : Vol = rational(1,1);",
                "missing_declaration/missing-name",
            ),
            "U11g4" => (
                "dimension L; dimension Area = L ^ 2; dimension Vol = Area * Area; unit m4 : Vol = rational(1,1);",
                "missing_declaration/missing-name",
            ),
            "U11i" => (
                "dimension L; unit m : L = rational(1,1); unit a : L = rational(1,1) * nothing; unit b : L = rational(0,1) * m;",
                "missing_declaration/missing-name",
            ),
            "U11j" => (
                "dimension L; unit m : L = rational(1,1); unit m_alt : L = rational(1,1); unit u_self : L = rational(1,1) * u_self;",
                "invalid_package/unit-graph-topology",
            ),
            "U11k" => (
                "dimension L; unit u_self : L = rational(1,1) * u_self; dimension T; unit z : T = rational(0,1);",
                "invalid_package/definition-cycle",
            ),
            "U11l" => (
                "dimension L; unit cm : L = rational(1,100) * m; unit m : L = rational(1,1) * cm;",
                "invalid_package/definition-cycle",
            ),
            "U11n" => (
                "dimension A = B; dimension B = A ^ 2; unit a : A = rational(1,1);",
                "invalid_package/definition-cycle",
            ),
            "U11o" => (
                "dimension L; dimension D = D * L; unit m : L = rational(1,1);",
                "invalid_package/definition-cycle",
            ),
            "U11p" => (
                "dimension L; dimension D = L * K; unit m : L = rational(1,1);",
                "missing_declaration/missing-name",
            ),
            "U11w" => (
                "dimension A = B * K; dimension B = A;",
                "missing_declaration/missing-name",
            ),
            other => panic!("unknown selected admission vector: {other}"),
        };
        let codes = qsl_assembly_codes(source);
        // Branch-gating catalog-code comparison: only the selected refusal admits.
        assert!(
            codes.iter().any(|code| code == expected),
            "{} expected {expected}, got {codes:?}",
            vector.id
        );
    }
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn qspec_dimensionless_derived_unit_is_admitted() {
    // Branch-gating QSpec ID comparison: U11g3 must remain authored.
    let vector = vectors_of(TestCase::Tc187)
        .find(|vector| vector.id == "U11g3")
        .expect("QSpec owns U11g3");
    assert_eq!(vector.id, "U11g3");
    qsl_assembly_admits("dimension L; dimension N = L / L; unit one : N = rational(1,1);");
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn qspec_multi_source_topology_cases_retain_their_refusal_causes() {
    // Branch-gating QSpec IDs: each selected row has its own source order.
    for vector in vectors_of(TestCase::Tc187)
        .filter(|vector| matches!(vector.id, "U11e" | "U11m" | "U11x" | "U11z3"))
    {
        // Branch-gating QSpec ID dispatch: unknown selected rows fail.
        let cases: &[(&str, &str)] = match vector.id {
            "U11e" => &[(
                "dimension L; unit m : L = rational(1,1) + rational(1,1);",
                "invalid_package/unit-graph-topology",
            )],
            "U11m" => &[
                (
                    "dimension Area = L ^ 2; dimension L; unit a : Area = rational(1,1) * b; unit b : Area = rational(1,1) * a;",
                    "invalid_package/definition-cycle",
                ),
                (
                    "dimension L; dimension Area = L ^ 2; unit a : Area = rational(1,1) * b; unit b : Area = rational(1,1) * a;",
                    "missing_declaration/missing-name",
                ),
            ],
            "U11x" => &[
                (
                    "dimension L; unit m : L = rational(1,1); dimension E = D; dimension D = L * K;",
                    "missing_declaration/missing-name",
                ),
                (
                    "dimension L; unit m : L = rational(1,1); dimension E = D; unit e : E = rational(0,1); dimension D = L * K;",
                    "invalid_package/unit-graph-topology",
                ),
            ],
            "U11z3" => &[
                (
                    "dimension E = A; unit e : E = rational(0,1); dimension A = B; dimension B = A;",
                    "invalid_package/unit-graph-topology",
                ),
                (
                    "dimension E = A; dimension A = B; dimension B = A;",
                    "invalid_package/definition-cycle",
                ),
            ],
            other => panic!("unknown selected multi-source vector: {other}"),
        };
        for (source, expected) in cases {
            let codes = qsl_assembly_codes(source);
            // Branch-gating catalog-code comparison: only the selected refusal admits.
            assert!(
                codes.iter().any(|code| code == expected),
                "{} expected {expected}, got {codes:?}",
                vector.id
            );
        }
    }
}

#[trace("TC-187", "IT-017-SC-01", "IT-017-SC-04")]
#[test]
fn qspec_quantity_parameter_and_result_types_follow_unit_identity() {
    const TIME: &str =
        "dimension Time; unit s : Time = rational(1,1); unit ms : Time = rational(1,1000) * s;";
    // Branch-gating QSpec IDs: the two typing rows must remain authored.
    for vector in vectors_of(TestCase::Tc187).filter(|vector| matches!(vector.id, "U30" | "U31")) {
        // Branch-gating QSpec ID dispatch: unknown selected rows fail.
        match vector.id {
            "U30" => {
                assert!(qsl_check_accepts(&format!(
                    "{TIME} function f using v(d: Quantity<ms>): Quantity<ms> pure {{ d }} \
                     function good using v(): Quantity<ms> pure {{ f(3 ms) }}"
                )));
                assert!(!qsl_check_accepts(&format!(
                    "{TIME} function f using v(d: Quantity<ms>): Quantity<ms> pure {{ d }} \
                     function bad using v(): Quantity<ms> pure {{ f(3 s) }}"
                )));
                for parameter in ["Quantity<Time>", "Quantity<nothing>", "ms"] {
                    let codes = qsl_assembly_codes(&format!(
                        "{TIME} function bad using v(d: {parameter}): Boolean pure {{ true }}"
                    ));
                    // Branch-gating catalog-code comparison: missing names refuse.
                    assert!(
                        codes
                            .iter()
                            .any(|code| code == "missing_declaration/missing-name"),
                        "U30 {parameter}: {codes:?}"
                    );
                }
            }
            "U31" => {
                assert!(qsl_check_accepts(&format!(
                    "{TIME} function f using v(d: Quantity<ms>): Quantity<ms> pure {{ d }}"
                )));
                assert!(!qsl_check_accepts(&format!(
                    "{TIME} function f using v(d: Quantity<ms>): Quantity<ms> pure \
                     {{ convert<Quantity<s>>(d) }}"
                )));
            }
            other => panic!("unknown selected quantity typing vector: {other}"),
        }
    }
}
