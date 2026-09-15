// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-007/TC-008: bounded common contracts and typed authority separation.

use quire_integration::{
    ArtifactId, Budget, CanonicalEncode, CanonicalMap, CanonicalSet, ComponentId, Disposition,
    ErrorCode, Meter, RepositoryId, ResourceDimension, RoleId, RunId, Usage,
};

#[test]
fn tc008_equal_text_in_different_identity_domains_has_distinct_canonical_authority() {
    let repository = RepositoryId::new("shared", 64).expect("repository identity");
    let component = ComponentId::new("shared", 64).expect("component identity");
    let role = RoleId::new("shared", 64).expect("role identity");
    let artifact = ArtifactId::new("shared", 64).expect("artifact identity");
    let run = RunId::new("shared", 64).expect("run identity");

    let encodings = [
        repository.canonical_bytes(64).expect("repository bytes"),
        component.canonical_bytes(64).expect("component bytes"),
        role.canonical_bytes(64).expect("role bytes"),
        artifact.canonical_bytes(64).expect("artifact bytes"),
        run.canonical_bytes(64).expect("run bytes"),
    ];
    for (index, left) in encodings.iter().enumerate() {
        for right in encodings.iter().skip(index + 1) {
            assert_ne!(left, right);
        }
    }

    let oversized = RepositoryId::new("shared", 5)
        .expect_err("identity bytes must be checked before retention");
    assert_eq!(oversized.code(), ErrorCode::ResourceExhausted);
    assert_eq!(
        oversized.resource_dimension(),
        Some(ResourceDimension::IdentityBytes)
    );
    let oversized_whitespace =
        RepositoryId::new("      ", 5).expect_err("byte bound must precede whitespace traversal");
    assert_eq!(oversized_whitespace.code(), ErrorCode::ResourceExhausted);
    let output_over = run
        .canonical_bytes(1)
        .expect_err("canonical output must honor its byte ceiling");
    assert_eq!(output_over.code(), ErrorCode::ResourceExhausted);
    assert_eq!(
        output_over.resource_dimension(),
        Some(ResourceDimension::OutputBytes)
    );
}

#[test]
fn tc008_non_success_dispositions_remain_closed_and_distinct() {
    let dispositions = [
        Disposition::Completed,
        Disposition::Missing,
        Disposition::Incompatible,
        Disposition::Unsupported,
        Disposition::Stale,
        Disposition::Incomplete,
        Disposition::Refused,
        Disposition::Failed,
        Disposition::Unexecuted,
    ];
    assert_eq!(
        dispositions
            .iter()
            .filter(|state| state.is_completed())
            .count(),
        1
    );
    assert_eq!(
        CanonicalSet::try_from_values(dispositions.to_vec(), 9)
            .expect("closed states")
            .len(),
        9
    );
}

#[test]
fn tc007_every_budget_dimension_admits_exact_and_refuses_one_over() {
    type MeterOperation = fn(&mut Meter) -> quire_integration::Result<()>;

    let budget = Budget::new(2, 2, 2, 2, 2, 2, 2);
    let mut exact = Meter::new(budget);
    exact.consume_input_bytes(2).expect("exact input bytes");
    exact.retain_items(2).expect("exact retained items");
    exact.traverse_edges(2).expect("exact dependency edges");
    exact.step(2).expect("exact traversal steps");
    exact.invoke_producers(2).expect("exact invocations");
    exact.emit_output_bytes(2).expect("exact output bytes");
    exact.consume_duration_millis(2).expect("exact duration");
    assert_eq!(
        exact.usage(),
        Usage {
            input_bytes: 2,
            retained_items: 2,
            dependency_edges: 2,
            traversal_steps: 2,
            producer_invocations: 2,
            output_bytes: 2,
            duration_millis: 2,
        }
    );

    let operations: [(MeterOperation, ResourceDimension); 7] = [
        (
            |meter| meter.consume_input_bytes(3),
            ResourceDimension::InputBytes,
        ),
        (
            |meter| meter.retain_items(3),
            ResourceDimension::RetainedItems,
        ),
        (
            |meter| meter.traverse_edges(3),
            ResourceDimension::DependencyEdges,
        ),
        (|meter| meter.step(3), ResourceDimension::TraversalSteps),
        (
            |meter| meter.invoke_producers(3),
            ResourceDimension::ProducerInvocations,
        ),
        (
            |meter| meter.emit_output_bytes(3),
            ResourceDimension::OutputBytes,
        ),
        (
            |meter| meter.consume_duration_millis(3),
            ResourceDimension::DurationMillis,
        ),
    ];
    for (operation, dimension) in operations {
        let mut meter = Meter::new(budget);
        let before = meter.usage();
        let error = operation(&mut meter).expect_err("one-over must refuse");
        assert_eq!(error.code(), ErrorCode::ResourceExhausted);
        assert_eq!(error.resource_dimension(), Some(dimension));
        assert_eq!(
            meter.usage(),
            before,
            "refusal must not retain partial usage"
        );
    }
}

