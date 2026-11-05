// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration-owned native caller of the actual exported generated function.
use qsl_replay::{
    acquire_checked_closure, ApplicationEnvironment, CompileUnit, DependencyInput, LibraryName,
    ReplayLimits, SourceIdentity, StageLimits,
};
use quire_contract_codegen::{checked_function_key, CheckedFunctionSource};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use quire_eval::{CallFailure, Evaluation, FamilyOutcome, FamilyResult};
use quire_exact::{Integer, LimitKind, Meter, ScalarLimits, Value};
use quire_semantic_value::object_closure::ObjectClosure;
const SOURCE: &str = __AUTHORED_SOURCE__;

fn assert_same_call(
    authority: &Result<Evaluation, CallFailure>,
    actual: &Result<Evaluation, CallFailure>,
) {
    match (authority, actual) {
        (Err(expected), Err(actual)) => assert_eq!(expected, actual),
        (Ok(expected), Ok(actual)) => {
            assert_eq!(
                expected.location, actual.location,
                "complete result location"
            );
            assert_eq!(expected.losses, actual.losses, "ordered located losses");
            match (&expected.outcome, &actual.outcome) {
                (FamilyOutcome::Evaluated(expected), FamilyOutcome::Evaluated(actual)) => {
                    assert_eq!(expected, actual)
                }
                (
                    FamilyOutcome::FamilyEvaluated(FamilyResult::Refused(expected)),
                    FamilyOutcome::FamilyEvaluated(FamilyResult::Refused(actual)),
                ) => {
                    assert_eq!(expected.catalog_code(), actual.catalog_code());
                    assert_eq!(
                        expected.catalog_code().prefix(),
                        actual.catalog_code().prefix()
                    );
                    assert_eq!(expected.catalog_fields(), actual.catalog_fields());
                }
                (
                    FamilyOutcome::FamilyEvaluated(FamilyResult::Undefined(expected)),
                    FamilyOutcome::FamilyEvaluated(FamilyResult::Undefined(actual)),
                ) => {
                    assert_eq!(expected.undefined_record(), actual.undefined_record());
                    assert_eq!(expected.catalog_prefix(), actual.catalog_prefix());
                }
                _ => panic!("different complete outcome variants: {expected:?} versus {actual:?}"),
            }
        }
        _ => panic!("different call dispositions: {authority:?} versus {actual:?}"),
    }
}

#[test]
fn actual_function_and_fresh_qsl_session_agree() {
    let dependencies = DependencyInput::default();
    // This authority is a fresh acquisition; it does not reuse a generated
    // prepared body, expected label, serialized output or manually charged call.
    let acquired = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "oracle-function"),
            package_name: "oracle-function".to_owned(),
            path: "oracle-function.native",
            bytes: SOURCE.as_bytes(),
        },
        [],
        &dependencies,
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .unwrap();
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquired.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("same exact source admits");
    };
    let functions = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .iter()
                    .map(AsRef::as_ref)
                    .eq(["combine"])
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 1);
    let key = checked_function_key(&package, &functions[0].node_id).unwrap();
    let owner = LibraryName::new(package.name().to_owned()).unwrap();
    let handle = acquired.handle();
    let selection = handle.lookup_callable(&owner, &key).unwrap();
    for work in [1024, 0, 1] {
        let limits = ScalarLimits {
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
        };
        let arguments = vec![
            Value::Integer(Integer::from(4_i64)),
            Value::Integer(Integer::from(3_i64)),
        ];
        let mut authority_meter = Meter::new(limits);
        let mut registry = handle.registry();
        let environment = ApplicationEnvironment::new(ObjectClosure::default());
        let authority = registry
            .begin_call(
                &selection,
                arguments.clone(),
                &environment,
                &mut authority_meter,
            )
            .unwrap()
            .run();
        let mut generated_meter = Meter::new(limits);
        let actual = quire_exact_function_oracles::__GENERATED_SYMBOL__(
            CheckedFunctionSource {
                source: SourceIdentity::new("agent-ix", "oracle-function"),
                path: "oracle-function.native",
                bytes: SOURCE.as_bytes(),
                packages: &[],
                dependencies: &dependencies,
                limits: StageLimits::default(),
                replay_limits: ReplayLimits::default(),
                body_resources: Default::default(),
            },
            &package,
            &package,
            arguments,
            ObjectClosure::default(),
            &mut generated_meter,
        )
        .expect("genuine source/environment authentication");
        // Compare every Evaluation field and CallFailure variant, complete
        // located losses and outcomes, then ordered charges and every counter.
        // Both meters have exactly the same original global allowance.
        eprintln!("whole call work={work}: authority={authority:?}; actual={actual:?}; authority charges={:?}; actual charges={:?}; authority counters={:?}; actual counters={:?}", authority_meter.admitted_charges(),generated_meter.admitted_charges(),LimitKind::ALL.map(|kind|(kind,authority_meter.consumed(kind))),LimitKind::ALL.map(|kind|(kind,generated_meter.consumed(kind))));
        assert_same_call(&authority, &actual);
        assert_eq!(
            authority_meter.admitted_charges(),
            generated_meter.admitted_charges(),
            "ordered charges at work={work}"
        );
        for kind in LimitKind::ALL {
            assert_eq!(
                authority_meter.consumed(kind),
                generated_meter.consumed(kind),
                "counter {kind:?} at work={work}"
            );
        }
    }
    println!("IT019_OBSERVATION whole-function-envelope: 3 exact comparisons");
}
