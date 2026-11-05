// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-113: source/native selected-model quantity subcells; generated remains Planned.
#[path = "support/tc187_p8_model_values.rs"]
mod model;

use ix_trace_rs::trace;

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-23")]
#[test]
fn probe_u38_original_equivalent_conversion_positive() {
    model::bind("U38", &["convert<Quantity<F::s>>(3 R::s)"], &["3 F::s"]);
    model::checked(&format!("{} function observe using v(): Quantity<F::s> pure {{ convert<Quantity<F::s>>(3 R::s) }}", model::RF), false);
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-26")]
#[test]
fn probe_u40_original_equivalent_conversion_positive() {
    model::bind(
        "U40",
        &["convert<Quantity<G::ms>>(3 R::s)"],
        &["3000 G::ms"],
    );
    model::checked(&format!("{} function observe using v(): Quantity<G::ms> pure {{ convert<Quantity<G::ms>>(3 R::s) }}", model::RG), true);
}

use std::collections::BTreeMap;

use qsl_package::CheckedPackage;
use qsl_semantics::check::{
    AssemblyCause, CheckCause, EquivalenceClasses, EquivalenceFault, EquivalenceRetained,
    PackageDeclarations,
};
use quire_eval::FamilyOutcome;
use quire_exact::{
    ComparisonOperator, IllTyped, IllTypedCause, Integer, LimitKind, Meter, Outcome, Quantity,
    Rational, ScalarLimits, Value,
};
use quire_semantic_value::{
    checking::CheckingLimits,
    quantity::{
        compare_quantity, convert_quantity, evaluate_quantity_unit, QuantityOperation,
        QuantityRefusal, QuantityTarget, QuantityUnit, UnitScope, UnitTable,
    },
};
#[path = "support/oracle_observation.rs"]
mod observation;

fn exact(n: i64) -> Rational {
    Rational::from_integer(n.into())
}
fn limits() -> ScalarLimits {
    ScalarLimits {
        integer_bits: 100000,
        decimal_digits: 100000,
        scale_expansion: 100000,
        text_input_bytes: 100000,
        text_scalars: 100000,
        normalized_scalars: 100000,
        unit_edges: 100000,
        value_occurrences: 100000,
        work_units: 100000,
        result_units: 100000,
    }
}
fn no_charge(meter: &Meter) {
    assert!(meter.admitted_charges().is_empty());
    for kind in LimitKind::ALL {
        assert_eq!(meter.consumed(kind), 0, "{kind:?}");
    }
}
fn mismatch(body: &str, clock: bool, selected: &str) {
    let source = model::source(body, clock);
    let package = model::assemble(&source, false).expect("original quantity declarations assemble");
    let regions = package.regions();
    let refusals = package
        .check(CheckingLimits::default())
        .expect_err("original typed refusal");
    let [refusal] = refusals.as_slice() else {
        panic!("one original refusal: {refusals:?}");
    };
    assert!(
        matches!(&refusal.cause, CheckCause::IllTyped(_)),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.cause.catalog_code().map(|code| code.to_string()),
        Some("ill_typed/type-mismatch".to_owned())
    );
    let region = regions.region(&refusal.location).expect("original region");
    assert_eq!(
        &source[usize::try_from(region.start()).expect("source start")
            ..usize::try_from(region.end()).expect("source end")],
        selected
    );
}
fn native_units() -> PackageDeclarations {
    model::assemble(
        &model::source(
            "dimension Dur = R::Time; unit dsec : Dur = rational(1, 1);",
            false,
        ),
        false,
    )
    .expect("original native units")
}
fn original_local(
    package: &PackageDeclarations,
    source: &str,
    name: &str,
) -> quire_semantic_value::unit::Unit {
    let keys: Vec<_> = package
        .nominal_spans
        .iter()
        .filter(|(_, span)| &source[span.start..span.end] == name)
        .map(|(key, _)| key.clone())
        .collect();
    let [key] = keys.as_slice() else {
        panic!("one original {name}: {keys:?}");
    };
    package
        .units
        .unit(key.clone())
        .expect("actual local unit")
        .clone()
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-21")]
#[test]
fn u36_calls_and_incompatible_operations_keep_original_source_loci() {
    model::bind(
        "U36",
        &[
            "f(3 R::s)",
            "f(3 F::s)",
            "f(3 s)",
            "3 R::s + 3 F::s",
            "3 R::s < 3 F::s",
            "convert<Quantity<F::s>>(3 R::s)",
            "convert<Quantity<dsec>>(3 R::s)",
        ],
        &[
            "the first call is admitted",
            "type-mismatch",
            "with no charge",
            "direct conversion to `dsec` refuses",
        ],
    );
    const F: &str = "function f using v(d: Quantity<R::s>): Boolean pure { true }";
    let package = model::checked(
        &format!("{F} function observe using v(): Boolean pure {{ f(3 R::s) }}"),
        false,
    );
    assert_eq!(
        package
            .graph()
            .function_state(1)
            .expect("actual call function")
            .name,
        "observe"
    );
    for arg in ["3 F::s", "3 s"] {
        mismatch(
            &format!("{F} function observe using v(): Boolean pure {{ f({arg}) }}"),
            false,
            arg,
        );
    }
    for expr in [
        "3 R::s + 3 F::s",
        "3 R::s < 3 F::s",
        "convert<Quantity<F::s>>(3 R::s)",
    ] {
        let return_type = if expr.contains('<') && !expr.starts_with("convert") {
            "Boolean"
        } else {
            "Quantity<R::s>"
        };
        mismatch(
            &format!("function observe using v(): {return_type} pure {{ {expr} }}"),
            false,
            expr,
        );
    }
    mismatch("dimension Dur = R::Time; unit dsec : Dur = rational(1, 1); function observe using v(): Quantity<dsec> pure { convert<Quantity<dsec>>(3 R::s) }", false, "convert<Quantity<dsec>>(3 R::s)");
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-21")]
#[test]
fn u36_native_incompatibility_refuses_before_every_charge() {
    model::bind(
        "U36",
        &[
            "3 R::s + 3 F::s",
            "3 R::s < 3 F::s",
            "convert<Quantity<F::s>>(3 R::s)",
            "convert<Quantity<dsec>>(3 R::s)",
        ],
        &[
            "incompatible dimensions with no charge",
            "direct conversion to `dsec` refuses",
        ],
    );
    let package = native_units();
    let r = model::imported_unit(&package, "acme/rpc", "s");
    let f = model::imported_unit(&package, "acme/flow", "s");
    assert_ne!(r.dimension_node(), f.dimension_node());
    let source = model::source(
        "dimension Dur = R::Time; unit dsec : Dur = rational(1, 1);",
        false,
    );
    let d = original_local(&package, &source, "dsec");
    assert_ne!(r.dimension_node(), d.dimension_node());
    assert_eq!(r.dimension(), d.dimension());
    let table = UnitTable::declared(&package.units);
    let rq = Quantity::new(exact(3), r.id());
    let fq = Quantity::new(exact(3), f.id());
    let mut meter = Meter::new(limits());
    let ill = IllTyped {
        cause: IllTypedCause::IncompatibleDimensions,
    };
    assert_eq!(
        evaluate_quantity_unit(
            QuantityOperation::Add(
                table.resolve(&rq).expect("R"),
                table.resolve(&fq).expect("F")
            ),
            &mut meter
        ),
        Err(QuantityRefusal::IllTyped(ill))
    );
    no_charge(&meter);
    assert_eq!(
        compare_quantity(
            ComparisonOperator::Less,
            table.resolve(&rq).expect("R"),
            table.resolve(&fq).expect("F"),
            &mut meter
        ),
        Err(ill)
    );
    no_charge(&meter);
    for target in [f.id(), d.id()] {
        assert_eq!(
            convert_quantity(
                table.resolve(&rq).expect("R"),
                table.get(target).expect("target"),
                &QuantityTarget::Exact,
                &mut meter
            ),
            Err(ill)
        );
        no_charge(&meter);
    }
}

fn selected(
    package: &CheckedPackage,
    whole: bool,
) -> (qsl_semantics::check::CheckedNode<'_>, usize) {
    let function = package.graph().function_state(0).expect("original observe");
    assert_eq!(function.name, "observe");
    if whole {
        (function.body, function.slots)
    } else {
        let children = function.body.children();
        let [left, _] = children.as_slice() else {
            panic!("original Boolean equality wrapper");
        };
        (*left, function.slots)
    }
}

fn agree(package: &CheckedPackage, whole: bool, native: &Outcome<Quantity>, native_meter: &Meter) {
    let (node, slots) = selected(package, whole);
    let mut meter = Meter::new(limits());
    let evaluation = qsl_eval::value::evaluate_closed(package, node, slots, &mut meter)
        .expect("original closed QSL stage");
    let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
        panic!("original value stage");
    };
    assert!(evaluation.location.is_none());
    assert!(evaluation.losses.is_empty());
    let actual = match native.clone() {
        Outcome::Completed(value) => Outcome::Completed(Value::Quantity(value)),
        Outcome::Undefined(reason) => Outcome::Undefined(reason),
        Outcome::Refused(reason) => Outcome::Refused(reason),
        Outcome::Incomplete(reason) => Outcome::Incomplete(reason),
    };
    assert_eq!(outcome, actual, "entire original outcome");
    let losses = Vec::<quire_semantic_value::loss::ValueLoss>::new();
    assert_eq!(
        observation::snapshot(&outcome, &losses, &meter),
        observation::snapshot(&actual, &losses, native_meter),
        "complete result, losses, ordered charges and all counters"
    );
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-21")]
#[test]
fn u36_quotients_and_products_preserve_native_canonical_maps_and_ledgers() {
    model::bind(
        "U36",
        &[
            "3 R::s / 3 F::s",
            "3 R::s / 3 R::s",
            "(3 R::s) * (2 F::s)",
            "(3 R::s) * (2 R::s)",
        ],
        &[
            "is not dimensionless",
            "second is dimensionless",
            "unequal compound units",
        ],
    );
    let original = native_units();
    let r = model::imported_unit(&original, "acme/rpc", "s");
    let f = model::imported_unit(&original, "acme/flow", "s");
    let table = UnitTable::declared(&original.units);
    let mut products = Vec::new();
    for (expr, divide, other, expected, dimension_terms, unit_terms) in [
        (
            "3 R::s / 3 F::s",
            true,
            &f,
            1,
            vec![(r.dimension_node(), 1), (f.dimension_node(), -1)],
            vec![(r.root(), 1), (f.root(), -1)],
        ),
        ("3 R::s / 3 R::s", true, &r, 1, vec![], vec![]),
        (
            "(3 R::s) * (2 F::s)",
            false,
            &f,
            6,
            vec![(r.dimension_node(), 1), (f.dimension_node(), 1)],
            vec![(r.root(), 1), (f.root(), 1)],
        ),
        (
            "(3 R::s) * (2 R::s)",
            false,
            &r,
            6,
            vec![(r.dimension_node(), 2)],
            vec![(r.root(), 2)],
        ),
    ] {
        let package = model::checked(
            &format!("function observe using v(): Boolean pure {{ ({expr}) = ({expr}) }}"),
            false,
        );
        let left = Quantity::new(exact(3), r.id());
        let right = Quantity::new(exact(if divide { 3 } else { 2 }), other.id());
        let a = table.resolve(&left).expect("actual R operand");
        let b = table.resolve(&right).expect("actual selected operand");
        let operation = if divide {
            QuantityOperation::Divide(a, b)
        } else {
            QuantityOperation::Multiply(a, b)
        };
        let mut meter = Meter::new(limits());
        let (outcome, unit) =
            evaluate_quantity_unit(operation, &mut meter).expect("native original operation");
        let Outcome::Completed(quantity) = &outcome else {
            panic!("complete original quantity: {outcome:?}");
        };
        assert_eq!(quantity.magnitude(), &exact(expected));
        assert_eq!(quantity.unit(), &unit.id());
        let QuantityUnit::Compound(compound) = unit.unit() else {
            panic!("canonical compound");
        };
        let expected_dimensions: BTreeMap<_, Integer> = dimension_terms
            .into_iter()
            .map(|(key, exponent)| (key, Integer::from(i64::from(exponent))))
            .collect();
        let expected_units: BTreeMap<_, Integer> = unit_terms
            .into_iter()
            .map(|(key, exponent)| (key, Integer::from(i64::from(exponent))))
            .collect();
        assert_eq!(
            compound
                .dimension()
                .exponents()
                .map(|(key, n)| (key, n.clone()))
                .collect::<BTreeMap<_, _>>(),
            expected_dimensions
        );
        assert_eq!(
            compound
                .terms()
                .map(|(key, n)| (key, n.clone()))
                .collect::<BTreeMap<_, _>>(),
            expected_units
        );
        agree(&package, false, &outcome, &meter);
        if !divide {
            products.push(unit);
        }
    }
    assert_eq!(products.len(), 2);
    assert_ne!(
        products[0], products[1],
        "equal magnitudes do not identify different model roots"
    );
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-21")]
#[test]
fn u36_original_whole_let_bridges_compound_to_dur_with_complete_ledger() {
    model::bind(
        "U36",
        &["let c = ((3 R::s) * (1 R::s)) / (1 R::s) in convert<Quantity<dsec>>(c)"],
        &[
            "binding `c` is `3`",
            "one term `R::s` at exponent 1",
            "3 dsec",
        ],
    );
    let declarations = "dimension Dur = R::Time; unit dsec : Dur = rational(1, 1);";
    let package = model::checked(&format!("{declarations} function observe using v(): Quantity<dsec> pure {{ let c = ((3 R::s) * (1 R::s)) / (1 R::s) in convert<Quantity<dsec>>(c) }}"), false);
    let source = model::source(declarations, false);
    let original = model::assemble(&source, false).expect("original owner unit acquisition");
    let r = model::imported_unit(&original, "acme/rpc", "s");
    let d = original_local(&original, &source, "dsec");
    let table = package.graph().scope().types().units();
    let mut scope = UnitScope::new(table);
    let three = Quantity::new(exact(3), r.id());
    let one = Quantity::new(exact(1), r.id());
    let mut meter = Meter::new(limits());
    let (product, product_unit) = evaluate_quantity_unit(
        QuantityOperation::Multiply(
            scope.resolve(&three).expect("R three"),
            scope.resolve(&one).expect("R one"),
        ),
        &mut meter,
    )
    .expect("native product");
    let Outcome::Completed(product) = product else {
        panic!("product must complete before quotient");
    };
    scope.form(product_unit);
    let (bound, bound_unit) = evaluate_quantity_unit(
        QuantityOperation::Divide(
            scope.resolve(&product).expect("actual formed product"),
            scope.resolve(&one).expect("R one"),
        ),
        &mut meter,
    )
    .expect("native quotient");
    let Outcome::Completed(bound) = bound else {
        panic!("quotient must complete before conversion");
    };
    assert_eq!(bound.magnitude(), &exact(3));
    let QuantityUnit::Compound(compound) = bound_unit.unit() else {
        panic!("let binding compound unit");
    };
    assert_eq!(
        compound
            .terms()
            .map(|(key, n)| (key, n.clone()))
            .collect::<Vec<_>>(),
        [(r.root(), Integer::from(1_i64))]
    );
    assert_eq!(
        compound
            .dimension()
            .exponents()
            .map(|(key, n)| (key, n.clone()))
            .collect::<Vec<_>>(),
        [(r.dimension_node(), Integer::from(1_i64))]
    );
    scope.form(bound_unit);
    let converted = convert_quantity(
        scope.resolve(&bound).expect("actual formed binding"),
        scope.get(d.id()).expect("actual Dur unit"),
        &QuantityTarget::Exact,
        &mut meter,
    )
    .expect("compound meets Dur map");
    let Outcome::Completed(converted) = converted else {
        panic!("bridge must complete");
    };
    let (result, unit) = converted
        .identified_quantity()
        .expect("exact original native conversion handoff");
    assert_eq!(result, Quantity::new(exact(3), d.id()));
    assert_eq!(unit.unit(), scope.get(d.id()).expect("original Dur target"));
    agree(&package, true, &Outcome::Completed(result), &meter);
}

fn span(source: &str, text: &str) -> qsl_foundation::Span {
    let starts: Vec<_> = source.match_indices(text).map(|(index, _)| index).collect();
    let [start] = starts.as_slice() else {
        panic!("one original {text}: {starts:?}");
    };
    qsl_foundation::Span {
        start: *start,
        end: start + text.len(),
    }
}
fn dimension(package: &PackageDeclarations, alias: &str) -> quire_exact::NodeKey {
    match alias {
        "R::Time" => model::imported_unit(package, "acme/rpc", "s").dimension_node(),
        "F::Time" => model::imported_unit(package, "acme/flow", "s").dimension_node(),
        "G::Time" => model::imported_unit(package, "acme/clock", "ms").dimension_node(),
        "Time" => {
            let source = model::source("", false);
            original_local(package, &source, "s").dimension_node()
        }
        _ => panic!("unselected dimension {alias}"),
    }
}
fn admitted_equivalences(body: &str, clock: bool, pairs: &[(&str, [&str; 2], Rational)]) {
    let source = model::source(body, clock);
    let original = model::assemble(&source, false).expect("original equivalence admission");
    assert_eq!(original.dimension_equivalences.len(), pairs.len());
    let classes = EquivalenceClasses::new(&original.dimension_equivalences);
    for (row, (text, names, ratio)) in original.dimension_equivalences.iter().zip(pairs) {
        assert_eq!(row.span, span(&source, text));
        let mut keys = names.map(|name| dimension(&original, name));
        let expected_scale = if keys[0] < keys[1] {
            ratio.clone()
        } else {
            keys.swap(0, 1);
            exact(1).div(ratio).expect("positive reciprocal")
        };
        assert_eq!(row.dimensions, keys);
        assert_eq!(row.root_scale, expected_scale);
        let (root, a) = classes.class(dimension(&original, names[0]));
        let (other, b) = classes.class(dimension(&original, names[1]));
        assert_eq!(root, other);
        assert_eq!(
            a.div(&b),
            Some(ratio.clone()),
            "original class root ratio {names:?}"
        );
        let id = serde_json::to_value(qsl_semantics::check::wire_id::structural_id(&row.key))
            .expect("actual row owner");
        assert_eq!(id["package"], "agent-ix/tc113-model-values");
        assert_eq!(id["path"][0], "scalar_type");
        assert_eq!(id["path"][1], "dimension_equivalence");
    }
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-23", "QSpec-FR-142-AC-26")]
#[test]
fn u38_u40_original_ratio_spellings_and_three_root_classes_admit() {
    model::bind(
        "U38",
        &["1 R::s = 1000 F::ms", "Time ~ F::Time by 1 s = 1 F::s"],
        &["with three roots"],
    );
    model::bind(
        "U40",
        &[
            "1 R::s = 1000 G::ms",
            "1 R::s = 1 G::s",
            "1000 G::ms = 1 R::s",
            "1 F::s = 1000 G::ms",
        ],
        &[
            "three spellings are admitted",
            "root ratio, 1000",
            "declaration is admitted",
        ],
    );
    admitted_equivalences(
        model::RF,
        false,
        &[(model::RF, ["R::Time", "F::Time"], exact(1))],
    );
    let local = "dimension Time ~ F::Time by 1 s = 1 F::s;";
    admitted_equivalences(
        &format!("{} {local}", model::RF),
        false,
        &[
            (model::RF, ["R::Time", "F::Time"], exact(1)),
            (local, ["Time", "F::Time"], exact(1)),
        ],
    );
    for (text, names, ratio) in [
        (model::RG, ["R::Time", "G::Time"], exact(1000)),
        (
            "dimension R::Time ~ G::Time by 1 R::s = 1 G::s;",
            ["R::Time", "G::Time"],
            exact(1000),
        ),
        (
            "dimension G::Time ~ R::Time by 1000 G::ms = 1 R::s;",
            ["G::Time", "R::Time"],
            exact(1).div(&exact(1000)).expect("thousandth"),
        ),
    ] {
        admitted_equivalences(text, true, &[(text, names, ratio)]);
    }
    let fg = "dimension F::Time ~ G::Time by 1 F::s = 1000 G::ms;";
    admitted_equivalences(
        &format!("{} {} {fg}", model::RG, model::RF),
        true,
        &[
            (model::RG, ["R::Time", "G::Time"], exact(1000)),
            (model::RF, ["R::Time", "F::Time"], exact(1)),
            (fg, ["F::Time", "G::Time"], exact(1000)),
        ],
    );
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-26")]
#[test]
fn u40_conflicting_cycle_retains_every_original_declaration_in_order() {
    model::bind(
        "U40",
        &["1 F::s = 1001 G::ms"],
        &[
            "conflicting-definition",
            "retaining the three declarations in source order",
        ],
    );
    let bad = "dimension F::Time ~ G::Time by 1 F::s = 1001 G::ms;";
    let source = model::source(&format!("{} {} {bad}", model::RG, model::RF), true);
    let refusal = model::assemble(&source, false).expect_err("original closing ratio conflicts");
    let [error] = refusal.errors.as_slice() else {
        panic!("one cycle refusal: {refusal:?}");
    };
    assert_eq!(
        error.cause,
        AssemblyCause::DimensionEquivalence {
            fault: EquivalenceFault::Conflicting,
            retained: [
                (model::RG, ["R::Time", "G::Time"]),
                (model::RF, ["R::Time", "F::Time"]),
                (bad, ["F::Time", "G::Time"])
            ]
            .into_iter()
            .map(|(text, names)| EquivalenceRetained::Declaration {
                dimensions: names.map(str::to_owned),
                span: span(&source, text)
            })
            .collect()
        }
    );
    assert_eq!(error.span, span(&source, bad));
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/conflicting-definition"
    );
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-24")]
#[test]
fn u38_missing_mass_unit_and_unanchored_force_keep_original_named_causes() {
    model::bind(
        "U38",
        &["1 R::kg = 1 F::kg", "Force = F::Mass"],
        &["at `F::kg`", "ix://acme/flow/dimension/Mass", "and `n`"],
    );
    for (body, missing) in [
        ("dimension R::Mass ~ F::Mass by 1 R::kg = 1 F::kg;", true),
        (
            "dimension Force = F::Mass; unit n : Force = rational(1, 1);",
            false,
        ),
    ] {
        let source = model::source(body, false);
        let refusal = model::assemble(&source, true).expect_err("original mass use refuses");
        let [error] = refusal.errors.as_slice() else {
            panic!("one first mass refusal: {refusal:?}");
        };
        assert_eq!(
            error.cause.catalog_code().to_string(),
            "missing_declaration/missing-name"
        );
        if missing {
            assert_eq!(
                error.cause,
                AssemblyCause::UnresolvedQuantityName {
                    name: "F::kg".to_owned()
                }
            );
            assert_eq!(error.span, span(&source, "F::kg"));
        } else {
            assert_eq!(
                error.cause,
                AssemblyCause::UnanchoredBaseDimension {
                    dimension: "ix://acme/flow/dimension/Mass".to_owned(),
                    unit: "n".to_owned()
                }
            );
            assert_eq!(
                &source[error.span.start..error.span.end],
                "model F = \"acme/flow\";"
            );
        }
    }
}

const TORQUE: &str = "dimension Length; unit m : Length = rational(1, 1); dimension Mass; unit kg : Mass = rational(1, 1); dimension Torque = Length ^ 2 * Mass / Time ^ 2; dimension Energy = Length ^ 2 * Mass / Time ^ 2; unit N_m : Torque = rational(1, 1); unit J : Energy = rational(1, 1);";
const TE: &str = "dimension Torque ~ Energy by 1 N_m = 1 J;";

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-24")]
#[test]
fn u38_derived_equivalence_admission_and_invalid_ratio_keep_original_dimensions() {
    model::bind(
        "U38",
        &["Torque ~ Energy by 1 N_m = 1 J", "by 1 N_m = 2 J"],
        &["converts exactly with it", "incompatible-definition"],
    );
    let source = model::source(&format!("{TORQUE} {TE}"), false);
    let package = model::assemble(&source, false).expect("original derived equivalence admits");
    let torque = original_local(&package, &source, "N_m");
    let energy = original_local(&package, &source, "J");
    assert_ne!(torque.dimension_node(), energy.dimension_node());
    assert_eq!(torque.dimension(), energy.dimension());
    let [row] = package.dimension_equivalences.as_slice() else {
        panic!("one original Torque/Energy relation");
    };
    assert_eq!(row.span, span(&source, TE));
    assert_eq!(row.root_scale, exact(1));
    let mut pair = [torque.dimension_node(), energy.dimension_node()];
    pair.sort();
    assert_eq!(row.dimensions, pair);
    let classes = EquivalenceClasses::new(&package.dimension_equivalences);
    assert_eq!(
        classes.class(torque.dimension_node()),
        classes.class(energy.dimension_node())
    );
    let bad = "dimension Torque ~ Energy by 1 N_m = 2 J;";
    let source = model::source(&format!("{TORQUE} {bad}"), false);
    let refusal = model::assemble(&source, false).expect_err("incompatible derived root ratio");
    let [error] = refusal.errors.as_slice() else {
        panic!("one derived ratio refusal: {refusal:?}");
    };
    assert_eq!(error.span, span(&source, bad));
    assert_eq!(
        error.cause,
        AssemblyCause::DimensionEquivalence {
            fault: EquivalenceFault::Incompatible,
            retained: vec![
                EquivalenceRetained::Dimension {
                    name: "Torque".to_owned(),
                    key: Some(torque.dimension_node())
                },
                EquivalenceRetained::Dimension {
                    name: "Energy".to_owned(),
                    key: Some(energy.dimension_node())
                }
            ]
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/incompatible-definition"
    );
    mismatch(&format!("{TORQUE} function observe using v(): Quantity<J> pure {{ convert<Quantity<J>>(1 N_m) }}"),false,"convert<Quantity<J>>(1 N_m)");
    let original = model::assemble(&model::source(TORQUE, false), false)
        .expect("original derived native graph");
    let table = UnitTable::declared(&original.units);
    let quantity = Quantity::new(exact(1), torque.id());
    let mut meter = Meter::new(limits());
    assert_eq!(
        convert_quantity(
            table.resolve(&quantity).expect("original torque"),
            table.get(energy.id()).expect("original energy"),
            &QuantityTarget::Exact,
            &mut meter
        ),
        Err(IllTyped {
            cause: IllTypedCause::IncompatibleDimensions
        })
    );
    no_charge(&meter);
}

#[trace("TC-113", "QSpec-TC-187", "QSpec-FR-142-AC-23", "QSpec-FR-142-AC-26")]
#[test]
fn u38_u40_unconverted_source_operations_and_unrelated_conversion_refuse() {
    model::bind(
        "U38",
        &[
            "3 R::s + 3 F::s",
            "3 R::s = 3 F::s",
            "f(3 F::s)",
            "convert<Quantity<R::m>>(3 F::s)",
        ],
        &["distinct units", "at the argument", "unrelated dimensions"],
    );
    model::bind("U40", &["3 R::s + 3 G::ms"], &["distinct units"]);
    for (eq, clock, other) in [(model::RF, false, "F::s"), (model::RG, true, "G::ms")] {
        let sum = format!("3 R::s + 3 {other}");
        mismatch(
            &format!("{eq} function observe using v(): Quantity<R::s> pure {{ {sum} }}"),
            clock,
            &sum,
        );
    }
    mismatch(
        &format!(
            "{} function observe using v(): Boolean pure {{ 3 R::s = 3 F::s }}",
            model::RF
        ),
        false,
        "3 R::s = 3 F::s",
    );
    mismatch(&format!("{} function f using v(d: Quantity<R::s>): Boolean pure {{ true }} function observe using v(): Boolean pure {{ f(3 F::s) }}",model::RF),false,"3 F::s");
    mismatch(&format!("{} function observe using v(): Quantity<R::m> pure {{ convert<Quantity<R::m>>(3 F::s) }}",model::RF),false,"convert<Quantity<R::m>>(3 F::s)");
    let original =
        model::assemble(&model::source(model::RF, false), false).expect("original Time relation");
    let f = model::imported_unit(&original, "acme/flow", "s");
    let m = model::imported_unit(&original, "acme/rpc", "m");
    let table = UnitTable::declared(&original.units);
    let quantity = Quantity::new(exact(3), f.id());
    let mut meter = Meter::new(limits());
    assert_eq!(
        convert_quantity(
            table.resolve(&quantity).expect("original flow"),
            table.get(m.id()).expect("unrelated original Length"),
            &QuantityTarget::Exact,
            &mut meter
        ),
        Err(IllTyped {
            cause: IllTypedCause::IncompatibleDimensions
        })
    );
    no_charge(&meter);
}
