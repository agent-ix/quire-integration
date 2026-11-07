// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original parameterized Integer operations compared with generated native scalar oracles.

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
use quire_exact::{Integer, LimitKind, Meter, Outcome, Value};

#[derive(Clone, Copy)]
enum ResultKind {
    Integer(i64),
    Boolean(bool),
}

struct Row {
    name: &'static str,
    expression: &'static str,
    parameters: &'static str,
    arguments: &'static [i64],
    expected: ResultKind,
}

const ROWS: &[Row] = &[
    Row {
        name: "addition",
        expression: "left + right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 1],
        expected: ResultKind::Integer(5),
    },
    Row {
        name: "subtraction",
        expression: "left - right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 7],
        expected: ResultKind::Integer(-3),
    },
    Row {
        name: "multiplication",
        expression: "left * right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 2],
        expected: ResultKind::Integer(8),
    },
    Row {
        name: "modulo",
        expression: "left mod right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[-7, 3],
        expected: ResultKind::Integer(2),
    },
    Row {
        name: "negation",
        expression: "-left",
        parameters: "left: Int[-9,9]",
        arguments: &[7],
        expected: ResultKind::Integer(-7),
    },
    Row {
        name: "less",
        expression: "left < right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 7],
        expected: ResultKind::Boolean(true),
    },
    Row {
        name: "greater",
        expression: "left > right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 7],
        expected: ResultKind::Boolean(false),
    },
    Row {
        name: "equal",
        expression: "left = right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 4],
        expected: ResultKind::Boolean(true),
    },
    Row {
        name: "unequal",
        expression: "left != right",
        parameters: "left: Int[-9,9], right: Int[-9,9]",
        arguments: &[4, 7],
        expected: ResultKind::Boolean(true),
    },
];

fn input_cases(row: &Row) -> Vec<(&[i64], ResultKind)> {
    let mut cases = vec![(row.arguments, row.expected)];
    if row.name == "multiplication" {
        cases.push((&[4, 7], ResultKind::Integer(28)));
    }
    cases
}

/// Trace: TC-102, IT-019-SC-05.
#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn original_integer_operations_agree_with_generated_native_observations() {
    let mut source = String::from(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n",
    );
    for row in ROWS {
        let result_type = match row.expected {
            ResultKind::Integer(_) => "Integer",
            ResultKind::Boolean(_) => "Boolean",
        };
        source.push_str(&format!(
            "function {} using v({}): {result_type} pure {{ {} }}\n",
            row.name, row.parameters, row.expression
        ));
    }
    let producer = family_checked_source::checked(&source);
    let emission = emit_checked(
        &producer,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("original checked source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact emitted package admits");
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
                panic!("{} genuine source derivation: {error:?}", ROWS[index].name)
            })
        })
        .collect::<Vec<_>>();
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("original operation items generate");
    assert_eq!(output.claim_map.items.len(), ROWS.len());
    let authority = family_checked_source::checked(&source);
    let mut expected = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{Integer,Meter,Outcome,ScalarLimits,Value};
fn lift<T>(outcome: Outcome<T>, value: impl FnOnce(T)->Value)->Outcome<Value> {
    match outcome { Outcome::Completed(v)=>Outcome::Completed(value(v)), Outcome::Undefined(v)=>Outcome::Undefined(v), Outcome::Refused(v)=>Outcome::Refused(v), Outcome::Incomplete(v)=>Outcome::Incomplete(v) }
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
            panic!("one original claim for {}", row.name);
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        assert!(
            matches!(
                (&items[index].operation, row.expected),
                (
                    ExactScalarOperation::IntegerArithmetic { .. },
                    ResultKind::Integer(_)
                ) | (
                    ExactScalarOperation::IntegerModulo { .. },
                    ResultKind::Integer(_)
                ) | (
                    ExactScalarOperation::Ordering { .. } | ExactScalarOperation::Equality { .. },
                    ResultKind::Boolean(_)
                )
            ),
            "{} selected operation: {:?}",
            row.name,
            items[index].operation
        );
        let identity = authority
            .graph()
            .function_identity(row.name)
            .expect("authored function identity");
        let function = authority
            .graph()
            .function_by_identity(identity)
            .expect("actual checked body");
        for (arguments, expected_case) in input_cases(row) {
            for work in [1024, 0, 1] {
                let mut meter = Meter::new(family_scalar_limits::limits(work));
                let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                    &authority,
                    function.body,
                    function.slots,
                    arguments
                        .iter()
                        .map(|value| Value::Integer(Integer::from(*value)))
                        .collect(),
                    function.formed_units,
                    &mut meter,
                )
                .expect("public checked body evaluation");
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("selected scalar value family");
                };
                if work == 1024 {
                    match (&outcome, expected_case) {
                        (
                            Outcome::Completed(Value::Integer(actual)),
                            ResultKind::Integer(expected),
                        ) => {
                            assert_eq!(actual, &Integer::from(expected), "{}", row.name)
                        }
                        (
                            Outcome::Completed(Value::Boolean(actual)),
                            ResultKind::Boolean(expected),
                        ) => {
                            assert_eq!(*actual, expected, "{}", row.name)
                        }
                        _ => panic!("{} required typed completion: {outcome:?}", row.name),
                    }
                    assert!(evaluation.location.is_none());
                } else if work == 0 {
                    assert!(
                        matches!(&outcome, Outcome::Incomplete(stop)
                        if stop.limit_kind == LimitKind::WorkUnits
                            && stop.limit == 0
                            && stop.consumed == 0),
                        "{} required work-zero stop: {outcome:?}",
                        row.name
                    );
                    assert!(evaluation.location.is_some());
                } else if matches!(&outcome, Outcome::Incomplete(_)) {
                    assert!(
                        evaluation.location.is_some(),
                        "{} stopped work {work} without original source locus",
                        row.name
                    );
                }
                assert!(
                    evaluation.losses.is_empty(),
                    "{}: {:?}",
                    row.name,
                    evaluation.losses
                );
                let losses = evaluation
                    .losses
                    .into_iter()
                    .map(|loss| loss.loss)
                    .collect::<Vec<_>>();
                expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
                let lift = match expected_case {
                    ResultKind::Integer(_) => "Value::Integer",
                    ResultKind::Boolean(_) => "Value::Boolean",
                };
                let arguments = arguments
                    .iter()
                    .map(|value| format!("&Integer::from({value}_i64)"))
                    .collect::<Vec<_>>()
                    .join(", ");
                driver.push_str(&format!(r#"
{{ let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{}({}, &mut meter).expect("original scalar operand admission");
let actual=lift(actual,{});
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#, family_scalar_limits::limits(work), generated.symbol, arguments, lift));
            }
        }
    }
    driver.push_str("}\n");
    let actual = generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(actual.len(), expected.len());
    for ((row, arguments, work), (actual, expected)) in ROWS
        .iter()
        .flat_map(|row| {
            input_cases(row)
                .into_iter()
                .flat_map(move |(arguments, _)| {
                    [1024, 0, 1]
                        .into_iter()
                        .map(move |work| (row, arguments, work))
                })
        })
        .zip(actual.iter().zip(&expected))
    {
        assert_eq!(
            actual, expected,
            "{} {arguments:?} at work {work}",
            row.name
        );
    }
}
