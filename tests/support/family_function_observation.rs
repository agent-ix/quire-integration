// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native caller and independent FR-485 observation, with no local body factory.

use qsl_replay::{
    acquire_checked_closure, ApplicationBeginRefusal, ApplicationEnvironment, CompileUnit,
    DependencyInput, LibraryName, ReplayLimits, SourceIdentity, StageLimits,
};
use quire_contract_codegen::{checked_function_key, CheckedFunctionError, CheckedFunctionSource};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use quire_eval::{CallFailure, Evaluation, FamilyOutcome};
use quire_exact::{ChargePoint, Integer, LimitKind, Meter, Outcome, ScalarLimits, Value};
use quire_semantic_value::{call::InputRefusal, location::Origin, object_closure::ObjectClosure};

const SOURCE: &str = __AUTHORED_SOURCE__;

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

fn selected(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    let nodes = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration
                .as_ref()
                .is_some_and(|decl| decl.qualified_name.iter().map(AsRef::as_ref).eq([name]))
        })
        .collect::<Vec<_>>();
    let [node] = nodes.as_slice() else {
        panic!("one original declaration {name}");
    };
    node.node_id.clone()
}

fn generated(
    name: &str,
    bytes: &[u8],
    package: &CheckedPackageV2,
    arguments: Vec<Value>,
    meter: &mut Meter,
) -> Result<Result<Evaluation, CallFailure>, CheckedFunctionError> {
    let dependencies = DependencyInput::default();
    let source = CheckedFunctionSource {
        source: SourceIdentity::new("agent-ix", "family-functions"),
        path: "family-functions.native",
        bytes,
        packages: &[],
        dependencies: &dependencies,
        limits: StageLimits::default(),
        replay_limits: ReplayLimits::default(),
        body_resources: Default::default(),
    };
    match name {
        "difference" => quire_exact_function_oracles::__SYMBOL_0__(
            source,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        "product" => quire_exact_function_oracles::__SYMBOL_1__(
            source,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        "relay" => quire_exact_function_oracles::__SYMBOL_2__(
            source,
            package,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        _ => panic!("unselected function row"),
    }
}

fn same_envelope(expected: &Evaluation, actual: &Evaluation) {
    assert_eq!(expected.location, actual.location, "entire result location");
    assert_eq!(expected.losses, actual.losses, "ordered located losses");
    match (&expected.outcome, &actual.outcome) {
        (
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(expected))),
            FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(actual))),
        ) => assert_eq!(expected, actual),
        (
            FamilyOutcome::Evaluated(Outcome::Incomplete(expected)),
            FamilyOutcome::Evaluated(Outcome::Incomplete(actual)),
        ) => assert_eq!(expected, actual),
        _ => panic!(
            "required Integer completion or exact incomplete shape: {:?} versus {:?}",
            expected.outcome, actual.outcome
        ),
    }
}

fn required(
    name: &str,
    source_index: usize,
    expected: i64,
    work: u64,
    result: &Evaluation,
    meter: &Meter,
) {
    assert!(result.losses.is_empty());
    if work == 1024 {
        assert!(
            matches!(&result.outcome, FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value))) if value == &Integer::from(expected))
        );
        assert!(result.location.is_none());
    } else {
        let point = if work == 0 || name == "relay" {
            ChargePoint::FunctionCall
        } else {
            ChargePoint::IntegerArithmeticOperands
        };
        assert!(
            matches!(&result.outcome, FamilyOutcome::Evaluated(Outcome::Incomplete(record)) if record.limit_kind == LimitKind::WorkUnits && record.limit == work && record.consumed == work && record.next_charge == Integer::one() && record.charge_point == point)
        );
        let location = result
            .location
            .as_ref()
            .expect("a stopped checked body has its location");
        assert!(
            matches!(&location.origin, Origin::Body { function, index } if function == name && *index == source_index)
        );
        assert_eq!(location.depth(), 0);
        assert_eq!(location.last_index(), None);
        let admitted = meter.admitted_charges();
        assert_eq!(admitted.len(), usize::try_from(work).unwrap());
        if work == 1 {
            assert_eq!(admitted[0].point, ChargePoint::FunctionCall);
        }
        for kind in LimitKind::ALL {
            assert_eq!(
                meter.consumed(kind),
                if kind == LimitKind::WorkUnits {
                    work
                } else {
                    0
                }
            );
        }
    }
}

fn zero(meter: &Meter) {
    assert!(meter.admitted_charges().is_empty());
    for kind in LimitKind::ALL {
        assert_eq!(meter.consumed(kind), 0);
    }
}

