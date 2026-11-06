// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original affine, comparison and compound quantity operation boundaries.
#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;
#[allow(
    dead_code,
    reason = "shared support is also used by other independent quantity targets"
)]
#[path = "support/tc187_p9_source.rs"]
mod source;
use ix_trace_rs::trace;
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2, CheckedTerm};
use quire_eval::FamilyOutcome;
use quire_exact::{LimitKind, ScalarLimits};
const EXTRA:&str="unit inch : Distance = rational(127,5000) * m;
 dimension Area = Distance ^ 2; unit m2 : Area = rational(1,1); unit cm2 : Area = rational(1,10000) * m2;
 dimension Mass; unit kg : Mass = rational(1,1);
 dimension Torque = Mass * Distance ^ 2 / Duration ^ 2; unit N_m : Torque = rational(1,1);
 dimension Energy = Mass * Distance ^ 2 / Duration ^ 2; unit J : Energy = rational(1,1);
 dimension Temperature; unit K : Temperature = rational(1,1);
 unit degC : Temperature = rational(1,1) * K + rational(5463,20);
 unit degF : Temperature = rational(5,9) * K + rational(45967,180);
 unit rev : Temperature = rational(-1,1) * K;
 unit u1 : Temperature = rational(1,1) * K + rational(10,1);
 unit u2 : Temperature = rational(1,1) * u1 + rational(-10,1);
 unit u3 : Temperature = rational(1,1) * degC;";
const FUNCTIONS:&str="
 function offsetCancelled using v(): Quantity<u2> pure { 1 u2 + 2 u2 }
 function celsiusOrder using v(): Boolean pure { 1 degC < 2 degC }
 function reversedOrder using v(): Boolean pure { 1 rev < 2 rev }
 function celsiusEqual using v(): Boolean pure { 1 degC = 1 degC }

 function product using v(): Boolean pure { 1 cm * 1 inch = 1 cm * 1 inch }
 function areaMetres using v(): Quantity<m2> pure { let area = 2 m ^ 2 in convert<Quantity<m2>>(area) }
 function areaCentimetres using v(): Quantity<cm2> pure { let area = 2 m ^ 2 in convert<Quantity<cm2>>(area) }
";
fn limits(i: u64, e: u64, o: u64, w: u64, r: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: i,
        decimal_digits: 0,
        unit_edges: e,
        value_occurrences: o,
        work_units: w,
        result_units: r,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
    }
}
#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-1",
    "QSpec-FR-142-AC-2",
    "QSpec-FR-142-AC-5",
    "QSpec-FR-142-AC-6",
    "QSpec-FR-142-AC-9",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn original_quantity_boundaries_preserve_composed_units_and_adverse_stops() {
    for (id, input, expectation) in [
        ("U20", "1 u2 + 2 u2", "3 u2"),
        ("U22", "4 m^2", "40000"),
        ("U23", "1 degC < 2 degC", "false"),
        ("U25", "kg*m^2*s^-2", "1 J"),
        ("U27", "1 cm / 0 cm", "undefined"),
        ("U28", "1 cm * 1 in", "127/500000"),
    ] {
        source::bind(id, &[input], &[expectation]);
    }
    source::bind(
        "U28",
        &["L(20,0,2,2,11,1)", "L(13,0,2,2,11,1)", "L(19,0,2,2,11,1)"],
        &[
            "consumed: 9",
            "next_charge: 14",
            "consumed: 15",
            "next_charge: 20",
        ],
    );
    let original = source::original_with(EXTRA, FUNCTIONS);
    for (name, child, l) in [
        ("offsetCancelled", false, limits(4, 0, 2, 5, 1)),
        ("celsiusOrder", false, limits(15, 2, 2, 9, 1)),
        ("celsiusOrder", false, limits(15, 2, 2, 8, 1)),
        ("reversedOrder", false, limits(15, 2, 2, 9, 1)),
        ("celsiusEqual", false, limits(15, 2, 2, 9, 1)),
        ("product", true, limits(20, 2, 2, 11, 1)),
        ("product", true, limits(13, 2, 2, 11, 1)),
        ("product", true, limits(19, 2, 2, 11, 1)),
    ] {
        let (evaluation, meter) = source::evaluate(&original, name, child, l);
        check_operation(&original, name, child, l, &evaluation, &meter);
    }
    assert_eq!(
        source::generation(
            &original,
            &[
                ("offsetCancelled", false, limits(4, 0, 2, 5, 1)),
                ("celsiusOrder", false, limits(15, 2, 2, 9, 1)),
                ("celsiusOrder", false, limits(15, 2, 2, 8, 1)),
                ("reversedOrder", false, limits(15, 2, 2, 9, 1)),
                ("celsiusEqual", false, limits(15, 2, 2, 9, 1)),
                ("product", true, limits(20, 2, 2, 11, 1)),
            ],
            &[],
            &["product"]
        ),
        5
    );
}

