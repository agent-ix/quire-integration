// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-002: independent, canonical, exact-producer execution bundles.

use std::cell::Cell;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_integration::ecosystem_lock::{
    validate, CapabilityBinding, CapabilitySelection, ComponentBinding, Limits, LockSelection,
    PackageBinding, Pin, PinState, RepositoryBinding, RoleBinding, WorkflowBinding,
};
use quire_integration::execution_bundle::{
    BundleDisposition, BundleErrorCode, Coordinator, Domain, ExecutionBundle, ExecutionLimits,
    ExecutionProducer, ExecutionRequest, ItemKey, ItemRequest, NoResultCause, OwnerResult,
    ResultAxes, SharedPrerequisites, StrictReadContext, StrictReader,
};
use quire_integration::{
    ArtifactId, Budget, BuildId, ComponentId, ContractId, DefinitionId, DeploymentId, Disposition,
    Meter, PackageId, ProviderId, RepositoryId, RoleId, ToolchainId, WorkflowInstanceId,
};
use sha2::{Digest as _, Sha256};

const ID_LIMIT: u64 = 128;

trait FixtureIdentity: Sized {
    fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self>;
}

macro_rules! fixture_identities {
    ($($identity:ty),+ $(,)?) => {
        $(
            impl FixtureIdentity for $identity {
                fn fixture_new(value: &str, maximum: u64) -> quire_integration::Result<Self> {
                    Self::new(value, maximum)
                }
            }
        )+
    };
}

fixture_identities!(
    ArtifactId,
    BuildId,
    ComponentId,
    ContractId,
    DefinitionId,
    DeploymentId,
    PackageId,
    ProviderId,
    RepositoryId,
    RoleId,
    ToolchainId,
    WorkflowInstanceId,
);

fn id<T: FixtureIdentity>(value: &str) -> T {
    T::fixture_new(value, ID_LIMIT).expect("bounded fixture identity")
}

fn revision(seed: u8) -> String {
    format!("git:{}", char::from(seed).to_string().repeat(40))
}

fn exact<T: quire_integration::ecosystem_lock::PinIdentity>(identity: T, seed: u8) -> Pin<T> {
    Pin::new(identity, &revision(seed), PinState::Exact, ID_LIMIT).expect("exact fixture pin")
}

fn capability() -> CapabilitySelection {
    CapabilitySelection::new(
        id::<RoleId>("execution-owner"),
        id::<ProviderId>("local"),
        id::<BuildId>("owner-build"),
        id::<ContractId>("quire.execution.owner/v1"),
        id::<ArtifactId>("owner-options:default"),
    )
}

fn validated_lock() -> quire_integration::ecosystem_lock::ValidatedLock {
    let repository = id::<RepositoryId>("agent-ix/owner");
    let package = id::<PackageId>("owner");
    let component = id::<ComponentId>("owner");
    let definitions = [
        "config-version",
        "timed-refund",
        "compensated-choreography",
        "state:config-version",
        "temporal:config-version",
        "observation:config-version",
        "protocol:config-version",
        "analysis:config-version",
        "backend:config-version",
        "state:refund",
        "temporal:refund",
        "observation:refund",
        "protocol:refund",
        "analysis:refund",
        "backend:refund",
        "state:choreography",
        "temporal:choreography",
        "observation:choreography",
        "protocol:choreography",
        "analysis:choreography",
        "backend:choreography",
        "left",
        "right",
        "state:pending",
        "state:bounded",
        "temporal:one-over",
        "state:reader",
        "temporal:reader",
    ]
    .into_iter()
    .map(|name| exact(id::<DefinitionId>(name), b'c'))
    .collect();
    let selection = LockSelection::new(
        vec![RepositoryBinding::new(
            exact(repository.clone(), b'a'),
            "https://github.com/agent-ix/owner",
            ID_LIMIT,
        )
        .expect("repository")],
        vec![
            PackageBinding::new(
                exact(package.clone(), b'b'),
                "1.0.0",
                repository.clone(),
                &revision(b'a'),
                &["default"],
                1,
                ID_LIMIT,
            )
            .expect("package"),
            PackageBinding::new(
                exact(id::<PackageId>("other"), b'b'),
                "1.0.0",
                repository.clone(),
                &revision(b'a'),
                &["default"],
                1,
                ID_LIMIT,
            )
            .expect("other package"),
        ],
        definitions,
        vec![exact(id::<ProviderId>("local"), b'd')],
        vec![exact(id::<ToolchainId>("rust-1.98.1"), b'e')],
        vec![exact(id::<BuildId>("owner-build"), b'f')],
        vec![exact(id::<DeploymentId>("local-test"), b'1')],
        vec![ComponentBinding::new(
            exact(component.clone(), b'2'),
            repository,
            package,
        )],
        vec![RoleBinding::new(
            exact(id::<RoleId>("execution-owner"), b'3'),
            component,
        )],
        vec![CapabilityBinding::new(capability())],
        vec![
            WorkflowBinding::new(
                exact(id::<WorkflowInstanceId>("config-version:fixture"), b'4'),
                id::<DefinitionId>("config-version"),
            ),
            WorkflowBinding::new(
                exact(id::<WorkflowInstanceId>("refund:fixture"), b'4'),
                id::<DefinitionId>("timed-refund"),
            ),
            WorkflowBinding::new(
                exact(id::<WorkflowInstanceId>("choreography:fixture"), b'4'),
                id::<DefinitionId>("compensated-choreography"),
            ),
        ],
        vec![],
    );
    let limits = Limits::for_selection(&selection, 64 * 1024, ID_LIMIT).expect("lock limits");
    validate(&selection, limits).expect("validated exact lock")
}

