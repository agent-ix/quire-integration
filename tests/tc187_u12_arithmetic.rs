// SPDX-License-Identifier: AGPL-3.0-or-later
//! Four original U12 arithmetic operations and canonical compound results.

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
    "QSpec-FR-142",
    "QSpec-FR-142-AC-6"
)]
#[test]
fn public_u12_arithmetic_values_have_their_canonical_compound_units() {
    source::bind(
        "U12",
        &["2 m * 3 s", "6 m / 2 s", "square `2 m`", "5 m / 5 m"],
        &["`6`", "`3`", "`4`", "`1`", "empty dimensionless"],
    );
    let cases = [
        ("product", "2 m * 3 s", 6_i64, vec![("m", 1_i64), ("s", 1)]),
        ("quotient", "6 m / 2 s", 3, vec![("m", 1), ("s", -1)]),
        ("square", "2 m ^ 2", 4, vec![("m", 2)]),
        ("dimensionless", "5 m / 5 m", 1, vec![]),
    ];
    let functions = cases
        .iter()
        .map(|(name, expression, _, _)| {
            format!("function {name} using v(): Boolean pure {{ {expression} = {expression} }}\n")
        })
        .collect::<String>();
    let original = source::original(&functions);
    for (name, _, expected, terms) in &cases {
        let (evaluation, meter) =
            source::evaluate(&original, name, true, source::limits(1024, 1024));
        assert!(evaluation.losses.is_empty());
        assert!(evaluation.location.is_none());
        let FamilyOutcome::Evaluated(Outcome::Completed(Value::Quantity(actual))) =
            evaluation.outcome
        else {
            panic!("{name}: {:?}", evaluation.outcome);
        };
        assert_eq!(
            actual.magnitude(),
            &Rational::from_integer(Integer::from(*expected)),
            "{name}"
        );
        let terms = terms
            .iter()
            .map(|(unit, power)| (original.keys[*unit].clone(), Integer::from(*power)))
            .collect::<std::collections::BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();
        let compound = original
            .units
            .compound_unit(&terms)
            .expect("owner canonical compound construction");
        assert_eq!(
            actual.unit(),
            &compound.id().expect("actual owner compound identity"),
            "{name}"
        );
        assert_eq!(
            actual.unit().compound_terms(),
            Some(terms.as_slice()),
            "{name} structural root terms"
        );
        let dimension_terms: &[(&str, i64)] = match *name {
            "product" => &[("Distance", 1), ("Duration", 1)],
            "quotient" => &[("Distance", 1), ("Duration", -1)],
            "square" => &[("Distance", 2)],
            "dimensionless" => &[],
            other => panic!("unexpected quantity operation {other}"),
        };
        let dimensions = dimension_terms
            .iter()
            .map(|(dimension, power)| (original.keys[*dimension].clone(), Integer::from(*power)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            compound
                .dimension()
                .exponents()
                .map(|(key, power)| (key, power.clone()))
                .collect::<std::collections::BTreeMap<_, _>>(),
            dimensions,
            "{name} base-dimension map"
        );
        assert_eq!(
            compound
                .terms()
                .map(|(key, power)| (key, power.clone()))
                .collect::<Vec<_>>(),
            terms
        );
        if *name == "dimensionless" {
            assert_eq!(
                compound.dimension(),
                &quire_semantic_value::unit::Dimension::dimensionless()
            );
            assert!(compound.terms().next().is_none());
        }
        let reads = match *name {
            "product" => &[2, 2][..],
            "quotient" | "dimensionless" => &[3, 3][..],
            "square" => &[2][..],
            other => panic!("unexpected operation {other}"),
        };
        source::assert_ledger(&meter, reads, true);
        let unpaid_work = if *name == "square" { 3 } else { 4 };
        let (denied, denied_meter) =
            source::evaluate(&original, name, true, source::limits(1024, unpaid_work));
        assert!(denied.losses.is_empty(), "{name} bounded losses");
        let FamilyOutcome::Evaluated(Outcome::Incomplete(stop)) = denied.outcome else {
            panic!(
                "{name}: expected unpaid retention, got {:?}",
                denied.outcome
            );
        };
        assert_eq!(
            stop,
            Incomplete {
                limit_kind: LimitKind::WorkUnits,
                limit: unpaid_work,
                consumed: unpaid_work,
                next_charge: Integer::from(1_i64),
                charge_point: ChargePoint::UnitResultRetain,
            },
            "{name} exact unpaid retention"
        );
        let graph = original.package.graph();
        let function = graph
            .function_by_identity(graph.function_identity(name).expect("source identity"))
            .expect("source function");
        let children = function.body.children();
        let [operation, _] = children.as_slice() else {
            panic!("{name}: equality children");
        };
        assert_eq!(
            denied.location.as_ref(),
            Some(operation.location()),
            "{name} original operation locus"
        );
        source::assert_ledger(&denied_meter, reads, false);
    }
    assert_eq!(
        source::generation(
            &original,
            &cases
                .iter()
                .flat_map(|(name, _, _, _)| {
                    let unpaid_work = if *name == "square" { 3 } else { 4 };
                    [
                        (*name, true, source::limits(1024, 1024)),
                        (*name, true, source::limits(1024, unpaid_work)),
                    ]
                })
                .collect::<Vec<_>>(),
            &[],
            &[]
        ),
        8
    );
}
