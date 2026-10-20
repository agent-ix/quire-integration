// SPDX-License-Identifier: AGPL-3.0-or-later
//! IR-914: a QSL export refusal is withheld while bound siblings reach CG.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::Duration;

use ix_trace_rs::trace;
use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::SourceIdentity;
use qsl_package::{BoundItem, CheckedPackage, ExportRefusal};
use qsl_replay::spine::{CheckedUnit, EmittedUnit, SpineLimits};
use qsl_route::routing::Disposition;
use qsl_route::{Candidate, ManifestDigest};
use qsl_semantics::value::{CatalogRole, DefinitionLock};
use quire_contract_codegen::{
    BackendDescriptor, BackendKind, BackendProviderEnvelope, ContractItemSettlement,
    ContractNegotiationError, ContractRequestPosition, GenerationContexts, RoutedGeneration,
    RoutedGenerationError, RoutedGenerationItem,
};
use quire_contract_ir::{
    CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    ContractPackageV1,
};
use quire_driver::{
    build_registry, drive, CompileInput, DriverRefusal, GenerationInput, KaniInput,
    ProviderManifest, QslCg, Upstream,
};
use quire_exact::Cancel;

struct ExportVector {
    envelope: RefCell<Option<BackendProviderEnvelope>>,
}

impl Upstream for ExportVector {
    fn has_abstraction_relation(&self, _package: &CheckedPackage) -> bool {
        true
    }

    fn export_bindings(
        &self,
        _package: &CheckedPackage,
        items: &[OccurrenceKey],
    ) -> Result<Vec<Result<BoundItem, ExportRefusal>>, ExportRefusal> {
        assert_eq!(items.len(), 3);
        Ok(items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index == 1 {
                    Err(ExportRefusal::Unbound {
                        item: item.clone(),
                        elements: Vec::new(),
                    })
                } else {
                    Ok(BoundItem {
                        item: item.clone(),
                        bindings: Vec::new(),
                    })
                }
            })
            .collect())
    }

    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal> {
        QslCg.compile(source, path, bytes, packages, limits, cancel)
    }

    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        QslCg.read_package(emitted, limits, evidence)
    }

    fn backend_kind(&self, descriptor: &BackendDescriptor) -> Option<BackendKind> {
        QslCg.backend_kind(descriptor)
    }

    fn negotiate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ContractItemSettlement>, Box<ContractNegotiationError>> {
        *self.envelope.borrow_mut() = Some(envelope.clone());
        QslCg.negotiate(package, positions, envelope)
    }

    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>> {
        QslCg.route(dispositions)
    }

    fn generate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        QslCg.generate(package, positions, routed, contexts)
    }
}

#[test]
#[trace("FR-010-AC-4", "FR-001-AC-17", "TC-017")]
fn refused_binding_is_withheld_as_bound_siblings_flow_to_codegen() {
    let lock = DefinitionLock::pinned();
    let root = lock.entry(CatalogRole::Root).identity;
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{root}\";\n\
         function a using v(x: Int[0, 9]): Int[-9, 0] pure {{ -x }}\n\
         function b using v(y: Int[0, 9]): Int[-9, 0] pure {{ -y }}\n\
         function c using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n"
    );
    let registry = build_registry(&[ProviderManifest {
        identity: "kani".to_owned(),
        manifest_digest: ManifestDigest::from_digest([7; 32]),
        advertises: vec![(
            Some("value-validity".to_owned()),
            Some("bounded".to_owned()),
        )],
        domains: Some(vec!["integer".to_owned()]),
        bounds: None,
        wire: None,
    }])
    .expect("the Kani provider registers");
    let export = ExportVector {
        envelope: RefCell::new(None),
    };
    let driven = drive(
        &export,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "ir914-integration", "git", "1"),
            path: "ir914.native",
            bytes: source.as_bytes(),
            packages: &BTreeMap::new(),
            limits: SpineLimits::default(),
            cancel: Cancel::new(),
        },
        &registry,
        None,
        GenerationInput {
            kani: Some(KaniInput {
                ceilings: quire_contract_codegen::ProofCeilings {
                    memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive ceiling"),
                    wall_clock: Duration::from_secs(30),
                },
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("a per-item export refusal does not refuse the run");
    let envelope = export.envelope.borrow();
    assert_eq!(envelope.as_ref().expect("CG negotiated").items.len(), 2);
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item
                .backend
                .as_ref()
                .expect("bound item settles")
                .request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(driven.item_refusals.len(), 1);
    assert_eq!(driven.item_refusals[0].request_index, 1);
    assert_eq!(
        driven.item_refusals[0].occurrence,
        *driven.request_items[1].occurrence()
    );
    assert_eq!(
        driven
            .generation
            .items
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2],
        "both bound siblings generate and the refused item does not"
    );
}