fn key(domain: Domain, name: &str) -> ItemKey {
    ItemKey::new(domain, id::<DefinitionId>(name))
}

fn item(domain: Domain, name: &str) -> ItemRequest {
    ItemRequest::new(
        key(domain, name),
        capability(),
        id::<ArtifactId>(&format!("artifact:{name}")),
        prerequisites(),
    )
}

fn request(items: Vec<ItemRequest>) -> ExecutionRequest {
    request_for("refund:fixture", items)
}

fn request_for(workflow: &str, items: Vec<ItemRequest>) -> ExecutionRequest {
    request_for_package("owner", workflow, items)
}

fn request_for_package(package: &str, workflow: &str, items: Vec<ItemRequest>) -> ExecutionRequest {
    ExecutionRequest::new(
        id::<PackageId>(package),
        id::<WorkflowInstanceId>(workflow),
        items,
        16,
    )
    .expect("bounded unique request")
}

fn prerequisites() -> SharedPrerequisites {
    SharedPrerequisites::new(
        id::<ArtifactId>("model:fixture"),
        id::<ArtifactId>("package:fixture"),
        id::<ArtifactId>("clock:fixture"),
        id::<ArtifactId>("population:fixture"),
    )
}

fn limits(max_items: u64, max_result_bytes: u64, max_output_bytes: u64) -> ExecutionLimits {
    limits_with_artifact(max_items, 256, max_result_bytes, max_output_bytes)
}

fn limits_with_artifact(
    max_items: u64,
    max_artifact_bytes: u64,
    max_result_bytes: u64,
    max_output_bytes: u64,
) -> ExecutionLimits {
    ExecutionLimits::new(
        max_items,
        max_artifact_bytes,
        max_result_bytes,
        max_output_bytes,
    )
}

fn meter(invocations: u64) -> Meter {
    Meter::new(Budget::new(4096, 32, 0, 32, invocations, 4096, 4096))
}

fn finish_result(
    coordinator: Coordinator<'_>,
) -> quire_integration::execution_bundle::Result<ExecutionBundle> {
    match coordinator.finish()? {
        quire_integration::execution_bundle::CompositionOutcome::Bundle(bundle) => Ok(*bundle),
        quire_integration::execution_bundle::CompositionOutcome::Refused(_) => {
            panic!("non-contradictory fixture unexpectedly refused composition")
        }
    }
}

fn finish_bundle(coordinator: Coordinator<'_>) -> ExecutionBundle {
    finish_result(coordinator).expect("non-contradictory fixture emits a bundle")
}

fn no_result_cause(disposition: Disposition) -> NoResultCause {
    match disposition {
        Disposition::Missing => NoResultCause::MissingInput,
        Disposition::Incompatible => NoResultCause::IncompatibleSelection,
        Disposition::Unsupported => NoResultCause::UnsupportedCapability,
        Disposition::Stale => NoResultCause::StaleEvidence,
        Disposition::Incomplete => NoResultCause::IncompleteEvidence,
        Disposition::Refused => NoResultCause::ReaderRefused,
        Disposition::Failed => NoResultCause::ProducerFailed,
        Disposition::Unexecuted => NoResultCause::NotInvoked,
        Disposition::Completed => panic!("completed results have owner evidence"),
    }
}

fn axes(label: &str) -> ResultAxes {
    ResultAxes::new(
        &format!("truth:{label}"),
        &format!("execution:{label}"),
        &format!("adequacy:{label}"),
        &format!("completeness:{label}"),
        id::<ArtifactId>(&format!("provenance:{label}")),
        ID_LIMIT,
    )
    .expect("bounded independent axes")
}

