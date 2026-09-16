// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-001: exact ecosystem-lock validation and canonicalization.

use std::cell::Cell;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_integration::ecosystem_lock::{
    validate, validate_then, validate_with_monotonic_clock, CapabilityBinding, CapabilitySelection,
    Compatibility, ComponentBinding, DependencyEdge, Limits, LockSelection, MonotonicClock,
    PackageBinding, Pin, PinState, Reachability, RefusalCode, RepositoryBinding, RoleBinding,
    SelectionDomain, WorkflowBinding, CONTRACT, SCHEMA_SHA256,
};
use quire_integration::{
    ArtifactId, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, PackageId,
    ProviderId, RepositoryId, RoleId, RunId, ToolchainId, WorkflowInstanceId,
};
use sha2::{Digest as _, Sha256};

const ID_LIMIT: u64 = 128;

#[derive(Debug, Default)]
struct StepClock(AtomicU64);

impl StepClock {
    fn calls(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

impl MonotonicClock for StepClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

fn id<T>(
    value: &'static str,
    constructor: fn(&'static str, u64) -> quire_integration::Result<T>,
) -> T {
    constructor(value, ID_LIMIT).expect("fixture identity")
}

fn exact<T: quire_integration::ecosystem_lock::PinIdentity>(identity: T, revision: &str) -> Pin<T> {
    let revision = exact_revision(revision);
    Pin::new(identity, &revision, PinState::Exact, ID_LIMIT).expect("immutable fixture pin")
}

fn exact_revision(label: &str) -> String {
    if label.len() == 40
        && label
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        format!("git:{label}")
    } else {
        format!("sha256:{:x}", Sha256::digest(label.as_bytes()))
    }
}

fn fixture() -> LockSelection {
    let repository = id("agent-ix/quire-observation", RepositoryId::new);
    let core_component = id("quire-observation-core", ComponentId::new);
    let query_component = id("quire-observation-query", ComponentId::new);
    let core_package = id("quire-observation-core", PackageId::new);
    let query_package = id("quire-observation-query", PackageId::new);
    let role_owner = id("observation-owner", RoleId::new);
    let role_lineage = id("observation-lineage", RoleId::new);
    let role_query = id("observation-query", RoleId::new);
    LockSelection::new(
        vec![RepositoryBinding::new(
            exact(
                repository.clone(),
                "581d98f1ac9f1467cc5355785d67b242f8d5d150",
            ),
            "https://github.com/agent-ix/quire-observation",
            ID_LIMIT,
        )
        .unwrap()],
        vec![
            PackageBinding::new(
                exact(core_package.clone(), "core-package-content-v1"),
                "0.1.0",
                repository.clone(),
                &exact_revision("581d98f1ac9f1467cc5355785d67b242f8d5d150"),
                &["default"],
                ID_LIMIT,
                ID_LIMIT,
            )
            .unwrap(),
            PackageBinding::new(
                exact(query_package.clone(), "query-package-content-v1"),
                "0.1.0",
                repository.clone(),
                &exact_revision("581d98f1ac9f1467cc5355785d67b242f8d5d150"),
                &["default"],
                ID_LIMIT,
                ID_LIMIT,
            )
            .unwrap(),
        ],
        vec![exact(
            id("timed-refund", DefinitionId::new),
            "sha256:definition",
        )],
        vec![
            exact(id("local", ProviderId::new), "provider-v1"),
            exact(id("local-alt", ProviderId::new), "provider-alt-v1"),
        ],
        vec![exact(id("rust-1.98.1", ToolchainId::new), "rustc-1.98.1")],
        vec![
            exact(id("qobs-build", BuildId::new), "sha256:build"),
            exact(id("qobs-build-alt", BuildId::new), "sha256:build-alt"),
        ],
        vec![exact(
            id("local-test", DeploymentId::new),
            "sha256:deployment",
        )],
        vec![
            ComponentBinding::new(
                exact(core_component.clone(), "quire.observation-authority/v1"),
                repository.clone(),
                core_package.clone(),
            ),
            ComponentBinding::new(
                exact(query_component.clone(), "quire.observation.query/v1"),
                repository,
                query_package.clone(),
            ),
        ],
        vec![
            RoleBinding::new(
                exact(role_owner, "quire.observation-authority/v1"),
                core_component.clone(),
            ),
            RoleBinding::new(
                exact(role_lineage, "quire.observation-authority-lineage/v1"),
                core_component,
            ),
            RoleBinding::new(
                exact(role_query, "quire.observation.query/v1"),
                query_component,
            ),
        ],
        vec![
            CapabilityBinding::new(CapabilitySelection::new(
                id("observation-owner", RoleId::new),
                id("local", ProviderId::new),
                id("qobs-build", BuildId::new),
                id("quire.observation-authority/v1", ContractId::new),
                id("qobs-options:default", ArtifactId::new),
            )),
            CapabilityBinding::new(CapabilitySelection::new(
                id("observation-query", RoleId::new),
                id("local", ProviderId::new),
                id("qobs-build", BuildId::new),
                id("quire.observation.query/v1", ContractId::new),
                id("query-options:bounded", ArtifactId::new),
            )),
            CapabilityBinding::new(CapabilitySelection::new(
                id("observation-lineage", RoleId::new),
                id("local", ProviderId::new),
                id("qobs-build", BuildId::new),
                id("quire.observation-authority-lineage/v1", ContractId::new),
                id("lineage-options:default", ArtifactId::new),
            )),
        ],
        vec![WorkflowBinding::new(
            exact(
                id("refund:2026-09-15", WorkflowInstanceId::new),
                "workflow-revision-1",
            ),
            id("timed-refund", DefinitionId::new),
        )],
        vec![DependencyEdge::new(
            core_package,
            query_package,
            "quire.observation-authority/v1",
            Reachability::Reachable,
            Compatibility::Compatible,
            ID_LIMIT,
        )
        .unwrap()],
    )
}

fn limits(selection: &LockSelection) -> Limits {
    Limits::for_selection(selection, 1_000_000, ID_LIMIT, 10_000)
        .expect("fixture-derived exact limits")
}

fn assert_selection_changes(
    baseline: &quire_integration::ecosystem_lock::ValidatedLock,
    selection: &LockSelection,
) {
    let changed =
        validate(selection, limits(selection)).expect("meaning-affecting mutation remains valid");
    assert_ne!(changed.identity(), baseline.identity());
    assert_ne!(changed.bytes(), baseline.bytes());
}

fn permute<T>(values: &mut [T], seeds: &[u64]) {
    for (index, seed) in seeds.iter().copied().enumerate().take(values.len()) {
        let target = index
            + usize::try_from(seed % u64::try_from(values.len() - index).unwrap())
                .expect("bounded permutation index");
        values.swap(index, target);
    }
}

fn classification(index: u8) -> (Reachability, Compatibility) {
    let reachability = match index / 5 {
        0 => Reachability::Reachable,
        1 => Reachability::Unreachable,
        2 => Reachability::Unresolved,
        _ => Reachability::Cyclic,
    };
    let compatibility = match index % 5 {
        0 => Compatibility::Compatible,
        1 => Compatibility::Incompatible,
        2 => Compatibility::Unknown,
        3 => Compatibility::Affected,
        _ => Compatibility::Stale,
    };
    (reachability, compatibility)
}

fn classification_error(
    reachability: Reachability,
    compatibility: Compatibility,
) -> Option<RefusalCode> {
    match reachability {
        Reachability::Unreachable => Some(RefusalCode::UnreachableDependency),
        Reachability::Unresolved => Some(RefusalCode::UnresolvedDependency),
        Reachability::Cyclic => Some(RefusalCode::CyclicDependency),
        Reachability::Reachable => match compatibility {
            Compatibility::Compatible => None,
            Compatibility::Incompatible => Some(RefusalCode::IncompatibleContract),
            Compatibility::Unknown => Some(RefusalCode::UnknownCompatibility),
            Compatibility::Affected => Some(RefusalCode::AffectedSelection),
            Compatibility::Stale => Some(RefusalCode::StaleSelection),
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    #[trace("TC-007")]
    fn tc001_generated_discovery_permutations_are_canonical(
        package_seeds in any::<[u64; 2]>(),
        component_seeds in any::<[u64; 2]>(),
        role_seeds in any::<[u64; 3]>(),
        capability_seeds in any::<[u64; 3]>(),
    ) {
        // FR-001-AC-1
        let baseline_selection = fixture();
        let baseline = validate(&baseline_selection, limits(&baseline_selection)).unwrap();
        let mut candidate = fixture();
        permute(candidate.packages_mut(), &package_seeds);
        permute(candidate.components_mut(), &component_seeds);
        permute(candidate.roles_mut(), &role_seeds);
        permute(candidate.capabilities_mut(), &capability_seeds);
        let permuted = validate(&candidate, limits(&candidate)).unwrap();
        prop_assert_eq!(permuted.identity(), baseline.identity());
        prop_assert_eq!(permuted.bytes(), baseline.bytes());
    }

    #[test]
    fn tc001_generated_meaning_mutations_change_lock_and_run_identity(
        domain in 0_u8..14,
        nonce in any::<u64>(),
    ) {
        // FR-001-AC-2
        let baseline_selection = fixture();
        let baseline = validate(&baseline_selection, limits(&baseline_selection)).unwrap();
        let run = RunId::new("run:timed-refund", ID_LIMIT).unwrap();
        let baseline_run = baseline.derive_run_identity(&run).unwrap();
        let label = format!("generated-{domain}-{nonce}");
        let revision = exact_revision(&label);
        let mut candidate = fixture();
        match domain {
            0 => {
                candidate.repositories_mut()[0].pin_mut().set_revision(&revision, ID_LIMIT).unwrap();
                for package in candidate.packages_mut() {
                    package.set_repository_revision(&revision, ID_LIMIT).unwrap();
                }
            }
            1 => candidate.packages_mut()[0].pin_mut().set_revision(&revision, ID_LIMIT).unwrap(),
            2 => candidate.packages_mut()[0]
                .set_version(&format!("1.0.{nonce}"), ID_LIMIT)
                .unwrap(),
            3 => candidate.definitions_mut()[0].set_revision(&revision, ID_LIMIT).unwrap(),
            4 => candidate.providers_mut()[0].set_revision(&revision, ID_LIMIT).unwrap(),
            5 => candidate.toolchains_mut()[0].set_revision(&revision, ID_LIMIT).unwrap(),
            6 => candidate.builds_mut()[0].set_revision(&revision, ID_LIMIT).unwrap(),
            7 => candidate.deployments_mut()[0].set_revision(&revision, ID_LIMIT).unwrap(),
            8 => candidate.components_mut()[0].pin_mut().set_revision(&revision, ID_LIMIT).unwrap(),
            9 => candidate.roles_mut()[0].pin_mut().set_revision(&revision, ID_LIMIT).unwrap(),
            10 => candidate.workflows_mut()[0].pin_mut().set_revision(&revision, ID_LIMIT).unwrap(),
            11 => {
                let selection = candidate.capabilities()[0].selection();
                let changed = CapabilitySelection::new(
                    selection.role().clone(),
                    selection.provider().clone(),
                    selection.build().clone(),
                    ContractId::new(&label, ID_LIMIT).unwrap(),
                    selection.options().clone(),
                );
                *candidate.capabilities_mut()[0].selection_mut() = changed;
            }
            12 => {
                let selection = candidate.capabilities()[0].selection();
                let changed = CapabilitySelection::new(
                    selection.role().clone(),
                    selection.provider().clone(),
                    selection.build().clone(),
                    selection.contract().clone(),
                    ArtifactId::new(&label, ID_LIMIT).unwrap(),
                );
                *candidate.capabilities_mut()[0].selection_mut() = changed;
            }
            _ => candidate.dependencies_mut()[0].set_contract(&label, ID_LIMIT).unwrap(),
        }
        let changed = validate(&candidate, limits(&candidate)).unwrap();
        prop_assert_ne!(changed.identity(), baseline.identity());
        prop_assert_ne!(changed.bytes(), baseline.bytes());
        prop_assert_ne!(changed.derive_run_identity(&run).unwrap(), baseline_run);
    }

    #[test]
    fn tc001_generated_reachability_and_compatibility_remain_distinct(index in 0_u8..20) {
        // FR-001-AC-5
        let (reachability, compatibility) = classification(index);
        let mut selection = fixture();
        selection.dependencies_mut()[0].set_reachability(reachability);
        selection.dependencies_mut()[0].set_compatibility(compatibility);
        let result = validate(&selection, limits(&selection));
        match classification_error(reachability, compatibility) {
            Some(expected) => prop_assert_eq!(result.unwrap_err().code(), expected),
            None => prop_assert!(result.is_ok()),
        }
    }
}

#[test]
#[trace("TC-001", "TC-007", "FR-001-AC-1")]
fn tc001_discovery_order_does_not_change_canonical_lock_or_identity() {
    let left_selection = fixture();
    let left = validate(&left_selection, limits(&left_selection)).expect("valid left lock");
    for permutation in 0_u8..16 {
        let mut candidate = fixture();
        if permutation & 1 != 0 {
            candidate.packages_mut().reverse();
        }
        if permutation & 2 != 0 {
            candidate.components_mut().reverse();
        }
        if permutation & 4 != 0 {
            candidate.roles_mut().reverse();
        }
        if permutation & 8 != 0 {
            candidate.capabilities_mut().reverse();
        }
        let permuted = validate(&candidate, limits(&candidate)).expect("generated permutation");
        assert_eq!(left.identity(), permuted.identity());
        assert_eq!(left.bytes(), permuted.bytes());
    }
    assert_eq!(left.contract(), CONTRACT);
    assert_eq!(left.schema_sha256(), SCHEMA_SHA256);
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(quire_integration::ecosystem_lock::SCHEMA_BYTES)
        ),
        SCHEMA_SHA256
    );

    let reread = quire_integration::ecosystem_lock::read(
        left.bytes(),
        &left_selection,
        limits(&left_selection),
    )
    .expect("strict reader accepts only the same canonical lock");
    assert_eq!(reread.bytes(), left.bytes());

    let schema: serde_json::Value =
        serde_json::from_slice(quire_integration::ecosystem_lock::SCHEMA_BYTES).unwrap();
    let validator = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("pinned ecosystem-lock schema compiles");
    let emitted: serde_json::Value = serde_json::from_slice(left.bytes()).unwrap();
    assert!(validator.is_valid(&emitted));
}

#[test]
#[trace("TC-001", "TC-007", "FR-001-AC-3")]
fn tc001_every_collection_and_byte_limit_admits_exact_and_refuses_one_over() {
    type Lower = fn(&mut Limits);
    let selection = fixture();
    let mut exact = limits(&selection);
    loop {
        let baseline = validate(&selection, exact).expect("baseline lock");
        let measured = u64::try_from(baseline.bytes().len()).expect("output length");
        if measured == exact.max_output_bytes && measured == exact.max_document_bytes {
            break;
        }
        exact.max_output_bytes = measured;
        exact.max_document_bytes = measured;
    }
    let exact_output = exact.max_output_bytes;
    assert!(validate(&selection, exact).is_ok());

    let cases: [(Lower, SelectionDomain); 13] = [
        (
            |limits| limits.max_repositories = 0,
            SelectionDomain::Repository,
        ),
        (|limits| limits.max_packages = 1, SelectionDomain::Package),
        (
            |limits| limits.max_definitions = 0,
            SelectionDomain::Definition,
        ),
        (|limits| limits.max_providers = 0, SelectionDomain::Provider),
        (
            |limits| limits.max_toolchains = 0,
            SelectionDomain::Toolchain,
        ),
        (|limits| limits.max_builds = 0, SelectionDomain::Build),
        (
            |limits| limits.max_deployments = 0,
            SelectionDomain::Deployment,
        ),
        (
            |limits| limits.max_components = 1,
            SelectionDomain::Component,
        ),
        (|limits| limits.max_roles = 2, SelectionDomain::Role),
        (
            |limits| limits.max_capabilities = 2,
            SelectionDomain::Capability,
        ),
        (
            |limits| limits.max_workflows = 0,
            SelectionDomain::WorkflowInstance,
        ),
        (
            |limits| limits.max_dependencies = 0,
            SelectionDomain::Dependency,
        ),
        (
            |limits| limits.max_traversal_steps -= 1,
            SelectionDomain::Dependency,
        ),
    ];
    for (lower, domain) in cases {
        let mut one_under = exact;
        lower(&mut one_under);
        let refusal = validate(&selection, one_under).expect_err("one-over collection");
        assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
        assert_eq!(refusal.domain(), Some(domain));
        assert!(refusal.selection_identity().is_some());
    }

    let input_one_under = Limits {
        max_input_bytes: exact.max_input_bytes - 1,
        ..exact
    };
    let refusal = validate(&selection, input_one_under).expect_err("one-over input bytes");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Lock));
    assert_eq!(refusal.selection_identity(), Some(CONTRACT));

