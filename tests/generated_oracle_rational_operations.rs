// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original parameterized Rational arithmetic against generated native scalar oracles.

#[path = "support/family_checked_source.rs"]
mod family_checked_source;
#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/family_scalar_limits.rs"]
mod family_scalar_limits;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;

use ix_trace_rs::trace;
use qsl_package::emit_checked;
use quire_contract_codegen::{
    derive_exact_scalar_items, generate_exact_scalar_oracles, ClaimDisposition,
    ExactScalarOperation, OperationProvenance,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use quire_eval::FamilyOutcome;
use quire_exact::{Integer, LimitKind, Meter, Outcome, Rational, Value};

#[derive(Clone, Copy)]
enum Expected {
    Rational(i64, i64),
    Boolean(bool),
}

/// One authored input vector; Integer operands carry a unit denominator.
struct Case {
    arguments: &'static [(i64, i64)],
    expected: Expected,
}

struct Row {
    name: &'static str,
    expression: &'static str,
    parameters: &'static str,
    result: &'static str,
    integer_operands: bool,
    cases: &'static [Case],
}

const RATIONAL_RESULT: &str = "Rational[-200,200;1,200]";

const fn rational(arguments: &'static [(i64, i64)], numerator: i64, denominator: i64) -> Case {
    Case {
        arguments,
        expected: Expected::Rational(numerator, denominator),
    }
}

const fn boolean(arguments: &'static [(i64, i64)], expected: bool) -> Case {
    Case {
        arguments,
        expected: Expected::Boolean(expected),
    }
}

const ROWS: &[Row] = &[
    Row {
        name: "radd",
        expression: "left + right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: RATIONAL_RESULT,
        integer_operands: false,
        cases: &[
            rational(&[(1, 2), (1, 3)], 5, 6),
            rational(&[(9, 1), (9, 1)], 18, 1),
            rational(&[(-9, 1), (-9, 1)], -18, 1),
        ],
    },
    Row {
        name: "rsub",
        expression: "left - right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: RATIONAL_RESULT,
        integer_operands: false,
        cases: &[
            rational(&[(4, 3), (7, 2)], -13, 6),
            rational(&[(9, 1), (-9, 1)], 18, 1),
        ],
    },
    Row {
        name: "rmul",
        expression: "left * right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: RATIONAL_RESULT,
        integer_operands: false,
        cases: &[
            rational(&[(2, 3), (3, 4)], 1, 2),
            rational(&[(9, 1), (9, 1)], 81, 1),
            rational(&[(-9, 1), (9, 1)], -81, 1),
        ],
    },
    Row {
        name: "rneg",
        expression: "-left",
        parameters: "left: Rational[-9,9;1,9]",
        result: RATIONAL_RESULT,
        integer_operands: false,
        cases: &[
            rational(&[(7, 3)], -7, 3),
            rational(&[(-9, 1)], 9, 1),
        ],
    },
    Row {
        name: "rdiv",
        expression: "left / right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[1,9;1,9]",
        result: RATIONAL_RESULT,
        integer_operands: false,
        cases: &[
            rational(&[(1, 2), (3, 4)], 2, 3),
            rational(&[(-9, 1), (1, 9)], -81, 1),
            rational(&[(9, 1), (9, 1)], 1, 1),
        ],
    },
    Row {
        name: "idiv",
        expression: "left / right",
        parameters: "left: Int[-9,9], right: Int[1,9]",
        result: RATIONAL_RESULT,
        integer_operands: true,
        cases: &[
            rational(&[(7, 1), (2, 1)], 7, 2),
            rational(&[(-9, 1), (9, 1)], -1, 1),
            rational(&[(9, 1), (1, 1)], 9, 1),
        ],
    },
    Row {
        name: "rless",
        expression: "left < right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: "Boolean",
        integer_operands: false,
        cases: &[
            boolean(&[(1, 3), (1, 2)], true),
            boolean(&[(1, 2), (1, 2)], false),
            boolean(&[(1, 2), (1, 3)], false),
        ],
    },
    Row {
        name: "rlessOrEqual",
        expression: "left <= right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: "Boolean",
        integer_operands: false,
        cases: &[
            boolean(&[(1, 3), (1, 2)], true),
            boolean(&[(1, 2), (1, 2)], true),
            boolean(&[(1, 2), (1, 3)], false),
        ],
    },
    Row {
        name: "rgreater",
        expression: "left > right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: "Boolean",
        integer_operands: false,
        cases: &[
            boolean(&[(1, 3), (1, 2)], false),
            boolean(&[(1, 2), (1, 2)], false),
            boolean(&[(1, 2), (1, 3)], true),
        ],
    },
    Row {
        name: "rgreaterOrEqual",
        expression: "left >= right",
        parameters: "left: Rational[-9,9;1,9], right: Rational[-9,9;1,9]",
        result: "Boolean",
        integer_operands: false,
        cases: &[
            boolean(&[(1, 3), (1, 2)], false),
            boolean(&[(1, 2), (1, 2)], true),
            boolean(&[(1, 2), (1, 3)], true),
            boolean(&[(9, 1), (-9, 1)], true),
        ],
    },
];

#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn original_rational_arithmetic_agrees_with_generated_native_observations() {
    let mut source = String::from(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n",
    );
    for row in ROWS {
        source.push_str(&format!(
            "function {} using v({}): {} pure {{ {} }}\n",
            row.name, row.parameters, row.result, row.expression
        ));
    }
    let producer = family_checked_source::checked(&source);
    let emission = emit_checked(
        &producer,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("original Rational source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact Rational package admits");
    };
    let nodes = ROWS
        .iter()
        .map(|row| family_function_selection::body(&package, row.name))
        .collect::<Vec<_>>();
    let items = derive_exact_scalar_items(&package, &nodes)
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.unwrap_or_else(|error| {
                panic!("{} original derivation: {error:?}", ROWS[index].name)
            })
        })
        .collect::<Vec<_>>();
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("Rational operation items generate");
    assert_eq!(output.claim_map.items.len(), ROWS.len());
    let authority = family_checked_source::checked(&source);
    let mut expected = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{Integer,Meter,Outcome,Rational,ScalarLimits,Value};