struct Reader {
    calls: Cell<u64>,
    reject: bool,
    selection: CapabilitySelection,
    artifact: ArtifactId,
}

impl StrictReader for Reader {
    type Checked = Vec<u8>;

    fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    fn read(
        &self,
        context: StrictReadContext<'_>,
        artifact: &[u8],
        meter: &mut Meter,
    ) -> Result<Self::Checked, quire_integration::execution_bundle::ReaderRefusal> {
        assert_eq!(context.artifact(), &self.artifact);
        assert_eq!(context.admission().contract(), "quire.execution.owner/v1");
        self.calls.set(self.calls.get() + 1);
        if self.reject {
            return Err(
                quire_integration::execution_bundle::ReaderRefusal::malformed(
                    "fixture strict-reader refusal",
                ),
            );
        }
        meter
            .consume_input_bytes(u64::try_from(artifact.len()).expect("fixture length"))
            .map_err(quire_integration::execution_bundle::ReaderRefusal::resource)?;
        Ok(artifact.to_vec())
    }
}

struct Producer {
    calls: Cell<u64>,
    disposition: Disposition,
    label: &'static str,
    selection: CapabilitySelection,
}

impl ExecutionProducer<Vec<u8>> for Producer {
    fn selection(&self) -> &CapabilitySelection {
        &self.selection
    }

    fn evaluate(
        &self,
        admission: quire_integration::ecosystem_lock::AdmittedCapability<'_>,
        checked: &Vec<u8>,
        meter: &mut Meter,
        limits: ExecutionLimits,
    ) -> Result<OwnerResult, quire_integration::execution_bundle::ProducerFailure> {
        assert_eq!(admission.contract(), "quire.execution.owner/v1");
        assert_eq!(admission.package(), "owner");
        assert_eq!(admission.features(), &["default"]);
        self.calls.set(self.calls.get() + 1);
        meter
            .consume_duration_millis(1)
            .map_err(quire_integration::execution_bundle::ProducerFailure::resource)?;
        OwnerResult::new(
            self.disposition,
            checked,
            axes(self.label),
            limits.max_result_bytes(),
        )
        .map_err(quire_integration::execution_bundle::ProducerFailure::boundary)
    }
}

fn execute(
    coordinator: &mut Coordinator<'_>,
    item: &ItemRequest,
    disposition: Disposition,
    label: &'static str,
) {
    let reader = Reader {
        calls: Cell::new(0),
        reject: false,
        selection: capability(),
        artifact: item.artifact().clone(),
    };
    let producer = Producer {
        calls: Cell::new(0),
        disposition,
        label,
        selection: capability(),
    };
    coordinator
        .execute(
            item.key(),
            item.artifact(),
            label.as_bytes(),
            &reader,
            &producer,
            &mut meter(1),
        )
        .expect("selected item executes");
    assert_eq!(reader.calls.get(), 1);
    assert_eq!(producer.calls.get(), 1);
}

#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
fn tc002_requested_items_execute_once_and_completion_order_cannot_change_bytes() {
    let lock = validated_lock();
    let build = |workflow: &str, items: &[ItemRequest], order: &[usize]| {
        let mut coordinator = Coordinator::new(
            &lock,
            request_for(workflow, items.to_vec()),
            prerequisites(),
            limits(6, 128, 64 * 1024),
        )
        .expect("coordinator");
        for index in order {
            execute(
                &mut coordinator,
                &items[*index],
                Disposition::Completed,
                [
                    "state",
                    "temporal",
                    "observation",
                    "protocol",
                    "analysis",
                    "backend",
                ][*index],
            );
        }
        finish_bundle(coordinator)
    };

    for (workflow, semantic_name) in [
        ("config-version:fixture", "config-version"),
        ("refund:fixture", "refund"),
        ("choreography:fixture", "choreography"),
    ] {
        let items = [
            item(Domain::State, &format!("state:{semantic_name}")),
            item(Domain::Temporal, &format!("temporal:{semantic_name}")),
            item(Domain::Observation, &format!("observation:{semantic_name}")),
            item(Domain::Protocol, &format!("protocol:{semantic_name}")),
            item(Domain::Analysis, &format!("analysis:{semantic_name}")),
            item(Domain::Backend, &format!("backend:{semantic_name}")),
        ];
        let canonical = build(workflow, &items, &[0, 1, 2, 3, 4, 5]);
        let permuted = build(workflow, &items, &[5, 2, 0, 4, 1, 3]);
        assert_eq!(canonical.bytes(), permuted.bytes());
        assert_eq!(canonical.items().len(), 6);
        assert_eq!(canonical.disposition(), BundleDisposition::Completed);
        assert_eq!(canonical.items()[0].key().domain(), Domain::State);
        assert_eq!(canonical.items()[1].key().domain(), Domain::Temporal);
        assert_eq!(canonical.items()[2].key().domain(), Domain::Observation);
        assert_eq!(canonical.items()[3].key().domain(), Domain::Protocol);
        assert_eq!(canonical.items()[4].key().domain(), Domain::Analysis);
        assert_eq!(canonical.items()[5].key().domain(), Domain::Backend);
    }
}