#[test]
fn selected_operations_and_boundaries_match_independent_sessions() {
    let acquisition = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "family-functions"),
            package_name: "family-functions".to_owned(),
            path: "family-functions.native",
            bytes: SOURCE.as_bytes(),
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("fresh independent authority acquisition");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquisition.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("independent original bytes admit");
    };
    let handle = acquisition.handle();
    let owner = LibraryName::new(package.name().to_owned()).unwrap();
    for (source_index, (name, expected)) in [("difference", 1_i64), ("product", 12), ("relay", 1)]
        .into_iter()
        .enumerate()
    {
        let key = checked_function_key(&package, &selected(&package, name)).unwrap();
        let selection = handle.lookup_callable(&owner, &key).unwrap();
        let foreign = SOURCE.replace("left - right", "left + right");
        assert_ne!(foreign, SOURCE);
        let mut rejected_meter = Meter::new(limits(1024));
        let refusal = generated(
            name,
            foreign.as_bytes(),
            &package,
            vec![
                Value::Integer(Integer::from(4_i64)),
                Value::Integer(Integer::from(3_i64)),
            ],
            &mut rejected_meter,
        );
        assert!(
            matches!(refusal, Err(CheckedFunctionError::OwnerIdentityMismatch)),
            "valid changed source retains the original owner refusal: {refusal:?}"
        );
        zero(&rejected_meter);
        for work in [1024, 0, 1] {
            let arguments = vec![
                Value::Integer(Integer::from(4_i64)),
                Value::Integer(Integer::from(3_i64)),
            ];
            let mut authority_meter = Meter::new(limits(work));
            let environment = ApplicationEnvironment::new(ObjectClosure::default());
            let mut registry = handle.registry();
            let authority = registry
                .begin_call(
                    &selection,
                    arguments.clone(),
                    &environment,
                    &mut authority_meter,
                )
                .expect("real authority Begin")
                .run()
                .expect("required authority evaluation");
            let mut generated_meter = Meter::new(limits(work));
            let actual = generated(
                name,
                SOURCE.as_bytes(),
                &package,
                arguments,
                &mut generated_meter,
            )
            .expect("original generated source authenticates")
            .expect("required generated evaluation");
            same_envelope(&authority, &actual);
            required(
                name,
                source_index,
                expected,
                work,
                &authority,
                &authority_meter,
            );
            required(
                name,
                source_index,
                expected,
                work,
                &actual,
                &generated_meter,
            );
            assert_eq!(
                authority_meter.admitted_charges(),
                generated_meter.admitted_charges()
            );
            for kind in LimitKind::ALL {
                assert_eq!(
                    authority_meter.consumed(kind),
                    generated_meter.consumed(kind)
                );
            }
            eprintln!("SC05 {name} work={work}: authority={authority:?}; generated={actual:?}; charges={:?}; counters={:?}",generated_meter.admitted_charges(),LimitKind::ALL.map(|kind|(kind,generated_meter.consumed(kind))));
        }
        for (arguments, required_refusal) in [
            (
                vec![Value::Integer(Integer::one())],
                InputRefusal::Arity {
                    declared: 2,
                    supplied: 1,
                },
            ),
            (
                vec![Value::Boolean(true), Value::Integer(Integer::one())],
                InputRefusal::WrongValueKind { parameter: 0 },
            ),
            (
                vec![Value::Boolean(true)],
                InputRefusal::Arity {
                    declared: 2,
                    supplied: 1,
                },
            ),
        ] {
            let environment = ApplicationEnvironment::new(ObjectClosure::default());
            let mut authority_meter = Meter::new(limits(1024));
            let mut registry = handle.registry();
            let expected = match registry.begin_call(
                &selection,
                arguments.clone(),
                &environment,
                &mut authority_meter,
            ) {
                Err(error) => error,
                Ok(_) => panic!("invalid argument input cannot begin"),
            };
            assert_eq!(
                expected,
                ApplicationBeginRefusal::Call(CallFailure::Input(required_refusal))
            );
            let mut meter = Meter::new(limits(1024));
            let Err(CheckedFunctionError::ApplicationBeginRefused(actual)) =
                generated(name, SOURCE.as_bytes(), &package, arguments, &mut meter)
            else {
                panic!("generated input refusal retains the native Begin payload");
            };
            assert_eq!(actual, expected);
            zero(&authority_meter);
            zero(&meter);
        }
    }
    println!(
        "IT019_OBSERVATION SC05 function rows: 9 envelopes, 9 input refusals, 3 owner refusals"
    );
}
