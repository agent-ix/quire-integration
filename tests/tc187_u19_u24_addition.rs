// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original same-unit addition bodies and exact retention boundary.
#[path = "support/family_function_selection.rs"]
mod family_function_selection;
#[path = "support/generated_oracle.rs"]
mod generated_oracle;
#[path = "support/oracle_observation.rs"]
mod oracle_observation;
#[path = "support/tc187_p9_source.rs"]
mod source;

use ix_trace_rs::trace;
use quire_contract_ir::{CheckedNodeId, CheckedPackageV2, CheckedTerm};
use quire_eval::FamilyOutcome;
use quire_exact::{ChargePoint, Incomplete, Integer, LimitKind, Outcome, Rational, Value};

#[trace(
    "TC-106",
    "IT-019-SC-04",
    "QSpec-TC-187",
    "QSpec-FR-142-AC-1",
    "QSpec-FR-142-AC-7"
)]
#[test]
fn original_additions_retain_their_units_and_exact_unpaid_retention() {
    source::bind(
        "U19",
        &["2 m + 3 m", "L(4,0,0,2,5,1)", "L(4,0,0,2,4,1)"],
        &[
            "`5 m`",
            "unit.result-retain",
            "consumed: 4",
            "next_charge: 1",
        ],
    );
    source::bind(
        "U24",
        &["1 cm + 2 cm", "L(4,0,0,2,5,1)"],
        &["`3 cm`", "no `unit.edge`"],
    );
    let original = source::original("function metre using v(): Quantity<m> pure { 2 m + 3 m }\nfunction centimetre using v(): Quantity<cm> pure { 1 cm + 2 cm }");
    for (name, unit, expected, work) in [
        ("metre", "m", 5_i64, 5_u64),
        ("metre", "m", 5, 4),
        ("centimetre", "cm", 3, 5),
    ] {
        let (evaluation, meter) = source::evaluate(&original, name, false, source::limits(4, work));
        assert!(evaluation.losses.is_empty());
        let FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
            panic!("original quantity family");
        };
        if work == 5 {
            let Outcome::Completed(Value::Quantity(actual)) = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(
                actual.magnitude(),
                &Rational::from_integer(Integer::from(expected))
            );
            assert_eq!(
                actual.unit(),
                &original
                    .units
                    .declared_unit_id(original.keys[unit].clone())
                    .expect("source unit")
            );
            assert!(evaluation.location.is_none());
        } else {
            let Outcome::Incomplete(actual) = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(
                actual,
                Incomplete {
                    limit_kind: LimitKind::WorkUnits,
                    limit: 4,
                    consumed: 4,
                    next_charge: Integer::from(1_i64),
                    charge_point: ChargePoint::UnitResultRetain
                }
            );
            let graph = original.package.graph();
            let function = graph
                .function_by_identity(
                    graph
                        .function_identity(name)
                        .expect("original function identity"),
                )
                .expect("original function");
            assert_eq!(evaluation.location.as_ref(), Some(function.body.location()));
        }
        source::assert_ledger(
            &meter,
            if unit == "cm" { &[1, 2] } else { &[2, 2] },
            work == 5,
        );
    }
    assert_eq!(
        source::generation(
            &original,
            &[
                ("metre", false, source::limits(4, 5)),
                ("metre", false, source::limits(4, 4)),
                ("centimetre", false, source::limits(4, 5))
            ],
            &[],
            &[]
        ),
        3
    );
}