    let document_one_under = Limits {
        max_document_bytes: exact.max_document_bytes - 1,
        ..exact
    };
    let refusal = quire_integration::ecosystem_lock::read(
        validate(&selection, exact).unwrap().bytes(),
        &selection,
        document_one_under,
    )
    .expect_err("one-over strict-reader document bytes");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Lock));
    assert_eq!(refusal.selection_identity(), Some(CONTRACT));

    let output_one_under = Limits {
        max_output_bytes: exact_output - 1,
        ..exact
    };
    let refusal = validate(&selection, output_one_under).expect_err("one-over output bytes");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Lock));
    assert_eq!(refusal.selection_identity(), Some(CONTRACT));
}

#[test]
#[trace("TC-001", "FR-001-AC-3")]
fn tc001_strict_reader_rejects_noncanonical_and_unknown_fields() {
    let selection = fixture();
    let exact = limits(&selection);
    let lock = validate(&selection, exact).expect("baseline lock");

    let mut whitespace = lock.bytes().to_vec();
    whitespace.push(b'\n');
    let refusal = quire_integration::ecosystem_lock::read(&whitespace, &selection, exact)
        .expect_err("noncanonical whitespace");
    assert_eq!(refusal.code(), RefusalCode::CanonicalMismatch);

    let mut unknown: serde_json::Value = serde_json::from_slice(lock.bytes()).unwrap();
    unknown["ambientDefault"] = true.into();
    let unknown = serde_json::to_vec(&unknown).unwrap();
    let refusal = quire_integration::ecosystem_lock::read(&unknown, &selection, exact)
        .expect_err("unknown field");
    assert_eq!(refusal.code(), RefusalCode::CanonicalMismatch);
}

