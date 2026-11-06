// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original exact and carried quantity conversions at their charged boundaries.
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

const EXTRA: &str = "unit inch : Distance = rational(127,5000) * m;
 dimension Temperature; unit K : Temperature = rational(1,1);
 unit degC : Temperature = rational(1,1) * K + rational(5463,20);
 unit u1 : Temperature = rational(1,1) * K + rational(10,1);
 unit u2 : Temperature = rational(1,1) * u1 + rational(-10,1);";
const FUNCTIONS:&str="
 function inches using v(): Quantity<cm> pure { convert<Quantity<cm>>(1 inch) }
 function chain using v(): Quantity<u1> pure { convert<Quantity<u1>>(1 u2) }
 function decimalExact using v(): Quantity<m,Decimal[-100,100;2,2;exact]> pure { convert<Quantity<m,Decimal[-100,100;2,2;exact]>>(1 inch) }
 function decimalRounded using v(): Quantity<m,Decimal[-100,100;2,2;nearest-even]> pure { convert<Quantity<m,Decimal[-100,100;2,2;nearest-even]>>(1 inch) }
 function decimalOutside using v(): Quantity<m,Decimal[-2,2;0,0;exact]> pure { convert<Quantity<m,Decimal[-2,2;0,0;exact]>>(3 m) }
 function hugeScale using v(): Quantity<m,Decimal[0,1;0,4294967295;exact]> pure { convert<Quantity<m,Decimal[0,1;0,4294967295;exact]>>(1 m) }
 function integerExact using v(): Quantity<m,Int[-2,2]> pure { convert<Quantity<m,Int[-2,2]>>(1 inch) }
 function integerRounded using v(): Quantity<m,Int[-2,2;nearest-even]> pure { convert<Quantity<m,Int[-2,2;nearest-even]>>(1 inch) }
 function integerOutside using v(): Quantity<m,Int[-2,2]> pure { convert<Quantity<m,Int[-2,2]>>(3 m) }
