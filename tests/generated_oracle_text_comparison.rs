// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original Text comparisons against generated native scalar oracles.

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
use qsl_package::{emit_checked, EmitLimits};
use quire_contract_codegen::{
    derive_exact_scalar_items, generate_exact_scalar_oracles, ClaimDisposition,
    ExactScalarOperation, OperationProvenance,
};
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use quire_eval::FamilyOutcome;
use quire_exact::{
    admit_text, LimitKind, Meter, Outcome, Text, TextPayload, TextProfile, TextType, Value,
};

struct Row {
    name: &'static str,
    syntax: &'static str,
    left: &'static str,
    right: &'static str,
    expected: Expected,
}

#[derive(Clone, Copy)]
enum Expected {
    Fixed(bool),
    CanonicalEquivalent,
}

impl Expected {
    fn value(self, profile: TextProfile) -> bool {
        match self {
            Self::Fixed(value) => value,
            Self::CanonicalEquivalent => matches!(
                profile,
                TextProfile::Nfc | TextProfile::Nfd | TextProfile::Nfkc | TextProfile::Nfkd
            ),
        }
    }
}

struct Case<'a> {
    name: String,
    profile: TextProfile,
    row: &'a Row,
}

const ROWS: &[Row] = &[
    Row {
        name: "equal",
        syntax: "=",
        left: "e\u{301}",
        right: "é",
        expected: Expected::CanonicalEquivalent,
    },
    Row {
        name: "unequal",
        syntax: "!=",
        left: "a",
        right: "b",
        expected: Expected::Fixed(true),
    },
    Row {
        name: "less",
        syntax: "<",
        left: "a",
        right: "b",
        expected: Expected::Fixed(true),
    },
    Row {
        name: "greater",
        syntax: ">",
        left: "a",
        right: "b",
        expected: Expected::Fixed(false),
    },
    Row {
        name: "less_or_equal",
        syntax: "<=",
        left: "a",
        right: "a",
        expected: Expected::Fixed(true),
    },
    Row {
        name: "greater_or_equal",
        syntax: ">=",
        left: "a",
        right: "b",
        expected: Expected::Fixed(false),
    },
];

fn text(value: &str, profile: TextProfile) -> Text {
    admit_text(
        &TextPayload::from_utf8(value.as_bytes()).expect("authored UTF-8"),
        &TextType::new(0, 16, profile).expect("authored text bounds"),
        &mut Meter::new(family_scalar_limits::limits(1024)),
    )
    .completed()
    .expect("authored operand admits independently of comparison")
}