proptest! {
    #[test]
    #[trace("TC-002", "FR-002-AC-2")]
    fn tc002_bundle_disposition_is_total_and_does_not_relabel_siblings(
        left in 0_u8..9,
        right in 0_u8..9,
    ) {
        let states = [
            Disposition::Completed,
            Disposition::Missing,
            Disposition::Incompatible,
            Disposition::Unsupported,
            Disposition::Stale,
            Disposition::Incomplete,
            Disposition::Unexecuted,
            Disposition::Refused,
            Disposition::Failed,
        ];
        let dispositions = [states[usize::from(left)], states[usize::from(right)]];
        let items = vec![item(Domain::State, "left"), item(Domain::Temporal, "right")];
        let lock = validated_lock();
        let mut coordinator = Coordinator::new(
            &lock,
            request(items.clone()),
            prerequisites(),
            limits(2, 128, 64 * 1024),
        ).unwrap();
        for (item, disposition) in items.iter().zip(dispositions) {
            if disposition == Disposition::Completed
                || disposition == Disposition::Refused
                || disposition == Disposition::Failed
            {
                execute(&mut coordinator, item, disposition, "owner");
            } else {
                coordinator.mark_unavailable(
                    item.key(),
                    disposition,
                    no_result_cause(disposition),
                ).unwrap();
            }
        }
        let bundle = finish_bundle(coordinator);
        let expected = if dispositions.contains(&Disposition::Failed) {
            BundleDisposition::Failed
        } else if dispositions.contains(&Disposition::Refused) {
            BundleDisposition::Refused
        } else if dispositions.iter().all(|value| *value == Disposition::Completed) {
            BundleDisposition::Completed
        } else {
            BundleDisposition::Incomplete
        };
        prop_assert_eq!(bundle.disposition(), expected);
        prop_assert_eq!(bundle.items()[0].disposition(), dispositions[0]);
        prop_assert_eq!(bundle.items()[1].disposition(), dispositions[1]);
    }
}

#[test]
#[trace("TC-002", "FR-002-AC-2")]
fn tc002_owner_bytes_and_every_independent_axis_are_preserved() {
    let selected = item(Domain::Observation, "observation:refund");
    let lock = validated_lock();
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    execute(
        &mut coordinator,
        &selected,
        Disposition::Incomplete,
        "axis-fixture",
    );
    let bundle = finish_bundle(coordinator);
    let retained = &bundle.items()[0];
    assert_eq!(retained.owner_bytes(), Some(b"axis-fixture".as_slice()));
    let retained_axes = retained.axes().expect("owner axes retained");
    assert_eq!(retained_axes.truth(), "truth:axis-fixture");
    assert_eq!(retained_axes.execution(), "execution:axis-fixture");
    assert_eq!(retained_axes.adequacy(), "adequacy:axis-fixture");
    assert_eq!(retained_axes.completeness(), "completeness:axis-fixture");
    assert_eq!(
        retained_axes.provenance().as_str(),
        "provenance:axis-fixture"
    );
}

#[test]
#[trace("TC-002", "FR-002-AC-2")]
fn tc002_post_invocation_resource_failure_remains_an_executed_failure() {
    let selected = item(Domain::Analysis, "analysis:refund");
    let lock = validated_lock();
    let reader = Reader {
        calls: Cell::new(0),
        reject: false,
        selection: capability(),
        artifact: selected.artifact().clone(),
    };
    let producer = Producer {
        calls: Cell::new(0),
        disposition: Disposition::Completed,
        label: "exhausted",
        selection: capability(),
    };
    let mut constrained = Meter::new(Budget::new(4096, 32, 0, 32, 1, 4096, 0));
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            selected.key(),
            selected.artifact(),
            b"checked",
            &reader,
            &producer,
            &mut constrained,
        )
        .unwrap();
    assert_eq!(reader.calls.get(), 1);
    assert_eq!(producer.calls.get(), 1);
    let bundle = finish_bundle(coordinator);
    assert_eq!(bundle.items()[0].disposition(), Disposition::Failed);
    assert_eq!(
        bundle.items()[0].cause(),
        Some(NoResultCause::ProducerResourceExhausted)
    );
}

