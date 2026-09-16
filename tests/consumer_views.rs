// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-004: bounded deterministic views and non-authoritative consumer accounting.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_integration::consumer_views::*;
use quire_integration::ecosystem_lock::{
    validate, AdmittedCapability, CapabilityBinding, CapabilitySelection, ComponentBinding,
    Limits as LockLimits, LockSelection, PackageBinding, Pin, PinState, RepositoryBinding,
    RoleBinding, WorkflowBinding,
};
use quire_integration::{
    ArtifactId, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, PackageId,
    ProviderId, RepositoryId, RoleId, ToolchainId, WorkflowInstanceId,
};
use sha2::{Digest as _, Sha256};

const ID_LIMIT: u64 = 256;

macro_rules! id {
    ($kind:ty, $value:expr) => {
        <$kind>::new($value, ID_LIMIT).expect("bounded fixture identity")
    };
}

fn revision(seed: char) -> String {
    format!("git:{}", seed.to_string().repeat(40))
}

fn exact<T: quire_integration::ecosystem_lock::PinIdentity>(identity: T, seed: char) -> Pin<T> {
    Pin::new(identity, &revision(seed), PinState::Exact, ID_LIMIT).expect("exact fixture pin")
}

fn reader_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id!(RoleId, "checked-reader"),
        id!(ProviderId, "checked-reader-provider"),
        id!(BuildId, "checked-reader-build"),
        id!(ContractId, CHECKED_READER_CONTRACT),
        id!(ArtifactId, "view-profile:v1"),
    )
}

fn consumer_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id!(RoleId, "semantic-consumer"),
        id!(ProviderId, "semantic-consumer-provider"),
        id!(BuildId, "semantic-consumer-build"),
        id!(ContractId, CONSUMER_CONTRACT),
        id!(ArtifactId, "profile-artifact:v1"),
    )
}

fn validated_lock() -> quire_integration::ecosystem_lock::ValidatedLock {
    validated_lock_for("local-test")
}

fn validated_lock_for(deployment_id: &str) -> quire_integration::ecosystem_lock::ValidatedLock {
    let repository = id!(RepositoryId, "agent-ix/view-fixture");
    let package = id!(PackageId, "view-fixture");
    let component = id!(ComponentId, "view-fixture");
    let definition = id!(DefinitionId, "checked-model");
    let selection = LockSelection::new(
        vec![RepositoryBinding::new(
            exact(repository.clone(), 'a'),
            "https://github.com/agent-ix/view-fixture",
            ID_LIMIT,
        )
        .expect("repository binding")],
        vec![PackageBinding::new(
            exact(package.clone(), 'b'),
            "1.0.0",
            repository.clone(),
            &revision('a'),
            &["default"],
            1,
            ID_LIMIT,
        )
        .expect("package binding")],
        vec![exact(definition.clone(), 'c')],
        vec![
            exact(id!(ProviderId, "checked-reader-provider"), 'd'),
            exact(id!(ProviderId, "semantic-consumer-provider"), 'e'),
        ],
        vec![exact(id!(ToolchainId, "rust-1.98-or-later"), 'f')],
        vec![
            exact(id!(BuildId, "checked-reader-build"), '1'),
            exact(id!(BuildId, "semantic-consumer-build"), '2'),
        ],
        vec![exact(id!(DeploymentId, deployment_id), '3')],
        vec![ComponentBinding::new(
            exact(component.clone(), '4'),
            repository,
            package,
        )],
        vec![
            RoleBinding::new(exact(id!(RoleId, "checked-reader"), '5'), component.clone()),
            RoleBinding::new(exact(id!(RoleId, "semantic-consumer"), '6'), component),
        ],
        vec![
            CapabilityBinding::new(reader_selection()),
            CapabilityBinding::new(consumer_selection()),
        ],
        vec![WorkflowBinding::new(
            exact(id!(WorkflowInstanceId, "view:fixture"), '7'),
            definition,
        )],
        vec![],
    );
    let limits = LockLimits::for_selection(&selection, 64 * 1024, ID_LIMIT).expect("lock limits");
    validate(&selection, limits).expect("validated exact lock")
}

fn limits() -> Limits {
    Limits {
        max_identity_bytes: ID_LIMIT,
        max_input_bytes: 4096,
        max_nodes: 16,
        max_source_artifacts: 4,
        max_projection_edges: 16,
        max_reader_invocations: 1,
        max_edges: 16,
        max_traversal_steps: 16,
        max_results: 16,
        max_frontier: 16,
        max_obligations: 16,
        max_consumer_invocations: 1,
        max_mappings: 16,
        max_output_bytes: 4096,
        max_accounting_bytes: 64 * 1024,
        max_duration_ms: 1000,
    }
}

fn semantic(value: &str) -> QualifiedSemanticId {
    QualifiedSemanticId {
        definition: id!(DefinitionId, "checked-model"),
        semantic: SemanticId::new(value, ID_LIMIT).expect("semantic id"),
    }
}

fn source(value: &str) -> SourceRef {
    SourceRef {
        artifact: id!(ArtifactId, "checked-owner-artifact"),
        region: SourceRegionId::new(value, ID_LIMIT).expect("region id"),
    }
}

fn node(value: &str) -> SemanticNode {
    SemanticNode {
        id: semantic(value),
        source: source(&format!("source:{value}")),
    }
}

fn edge(from: &str, to: &str) -> SemanticEdge {
    SemanticEdge {
        source: semantic(from),
        relation: RelationKind::Contains,
        target: semantic(to),
        provenance: source(&format!("edge:{from}:{to}")),
    }
}

fn projection(reverse_nodes: bool, reverse_edges: bool) -> ReaderProjection {
    let mut nodes = vec![node("root"), node("alpha"), node("beta"), node("gamma")];
    let mut edges = vec![
        edge("root", "beta"),
        edge("root", "alpha"),
        edge("alpha", "gamma"),
        edge("beta", "gamma"),
        edge("gamma", "root"),
    ];
    if reverse_nodes {
        nodes.reverse();
    }
    if reverse_edges {
        edges.reverse();
    }
    ReaderProjection {
        package: id!(PackageId, "view-fixture"),
        artifact: id!(ArtifactId, "checked-owner-artifact"),
        digest: owner_digest(),
        nodes,
        edges,
    }
}

fn owner_digest() -> [u8; 32] {
    Sha256::digest(b"owner:checked-package:v1").into()
}

fn owner() -> OwnerPackage {
    OwnerPackage {
        contract_version: "quire.checked-package/v1".to_owned(),
        package: id!(PackageId, "view-fixture"),
        artifact: id!(ArtifactId, "checked-owner-artifact"),
        canonical_bytes: b"owner:checked-package:v1".to_vec(),
        digest: owner_digest(),
    }
}

fn query() -> ViewQuery {
    ViewQuery {
        reader: reader_selection(),
        package: id!(PackageId, "view-fixture"),
        owner_artifact: id!(ArtifactId, "checked-owner-artifact"),
        package_digest: owner_digest(),
        root: semantic("root"),
        relation: RelationSelection {
            contract_version: "quire.semantic-relation/v1".to_owned(),
            kind: RelationKind::Contains,
        },
        filter: ViewFilter::All,
        projection: ViewProjection::IdentityAndSource,
        profile: id!(ArtifactId, "view-profile:v1"),
    }
}

