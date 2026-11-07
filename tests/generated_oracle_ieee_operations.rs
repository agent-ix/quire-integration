// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original IEEE arithmetic and width conversion against generated native scalar oracles.

#[path = "support/family_checked_ieee_source.rs"]
mod family_checked_ieee_source;
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
use quire_exact::{IeeeValue, IeeeWidth, LimitKind, Meter, Outcome, RoundingMode, Value};
use quire_semantic_value::loss::ValueLoss;

#[derive(Clone, Copy)]
enum Width {
    Single,
    Double,
}

impl Width {
    fn name(self) -> &'static str {
        match self {
            Self::Single => "Float32",
            Self::Double => "Float64",
        }
    }

    fn value(self, single: u32, double: u64) -> (String, IeeeValue) {
        match self {
            Self::Single => (
                format!("IeeeValue::binary32({single}_u32)"),
                IeeeValue::binary32(single),
            ),
            Self::Double => (
                format!("IeeeValue::binary64({double}_u64)"),
                IeeeValue::binary64(double),
            ),
        }
    }
}

/// Bit patterns authored for both widths: ordinary, inexact, overflowing, subnormal,
/// zero and not-a-number operands.
const OPERANDS: [(&str, u32, u64); 8] = [
    ("one", 0x3f80_0000, 0x3ff0_0000_0000_0000),
    ("three", 0x4040_0000, 0x4008_0000_0000_0000),
    ("tenth", 0x3dcc_cccd, 0x3fb9_9999_9999_999a),
    ("maximum", 0x7f7f_ffff, 0x7fef_ffff_ffff_ffff),
    ("tiny", 0x0000_0001, 0x0000_0000_0000_0001),
    ("zero", 0x0000_0000, 0x0000_0000_0000_0000),
    ("infinity", 0x7f80_0000, 0x7ff0_0000_0000_0000),
    ("nan", 0x7fc0_0000, 0x7ff8_0000_0000_0000),
];

const PAIRS: [(usize, usize); 9] = [
    (0, 1),
    (2, 1),
    (3, 3),
    (3, 2),
    (4, 1),
    (0, 5),
    (5, 5),
    (6, 6),
    (7, 0),
];

const OPERATORS: [(&str, &str); 4] = [
    ("add", "+"),
    ("subtract", "-"),
    ("multiply", "*"),
    ("divide", "/"),
];

struct Arithmetic {
    name: String,
    width: Width,
    mode: RoundingMode,
    operator: &'static str,
}

struct Conversion {
    name: String,
    source: Width,
    mode: RoundingMode,
}

fn mode_suffix(mode: RoundingMode) -> String {
    mode.as_str().replace('-', "_")
}

#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn original_ieee_arithmetic_and_width_conversion_agree_with_generated_native_observations() {
    let mut arithmetic = Vec::new();
    let mut conversions = Vec::new();
    let mut source = String::from(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n",
    );
    for width in [Width::Single, Width::Double] {
        for mode in RoundingMode::ALL {
            for (operation, symbol) in OPERATORS {
                let name = format!("{operation}_{}_{}", width.name(), mode_suffix(mode));
                let ty = format!("{}[{}]", width.name(), mode.as_str());
                source.push_str(&format!(
                    "function {name} using v(a: {ty}, b: {ty}): {ty} pure {{ a {symbol} b }}\n"
                ));
                arithmetic.push(Arithmetic {
                    name,
                    width,
                    mode,
                    operator: operation,
                });
            }
        }
    }
    for (from, to) in [
        (Width::Single, Width::Double),
        (Width::Double, Width::Single),
    ] {
        for mode in RoundingMode::ALL {
            let name = format!(
                "convert_{}_{}_{}",
                from.name(),
                to.name(),
                mode_suffix(mode)
            );
            source.push_str(&format!(
                "function {name} using v(a: {}[{m}]): {}[{m}] pure {{ convert<{}[{m}]>(a) }}\n",
                from.name(),
                to.name(),
                to.name(),
                m = mode.as_str()
            ));
            conversions.push(Conversion {
                name,
                source: from,
                mode,
            });
        }
    }
    let producer = family_checked_ieee_source::checked(&source);
    let emission = emit_checked(
        &producer,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("original IEEE source emits");
    assert!(emission.omitted().is_empty());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact IEEE package admits");
    };
    let names = arithmetic
        .iter()
        .map(|row| row.name.clone())
        .chain(conversions.iter().map(|row| row.name.clone()))
        .collect::<Vec<_>>();
    let nodes = names
        .iter()
        .map(|name| family_function_selection::body(&package, name))
        .collect::<Vec<_>>();
    let items = derive_exact_scalar_items(&package, &nodes)
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.unwrap_or_else(|error| panic!("{} derivation: {error:?}", names[index]))
        })
        .collect::<Vec<_>>();
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("IEEE operation items generate");
    assert_eq!(output.claim_map.items.len(), names.len());
    let authority = family_checked_ieee_source::checked(&source);
    let mut expected = Vec::new();
    let mut labels = Vec::new();
    let mut functions = String::new();
    let mut calls = String::new();
    let header = r#"