#[test]
#[trace("TC-002", "FR-002-AC-4")]
fn tc002_shared_contradiction_names_exact_affected_items_and_keeps_other_evidence() {
    let unaffected = item(Domain::Analysis, "analysis:refund");
    let mut affected = item(Domain::Backend, "backend:refund");
    let mut also_affected = item(Domain::Protocol, "protocol:refund");
    affected.set_prerequisites(SharedPrerequisites::new(
        prerequisites().model().clone(),
        id::<ArtifactId>("package:contradictory"),
        prerequisites().clock().clone(),
        prerequisites().population().clone(),
    ));
    also_affected.set_prerequisites(affected.prerequisites().clone());
    let lock = validated_lock();
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![
            affected.clone(),
            also_affected.clone(),
            unaffected.clone(),
        ]),
        prerequisites(),
        limits(3, 128, 64 * 1024),
    )
    .unwrap();
    execute(
        &mut coordinator,
        &unaffected,
        Disposition::Completed,
        "unaffected",
    );
    let reader = Reader {
        calls: Cell::new(0),
        reject: false,
        selection: capability(),
        artifact: affected.artifact().clone(),
    };
    let producer = Producer {
        calls: Cell::new(0),
        disposition: Disposition::Completed,
        label: "must-not-run",
        selection: capability(),
    };
    coordinator
        .execute(
            affected.key(),
            affected.artifact(),
            b"contradictory",
            &reader,
            &producer,
            &mut meter(1),
        )
        .expect("contradiction is detected before dispatch");
    assert_eq!(reader.calls.get(), 0);
    assert_eq!(producer.calls.get(), 0);
    coordinator
        .mark_unavailable(
            also_affected.key(),
            Disposition::Missing,
            NoResultCause::MissingInput,
        )
        .expect("preexisting affected result is accepted before finalization");
    let refusal = coordinator
        .finish()
        .expect("terminalization is total")
        .into_refusal()
        .expect("ordinary finalization returns attributable refusal");
    assert_eq!(
        refusal.affected(),
        &[also_affected.key().clone(), affected.key().clone()]
    );
    assert_eq!(refusal.items().len(), 3);
    let preserved = refusal
        .items()
        .iter()
        .find(|item| item.key() == unaffected.key())
        .expect("unaffected evidence retained");
    assert_eq!(preserved.disposition(), Disposition::Completed);
    assert_eq!(preserved.owner_bytes(), Some(b"unaffected".as_slice()));
    let contradicted = refusal
        .items()
        .iter()
        .find(|item| item.key() == affected.key())
        .expect("affected result retained");
    assert_eq!(contradicted.disposition(), Disposition::Refused);
    assert_eq!(
        contradicted.cause(),
        Some(NoResultCause::PrerequisiteContradiction)
    );
    let also_contradicted = refusal
        .items()
        .iter()
        .find(|item| item.key() == also_affected.key())
        .expect("pre-marked affected result retained");
    assert_eq!(also_contradicted.disposition(), Disposition::Refused);
    assert_eq!(
        also_contradicted.cause(),
        Some(NoResultCause::PrerequisiteContradiction)
    );
}