struct TestReader {
    selection: CapabilitySelection,
    source_artifacts: std::collections::BTreeSet<ArtifactId>,
    projection: ReaderProjection,
    error: Option<ReaderError>,
    calls: Cell<u64>,
}

impl TestReader {
    fn new(projection: ReaderProjection) -> Self {
        Self {
            selection: reader_selection(),
            source_artifacts: [
                id!(ArtifactId, "checked-owner-artifact"),
                id!(ArtifactId, "included-source-artifact"),
            ]
            .into_iter()
            .collect(),
            projection,
            error: None,
            calls: Cell::new(0),
        }
    }

    fn failing(projection: ReaderProjection, error: ReaderError) -> Self {
        let mut reader = Self::new(projection);
        reader.error = Some(error);
        reader
    }
}

impl CheckedPackageReader for TestReader {
    fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    fn source_artifacts(&self) -> &std::collections::BTreeSet<ArtifactId> {
        &self.source_artifacts
    }

    fn read(
        &self,
        admission: &AdmittedCapability<'_>,
        _: &OwnerPackage,
        budget: CallBudget,
    ) -> std::result::Result<ReaderProjection, ReaderError> {
        assert_eq!(admission.contract(), CHECKED_READER_CONTRACT);
        assert!(budget.remaining_duration_ms() > 0);
        assert!(budget.max_output_bytes() > 0);
        assert!(budget.max_nodes() > 0);
        assert!(budget.max_edges() > 0);
        self.calls.set(self.calls.get() + 1);
        self.error.map_or_else(|| Ok(self.projection.clone()), Err)
    }
}

fn view_with(projected: ReaderProjection, bounded: Limits) -> SemanticView {
    ViewCoordinator::new(bounded)
        .query(
            &validated_lock(),
            &owner(),
            &query(),
            &TestReader::new(projected),
        )
        .expect("valid semantic view")
}

fn obligation(name: &str, subject: &str, bound: Option<u64>) -> RequestedObligation {
    RequestedObligation {
        id: ObligationId::new(name, ID_LIMIT).expect("obligation id"),
        semantic: semantic(subject),
        source: source(&format!("source:{subject}")),
        output_bound: bound,
    }
}

fn manifest(supported: &[&str]) -> ConsumerAdmission {
    ConsumerAdmission {
        selection: consumer_selection(),
        profile: ProfileId::new("profile:v1", ID_LIMIT).expect("profile id"),
        profile_artifact: id!(ArtifactId, "profile-artifact:v1"),
        region_encoding: RegionEncoding::Utf8,
        kind: ConsumerKind::CodeGenerator,
        supported_obligations: supported
            .iter()
            .map(|value| ObligationId::new(value, ID_LIMIT).expect("supported obligation"))
            .collect(),
    }
}

fn request(obligations: Vec<RequestedObligation>) -> ConsumerRequest {
    ConsumerRequest {
        selection: consumer_selection(),
        profile: ProfileId::new("profile:v1", ID_LIMIT).expect("profile id"),
        profile_artifact: id!(ArtifactId, "profile-artifact:v1"),
        region_encoding: RegionEncoding::Utf8,
        kind: ConsumerKind::CodeGenerator,
        obligations,
    }
}

fn completion(request: &RequestedObligation, output: ConsumerOutput) -> ConsumerCompletion {
    ConsumerCompletion {
        obligation: request.id.clone(),
        semantic: request.semantic.clone(),
        source: request.source.clone(),
        output,
    }
}

struct TestConsumer {
    manifest: ConsumerAdmission,
    completions: Vec<ConsumerCompletion>,
    error: Option<ConsumerError>,
    error_after_submissions: Option<ConsumerError>,
    calls: u64,
}

impl TestConsumer {
    fn new(manifest: ConsumerAdmission, completions: Vec<ConsumerCompletion>) -> Self {
        Self {
            manifest,
            completions,
            error: None,
            error_after_submissions: None,
            calls: 0,
        }
    }
}

impl SemanticConsumer for TestConsumer {
    fn admission(&self) -> &ConsumerAdmission {
        &self.manifest
    }

    fn consume(
        &mut self,
        permit: &ConsumerPermit,
        _: &SemanticView,
        obligations: &[RequestedObligation],
        output: &mut ConsumerOutputSink,
    ) -> std::result::Result<(), ConsumerError> {
        assert!(permit.remaining_duration_ms() > 0);
        assert!(permit.max_output_bytes() > 0);
        assert!(permit.remaining_mappings() > 0);
        assert!(!obligations.is_empty());
        self.calls += 1;
        if let Some(error) = self.error {
            return Err(error);
        }
        for completion in &self.completions {
            if output.submit(completion.borrowed()).is_err() {
                break;
            }
        }
        self.error_after_submissions.map_or(Ok(()), Err)
    }
}

#[derive(Debug, Default)]
struct StepClock(AtomicU64);