#[test]
#[trace("TC-001", "FR-001-AC-2")]
fn tc001_every_meaning_affecting_pin_changes_lock_identity() {
    let baseline_selection = fixture();
    let baseline = validate(&baseline_selection, limits(&baseline_selection)).unwrap();
    let run = RunId::new("run:timed-refund", ID_LIMIT).unwrap();
    let baseline_run = baseline.derive_run_identity(&run).unwrap();
    let mut changed_selection = fixture();
    changed_selection.packages_mut()[0]
        .pin_mut()
        .set_revision(&exact_revision("0.1.1"), ID_LIMIT)
        .unwrap();
    let changed = validate(&changed_selection, limits(&changed_selection)).unwrap();
    assert_ne!(changed.identity(), baseline.identity());
    assert_ne!(changed.bytes(), baseline.bytes());
    assert_ne!(changed.derive_run_identity(&run).unwrap(), baseline_run);

    let mut changed_version = fixture();
    changed_version.packages_mut()[0]
        .set_version("0.1.1", ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &changed_version);

    let mut repository = fixture();
    repository.repositories_mut()[0]
        .pin_mut()
        .set_revision(
            &exact_revision("681d98f1ac9f1467cc5355785d67b242f8d5d150"),
            ID_LIMIT,
        )
        .unwrap();
    repository.packages_mut()[0]
        .set_repository_revision(
            &exact_revision("681d98f1ac9f1467cc5355785d67b242f8d5d150"),
            ID_LIMIT,
        )
        .unwrap();
    repository.packages_mut()[1]
        .set_repository_revision(
            &exact_revision("681d98f1ac9f1467cc5355785d67b242f8d5d150"),
            ID_LIMIT,
        )
        .unwrap();
    assert_selection_changes(&baseline, &repository);

    let mut definition = fixture();
    definition.definitions_mut()[0]
        .set_revision(&exact_revision("definition-v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &definition);
    let mut provider = fixture();
    provider.providers_mut()[0]
        .set_revision(&exact_revision("provider-v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &provider);
    let mut toolchain = fixture();
    toolchain.toolchains_mut()[0]
        .set_revision(&exact_revision("rustc-1.98.2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &toolchain);
    let mut build = fixture();
    build.builds_mut()[0]
        .set_revision(&exact_revision("build-v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &build);
    let mut deployment = fixture();
    deployment.deployments_mut()[0]
        .set_revision(&exact_revision("deployment-v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &deployment);
    let mut component = fixture();
    component.components_mut()[0]
        .pin_mut()
        .set_revision(&exact_revision("quire.observation-authority/v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &component);
    let mut role = fixture();
    role.roles_mut()[0]
        .pin_mut()
        .set_revision(&exact_revision("quire.observation-authority/v2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &role);

    let mut capability_role = fixture();
    let first = capability_role.capabilities()[0].selection().clone();
    let second = capability_role.capabilities()[1].selection().clone();
    *capability_role.capabilities_mut()[0].selection_mut() = CapabilitySelection::new(
        second.role().clone(),
        first.provider().clone(),
        first.build().clone(),
        first.contract().clone(),
        first.options().clone(),
    );
    *capability_role.capabilities_mut()[1].selection_mut() = CapabilitySelection::new(
        first.role().clone(),
        second.provider().clone(),
        second.build().clone(),
        second.contract().clone(),
        second.options().clone(),
    );
    assert_selection_changes(&baseline, &capability_role);

    let mut capability_provider = fixture();
    let selection = capability_provider.capabilities()[0].selection();
    let changed = CapabilitySelection::new(
        selection.role().clone(),
        id("local-alt", ProviderId::new),
        selection.build().clone(),
        selection.contract().clone(),
        selection.options().clone(),
    );
    *capability_provider.capabilities_mut()[0].selection_mut() = changed;
    assert_selection_changes(&baseline, &capability_provider);

    let mut capability_build = fixture();
    let selection = capability_build.capabilities()[0].selection();
    let changed = CapabilitySelection::new(
        selection.role().clone(),
        selection.provider().clone(),
        id("qobs-build-alt", BuildId::new),
        selection.contract().clone(),
        selection.options().clone(),
    );
    *capability_build.capabilities_mut()[0].selection_mut() = changed;
    assert_selection_changes(&baseline, &capability_build);

    let mut capability_contract = fixture();
    let selection = capability_contract.capabilities()[0].selection();
    let changed = CapabilitySelection::new(
        selection.role().clone(),
        selection.provider().clone(),
        selection.build().clone(),
        id("quire.observation-authority/v2", ContractId::new),
        selection.options().clone(),
    );
    *capability_contract.capabilities_mut()[0].selection_mut() = changed;
    assert_selection_changes(&baseline, &capability_contract);

    let mut capability_options = fixture();
    let selection = capability_options.capabilities()[0].selection();
    let changed = CapabilitySelection::new(
        selection.role().clone(),
        selection.provider().clone(),
        selection.build().clone(),
        selection.contract().clone(),
        id("qobs-options:strict", ArtifactId::new),
    );
    *capability_options.capabilities_mut()[0].selection_mut() = changed;
    assert_selection_changes(&baseline, &capability_options);

    let mut workflow = fixture();
    workflow.workflows_mut()[0]
        .pin_mut()
        .set_revision(&exact_revision("workflow-revision-2"), ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &workflow);
    let mut dependency = fixture();
    dependency.dependencies_mut()[0]
        .set_contract("quire.observation-authority/v2", ID_LIMIT)
        .unwrap();
    assert_selection_changes(&baseline, &dependency);

    let changed_limits = Limits {
        max_document_bytes: limits(&baseline_selection).max_document_bytes + 1,
        ..limits(&baseline_selection)
    };
    let limit_changed = validate(&baseline_selection, changed_limits).unwrap();
    assert_ne!(limit_changed.identity(), baseline.identity());
    assert_ne!(
        limit_changed.derive_run_identity(&run).unwrap(),
        baseline_run
    );

    let mismatch = quire_integration::ecosystem_lock::read(
        baseline.bytes(),
        &changed_selection,
        limits(&changed_selection),
    )
    .expect_err("strict reader rejects bytes for a different selection");
    assert_eq!(mismatch.code(), RefusalCode::CanonicalMismatch);
}

#[test]
#[trace("TC-001", "TC-007", "FR-001-AC-3")]
fn tc001_invalid_states_and_one_over_refuse_before_invocation() {
    let invoked = Cell::new(0_u64);
    let cases = [
        (PinState::Missing, RefusalCode::MissingSelection),
        (PinState::Stale, RefusalCode::StaleSelection),
        (PinState::Dirty, RefusalCode::DirtySelection),
        (PinState::Moving, RefusalCode::MovingSelection),
        (PinState::Unknown, RefusalCode::UnknownSelection),
    ];
    for (state, expected) in cases {
        let mut selection = fixture();
        selection.repositories_mut()[0]
            .pin_mut()
            .set_state(state)
            .unwrap();
        let result = validate(&selection, limits(&selection));
        assert_eq!(
            result.expect_err("invalid pin must refuse").code(),
            expected
        );
        let refusal = validate(&selection, limits(&selection)).expect_err("invalid pin");
        assert_eq!(
            refusal.selection_identity(),
            Some("agent-ix/quire-observation")
        );
        assert_eq!(invoked.get(), 0);
    }

    let selection = fixture();
    let exact = limits(&selection);
    let one_over = Limits {
        max_repositories: 0,
        ..exact
    };
    let result = validate(&selection, one_over);
    let refusal = result.expect_err("one-over repository count must refuse");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Repository));
    assert_eq!(invoked.get(), 0);

    validate_then(&selection, exact, |_| invoked.set(invoked.get() + 1))
        .expect("validated lock admits the invocation seam");
    assert_eq!(invoked.get(), 1);
}

#[test]
#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-3")]
fn tc001_dependency_free_complete_lock_is_valid() {
    let mut selection = fixture();
    selection.dependencies_mut().clear();
    let lock = validate(&selection, limits(&selection)).expect("dependencies are optional");
    let value: serde_json::Value = serde_json::from_slice(lock.bytes()).unwrap();
    assert_eq!(value["dependencies"], serde_json::json!([]));
}

#[test]
#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-3")]
fn tc001_package_features_are_a_canonical_duplicate_free_set() {
    let mut left_selection = fixture();
    left_selection.packages_mut()[0]
        .set_features(&["feature-b", "feature-a"], 2, ID_LIMIT)
        .unwrap();
    let mut right_selection = fixture();
    right_selection.packages_mut()[0]
        .set_features(&["feature-a", "feature-b"], 2, ID_LIMIT)
        .unwrap();
    let left = validate(&left_selection, limits(&left_selection)).unwrap();
    let right = validate(&right_selection, limits(&right_selection)).unwrap();
    assert_eq!(left.identity(), right.identity());
    assert_eq!(left.bytes(), right.bytes());

    let refusal = right_selection.packages_mut()[0]
        .set_features(&["duplicate", "duplicate"], 2, ID_LIMIT)
        .expect_err("duplicate features must refuse");
    assert_eq!(refusal.code(), RefusalCode::DuplicateSelection);
}

#[test]
#[trace("TC-001", "FR-001-AC-3", "FR-001-AC-4")]
fn tc001_duplicate_and_cross_domain_mappings_refuse_without_collapsing_roles() {
    let selection = fixture();
    let validated = validate(&selection, limits(&selection)).expect("multi-role lock");
    assert_eq!(validated.roles().count(), 3);
    assert_eq!(validated.components().count(), 2);
    assert_eq!(validated.repositories().count(), 1);

    let mut duplicate = fixture();
    let duplicated_role = duplicate.roles()[0].clone();
    duplicate.roles_mut().push(duplicated_role);
    let refusal = validate(&duplicate, limits(&duplicate)).expect_err("duplicate role");
    assert_eq!(refusal.code(), RefusalCode::DuplicateSelection);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Role));

    let mut cross_domain = fixture();
    *cross_domain.roles_mut()[0].component_mut() = id("foreign", ComponentId::new);
    let refusal = validate(&cross_domain, limits(&cross_domain)).expect_err("foreign component");
    assert_eq!(refusal.code(), RefusalCode::CrossDomainReference);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Component));

    let mut mismatched_revision = fixture();
    mismatched_revision.packages_mut()[0]
        .set_repository_revision("foreign-revision", ID_LIMIT)
        .unwrap();
    let refusal = validate(&mismatched_revision, limits(&mismatched_revision))
        .expect_err("package cannot repoint its selected repository revision");
    assert_eq!(refusal.code(), RefusalCode::CrossDomainReference);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Repository));
}

#[test]
#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-4")]
fn tc001_capabilities_cover_roles_once_and_admit_only_the_exact_bound_tuple() {
    let selection = fixture();
    let lock = validate(&selection, limits(&selection)).expect("complete capability closure");
    let requested = selection.capabilities()[0].selection();
    let admitted = lock
        .admit_capability(requested)
        .expect("exact capability tuple is admitted");
    assert_eq!(admitted.lock_identity(), lock.identity());
    assert_eq!(admitted.role(), requested.role().as_str());
    assert_eq!(admitted.provider(), requested.provider().as_str());
    assert_eq!(admitted.build(), requested.build().as_str());
    assert_eq!(admitted.contract(), requested.contract().as_str());
    assert_eq!(admitted.options(), requested.options().as_str());
    assert_eq!(admitted.component(), "quire-observation-core");
    assert_eq!(admitted.package(), "quire-observation-core");
    assert_eq!(admitted.features(), &["default"]);

    let mismatches = [
        CapabilitySelection::new(
            id("observation-query", RoleId::new),
            requested.provider().clone(),
            requested.build().clone(),
            requested.contract().clone(),
            requested.options().clone(),
        ),
        CapabilitySelection::new(
            requested.role().clone(),
            id("local-alt", ProviderId::new),
            requested.build().clone(),
            requested.contract().clone(),
            requested.options().clone(),
        ),
        CapabilitySelection::new(
            requested.role().clone(),
            requested.provider().clone(),
            id("qobs-build-alt", BuildId::new),
            requested.contract().clone(),
            requested.options().clone(),
        ),
        CapabilitySelection::new(
            requested.role().clone(),
            requested.provider().clone(),
            requested.build().clone(),
            id("quire.observation-authority/v2", ContractId::new),
            requested.options().clone(),
        ),
        CapabilitySelection::new(
            requested.role().clone(),
            requested.provider().clone(),
            requested.build().clone(),
            requested.contract().clone(),
            id("qobs-options:strict", ArtifactId::new),
        ),
    ];
    for mismatch in mismatches {
        let refusal = lock
            .admit_capability(&mismatch)
            .expect_err("one changed capability axis is not admitted");
        assert_eq!(refusal.code(), RefusalCode::UnknownSelection);
        assert_eq!(refusal.domain(), Some(SelectionDomain::Capability));
        assert!(refusal.selection_identity().is_some());
    }

    let mut missing = fixture();
    let removed = missing.capabilities_mut().pop().unwrap();
    let refusal = validate(&missing, limits(&missing)).expect_err("every role needs one binding");
    assert_eq!(refusal.code(), RefusalCode::MissingSelection);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Capability));
    assert_eq!(
        refusal.selection_identity(),
        Some(removed.selection().role().as_str())
    );

    let mut duplicate = fixture();
    let selected = duplicate.capabilities()[0].selection().clone();
    duplicate
        .capabilities_mut()
        .push(CapabilityBinding::new(CapabilitySelection::new(
            selected.role().clone(),
            selected.provider().clone(),
            selected.build().clone(),
            id("duplicate-contract", ContractId::new),
            selected.options().clone(),
        )));
    let refusal = validate(&duplicate, limits(&duplicate)).expect_err("role is bound twice");
    assert_eq!(refusal.code(), RefusalCode::DuplicateSelection);
    assert_eq!(refusal.domain(), Some(SelectionDomain::Capability));
    assert_eq!(refusal.selection_identity(), Some("observation-owner"));

    type Foreign = fn(&mut CapabilitySelection);
    let foreign_cases: [(Foreign, SelectionDomain); 3] = [
        (
            |selection| {
                *selection = CapabilitySelection::new(
                    id("foreign-role", RoleId::new),
                    selection.provider().clone(),
                    selection.build().clone(),
                    selection.contract().clone(),
                    selection.options().clone(),
                );
            },
            SelectionDomain::Role,
        ),
        (
            |selection| {
                *selection = CapabilitySelection::new(
                    selection.role().clone(),
                    id("foreign-provider", ProviderId::new),
                    selection.build().clone(),
                    selection.contract().clone(),
                    selection.options().clone(),
                );
            },
            SelectionDomain::Provider,
        ),
        (
            |selection| {
                *selection = CapabilitySelection::new(
                    selection.role().clone(),
                    selection.provider().clone(),
                    id("foreign-build", BuildId::new),
                    selection.contract().clone(),
                    selection.options().clone(),
                );
            },
            SelectionDomain::Build,
        ),
    ];
    for (mutate, domain) in foreign_cases {
        let mut foreign = fixture();
        mutate(foreign.capabilities_mut()[0].selection_mut());
        let refusal = validate(&foreign, limits(&foreign)).expect_err("foreign capability axis");
        assert_eq!(refusal.code(), RefusalCode::CrossDomainReference);
        assert_eq!(refusal.domain(), Some(domain));
    }
}

#[test]
#[trace("TC-001", "FR-001-AC-5")]
fn tc001_reachability_and_compatibility_states_are_independent() {
    for index in 0_u8..20 {
        let (reachability, compatibility) = classification(index);
        let expected = classification_error(reachability, compatibility);
        let mut selection = fixture();
        selection.dependencies_mut()[0].set_reachability(reachability);
        selection.dependencies_mut()[0].set_compatibility(compatibility);
        let result = validate(&selection, limits(&selection));
        if let Some(expected) = expected {
            assert_eq!(
                result.expect_err("non-compatible edge must refuse").code(),
                expected
            );
        } else {
            result.expect("reachable compatible edge");
        }
    }

    let mut cycle = fixture();
    cycle.dependencies_mut().push(
        DependencyEdge::new(
            id("quire-observation-query", PackageId::new),
            id("quire-observation-core", PackageId::new),
            "quire.observation.query-to-core/v1",
            Reachability::Reachable,
            Compatibility::Compatible,
            ID_LIMIT,
        )
        .unwrap(),
    );
    let refusal = validate(&cycle, limits(&cycle)).expect_err("resolved graph cycle");
    assert_eq!(refusal.code(), RefusalCode::CyclicDependency);
}

#[test]
#[trace("TC-007", "NFR-001-AC-3")]
fn tc007_lock_deadline_is_work_proportional_and_excluded_from_identity() {
    let baseline = fixture();
    let mut generous = limits(&baseline);
    generous.max_duration_ms = 1_000_000;
    let baseline_clock = Arc::new(StepClock::default());
    let baseline_lock = validate_with_monotonic_clock(&baseline, generous, baseline_clock.clone())
        .expect("generous injected deadline admits the baseline lock");
    let baseline_calls = baseline_clock.calls();
    assert!(baseline_calls > 1);

    let mut alternate_duration = generous;
    alternate_duration.max_duration_ms = 2_000_000;
    let same_lock = validate(&baseline, alternate_duration)
        .expect("operational duration remains outside canonical identity");
    assert_eq!(baseline_lock.identity(), same_lock.identity());
    assert_eq!(baseline_lock.bytes(), same_lock.bytes());

    let mut expanded = fixture();
    expanded.packages_mut()[0]
        .set_features(&["a", "b", "c", "d", "e", "f", "g", "h"], 8, ID_LIMIT)
        .expect("expanded feature set remains valid");
    let mut deadline = limits(&expanded);
    deadline.max_duration_ms = baseline_calls;
    let refusal =
        validate_with_monotonic_clock(&expanded, deadline, Arc::new(StepClock::default()))
            .expect_err("additional per-field work must exhaust the baseline-sized deadline");
    assert_eq!(refusal.code(), RefusalCode::DurationExpired);

    let mut zero = limits(&baseline);
    zero.max_duration_ms = 0;
    let refusal = validate_with_monotonic_clock(&baseline, zero, Arc::new(StepClock::default()))
        .expect_err("zero duration refuses before lock output exists");
    assert_eq!(refusal.code(), RefusalCode::DurationExpired);
}

#[test]
#[trace("TC-001", "FR-001-AC-3")]
fn tc001_moving_revisions_cannot_be_constructed_as_exact_pins() {
    for revision in [
        "main",
        "develop",
        "release",
        "trunk",
        "v1.2.3",
        "latest",
        "HEAD",
        "refs/heads/release",
        "v1.*",
    ] {
        let refusal = Pin::new(
            id("agent-ix/moving", RepositoryId::new),
            revision,
            PinState::Exact,
            ID_LIMIT,
        )
        .expect_err("moving revision cannot be exact");
        assert_eq!(refusal.code(), RefusalCode::MovingSelection);
    }
}

#[test]
#[trace("TC-001", "FR-001-AC-3")]
fn tc001_external_strings_are_bounded_before_retention() {
    let repository = id("agent-ix/bounded", RepositoryId::new);
    let repository_pin = exact(repository.clone(), "bounded-repository");
    let refusal = RepositoryBinding::new(repository_pin, "too-long", 1)
        .expect_err("remote must be bounded before retention");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.selection_identity(), Some("agent-ix/bounded"));

    let package = id("bounded-package", PackageId::new);
    let package_pin = exact(package.clone(), "bounded-package-content");
    let repository_revision = exact_revision("bounded-repository");
    let refusal = PackageBinding::new(
        package_pin,
        "too-long",
        repository.clone(),
        &repository_revision,
        &["default"],
        1,
        1,
    )
    .expect_err("version must be bounded before retention");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.selection_identity(), Some("bounded-package"));

    for version in [
        "latest",
        "current",
        "*",
        "^1",
        ">=1",
        "1.0.0-01",
        "1.0.0-a..b",
        "1.0.0-a.",
    ] {
        let refusal = PackageBinding::new(
            exact(package.clone(), "bounded-package-content"),
            version,
            repository.clone(),
            &repository_revision,
            &["default"],
            1,
            ID_LIMIT,
        )
        .expect_err("moving or ranged package version must refuse");
        assert_eq!(refusal.code(), RefusalCode::MovingSelection);
        assert_eq!(refusal.selection_identity(), Some("bounded-package"));

        let schema: serde_json::Value =
            serde_json::from_slice(quire_integration::ecosystem_lock::SCHEMA_BYTES).unwrap();
        let validator = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&schema)
            .unwrap();
        let baseline = fixture();
        let lock = validate(&baseline, limits(&baseline)).unwrap();
        let mut document: serde_json::Value = serde_json::from_slice(lock.bytes()).unwrap();
        document["packages"][0]["version"] = version.into();
        assert!(
            !validator.is_valid(&document),
            "schema accepted non-exact package version {version}"
        );
    }

    let refusal = DependencyEdge::new(
        package.clone(),
        id("bounded-target", PackageId::new),
        "too-long",
        Reachability::Reachable,
        Compatibility::Compatible,
        1,
    )
    .expect_err("contract must be bounded before retention");
    assert_eq!(refusal.code(), RefusalCode::ResourceExhausted);
    assert_eq!(refusal.selection_identity(), Some("bounded-package"));
}