#[test]
#[trace("TC-002", "FR-002-AC-5")]
fn tc002_unknown_capability_and_strict_reader_refusal_never_invoke_a_producer() {
    let selected = item(Domain::Temporal, "temporal:refund");
    let lock = validated_lock();
    let mut wrong = selected.clone();
    wrong.set_capability(CapabilitySelection::new(
        id::<RoleId>("execution-owner"),
        id::<ProviderId>("foreign"),
        id::<BuildId>("owner-build"),
        id::<ContractId>("quire.execution.owner/v1"),
        id::<ArtifactId>("owner-options:default"),
    ));
    let reader = Reader {
        calls: Cell::new(0),
        reject: false,
        selection: capability(),
        artifact: wrong.artifact().clone(),
    };
    let producer = Producer {
        calls: Cell::new(0),
        disposition: Disposition::Completed,
        label: "must-not-run",
        selection: capability(),
    };
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![wrong.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            wrong.key(),
            wrong.artifact(),
            b"owner",
            &reader,
            &producer,
            &mut meter(1),
        )
        .expect("unknown capability becomes explicit item outcome");
    assert_eq!(reader.calls.get(), 0);
    assert_eq!(producer.calls.get(), 0);
    assert_eq!(
        finish_bundle(coordinator).items()[0].disposition(),
        Disposition::Unsupported
    );

    let foreign_producer = Producer {
        calls: Cell::new(0),
        disposition: Disposition::Completed,
        label: "must-not-run",
        selection: CapabilitySelection::new(
            id::<RoleId>("execution-owner"),
            id::<ProviderId>("foreign"),
            id::<BuildId>("owner-build"),
            id::<ContractId>("quire.execution.owner/v1"),
            id::<ArtifactId>("owner-options:default"),
        ),
    };
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            selected.key(),
            selected.artifact(),
            b"owner",
            &reader,
            &foreign_producer,
            &mut meter(1),
        )
        .expect("foreign producer becomes explicit unsupported outcome");
    assert_eq!(reader.calls.get(), 0);
    assert_eq!(foreign_producer.calls.get(), 0);
    assert_eq!(
        finish_bundle(coordinator).items()[0].disposition(),
        Disposition::Unsupported
    );

    let foreign_reader = Reader {
        calls: Cell::new(0),
        reject: false,
        selection: CapabilitySelection::new(
            id::<RoleId>("execution-owner"),
            id::<ProviderId>("foreign"),
            id::<BuildId>("owner-build"),
            id::<ContractId>("quire.execution.owner/v1"),
            id::<ArtifactId>("owner-options:default"),
        ),
        artifact: selected.artifact().clone(),
    };
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            selected.key(),
            selected.artifact(),
            b"owner",
            &foreign_reader,
            &producer,
            &mut meter(1),
        )
        .expect("foreign strict reader becomes explicit unsupported outcome");
    assert_eq!(foreign_reader.calls.get(), 0);
    assert_eq!(producer.calls.get(), 0);

    let mut coordinator = Coordinator::new(
        &lock,
        request_for_package("other", "refund:fixture", vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            selected.key(),
            selected.artifact(),
            b"owner",
            &reader,
            &producer,
            &mut meter(1),
        )
        .expect("cross-package capability becomes explicit incompatible outcome");
    assert_eq!(reader.calls.get(), 0);
    assert_eq!(producer.calls.get(), 0);
    let cross_package = finish_bundle(coordinator);
    assert_eq!(
        cross_package.items()[0].disposition(),
        Disposition::Incompatible
    );
    assert_eq!(
        cross_package.items()[0].cause(),
        Some(NoResultCause::IncompatibleSelection)
    );
    quire_integration::execution_bundle::read(cross_package.bytes(), limits(1, 128, 64 * 1024))
        .expect("cross-package outcome remains self-readable");

    let rejecting_reader = Reader {
        calls: Cell::new(0),
        reject: true,
        selection: capability(),
        artifact: selected.artifact().clone(),
    };
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .execute(
            selected.key(),
            selected.artifact(),
            b"malformed",
            &rejecting_reader,
            &producer,
            &mut meter(1),
        )
        .expect("reader refusal becomes explicit item outcome");
    assert_eq!(rejecting_reader.calls.get(), 1);
    assert_eq!(producer.calls.get(), 0);
    assert_eq!(
        finish_bundle(coordinator).items()[0].disposition(),
        Disposition::Refused
    );
}