fn lift<T>(outcome: Outcome<T>, value: impl FnOnce(T)->Value)->Outcome<Value> { match outcome { Outcome::Completed(v)=>Outcome::Completed(value(v)), Outcome::Undefined(v)=>Outcome::Undefined(v), Outcome::Refused(v)=>Outcome::Refused(v), Outcome::Incomplete(v)=>Outcome::Incomplete(v) } }
#[test]
fn observe() {
"#,
    );
    let mut labels = Vec::new();
    for (index, row) in ROWS.iter().enumerate() {
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one original Rational claim for {}", row.name);
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        assert!(
            matches!(
                (&items[index].operation, row.cases[0].expected),
                (
                    ExactScalarOperation::RationalArithmetic { .. },
                    Expected::Rational(..)
                ) | (ExactScalarOperation::Ordering { .. }, Expected::Boolean(_))
            ),
            "{} selected operation: {:?}",
            row.name,
            items[index].operation
        );
        let graph = authority.graph();
        let function = graph
            .function_by_identity(graph.function_identity(row.name).expect("source identity"))
            .expect("checked body");
        for case in row.cases {
            for work in [1024, 0, 1] {
                let mut meter = Meter::new(family_scalar_limits::limits(work));
                let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                    &authority,
                    function.body,
                    function.slots,
                    case.arguments
                        .iter()
                        .map(|(numerator, denominator)| {
                            if row.integer_operands {
                                Value::Integer(Integer::from(*numerator))
                            } else {
                                Value::Rational(
                                    Rational::new(
                                        Integer::from(*numerator),
                                        Integer::from(*denominator),
                                    )
                                    .expect("authored rational input"),
                                )
                            }
                        })
                        .collect(),
                    function.formed_units,
                    &mut meter,
                )
                .expect("checked Rational body evaluation");
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("Rational value family");
                };
                if work == 1024 {
                    let required = match case.expected {
                        Expected::Rational(numerator, denominator) => Value::Rational(
                            Rational::new(Integer::from(numerator), Integer::from(denominator))
                                .expect("authored expected rational"),
                        ),
                        Expected::Boolean(value) => Value::Boolean(value),
                    };
                    assert!(
                        matches!(&outcome, Outcome::Completed(actual) if format!("{actual:?}") == format!("{required:?}")),
                        "{} required result: {outcome:?}",
                        row.name
                    );
                    assert!(evaluation.location.is_none());
                } else if work == 0 {
                    assert!(
                        matches!(&outcome, Outcome::Incomplete(stop) if stop.limit_kind == LimitKind::WorkUnits && stop.limit == 0 && stop.consumed == 0),
                        "{} work-zero: {outcome:?}",
                        row.name
                    );
                    assert!(evaluation.location.is_some());
                } else if matches!(&outcome, Outcome::Incomplete(_)) {
                    assert!(evaluation.location.is_some());
                }
                assert!(evaluation.losses.is_empty());
                let losses = evaluation
                    .losses
                    .into_iter()
                    .map(|loss| loss.loss)
                    .collect::<Vec<_>>();
                expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
                labels.push(format!("{} {:?} at work {work}", row.name, case.arguments));
                let arguments = case
                    .arguments
                    .iter()
                    .map(|(numerator, denominator)| {
                        if row.integer_operands {
                            format!("&Integer::from({numerator}_i64)")
                        } else {
                            format!("&Rational::new(Integer::from({numerator}_i64),Integer::from({denominator}_i64)).expect(\"authored rational input\")")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let lift = match case.expected {
                    Expected::Rational(..) => "Value::Rational",
                    Expected::Boolean(_) => "Value::Boolean",
                };
                driver.push_str(&format!(
                    r#"
{{ let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{}({}, &mut meter).expect("Rational operand admission");
let actual=lift(actual,{});
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#,
                    family_scalar_limits::limits(work),
                    generated.symbol,
                    arguments,
                    lift
                ));
            }
        }
    }
    driver.push_str("}\n");
    let actual = generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(actual.len(), expected.len());
    for ((label, actual), expected) in labels.iter().zip(&actual).zip(&expected) {
        assert_eq!(actual, expected, "{label}");
    }
}
