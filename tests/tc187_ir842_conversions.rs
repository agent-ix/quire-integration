// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original affine, scale-one alias, centimetre and zero-power quantity operations
//! compared with their generated native oracles.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;
#[allow(
    dead_code,
    reason = "shared quantity support also serves other independent vector cases"
)]
#[path = "support/tc187_p9_source.rs"]
mod source;

use std::num::NonZeroU64;

use ix_trace_rs::trace;
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2, CheckedTerm};
use quire_exact::{ChargePoint, InjectedDenial, Integer, Outcome, Rational, ScalarLimits, Value};

const EXTRA: &str = "dimension Area = Distance ^ 2; unit m2 : Area = rational(1, 1);
 unit cm2 : Area = rational(1, 10000) * m2;
 dimension Mass; unit kg : Mass = rational(1, 1);
 dimension Energy = Mass * Distance ^ 2 / Duration ^ 2; unit J : Energy = rational(1, 1);
 dimension Temperature; unit K : Temperature = rational(1, 1);
 unit degC : Temperature = rational(1, 1) * K + rational(5463, 20);
 unit degF : Temperature = rational(5, 9) * K + rational(45967, 180);
 unit rev : Temperature = rational(-1, 1) * K;
 unit u1 : Temperature = rational(1, 1) * K + rational(10, 1);
 unit u2 : Temperature = rational(1, 1) * u1 + rational(-10, 1);
 unit metre_alias : Distance = rational(1, 1) * m;";

const FUNCTIONS: &str = "
 function celsiusToKelvin using v(): Quantity<K> pure { convert<Quantity<K>>(0 degC) }
 function fahrenheitToKelvin using v(): Quantity<K> pure { convert<Quantity<K>>(32 degF) }
 function aliasToMetre using v(): Quantity<m> pure { convert<Quantity<m>>(7 metre_alias) }
 function centimetresToMetres using v(): Quantity<m> pure { convert<Quantity<m>>(100 cm) }
 function areaToCentimetres using v(): Quantity<cm2> pure { convert<Quantity<cm2>>(2 m ^ 2) }
 function areaToMetres using v(): Quantity<m2> pure { convert<Quantity<m2>>(2 m ^ 2) }
 function compoundToJoule using v(): Quantity<J> pure { convert<Quantity<J>>(1 kg * 1 m ^ 2 * 1 s ^ -2) }
 function quotientNormalized using v(): Boolean pure { (1 m * 1 s) / 1 s = (1 m * 1 s) / 1 s }
 function lengthToPowerZero using v(): Boolean pure { 1 m ^ 0 = 1 m ^ 0 }
";

fn limits(integer_bits: u64, unit_edges: u64, work_units: u64, result_units: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges,
        value_occurrences: 1,
        work_units,
        result_units,
    }
}

/// Require QSL's own completed value for `name` at a generous allowance: the QSpec-stated
/// magnitude, and the declared unit when one is named.
fn assert_value(
    original: &source::Original,
    name: &str,
    child: bool,
    magnitude: (i64, i64),
    unit: Option<&str>,
) {
    let (evaluation, _) = source::evaluate(original, name, child, roomy(1024));
    let quire_eval::FamilyOutcome::Evaluated(Outcome::Completed(Value::Quantity(quantity))) =
        evaluation.outcome
    else {
        panic!(
            "{name}: QSL completes with a quantity: {:?}",
            evaluation.outcome
        );
    };
    assert_eq!(
        quantity.magnitude(),
        &Rational::new(Integer::from(magnitude.0), Integer::from(magnitude.1))
            .expect("authored magnitude"),
        "{name}"
    );
    if let Some(unit) = unit {
        assert_eq!(
            quantity.unit(),
            &original
                .units
                .declared_unit_id(original.keys[unit].clone())
                .expect("declared unit"),
            "{name}"
        );
    }
}