#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-5")]
fn tc002_missing_and_duplicate_completions_are_explicit_without_fallback() {
    let selected = item(Domain::Protocol, "protocol:refund");
    let lock = validated_lock();
    let mut coordinator = Coordinator::new(
        &lock,
        request(vec![selected.clone()]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    coordinator
        .mark_unavailable(
            selected.key(),
            Disposition::Unsupported,
            NoResultCause::UnsupportedCapability,
        )
        .unwrap();
    let duplicate = coordinator
        .mark_unavailable(
            selected.key(),
            Disposition::Unsupported,
            NoResultCause::UnsupportedCapability,
        )
        .expect_err("second completion refuses");
    assert_eq!(duplicate.code(), BundleErrorCode::DuplicateResult);

    let pending = item(Domain::State, "state:pending");
    let coordinator = Coordinator::new(
        &lock,
        request(vec![pending]),
        prerequisites(),
        limits(1, 128, 64 * 1024),
    )
    .unwrap();
    let bundle = finish_bundle(coordinator);
    assert_eq!(bundle.items()[0].disposition(), Disposition::Unexecuted);
    assert_eq!(bundle.items()[0].cause(), Some(NoResultCause::NotInvoked));
}

#[test]
#[trace("TC-002", "FR-002-AC-1")]
fn tc002_item_result_and_output_bounds_admit_exact_and_refuse_one_over() {
    let selected = item(Domain::State, "state:bounded");
    let lock = validated_lock();
    let item_count_refusal = ExecutionRequest::new(
        id::<PackageId>("owner"),
        id::<WorkflowInstanceId>("refund:fixture"),
        vec![
            selected.clone(),
            item(Domain::Temporal, "temporal:one-over"),
        ],
        1,
    )
    .expect_err("one-over request count refuses before canonical retention");
    assert_eq!(
        item_count_refusal.code(),
        BundleErrorCode::ResourceExhausted
    );
    let foreign_request = ExecutionRequest::new(
        id::<PackageId>("foreign"),
        id::<WorkflowInstanceId>("refund:fixture"),
        vec![selected.clone()],
        1,
    )
    .unwrap();
    let foreign_refusal = match Coordinator::new(
        &lock,
        foreign_request,
        prerequisites(),
        limits(1, 4, 64 * 1024),
    ) {
        Ok(_) => panic!("package absent from lock must refuse before dispatch"),
        Err(refusal) => refusal,
    };
    assert_eq!(foreign_refusal.code(), BundleErrorCode::UnknownItem);
    let unlocked_request = request(vec![item(Domain::State, "unlocked-definition")]);
    let unlocked_refusal = match Coordinator::new(
        &lock,
        unlocked_request,
        prerequisites(),
        limits(1, 4, 64 * 1024),
    ) {
        Ok(_) => panic!("definition absent from lock must refuse before dispatch"),
        Err(refusal) => refusal,
    };
    assert_eq!(unlocked_refusal.code(), BundleErrorCode::UnknownItem);
    let build = |output_ceiling| {
        let reader = Reader {
            calls: Cell::new(0),
            reject: false,
            selection: capability(),
            artifact: selected.artifact().clone(),
        };
        let producer = Producer {
            calls: Cell::new(0),
            disposition: Disposition::Completed,
            label: "1234",
            selection: capability(),
        };
        let mut coordinator = Coordinator::new(
            &lock,
            request(vec![selected.clone()]),
            prerequisites(),
            limits(1, 4, output_ceiling),
        )
        .unwrap();
        coordinator
            .execute(
                selected.key(),
                selected.artifact(),
                b"1234",
                &reader,
                &producer,
                &mut meter(1),
            )
            .expect("exact result byte limit");
        finish_result(coordinator)
    };

    let mut exact_length = 64 * 1024;
    let mut fixed_point = false;
    for _ in 0..8 {
        let bundle = build(exact_length).expect("ceiling admits bounded output");
        let measured = u64::try_from(bundle.bytes().len()).unwrap();
        if measured == exact_length {
            fixed_point = true;
            break;
        }
        exact_length = measured;
    }
    assert!(fixed_point, "canonical length ceiling must converge");
    assert_eq!(
        u64::try_from(build(exact_length).unwrap().bytes().len()).unwrap(),
        exact_length
    );

    let refusal = OwnerResult::new(Disposition::Completed, b"12345", axes("one-over"), 4)
        .expect_err("one-over owner bytes refuse before retention");
    assert_eq!(refusal.code(), BundleErrorCode::ResourceExhausted);

    assert_eq!(
        build(exact_length - 1)
            .expect_err("one-under output refuses")
            .code(),
        BundleErrorCode::ResourceExhausted
    );
}

#[test]
#[trace("TC-002", "FR-002-AC-3")]
fn tc002_bundle_reader_rejects_unknown_fields_and_noncanonical_item_order() {
    let items = vec![
        item(Domain::State, "state:reader"),
        item(Domain::State, "state:refund"),
        item(Domain::Temporal, "temporal:reader"),
    ];
    let lock = validated_lock();
    let mut coordinator = Coordinator::new(
        &lock,
        request(items.clone()),
        prerequisites(),
        limits(3, 128, 64 * 1024),
    )
    .unwrap();
    execute(&mut coordinator, &items[0], Disposition::Completed, "state");
    execute(
        &mut coordinator,
        &items[1],
        Disposition::Completed,
        "state-refund",
    );
    execute(
        &mut coordinator,
        &items[2],
        Disposition::Completed,
        "temporal",
    );
    let bundle = finish_bundle(coordinator);
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(quire_integration::execution_bundle::SCHEMA_BYTES)
        ),
        quire_integration::execution_bundle::SCHEMA_SHA256
    );
    let schema: serde_json::Value =
        serde_json::from_slice(quire_integration::execution_bundle::SCHEMA_BYTES).unwrap();
    let validator = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("pinned execution-bundle schema compiles");
    let emitted: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    assert!(validator.is_valid(&emitted));
    let read = quire_integration::execution_bundle::read(bundle.bytes(), limits(3, 128, 64 * 1024))
        .expect("canonical bytes strict-read");
    assert_eq!(read.bytes(), bundle.bytes());

    let mut cross_domain = emitted.clone();
    let temporal = cross_domain["temporalResults"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    cross_domain["stateResults"]
        .as_array_mut()
        .unwrap()
        .push(temporal);
    assert!(!validator.is_valid(&cross_domain));
    let cross_domain = serde_json::to_vec(&cross_domain).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&cross_domain, limits(3, 128, 64 * 1024))
            .expect_err("cross-domain result placement refuses")
            .code(),
        BundleErrorCode::Malformed
    );

    let mut unknown: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    unknown["unknown"] = serde_json::json!(true);
    let unknown = serde_json::to_vec(&unknown).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&unknown, limits(3, 128, 64 * 1024))
            .expect_err("unknown field refuses")
            .code(),
        BundleErrorCode::Malformed
    );

    let mut oversized_key: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    oversized_key
        .as_object_mut()
        .unwrap()
        .insert("k".repeat(257), serde_json::Value::Null);
    let oversized_key = serde_json::to_vec(&oversized_key).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&oversized_key, limits(3, 128, 64 * 1024))
            .expect_err("oversized unknown key refuses before key allocation")
            .code(),
        BundleErrorCode::ResourceExhausted
    );

    let mut nested_unknown: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    nested_unknown["stateResults"][0]["unknown"] = serde_json::json!(true);
    assert!(!validator.is_valid(&nested_unknown));
    let nested_unknown = serde_json::to_vec(&nested_unknown).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&nested_unknown, limits(3, 128, 64 * 1024))
            .expect_err("nested unknown field refuses")
            .code(),
        BundleErrorCode::Malformed
    );

    let mut oversized_axis: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    oversized_axis["stateResults"][0]["axes"]["truth"] = serde_json::Value::String("x".repeat(257));
    let oversized_axis = serde_json::to_vec(&oversized_axis).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&oversized_axis, limits(3, 128, 64 * 1024))
            .expect_err("string preflight refuses before owned deserialization")
            .code(),
        BundleErrorCode::ResourceExhausted
    );

    let mut contradicted: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    contradicted["request"]["items"][0]["prerequisites"]["package"] =
        serde_json::json!("package:contradictory");
    let contradicted = serde_json::to_vec(&contradicted).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&contradicted, limits(3, 128, 64 * 1024))
            .expect_err("contradictory item prerequisites refuse the bundle")
            .code(),
        BundleErrorCode::PrerequisiteContradiction
    );

    let mut too_many: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    let extra_request = too_many["request"]["items"][0].clone();
    too_many["request"]["items"]
        .as_array_mut()
        .unwrap()
        .push(extra_request);
    let extra_result = too_many["stateResults"][0].clone();
    too_many["stateResults"]
        .as_array_mut()
        .unwrap()
        .push(extra_result);
    let too_many = serde_json::to_vec(&too_many).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&too_many, limits(3, 128, 64 * 1024))
            .expect_err("item preflight refuses before owned deserialization")
            .code(),
        BundleErrorCode::ResourceExhausted
    );

    assert_eq!(
        quire_integration::execution_bundle::read(bundle.bytes(), limits(3, 4, 64 * 1024))
            .expect_err("owner-byte preflight refuses before blob retention")
            .code(),
        BundleErrorCode::ResourceExhausted
    );

    let mut reordered: serde_json::Value = serde_json::from_slice(bundle.bytes()).unwrap();
    reordered["stateResults"].as_array_mut().unwrap().reverse();
    let reordered = serde_json::to_vec(&reordered).unwrap();
    assert_eq!(
        quire_integration::execution_bundle::read(&reordered, limits(3, 128, 64 * 1024))
            .expect_err("noncanonical order refuses")
            .code(),
        BundleErrorCode::CanonicalMismatch
    );

    let compact_item = item(Domain::State, "state:reader");
    let compact_limits = limits_with_artifact(1, 32, 128, 64 * 1024);
    let mut compact = Coordinator::new(
        &lock,
        request(vec![compact_item.clone()]),
        prerequisites(),
        compact_limits,
    )
    .expect("all caller-owned strings fit the compact ceiling");
    execute(
        &mut compact,
        &compact_item,
        Disposition::Completed,
        "compact",
    );
    let compact = finish_bundle(compact);
    assert_eq!(
        quire_integration::execution_bundle::read(compact.bytes(), compact_limits)
            .expect("fixed protocol literals are independent of the artifact ceiling")
            .bytes(),
        compact.bytes()
    );
}