impl MonotonicClock for StepClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    #[trace("TC-004", "FR-004-AC-1")]
    fn tc004_equal_inputs_are_canonical_across_all_input_orders(
        reverse_nodes in any::<bool>(),
        reverse_edges in any::<bool>(),
        reverse_obligations in any::<bool>(),
        reverse_completions in any::<bool>(),
    ) {
        let lock = validated_lock();
        let canonical_view = ViewCoordinator::new(limits())
            .query(&lock, &owner(), &query(), &TestReader::new(projection(false, false)))
            .unwrap();
        let permuted_view = ViewCoordinator::new(limits())
            .query(&lock, &owner(), &query(), &TestReader::new(projection(reverse_nodes, reverse_edges)))
            .unwrap();
        prop_assert_eq!(canonical_view.canonical_bytes(), permuted_view.canonical_bytes());
        prop_assert_eq!(canonical_view.identity(), permuted_view.identity());

        let alpha = obligation("obligation:alpha", "alpha", Some(64));
        let beta = obligation("obligation:beta", "beta", Some(64));
        let mut obligations = vec![alpha.clone(), beta.clone()];
        if reverse_obligations { obligations.reverse(); }
        let consumer_request = request(obligations);
        let completions = vec![
            completion(&alpha, ConsumerOutput::Generated {
                bytes: b"alpha".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 5 }],
                mapping_loss: MappingLoss::Preserved,
            }),
            completion(&beta, ConsumerOutput::Displayed {
                bytes: b"beta".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: MappingLoss::Conditional,
            }),
        ];
        let mut reversed = completions.clone();
        if reverse_completions { reversed.reverse(); }
        let mut left = TestConsumer::new(manifest(&["obligation:alpha", "obligation:beta"]), completions);
        let mut right = TestConsumer::new(manifest(&["obligation:alpha", "obligation:beta"]), reversed);
        let coordinator = ConsumerCoordinator::new(limits());
        let left_run = coordinator.consume(&lock, &canonical_view, &consumer_request, &mut left).unwrap();
        let right_run = coordinator.consume(&lock, &permuted_view, &consumer_request, &mut right).unwrap();
        prop_assert_eq!(left_run.canonical_bytes(), right_run.canonical_bytes());
        prop_assert_eq!(left_run.identity(), right_run.identity());
    }

    #[test]
    #[trace("TC-004", "FR-004-AC-3")]
    fn tc004_generated_one_over_bounds_preserve_the_exact_edge_frontier(
        cause in 0_u8..3,
        reverse_nodes in any::<bool>(),
        reverse_edges in any::<bool>(),
    ) {
        let mut bounded = limits();
        let expected_cause = match cause {
            0 => {
                bounded.max_traversal_steps = 3;
                BoundExceeded::TraversalSteps
            }
            1 => {
                bounded.max_results = 3;
                BoundExceeded::Results
            }
            _ => {
                bounded.max_edges = 4;
                BoundExceeded::Edges
            }
        };
        let view = view_with(projection(reverse_nodes, reverse_edges), bounded);
        let TraversalOutcome::Incomplete { cause, frontier } = view.outcome() else {
            return Err(TestCaseError::fail("one-over traversal claimed complete"));
        };
        prop_assert_eq!(*cause, expected_cause);
        prop_assert_eq!(frontier.len(), 2);
        prop_assert_eq!(&frontier[0].target, &semantic("gamma"));
        prop_assert_eq!(frontier[0].predecessor.as_ref(), Some(&semantic("alpha")));
        prop_assert_eq!(&frontier[1].target, &semantic("gamma"));
        prop_assert_eq!(frontier[1].predecessor.as_ref(), Some(&semantic("beta")));
    }

    #[test]
    #[trace("TC-004", "FR-004-AC-2")]
    fn tc004_mapping_loss_is_independent_of_facade_item_state(
        loss_case in 0_u8..4,
        outcome_case in 0_u8..3,
    ) {
        let loss = match loss_case {
            0 => MappingLoss::Preserved,
            1 => MappingLoss::Conditional,
            2 => MappingLoss::Unrepresented,
            _ => MappingLoss::Refused,
        };
        let requested = obligation("axis-matrix", "root", Some(64));
        let output = match outcome_case {
            0 => ConsumerOutput::Displayed {
                bytes: b"axis".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: loss,
            },
            1 => ConsumerOutput::Omitted {
                reason: "profile decision".to_owned(),
                mapping_loss: loss,
            },
            _ => ConsumerOutput::Generated {
                bytes: b"axis".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: loss,
            },
        };
        let mut consumer_manifest = manifest(&["axis-matrix"]);
        consumer_manifest.kind = ConsumerKind::Renderer;
        let mut consumer_request = request(vec![requested.clone()]);
        consumer_request.kind = ConsumerKind::Renderer;
        let mut consumer = TestConsumer::new(
            consumer_manifest,
            vec![completion(&requested, output)],
        );
        let run = ConsumerCoordinator::new(limits())
            .consume(
                &validated_lock(),
                &view_with(projection(false, false), limits()),
                &consumer_request,
                &mut consumer,
            )
            .unwrap();
        prop_assert_eq!(run.state(), CoordinationState::Complete);
        prop_assert_eq!(run.records()[0].mapping_loss, Some(loss));
        let correct_outcome = match outcome_case {
            0 => matches!(run.records()[0].outcome, AccountingOutcome::Displayed { .. }),
            1 => matches!(run.records()[0].outcome, AccountingOutcome::Omitted { .. }),
            _ => matches!(run.records()[0].outcome, AccountingOutcome::Generated { .. }),
        };
        prop_assert!(correct_outcome, "facade outcome changed with mapping-loss axis");
    }

    #[test]
    #[trace("TC-004", "FR-004-AC-5")]
    fn tc004_generated_presentation_text_cannot_mutate_checked_semantics(
        presentation in "[A-Za-z0-9 _-]{1,32}",
    ) {
        let lock = validated_lock();
        let view = view_with(projection(false, false), limits());
        let before = (view.canonical_bytes().to_vec(), view.package_digest());
        let requested = obligation("generated-presentation", "root", Some(64));
        let bytes = presentation.into_bytes();
        let end = u64::try_from(bytes.len()).unwrap();
        let mut consumer = TestConsumer::new(
            manifest(&["generated-presentation"]),
            vec![completion(
                &requested,
                ConsumerOutput::Generated {
                    bytes,
                    regions: vec![OutputRegion { start: 0, end }],
                    mapping_loss: MappingLoss::Conditional,
                },
            )],
        );
        let run = ConsumerCoordinator::new(limits())
            .consume(&lock, &view, &request(vec![requested]), &mut consumer)
            .unwrap();
        let generated = matches!(run.records()[0].outcome, AccountingOutcome::Generated { .. });
        prop_assert!(generated, "generated presentation lost its facade state");
        prop_assert_eq!(run.records()[0].mapping_loss, Some(MappingLoss::Conditional));
        prop_assert_eq!(view.canonical_bytes(), before.0);
        prop_assert_eq!(view.package_digest(), before.1);
    }
}

#[test]
#[trace("TC-004", "FR-004-AC-2")]
fn tc004_every_view_and_consumer_item_has_exact_identity_and_one_state() {
    let lock = validated_lock();
    let view = view_with(projection(false, false), limits());
    let mut mixed_sources = projection(false, false);
    mixed_sources
        .nodes
        .iter_mut()
        .find(|node| node.id.semantic.as_str() == "alpha")
        .expect("alpha fixture")
        .source
        .artifact = id!(ArtifactId, "included-source-artifact");
    let mut filtered_query = query();
    filtered_query.filter = ViewFilter::SourceArtifact(id!(ArtifactId, "checked-owner-artifact"));
    let filtered = ViewCoordinator::new(limits())
        .query(
            &lock,
            &owner(),
            &filtered_query,
            &TestReader::new(mixed_sources),
        )
        .expect("selected source filter");
    assert_eq!(
        filtered.items().len(),
        4,
        "filter cannot prune reachability"
    );
    assert_eq!(
        filtered
            .items()
            .iter()
            .find(|item| item.semantic.semantic.as_str() == "alpha")
            .expect("visited alpha")
            .state,
        ViewItemState::Omitted
    );
    let displayed = obligation("a:displayed", "alpha", Some(64));
    let omitted = obligation("b:omitted", "beta", Some(64));
    let generated = obligation("c:generated", "gamma", Some(64));
    let unsupported = obligation("d:unsupported", "root", Some(64));
    let requires = obligation("e:requires-bound", "root", None);
    let invalid = obligation("f:invalid", "foreign", Some(64));
    let failed = obligation("g:failed", "root", Some(64));
    let obligations = vec![
        displayed.clone(),
        omitted.clone(),
        generated.clone(),
        unsupported,
        requires,
        invalid,
        failed.clone(),
    ];
    let completions = vec![
        completion(
            &displayed,
            ConsumerOutput::Displayed {
                bytes: b"display".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 7 }],
                mapping_loss: MappingLoss::Preserved,
            },
        ),
        completion(
            &omitted,
            ConsumerOutput::Omitted {
                reason: "profile intentionally omits this presentation".to_owned(),
                mapping_loss: MappingLoss::Unrepresented,
            },
        ),
        completion(
            &generated,
            ConsumerOutput::Generated {
                bytes: b"generated".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 9 }],
                mapping_loss: MappingLoss::Conditional,
            },
        ),
        completion(
            &failed,
            ConsumerOutput::Failed {
                code: "tool-failed".to_owned(),
            },
        ),
    ];
    let mut consumer = TestConsumer::new(
        manifest(&[
            "a:displayed",
            "b:omitted",
            "c:generated",
            "e:requires-bound",
            "f:invalid",
            "g:failed",
        ]),
        completions,
    );
    let run = ConsumerCoordinator::new(limits())
        .consume(&lock, &view, &request(obligations), &mut consumer)
        .expect("complete bounded accounting envelope");
    assert_eq!(run.records().len(), 7);
    assert_eq!(consumer.calls, 1);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Displayed { .. }
    ));
    assert_eq!(run.records()[0].mapping_loss, Some(MappingLoss::Preserved));
    assert!(matches!(
        run.records()[1].outcome,
        AccountingOutcome::Omitted { .. }
    ));
    assert_eq!(
        run.records()[1].mapping_loss,
        Some(MappingLoss::Unrepresented)
    );
    assert!(matches!(
        run.records()[2].outcome,
        AccountingOutcome::Generated { .. }
    ));
    assert_eq!(
        run.records()[2].mapping_loss,
        Some(MappingLoss::Conditional)
    );
    assert!(matches!(
        run.records()[3].outcome,
        AccountingOutcome::Unsupported { .. }
    ));
    assert!(matches!(
        run.records()[4].outcome,
        AccountingOutcome::RequiresBound { .. }
    ));
    assert!(matches!(
        run.records()[5].outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::ForeignSemantic
        }
    ));
    assert!(matches!(
        run.records()[6].outcome,
        AccountingOutcome::Failed { .. }
    ));
    assert_eq!(run.state(), CoordinationState::Failed);
}

