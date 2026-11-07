// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original parameterized Decimal operations against generated native scalar oracles.

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
use quire_exact::{Decimal, Integer, LimitKind, Meter, Outcome, Value};
use quire_semantic_value::loss::ValueLoss;

/// What the work-1024 observation must be.
#[derive(Clone, Copy)]
enum Expected {
    Decimal(i64, u32),
    Boolean(bool),
    Refused,
}

/// One authored input vector as `(coefficient, scale)` operands.
struct Case {
    arguments: &'static [(i64, u32)],
    expected: Expected,
}

struct Row {
    name: &'static str,
    expression: &'static str,
    parameters: &'static str,
    result: &'static str,
    cases: &'static [Case],
}

const fn value(arguments: &'static [(i64, u32)], coefficient: i64, scale: u32) -> Case {
    Case {
        arguments,
        expected: Expected::Decimal(coefficient, scale),
    }
}

const fn boolean(arguments: &'static [(i64, u32)], expected: bool) -> Case {
    Case {
        arguments,
        expected: Expected::Boolean(expected),
    }
}

const fn refused(arguments: &'static [(i64, u32)]) -> Case {
    Case {
        arguments,
        expected: Expected::Refused,
    }
}

const WIDE: &str = "Decimal[-1000,1000;0,2;exact]";
const PAIR: &str = "left: Decimal[-1000,1000;0,2;exact], right: Decimal[-1000,1000;0,2;exact]";
const ROUND_INPUT: &str = "operand: Decimal[-25,25;1,1;exact]";
const ROUND_CASES: &[Case] = &[
    value(&[(25, 1)], 3, 0),
    value(&[(-25, 1)], -3, 0),
];

const ROWS: &[Row] = &[
    Row {
        name: "dadd",
        expression: "left + right",
        parameters: PAIR,
        result: WIDE,
        cases: &[
            value(&[(150, 2), (225, 2)], 375, 2),
            value(&[(99999, 2), (1, 2)], 100000, 2),
            refused(&[(99999, 2), (2, 2)]),
            refused(&[(-99999, 2), (-2, 2)]),
        ],
    },
    Row {
        name: "dsub",
        expression: "left - right",
        parameters: PAIR,
        result: WIDE,
        cases: &[
            value(&[(150, 2), (225, 2)], -75, 2),
            value(&[(-99999, 2), (1, 2)], -100000, 2),
            refused(&[(-99999, 2), (2, 2)]),
        ],
    },
    Row {
        name: "dmul",
        expression: "left * right",
        parameters: PAIR,
        result: WIDE,
        cases: &[
            value(&[(150, 2), (200, 2)], 300, 2),
            value(&[(1000, 0), (1, 0)], 100000, 2),
            refused(&[(1000, 0), (2, 0)]),
        ],
    },
    Row {
        name: "dneg",
        expression: "-operand",
        parameters: "operand: Decimal[-1000,1000;0,2;exact]",
        result: WIDE,
        cases: &[
            value(&[(150, 2)], -150, 2),
            value(&[(-100000, 2)], 100000, 2),
        ],
    },
    Row {
        name: "ddivRounded",
        expression: "decimal(1200,3) / decimal(7,0)",
        parameters: "",
        result: "Decimal[-1000,1000;0,1;nearest-even]",
        cases: &[value(&[(1200, 3), (7, 0)], 2, 1)],
    },
    Row {
        name: "ddivExact",
        expression: "decimal(1,0) / decimal(4,0)",
        parameters: "",
        result: "Decimal[-1000,1000;0,2;nearest-even]",
        cases: &[value(&[(1, 0), (4, 0)], 25, 2)],
    },
    Row {
        name: "ddivOutOfDomain",
        expression: "decimal(1000,0) / decimal(1,3)",
        parameters: "",
        result: "Decimal[-1000,1000;0,3;nearest-even]",
        cases: &[refused(&[(1000, 0), (1, 3)])],
    },
    Row {
        name: "dless",
        expression: "left < right",
        parameters: PAIR,
        result: "Boolean",
        cases: &[
            boolean(&[(15, 1), (150, 2)], false),
            boolean(&[(149, 2), (15, 1)], true),
            boolean(&[(150, 2), (149, 2)], false),
        ],
    },
    Row {
        name: "dlessOrEqual",
        expression: "left <= right",
        parameters: PAIR,
        result: "Boolean",
        cases: &[
            boolean(&[(15, 1), (150, 2)], true),
            boolean(&[(149, 2), (15, 1)], true),
            boolean(&[(150, 2), (149, 2)], false),
        ],
    },
    Row {
        name: "dgreater",
        expression: "left > right",
        parameters: PAIR,
        result: "Boolean",
        cases: &[
            boolean(&[(15, 1), (150, 2)], false),
            boolean(&[(149, 2), (15, 1)], false),
            boolean(&[(150, 2), (149, 2)], true),
        ],
    },
    Row {
        name: "dgreaterOrEqual",
        expression: "left >= right",
        parameters: PAIR,
        result: "Boolean",
        cases: &[
            boolean(&[(15, 1), (150, 2)], true),
            boolean(&[(149, 2), (15, 1)], false),
            boolean(&[(150, 2), (149, 2)], true),
        ],
    },
    Row {
        name: "roundTowardZero",
        expression: "convert<Decimal[-9,9;0,0;toward-zero]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;toward-zero]",
        cases: &[value(&[(25, 1)], 2, 0), value(&[(-25, 1)], -2, 0)],
    },
    Row {
        name: "roundTowardPositive",
        expression: "convert<Decimal[-9,9;0,0;toward-positive]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;toward-positive]",
        cases: &[value(&[(25, 1)], 3, 0), value(&[(-25, 1)], -2, 0)],
    },
    Row {
        name: "roundTowardNegative",
        expression: "convert<Decimal[-9,9;0,0;toward-negative]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;toward-negative]",
        cases: &[value(&[(25, 1)], 2, 0), value(&[(-25, 1)], -3, 0)],
    },
    Row {
        name: "roundNearestEven",
        expression: "convert<Decimal[-9,9;0,0;nearest-even]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;nearest-even]",
        cases: &[value(&[(25, 1)], 2, 0), value(&[(-25, 1)], -2, 0)],
    },
    Row {
        name: "roundNearestAway",
        expression: "convert<Decimal[-9,9;0,0;nearest-away]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;nearest-away]",
        cases: ROUND_CASES,
    },
    Row {
        name: "roundExact",
        expression: "convert<Decimal[-9,9;0,0;exact]>(operand)",
        parameters: ROUND_INPUT,
        result: "Decimal[-9,9;0,0;exact]",
        cases: &[refused(&[(25, 1)]), refused(&[(-25, 1)])],
    },
];