mod oracle_observation;
use quire_exact::{IeeeValue,Meter,Outcome,ScalarLimits,Value};
use quire_semantic_value::loss::ValueLoss;
fn observation(outcome: Outcome<quire_exact::IeeeResult>)->(Outcome<Value>,Vec<ValueLoss>) {
    match outcome {
        Outcome::Completed(result)=>{ let flags=result.flags(); let losses=if flags.is_empty() { Vec::new() } else { vec![ValueLoss::IeeeFlags(flags)] }; (Outcome::Completed(Value::Float(result.value())),losses) }
        Outcome::Undefined(v)=>(Outcome::Undefined(v),Vec::new()),
        Outcome::Refused(v)=>(Outcome::Refused(v),Vec::new()),
        Outcome::Incomplete(v)=>(Outcome::Incomplete(v),Vec::new()),
    }
}
#[test]
fn observe() {
"#;
    let mut record = |name: &str,
                      label: &str,
                      arguments: &[(String, IeeeValue)],
                      work: u64,
                      symbol: &str| {
        let graph = authority.graph();
        let function = graph
            .function_by_identity(graph.function_identity(name).expect("source identity"))
            .expect("checked body");
        let mut meter = Meter::new(family_scalar_limits::limits(work));
        let evaluation = qsl_eval::value::evaluate_closed_with_frame(
            &authority,
            function.body,
            function.slots,
            arguments
                .iter()
                .map(|(_, value)| Value::Float(*value))
                .collect(),
            function.formed_units,
            &mut meter,
        )
        .expect("checked IEEE body evaluation");
        let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
            panic!("IEEE value family");
        };
        if work == 0 {
            assert!(
                matches!(&outcome, Outcome::Incomplete(stop) if stop.limit_kind == LimitKind::WorkUnits && stop.limit == 0 && stop.consumed == 0),
                "{label} work-zero: {outcome:?}"
            );
            assert!(evaluation.location.is_some());
        }
        let losses = evaluation
            .losses
            .into_iter()
            .map(|loss| loss.loss)
            .collect::<Vec<ValueLoss>>();
        expected.push(oracle_observation::snapshot(&outcome, &losses, &meter));
        labels.push(format!("{label} at work {work}"));
        let arguments = arguments
            .iter()
            .map(|(text, _)| text.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let number = expected.len();
        functions.push_str(&format!(
            r#"
fn case_{number}() {{
let mut meter=Meter::new({:?});
let actual=quire_exact_scalar_oracles::{symbol}({arguments}, &mut meter).expect("IEEE operand admission");
let (actual,losses)=observation(actual);
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter));
}}
"#,
            family_scalar_limits::limits(work),
        ));
        calls.push_str(&format!("case_{number}();\n"));
        outcome
    };
    for (index, row) in arithmetic.iter().enumerate() {
        let claim = output
            .claim_map
            .items
            .iter()
            .find(|claim| claim.node_id == nodes[index])
            .expect("one original IEEE claim");
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        assert!(
            matches!(
                &items[index].operation,
                ExactScalarOperation::IeeeArithmetic { width, rounding, .. }
                    if *rounding == row.mode
                        && matches!((width, row.width), (IeeeWidth::Binary32, Width::Single) | (IeeeWidth::Binary64, Width::Double))
            ),
            "{} selected operation: {:?}",
            row.name,
            items[index].operation
        );
        for (left, right) in PAIRS {
            let arguments = [
                row.width.value(OPERANDS[left].1, OPERANDS[left].2),
                row.width.value(OPERANDS[right].1, OPERANDS[right].2),
            ];
            let label = format!("{} {}/{}", row.name, OPERANDS[left].0, OPERANDS[right].0);
            for work in [1024, 0, 1] {
                let outcome = record(&row.name, &label, &arguments, work, &generated.symbol);
                // Sums, differences and products of one and three are exact in every mode.
                if work == 1024 && (left, right) == (0, 1) && row.operator != "divide" {
                    let Outcome::Completed(Value::Float(value)) = outcome else {
                        panic!("{label} completes: {outcome:?}");
                    };
                    let required = match (row.width, row.operator) {
                        (Width::Single, "add") => IeeeValue::binary32(0x4080_0000),
                        (Width::Double, "add") => IeeeValue::binary64(0x4010_0000_0000_0000),
                        (Width::Single, "subtract") => IeeeValue::binary32(0xc000_0000),
                        (Width::Double, "subtract") => IeeeValue::binary64(0xc000_0000_0000_0000),
                        (Width::Single, _) => IeeeValue::binary32(0x4040_0000),
                        (Width::Double, _) => IeeeValue::binary64(0x4008_0000_0000_0000),
                    };
                    assert_eq!(value, required, "{label}");
                }
            }
        }
    }
    for (offset, row) in conversions.iter().enumerate() {
        let index = arithmetic.len() + offset;
        let claim = output
            .claim_map
            .items
            .iter()
            .find(|claim| claim.node_id == nodes[index])
            .expect("one original IEEE conversion claim");
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{} generates: {:?}", row.name, claim.result);
        };
        assert!(
            matches!(
                &items[index].operation,
                ExactScalarOperation::IeeeWidthConversion { rounding, .. } if *rounding == row.mode
            ),
            "{} selected operation: {:?}",
            row.name,
            items[index].operation
        );
        for (operand, single, double) in OPERANDS {
            let arguments = [row.source.value(single, double)];
            let label = format!("{} {}", row.name, operand);
            for work in [1024, 0, 1] {
                let _stopped_or_completed =
                    record(&row.name, &label, &arguments, work, &generated.symbol);
            }
        }
    }
    let driver = format!("{header}{calls}}}\n{functions}");
    let actual = generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(actual.len(), expected.len());
    for ((label, actual), expected) in labels.iter().zip(&actual).zip(&expected) {
        assert_eq!(actual, expected, "{label}");
    }
}