#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc004_exact_bounds_complete_and_one_over_has_exact_frontier() {
    let mut exact = limits();
    exact.max_traversal_steps = 4;
    exact.max_results = 4;
    exact.max_edges = 5;
    let complete = view_with(projection(false, false), exact);
    assert_eq!(complete.items().len(), 4);
    assert_eq!(complete.usage().traversal_steps, 4);
    assert_eq!(complete.usage().traversed_edges, 5);
    assert_eq!(complete.outcome(), &TraversalOutcome::Complete);

    for cause in [
        BoundExceeded::TraversalSteps,
        BoundExceeded::Results,
        BoundExceeded::Edges,
    ] {
        let mut bounded = limits();
        match cause {
            BoundExceeded::TraversalSteps => bounded.max_traversal_steps = 3,
            BoundExceeded::Results => bounded.max_results = 3,
            BoundExceeded::Edges => bounded.max_edges = 4,
            BoundExceeded::Duration => unreachable!(),
        }
        let incomplete = view_with(projection(false, false), bounded);
        let TraversalOutcome::Incomplete {
            cause: actual,
            frontier,
        } = incomplete.outcome()
        else {
            panic!("one-over reachable graph must be incomplete");
        };
        assert_eq!(*actual, cause);
        assert_eq!(incomplete.usage().traversal_steps, 3);
        assert_eq!(incomplete.usage().results, 3);
        assert_eq!(incomplete.usage().traversed_edges, 4);
        assert_eq!(
            frontier,
            &vec![
                FrontierEntry {
                    depth: 2,
                    target: semantic("gamma"),
                    predecessor: Some(semantic("alpha")),
                    relation: RelationKind::Contains,
                    provenance: source("edge:alpha:gamma"),
                },
                FrontierEntry {
                    depth: 2,
                    target: semantic("gamma"),
                    predecessor: Some(semantic("beta")),
                    relation: RelationKind::Contains,
                    provenance: source("edge:beta:gamma"),
                },
            ]
        );
    }

    let mut exact_frontier = limits();
    exact_frontier.max_results = 3;
    exact_frontier.max_frontier = 2;
    assert!(matches!(
        view_with(projection(false, false), exact_frontier).outcome(),
        TraversalOutcome::Incomplete { frontier, .. } if frontier.len() == 2
    ));

    let mut no_frontier = limits();
    no_frontier.max_results = 3;
    no_frontier.max_frontier = 1;
    let error = ViewCoordinator::new(no_frontier)
        .query(
            &validated_lock(),
            &owner(),
            &query(),
            &TestReader::new(projection(false, false)),
        )
        .expect_err("frontier envelope must fit independently");
    assert_eq!(error.resource(), Some(Resource::Frontier));

    let mut duration_limits = limits();
    duration_limits.max_duration_ms = 4;
    let duration_limited =
        ViewCoordinator::with_monotonic_clock(duration_limits, Arc::new(StepClock::default()))
            .query(
                &validated_lock(),
                &owner(),
                &query(),
                &TestReader::new(projection(false, false)),
            )
            .expect("duration exhaustion during traversal keeps exact frontier");
    let TraversalOutcome::Incomplete { cause, frontier } = duration_limited.outcome() else {
        panic!("duration one-over must be incomplete");
    };
    assert_eq!(*cause, BoundExceeded::Duration);
    assert_eq!(duration_limited.usage().traversal_steps, 1);
    assert_eq!(
        frontier
            .iter()
            .map(|entry| entry.target.semantic.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "beta"]
    );

    let mut duplicate_cleanup_limits = limits();
    duplicate_cleanup_limits.max_duration_ms = 8;
    let duplicate_cleanup = ViewCoordinator::with_monotonic_clock(
        duplicate_cleanup_limits,
        Arc::new(StepClock::default()),
    )
    .query(
        &validated_lock(),
        &owner(),
        &query(),
        &TestReader::new(projection(false, false)),
    )
    .expect("visited duplicate frontier entries require no timed semantic work");
    assert_eq!(duplicate_cleanup.outcome(), &TraversalOutcome::Complete);
    assert!(!duplicate_cleanup.duration_exhausted());

    let target_order_projection = ReaderProjection {
        package: id!(PackageId, "view-fixture"),
        artifact: id!(ArtifactId, "checked-owner-artifact"),
        digest: owner_digest(),
        nodes: vec![node("root"), node("a"), node("b"), node("c"), node("z")],
        edges: vec![
            edge("root", "a"),
            edge("root", "b"),
            edge("a", "z"),
            edge("b", "c"),
        ],
    };
    let mut target_order_limits = limits();
    target_order_limits.max_results = 3;
    let target_order = view_with(target_order_projection, target_order_limits);
    let TraversalOutcome::Incomplete { frontier, .. } = target_order.outcome() else {
        panic!("target-order fixture must stop with two edges");
    };
    assert_eq!(
        frontier
            .iter()
            .map(|entry| entry.target.semantic.as_str())
            .collect::<Vec<_>>(),
        vec!["c", "z"]
    );
}