fn check_operation(
    original: &source::Original,
    name: &str,
    child: bool,
    l: ScalarLimits,
    evaluation: &quire_eval::Evaluation,
    meter: &quire_exact::Meter,
) {
    use quire_exact::{ChargePoint as P, Incomplete, Integer, Outcome, Value};
    let FamilyOutcome::Evaluated(outcome) = &evaluation.outcome else {
        panic!("quantity operation family");
    };
    assert!(evaluation.losses.is_empty());
    let graph = original.package.graph();
    let function = graph
        .function_by_identity(graph.function_identity(name).expect("original identity"))
        .expect("original function");
    let node = if child {
        function.body.children()[0]
    } else {
        function.body
    };
    if name == "offsetCancelled" {
        let Outcome::Completed(Value::Quantity(q)) = outcome else {
            panic!("nonaffine sibling must complete: {outcome:?}");
        };
        assert_eq!(
            q.magnitude(),
            &quire_exact::Rational::from_integer(Integer::from(3_i64))
        );
        assert_eq!(
            q.unit(),
            &original
                .units
                .declared_unit_id(original.keys["u2"].clone())
                .expect("source u2")
        );
        assert!(!original
            .units
            .unit(original.keys["u2"].clone())
            .expect("u2 graph unit")
            .is_affine());
        source::assert_ledger(meter, &[1, 2], true);
        assert!(evaluation.location.is_none());
        return;
    }
    let c = source::charge;
    let right_bits = if name == "celsiusEqual" || name == "product" {
        1
    } else {
        2
    };
    let mut events = vec![
        c(P::UnitIdentityRead, Some(1), None, None, Some(1)),
        c(P::UnitIdentityRead, Some(right_bits), None, None, Some(2)),
        c(P::UnitEdge, None, None, Some(1), None),
        c(P::UnitEdge, None, None, Some(2), None),
    ];
    if name == "product" {
        let amounts: &[u64] = if l.integer_bits == 13 {
            &[8, 9]
        } else if l.integer_bits == 19 {
            &[8, 9, 14, 15]
        } else {
            &[8, 9, 14, 15, 20]
        };
        for bits in amounts {
            events.push(c(P::UnitRationalArithmetic, Some(*bits), None, None, None));
        }
        if l.integer_bits == 20 {
            let Outcome::Completed(Value::Quantity(q)) = outcome else {
                panic!("compound product must complete: {outcome:?}");
            };
            assert_eq!(
                q.magnitude(),
                &quire_exact::Rational::new(Integer::from(127_i64), Integer::from(500_000_i64))
                    .expect("independent product")
            );
            assert_eq!(
                q.unit().compound_terms(),
                Some(&[(original.keys["m"].clone(), Integer::from(2_i64))][..])
            );
            events.extend([
                c(P::UnitTargetDomain, None, None, None, None),
                c(P::UnitResultRetain, None, None, None, Some(1)),
            ]);
            source::accounting(meter, &events, [20, 0, 2, 2, 11, 1]);
            assert!(evaluation.location.is_none());
        } else {
            let Outcome::Incomplete(actual) = outcome else {
                panic!("product size denial: {outcome:?}");
            };
            let (consumed, next, work) = if l.integer_bits == 13 {
                (9, 14_i64, 6)
            } else {
                (15, 20, 8)
            };
            assert_eq!(
                actual,
                &Incomplete {
                    limit_kind: LimitKind::IntegerBits,
                    limit: l.integer_bits,
                    consumed,
                    next_charge: Integer::from(next),
                    charge_point: P::UnitRationalArithmetic
                }
            );
            source::accounting(meter, &events, [consumed, 0, 2, 2, work, 0]);
            assert_eq!(evaluation.location.as_ref(), Some(node.location()));
        }
    } else {
        let amounts: &[u64] = if name == "reversedOrder" {
            &[2, 3, 3, 4]
        } else if name == "celsiusEqual" {
            &[2, 15, 2, 15]
        } else {
            &[2, 15, 3, 15]
        };
        for bits in amounts {
            events.push(c(P::UnitRationalArithmetic, Some(*bits), None, None, None));
        }
        if l.work_units == 8 {
            let Outcome::Incomplete(actual) = outcome else {
                panic!("comparison retention stop: {outcome:?}");
            };
            assert_eq!(
                actual,
                &Incomplete {
                    limit_kind: LimitKind::WorkUnits,
                    limit: 8,
                    consumed: 8,
                    next_charge: Integer::from(1_i64),
                    charge_point: P::UnitResultRetain
                }
            );
            source::accounting(meter, &events, [15, 0, 2, 2, 8, 0]);
            assert_eq!(evaluation.location.as_ref(), Some(node.location()));
        } else {
            let Outcome::Completed(Value::Boolean(actual)) = outcome else {
                panic!("comparison Boolean: {outcome:?}");
            };
            assert_eq!(*actual, name != "reversedOrder");
            events.push(c(P::UnitResultRetain, None, None, None, Some(1)));
            source::accounting(
                meter,
                &events,
                [if name == "reversedOrder" { 4 } else { 15 }, 0, 2, 2, 9, 1],
            );
            assert!(evaluation.location.is_none());
        }
    }
}