#[test]
fn tc007_canonical_collections_are_permutation_invariant_and_fail_closed() {
    let left = CanonicalMap::try_from_entries(
        vec![
            (RunId::new("b", 64).unwrap(), 2_u64),
            (RunId::new("a", 64).unwrap(), 1),
        ],
        2,
    )
    .expect("bounded map");
    let right = CanonicalMap::try_from_entries(
        vec![
            (RunId::new("a", 64).unwrap(), 1_u64),
            (RunId::new("b", 64).unwrap(), 2),
        ],
        2,
    )
    .expect("bounded map");
    assert_eq!(left, right);
    assert_eq!(
        left.canonical_bytes(256).unwrap(),
        right.canonical_bytes(256).unwrap()
    );
    let exact_bytes = left.canonical_len().expect("checked encoded length");
    assert_eq!(
        u64::try_from(left.canonical_bytes(exact_bytes).unwrap().len()).unwrap(),
        exact_bytes
    );
    let one_under = left
        .canonical_bytes(exact_bytes - 1)
        .expect_err("one-under byte budget must refuse before allocation");
    assert_eq!(one_under.code(), ErrorCode::ResourceExhausted);
    assert_eq!(
        one_under.resource_dimension(),
        Some(ResourceDimension::OutputBytes)
    );

    let over = CanonicalMap::try_from_entries(
        vec![
            (RunId::new("a", 64).unwrap(), 1_u64),
            (RunId::new("b", 64).unwrap(), 2),
        ],
        1,
    )
    .expect_err("one-over must refuse before retaining a map");
    assert_eq!(over.code(), ErrorCode::ResourceExhausted);
    assert_eq!(
        over.resource_dimension(),
        Some(ResourceDimension::RetainedItems)
    );

    let duplicate = CanonicalMap::try_from_entries(
        vec![
            (RunId::new("a", 64).unwrap(), 1_u64),
            (RunId::new("a", 64).unwrap(), 2),
        ],
        2,
    )
    .expect_err("duplicate identity must refuse");
    assert_eq!(duplicate.code(), ErrorCode::DuplicateIdentity);
}

#[test]
fn tc008_capability_traits_are_separate_and_deny_ambient_authority() {
    fn assert_producer<T: quire_integration::ProducerCapability>() {}
    fn assert_consumer<T: quire_integration::ConsumerCapability>() {}
    fn assert_persistence<T: quire_integration::PersistenceCapability>() {}
    fn assert_adapter<T: quire_integration::EffectAdapterCapability>() {}
    fn assert_evidence<T: quire_integration::EvidenceIntakeCapability>() {}
    fn assert_decision<T: quire_integration::DecisionCapability>() {}

    struct Producer;
    struct Consumer;
    struct Persistence;
    struct Adapter;
    struct Evidence;
    struct Decision;

    impl quire_integration::ProducerCapability for Producer {
        type Input = ();
        type Output = ();

        fn evaluate(&self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }
    impl quire_integration::ConsumerCapability for Consumer {
        type Input = ();
        type Output = ();

        fn consume(&self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }
    impl quire_integration::PersistenceCapability for Persistence {
        type Record = ();

        fn persist(&mut self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }
    impl quire_integration::EffectAdapterCapability for Adapter {
        type Request = ();
        type Receipt = ();

        fn dispatch(&mut self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }
    impl quire_integration::EvidenceIntakeCapability for Evidence {
        type Evidence = ();

        fn ingest(&mut self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }
    impl quire_integration::DecisionCapability for Decision {
        type Scope = ();
        type Decision = ();

        fn decide(&mut self, _: &(), _: &mut Meter) -> quire_integration::Result<()> {
            Ok(())
        }
    }

    assert_producer::<Producer>();
    assert_consumer::<Consumer>();
    assert_persistence::<Persistence>();
    assert_adapter::<Adapter>();
    assert_evidence::<Evidence>();
    assert_decision::<Decision>();
}