#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-004-AC-4")]
fn tc004_feature_local_retention_and_invocation_limits_are_exact() {
    let lock = validated_lock();

    let owner_length = u64::try_from(owner().canonical_bytes.len()).unwrap();
    let mut exact_input = limits();
    exact_input.max_input_bytes = owner_length;
    assert!(ViewCoordinator::new(exact_input)
        .query(
            &lock,
            &owner(),
            &query(),
            &TestReader::new(projection(false, false)),
        )
        .is_ok());
    let mut one_over_input = exact_input;
    one_over_input.max_input_bytes -= 1;
    let reader = TestReader::new(projection(false, false));
    assert_eq!(
        ViewCoordinator::new(one_over_input)
            .query(&lock, &owner(), &query(), &reader)
            .expect_err("input one-over refuses before hashing or reading")
            .resource(),
        Some(Resource::InputBytes)
    );
    assert_eq!(reader.calls.get(), 0);

    let mut exact_projection = limits();
    exact_projection.max_source_artifacts = 2;
    exact_projection.max_nodes = 4;
    exact_projection.max_projection_edges = 5;
    let exact_projection_view = view_with(projection(false, false), exact_projection);
    assert_eq!(exact_projection_view.outcome(), &TraversalOutcome::Complete);
    assert_eq!(exact_projection_view.usage().reader_invocations, 1);
    assert_eq!(exact_projection_view.usage().source_artifacts, 2);
    assert_eq!(exact_projection_view.usage().projected_nodes, 4);
    assert_eq!(exact_projection_view.usage().projected_edges, 5);
    for (resource, bounded) in [
        {
            let mut value = exact_projection;
            value.max_source_artifacts = 1;
            (Resource::SourceArtifacts, value)
        },
        {
            let mut value = exact_projection;
            value.max_nodes = 3;
            (Resource::Nodes, value)
        },
        {
            let mut value = exact_projection;
            value.max_projection_edges = 4;
            (Resource::Edges, value)
        },
    ] {
        assert_eq!(
            ViewCoordinator::new(bounded)
                .query(
                    &lock,
                    &owner(),
                    &query(),
                    &TestReader::new(projection(false, false)),
                )
                .expect_err("projection retention one-over refuses")
                .resource(),
            Some(resource)
        );
    }

    let mut no_reader_call = exact_projection;
    no_reader_call.max_reader_invocations = 0;
    let reader = TestReader::new(projection(false, false));
    assert_eq!(
        ViewCoordinator::new(no_reader_call)
            .query(&lock, &owner(), &query(), &reader)
            .expect_err("zero reader invocation ceiling refuses before call")
            .resource(),
        Some(Resource::ReaderInvocations)
    );
    assert_eq!(reader.calls.get(), 0);

    let baseline = view_with(projection(false, false), limits());
    let encoded_length = u64::try_from(baseline.canonical_bytes().len()).unwrap();
    let mut exact_accounting = limits();
    exact_accounting.max_accounting_bytes = encoded_length;
    assert!(!view_with(projection(false, false), exact_accounting)
        .canonical_bytes()
        .is_empty());
    let mut one_over_accounting = exact_accounting;
    one_over_accounting.max_accounting_bytes -= 1;
    assert_eq!(
        ViewCoordinator::new(one_over_accounting)
            .query(
                &lock,
                &owner(),
                &query(),
                &TestReader::new(projection(false, false)),
            )
            .expect_err("accounting envelope one-over refuses")
            .resource(),
        Some(Resource::AccountingBytes)
    );

    let view = view_with(projection(false, false), limits());
    let requested = obligation("bounded-call", "root", Some(3));
    let exact_completion = completion(
        &requested,
        ConsumerOutput::Generated {
            bytes: b"abc".to_vec(),
            regions: vec![
                OutputRegion { start: 0, end: 1 },
                OutputRegion { start: 1, end: 2 },
                OutputRegion { start: 2, end: 3 },
            ],
            mapping_loss: MappingLoss::Preserved,
        },
    );
    let mut exact_consumer_limits = limits();
    exact_consumer_limits.max_obligations = 1;
    exact_consumer_limits.max_consumer_invocations = 1;
    exact_consumer_limits.max_mappings = 3;
    exact_consumer_limits.max_output_bytes = 3;
    let mut exact_consumer =
        TestConsumer::new(manifest(&["bounded-call"]), vec![exact_completion.clone()]);
    let exact_run = ConsumerCoordinator::new(exact_consumer_limits)
        .consume(
            &lock,
            &view,
            &request(vec![requested.clone()]),
            &mut exact_consumer,
        )
        .expect("all exact consumer limits complete");
    assert_eq!(exact_run.state(), CoordinationState::Complete);
    assert_eq!(
        exact_run.usage(),
        ConsumerUsage {
            invocations: 1,
            output_bytes: 3,
            mappings: 3,
        }
    );

    let mut result_one_over = TestConsumer::new(
        manifest(&["bounded-call"]),
        vec![exact_completion.clone(), exact_completion.clone()],
    );
    let run = ConsumerCoordinator::new(exact_consumer_limits)
        .consume(
            &lock,
            &view,
            &request(vec![requested.clone()]),
            &mut result_one_over,
        )
        .expect("result one-over discards the previously accepted sibling payload");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::ConsumerResults,
            minimum: 2
        }
    ));

    let mut no_invocation = exact_consumer_limits;
    no_invocation.max_consumer_invocations = 0;
    let mut consumer = TestConsumer::new(manifest(&["bounded-call"]), Vec::new());
    let run = ConsumerCoordinator::new(no_invocation)
        .consume(
            &lock,
            &view,
            &request(vec![requested.clone()]),
            &mut consumer,
        )
        .expect("zero invocation ceiling remains accountable");
    assert_eq!(consumer.calls, 0);
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert_eq!(run.usage().invocations, 0);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::Invocations,
            minimum: 1
        }
    ));

    let mut one_over_mappings = exact_consumer_limits;
    one_over_mappings.max_mappings = 2;
    let mut consumer = TestConsumer::new(manifest(&["bounded-call"]), vec![exact_completion]);
    consumer.error_after_submissions = Some(ConsumerError::Exhausted);
    let run = ConsumerCoordinator::new(one_over_mappings)
        .consume(&lock, &view, &request(vec![requested]), &mut consumer)
        .expect("mapping one-over remains accountable");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert_eq!(run.usage().mappings, 3);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::Mappings,
            minimum: 3
        }
    ));

    let alpha = obligation("aggregate-alpha", "alpha", Some(2));
    let beta = obligation("aggregate-beta", "beta", Some(2));
    let aggregate_completions = vec![
        completion(
            &alpha,
            ConsumerOutput::Generated {
                bytes: b"ab".to_vec(),
                regions: vec![
                    OutputRegion { start: 0, end: 1 },
                    OutputRegion { start: 1, end: 2 },
                ],
                mapping_loss: MappingLoss::Conditional,
            },
        ),
        completion(
            &beta,
            ConsumerOutput::Displayed {
                bytes: b"cd".to_vec(),
                regions: vec![
                    OutputRegion { start: 0, end: 1 },
                    OutputRegion { start: 1, end: 2 },
                ],
                mapping_loss: MappingLoss::Refused,
            },
        ),
    ];
    let aggregate_request = request(vec![alpha.clone(), beta.clone()]);
    let mut aggregate_exact = limits();
    aggregate_exact.max_obligations = 2;
    aggregate_exact.max_mappings = 4;
    aggregate_exact.max_output_bytes = 4;
    let mut consumer = TestConsumer::new(
        manifest(&["aggregate-alpha", "aggregate-beta"]),
        aggregate_completions.clone(),
    );
    let run = ConsumerCoordinator::new(aggregate_exact)
        .consume(&lock, &view, &aggregate_request, &mut consumer)
        .expect("aggregate exact mapping/output limits complete");
    assert_eq!(run.state(), CoordinationState::Complete);
    assert_eq!(run.usage().mappings, 4);
    assert_eq!(run.usage().output_bytes, 4);

    let mut aggregate_one_over = aggregate_exact;
    aggregate_one_over.max_mappings = 3;
    let mut consumer = TestConsumer::new(
        manifest(&["aggregate-alpha", "aggregate-beta"]),
        aggregate_completions,
    );
    let run = ConsumerCoordinator::new(aggregate_one_over)
        .consume(&lock, &view, &aggregate_request, &mut consumer)
        .expect("aggregate mapping one-over retains no partial package");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert_eq!(run.usage().mappings, 4);
    assert!(run.records().iter().all(|record| matches!(
        record.outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::Mappings,
            minimum: 4
        }
    )));

    let mut obligation_one_over = aggregate_exact;
    obligation_one_over.max_obligations = 1;
    let mut consumer =
        TestConsumer::new(manifest(&["aggregate-alpha", "aggregate-beta"]), Vec::new());
    assert_eq!(
        ConsumerCoordinator::new(obligation_one_over)
            .consume(&lock, &view, &aggregate_request, &mut consumer)
            .expect_err("obligation one-over refuses before invocation")
            .resource(),
        Some(Resource::Obligations)
    );
    assert_eq!(consumer.calls, 0);
}

