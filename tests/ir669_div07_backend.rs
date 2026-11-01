// SPDX-License-Identifier: AGPL-3.0-or-later
//! The finite backend retains a typed per-item bound refusal for integer modulus.

use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::Duration;

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::SpineLimits;
use quire_contract_codegen::ProofCeilings;
use quire_driver::{
    build_registry, drive, CompileInput, GenerationInput, KaniInput, ProviderManifest, QslCg,
};
use quire_exact::Cancel;

fn finite_kani_registry() -> quire_driver::AdmittedRegistry {
    build_registry(&[ProviderManifest {
        identity: "kani".to_owned(),
        advertises: vec![(
            Some("value-validity".to_owned()),
            Some("bounded".to_owned()),
        )],
        domains: Some(vec!["integer".to_owned()]),
        bounds: None,
        wire: None,
    }])
    .expect("finite Kani provider registers")
}

#[test]
#[trace("FR-147-AC-3")]
fn missing_division_result_bound_settles_requires_bound_without_execution() {
    let declaration = "function op using v(a: Int[0, 9], b: Int[1, 3]): Integer pure { a mod b }";
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n{declaration}\n"
    );
    let driven = drive(
        &QslCg,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "ir-669-div07", "fixture", "1"),
            path: "ir669-div07.native",
            bytes: source.as_bytes(),
            package_name: "ir-669-div07",
            packages: &BTreeMap::new(),
            limits: SpineLimits::default(),
            cancel: Cancel::new(),
        },
        &finite_kani_registry(),
        None,
        GenerationInput {
            kani: Some(KaniInput {
                ceilings: ProofCeilings {
                    memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive ceiling"),
                    wall_clock: Duration::from_secs(30),
                },
                subject_path: "crate::subject",
                unwind: 1,
            }),
            scalar_execution: None,
        },
    )
    .expect("typed QSL division request reaches negotiation");
    assert_eq!(driven.settlements.len(), 1, "result bound");
    assert!(
        matches!(
            &driven.settlements[0]
                .backend
                .as_ref()
                .expect("backend settled")
                .disposition,
            quire_contract_codegen::Disposition::RequiresBound { .. }
        ),
        "result bound settlement: {:?}",
        driven.settlements
    );
    assert!(driven.scalar_executions().is_empty(), "result bound");
    assert!(driven.generation().items.is_empty(), "result bound");
}
