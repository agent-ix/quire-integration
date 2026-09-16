// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-005: bounded compatibility impact and stale-evidence classification.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_integration::compatibility_impact::*;
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

fn contracts() -> [InputContract; 6] {
    [
        InputContract::Comparison,
        InputContract::Graph,
        InputContract::Corpus,
        InputContract::Evidence,
        InputContract::Policy,
        InputContract::Manifest,
    ]
}

fn selection(contract: InputContract) -> CapabilitySelection {
    let suffix = match contract {
        InputContract::Comparison => "comparison",
        InputContract::Graph => "graph",
        InputContract::Corpus => "corpus",
        InputContract::Evidence => "evidence",
        InputContract::Policy => "policy",
        InputContract::Manifest => "manifest",
    };
    CapabilitySelection::new(
        id!(RoleId, format!("impact-reader:{suffix}")),
        id!(ProviderId, format!("impact-provider:{suffix}")),
        id!(BuildId, format!("impact-build:{suffix}")),
        id!(ContractId, contract.label()),
        id!(ArtifactId, format!("impact-options:{suffix}")),
    )
}

fn vector_provider_selection() -> CapabilitySelection {
    CapabilitySelection::new(
        id!(RoleId, "verification"),
        id!(ProviderId, "verification-provider"),
        id!(BuildId, "verification-build"),
        id!(ContractId, "quire.vector-provider/v1"),
        id!(ArtifactId, "verification-options:v1"),
    )
}