/// Trace: TC-102, IT-019-SC-05.
#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn original_text_comparisons_agree_with_generated_native_observations() {
    let cases = TextProfile::ALL
        .into_iter()
        .enumerate()
        .flat_map(|(profile_index, profile)| {
            ROWS.iter().map(move |row| Case {
                name: format!("text{profile_index}_{}", row.name),
                profile,
                row,
            })
        })
        .collect::<Vec<_>>();
    let mut source = String::from(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n",
    );
    for case in &cases {
        source.push_str(&format!(
            "function {} using v(left: Text[0,16;{}], right: Text[0,16;{}]): Boolean pure {{ left {} right }}\n",
            case.name, case.profile.as_str(), case.profile.as_str(), case.row.syntax,
        ));
    }
    let producer = family_checked_source::checked(&source);
    let emission = emit_checked(
        &producer,
        EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
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
    let nodes = cases
        .iter()
        .map(|case| family_function_selection::body(&package, &case.name))
        .collect::<Vec<_>>();
    let items = derive_exact_scalar_items(&package, &nodes)
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.unwrap_or_else(|error| {
                panic!("{} genuine source derivation: {error:?}", cases[index].name)
            })
        })
        .collect::<Vec<_>>();
    let output = generate_exact_scalar_oracles(&package, &items)
        .expect("original text comparisons generate");
    assert_eq!(output.claim_map.items.len(), cases.len());
    let authority = family_checked_source::checked(&source);
    let mut expected = Vec::new();
    let mut driver = String::from(
        r#"
mod oracle_observation;
use quire_exact::{admit_text,Meter,Outcome,ScalarLimits,TextPayload,TextProfile,TextType,Value};
fn text(value: &str, profile: TextProfile) -> quire_exact::Text {
    admit_text(&TextPayload::from_utf8(value.as_bytes()).expect("authored UTF-8"),
        &TextType::new(0,16,profile).expect("authored text bounds"),
        &mut Meter::new(ScalarLimits { integer_bits:1024,decimal_digits:1024,scale_expansion:1024,text_input_bytes:1024,text_scalars:1024,normalized_scalars:1024,unit_edges:1024,value_occurrences:1024,work_units:1024,result_units:1024 }))
    .completed().expect("authored operand admits independently of comparison")
}

fn lift(outcome: Outcome<bool>)->Outcome<Value> { match outcome { Outcome::Completed(v)=>Outcome::Completed(Value::Boolean(v)),Outcome::Undefined(v)=>Outcome::Undefined(v),Outcome::Refused(v)=>Outcome::Refused(v),Outcome::Incomplete(v)=>Outcome::Incomplete(v) } }
#[test]
fn observe() {
"#,
    );
    for (index, case) in cases.iter().enumerate() {
        let row = case.row;
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[index])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one original text claim for {}", case.name);
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        assert!(
            matches!(
                &items[index].operation,
                ExactScalarOperation::TextComparison { .. }
            ),
            "{} selected operation: {:?}",
            case.name,
            items[index].operation
        );
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", case.name, claim.result);
        };
        let identity = authority
            .graph()
            .function_identity(&case.name)
            .expect("authored function identity");
        let function = authority
            .graph()
            .function_by_identity(identity)
            .expect("actual checked body");
        let left = text(row.left, case.profile);
        let right = text(row.right, case.profile);
        for work in [1024, 0, 1] {
            let mut meter = Meter::new(family_scalar_limits::limits(work));
            let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                &authority,
                function.body,
                function.slots,
                vec![Value::Text(left.clone()), Value::Text(right.clone())],
                function.formed_units,
                &mut meter,
            )
            .expect("public checked body evaluation");
            let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                panic!("selected text value family");
            };
            if work == 1024 {
                assert!(
                    matches!(&outcome, Outcome::Completed(Value::Boolean(value))
                        if *value == row.expected.value(case.profile)),
                    "{} required typed result: {outcome:?}",
                    case.name
                );
                assert!(evaluation.location.is_none());
            } else if work == 0 {
                assert!(
                    matches!(&outcome, Outcome::Incomplete(stop)
                        if stop.limit_kind == LimitKind::WorkUnits
                            && stop.limit == 0
                            && stop.consumed == 0),
                    "{} required work-zero stop: {outcome:?}",
                    case.name
                );
                assert!(evaluation.location.is_some());
            } else if matches!(&outcome, Outcome::Incomplete(_)) {
                assert!(evaluation.location.is_some());
            }
            assert!(
                evaluation.losses.is_empty(),
                "{}: {:?}",
                case.name,
                evaluation.losses
            );
            let losses = evaluation
                .losses
                .into_iter()
                .map(|loss| loss.loss)
                .collect::<Vec<_>>();
            expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
            driver.push_str(&format!(r#"
{{ let left=text({:?},TextProfile::{:?}); let right=text({:?},TextProfile::{:?}); let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{}(&left,&right,&mut meter).expect("original text operand admission");
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&lift(actual),&losses,&meter)); }}
"#, row.left, case.profile, row.right, case.profile, family_scalar_limits::limits(work), generated.symbol));
        }
    }
    driver.push_str("}\n");
    let actual = generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(actual.len(), expected.len());
    for ((case, work), (actual, expected)) in cases
        .iter()
        .flat_map(|case| [1024, 0, 1].into_iter().map(move |work| (case, work)))
        .zip(actual.iter().zip(&expected))
    {
        assert_eq!(actual, expected, "{} at work {work}", case.name);
    }
}
