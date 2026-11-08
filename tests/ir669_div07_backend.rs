// SPDX-License-Identifier: AGPL-3.0-or-later
//! The finite backend retains inferred domains and the typed integer-modulus renderer refusal.

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
#[trace("TC-097", "IT-017-SC-04", "FR-147-AC-3")]
pub(crate) fn inferred_modulus_domains_retain_renderer_refusal_without_execution() {
    let declaration = "function op using v(a: Int[0, 9], b: Int[1, 3]): Integer pure { a mod b }";
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n{declaration}\n"
    );
    let driven = drive(
        &QslCg,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "ir-669-div07"),
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
    let quire_contract_codegen::Disposition::Unsupported { cause } = &driven.settlements[0].backend.as_ref().expect("backend settled").disposition else {
        panic!("finite inferred domains retain the current renderer refusal: {:?}", driven.settlements);
    };
    let quire_contract_codegen::Cause::KaniFormRefused { form, refusal, backend } = cause.as_ref() else {
        panic!("native Kani renderer cause: {cause:?}");
    };
    assert_eq!(backend, "kani");
    assert_eq!(form.operation_identity.as_deref(), Some("quire.op.integer.mod"));
    let quire_contract_codegen::UnsupportedObligation::OperationNotRendered { operation_identity, derived_domains } = refusal else {
        panic!("native operation refusal: {refusal:?}");
    };
    assert_eq!(operation_identity, "quire.op.integer.mod");
    assert_eq!(derived_domains.len(), 2);
    for (domain, (expected_lower, expected_upper)) in derived_domains.iter().zip([("0", "9"), ("1", "3")]) {
        let quire_contract_codegen::DerivedDomain::IntegerRange { bound, lower, upper } = domain else {
            panic!("inferred original finite integer domain: {domain:?}");
        };
        assert!(driven.checked().term(bound).is_some(), "original inferred bound belongs to the admitted package");
        assert_eq!((lower.as_str(), upper.as_str()), (expected_lower, expected_upper));
    }
    assert!(driven.scalar_executions().is_empty(), "result bound");
    assert!(driven.generation().items.is_empty(), "result bound");
}