";
fn limits(i: u64, d: u64, e: u64, o: u64, w: u64, r: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: i,
        decimal_digits: d,
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
fn observe(name: &str, allowance: ScalarLimits, original: &source::Original) {
    use quire_exact::{
        ChargePoint as P, Incomplete, InexactTarget, Integer, IntegerInterval, Outcome, Refusal,
        RoundingMode, Value,
    };
    let (evaluation, meter) = source::evaluate(original, name, false, allowance);
    let FamilyOutcome::Evaluated(outcome) = &evaluation.outcome else {
        panic!("original value family");
    };
    let graph = original.package.graph();
    let function = graph
        .function_by_identity(graph.function_identity(name).expect("original identity"))
        .expect("original function");
    let c = source::charge;
    let mut events = vec![c(
        P::UnitIdentityRead,
        Some(if name.ends_with("Outside") { 2 } else { 1 }),
        None,
        None,
        Some(1),
    )];
    let totals;
    let mut stop = None;
    let mut expected_value = None;
    match name {
        "inches" | "chain" => {
            let edges = if name == "inches" { 2 } else { 3 };
            let admitted = edges.min(allowance.unit_edges);
            for edge in 1..=admitted {
                events.push(c(P::UnitEdge, None, None, Some(edge), None));
            }
            if admitted < edges {
                stop = Some(Incomplete {
                    limit_kind: LimitKind::UnitEdges,
                    limit: admitted,
                    consumed: admitted,
                    next_charge: Integer::from(admitted + 1),
                    charge_point: P::UnitEdge,
                });
                totals = [1, 0, admitted, 1, admitted + 1, 0];
            } else {
                let amounts: &[u64] = if name == "inches" {
                    &[14, 15, 15, 14]
                } else {
                    &[2, 6, 5, 6, 6, 5]
                };
                for bits in amounts {
                    events.push(c(P::UnitRationalArithmetic, Some(*bits), None, None, None));
                }
                events.push(c(P::UnitTargetDomain, None, None, None, None));
                events.push(c(P::UnitResultRetain, None, None, None, Some(1)));
                totals = if name == "inches" {
                    [15, 0, 2, 1, 9, 1]
                } else {
                    [6, 0, 3, 1, 12, 1]
                };
                expected_value = Some(if name == "inches" {
                    ("cm", 127_i64, 50_i64)
                } else {
                    ("u1", -9, 1)
                });
            }
        }
        "decimalExact" | "integerExact" | "decimalRounded" | "integerRounded" => {
            events.extend([
                c(P::UnitEdge, None, None, Some(1), None),
                c(P::UnitRationalArithmetic, Some(14), None, None, None),
                c(P::UnitRationalArithmetic, Some(15), None, None, None),
            ]);
            if name.ends_with("Exact") {
                let Outcome::Refused(Refusal::InexactConversion { target }) = outcome else {
                    panic!("exact placement must refuse: {outcome:?}");
                };
                match (name, target) {
                    ("decimalExact", InexactTarget::Decimal(target)) => {
                        assert_eq!(target.lower(), &Integer::from(-100_i64));
                        assert_eq!(target.upper(), &Integer::from(100_i64));
                        assert_eq!(
                            (target.min_scale(), target.max_scale(), target.rounding()),
                            (2, 2, RoundingMode::Exact)
                        );
                    }
                    ("integerExact", InexactTarget::Integer(target)) => assert_eq!(
                        **target,
                        IntegerInterval::spanning(Integer::from(-2_i64), Integer::from(2_i64))
                    ),
                    _ => panic!("exact carried target retained: {target:?}"),
                }
                totals = [15, 0, 1, 1, 4, 0];
            } else if allowance.work_units == 4 {
                stop = Some(Incomplete {
                    limit_kind: LimitKind::WorkUnits,
                    limit: 4,
                    consumed: 4,
                    next_charge: Integer::from(1_i64),
                    charge_point: P::UnitTargetDomain,
                });
                totals = [15, 0, 1, 1, 4, 0];
            } else {
                assert_eq!(name, "decimalRounded");
                expected_value = Some(("m", 3, 100));
                events.extend([
                    c(P::UnitTargetDomain, Some(14), Some(5), None, None),
                    c(P::UnitResultRetain, None, None, None, Some(1)),
                ]);
                totals = [15, 5, 1, 1, 6, 1];
            }
        }
        "decimalOutside" | "integerOutside" => {
            if allowance.work_units == 1 {
                stop = Some(Incomplete {
                    limit_kind: LimitKind::WorkUnits,
                    limit: 1,
                    consumed: 1,
                    next_charge: Integer::from(1_i64),
                    charge_point: P::UnitTargetDomain,
                });
                totals = [2, 0, 0, 1, 1, 0];
            } else {
                events.push(c(
                    P::UnitTargetDomain,
                    Some(2),
                    if name == "decimalOutside" {
                        Some(1)
                    } else {
                        None
                    },
                    None,
                    None,
                ));
                match (name, outcome) {
                    (
                        "decimalOutside",
                        Outcome::Refused(Refusal::DecimalOutOfDomain { target }),
                    ) => {
                        assert_eq!(target.lower(), &Integer::from(-2_i64));
                        assert_eq!(target.upper(), &Integer::from(2_i64));
                        assert_eq!(
                            (target.min_scale(), target.max_scale(), target.rounding()),
                            (0, 0, RoundingMode::Exact)
                        );
                    }
                    (
                        "integerOutside",
                        Outcome::Refused(Refusal::IntegerOutOfDomain { target }),
                    ) => assert_eq!(
                        **target,
                        IntegerInterval::spanning(Integer::from(-2_i64), Integer::from(2_i64))
                    ),
                    _ => panic!("domain membership refusal: {outcome:?}"),
                }
                totals = [2, u64::from(name == "decimalOutside"), 0, 1, 2, 0];
            }
        }
        "hugeScale" => {
            stop = Some(Incomplete {
                limit_kind: LimitKind::DecimalDigits,
                limit: 64,
                consumed: 0,
                next_charge: Integer::from(4_294_967_296_u64),
                charge_point: P::UnitTargetDomain,
            });
            totals = [1, 0, 0, 1, 1, 0];
        }
        _ => panic!("unselected conversion {name}"),
    }
    if let Some(expected) = stop {
        let Outcome::Incomplete(actual) = outcome else {
            panic!("required first unpaid event: {outcome:?}");
        };
        assert_eq!(actual, &expected);
    }
    if let Some((unit, n, d)) = expected_value {
        let Outcome::Completed(Value::Quantity(actual)) = outcome else {
            panic!("required quantity completion: {outcome:?}");
        };
        assert_eq!(
            actual.magnitude(),
            &quire_exact::Rational::new(Integer::from(n), Integer::from(d))
                .expect("independent rational")
        );
        assert_eq!(
            actual.unit(),
            &original
                .units
                .declared_unit_id(original.keys[unit].clone())
                .expect("actual target unit")
        );
        assert!(evaluation.location.is_none());
    } else {
        assert_eq!(evaluation.location.as_ref(), Some(function.body.location()));
    }
    if name == "decimalRounded" && allowance.work_units == 6 {
        let [loss] = evaluation.losses.as_slice() else {
            panic!("one rounded carrier loss");
        };
        assert_eq!(&loss.location, function.body.location());
        let quire_semantic_value::loss::ValueLoss::Decimal(decimal) = &loss.loss else {
            panic!("decimal rounding loss");
        };
        assert_eq!(decimal.exact_numerator(), &Integer::from(127_i64));
        assert_eq!(decimal.exact_denominator(), Integer::from(5000_i64));
        assert_eq!(decimal.rounded_coefficient(), &Integer::from(3_i64));
        assert_eq!(decimal.rounded_scale(), 2);
        assert_eq!(decimal.mode(), RoundingMode::NearestEven);
    } else {
        assert!(evaluation.losses.is_empty());
    }
    source::accounting(&meter, &events, totals);
}

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-1",
    "QSpec-FR-142-AC-3",
    "QSpec-FR-142-AC-5",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn original_conversion_boundaries_preserve_values_losses_and_unpaid_events() {
    for (id, input, expectation) in [
        ("U15", "L(15,0,2,1,9,1)", "127/50"),
        ("U16", "L(15,5,1,1,6,1)", "unit.target-domain"),
        ("U17", "L(2,1,0,1,2,0)", "membership"),
        ("U18", "4294967295", "4294967296"),
        ("U26", "1 u2", "-9 u1"),
        ("U29", "[-2,2]", "unit.target-domain"),
    ] {
        source::bind(id, &[input], &[expectation]);
    }
    source::bind(
        "U07",
        &["1 in", "scale two"],
        &[
            "(3,2) m",
            "exact_denominator: 5000",
            "rounded_coefficient: 3",
        ],
    );
    source::bind("U08", &["[-2,2]", "scale zero"], &["refuse atomically"]);
    source::bind(
        "U15",
        &["L(7,0,1,1,9,1)"],
        &["consumed: 1", "next_charge: 2", "unit.edge"],
    );
    source::bind(
        "U16",
        &["exact", "nearest-even", "L(15,5,1,1,4,1)"],
        &["four work units", "integer_bits: 15", "consumed: 4"],
    );
    source::bind(
        "U17",
        &["3 m", "L(2,1,0,1,1,0)"],
        &["before `unit.result-retain`", "consumed: 1"],
    );
    source::bind(
        "U18",
        &["L(18446744073709551615,64,0,1,2,1)"],
        &["consumed: 0", "never materialized"],
    );
    source::bind(
        "U26",
        &["L(6,0,3,1,12,1)", "L(6,0,2,1,12,1)"],
        &["six rational events", "next_charge: 3"],
    );
    source::bind(
        "U29",
        &["exact", "nearest-even", "3 m", "L(2,0,0,1,1,0)"],
        &["rounds to `0`", "integer-domain membership", "consumed: 1"],
    );
    let original = source::original_with(EXTRA, FUNCTIONS);
    for (name, l) in [
        ("inches", limits(15, 0, 2, 1, 9, 1)),
        ("inches", limits(7, 0, 1, 1, 9, 1)),
        ("chain", limits(6, 0, 3, 1, 12, 1)),
        ("chain", limits(6, 0, 2, 1, 12, 1)),
        ("decimalExact", limits(15, 5, 1, 1, 6, 1)),
        ("decimalRounded", limits(15, 5, 1, 1, 6, 1)),
        ("decimalExact", limits(15, 5, 1, 1, 4, 1)),
        ("decimalRounded", limits(15, 5, 1, 1, 4, 1)),
        ("decimalOutside", limits(2, 1, 0, 1, 2, 0)),
        ("decimalOutside", limits(2, 1, 0, 1, 1, 0)),
        ("hugeScale", limits(u64::MAX, 64, 0, 1, 2, 1)),
        ("integerExact", limits(15, 0, 1, 1, 4, 1)),
        ("integerRounded", limits(15, 0, 1, 1, 4, 1)),
        ("integerOutside", limits(2, 0, 0, 1, 2, 0)),
        ("integerOutside", limits(2, 0, 0, 1, 1, 0)),
    ] {
        observe(name, l, &original);
    }

    let exact_functions = FUNCTIONS
        .lines()
        .filter(|line| line.contains("function inches ") || line.contains("function chain "))
        .collect::<Vec<_>>()
        .join("\n");
    let exact = source::original_with(EXTRA, &exact_functions);
    assert_eq!(
        source::generation(
            &exact,
            &[
                ("inches", false, limits(15, 0, 2, 1, 9, 1)),
                ("inches", false, limits(7, 0, 1, 1, 9, 1)),
                ("chain", false, limits(6, 0, 3, 1, 12, 1)),
                ("chain", false, limits(6, 0, 2, 1, 12, 1))
            ],
            &[],
            &[]
        ),
        4
    );
    for name in [
        "decimalExact",
        "decimalRounded",
        "decimalOutside",
        "hugeScale",
        "integerExact",
        "integerRounded",
        "integerOutside",
    ] {
        let function = FUNCTIONS
            .lines()
            .find(|line| line.contains(&format!("function {name} ")))
            .expect("original carried function");
        let candidate = source::original_with(EXTRA, function);
        assert_eq!(
            source::generation_with(
                &candidate,
                &[(name, false, limits(1024, 1024, 4, 2, 32, 1))],
                &[],
                &[],
                &[name]
            ),
            0
        );
    }
}