fn validated_lock(
    deployment: &str,
    seed: char,
) -> quire_integration::ecosystem_lock::ValidatedLock {
    let repository = id!(RepositoryId, "agent-ix/impact-fixture");
    let package = id!(PackageId, "impact-fixture");
    let component = id!(ComponentId, "impact-fixture");
    let definition = id!(DefinitionId, "impact-model");
    let mut selections: Vec<_> = contracts().into_iter().map(selection).collect();
    selections.push(vector_provider_selection());
    let lock = LockSelection::new(
        vec![RepositoryBinding::new(
            exact(repository.clone(), 'a'),
            "https://github.com/agent-ix/impact-fixture",
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
        selections
            .iter()
            .enumerate()
            .map(|(index, value)| {
                exact(
                    value.provider().clone(),
                    ['d', 'e', 'f', '1', '2', '3', '4'][index],
                )
            })
            .collect(),
        vec![exact(id!(ToolchainId, "rust-1.98-or-later"), 'a')],
        selections
            .iter()
            .enumerate()
            .map(|(index, value)| {
                exact(
                    value.build().clone(),
                    ['5', '6', '7', '8', '9', 'a', 'b'][index],
                )
            })
            .collect(),
        vec![exact(id!(DeploymentId, deployment), seed)],
        vec![ComponentBinding::new(
            exact(component.clone(), 'b'),
            repository,
            package,
        )],
        selections
            .iter()
            .enumerate()
            .map(|(index, value)| {
                RoleBinding::new(
                    exact(
                        value.role().clone(),
                        ['a', 'b', 'c', 'd', 'e', 'f', '1'][index],
                    ),
                    component.clone(),
                )
            })
            .collect(),
        selections
            .iter()
            .cloned()
            .map(CapabilityBinding::new)
            .collect(),
        vec![WorkflowBinding::new(
            exact(id!(WorkflowInstanceId, "impact:fixture"), 'c'),
            definition,
        )],
        vec![],
    );
    let limits =
        LockLimits::for_selection(&lock, 128 * 1024, ID_LIMIT, 10_000).expect("lock limits");
    validate(&lock, limits).expect("validated exact lock")
}

fn limits() -> Limits {
    Limits {
        max_document_bytes: 4096,
        max_total_input_bytes: 24 * 1024,
        max_identity_bytes: ID_LIMIT,
        max_reason_bytes: 256,
        max_correspondences: 16,
        max_assessments: 16,
        max_changed_roots: 16,
        max_graph_nodes: 32,
        max_graph_edges: 64,
        max_evidence_records: 16,
        max_evidence_rights: 16,
        max_evidence_limitations: 16,
        max_reader_invocations: 6,
        max_policy_seed_rules: 64,
        max_policy_propagation_rules: 32,
        max_policy_vector_rules: 32,
        max_vector_definitions: 16,
        max_manifest_entries: 16,
        max_manifest_providers: 16,
        max_unresolved_entries: 16,
        max_total_reason_bytes: 1024,
        max_topology_items: 128,
        max_traversal_steps: 128,
        max_traversed_edges: 128,
        max_subject_records: 128,
        max_path_edges: 256,
        max_witnesses: 128,
        max_witness_bytes: 128 * 1024,
        max_frontier_entries: 128,
        max_output_bytes: 1024 * 1024,
        max_accounting_bytes: 2 * 1024 * 1024,
        max_duration_ms: 10_000,
    }
}

fn subject(kind: SubjectKind, local: &str, revision: &str) -> QualifiedSubjectId {
    QualifiedSubjectId {
        kind,
        authority: AuthorityId::new("agent-ix/impact-owner", ID_LIMIT).expect("authority"),
        revision: RevisionId::new(revision, ID_LIMIT).expect("revision"),
        local: SubjectId::new(local, ID_LIMIT).expect("subject"),
    }
}

fn relation() -> RelationId {
    RelationId::new("depends-on", ID_LIMIT).expect("relation")
}

fn scope(value: &str) -> ScopeId {
    ScopeId::new(value, ID_LIMIT).expect("scope")
}

fn provenance(value: &str) -> ProvenanceId {
    ProvenanceId::new(value, ID_LIMIT).expect("provenance")
}

fn all_seed_rules(active: AssessmentTuple, active_effects: BTreeSet<Effect>) -> Vec<SeedRule> {
    let structural = [
        StructuralAssessment::Unchanged,
        StructuralAssessment::AdditiveCompatible,
        StructuralAssessment::PayloadBreaking,
        StructuralAssessment::Removed,
        StructuralAssessment::Unknown,
    ];
    let behavioral = [
        BehavioralAssessment::NotAssessed,
        BehavioralAssessment::Conforming,
        BehavioralAssessment::Regressed,
        BehavioralAssessment::Absent,
        BehavioralAssessment::Unsupported,
        BehavioralAssessment::Unavailable,
        BehavioralAssessment::Unknown,
    ];
    let presence = [
        SubjectPresence::BothPresent,
        SubjectPresence::BaselineOnly,
        SubjectPresence::CandidateOnly,
        SubjectPresence::Unknown,
    ];
    let mut result = Vec::new();
    for structural in structural {
        for behavioral in behavioral {
            for presence in presence {
                let structural_ok = match structural {
                    StructuralAssessment::Unchanged | StructuralAssessment::PayloadBreaking => {
                        presence == SubjectPresence::BothPresent
                    }
                    StructuralAssessment::Removed => presence == SubjectPresence::BaselineOnly,
                    StructuralAssessment::AdditiveCompatible => matches!(
                        presence,
                        SubjectPresence::BothPresent | SubjectPresence::CandidateOnly
                    ),
                    StructuralAssessment::Unknown => true,
                };
                let behavior_ok = !matches!(
                    behavioral,
                    BehavioralAssessment::Conforming
                        | BehavioralAssessment::Regressed
                        | BehavioralAssessment::Absent
                ) || presence == SubjectPresence::BothPresent;
                if structural_ok && behavior_ok {
                    let assessment = AssessmentTuple {
                        structural,
                        behavioral,
                        presence,
                    };
                    result.push(SeedRule {
                        id: RuleId::new(
                            format!("seed:{structural:?}:{behavioral:?}:{presence:?}"),
                            ID_LIMIT,
                        )
                        .expect("seed rule"),
                        assessment,
                        effects: if assessment == active {
                            active_effects.clone()
                        } else {
                            BTreeSet::new()
                        },
                    });
                }
            }
        }
    }
    result
}

#[derive(Clone)]
struct FixtureReader {
    selection: CapabilitySelection,
    projection: Projection,
    failure: Option<ReaderError>,
    calls: usize,
    poison_sink: bool,
    graph_vocabulary_first: bool,
}

impl StrictImpactReader for FixtureReader {
    fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    fn read(
        &mut self,
        _admission: &AdmittedCapability<'_>,
        document: &OwnerDocument,
        budget: ReaderBudget,
        sink: &mut ProjectionSink,
    ) -> std::result::Result<(), ReaderError> {
        self.calls += 1;
        assert_eq!(
            document.contract.label(),
            self.selection.contract().as_str()
        );
        assert!(budget.remaining_duration_ms() <= budget.limits().max_duration_ms);
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if self.poison_sink {
            let _ = sink.begin(ProjectionHeader::Corpus);
        }
        emit_projection(sink, self.projection.clone(), self.graph_vocabulary_first)
    }
}

fn emit_projection(
    sink: &mut ProjectionSink,
    projection: Projection,
    graph_vocabulary_first: bool,
) -> std::result::Result<(), ReaderError> {
    let (header, items) = match projection {
        Projection::Comparison(value) => (
            ProjectionHeader::Comparison {
                baseline_lock: value.baseline_lock,
                candidate_lock: value.candidate_lock,
                comparison_rule: value.comparison_rule,
                rule_revision: value.rule_revision,
                rule_digest: value.rule_digest,
                change_set: value.change_set,
                change_set_digest: value.change_set_digest,
            },
            value
                .correspondences
                .into_iter()
                .map(ProjectionItem::Correspondence)
                .collect(),
        ),
        Projection::Graph(value) => {
            let mut items = Vec::new();
            let vocabulary = value
                .authorities
                .into_iter()
                .map(ProjectionItem::GraphAuthority)
                .chain(
                    value
                        .relations
                        .into_iter()
                        .map(ProjectionItem::GraphRelation),
                )
                .chain(value.scopes.into_iter().map(ProjectionItem::GraphScope))
                .chain(
                    value
                        .provenances
                        .into_iter()
                        .map(ProjectionItem::GraphProvenance),
                );
            let data = value
                .nodes
                .into_iter()
                .map(ProjectionItem::GraphNode)
                .chain(value.edges.into_iter().map(ProjectionItem::GraphEdge))
                .chain(value.unresolved.into_iter().map(ProjectionItem::Unresolved));
            if graph_vocabulary_first {
                items.extend(vocabulary);
                items.extend(data);
            } else {
                items.extend(data);
                items.extend(vocabulary);
            }
            (ProjectionHeader::Graph, items)
        }
        Projection::Corpus(value) => (
            ProjectionHeader::Corpus,
            value
                .vectors
                .into_iter()
                .map(ProjectionItem::VectorDefinition)
                .collect(),
        ),
        Projection::Evidence(value) => (
            ProjectionHeader::Evidence,
            value
                .records
                .into_iter()
                .map(ProjectionItem::EvidenceRecord)
                .collect(),
        ),
        Projection::Policy(value) => {
            let mut items = Vec::new();
            items.extend(value.seed_rules.into_iter().map(ProjectionItem::SeedRule));
            items.extend(
                value
                    .propagation_rules
                    .into_iter()
                    .map(ProjectionItem::PropagationRule),
            );
            items.extend(
                value
                    .vector_rules
                    .into_iter()
                    .map(ProjectionItem::VectorRule),
            );
            (
                ProjectionHeader::Policy {
                    identity: value.identity,
                    revision: value.revision,
                    digest: value.digest,
                },
                items,
            )
        }
        Projection::Manifest(value) => {
            let mut items = Vec::new();
            items.extend(
                value
                    .providers
                    .into_iter()
                    .map(ProjectionItem::ManifestProvider),
            );
            items.extend(value.entries.into_iter().map(ProjectionItem::ManifestEntry));
            (ProjectionHeader::Manifest, items)
        }
    };
    sink.begin(header)?;
    for item in items {
        sink.push(item)?;
    }
    Ok(())
}

#[derive(Clone)]
struct Fixture {
    baseline: quire_integration::ecosystem_lock::ValidatedLock,
    candidate: quire_integration::ecosystem_lock::ValidatedLock,
    inputs: Vec<ReaderInput>,
    readers: Vec<FixtureReader>,
}

fn fixture() -> Fixture {
    let baseline = validated_lock("baseline", '1');
    let candidate = validated_lock("candidate", '2');
    let baseline_root = subject(SubjectKind::Requirement, "root", "baseline-r1");
    let root = subject(SubjectKind::Requirement, "root", "candidate-r2");
    let left = subject(SubjectKind::Model, "left", "candidate-r2");
    let right = subject(SubjectKind::Model, "right", "candidate-r2");
    let shared = subject(SubjectKind::Evidence, "shared", "candidate-r2");
    let disconnected = subject(SubjectKind::Evidence, "disconnected", "candidate-r2");
    let assessment = AssessmentTuple {
        structural: StructuralAssessment::PayloadBreaking,
        behavioral: BehavioralAssessment::Regressed,
        presence: SubjectPresence::BothPresent,
    };
    let comparison = ComparisonProjection {
        baseline_lock: baseline.identity(),
        candidate_lock: candidate.identity(),
        comparison_rule: RuleId::new("comparison-rule", ID_LIMIT).expect("rule"),
        rule_revision: RevisionId::new("comparison-r1", ID_LIMIT).expect("revision"),
        rule_digest: Sha256::digest(b"comparison-rule").into(),
        change_set: ChangeSetId::new("change-set", ID_LIMIT).expect("change set"),
        change_set_digest: Sha256::digest(b"change-set").into(),
        correspondences: vec![Correspondence {
            id: CorrespondenceId::new("root-direction", ID_LIMIT).expect("correspondence"),
            baseline: Some(baseline_root),
            candidate: Some(root.clone()),
            structural: assessment.structural,
            behavioral: assessment.behavioral,
            comparable_results: ComparableResults::Both,
            comparable_key: Some(ComparableResultKey {
                subject: root.clone(),
                vector: VectorId::new("vector:compatibility", ID_LIMIT).expect("vector"),
                method: MethodId::new("property", ID_LIMIT).expect("method"),
                provider: id!(ProviderId, "verification-provider"),
                environment: EnvironmentId::new("local-rust", ID_LIMIT).expect("environment"),
                assumptions: id!(ArtifactId, "assumptions:v1"),
                domain: scope("scope:qualification"),
                baseline_lock: baseline.identity(),
                candidate_lock: candidate.identity(),
            }),
            presence: assessment.presence,
        }],
    };
    let edge = |source: &QualifiedSubjectId,
                target: &QualifiedSubjectId,
                edge_scope: &str,
                edge_provenance: &str| GraphEdge {
        source: source.clone(),
        relation: relation(),
        target: target.clone(),
        scope: scope(edge_scope),
        provenance: provenance(edge_provenance),
    };
    let graph = GraphProjection {
        authorities: vec![AuthorityId::new("agent-ix/impact-owner", ID_LIMIT).expect("authority")],
        relations: vec![relation()],
        scopes: vec![scope("scope:affected"), scope("scope:stale")],
        provenances: vec![
            provenance("p:root-left"),
            provenance("p:left-shared"),
            provenance("p:root-right"),
            provenance("p:right-shared"),
            provenance("p:shared-right"),
        ],
        nodes: vec![
            GraphNode {
                id: root.clone(),
                evidence: None,
            },
            GraphNode {
                id: left.clone(),
                evidence: None,
            },
            GraphNode {
                id: right.clone(),
                evidence: None,
            },
            GraphNode {
                id: shared.clone(),
                evidence: Some(EvidenceId::new("evidence:shared", ID_LIMIT).expect("evidence")),
            },
            GraphNode {
                id: disconnected.clone(),
                evidence: Some(
                    EvidenceId::new("evidence:disconnected", ID_LIMIT).expect("evidence"),
                ),
            },
        ],
        edges: vec![
            edge(&root, &left, "scope:affected", "p:root-left"),
            edge(&left, &shared, "scope:affected", "p:left-shared"),
            edge(&root, &right, "scope:stale", "p:root-right"),
            edge(&right, &shared, "scope:stale", "p:right-shared"),
            edge(&shared, &right, "scope:stale", "p:shared-right"),
        ],
        unresolved: vec![],
    };
    let vector_bytes = b"fr336:vector:compatibility".to_vec();
    let vector = VectorDefinition {
        id: VectorId::new("vector:compatibility", ID_LIMIT).expect("vector"),
        digest: Sha256::digest(&vector_bytes).into(),
        owner_bytes: vector_bytes,
    };
    let corpus = CorpusProjection {
        vectors: vec![vector.clone()],
    };
    let evidence = EvidenceProjection {
        records: vec![
            EvidenceRecord {
                id: EvidenceId::new("evidence:shared", ID_LIMIT).expect("evidence"),
                subject: shared.clone(),
                relation: EvidenceRelation::Supports,
                owner_bytes: b"fr337:evidence:shared".to_vec(),
                scope: scope("scope:stale"),
                provenance: provenance("evidence:shared"),
                rights: vec!["private-test".to_owned()],
                limitations: vec!["fixture-only".to_owned()],
                freshness: Freshness::Current,
            },
            EvidenceRecord {
                id: EvidenceId::new("evidence:disconnected", ID_LIMIT).expect("evidence"),
                subject: disconnected,
                relation: EvidenceRelation::Contradicts,
                owner_bytes: b"fr337:evidence:disconnected".to_vec(),
                scope: scope("scope:stale"),
                provenance: provenance("evidence:disconnected"),
                rights: vec!["private-test".to_owned()],
                limitations: vec!["fixture-only".to_owned()],
                freshness: Freshness::Stale,
            },
        ],
    };
    let effects = |members: &[Effect]| members.iter().copied().collect::<BTreeSet<_>>();
    let propagation = |name: &str,
                       source_kind: SubjectKind,
                       target_kind: SubjectKind,
                       exact_scope: &str,
                       effect: Effect| PropagationRule {
        id: RuleId::new(name, ID_LIMIT).expect("rule"),
        relation: relation(),
        source_kind,
        target_kind,
        direction: Direction::Forward,
        scope: scope(exact_scope),
        effects: effects(&[effect]),
    };
    let vector_key = VectorKey {
        vector: vector.id.clone(),
        subject: root,
        scope: scope("scope:qualification"),
        method: MethodId::new("property", ID_LIMIT).expect("method"),
        environment: EnvironmentId::new("local-rust", ID_LIMIT).expect("environment"),
        provider_role: ProviderRoleId::new("verification", ID_LIMIT).expect("role"),
        profile: PolicyProfileId::new("impact-policy", ID_LIMIT).expect("profile"),
    };
    let mut policy = ImpactPolicy {
        identity: PolicyProfileId::new("impact-policy", ID_LIMIT).expect("profile"),
        revision: RevisionId::new("policy-r1", ID_LIMIT).expect("revision"),
        digest: [0; 32],
        seed_rules: all_seed_rules(
            assessment,
            effects(&[Effect::Affectedness, Effect::Staleness]),
        ),
        propagation_rules: vec![
            propagation(
                "affected:root-left",
                SubjectKind::Requirement,
                SubjectKind::Model,
                "scope:affected",
                Effect::Affectedness,
            ),
            propagation(
                "affected:left-shared",
                SubjectKind::Model,
                SubjectKind::Evidence,
                "scope:affected",
                Effect::Affectedness,
            ),
            propagation(
                "stale:root-right",
                SubjectKind::Requirement,
                SubjectKind::Model,
                "scope:stale",
                Effect::Staleness,
            ),
            propagation(
                "stale:right-shared",
                SubjectKind::Model,
                SubjectKind::Evidence,
                "scope:stale",
                Effect::Staleness,
            ),
            propagation(
                "stale:shared-right",
                SubjectKind::Evidence,
                SubjectKind::Model,
                "scope:stale",
                Effect::Staleness,
            ),
        ],
        vector_rules: vec![VectorRule {
            id: RuleId::new("vector:required", ID_LIMIT).expect("rule"),
            key: vector_key,
            requirement: VectorRequirement::Required,
        }],
    };
    policy.digest = policy.recompute_digest().expect("policy digest");
    let manifest = ManifestProjection {
        providers: vec![vector_provider_selection()],
        entries: vec![ManifestEntry {
            provider: vector_provider_selection(),
            vector: vector.id,
            status: ManifestStatus::Supported,
        }],
    };
    let projections = vec![
        Projection::Comparison(comparison),
        Projection::Graph(graph),
        Projection::Corpus(corpus),
        Projection::Evidence(evidence),
        Projection::Policy(policy),
        Projection::Manifest(manifest),
    ];
    let inputs = contracts()
        .into_iter()
        .map(|contract| ReaderInput {
            contract,
            selection: selection(contract),
            lock: if contract == InputContract::Manifest {
                LockSide::Candidate
            } else {
                LockSide::Baseline
            },
            document: OwnerDocument::new(
                contract,
                format!("{}:fixture", contract.label()).into_bytes(),
            ),
        })
        .collect::<Vec<_>>();
    let readers = inputs
        .iter()
        .zip(projections)
        .map(|(input, projection)| FixtureReader {
            selection: input.selection.clone(),
            projection,
            failure: None,
            calls: 0,
            poison_sink: false,
            graph_vocabulary_first: false,
        })
        .collect();
    Fixture {
        baseline,
        candidate,
        inputs,
        readers,
    }
}

fn two_root_fixture() -> Fixture {
    let mut fixture = fixture();
    let baseline = subject(SubjectKind::Requirement, "root:second", "baseline-r1");
    let candidate = subject(SubjectKind::Requirement, "root:second", "candidate-r2");
    let mut correspondence = comparison_mut(&mut fixture).correspondences[0].clone();
    correspondence.id =
        CorrespondenceId::new("root-direction:second", ID_LIMIT).expect("correspondence");
    correspondence.baseline = Some(baseline);
    correspondence.candidate = Some(candidate.clone());
    correspondence
        .comparable_key
        .as_mut()
        .expect("comparable key")
        .subject = candidate.clone();
    comparison_mut(&mut fixture)
        .correspondences
        .push(correspondence);
    graph_mut(&mut fixture).nodes.push(GraphNode {
        id: candidate,
        evidence: None,
    });
    fixture
}

fn two_vector_fixture() -> Fixture {
    let mut fixture = fixture();
    let owner_bytes = b"fr336:vector:second".to_vec();
    let vector = VectorDefinition {
        id: VectorId::new("vector:second", ID_LIMIT).expect("vector"),
        digest: Sha256::digest(&owner_bytes).into(),
        owner_bytes,
    };
    corpus_mut(&mut fixture).vectors.push(vector.clone());
    let mut rule = policy_mut(&mut fixture).vector_rules[0].clone();
    rule.id = RuleId::new("vector:second", ID_LIMIT).expect("rule");
    rule.key.vector = vector.id.clone();
    policy_mut(&mut fixture).vector_rules.push(rule);
    let policy = policy_mut(&mut fixture);
    policy.digest = policy.recompute_digest().expect("policy digest");
    manifest_mut(&mut fixture).entries.push(ManifestEntry {
        provider: vector_provider_selection(),
        vector: vector.id,
        status: ManifestStatus::Supported,
    });
    fixture
}

fn unresolved_boundary_fixture() -> Fixture {
    let mut fixture = fixture();
    let root = comparison_mut(&mut fixture).correspondences[0]
        .candidate
        .clone()
        .expect("candidate root");
    let predecessors: Vec<_> = graph_mut(&mut fixture)
        .nodes
        .iter()
        .filter(|node| matches!(node.id.local.as_str(), "left" | "right"))
        .map(|node| node.id.clone())
        .collect();
    for (index, predecessor) in predecessors.into_iter().enumerate() {
        let (entry_scope, entry_provenance) = if predecessor.local.as_str() == "left" {
            ("scope:affected", "p:unresolved-affected")
        } else {
            ("scope:stale", "p:unresolved-stale")
        };
        let mut entry = UnresolvedDependency {
            root: root.clone(),
            predecessor,
            target: subject(
                SubjectKind::Evidence,
                &format!("missing:{index}"),
                "candidate-r2",
            ),
            relation: relation(),
            scope: scope(entry_scope),
            provenance: provenance(entry_provenance),
            reason: format!("unresolved owner endpoint {index}"),
            digest: [0; 32],
        };
        entry.digest = entry.recompute_digest().expect("unresolved digest");
        let graph = graph_mut(&mut fixture);
        graph.provenances.push(provenance(entry_provenance));
        graph.unresolved.push(entry);
    }
    fixture
}

fn two_provider_fixture() -> Fixture {
    let mut fixture = fixture();
    manifest_mut(&mut fixture)
        .providers
        .push(selection(InputContract::Comparison));
    fixture
}

fn finite_diamond_fixture(left_edges: usize, right_edges: usize) -> Fixture {
    assert!((2..=5).contains(&left_edges));
    assert!((2..=5).contains(&right_edges));
    let mut fixture = fixture();
    let root = comparison_mut(&mut fixture).correspondences[0]
        .candidate
        .clone()
        .expect("candidate root");
    let (shared, disconnected) = {
        let graph = graph_mut(&mut fixture);
        (
            graph
                .nodes
                .iter()
                .find(|node| node.id.local.as_str() == "shared")
                .expect("shared")
                .clone(),
            graph
                .nodes
                .iter()
                .find(|node| node.id.local.as_str() == "disconnected")
                .expect("disconnected")
                .clone(),
        )
    };
    let mut nodes = vec![
        GraphNode {
            id: root.clone(),
            evidence: None,
        },
        shared.clone(),
        disconnected,
    ];
    let mut edges = Vec::new();
    let mut provenances = Vec::new();
    for (branch, edge_count) in [('a', left_edges), ('b', right_edges)] {
        let intermediates: Vec<_> = (0..edge_count - 1)
            .map(|index| {
                subject(
                    SubjectKind::Model,
                    &format!("{branch}:{index}"),
                    "candidate-r2",
                )
            })
            .collect();
        nodes.extend(
            intermediates
                .iter()
                .cloned()
                .map(|id| GraphNode { id, evidence: None }),
        );
        let mut path_nodes = Vec::with_capacity(edge_count + 1);
        path_nodes.push(root.clone());
        path_nodes.extend(intermediates);
        path_nodes.push(shared.id.clone());
        for (index, pair) in path_nodes.windows(2).enumerate() {
            let edge_provenance = provenance(&format!("p:{branch}:{index}"));
            provenances.push(edge_provenance.clone());
            edges.push(GraphEdge {
                source: pair[0].clone(),
                relation: relation(),
                target: pair[1].clone(),
                scope: scope("scope:affected"),
                provenance: edge_provenance,
            });
        }
    }
    let graph = graph_mut(&mut fixture);
    graph.scopes = vec![scope("scope:affected")];
    graph.provenances = provenances;
    graph.nodes = nodes;
    graph.edges = edges;
    graph.unresolved.clear();

    let effects: BTreeSet<_> = [Effect::Affectedness].into_iter().collect();
    let policy = policy_mut(&mut fixture);
    policy.propagation_rules = vec![
        PropagationRule {
            id: RuleId::new("diamond:root-model", ID_LIMIT).expect("rule"),
            relation: relation(),
            source_kind: SubjectKind::Requirement,
            target_kind: SubjectKind::Model,
            direction: Direction::Forward,
            scope: scope("scope:affected"),
            effects: effects.clone(),
        },
        PropagationRule {
            id: RuleId::new("diamond:model-model", ID_LIMIT).expect("rule"),
            relation: relation(),
            source_kind: SubjectKind::Model,
            target_kind: SubjectKind::Model,
            direction: Direction::Forward,
            scope: scope("scope:affected"),
            effects: effects.clone(),
        },
        PropagationRule {
            id: RuleId::new("diamond:model-evidence", ID_LIMIT).expect("rule"),
            relation: relation(),
            source_kind: SubjectKind::Model,
            target_kind: SubjectKind::Evidence,
            direction: Direction::Forward,
            scope: scope("scope:affected"),
            effects,
        },
    ];
    policy.digest = policy.recompute_digest().expect("policy digest");
    fixture
}

fn run(fixture: &mut Fixture, run_limits: Limits) -> Result<ImpactEnvelope> {
    let mut readers: Vec<&mut dyn StrictImpactReader> = fixture
        .readers
        .iter_mut()
        .map(|reader| reader as &mut dyn StrictImpactReader)
        .collect();
    ImpactCoordinator::new(run_limits).classify(
        &fixture.baseline,
        &fixture.candidate,
        &fixture.inputs,
        &mut readers,
    )
}

#[derive(Clone, Copy)]
enum BoundaryPhase {
    Refused,
    Incomplete {
        usage: Usage,
        frontier_entries: usize,
    },
}

fn usage_for(resource: Resource, usage: Usage) -> Option<u64> {
    match resource {
        Resource::TotalInputBytes => Some(usage.input_bytes),
        Resource::ReaderInvocations => Some(usage.reader_invocations),
        Resource::TopologyItems => Some(usage.topology_items),
        Resource::TraversalSteps => Some(usage.traversal_steps),
        Resource::TraversedEdges => Some(usage.traversed_edges),
        Resource::SubjectRecords => Some(usage.subject_records),
        Resource::PathEdges => Some(usage.path_edges),
        Resource::Witnesses => Some(usage.witnesses),
        Resource::WitnessBytes => Some(usage.witness_bytes),
        Resource::FrontierEntries => Some(usage.frontier_entries),
        _ => None,
    }
}

fn one_over_traversal_usage() -> Usage {
    Usage {
        input_bytes: 212,
        reader_invocations: 6,
        topology_items: 10,
        traversal_steps: 5,
        traversed_edges: 4,
        subject_records: 5,
        path_edges: 6,
        witnesses: 6,
        witness_bytes: 2_250,
        frontier_entries: 1,
    }
}

fn one_over_subject_usage() -> Usage {
    Usage {
        input_bytes: 212,
        reader_invocations: 6,
        topology_items: 10,
        subject_records: 4,
        frontier_entries: 2,
        ..Usage::default()
    }
}

fn assert_golden_boundary(
    make_fixture: fn() -> Fixture,
    resource: Resource,
    demand: u64,
    phase: BoundaryPhase,
    set: impl Fn(&mut Limits, u64),
) {
    assert!(
        demand > 1,
        "one-over fixture must have nonzero retained demand"
    );
    let mut exact_limits = limits();
    set(&mut exact_limits, demand);
    let exact = run(&mut make_fixture(), exact_limits).expect("golden exact boundary");
    assert_eq!(exact.outcome(), &TraversalOutcome::Complete);
    if let Some(actual) = usage_for(resource, exact.usage()) {
        assert_eq!(actual, demand, "exact {resource:?} usage");
    }

    let mut one_over_limits = limits();
    set(&mut one_over_limits, demand - 1);
    let mut one_over_fixture = make_fixture();
    match phase {
        BoundaryPhase::Refused => {
            let error = run(&mut one_over_fixture, one_over_limits)
                .expect_err("golden one-over boundary must refuse");
            assert_eq!(
                (error.code(), error.resource()),
                (ErrorCode::ResourceExhausted, Some(resource)),
                "one-over {resource:?}"
            );
            if resource == Resource::ReaderInvocations {
                assert!(one_over_fixture
                    .readers
                    .iter()
                    .all(|reader| reader.calls == 0));
            }
        }
        BoundaryPhase::Incomplete {
            usage,
            frontier_entries,
        } => {
            let envelope = run(&mut one_over_fixture, one_over_limits)
                .expect("golden one-over traversal boundary must be incomplete");
            assert_eq!(envelope.outcome().cause(), Some(resource));
            assert_eq!(envelope.usage(), usage, "one-over {resource:?} usage");
            assert_eq!(
                envelope.outcome().frontier().len(),
                frontier_entries,
                "one-over {resource:?} frontier"
            );
        }
    }
}

fn projection_mut(fixture: &mut Fixture, contract: InputContract) -> &mut Projection {
    &mut fixture
        .readers
        .iter_mut()
        .find(|reader| reader.projection.contract() == contract)
        .expect("selected reader")
        .projection
}

fn comparison_mut(fixture: &mut Fixture) -> &mut ComparisonProjection {
    let Projection::Comparison(value) = projection_mut(fixture, InputContract::Comparison) else {
        unreachable!("comparison projection")
    };
    value
}

fn policy_mut(fixture: &mut Fixture) -> &mut ImpactPolicy {
    let Projection::Policy(value) = projection_mut(fixture, InputContract::Policy) else {
        unreachable!("policy projection")
    };
    value
}

fn graph_mut(fixture: &mut Fixture) -> &mut GraphProjection {
    let Projection::Graph(value) = projection_mut(fixture, InputContract::Graph) else {
        unreachable!("graph projection")
    };
    value
}

fn evidence_mut(fixture: &mut Fixture) -> &mut EvidenceProjection {
    let Projection::Evidence(value) = projection_mut(fixture, InputContract::Evidence) else {
        unreachable!("evidence projection")
    };
    value
}

fn corpus_mut(fixture: &mut Fixture) -> &mut CorpusProjection {
    let Projection::Corpus(value) = projection_mut(fixture, InputContract::Corpus) else {
        unreachable!("corpus projection")
    };
    value
}

fn manifest_mut(fixture: &mut Fixture) -> &mut ManifestProjection {
    let Projection::Manifest(value) = projection_mut(fixture, InputContract::Manifest) else {
        unreachable!("manifest projection")
    };
    value
}

#[derive(Default)]
struct StepClock(AtomicU64);

impl MonotonicClock for StepClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

#[test]
#[trace("TC-005", "FR-005-AC-1")]
fn tc005_preserves_admitted_axis_tuples_and_refuses_outsiders() {
    let structural_values = [
        StructuralAssessment::Unchanged,
        StructuralAssessment::AdditiveCompatible,
        StructuralAssessment::PayloadBreaking,
        StructuralAssessment::Removed,
        StructuralAssessment::Unknown,
    ];
    let behavioral_values = [
        BehavioralAssessment::NotAssessed,
        BehavioralAssessment::Conforming,
        BehavioralAssessment::Regressed,
        BehavioralAssessment::Absent,
        BehavioralAssessment::Unsupported,
        BehavioralAssessment::Unavailable,
        BehavioralAssessment::Unknown,
    ];
    let presence_values = [
        SubjectPresence::BothPresent,
        SubjectPresence::BaselineOnly,
        SubjectPresence::CandidateOnly,
        SubjectPresence::Unknown,
    ];
    for structural in structural_values {
        for behavioral in behavioral_values {
            for presence in presence_values {
                let structurally_admitted = match structural {
                    StructuralAssessment::Unchanged | StructuralAssessment::PayloadBreaking => {
                        presence == SubjectPresence::BothPresent
                    }
                    StructuralAssessment::Removed => presence == SubjectPresence::BaselineOnly,
                    StructuralAssessment::AdditiveCompatible => matches!(
                        presence,
                        SubjectPresence::BothPresent | SubjectPresence::CandidateOnly
                    ),
                    StructuralAssessment::Unknown => true,
                };
                let behaviorally_admitted = !matches!(
                    behavioral,
                    BehavioralAssessment::Conforming
                        | BehavioralAssessment::Regressed
                        | BehavioralAssessment::Absent
                ) || presence == SubjectPresence::BothPresent;
                let admitted = structurally_admitted && behaviorally_admitted;
                let comparable_results = match behavioral {
                    BehavioralAssessment::Conforming | BehavioralAssessment::Regressed => {
                        ComparableResults::Both
                    }
                    BehavioralAssessment::Absent => ComparableResults::CandidateMissing,
                    _ => ComparableResults::NotApplicable,
                };
                let mut fixture = fixture();
                let (baseline_subject, candidate_subject, comparable_key) = {
                    let correspondence = &comparison_mut(&mut fixture).correspondences[0];
                    (
                        correspondence.baseline.clone().expect("baseline subject"),
                        correspondence.candidate.clone().expect("candidate subject"),
                        correspondence.comparable_key.clone(),
                    )
                };
                if presence == SubjectPresence::BaselineOnly {
                    let graph = graph_mut(&mut fixture);
                    for node in &mut graph.nodes {
                        if node.id == candidate_subject {
                            node.id = baseline_subject.clone();
                        }
                    }
                    for edge in &mut graph.edges {
                        if edge.source == candidate_subject {
                            edge.source = baseline_subject.clone();
                        }
                        if edge.target == candidate_subject {
                            edge.target = baseline_subject.clone();
                        }
                    }
                    policy_mut(&mut fixture).vector_rules[0].key.subject = baseline_subject.clone();
                }
                let correspondence = &mut comparison_mut(&mut fixture).correspondences[0];
                correspondence.structural = structural;
                correspondence.behavioral = behavioral;
                correspondence.comparable_results = comparable_results;
                correspondence.comparable_key = (comparable_results
                    != ComparableResults::NotApplicable)
                    .then_some(comparable_key)
                    .flatten();
                correspondence.presence = presence;
                match presence {
                    SubjectPresence::BothPresent | SubjectPresence::Unknown => {}
                    SubjectPresence::BaselineOnly => correspondence.candidate = None,
                    SubjectPresence::CandidateOnly => correspondence.baseline = None,
                }
                let assessment = AssessmentTuple {
                    structural,
                    behavioral,
                    presence,
                };
                let policy = policy_mut(&mut fixture);
                for rule in &mut policy.seed_rules {
                    rule.effects = if rule.assessment == assessment {
                        [Effect::Affectedness, Effect::Staleness]
                            .into_iter()
                            .collect()
                    } else {
                        BTreeSet::new()
                    };
                }
                policy.digest = policy.recompute_digest().expect("policy digest");
                if admitted {
                    let result = run(&mut fixture, limits()).expect("admitted tuple");
                    assert_eq!(
                        (
                            result.assessments()[0].structural,
                            result.assessments()[0].behavioral,
                            result.assessments()[0].presence
                        ),
                        (structural, behavioral, presence)
                    );
                } else {
                    let error = run(&mut fixture, limits()).expect_err("table outsider");
                    assert_eq!(error.code(), ErrorCode::OwnerContradiction);
                }
            }
        }
    }
}

#[test]
#[trace("TC-005", "FR-005-AC-2")]
fn tc005_propagates_disjoint_effects_with_canonical_witnesses() {
    let mut base_fixture = fixture();
    let result = run(&mut base_fixture, limits()).expect("impact result");
    let shared = result
        .subjects()
        .iter()
        .find(|record| record.subject.local.as_str() == "shared")
        .expect("shared record");
    assert_eq!(shared.reachability, Reachability::Affected);
    let affected = shared
        .affectedness_witness
        .as_ref()
        .expect("affected witness");
    let stale = shared.staleness_witness.as_ref().expect("stale witness");
    assert_eq!(
        (affected.path.len(), affected.path[0].target.local.as_str()),
        (2, "left")
    );
    assert_eq!(
        (stale.path.len(), stale.path[0].target.local.as_str()),
        (2, "right")
    );
    assert_eq!(result.cycles().len(), 1);
    assert_eq!(
        (result.cycles()[0].effect, result.cycles()[0].cycle.len()),
        (Effect::Staleness, 2)
    );
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_maps_each_bound_to_its_exact_phase_outcome() {
    let mut topology_fixture = fixture();
    let mut topology_limits = limits();
    topology_limits.max_topology_items = 9;
    let topology = run(&mut topology_fixture, topology_limits).expect_err("topology refusal");
    assert_eq!(
        (topology.code(), topology.resource()),
        (ErrorCode::ResourceExhausted, Some(Resource::TopologyItems))
    );

    let mut traversal_fixture = fixture();
    let mut traversal_limits = limits();
    traversal_limits.max_traversal_steps = 2;
    let traversal = run(&mut traversal_fixture, traversal_limits).expect("incomplete envelope");
    assert_eq!(traversal.outcome().cause(), Some(Resource::TraversalSteps));
    assert!(!traversal.outcome().frontier().is_empty());
    assert!(traversal.usage().traversal_steps <= 2);

    let mut reader_fixture = fixture();
    reader_fixture.readers[0].failure = Some(ReaderError::Failed);
    let reader = run(&mut reader_fixture, limits()).expect_err("reader failure");
    assert_eq!(reader.code(), ErrorCode::ReaderFailure);

    let mut duration_fixture = fixture();
    duration_fixture.readers[0].failure = Some(ReaderError::Exhausted(Resource::Duration));
    let duration = run(&mut duration_fixture, limits()).expect_err("reader duration");
    assert_eq!(duration.code(), ErrorCode::DurationExpired);

    let mut exhausted_fixture = fixture();
    exhausted_fixture.readers[0].failure = Some(ReaderError::Exhausted(Resource::Correspondences));
    let exhausted = run(&mut exhausted_fixture, limits()).expect_err("reader resource");
    assert_eq!(
        (exhausted.code(), exhausted.resource()),
        (
            ErrorCode::ResourceExhausted,
            Some(Resource::Correspondences)
        )
    );
}

#[test]
#[trace("TC-005", "FR-005-AC-4")]
fn tc005_applies_the_complete_freshness_table_without_invented_witnesses() {
    for (prior, propagation, expected) in [
        (
            Freshness::Current,
            FreshnessPropagation::Reached,
            Freshness::Stale,
        ),
        (
            Freshness::Current,
            FreshnessPropagation::NotReached,
            Freshness::Current,
        ),
        (
            Freshness::Current,
            FreshnessPropagation::Unknown,
            Freshness::Unknown,
        ),
        (
            Freshness::Current,
            FreshnessPropagation::Incomplete,
            Freshness::Unknown,
        ),
        (
            Freshness::Stale,
            FreshnessPropagation::Reached,
            Freshness::Stale,
        ),
        (
            Freshness::Stale,
            FreshnessPropagation::NotReached,
            Freshness::Stale,
        ),
        (
            Freshness::Stale,
            FreshnessPropagation::Unknown,
            Freshness::Stale,
        ),
        (
            Freshness::Stale,
            FreshnessPropagation::Incomplete,
            Freshness::Stale,
        ),
        (
            Freshness::Unknown,
            FreshnessPropagation::Reached,
            Freshness::Unknown,
        ),
        (
            Freshness::Unknown,
            FreshnessPropagation::NotReached,
            Freshness::Unknown,
        ),
        (
            Freshness::Unknown,
            FreshnessPropagation::Unknown,
            Freshness::Unknown,
        ),
        (
            Freshness::Unknown,
            FreshnessPropagation::Incomplete,
            Freshness::Unknown,
        ),
    ] {
        assert_eq!(transition_freshness(prior, propagation), expected);
    }

    let mut freshness_fixture = fixture();
    let result = run(&mut freshness_fixture, limits()).expect("impact result");
    let changed = result
        .freshness()
        .iter()
        .find(|record| record.evidence.as_str() == "evidence:shared")
        .expect("changed freshness");
    assert_eq!(
        (changed.relation, changed.freshness),
        (EvidenceRelation::Supports, Freshness::Stale)
    );
    assert!(changed.witness.is_some());
    let prior_stale = result
        .freshness()
        .iter()
        .find(|record| record.evidence.as_str() == "evidence:disconnected")
        .expect("prior stale");
    assert_eq!(
        (prior_stale.relation, prior_stale.freshness),
        (EvidenceRelation::Contradicts, Freshness::Stale)
    );
    assert!(prior_stale.witness.is_none());

    for relation in [EvidenceRelation::Supports, EvidenceRelation::Contradicts] {
        for prior in [Freshness::Current, Freshness::Stale, Freshness::Unknown] {
            let mut owner_fixture = fixture();
            for record in &mut evidence_mut(&mut owner_fixture).records {
                record.relation = relation;
                record.freshness = prior;
            }
            let expected_owner: Vec<_> = evidence_mut(&mut owner_fixture)
                .records
                .iter()
                .map(|record| {
                    (
                        record.id.clone(),
                        record.owner_bytes.clone(),
                        record.scope.clone(),
                        record.provenance.clone(),
                    )
                })
                .collect();
            let result = run(&mut owner_fixture, limits()).expect("owner freshness cross-product");
            for (id, owner_bytes, owner_scope, owner_provenance) in expected_owner {
                let record = result
                    .freshness()
                    .iter()
                    .find(|record| record.evidence == id)
                    .expect("freshness record");
                assert_eq!(
                    (
                        record.relation,
                        &record.owner_bytes,
                        &record.scope,
                        &record.provenance,
                        record.prior,
                    ),
                    (
                        relation,
                        &owner_bytes,
                        &owner_scope,
                        &owner_provenance,
                        prior,
                    )
                );
                assert_eq!(
                    record.freshness,
                    transition_freshness(prior, record.propagation)
                );
            }
        }
    }
}

#[test]
#[trace("TC-005", "FR-005-AC-5")]
fn tc005_accounts_for_each_catalog_vector_once_with_independent_axes() {
    let mut available_fixture = fixture();
    let available = run(&mut available_fixture, limits()).expect("available vector");
    assert_eq!(available.vectors().len(), 1);
    assert_eq!(
        available.vectors()[0].requirement,
        VectorRequirement::Required
    );
    assert!(matches!(
        available.vectors()[0].availability,
        VectorAvailability::Available(_)
    ));

    let mut unavailable_fixture = fixture();
    manifest_mut(&mut unavailable_fixture).entries.clear();
    let unavailable = run(&mut unavailable_fixture, limits()).expect("unavailable vector");
    assert_eq!(
        unavailable.vectors()[0].availability,
        VectorAvailability::Unavailable
    );

    let mut unsupported_fixture = fixture();
    manifest_mut(&mut unsupported_fixture).entries[0].status = ManifestStatus::Excluded;
    let unsupported = run(&mut unsupported_fixture, limits()).expect("unsupported vector");
    assert!(matches!(
        unsupported.vectors()[0].availability,
        VectorAvailability::Unsupported(_)
    ));

    let mut unknown_fixture = fixture();
    manifest_mut(&mut unknown_fixture).entries[0].status =
        ManifestStatus::Unknown("provider inventory pending".to_owned());
    let unknown = run(&mut unknown_fixture, limits()).expect("unknown vector");
    assert!(matches!(
        &unknown.vectors()[0].availability,
        VectorAvailability::Unknown { reason, .. } if reason == "provider inventory pending"
    ));

    let mut not_applicable_fixture = fixture();
    let policy = policy_mut(&mut not_applicable_fixture);
    policy.vector_rules[0].requirement = VectorRequirement::NotApplicable;
    policy.digest = policy.recompute_digest().expect("policy digest");
    let not_applicable = run(&mut not_applicable_fixture, limits()).expect("not-applicable vector");
    assert_eq!(
        not_applicable.vectors()[0].requirement,
        VectorRequirement::NotApplicable
    );
    assert!(matches!(
        not_applicable.vectors()[0].availability,
        VectorAvailability::Available(_)
    ));
}

#[test]
#[trace("TC-005", "FR-005-AC-6")]
fn tc005_canonicalizes_permutations_and_binds_mutations() {
    let mut canonical_fixture = fixture();
    let canonical = run(&mut canonical_fixture, limits()).expect("canonical result");
    let mut permuted_fixture = fixture();
    permuted_fixture.inputs.reverse();
    permuted_fixture.readers.reverse();
    for reader in &mut permuted_fixture.readers {
        match &mut reader.projection {
            Projection::Comparison(value) => value.correspondences.reverse(),
            Projection::Graph(value) => {
                value.authorities.reverse();
                value.relations.reverse();
                value.scopes.reverse();
                value.provenances.reverse();
                value.nodes.reverse();
                value.edges.reverse();
                value.unresolved.reverse();
            }
            Projection::Corpus(value) => value.vectors.reverse(),
            Projection::Evidence(value) => value.records.reverse(),
            Projection::Policy(value) => {
                value.seed_rules.reverse();
                value.propagation_rules.reverse();
                value.vector_rules.reverse();
            }
            Projection::Manifest(value) => {
                value.providers.reverse();
                value.entries.reverse();
            }
        }
    }
    let permuted = run(&mut permuted_fixture, limits()).expect("permuted result");
    assert_eq!(canonical.body_bytes(), permuted.body_bytes());
    assert_eq!(canonical.identity(), permuted.identity());

    macro_rules! assert_limit_binds_identity {
        ($($field:ident),+ $(,)?) => {
            $(
                let mut changed_fixture = fixture();
                let mut changed_limits = limits();
                changed_limits.$field += 1;
                let changed = run(&mut changed_fixture, changed_limits)
                    .expect(concat!("changed ", stringify!($field)));
                assert_ne!(canonical.identity(), changed.identity(), "{}", stringify!($field));
            )+
        };
    }
    assert_limit_binds_identity!(
        max_document_bytes,
        max_total_input_bytes,
        max_identity_bytes,
        max_reason_bytes,
        max_correspondences,
        max_assessments,
        max_changed_roots,
        max_graph_nodes,
        max_graph_edges,
        max_evidence_records,
        max_evidence_rights,
        max_evidence_limitations,
        max_reader_invocations,
        max_policy_seed_rules,
        max_policy_propagation_rules,
        max_policy_vector_rules,
        max_vector_definitions,
        max_manifest_entries,
        max_manifest_providers,
        max_unresolved_entries,
        max_total_reason_bytes,
        max_topology_items,
        max_traversal_steps,
        max_traversed_edges,
        max_subject_records,
        max_path_edges,
        max_witnesses,
        max_witness_bytes,
        max_frontier_entries,
        max_output_bytes,
        max_accounting_bytes,
    );

    let mut changed_policy_fixture = fixture();
    let policy = policy_mut(&mut changed_policy_fixture);
    policy.propagation_rules[0].id = RuleId::new("affected:mutated", ID_LIMIT).expect("rule");
    policy.digest = policy.recompute_digest().expect("policy digest");
    let changed = run(&mut changed_policy_fixture, limits()).expect("changed policy");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_direction_fixture = fixture();
    let policy = policy_mut(&mut changed_direction_fixture);
    policy.propagation_rules[0].direction = Direction::Reverse;
    policy.propagation_rules[0].source_kind = SubjectKind::Model;
    policy.propagation_rules[0].target_kind = SubjectKind::Requirement;
    policy.digest = policy.recompute_digest().expect("policy digest");
    let changed = run(&mut changed_direction_fixture, limits()).expect("changed direction");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_identity_fixture = fixture();
    comparison_mut(&mut changed_identity_fixture).change_set =
        ChangeSetId::new("change-set:mutated", ID_LIMIT).expect("change set");
    let changed = run(&mut changed_identity_fixture, limits()).expect("changed identity");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_owner_fixture = fixture();
    evidence_mut(&mut changed_owner_fixture).records[0]
        .owner_bytes
        .push(b'!');
    let changed = run(&mut changed_owner_fixture, limits()).expect("changed owner value");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_profile_fixture = fixture();
    let policy = policy_mut(&mut changed_profile_fixture);
    policy.identity = PolicyProfileId::new("impact-policy:mutated", ID_LIMIT).expect("profile");
    policy.vector_rules[0].key.profile = policy.identity.clone();
    policy.digest = policy.recompute_digest().expect("policy digest");
    let changed = run(&mut changed_profile_fixture, limits()).expect("changed profile");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_vector_fixture = fixture();
    let vector = &mut corpus_mut(&mut changed_vector_fixture).vectors[0];
    vector.owner_bytes.push(b'!');
    vector.digest = Sha256::digest(&vector.owner_bytes).into();
    let changed = run(&mut changed_vector_fixture, limits()).expect("changed vector");
    assert_ne!(canonical.identity(), changed.identity());

    let mut changed_state_fixture = fixture();
    manifest_mut(&mut changed_state_fixture).entries[0].status = ManifestStatus::Excluded;
    let changed = run(&mut changed_state_fixture, limits()).expect("changed state");
    assert_ne!(canonical.identity(), changed.identity());
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_preflights_every_reader_and_latches_sink_refusal() {
    let mut preflight_fixture = fixture();
    let invalid = CapabilitySelection::new(
        id!(RoleId, "impact-reader:manifest"),
        id!(ProviderId, "foreign-provider"),
        id!(BuildId, "impact-build:manifest"),
        id!(ContractId, MANIFEST_CONTRACT),
        id!(ArtifactId, "impact-options:manifest"),
    );
    preflight_fixture.inputs[5].selection = invalid.clone();
    preflight_fixture.readers[5].selection = invalid;
    let error = run(&mut preflight_fixture, limits()).expect_err("foreign selection");
    assert_eq!(error.code(), ErrorCode::CapabilityMismatch);
    assert!(preflight_fixture
        .readers
        .iter()
        .all(|reader| reader.calls == 0));

    let mut poisoned_fixture = fixture();
    poisoned_fixture.readers[0].poison_sink = true;
    let error = run(&mut poisoned_fixture, limits()).expect_err("latched sink refusal");
    assert_eq!(error.code(), ErrorCode::ReaderFailure);
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_marks_cycle_only_and_unresolved_premises_locally_unknown() {
    let mut cycle_fixture = fixture();
    let left = graph_mut(&mut cycle_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "left")
        .expect("left")
        .id
        .clone();
    let graph = graph_mut(&mut cycle_fixture);
    graph.provenances.push(provenance("p:left-loop"));
    graph.edges.push(GraphEdge {
        source: left.clone(),
        relation: relation(),
        target: left,
        scope: scope("scope:affected"),
        provenance: provenance("p:left-loop"),
    });
    let policy = policy_mut(&mut cycle_fixture);
    policy.propagation_rules.push(PropagationRule {
        id: RuleId::new("affected:left-loop", ID_LIMIT).expect("rule"),
        relation: relation(),
        source_kind: SubjectKind::Model,
        target_kind: SubjectKind::Model,
        direction: Direction::Forward,
        scope: scope("scope:affected"),
        effects: [Effect::Affectedness].into_iter().collect(),
    });
    policy.digest = policy.recompute_digest().expect("policy digest");
    let result = run(&mut cycle_fixture, limits()).expect("cycle-only result");
    let left = result
        .subjects()
        .iter()
        .find(|record| record.subject.local.as_str() == "left")
        .expect("left subject");
    assert_eq!(left.reachability, Reachability::Unknown);
    assert!(result
        .cycles()
        .iter()
        .any(|record| record.effect == Effect::Affectedness));

    let mut unresolved_fixture = fixture();
    let root = comparison_mut(&mut unresolved_fixture).correspondences[0]
        .candidate
        .clone()
        .expect("candidate root");
    let predecessor = graph_mut(&mut unresolved_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "right")
        .expect("right")
        .id
        .clone();
    let target = subject(SubjectKind::Evidence, "missing", "candidate-r2");
    let mut unresolved = UnresolvedDependency {
        root,
        predecessor,
        target: target.clone(),
        relation: relation(),
        scope: scope("scope:stale"),
        provenance: provenance("p:unresolved"),
        reason: "owner endpoint unavailable".to_owned(),
        digest: [0; 32],
    };
    unresolved.digest = unresolved.recompute_digest().expect("unresolved digest");
    let graph = graph_mut(&mut unresolved_fixture);
    graph.provenances.push(provenance("p:unresolved"));
    graph.unresolved.push(unresolved);
    evidence_mut(&mut unresolved_fixture)
        .records
        .push(EvidenceRecord {
            id: EvidenceId::new("evidence:missing", ID_LIMIT).expect("evidence"),
            subject: target.clone(),
            relation: EvidenceRelation::Supports,
            owner_bytes: b"fr337:evidence:missing".to_vec(),
            scope: scope("scope:stale"),
            provenance: provenance("evidence:missing"),
            rights: vec!["private-test".to_owned()],
            limitations: vec!["fixture-only".to_owned()],
            freshness: Freshness::Current,
        });
    let result = run(&mut unresolved_fixture, limits()).expect("unresolved result");
    assert_eq!(
        result
            .subjects()
            .iter()
            .find(|record| record.subject == target)
            .expect("unresolved target")
            .reachability,
        Reachability::Unaffected
    );
    let freshness = result
        .freshness()
        .iter()
        .find(|record| record.evidence.as_str() == "evidence:missing")
        .expect("unresolved evidence");
    assert_eq!(
        (freshness.propagation, freshness.freshness),
        (FreshnessPropagation::Unknown, Freshness::Unknown)
    );

    let mut foreign_root_fixture = fixture();
    let foreign_root = graph_mut(&mut foreign_root_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "disconnected")
        .expect("disconnected")
        .id
        .clone();
    let predecessor = foreign_root.clone();
    let mut foreign_root_entry = UnresolvedDependency {
        root: foreign_root,
        predecessor,
        target: subject(SubjectKind::Model, "missing-model", "candidate-r2"),
        relation: relation(),
        scope: scope("scope:stale"),
        provenance: provenance("p:foreign-root"),
        reason: "unrelated root".to_owned(),
        digest: [0; 32],
    };
    foreign_root_entry.digest = foreign_root_entry
        .recompute_digest()
        .expect("unresolved digest");
    let graph = graph_mut(&mut foreign_root_fixture);
    graph.provenances.push(provenance("p:foreign-root"));
    graph.unresolved.push(foreign_root_entry);
    let error = run(&mut foreign_root_fixture, limits()).expect_err("foreign changed root");
    assert_eq!(error.code(), ErrorCode::ForeignInput);

    let mut disconnected_fixture = fixture();
    let root = comparison_mut(&mut disconnected_fixture).correspondences[0]
        .candidate
        .clone()
        .expect("candidate root");
    let predecessor = graph_mut(&mut disconnected_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "disconnected")
        .expect("disconnected")
        .id
        .clone();
    let mut disconnected_entry = UnresolvedDependency {
        root,
        predecessor,
        target: subject(SubjectKind::Model, "missing-model", "candidate-r2"),
        relation: relation(),
        scope: scope("scope:stale"),
        provenance: provenance("p:disconnected"),
        reason: "unreachable predecessor".to_owned(),
        digest: [0; 32],
    };
    disconnected_entry.digest = disconnected_entry
        .recompute_digest()
        .expect("unresolved digest");
    let graph = graph_mut(&mut disconnected_fixture);
    graph.provenances.push(provenance("p:disconnected"));
    graph.unresolved.push(disconnected_entry);
    let error = run(&mut disconnected_fixture, limits()).expect_err("disconnected predecessor");
    assert_eq!(error.code(), ErrorCode::ForeignInput);
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_accounts_cycle_edge_and_subject_retention_bounds() {
    let mut cycle_fixture = fixture();
    let mut cycle_limits = limits();
    cycle_limits.max_path_edges = 1;
    let result = run(&mut cycle_fixture, cycle_limits).expect("cycle retention incomplete");
    assert_eq!(result.outcome().cause(), Some(Resource::PathEdges));
    assert_eq!(result.usage().path_edges, 0);
    assert!(result
        .outcome()
        .frontier()
        .iter()
        .any(|entry| entry.effect == Effect::Staleness));

    let mut edge_fixture = fixture();
    let mut edge_limits = limits();
    edge_limits.max_traversed_edges = 4;
    let result = run(&mut edge_fixture, edge_limits).expect("edge incomplete");
    assert_eq!(result.outcome().cause(), Some(Resource::TraversedEdges));
    assert_eq!(result.usage().traversed_edges, 4);

    let mut subject_fixture = fixture();
    let mut subject_limits = limits();
    subject_limits.max_subject_records = 4;
    let result = run(&mut subject_fixture, subject_limits).expect("subject incomplete");
    assert_eq!(result.outcome().cause(), Some(Resource::SubjectRecords));
    assert_eq!(result.subjects().len(), 4);
    assert_eq!(result.usage().traversal_steps, 0);
    assert_eq!(
        result
            .outcome()
            .frontier()
            .iter()
            .map(|entry| entry.effect)
            .collect::<BTreeSet<_>>(),
        [Effect::Affectedness, Effect::Staleness]
            .into_iter()
            .collect()
    );

    let mut simultaneous_fixture = fixture();
    let mut simultaneous_limits = limits();
    simultaneous_limits.max_traversed_edges = 1;
    simultaneous_limits.max_subject_records = 1;
    let result = run(&mut simultaneous_fixture, simultaneous_limits)
        .expect("simultaneous exhaustion is incomplete");
    assert_eq!(result.outcome().cause(), Some(Resource::SubjectRecords));

    let mut ordered_cycles_fixture = two_root_fixture();
    let roots: Vec<_> = comparison_mut(&mut ordered_cycles_fixture)
        .correspondences
        .iter()
        .map(|correspondence| correspondence.candidate.clone().expect("candidate root"))
        .collect();
    let left = graph_mut(&mut ordered_cycles_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "left")
        .expect("left")
        .id
        .clone();
    let right = graph_mut(&mut ordered_cycles_fixture)
        .nodes
        .iter()
        .find(|node| node.id.local.as_str() == "right")
        .expect("right")
        .id
        .clone();
    let graph = graph_mut(&mut ordered_cycles_fixture);
    graph.provenances.extend([
        provenance("p:second-left"),
        provenance("p:second-right"),
        provenance("p:left-loop"),
    ]);
    graph.edges.extend([
        GraphEdge {
            source: roots[1].clone(),
            relation: relation(),
            target: left.clone(),
            scope: scope("scope:affected"),
            provenance: provenance("p:second-left"),
        },
        GraphEdge {
            source: roots[1].clone(),
            relation: relation(),
            target: right,
            scope: scope("scope:stale"),
            provenance: provenance("p:second-right"),
        },
        GraphEdge {
            source: left.clone(),
            relation: relation(),
            target: left,
            scope: scope("scope:affected"),
            provenance: provenance("p:left-loop"),
        },
    ]);
    let policy = policy_mut(&mut ordered_cycles_fixture);
    policy.propagation_rules.push(PropagationRule {
        id: RuleId::new("affected:left-loop", ID_LIMIT).expect("rule"),
        relation: relation(),
        source_kind: SubjectKind::Model,
        target_kind: SubjectKind::Model,
        direction: Direction::Forward,
        scope: scope("scope:affected"),
        effects: [Effect::Affectedness].into_iter().collect(),
    });
    policy.digest = policy.recompute_digest().expect("policy digest");
    let mut cycle_limits = limits();
    cycle_limits.max_witnesses = 1;
    let result = run(&mut ordered_cycles_fixture, cycle_limits).expect("cycle prefix");
    assert_eq!(result.outcome().cause(), Some(Resource::Witnesses));
    assert_eq!(result.cycles().len(), 1);
    assert_eq!(
        (result.cycles()[0].root.clone(), result.cycles()[0].effect),
        (roots[0].clone(), Effect::Affectedness)
    );
}

#[test]
#[trace("TC-005", "FR-005-AC-5")]
fn tc005_groups_vectors_by_complete_key_and_rejects_foreign_members() {
    let mut multi_key_fixture = fixture();
    let policy = policy_mut(&mut multi_key_fixture);
    let mut second = policy.vector_rules[0].clone();
    second.id = RuleId::new("vector:second-subject", ID_LIMIT).expect("rule");
    second.key.scope = scope("scope:secondary");
    policy.vector_rules.push(second);
    policy.digest = policy.recompute_digest().expect("policy digest");
    let result = run(&mut multi_key_fixture, limits()).expect("multiple complete keys");
    assert_eq!(result.vectors().len(), 2);

    let mut manifest_fixture = fixture();
    manifest_mut(&mut manifest_fixture)
        .entries
        .push(ManifestEntry {
            provider: vector_provider_selection(),
            vector: VectorId::new("vector:foreign", ID_LIMIT).expect("vector"),
            status: ManifestStatus::Supported,
        });
    let error = run(&mut manifest_fixture, limits()).expect_err("foreign manifest member");
    assert_eq!(error.code(), ErrorCode::Malformed);

    let mut profile_fixture = fixture();
    let policy = policy_mut(&mut profile_fixture);
    policy.vector_rules[0].key.profile =
        PolicyProfileId::new("foreign-policy", ID_LIMIT).expect("profile");
    policy.digest = policy.recompute_digest().expect("policy digest");
    let error = run(&mut profile_fixture, limits()).expect_err("foreign policy profile");
    assert_eq!(error.code(), ErrorCode::ForeignInput);

    let mut duplicate_fixture = fixture();
    let duplicate = corpus_mut(&mut duplicate_fixture).vectors[0].clone();
    corpus_mut(&mut duplicate_fixture).vectors.push(duplicate);
    let error = run(&mut duplicate_fixture, limits()).expect_err("duplicate definition");
    assert_eq!(error.code(), ErrorCode::Malformed);

    let mut missing_fixture = fixture();
    corpus_mut(&mut missing_fixture).vectors.clear();
    let error = run(&mut missing_fixture, limits()).expect_err("missing definition");
    assert_eq!(error.code(), ErrorCode::Malformed);

    let mut conflicting_fixture = fixture();
    let policy = policy_mut(&mut conflicting_fixture);
    let mut conflicting = policy.vector_rules[0].clone();
    conflicting.id = RuleId::new("vector:conflicting", ID_LIMIT).expect("rule");
    conflicting.requirement = VectorRequirement::NotApplicable;
    policy.vector_rules.push(conflicting);
    policy.digest = policy.recompute_digest().expect("policy digest");
    let error = run(&mut conflicting_fixture, limits()).expect_err("conflicting requirement");
    assert_eq!(error.code(), ErrorCode::Malformed);

    let mut incomplete_inventory_fixture = fixture();
    manifest_mut(&mut incomplete_inventory_fixture)
        .providers
        .clear();
    let error = run(&mut incomplete_inventory_fixture, limits())
        .expect_err("incomplete provider inventory");
    assert_eq!(error.code(), ErrorCode::ForeignInput);
}

#[test]
#[trace("TC-005", "FR-005-AC-1")]
fn tc005_rejects_unkeyed_results_incomplete_seed_tables_and_foreign_graph_values() {
    let mut key_fixture = fixture();
    comparison_mut(&mut key_fixture).correspondences[0].comparable_key = None;
    let error = run(&mut key_fixture, limits()).expect_err("unkeyed comparison");
    assert_eq!(error.code(), ErrorCode::OwnerContradiction);

    let mut seed_fixture = fixture();
    let policy = policy_mut(&mut seed_fixture);
    policy.seed_rules.pop();
    policy.digest = policy.recompute_digest().expect("policy digest");
    let error = run(&mut seed_fixture, limits()).expect_err("incomplete seed table");
    assert_eq!(error.code(), ErrorCode::Malformed);

    let mut graph_fixture = fixture();
    graph_mut(&mut graph_fixture).edges[0].scope = scope("scope:foreign");
    let error = run(&mut graph_fixture, limits()).expect_err("foreign graph scope");
    assert_eq!(error.code(), ErrorCode::ForeignInput);

    let mut evidence_fixture = fixture();
    evidence_mut(&mut evidence_fixture).records[0].subject =
        subject(SubjectKind::Evidence, "other", "candidate-r2");
    let error = run(&mut evidence_fixture, limits()).expect_err("rebound evidence identity");
    assert_eq!(error.code(), ErrorCode::ForeignInput);
}

#[test]
#[trace("TC-005", "TC-007", "FR-005-AC-3", "FR-005-AC-6", "NFR-001-AC-3")]
fn tc005_deadline_is_failed_and_operational_limit_is_not_identity() {
    let mut deadline_fixture = fixture();
    let mut readers: Vec<&mut dyn StrictImpactReader> = deadline_fixture
        .readers
        .iter_mut()
        .map(|reader| reader as &mut dyn StrictImpactReader)
        .collect();
    let mut deadline_limits = limits();
    deadline_limits.max_duration_ms = 2;
    let error =
        ImpactCoordinator::with_monotonic_clock(deadline_limits, Arc::new(StepClock::default()))
            .classify(
                &deadline_fixture.baseline,
                &deadline_fixture.candidate,
                &deadline_fixture.inputs,
                &mut readers,
            )
            .expect_err("deadline failure");
    assert_eq!(error.code(), ErrorCode::DurationExpired);

    let mut nonmatching_fixture = fixture();
    let policy = policy_mut(&mut nonmatching_fixture);
    for index in 0..100 {
        policy.propagation_rules.push(PropagationRule {
            id: RuleId::new(format!("nonmatching:{index}"), ID_LIMIT).expect("rule"),
            relation: RelationId::new(format!("nonmatching:{index}"), ID_LIMIT).expect("relation"),
            source_kind: SubjectKind::Requirement,
            target_kind: SubjectKind::Model,
            direction: Direction::Forward,
            scope: scope(&format!("nonmatching:{index}")),
            effects: [Effect::Affectedness].into_iter().collect(),
        });
    }
    policy.digest = policy.recompute_digest().expect("policy digest");
    let mut nonmatching_limits = limits();
    nonmatching_limits.max_document_bytes = 128 * 1024;
    nonmatching_limits.max_total_input_bytes = 256 * 1024;
    nonmatching_limits.max_policy_propagation_rules = 128;
    nonmatching_limits.max_duration_ms = 100;
    let mut readers: Vec<&mut dyn StrictImpactReader> = nonmatching_fixture
        .readers
        .iter_mut()
        .map(|reader| reader as &mut dyn StrictImpactReader)
        .collect();
    let error =
        ImpactCoordinator::with_monotonic_clock(nonmatching_limits, Arc::new(StepClock::default()))
            .classify(
                &nonmatching_fixture.baseline,
                &nonmatching_fixture.candidate,
                &nonmatching_fixture.inputs,
                &mut readers,
            )
            .expect_err("nonmatching topology scan deadline");
    assert_eq!(error.code(), ErrorCode::DurationExpired);

    let mut first_fixture = fixture();
    let first = run(&mut first_fixture, limits()).expect("first result");
    let mut second_fixture = fixture();
    let mut second_limits = limits();
    second_limits.max_duration_ms += 1;
    let second = run(&mut second_fixture, second_limits).expect("second result");
    assert_eq!(first.identity(), second.identity());
    assert_eq!(first.body_bytes(), second.body_bytes());
}

#[test]
#[trace("TC-005", "FR-005-AC-1", "FR-005-AC-3")]
fn tc005_empty_change_sets_complete_under_equal_or_distinct_locks() {
    let make_empty = |fixture: &mut Fixture| {
        comparison_mut(fixture).correspondences.clear();
        let graph = graph_mut(fixture);
        graph.authorities.clear();
        graph.relations.clear();
        graph.scopes.clear();
        graph.provenances.clear();
        graph.nodes.clear();
        graph.edges.clear();
        graph.unresolved.clear();
    };

    let mut distinct_fixture = fixture();
    make_empty(&mut distinct_fixture);
    let distinct = run(&mut distinct_fixture, limits()).expect("distinct empty change set");
    assert_eq!(distinct.outcome(), &TraversalOutcome::Complete);
    assert!(distinct.assessments().is_empty());
    assert!(distinct.subjects().is_empty());

    let mut equal_fixture = fixture();
    make_empty(&mut equal_fixture);
    let lock = equal_fixture.baseline.clone();
    comparison_mut(&mut equal_fixture).candidate_lock = lock.identity();
    let mut readers: Vec<&mut dyn StrictImpactReader> = equal_fixture
        .readers
        .iter_mut()
        .map(|reader| reader as &mut dyn StrictImpactReader)
        .collect();
    let equal = ImpactCoordinator::new(limits())
        .classify(&lock, &lock, &equal_fixture.inputs, &mut readers)
        .expect("equal empty change set");
    assert_eq!(equal.outcome(), &TraversalOutcome::Complete);
    assert!(equal.assessments().is_empty());
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_names_every_invalid_closed_limit_dimension() {
    macro_rules! assert_zero_limit {
        ($field:ident, $resource:expr) => {{
            let mut fixture = fixture();
            let mut run_limits = limits();
            run_limits.$field = 0;
            let error = run(&mut fixture, run_limits).expect_err(stringify!($field));
            assert_eq!(
                (error.code(), error.resource()),
                (ErrorCode::ResourceExhausted, Some($resource)),
                "{}",
                stringify!($field)
            );
        }};
    }
    assert_zero_limit!(max_document_bytes, Resource::DocumentBytes);
    assert_zero_limit!(max_total_input_bytes, Resource::TotalInputBytes);
    assert_zero_limit!(max_identity_bytes, Resource::IdentityBytes);
    assert_zero_limit!(max_reason_bytes, Resource::ReasonBytes);
    assert_zero_limit!(max_correspondences, Resource::Correspondences);
    assert_zero_limit!(max_assessments, Resource::Assessments);
    assert_zero_limit!(max_changed_roots, Resource::ChangedRoots);
    assert_zero_limit!(max_graph_nodes, Resource::GraphNodes);
    assert_zero_limit!(max_graph_edges, Resource::GraphEdges);
    assert_zero_limit!(max_evidence_records, Resource::EvidenceRecords);
    assert_zero_limit!(max_evidence_rights, Resource::EvidenceRights);
    assert_zero_limit!(max_evidence_limitations, Resource::EvidenceLimitations);
    assert_zero_limit!(max_reader_invocations, Resource::ReaderInvocations);
    assert_zero_limit!(max_policy_seed_rules, Resource::PolicySeedRules);
    assert_zero_limit!(
        max_policy_propagation_rules,
        Resource::PolicyPropagationRules
    );
    assert_zero_limit!(max_policy_vector_rules, Resource::PolicyVectorRules);
    assert_zero_limit!(max_vector_definitions, Resource::VectorDefinitions);
    assert_zero_limit!(max_manifest_providers, Resource::ManifestProviders);
    assert_zero_limit!(max_manifest_entries, Resource::ManifestEntries);
    assert_zero_limit!(max_unresolved_entries, Resource::UnresolvedEntries);
    assert_zero_limit!(max_total_reason_bytes, Resource::TotalReasonBytes);
    assert_zero_limit!(max_topology_items, Resource::TopologyItems);
    assert_zero_limit!(max_traversal_steps, Resource::TraversalSteps);
    assert_zero_limit!(max_traversed_edges, Resource::TraversedEdges);
    assert_zero_limit!(max_subject_records, Resource::SubjectRecords);
    assert_zero_limit!(max_path_edges, Resource::PathEdges);
    assert_zero_limit!(max_witnesses, Resource::Witnesses);
    assert_zero_limit!(max_witness_bytes, Resource::WitnessBytes);
    assert_zero_limit!(max_frontier_entries, Resource::FrontierEntries);
    assert_zero_limit!(max_output_bytes, Resource::OutputBytes);
    assert_zero_limit!(max_accounting_bytes, Resource::AccountingBytes);
    assert_zero_limit!(max_duration_ms, Resource::Duration);
}

#[test]
#[trace("TC-005", "TC-007", "FR-005-AC-3")]
fn tc005_admits_exact_and_classifies_one_over_active_boundaries() {
    macro_rules! boundary {
        ($field:ident, $resource:expr, $demand:expr, $phase:expr) => {{
            assert_golden_boundary(fixture, $resource, $demand, $phase, |limits, value| {
                limits.$field = value
            });
        }};
    }
    boundary!(
        max_document_bytes,
        Resource::DocumentBytes,
        3_441,
        BoundaryPhase::Refused
    );
    boundary!(
        max_total_input_bytes,
        Resource::TotalInputBytes,
        212,
        BoundaryPhase::Refused
    );
    boundary!(
        max_identity_bytes,
        Resource::IdentityBytes,
        49,
        BoundaryPhase::Refused
    );
    boundary!(
        max_graph_nodes,
        Resource::GraphNodes,
        5,
        BoundaryPhase::Refused
    );
    boundary!(
        max_graph_edges,
        Resource::GraphEdges,
        5,
        BoundaryPhase::Refused
    );
    boundary!(
        max_evidence_records,
        Resource::EvidenceRecords,
        2,
        BoundaryPhase::Refused
    );
    boundary!(
        max_evidence_rights,
        Resource::EvidenceRights,
        2,
        BoundaryPhase::Refused
    );
    boundary!(
        max_evidence_limitations,
        Resource::EvidenceLimitations,
        2,
        BoundaryPhase::Refused
    );
    boundary!(
        max_reader_invocations,
        Resource::ReaderInvocations,
        6,
        BoundaryPhase::Refused
    );
    boundary!(
        max_policy_seed_rules,
        Resource::PolicySeedRules,
        48,
        BoundaryPhase::Refused
    );
    boundary!(
        max_policy_propagation_rules,
        Resource::PolicyPropagationRules,
        5,
        BoundaryPhase::Refused
    );
    boundary!(
        max_topology_items,
        Resource::TopologyItems,
        10,
        BoundaryPhase::Refused
    );
    boundary!(
        max_traversal_steps,
        Resource::TraversalSteps,
        6,
        BoundaryPhase::Incomplete {
            usage: one_over_traversal_usage(),
            frontier_entries: 1,
        }
    );
    boundary!(
        max_traversed_edges,
        Resource::TraversedEdges,
        5,
        BoundaryPhase::Incomplete {
            usage: one_over_traversal_usage(),
            frontier_entries: 1,
        }
    );
    boundary!(
        max_subject_records,
        Resource::SubjectRecords,
        5,
        BoundaryPhase::Incomplete {
            usage: one_over_subject_usage(),
            frontier_entries: 2,
        }
    );
    boundary!(
        max_path_edges,
        Resource::PathEdges,
        8,
        BoundaryPhase::Incomplete {
            usage: one_over_traversal_usage(),
            frontier_entries: 1,
        }
    );
    boundary!(
        max_witnesses,
        Resource::Witnesses,
        7,
        BoundaryPhase::Incomplete {
            usage: one_over_traversal_usage(),
            frontier_entries: 1,
        }
    );
    boundary!(
        max_witness_bytes,
        Resource::WitnessBytes,
        2_757,
        BoundaryPhase::Incomplete {
            usage: one_over_traversal_usage(),
            frontier_entries: 1,
        }
    );
    boundary!(
        max_output_bytes,
        Resource::OutputBytes,
        7_161,
        BoundaryPhase::Refused
    );
    boundary!(
        max_accounting_bytes,
        Resource::AccountingBytes,
        7_238,
        BoundaryPhase::Refused
    );
}

#[test]
#[trace("TC-005", "TC-007", "FR-005-AC-3")]
fn tc005_admits_exact_and_refuses_one_over_sparse_boundaries() {
    macro_rules! boundary {
        ($fixture:ident, $field:ident, $resource:expr, $demand:expr) => {{
            assert_golden_boundary(
                $fixture,
                $resource,
                $demand,
                BoundaryPhase::Refused,
                |limits, value| limits.$field = value,
            );
        }};
    }
    boundary!(
        two_root_fixture,
        max_correspondences,
        Resource::Correspondences,
        2
    );
    boundary!(two_root_fixture, max_assessments, Resource::Assessments, 2);
    boundary!(
        two_root_fixture,
        max_changed_roots,
        Resource::ChangedRoots,
        2
    );
    boundary!(
        two_vector_fixture,
        max_policy_vector_rules,
        Resource::PolicyVectorRules,
        2
    );
    boundary!(
        two_vector_fixture,
        max_vector_definitions,
        Resource::VectorDefinitions,
        2
    );
    boundary!(
        two_vector_fixture,
        max_manifest_entries,
        Resource::ManifestEntries,
        2
    );
    boundary!(
        two_provider_fixture,
        max_manifest_providers,
        Resource::ManifestProviders,
        2
    );
    boundary!(
        unresolved_boundary_fixture,
        max_reason_bytes,
        Resource::ReasonBytes,
        27
    );
    boundary!(
        unresolved_boundary_fixture,
        max_unresolved_entries,
        Resource::UnresolvedEntries,
        2
    );
    boundary!(
        unresolved_boundary_fixture,
        max_total_reason_bytes,
        Resource::TotalReasonBytes,
        54
    );

    let mut exact_fixture = fixture();
    let mut exact_limits = limits();
    exact_limits.max_subject_records = 1;
    exact_limits.max_frontier_entries = 2;
    let exact = run(&mut exact_fixture, exact_limits).expect("exact frontier");
    assert_eq!(exact.outcome().cause(), Some(Resource::SubjectRecords));
    assert_eq!(exact.outcome().frontier().len(), 2);
    assert_eq!(
        exact.usage(),
        Usage {
            input_bytes: 212,
            reader_invocations: 6,
            topology_items: 10,
            subject_records: 1,
            frontier_entries: 2,
            ..Usage::default()
        }
    );

    let mut one_over_fixture = fixture();
    let mut one_over_limits = exact_limits;
    one_over_limits.max_frontier_entries = 1;
    let error = run(&mut one_over_fixture, one_over_limits).expect_err("one-over frontier");
    assert_eq!(
        (error.code(), error.resource()),
        (
            ErrorCode::ResourceExhausted,
            Some(Resource::FrontierEntries)
        )
    );
}

#[test]
#[trace("TC-005", "FR-005-AC-3")]
fn tc005_bounds_incremental_projection_bytes_before_retention() {
    const EXACT_OWNER_BYTES: usize = 4_027;
    let mut exact_fixture = fixture();
    let vector = &mut corpus_mut(&mut exact_fixture).vectors[0];
    vector.owner_bytes = vec![b'x'; EXACT_OWNER_BYTES];
    vector.digest = Sha256::digest(&vector.owner_bytes).into();
    run(&mut exact_fixture, limits()).expect("exact projection-byte boundary");

    let mut one_over_fixture = fixture();
    let vector = &mut corpus_mut(&mut one_over_fixture).vectors[0];
    vector.owner_bytes = vec![b'x'; EXACT_OWNER_BYTES + 1];
    vector.digest = Sha256::digest(&vector.owner_bytes).into();
    let error = run(&mut one_over_fixture, limits()).expect_err("one-over projection bytes");
    assert_eq!(
        (error.code(), error.resource()),
        (ErrorCode::ResourceExhausted, Some(Resource::DocumentBytes))
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    #[trace(
        "TC-005",
        "TC-007",
        "FR-005-AC-2",
        "FR-005-AC-6"
    )]
    fn tc005_property_permutations_reproduce_canonical_bytes(
        reverse_inputs in any::<bool>(),
        reverse_graph in any::<bool>(),
        reverse_policy in any::<bool>(),
        reverse_evidence in any::<bool>(),
    ) {
        let mut expected_fixture = fixture();
        let expected = run(&mut expected_fixture, limits()).expect("expected result");
        let mut permuted_fixture = fixture();
        if reverse_inputs {
            permuted_fixture.inputs.reverse();
            permuted_fixture.readers.reverse();
        }
        if reverse_graph {
            let graph = graph_mut(&mut permuted_fixture);
            graph.nodes.reverse();
            graph.edges.reverse();
            graph.scopes.reverse();
            graph.provenances.reverse();
        }
        if reverse_graph {
            permuted_fixture
                .readers
                .iter_mut()
                .find(|reader| reader.projection.contract() == InputContract::Graph)
                .expect("graph reader")
                .graph_vocabulary_first = true;
        }
        if reverse_policy {
            let policy = policy_mut(&mut permuted_fixture);
            policy.seed_rules.reverse();
            policy.propagation_rules.reverse();
            policy.vector_rules.reverse();
        }
        if reverse_evidence {
            evidence_mut(&mut permuted_fixture).records.reverse();
        }
        let actual = run(&mut permuted_fixture, limits()).expect("permuted result");
        prop_assert_eq!(actual.body_bytes(), expected.body_bytes());
        prop_assert_eq!(actual.identity(), expected.identity());
    }

    #[test]
    #[trace("TC-005", "FR-005-AC-2")]
    fn tc005_generated_diamonds_choose_the_reference_shortest_path(
        left_edges in 2_usize..6,
        right_edges in 2_usize..6,
        reverse in any::<bool>(),
    ) {
        let mut diamond = finite_diamond_fixture(left_edges, right_edges);
        if reverse {
            let graph = graph_mut(&mut diamond);
            graph.nodes.reverse();
            graph.edges.reverse();
            graph.provenances.reverse();
        }
        let result = run(&mut diamond, limits()).expect("finite diamond");
        let witness = result
            .subjects()
            .iter()
            .find(|record| record.subject.local.as_str() == "shared")
            .and_then(|record| record.affectedness_witness.as_ref())
            .expect("shared affectedness witness");
        prop_assert_eq!(witness.path.len(), left_edges.min(right_edges));
        let expected_branch = if left_edges <= right_edges { "a:0" } else { "b:0" };
        prop_assert_eq!(witness.path[0].target.local.as_str(), expected_branch);
    }
}