fn roomy(work_units: u64) -> ScalarLimits {
    full(1024, 16, 16, work_units, 16)
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u04_and_u09b_original_conversions_match_generated_native_oracles() {
    source::bind("U04", &["0 degC", "32 degF"], &["5463/20"]);
    source::bind("U09b", &["scale one"], &["conversion succeeds exactly"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    assert_value(&original, "celsiusToKelvin", false, (5463, 20), Some("K"));
    assert_value(
        &original,
        "fahrenheitToKelvin",
        false,
        (5463, 20),
        Some("K"),
    );
    assert_value(&original, "aliasToMetre", false, (7, 1), Some("m"));
    let mut cases = Vec::new();
    for name in ["celsiusToKelvin", "fahrenheitToKelvin", "aliasToMetre"] {
        for work in [1024, 0, 1] {
            cases.push((name, false, roomy(work)));
        }
    }
    assert_eq!(source::generation(&original, &cases, &[], &[]), 9);
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u10_original_exact_tuple_and_named_final_denial_match_generated_native_oracles() {
    source::bind("U10", &["Convert `100 cm` to `m`"], &["`1 m`"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    assert_value(&original, "centimetresToMetres", false, (1, 1), Some("m"));
    let tuple = limits(8, 1, 6, 1);
    assert_eq!(
        source::generation(
            &original,
            &[("centimetresToMetres", false, tuple)],
            &[],
            &[]
        ),
        1
    );
    let denial = InjectedDenial {
        point: ChargePoint::UnitResultRetain,
        occurrence: NonZeroU64::new(1).expect("first occurrence"),
    };
    assert_eq!(
        source::generation_denied(&original, &[("centimetresToMetres", false, tuple)], denial),
        1
    );
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u02_original_zero_power_normalizes_to_the_empty_dimension_in_generated_native_oracles() {
    source::bind("U02", &["`L^0`"], &["the empty dimension"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    let cases =
        [1024, 0, 1, 2, 3].map(|work| ("lengthToPowerZero", true, limits(1024, 0, work, 4)));
    assert_eq!(source::generation(&original, &cases, &[], &[]), 5);
}

fn full(
    integer_bits: u64,
    unit_edges: u64,
    value_occurrences: u64,
    work_units: u64,
    result_units: u64,
) -> ScalarLimits {
    ScalarLimits {
        value_occurrences,
        ..limits(integer_bits, unit_edges, work_units, result_units)
    }
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u22_original_compound_area_conversions_match_generated_native_oracles() {
    source::bind("U22", &["4 m^2"], &["40000"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    assert_value(
        &original,
        "areaToCentimetres",
        false,
        (40000, 1),
        Some("cm2"),
    );
    assert_value(&original, "areaToMetres", false, (4, 1), Some("m2"));
    let mut cases = Vec::new();
    for name in ["areaToCentimetres", "areaToMetres"] {
        for l in [
            full(17, 1, 1, 6, 1),
            full(16, 1, 1, 6, 1),
            roomy(1024),
            roomy(0),
            roomy(1),
        ] {
            cases.push((name, false, l));
        }
    }
    assert_eq!(source::generation(&original, &cases, &[], &[]), 10);
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u25_original_compound_to_joule_conversion_matches_generated_native_oracles() {
    source::bind("U25", &["kg*m^2*s^-2"], &["1 J"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    assert_value(&original, "compoundToJoule", false, (1, 1), Some("J"));
    let cases = [
        full(1, 0, 1, 3, 1),
        roomy(1024),
        roomy(0),
        roomy(1),
        roomy(2),
    ]
    .map(|l| ("compoundToJoule", false, l));
    assert_eq!(source::generation(&original, &cases, &[], &[]), 5);
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn u02_original_product_quotient_normalizes_to_length_in_generated_native_oracles() {
    source::bind("U02", &["(L*T)/T"], &["`L`"]);
    let original = source::original_with(EXTRA, FUNCTIONS);
    assert_value(&original, "quotientNormalized", true, (1, 1), None);
    let cases = [1024, 0, 1, 2, 3, 6].map(|work| ("quotientNormalized", true, roomy(work)));
    assert_eq!(source::generation(&original, &cases, &[], &[]), 6);
}

/// Every public quantity ordering and equality operator over the declared unit grid.
#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn quantity_comparison_operators_over_the_distance_and_time_grid_match_generated_native_oracles() {
    comparison_grid(&["m", "cm", "inch", "s", "kg", "K", "rev"]);
}

/// The affine, energy and area half of the comparison grid.
#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn quantity_comparison_operators_over_the_affine_and_area_grid_match_generated_native_oracles() {
    comparison_grid(&["degC", "degF", "u2", "J", "m2", "cm2"]);
}

/// Distinct units are statically ill-typed, so each grid unit is compared with itself over
/// ordered, equal and reversed magnitudes under every comparison operator.
fn comparison_grid(grid: &[&str]) {
    let extra = format!("{EXTRA} unit inch : Distance = rational(127, 5000) * m;");
    let values = [(1, 2), (2, 2), (3, 2)];
    let mut functions = String::new();
    let mut names = Vec::new();
    for (unit_index, unit) in grid.iter().enumerate() {
        for (operator_index, operator) in ["=", "!=", "<", "<=", ">", ">="].iter().enumerate() {
            for (value_index, (a, b)) in values.iter().enumerate() {
                let name = format!("compare_{unit_index}_{operator_index}_{value_index}");
                functions.push_str(&format!(
                    "function {name} using v(): Boolean pure {{ {a} {unit} {operator} {b} {unit} }}\n"
                ));
                names.push(name);
            }
        }
    }
    let original = source::original_with(&extra, &functions);
    let mut cases = Vec::new();
    for name in &names {
        for l in [roomy(1024), roomy(0), roomy(1), roomy(3), roomy(6)] {
            cases.push((name.as_str(), false, l));
        }
    }
    assert_eq!(
        source::generation(&original, &cases, &[], &[]),
        names.len() * 5
    );
}

/// Every public quantity arithmetic operator over the declared unit grid.
#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn quantity_arithmetic_operators_over_the_unit_grid_match_generated_native_oracles() {
    let extra = format!("{EXTRA} unit inch : Distance = rational(127, 5000) * m;");
    let mut expressions = Vec::new();
    for unit in ["m", "cm", "inch", "kg", "s", "rev", "u2"] {
        for operator in ["+", "-"] {
            expressions.push(format!("5 {unit} {operator} 3 {unit}"));
            expressions.push(format!("3 {unit} {operator} 5 {unit}"));
        }
    }
    for (left, right) in [
        ("m", "s"),
        ("cm", "inch"),
        ("m", "m"),
        ("kg", "s"),
        ("cm", "cm"),
    ] {
        for operator in ["*", "/"] {
            expressions.push(format!("6 {left} {operator} 4 {right}"));
            expressions.push(format!("1 {left} {operator} 3 {right}"));
        }
    }
    for unit in ["m", "cm", "inch", "s", "kg"] {
        for exponent in [0, 1, 2, 3, -1, -2] {
            expressions.push(format!("2 {unit} ^ {exponent}"));
        }
    }
    let mut functions = String::new();
    let mut names = Vec::new();
    for (index, expression) in expressions.iter().enumerate() {
        let name = format!("arithmetic_{index}");
        functions.push_str(&format!(
            "function {name} using v(): Boolean pure {{ ({expression}) = ({expression}) }}\n"
        ));
        names.push(name);
    }
    let original = source::original_with(&extra, &functions);
    let mut cases = Vec::new();
    for name in &names {
        for l in [roomy(1024), roomy(0), roomy(1), roomy(3), roomy(5)] {
            cases.push((name.as_str(), true, l));
        }
    }
    assert_eq!(
        source::generation(&original, &cases, &[], &[]),
        names.len() * 5
    );
}
