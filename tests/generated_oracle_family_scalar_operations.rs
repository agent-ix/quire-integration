// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected scalar operation rows, independently checked and executed natively.

#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;

use ix_trace_rs::trace;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_contract_codegen::{
    derive_exact_scalar_items, generate_exact_scalar_oracles, ClaimDisposition,
    ExactScalarOperation, OperationProvenance,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use quire_eval::FamilyOutcome;
use quire_exact::{ChargePoint, Integer, LimitKind, Meter, Outcome, ScalarLimits, Value};
use quire_semantic_value::checking::CheckingLimits;

struct Row {
    name: String,
    syntax: &'static str,
    unary: bool,
    boolean_result: bool,
    cases: Vec<(&'static str, &'static str, Value)>,
}

fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    for (name, syntax, answers) in [
        ("band", "and", [false, false, false, true]),
        ("bor", "or", [false, true, true, true]),
        ("bimplies", "implies", [true, true, false, true]),
        ("beq", "=", [true, false, false, true]),
        ("bne", "!=", [false, true, true, false]),
    ] {
        rows.push(Row {
            name: name.to_owned(),
            syntax,
            unary: false,
            boolean_result: true,
            cases: [
                ("false", "false"),
                ("false", "true"),
                ("true", "false"),
                ("true", "true"),
            ]
            .into_iter()
            .zip(answers)
            .map(|((a, b), value)| (a, b, Value::Boolean(value)))
            .collect(),
        });
    }
    rows.push(Row {
        name: "bnot".to_owned(),
        syntax: "not",
        unary: true,
        boolean_result: true,
        cases: vec![
            ("false", "false", Value::Boolean(true)),
            ("true", "false", Value::Boolean(false)),
        ],
    });
    rows.into_iter()
        .flat_map(|row| {
            row.cases
                .into_iter()
                .enumerate()
                .map(|(index, case)| Row {
                    name: format!("{}truth{index}", row.name),
                    syntax: row.syntax,
                    unary: row.unary,
                    boolean_result: true,
                    cases: vec![case],
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn source(rows: &[Row]) -> String {
    let mut source =
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n"
            .to_owned();
    for row in rows {
        let result = "Boolean";
        let expression = |a: &str, b: &str| {
            if row.unary {
                format!("{} {a}", row.syntax)
            } else {
                format!("{a} {} {b}", row.syntax)
            }
        };
        source.push_str(&format!(
            "function {} using v(): {result} pure {{ {} }}\n",
            row.name,
            expression(row.cases[0].0, row.cases[0].1)
        ));
    }
    source
}

fn checked(source: &str) -> CheckedPackage {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "scalar-operations"),
        "scalar-operations.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original source forms");
    CheckedPackage::link(
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "agent-ix/scalar-operations".to_owned(),
            unit,
            SelectedModels::default(),
            Vec::new(),
        )
        .expect("original source assembles")
        .check(CheckingLimits::default())
        .expect("original source checks"),
    )
}

fn limits(work: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: work,
        result_units: 1024,
    }
}

#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn selected_scalar_operations_match_independently_checked_qsl_bodies() {
    let rows = rows();
    let source = source(&rows);
    let producer = checked(&source);
    let emission = emit_checked(&producer).expect("original checked source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact emitted V2 admits");
    };
    let nodes = rows
        .iter()
        .map(|row| family_function_selection::body(&package, &row.name))
        .collect::<Vec<_>>();
    let items = derive_exact_scalar_items(&package, &nodes)
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.unwrap_or_else(|error| {
                panic!("{} genuine source derivation: {error:?}", rows[index].name)
            })
        })
        .collect::<Vec<_>>();
    for item in &items {
        assert!(
            matches!(
                item.operation,
                ExactScalarOperation::Boolean {
                    right_may_stop: false,
                    ..
                } | ExactScalarOperation::Equality {
                    operands: quire_contract_codegen::EqualityOperandKind::Boolean,
                    ..
                }
            ),
            "only selected eager Boolean rows: {:?}",
            item.operation
        );
    }
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("original operation items generate");
    assert_eq!(output.claim_map.items.len(), rows.len());
    let authority = checked(&source);
    let mut expected = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{Meter,Outcome,ScalarLimits,Value};