fn refused_source(expression: &str, result: &str, nonzero: bool) -> qsl_forms::Expression {
    use qsl_semantics::check::{CheckCause, Obligation, PackageDeclarations};
    use quire_semantic_value::checking::CheckingLimits;
    let functions = format!("function refused using v(): {result} pure {{ {expression} }}");
    let text = source::source_text(EXTRA, &functions);
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "quantity-operations"),
        "quantity-operations.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("refused original source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let reference = parsed.source().reference().clone();
    let forms = qsl_forms::build_unit(&parsed).expect("refused original source forms");
    let body = forms
        .forms()
        .iter()
        .find_map(|form| match form.form() {
            qsl_forms::DeclarationForm::Function(f) => Some(f.body.clone()),
            _ => None,
        })
        .expect("original refused body");
    let declarations = PackageDeclarations::assemble(
        reference.clone(),
        "agent-ix/quantity-operations".to_owned(),
        forms,
        qsl_semantics::model::intake::SelectedModels::default(),
        Vec::new(),
    )
    .expect("original refusal declarations assemble");
    let regions = declarations.regions();
    let refusals = declarations
        .check(CheckingLimits::default())
        .expect_err("no checked package published");
    let [refusal] = refusals.as_slice() else {
        panic!("one exact refusal: {refusals:?}");
    };
    if !nonzero {
        assert_eq!(
            refusal
                .cause
                .catalog_code()
                .expect("catalog code")
                .to_string(),
            "ill_typed/type-mismatch"
        );
    }
    assert_eq!(
        refusal.cause,
        if nonzero {
            CheckCause::Unproved(Obligation::Nonzero)
        } else {
            CheckCause::IllTyped(quire_exact::IllTypedCause::TypeMismatch)
        }
    );
    let region = regions
        .refusal_region(refusal)
        .expect("original refused source region");
    assert_eq!(region.source(), &reference);
    let start = usize::try_from(region.start()).expect("source start");
    let end = usize::try_from(region.end()).expect("source end");
    assert_eq!(
        &text[start..end],
        if nonzero { "1 cm / 0 cm" } else { expression }
    );
    body
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-2",
    "QSpec-FR-142-AC-5",
    "QSpec-FR-142-AC-6",
    "QSpec-FR-142-AC-9",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn original_static_controls_and_public_zero_division_expression_keep_distinct_axes() {
    use qsl_eval::value::CheckedPackageEvaluation;
    use quire_exact::{
        ChargePoint as P, IllTyped, IllTypedCause, Incomplete, Integer, Meter, Outcome, Quantity,
        Rational, Undefined,
    };
    use quire_semantic_value::checking::{CheckMode, CheckingLimits};
    use quire_semantic_value::quantity::{
        QuantityOperation, QuantityRefusal, QuantityTarget, QuantityUnit, UnitTable,
    };
    source::bind(
        "U20",
        &["1 u1 + 2 u1", "1 u3 * 1 m", "L(0,0,0,0,0,0)"],
        &["ill_typed", "no charge"],
    );
    source::bind(
        "U22",
        &["1 N_m", "J", "L(0,0,0,0,0,0)"],
        &["dimension nodes differ", "no charge"],
    );
    source::bind(
        "U23",
        &["0 degC < 32 degF", "0 degC == 32 degF", "L(0,0,0,0,0,0)"],
        &["distinct-unit", "no charge"],
    );
    source::bind(
        "U27",
        &["1 cm / 0 cm", "L(1,0,0,2,2,0)", "L(1,0,0,2,1,0)"],
        &["undefined", "unit.identity-read", "next_charge: 1"],
    );
    let original = source::original_with(EXTRA, FUNCTIONS);
    for (expression, result) in [
        ("1 u1 + 2 u1", "Quantity<u1>"),
        ("1 u3 * 1 m", "Quantity<m>"),
        ("convert<Quantity<J>>(1 N_m)", "Quantity<J>"),
        ("0 degC < 32 degF", "Boolean"),
        ("0 degC = 32 degF", "Boolean"),
    ] {
        refused_source(expression, result, false);
    }
    let table = UnitTable::declared(&original.units);
    let q = |name, value| {
        Quantity::new(
            Rational::from_integer(Integer::from(value)),
            original
                .units
                .declared_unit_id(original.keys[name].clone())
                .expect("genuine admitted unit"),
        )
    };
    for (left, right, operation, cause) in [
        (
            q("u1", 1_i64),
            q("u1", 2),
            0,
            IllTypedCause::AffineUnitArithmetic,
        ),
        (
            q("u3", 1),
            q("m", 1),
            1,
            IllTypedCause::AffineUnitArithmetic,
        ),
        (q("degC", 0), q("degF", 32), 2, IllTypedCause::DistinctUnits),
        (q("degC", 0), q("degF", 32), 3, IllTypedCause::DistinctUnits),
    ] {
        let mut meter = Meter::new(limits(0, 0, 0, 0, 0));
        let left = table.resolve(&left).expect("original left unit");
        let right = table.resolve(&right).expect("original right unit");
        if operation < 2 {
            let operation = if operation == 0 {
                QuantityOperation::Add(left, right)
            } else {
                QuantityOperation::Multiply(left, right)
            };
            assert_eq!(
                quire_semantic_value::quantity::evaluate_quantity(operation, &mut meter),
                Err(QuantityRefusal::IllTyped(IllTyped { cause }))
            );
        } else {
            let op = if operation == 2 {
                quire_exact::ComparisonOperator::Less
            } else {
                quire_exact::ComparisonOperator::Equal
            };
            assert_eq!(
                quire_semantic_value::quantity::compare_quantity(op, left, right, &mut meter),
                Err(IllTyped { cause })
            );
        }
        source::accounting(&meter, &[], [0; 6]);
    }
    let torque = q("N_m", 1);
    let target = QuantityUnit::Declared(Box::new(
        original
            .units
            .unit(original.keys["J"].clone())
            .expect("source joule")
            .clone(),
    ));
    let mut meter = Meter::new(limits(0, 0, 0, 0, 0));
    assert!(matches!(
        quire_semantic_value::quantity::convert_quantity(
            table.resolve(&torque).expect("torque unit"),
            &target,
            &QuantityTarget::Exact,
            &mut meter
        ),
        Err(IllTyped {
            cause: IllTypedCause::IncompatibleDimensions
        })
    ));
    source::accounting(&meter, &[], [0; 6]);
    let body = refused_source("1 cm / 0 cm = 1 cm / 0 cm", "Boolean", true);
    let children = body.root().children();
    let [left, _] = children.as_slice() else {
        panic!("original equality children");
    };
    let expression = body
        .cut(left.id(), &std::collections::BTreeMap::new())
        .expect("original left divide expression");
    let checked = original
        .package
        .graph()
        .check_expression(
            Vec::new(),
            &expression,
            None,
            CheckMode::Kernel,
            CheckingLimits::default(),
        )
        .expect("public original operation expression checks without total-function envelope");
    for work in [2, 1] {
        let mut meter = Meter::new(limits(1, 0, 2, work, 0));
        let evaluation = original
            .package
            .evaluate(
                &checked,
                Vec::new(),
                &qsl_semantics::model::object_environment::ObjectEnvironment::default(),
                &mut meter,
            )
            .expect("public original expression evaluates");
        assert!(evaluation.losses.is_empty());
        let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
            panic!("value expression family");
        };
        if work == 2 {
            assert!(matches!(
                outcome,
                Outcome::Undefined(Undefined::DivisionByZero)
            ));
        } else {
            let Outcome::Incomplete(actual) = outcome else {
                panic!("second read denial");
            };
            assert_eq!(
                actual,
                Incomplete {
                    limit_kind: LimitKind::WorkUnits,
                    limit: 1,
                    consumed: 1,
                    next_charge: Integer::from(1_i64),
                    charge_point: P::UnitIdentityRead
                }
            );
        }
        let mut events = vec![source::charge(
            P::UnitIdentityRead,
            Some(1),
            None,
            None,
            Some(1),
        )];
        if work == 2 {
            events.push(source::charge(
                P::UnitIdentityRead,
                Some(1),
                None,
                None,
                Some(2),
            ));
        }
        source::accounting(&meter, &events, [1, 0, 0, work, work, 0]);
        assert_eq!(
            evaluation.location.as_ref(),
            Some(checked.root().location())
        );
    }
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-2",
    "QSpec-FR-142-AC-4",
    "QSpec-FR-142-AC-6",
    "QSpec-FR-142-AC-10",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn genuine_compound_values_retain_separate_source_and_owner_conversion_evidence() {
    use quire_exact::{ChargePoint as P, Incomplete, Integer, Meter, Outcome, Quantity, Rational};
    use quire_semantic_value::checking::{CheckMode, CheckingLimits};
    use quire_semantic_value::quantity::{ConvertedValue, QuantityTarget, QuantityUnit, UnitTable};
    let original = source::original_with(EXTRA, FUNCTIONS);
    source::bind(
        "U22",
        &[
            "4 m^2",
            "L(3,0,0,1,3,1)",
            "L(17,0,1,1,6,1)",
            "L(16,0,1,1,6,1)",
        ],
        &["4 m2", "40000 cm2", "consumed: 5", "next_charge: 17"],
    );
    source::bind(
        "U25",
        &["1 N_m", "kg*m^2*s^-2", "L(1,0,0,1,3,1)"],
        &["both explicit steps", "1 J", "no edge"],
    );
    // The entire original let-body admits and evaluates, including its power work.
    for (name, unit, value) in [
        ("areaMetres", "m2", 4_i64),
        ("areaCentimetres", "cm2", 40_000),
    ] {
        let (evaluation, _) = source::evaluate(&original, name, false, limits(128, 8, 8, 64, 4));
        let FamilyOutcome::Evaluated(Outcome::Completed(quire_exact::Value::Quantity(q))) =
            evaluation.outcome
        else {
            panic!(
                "full original compound conversion value: {:?}",
                evaluation.outcome
            );
        };
        assert_eq!(q.magnitude(), &Rational::from_integer(Integer::from(value)));
        assert_eq!(
            q.unit(),
            &original
                .units
                .declared_unit_id(original.keys[unit].clone())
                .expect("declared area target")
        );
        assert!(evaluation.losses.is_empty());
        assert!(evaluation.location.is_none());
    }
    let graph = original.package.graph();
    let function = graph
        .function_by_identity(
            graph
                .function_identity("areaMetres")
                .expect("original let identity"),
        )
        .expect("original let function");
    let children = function.body.children();
    let [power, _conversion] = children.as_slice() else {
        panic!("original let operands");
    };
    let mut preparation = Meter::new(limits(128, 8, 8, 64, 4));
    let evaluated = qsl_eval::value::evaluate_closed(
        &original.package,
        *power,
        function.slots,
        &mut preparation,
    )
    .expect("original power preparation");
    let FamilyOutcome::Evaluated(Outcome::Completed(quire_exact::Value::Quantity(area))) =
        evaluated.outcome
    else {
        panic!("original 4 square metres");
    };
    assert_eq!(
        area.magnitude(),
        &Rational::from_integer(Integer::from(4_i64))
    );
    let parsed = &original.expressions["areaMetres"];
    let children = parsed.root().children();
    let [_value, body] = children.as_slice() else {
        panic!("original parsed let children");
    };
    let conversion = parsed
        .cut(body.id(), &std::collections::BTreeMap::new())
        .expect("original conversion subtree");
    let refusal = graph
        .check_expression(
            vec![("area".to_owned(), power.value_type().clone())],
            &conversion,
            None,
            CheckMode::Kernel,
            CheckingLimits::default(),
        )
        .expect_err("fresh compound parameter type lacks retained formed-unit authority");
    assert_eq!(
        refusal.cause,
        qsl_semantics::check::CheckCause::IllTyped(quire_exact::IllTypedCause::TypeMismatch)
    );
    let mut table = UnitTable::declared(&original.units);
    let compound = original
        .units
        .compound_unit(&[(original.keys["m"].clone(), Integer::from(2_i64))])
        .expect("genuine root compound");
    assert_eq!(
        area.unit(),
        &compound.id().expect("owner compound identity")
    );
    table
        .insert(QuantityUnit::Compound(compound))
        .expect("retain genuine compound value unit");
    for (target_name, bits) in [("m2", 3), ("cm2", 17), ("cm2", 16)] {
        let target = QuantityUnit::Declared(Box::new(
            original
                .units
                .unit(original.keys[target_name].clone())
                .expect("original area target")
                .clone(),
        ));
        let mut meter = Meter::new(limits(
            bits,
            u64::from(target_name == "cm2"),
            1,
            if target_name == "m2" { 3 } else { 6 },
            1,
        ));
        let actual = quire_semantic_value::quantity::convert_quantity(
            table
                .resolve(&area)
                .expect("original compound value resolved"),
            &target,
            &QuantityTarget::Exact,
            &mut meter,
        )
        .expect("owner conversion static admission");
        let c = source::charge;
        let mut events = vec![c(P::UnitIdentityRead, Some(3), None, None, Some(1))];
        if target_name == "cm2" {
            events.extend([
                c(P::UnitEdge, None, None, Some(1), None),
                c(P::UnitRationalArithmetic, Some(5), None, None, None),
            ]);
        }
        if bits == 16 {
            let Outcome::Incomplete(stop) = actual else {
                panic!("unpaid reverse divide event");
            };
            assert_eq!(
                stop,
                Incomplete {
                    limit_kind: LimitKind::IntegerBits,
                    limit: 16,
                    consumed: 5,
                    next_charge: Integer::from(17_i64),
                    charge_point: P::UnitRationalArithmetic
                }
            );
            source::accounting(&meter, &events, [5, 0, 1, 1, 3, 0]);
        } else {
            let Outcome::Completed(converted) = actual else {
                panic!("compound conversion completes");
            };
            let ConvertedValue::Exact(value) = converted.value() else {
                panic!("exact rational conversion");
            };
            assert_eq!(
                value,
                &Rational::from_integer(Integer::from(if target_name == "m2" {
                    4_i64
                } else {
                    40_000
                }))
            );
            if target_name == "cm2" {
                events.push(c(P::UnitRationalArithmetic, Some(17), None, None, None));
            }
            events.extend([
                c(P::UnitTargetDomain, None, None, None, None),
                c(P::UnitResultRetain, None, None, None, Some(1)),
            ]);
            source::accounting(
                &meter,
                &events,
                if target_name == "m2" {
                    [3, 0, 0, 1, 3, 1]
                } else {
                    [17, 0, 1, 1, 6, 1]
                },
            );
        }
    }
    let terms = [("kg", 1_i64), ("m", 2), ("s", -2)]
        .into_iter()
        .map(|(name, power)| (original.keys[name].clone(), Integer::from(power)))
        .collect::<std::collections::BTreeMap<_, _>>()
        .into_iter()
        .collect::<Vec<_>>();
    let bridge = original
        .units
        .compound_unit(&terms)
        .expect("owner compound bridge");
    assert_eq!(
        bridge
            .terms()
            .map(|(key, power)| (key, power.clone()))
            .collect::<Vec<_>>(),
        terms
    );
    let torque_unit = original
        .units
        .unit(original.keys["N_m"].clone())
        .expect("original torque metadata");
    let energy_unit = original
        .units
        .unit(original.keys["J"].clone())
        .expect("original energy metadata");
    assert_ne!(torque_unit.dimension_node(), energy_unit.dimension_node());
    assert_eq!(torque_unit.dimension(), energy_unit.dimension());
    assert_eq!(bridge.dimension(), torque_unit.dimension());
    let bridge_unit = QuantityUnit::Compound(bridge);
    let bridge_id = table
        .insert(bridge_unit.clone())
        .expect("original compound bridge identity");
    let torque = Quantity::new(
        Rational::from_integer(Integer::from(1_i64)),
        original
            .units
            .declared_unit_id(original.keys["N_m"].clone())
            .expect("original torque unit"),
    );
    let mut current = torque;
    for target in [
        bridge_unit,
        QuantityUnit::Declared(Box::new(
            original
                .units
                .unit(original.keys["J"].clone())
                .expect("original energy unit")
                .clone(),
        )),
    ] {
        let mut meter = Meter::new(limits(1, 0, 1, 3, 1));
        let actual = quire_semantic_value::quantity::convert_quantity(
            table
                .resolve(&current)
                .expect("prior explicit step identity retained"),
            &target,
            &QuantityTarget::Exact,
            &mut meter,
        )
        .expect("explicit owner bridge step admits");
        let Outcome::Completed(converted) = actual else {
            panic!("each explicit owner bridge step completes");
        };
        let (next, _) = converted
            .identified_quantity()
            .expect("retain actual prior conversion value and identity");
        assert_eq!(
            next.magnitude(),
            &Rational::from_integer(Integer::from(1_i64))
        );
        assert_eq!(
            next.unit(),
            &target.id().expect("actual step target identity")
        );
        if current.unit().declared_key().is_some() {
            assert_eq!(next.unit(), &bridge_id);
        }
        source::accounting(
            &meter,
            &[
                source::charge(P::UnitIdentityRead, Some(1), None, None, Some(1)),
                source::charge(P::UnitTargetDomain, None, None, None, None),
                source::charge(P::UnitResultRetain, None, None, None, Some(1)),
            ],
            [1, 0, 0, 1, 3, 1],
        );
        current = next;
    }
    assert_eq!(
        current.unit(),
        &original
            .units
            .declared_unit_id(original.keys["J"].clone())
            .expect("final original joule identity")
    );
    let text=source::source_text(EXTRA,"function compoundTarget using v(): Quantity<J> pure { convert<Quantity<kg*m^2*s^-2>>(1 N_m) }");
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "quantity-operations"),
        "quantity-operations.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("compound target parse returns diagnostics");
    assert!(!parsed.is_admissible());
    let [diagnostic] = parsed.diagnostics() else {
        panic!(
            "one original compound-target grammar refusal: {:?}",
            parsed.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.code,
        qsl_foundation::diagnostic::Code::InvalidSyntax
    );
    let region = diagnostic
        .region
        .as_ref()
        .expect("original syntax refusal region");
    assert_eq!(region.source(), parsed.source().reference());
    assert_eq!(
        &text[usize::try_from(region.start()).expect("source start")
            ..usize::try_from(region.end()).expect("source end")],
        "*"
    );
}
