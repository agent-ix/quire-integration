// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native caller and independent FR-485 observation, with no local body factory.

mod family_function_selection {
    include!("family_function_selection.rs");
}

mod family_scalar_limits {
    include!("family_scalar_limits.rs");
}

use family_function_selection::declared;
use family_scalar_limits::limits;
mod family_function_envelope { include!("family_function_envelope.rs"); }
use family_function_envelope::same_envelope;

use qsl_replay::{
    ApplicationBeginRefusal, ApplicationEnvironment, DependencyInput, LibraryName, ReplayLimits, SourceIdentity, StageLimits,
};
use quire_contract_codegen::{checked_function_key, CheckedFunctionError, CheckedFunctionSource};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use quire_eval::{CallFailure, Evaluation, FamilyOutcome};
use quire_exact::{ChargePoint, Integer, LimitKind, Meter, Outcome, Value};
use quire_semantic_value::{call::InputRefusal, location::Origin, object_closure::ObjectClosure};

mod family_function_equality_source { include!("family_function_equality_source.rs"); }
use family_function_equality_source::{acquire, SOURCE, NAMES};

fn generated(
    name: &str,
    bytes: &[u8],
    package: &CheckedPackageV2,
    arguments: Vec<Value>,
    meter: &mut Meter,
) -> Result<Result<Evaluation, CallFailure>, CheckedFunctionError> {
    let dependencies = DependencyInput::default();
    let source = CheckedFunctionSource {
        source: SourceIdentity::new("agent-ix", "function-equality"),
        path: "function-equality.native",
        bytes,
        packages: &[],
        dependencies: &dependencies,
        limits: StageLimits::default(),
        replay_limits: ReplayLimits::default(),
        body_resources: Default::default(),
    };
    match name {
        "boolEqual" => quire_exact_function_oracles::__SYMBOL_0__(
            source,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        "boolNotEqual" => quire_exact_function_oracles::__SYMBOL_1__(
            source,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        "intEqual" => quire_exact_function_oracles::__SYMBOL_2__(
            source,
            package,
            package,
            arguments,
            ObjectClosure::default(),
            meter,
        ),
        "intNotEqual" => quire_exact_function_oracles::__SYMBOL_3__(source, package, package, arguments, ObjectClosure::default(), meter),
        "pairEqual" => quire_exact_function_oracles::__SYMBOL_4__(source, package, package, arguments, ObjectClosure::default(), meter),
        "pairNotEqual" => quire_exact_function_oracles::__SYMBOL_5__(source, package, package, arguments, ObjectClosure::default(), meter),
        "flagEqual" => quire_exact_function_oracles::__SYMBOL_6__(source, package, package, arguments, ObjectClosure::default(), meter),
        "flagNotEqual" => quire_exact_function_oracles::__SYMBOL_7__(source, package, package, arguments, ObjectClosure::default(), meter),
        "choiceEqual" => quire_exact_function_oracles::__SYMBOL_8__(source, package, package, arguments, ObjectClosure::default(), meter),
        "choiceNotEqual" => quire_exact_function_oracles::__SYMBOL_9__(source, package, package, arguments, ObjectClosure::default(), meter),
        _ => panic!("unselected function row"),
    }
}

fn required(
    name: &str,
    source_index: usize,
    expected: bool,
    work: u64,
    result: &Evaluation,
    meter: &Meter,
) {
    assert!(result.losses.is_empty());
    if work == 1024 {
        assert!(
            matches!(&result.outcome, FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(value))) if value == &expected)
        );
        assert!(result.location.is_none());
    } else {
        let point = if work == 0 {
            ChargePoint::FunctionCall
        } else {
            ChargePoint::EqualityPlanForm
        };
        assert!(
            matches!(&result.outcome, FamilyOutcome::Evaluated(Outcome::Incomplete(record)) if record.limit_kind == LimitKind::WorkUnits && record.limit == work && record.consumed == work && record.next_charge == Integer::from(if work == 0 { 1_u64 } else if name.starts_with("pair") { 6_u64 } else if name.starts_with("flag") { 4_u64 } else { 2_u64 }) && record.charge_point == point)
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
    let (acquisition, package) = acquire(SOURCE.as_bytes());
    let foreign = SOURCE.replace("left = right", "left != right");
    assert_ne!(foreign, SOURCE);
    let (foreign_acquisition, foreign_package) = acquire(foreign.as_bytes());
    assert_eq!(foreign_package.name(), package.name());
    assert_ne!(
        foreign_acquisition.package().bytes(),
        acquisition.package().bytes()
    );
    let mut envelopes = 0;
    let mut input_refusals = 0;
    let mut owner_refusals = 0;
    let handle = acquisition.handle();
    let owner = LibraryName::new(package.name().to_owned()).unwrap();
    for (source_index, name) in NAMES.into_iter().enumerate() {
        let boolean = source_index < 2;
        let samples: Vec<(Value, Value, bool)> = match name {
            "boolEqual" => vec![(Value::Boolean(false),Value::Boolean(false),true),(Value::Boolean(false),Value::Boolean(true),false),(Value::Boolean(true),Value::Boolean(false),false),(Value::Boolean(true),Value::Boolean(true),true)],
            "boolNotEqual" => vec![(Value::Boolean(false),Value::Boolean(false),false),(Value::Boolean(false),Value::Boolean(true),true),(Value::Boolean(true),Value::Boolean(false),true),(Value::Boolean(true),Value::Boolean(true),false)],
            "intEqual" => vec![(Value::Integer(Integer::from(3_i64)),Value::Integer(Integer::from(3_i64)),true),(Value::Integer(Integer::from(2_i64)),Value::Integer(Integer::from(3_i64)),false),(Value::Integer(Integer::from(4_i64)),Value::Integer(Integer::from(3_i64)),false)],
            "intNotEqual" => vec![(Value::Integer(Integer::from(3_i64)),Value::Integer(Integer::from(3_i64)),false),(Value::Integer(Integer::from(2_i64)),Value::Integer(Integer::from(3_i64)),true),(Value::Integer(Integer::from(4_i64)),Value::Integer(Integer::from(3_i64)),true)],
            nominal => {
                let (first, second) = match nominal {
                    "pairEqual" | "pairNotEqual" => ("makePairA", "makePairB"),
                    "flagEqual" | "flagNotEqual" => ("makeFlagA", "makeFlagB"),
                    "choiceEqual" | "choiceNotEqual" => ("makeChoiceA", "makeChoiceB"),
                    _ => panic!("unselected nominal row"),
                };
                let construct = |name: &str| {
                    let key = checked_function_key(&package, &declared(&package, name)).expect("original constructor identity");
                    let selected = handle.lookup_callable(&owner, &key).expect("original constructor selection");
                    let environment = ApplicationEnvironment::new(ObjectClosure::default());
                    let mut registry = handle.registry();
                    let mut meter = Meter::new(limits(1024));
                    let evaluation = registry.begin_call(&selected, Vec::new(), &environment, &mut meter).expect("source constructor admission").run().expect("source constructor evaluation");
                    assert!(evaluation.location.is_none());
                    assert!(evaluation.losses.is_empty());
                    let FamilyOutcome::Evaluated(Outcome::Completed(value)) = evaluation.outcome else { panic!("source constructor completes: {evaluation:?}"); };
                    assert!(matches!(value, Value::Composite(_) | Value::Union(_)), "actual nominal input: {value:?}");
                    value
                };
                let left = construct(first);
                let right = construct(second);
                let equal = source_index % 2 == 0;
                vec![(left.clone(), left.clone(), equal), (left, right, !equal)]
            }
        };
        let key = checked_function_key(&package, &declared(&package, name)).unwrap();
        let selection = handle.lookup_callable(&owner, &key).unwrap();
        let mut rejected_meter = Meter::new(limits(1024));
        let refusal = generated(
            name,
            foreign.as_bytes(),
            &package,
            vec![samples[0].0.clone(), samples[0].1.clone()],
            &mut rejected_meter,
        );
        assert!(
            matches!(refusal, Err(CheckedFunctionError::OwnerIdentityMismatch)),
            "valid changed source retains the original owner refusal: {refusal:?}"
        );
        zero(&rejected_meter);
        owner_refusals += 1;
        for (left, right, expected) in &samples {
        for work in [1024, 0, 1] {
            let arguments = vec![left.clone(), right.clone()];
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
                *expected,
                work,
                &authority,
                &authority_meter,
            );
            required(
                name,
                source_index,
                *expected,
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
            envelopes += 1;
            eprintln!("SC05 {name} work={work}: authority={authority:?}; generated={actual:?}; charges={:?}; counters={:?}",generated_meter.admitted_charges(),LimitKind::ALL.map(|kind|(kind,generated_meter.consumed(kind))));
        }
        }
        let valid = samples[0].0.clone();
        let wrong = if boolean { Value::Integer(Integer::one()) } else { Value::Boolean(true) };
        let mut invalid = vec![
            (vec![valid.clone()], InputRefusal::Arity { declared: 2, supplied: 1 }),
            (vec![wrong.clone(), valid.clone()], InputRefusal::WrongValueKind { parameter: 0 }),
            (vec![wrong], InputRefusal::Arity { declared: 2, supplied: 1 }),
        ];
        if (2..4).contains(&source_index) { invalid.push((vec![Value::Integer(Integer::from(10_i64)), valid], InputRefusal::WrongValueKind { parameter: 0 })); }
        for (arguments, required_refusal) in invalid {
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
            input_refusals += 1;
        }
    }
    println!(
        "IT019_OBSERVATION SC05 primitive function equality: {envelopes} envelopes, {input_refusals} input refusals, {owner_refusals} owner refusals"
    );
}