#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-004-AC-4")]
fn tc004_post_finalization_deadline_is_accounted_for_every_prior_state() {
    let lock = validated_lock();
    let view = view_with(projection(false, false), limits());

    let completed = obligation("duration-complete", "root", Some(8));
    let mut consumer = TestConsumer::new(
        manifest(&["duration-complete"]),
        vec![completion(
            &completed,
            ConsumerOutput::Generated {
                bytes: b"ok".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 2 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let mut bounded = limits();
    bounded.max_duration_ms = 3;
    let run = ConsumerCoordinator::with_monotonic_clock(bounded, Arc::new(StepClock::default()))
        .consume(&lock, &view, &request(vec![completed]), &mut consumer)
        .expect("finalization timeout remains accountable");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert!(run.duration_exhausted());
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::Duration,
            ..
        }
    ));

    let failed = obligation("duration-failed", "root", Some(8));
    let mut consumer = TestConsumer::new(manifest(&["duration-failed"]), Vec::new());
    consumer.error = Some(ConsumerError::Failed);
    bounded.max_duration_ms = 2;
    let run = ConsumerCoordinator::with_monotonic_clock(bounded, Arc::new(StepClock::default()))
        .consume(&lock, &view, &request(vec![failed]), &mut consumer)
        .expect("failed state retains finalization timeout fact");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(run.duration_exhausted());

    let unsupported = obligation("duration-refused", "root", Some(8));
    let mut consumer = TestConsumer::new(manifest(&[]), Vec::new());
    let run = ConsumerCoordinator::with_monotonic_clock(bounded, Arc::new(StepClock::default()))
        .consume(&lock, &view, &request(vec![unsupported]), &mut consumer)
        .expect("refused accounting records finalization timeout");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert!(run.duration_exhausted());

    let incomplete = obligation("duration-incomplete", "root", Some(8));
    let mut consumer = TestConsumer::new(manifest(&["duration-incomplete"]), Vec::new());
    bounded.max_consumer_invocations = 0;
    let run = ConsumerCoordinator::with_monotonic_clock(bounded, Arc::new(StepClock::default()))
        .consume(&lock, &view, &request(vec![incomplete]), &mut consumer)
        .expect("incomplete accounting records finalization timeout");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert!(run.duration_exhausted());
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn tc004_adverse_phases_have_deterministic_typed_outcomes() {
    let lock = validated_lock();
    let mut bad_owner = owner();
    bad_owner.digest = [0; 32];
    let reader = TestReader::new(projection(false, false));
    let error = ViewCoordinator::new(limits())
        .query(&lock, &bad_owner, &query(), &reader)
        .expect_err("digest mismatch refuses before reader invocation");
    assert_eq!(error.code(), ErrorCode::ForeignInput);
    assert_eq!(reader.calls.get(), 0);

    let mut bad_relation = query();
    bad_relation.relation.contract_version = "foreign.relation/v1".to_owned();
    let reader = TestReader::new(projection(false, false));
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &bad_relation, &reader)
            .expect_err("foreign relation refuses")
            .code(),
        ErrorCode::ForeignInput
    );
    assert_eq!(reader.calls.get(), 0);

    let view = view_with(projection(false, false), limits());
    let requested = obligation("supported", "root", Some(512));
    let mut wrong_manifest = manifest(&["supported"]);
    wrong_manifest.profile = ProfileId::new("profile:other", ID_LIMIT).unwrap();
    let mut consumer = TestConsumer::new(wrong_manifest, Vec::new());
    assert_eq!(
        ConsumerCoordinator::new(limits())
            .consume(
                &lock,
                &view,
                &request(vec![requested.clone()]),
                &mut consumer
            )
            .expect_err("profile mismatch refuses before consumer invocation")
            .code(),
        ErrorCode::ForeignInput
    );
    assert_eq!(consumer.calls, 0);

    let duplicate = completion(
        &requested,
        ConsumerOutput::Generated {
            bytes: b"ok".to_vec(),
            regions: vec![OutputRegion { start: 0, end: 2 }],
            mapping_loss: MappingLoss::Preserved,
        },
    );
    let mut malformed =
        TestConsumer::new(manifest(&["supported"]), vec![duplicate.clone(), duplicate]);
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![requested.clone()]),
            &mut malformed,
        )
        .expect("malformed admitted batch becomes accounting");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::InventoryMismatch
        }
    ));

    let foreign_completion = obligation("foreign-completion", "root", Some(8));
    let mut invalid_then_failed = TestConsumer::new(
        manifest(&["supported"]),
        vec![completion(
            &foreign_completion,
            ConsumerOutput::Displayed {
                bytes: b"root".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    invalid_then_failed.error_after_submissions = Some(ConsumerError::Failed);
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![requested.clone()]),
            &mut invalid_then_failed,
        )
        .expect("sink invalidity takes precedence over a later consumer error");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::InventoryMismatch
        }
    ));

    let mut oversized = TestConsumer::new(
        manifest(&["supported"]),
        vec![completion(
            &requested,
            ConsumerOutput::Generated {
                bytes: vec![b'x'; 32],
                regions: vec![OutputRegion { start: 0, end: 32 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let mut output_limits = limits();
    output_limits.max_output_bytes = 16;
    let run = ConsumerCoordinator::new(output_limits)
        .consume(&lock, &view, &request(vec![requested]), &mut oversized)
        .expect("output exhaustion retains bounded accounting");
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound {
            resource: RequiredBound::OutputBytes,
            minimum: 32
        }
    ));

    let reader = TestReader::failing(projection(false, false), ReaderError::Failed);
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &query(), &reader)
            .expect_err("selected reader failure is typed")
            .code(),
        ErrorCode::ReaderFailure
    );
    assert_eq!(reader.calls.get(), 1);

    let reader = TestReader::failing(
        projection(false, false),
        ReaderError::Exhausted(ReaderResource::Nodes),
    );
    let error = ViewCoordinator::new(limits())
        .query(&lock, &owner(), &query(), &reader)
        .expect_err("reader node exhaustion retains its exact resource");
    assert_eq!(error.resource(), Some(Resource::Nodes));
    assert_eq!(reader.calls.get(), 1);

    let unsupported = obligation("unsupported-only", "root", Some(8));
    let missing_bound = obligation("missing-bound-only", "root", None);
    let mut local_only = TestConsumer::new(manifest(&["missing-bound-only"]), Vec::new());
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![unsupported, missing_bound]),
            &mut local_only,
        )
        .expect("item-local refusals retain accounting");
    assert_eq!(local_only.calls, 0);
    assert_eq!(run.state(), CoordinationState::Refused);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::RequiresBound { .. }
    ));
    assert!(matches!(
        run.records()[1].outcome,
        AccountingOutcome::Unsupported { .. }
    ));

    let valid_sibling = obligation("valid-sibling", "root", Some(8));
    let mut foreign_definition = obligation("foreign-definition", "root", Some(8));
    foreign_definition.semantic.definition = id!(DefinitionId, "foreign-definition:v1");
    let mut mixed_consumer = TestConsumer::new(
        manifest(&["foreign-definition", "valid-sibling"]),
        vec![completion(
            &valid_sibling,
            ConsumerOutput::Displayed {
                bytes: b"root".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![foreign_definition, valid_sibling]),
            &mut mixed_consumer,
        )
        .expect("foreign obligation definitions are item-local invalid");
    assert_eq!(mixed_consumer.calls, 1);
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(run.records().iter().any(|record| matches!(
        record.outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::ForeignSemantic
        }
    )));
    assert!(run
        .records()
        .iter()
        .any(|record| matches!(record.outcome, AccountingOutcome::Displayed { .. })));

    let failed_request = obligation("consumer-error", "root", Some(8));
    let mut failed_consumer = TestConsumer::new(manifest(&["consumer-error"]), Vec::new());
    failed_consumer.error = Some(ConsumerError::Failed);
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![failed_request]),
            &mut failed_consumer,
        )
        .expect("admitted consumer error is accountable");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Failed {
            code: FailureCode::ConsumerFailed,
            ..
        }
    ));

    let exhausted_request = obligation("consumer-exhausted", "root", Some(8));
    let mut exhausted_consumer = TestConsumer::new(manifest(&["consumer-exhausted"]), Vec::new());
    exhausted_consumer.error = Some(ConsumerError::Exhausted);
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![exhausted_request]),
            &mut exhausted_consumer,
        )
        .expect("admitted consumer exhaustion is accountable");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert_eq!(run.usage().invocations, 1);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Failed {
            code: FailureCode::ConsumerExhausted,
            ..
        }
    ));

    let mismatch_request = obligation("lock-mismatch", "root", Some(8));
    let mut mismatch_consumer = TestConsumer::new(manifest(&["lock-mismatch"]), Vec::new());
    assert_eq!(
        ConsumerCoordinator::new(limits())
            .consume(
                &validated_lock_for("other-deployment"),
                &view,
                &request(vec![mismatch_request]),
                &mut mismatch_consumer,
            )
            .expect_err("view and invocation lock identities must match")
            .code(),
        ErrorCode::CapabilityMismatch
    );
    assert_eq!(mismatch_consumer.calls, 0);

    let mut truncated_limits = limits();
    truncated_limits.max_results = 1;
    let truncated = view_with(projection(false, false), truncated_limits);
    let partial_request = obligation("partial-view", "root", Some(8));
    let mut partial_consumer = TestConsumer::new(
        manifest(&["partial-view"]),
        vec![completion(
            &partial_request,
            ConsumerOutput::Displayed {
                bytes: b"root".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &truncated,
            &request(vec![partial_request]),
            &mut partial_consumer,
        )
        .expect("consumer accounting preserves source-view incompleteness");
    assert_eq!(partial_consumer.calls, 1);
    assert_eq!(run.state(), CoordinationState::Incomplete);
    assert_eq!(run.view_outcome(), truncated.outcome());
    assert_eq!(run.view_usage(), truncated.usage());
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn tc004_foreign_filters_profiles_and_malformed_projections_refuse_stably() {
    let lock = validated_lock();

    let mut foreign = query();
    foreign.filter = ViewFilter::SourceArtifact(id!(ArtifactId, "foreign-artifact"));
    let reader = TestReader::new(projection(false, false));
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &foreign, &reader)
            .expect_err("foreign filter refuses before read")
            .code(),
        ErrorCode::ForeignInput
    );
    assert_eq!(reader.calls.get(), 0);

    let mut foreign_profile = query();
    foreign_profile.profile = id!(ArtifactId, "foreign-profile");
    let reader = TestReader::new(projection(false, false));
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &foreign_profile, &reader)
            .expect_err("foreign profile refuses before read")
            .code(),
        ErrorCode::ForeignInput
    );
    assert_eq!(reader.calls.get(), 0);

    let mut duplicate_nodes = projection(false, false);
    duplicate_nodes.nodes.push(node("root"));
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &query(), &TestReader::new(duplicate_nodes),)
            .expect_err("duplicate semantics are malformed")
            .code(),
        ErrorCode::Malformed
    );

    let mut dangling = projection(false, false);
    dangling.edges.push(edge("root", "foreign"));
    assert_eq!(
        ViewCoordinator::new(limits())
            .query(&lock, &owner(), &query(), &TestReader::new(dangling))
            .expect_err("dangling edges are malformed")
            .code(),
        ErrorCode::Malformed
    );
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn tc004_malformed_consumer_outputs_never_escape_as_artifacts() {
    let lock = validated_lock();
    let view = view_with(projection(false, false), limits());
    let overlap = obligation("a:overlap", "alpha", Some(64));
    let non_utf8 = obligation("b:non-utf8", "beta", Some(64));
    let malformed_range = obligation("c:malformed-range", "gamma", Some(64));
    let failed = obligation("d:failed", "root", Some(64));
    let completions = vec![
        completion(
            &overlap,
            ConsumerOutput::Generated {
                bytes: b"abcdef".to_vec(),
                regions: vec![
                    OutputRegion { start: 0, end: 4 },
                    OutputRegion { start: 3, end: 6 },
                ],
                mapping_loss: MappingLoss::Preserved,
            },
        ),
        completion(
            &non_utf8,
            ConsumerOutput::Displayed {
                bytes: vec![0xff],
                regions: vec![OutputRegion { start: 0, end: 1 }],
                mapping_loss: MappingLoss::Conditional,
            },
        ),
        completion(
            &malformed_range,
            ConsumerOutput::Generated {
                bytes: b"abc".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 4 }],
                mapping_loss: MappingLoss::Preserved,
            },
        ),
        completion(
            &failed,
            ConsumerOutput::Failed {
                code: "selected-tool-failed".to_owned(),
            },
        ),
    ];
    let mut consumer = TestConsumer::new(
        manifest(&["a:overlap", "b:non-utf8", "c:malformed-range", "d:failed"]),
        completions,
    );
    let run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &view,
            &request(vec![overlap, non_utf8, malformed_range, failed]),
            &mut consumer,
        )
        .expect("malformed admitted outputs remain bounded accounting");
    assert_eq!(run.state(), CoordinationState::Failed);
    assert!(run.records()[..2].iter().all(|record| matches!(
        record.outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::MalformedMapping
        }
    )));
    assert!(matches!(
        run.records()[2].outcome,
        AccountingOutcome::Invalid {
            code: InvalidCode::MalformedMapping
        }
    ));
    assert!(matches!(
        run.records()[3].outcome,
        AccountingOutcome::Failed {
            code: FailureCode::ConsumerFailed,
            ..
        }
    ));

    let binary = obligation("binary-profile", "root", Some(1));
    let mut binary_manifest = manifest(&["binary-profile"]);
    binary_manifest.region_encoding = RegionEncoding::Binary;
    let mut binary_request = request(vec![binary.clone()]);
    binary_request.region_encoding = RegionEncoding::Binary;
    let mut consumer = TestConsumer::new(
        binary_manifest,
        vec![completion(
            &binary,
            ConsumerOutput::Displayed {
                bytes: vec![0xff],
                regions: vec![OutputRegion { start: 0, end: 1 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let run = ConsumerCoordinator::new(limits())
        .consume(&lock, &view, &binary_request, &mut consumer)
        .expect("binary profile admits byte-aligned non-UTF8 regions");
    assert_eq!(run.state(), CoordinationState::Complete);
    assert!(matches!(
        run.records()[0].outcome,
        AccountingOutcome::Displayed { .. }
    ));
}

#[test]
#[trace("TC-004", "FR-004-AC-1")]
fn tc004_every_meaning_affecting_input_participates_in_identity() {
    let lock = validated_lock();
    let baseline = view_with(projection(false, false), limits());

    let mut changed_limits = limits();
    changed_limits.max_nodes -= 1;
    let limited = view_with(projection(false, false), changed_limits);
    assert_ne!(baseline.identity(), limited.identity());

    let mut projected_query = query();
    projected_query.projection = ViewProjection::IdentityOnly;
    let projected = ViewCoordinator::new(limits())
        .query(
            &lock,
            &owner(),
            &projected_query,
            &TestReader::new(projection(false, false)),
        )
        .unwrap();
    assert_ne!(baseline.identity(), projected.identity());
    assert!(baseline
        .items()
        .iter()
        .all(|item| item.projected_source.is_some()));
    assert!(projected
        .items()
        .iter()
        .all(|item| item.projected_source.is_none()));

    let mut alternate_owner = owner();
    alternate_owner.artifact = id!(ArtifactId, "included-source-artifact");
    let mut alternate_projection = projection(false, false);
    alternate_projection.artifact = alternate_owner.artifact.clone();
    let mut alternate_query = query();
    alternate_query.owner_artifact = alternate_owner.artifact.clone();
    let alternate = ViewCoordinator::new(limits())
        .query(
            &lock,
            &alternate_owner,
            &alternate_query,
            &TestReader::new(alternate_projection),
        )
        .unwrap();
    assert_ne!(baseline.identity(), alternate.identity());

    let requested = obligation("identity", "root", Some(64));
    let generated = completion(
        &requested,
        ConsumerOutput::Generated {
            bytes: b"identity".to_vec(),
            regions: vec![OutputRegion { start: 0, end: 8 }],
            mapping_loss: MappingLoss::Preserved,
        },
    );
    let mut codegen = TestConsumer::new(manifest(&["identity"]), vec![generated.clone()]);
    let codegen_run = ConsumerCoordinator::new(limits())
        .consume(
            &lock,
            &baseline,
            &request(vec![requested.clone()]),
            &mut codegen,
        )
        .unwrap();
    let mut binary_manifest = manifest(&["identity"]);
    binary_manifest.region_encoding = RegionEncoding::Binary;
    let mut binary_request = request(vec![requested.clone()]);
    binary_request.region_encoding = RegionEncoding::Binary;
    let mut binary_consumer = TestConsumer::new(binary_manifest, vec![generated.clone()]);
    let binary_run = ConsumerCoordinator::new(limits())
        .consume(&lock, &baseline, &binary_request, &mut binary_consumer)
        .unwrap();
    assert_ne!(codegen_run.identity(), binary_run.identity());

    let mut wider = requested.clone();
    wider.output_bound = Some(65);
    let mut wider_consumer = TestConsumer::new(
        manifest(&["identity"]),
        vec![completion(
            &wider,
            ConsumerOutput::Generated {
                bytes: b"identity".to_vec(),
                regions: vec![OutputRegion { start: 0, end: 8 }],
                mapping_loss: MappingLoss::Preserved,
            },
        )],
    );
    let wider_run = ConsumerCoordinator::new(limits())
        .consume(&lock, &baseline, &request(vec![wider]), &mut wider_consumer)
        .unwrap();
    assert_ne!(codegen_run.identity(), wider_run.identity());

    let mut analyzer_manifest = manifest(&["identity"]);
    analyzer_manifest.kind = ConsumerKind::Analyzer;
    let mut analyzer_request = request(vec![requested]);
    analyzer_request.kind = ConsumerKind::Analyzer;
    let mut analyzer = TestConsumer::new(analyzer_manifest, vec![generated]);
    let analyzer_run = ConsumerCoordinator::new(limits())
        .consume(&lock, &baseline, &analyzer_request, &mut analyzer)
        .unwrap();
    assert_ne!(codegen_run.identity(), analyzer_run.identity());
}

#[test]
#[trace("TC-004", "FR-004-AC-5")]
fn tc004_presentation_and_parser_metamorphisms_do_not_change_semantics() {
    let lock = validated_lock();
    let view = view_with(projection(false, false), limits());
    let semantic_bytes = view.canonical_bytes().to_vec();
    let source_digest = view.package_digest();
    let requested = obligation("presentation", "root", Some(1024));

    for bytes in [b"root -> alpha".to_vec(), b"ROOT beside ALPHA".to_vec()] {
        let length = u64::try_from(bytes.len()).unwrap();
        let mut consumer = TestConsumer::new(
            manifest(&["presentation"]),
            vec![completion(
                &requested,
                ConsumerOutput::Generated {
                    bytes,
                    regions: vec![OutputRegion {
                        start: 0,
                        end: length,
                    }],
                    mapping_loss: MappingLoss::Conditional,
                },
            )],
        );
        let run = ConsumerCoordinator::new(limits())
            .consume(
                &lock,
                &view,
                &request(vec![requested.clone()]),
                &mut consumer,
            )
            .unwrap();
        assert!(matches!(
            run.records()[0].outcome,
            AccountingOutcome::Generated { .. }
        ));
        assert_eq!(
            run.records()[0].mapping_loss,
            Some(MappingLoss::Conditional)
        );
        assert_eq!(view.canonical_bytes(), semantic_bytes);
        assert_eq!(view.package_digest(), source_digest);
    }
}

#[test]
#[trace("TC-004", "FR-004-AC-6")]
fn tc004_facade_types_do_not_implement_owner_promotion_traits() {
    macro_rules! assert_not_impl {
        ($subject:ty: $promotes:path) => {{
            trait AmbiguousIfImplemented<Marker> {
                fn check() {}
            }
            struct PromotionMarker;
            impl<T: ?Sized> AmbiguousIfImplemented<()> for T {}
            impl<T: ?Sized + $promotes> AmbiguousIfImplemented<PromotionMarker> for T {}
            let _ = <$subject as AmbiguousIfImplemented<_>>::check;
        }};
    }

    assert_not_impl!(ConsumerRun: Into<OwnerPackage>);
    assert_not_impl!(AccountingRecord: Into<ReaderProjection>);
}
