// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original selected integer division laws compared with generated native oracles.

#[path = "support/family_checked_source.rs"]
#[allow(dead_code, reason = "the selected-law test uses the extended checker")]
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
use quire_exact::{DivisionProfile, Integer, LimitKind, Meter, Outcome, Undefined, Value};

const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
profile v = \"quire.value.complete/v1\";\n\
function quotient using v(left: Int[-9,9], right: Int[-9,9]): Integer pure { left div right }";

fn expected(profile: DivisionProfile, left: i64, right: i64) -> Option<i64> {
    if right == 0 {
        return None;
    }
    Some(match (profile, left, right) {
        (DivisionProfile::Truncating, -7, 3) => -2,
        (DivisionProfile::Floor | DivisionProfile::Euclidean, -7, 3) => -3,
        (DivisionProfile::Truncating | DivisionProfile::Floor, -7, -3) => 2,
        (DivisionProfile::Euclidean, -7, -3) => 3,
        _ => panic!("unexpected authored division input"),
    })
}

/// Trace: TC-102, IT-019-SC-05.
#[trace("TC-102", "IT-019-SC-05")]
#[test]
fn selected_division_profiles_agree_with_original_qsl_operations() {
    for profile in DivisionProfile::ALL {
        let producer = family_checked_source::checked_with_division(SOURCE, Some(profile));
        let emission = emit_checked(
            &producer,
            EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
        )
        .expect("original selected division source emits");
        assert!(emission.omitted().is_empty());
        let mut evidence = CheckedPackageEvidence::new();
        evidence.support_feature("quire.value.complete/v1");
        let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
            emission.package().bytes(),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) else {
            panic!("original division package admits");
        };
        let node = family_function_selection::body(&package, "quotient");
        let [result] = derive_exact_scalar_items(&package, std::slice::from_ref(&node))
            .try_into()
            .expect("one original division derivation");
        let item = result.expect("original division descriptor");
        assert!(
            matches!(&item.operation, ExactScalarOperation::IntegerDivision { profile: actual, .. } if *actual == profile),
            "selected law {profile:?}: {:?}",
            item.operation
        );
        let output = generate_exact_scalar_oracles(&package, &[item])
            .expect("selected division oracle generates");
        let [claim] = output.claim_map.items.as_slice() else {
            panic!("one original division claim");
        };
        assert_eq!(claim.node_id, node);
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("selected division generates: {:?}", claim.result);
        };
        let authority = family_checked_source::checked_with_division(SOURCE, Some(profile));
        let identity = authority
            .graph()
            .function_identity("quotient")
            .expect("authored quotient identity");
        let function = authority
            .graph()
            .function_by_identity(identity)
            .expect("original checked body");
        let mut expected_snapshots = Vec::new();
        let mut driver = String::from(
            r#"
mod oracle_observation;
use quire_exact::{Integer,Meter,Outcome,ScalarLimits,Value};
fn lift(outcome: Outcome<Integer>)->Outcome<Value> { match outcome { Outcome::Completed(v)=>Outcome::Completed(Value::Integer(v)),Outcome::Undefined(v)=>Outcome::Undefined(v),Outcome::Refused(v)=>Outcome::Refused(v),Outcome::Incomplete(v)=>Outcome::Incomplete(v) } }
#[test]
fn observe() {
"#,
        );
        for (left, right) in [(-7_i64, 3_i64), (-7, -3), (7, 0)] {
            for work in [1024, 0, 1] {
                let limits = family_scalar_limits::limits(work);
                let mut meter = Meter::new(limits);
                let evaluation = qsl_eval::value::evaluate_closed_with_frame(
                    &authority,
                    function.body,
                    function.slots,
                    vec![
                        Value::Integer(Integer::from(left)),
                        Value::Integer(Integer::from(right)),
                    ],
                    function.formed_units,
                    &mut meter,
                )
                .expect("public checked division evaluation");
                let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
                    panic!("selected division value family");
                };
                if work == 1024 {
                    match (&outcome, expected(profile, left, right)) {
                        (Outcome::Completed(Value::Integer(actual)), Some(expected)) => {
                            assert_eq!(actual, &Integer::from(expected));
                        }
                        (Outcome::Undefined(Undefined::DivisionByZero), None) => {}
                        _ => panic!("{profile:?} ({left}, {right}) typed result: {outcome:?}"),
                    }
                    assert_eq!(evaluation.location.is_some(), right == 0);
                } else if work == 0 {
                    assert!(
                        matches!(&outcome, Outcome::Incomplete(stop)
                            if stop.limit_kind == LimitKind::WorkUnits
                                && stop.limit == 0
                                && stop.consumed == 0),
                        "{profile:?} ({left}, {right}) required work-zero stop: {outcome:?}"
                    );
                    assert!(evaluation.location.is_some());
                } else if matches!(&outcome, Outcome::Incomplete(_)) {
                    assert!(evaluation.location.is_some());
                }
                assert!(evaluation.losses.is_empty());
                let losses = Vec::<quire_semantic_value::loss::ValueLoss>::new();
                expected_snapshots.push(oracle_observation::snapshot(&outcome, &losses, &meter));
                driver.push_str(&format!(
                    r#"
{{ let mut meter=Meter::new({limits:?});
let actual=quire_exact_scalar_oracles::{}(&Integer::from({left}_i64),&Integer::from({right}_i64),&mut meter).expect("original operands admit");
let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&lift(actual),&losses,&meter)); }}
"#,
                    generated.symbol
                ));
            }
        }
        driver.push_str("}\n");
        let actual = generated_oracle::execute(&output.artifacts, &driver);
        assert_eq!(
            actual, expected_snapshots,
            "{profile:?} complete division observations"
        );
    }
}