fn decimal(coefficient: i64, scale: u32) -> Decimal {
    Decimal::new(Integer::from(coefficient), scale)
}

#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn original_decimal_operations_agree_with_generated_native_observations() {
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
    .expect("original Decimal source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact Decimal package admits");
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
        generate_exact_scalar_oracles(&package, &items).expect("Decimal operation items generate");
    assert_eq!(output.claim_map.items.len(), ROWS.len());
    let authority = family_checked_source::checked(&source);
    let mut expected = Vec::new();
    let mut labels = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{Decimal,Integer,Meter,Outcome,ScalarLimits,Value};
use quire_semantic_value::loss::ValueLoss;
fn decimal_observation(outcome: Outcome<quire_exact::DecimalResult>)->(Outcome<Value>,Vec<ValueLoss>) {
    match outcome {
        Outcome::Completed(result)=>{ let losses=result.loss().cloned().map(ValueLoss::Decimal).into_iter().collect(); (Outcome::Completed(Value::Decimal(result.value().clone())),losses) }
        Outcome::Undefined(v)=>(Outcome::Undefined(v),Vec::new()),
        Outcome::Refused(v)=>(Outcome::Refused(v),Vec::new()),
        Outcome::Incomplete(v)=>(Outcome::Incomplete(v),Vec::new()),
    }
}
fn boolean_observation(outcome: Outcome<bool>)->(Outcome<Value>,Vec<ValueLoss>) {
    match outcome {
        Outcome::Completed(v)=>(Outcome::Completed(Value::Boolean(v)),Vec::new()),
        Outcome::Undefined(v)=>(Outcome::Undefined(v),Vec::new()),
        Outcome::Refused(v)=>(Outcome::Refused(v),Vec::new()),
        Outcome::Incomplete(v)=>(Outcome::Incomplete(v),Vec::new()),
    }
}
#[test]
fn observe() {
"#,
    );
    for (index, row) in ROWS.iter().enumerate() {
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one original Decimal claim for {}", row.name);
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        assert!(
            matches!(
                (&items[index].operation, row.cases[0].expected),
                (
                    ExactScalarOperation::DecimalArithmetic { .. },
                    Expected::Decimal(..) | Expected::Refused
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
            for work in [1024, 0, 1, 2, 3] {
                let mut meter = Meter::new(family_scalar_limits::limits(work));
                let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                    &authority,
                    function.body,
                    function.slots,
                    if row.parameters.is_empty() {
                        Vec::new()
                    } else {
                        case.arguments
                            .iter()
                            .map(|(coefficient, scale)| {
                                Value::Decimal(decimal(*coefficient, *scale))
                            })
                            .collect()
                    },
                    function.formed_units,
                    &mut meter,
                )
                .expect("checked Decimal body evaluation");
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("Decimal value family");
                };
                if work == 1024 {
                    match (&outcome, case.expected) {
                        (Outcome::Completed(Value::Decimal(actual)), Expected::Decimal(c, s)) => {
                            assert_eq!(
                                actual.representation(),
                                decimal(c, s).representation(),
                                "{} {:?}",
                                row.name,
                                case.arguments
                            );
                        }
                        (Outcome::Completed(Value::Boolean(actual)), Expected::Boolean(b)) => {
                            assert_eq!(*actual, b, "{} {:?}", row.name, case.arguments);
                        }
                        (Outcome::Refused(_), Expected::Refused) => {}
                        _ => panic!("{} {:?} required result: {outcome:?}", row.name, case.arguments),
                    }
                } else if work == 0 {
                    assert!(
                        matches!(&outcome, Outcome::Incomplete(stop) if stop.limit_kind == LimitKind::WorkUnits && stop.limit == 0 && stop.consumed == 0),
                        "{} work-zero: {outcome:?}",
                        row.name
                    );
                    assert!(evaluation.location.is_some());
                }
                let losses = evaluation
                    .losses
                    .into_iter()
                    .map(|loss| loss.loss)
                    .collect::<Vec<ValueLoss>>();
                expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
                labels.push(format!("{} {:?} at work {work}", row.name, case.arguments));
                let arguments = case
                    .arguments
                    .iter()
                    .map(|(coefficient, scale)| {
                        format!("&Decimal::new(Integer::from({coefficient}_i64),{scale})")
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let observe = match case.expected {
                    Expected::Boolean(_) => "boolean_observation",
                    Expected::Decimal(..) | Expected::Refused => "decimal_observation",
                };
                driver.push_str(&format!(
                    r#"
{{ let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{}({}, &mut meter).expect("Decimal operand admission");
let (actual,losses)={observe}(actual);
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#,
                    family_scalar_limits::limits(work),
                    generated.symbol,
                    arguments,
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