fn lift<T>(outcome: Outcome<T>, value: impl FnOnce(T)->Value)->Outcome<Value> {
    match outcome { Outcome::Completed(v)=>Outcome::Completed(value(v)), Outcome::Undefined(v)=>Outcome::Undefined(v), Outcome::Refused(v)=>Outcome::Refused(v), Outcome::Incomplete(v)=>Outcome::Incomplete(v) }
}
#[test]
fn observe() {
"#,
    );
    for (index, row) in rows.iter().enumerate() {
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one selected original claim");
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        for (left, right, value) in &row.cases {
            let name = row.name.clone();
            let function = authority
                .graph()
                .function_by_identity(
                    authority
                        .graph()
                        .function_identity(&name)
                        .expect("authored function identity"),
                )
                .expect("actual checked function body");
            for work in [1024, 0, 1] {
                let mut meter = Meter::new(limits(work));
                let evaluation = qsl_eval::value::evaluate_closed(
                    &authority,
                    function.body,
                    function.slots,
                    &mut meter,
                )
                .expect("public checked body evaluation");
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("selected scalar value family");
                };
                let equality = matches!(
                    items[index].operation,
                    ExactScalarOperation::Equality { .. }
                );
                if work == 1024 || (work == 1 && !equality) {
                    match (&outcome, value) {
                        (Outcome::Completed(Value::Boolean(actual)), Value::Boolean(expected)) => {
                            assert_eq!(actual, expected, "{name}")
                        }
                        (Outcome::Completed(Value::Integer(actual)), Value::Integer(expected)) => {
                            assert_eq!(actual, expected, "{name}")
                        }
                        _ => panic!("{name} required typed completion: {outcome:?}"),
                    }
                    assert!(evaluation.location.is_none());
                }
                if work == 0 || (work == 1 && equality) {
                    let point = if equality {
                        ChargePoint::EqualityPlanForm
                    } else {
                        ChargePoint::BooleanResultRetain
                    };
                    let next = Integer::from(if equality { 2_u64 } else { 1_u64 });
                    assert!(
                        matches!(&outcome,Outcome::Incomplete(stop)
                        if stop.limit_kind==LimitKind::WorkUnits && stop.limit==work
                        && stop.consumed==0 && stop.next_charge==next && stop.charge_point==point),
                        "{name}: {outcome:?}"
                    );
                    assert!(meter.admitted_charges().is_empty());
                    for kind in LimitKind::ALL {
                        assert_eq!(meter.consumed(kind), 0, "{name}: {kind:?}");
                    }
                    assert!(evaluation.location.is_some());
                }
                assert!(evaluation.losses.is_empty());
                let losses = evaluation
                    .losses
                    .into_iter()
                    .map(|loss| loss.loss)
                    .collect::<Vec<_>>();
                expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
                let argument = |text: &str| text.to_owned();
                let arguments = if row.unary {
                    argument(left)
                } else if matches!(
                    items[index].operation,
                    ExactScalarOperation::Boolean {
                        right_may_stop: true,
                        ..
                    }
                ) {
                    format!("{}, || Outcome::Completed({right})", argument(left))
                } else {
                    format!("{}, {}", argument(left), argument(right))
                };
                let constructor = if row.boolean_result {
                    "Boolean"
                } else {
                    "Integer"
                };
                driver.push_str(&format!(r#"
{{ let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{}({}, &mut meter).expect("original scalar operand admission");
let actual=lift(actual,Value::{});
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#,limits(work),generated.symbol,arguments,constructor));
            }
        }
    }
    driver.push_str("}\n");
    assert_eq!(
        generated_oracle::execute(&output.artifacts, &driver),
        expected
    );
}
