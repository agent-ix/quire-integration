// SPDX-License-Identifier: AGPL-3.0-or-later
//! The composed lock has one owner for each shared type family and canonical encoder.

use std::{collections::BTreeSet, path::Path, process::Command};

use ix_trace_rs::trace;
use serde_json::Value;

#[test]
#[trace("IT-014-SC-01")]
fn composed_dependency_cohort_has_one_shared_type_identity() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .expect("cargo metadata executes");
    assert!(
        output.status.success(),
        "locked cargo metadata: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("package list");
    let sources = |name: &str| -> Vec<Option<&str>> {
        packages
            .iter()
            .filter(|package| package["name"].as_str() == Some(name))
            .map(|package| package["source"].as_str())
            .collect()
    };

    for (name, repository) in [
        ("quire-semantic-value", "quire-semantic-value"),
        ("quire-contract-model", "quire-contract-ir"),
        ("quire-contract-ir", "quire-contract-ir"),
        ("quire-contract-codegen", "quire-contract-codegen"),
        ("quire-contract-runtime", "quire-contract-runtime"),
        ("quire-spec-language", "quire-spec-language"),
        ("qsl-foundation", "quire-spec-language"),
        ("qsl-replay", "quire-spec-language"),
        (
            "quire-verification-contracts",
            "quire-verification-contracts",
        ),
    ] {
        let members = sources(name);
        assert_eq!(members.len(), 1, "{name} must have one package identity");
        assert!(
            members[0].is_some_and(|source| {
                source.starts_with(&format!("git+https://github.com/agent-ix/{repository}?"))
            }),
            "{name} must come from {repository}"
        );
    }

    for (repository, expected_names) in [
        (
            "quire-canonical",
            &["quire-canonical", "quire-canonical-derive"][..],
        ),
        (
            "quire-spec-language",
            &["quire-spec-language", "qsl-foundation", "qsl-replay"][..],
        ),
        (
            "quire-contract-ir",
            &["quire-contract-model", "quire-contract-ir"][..],
        ),
        (
            "filament-core-data",
            &[
                "agent-ix-extraction-frontend",
                "agent-ix-semantic-ir",
                "agent-ix-semantic-schema",
            ][..],
        ),
        ("quire-rs", &["quire-rs", "quire-rust-extraction"][..]),
    ] {
        for name in expected_names {
            let members = sources(name);
            assert_eq!(members.len(), 1, "{name} must have one package identity");
            assert!(
                members[0].is_some_and(|source| {
                    source.starts_with(&format!("git+https://github.com/agent-ix/{repository}?"))
                }),
                "{name} must come from {repository}"
            );
        }
        let revisions = packages
            .iter()
            .filter_map(|package| package["source"].as_str())
            .filter(|source| {
                source.starts_with(&format!("git+https://github.com/agent-ix/{repository}?"))
            })
            .filter_map(|source| source.rsplit_once('#').map(|(_, revision)| revision))
            .collect::<BTreeSet<_>>();
        assert_eq!(revisions.len(), 1, "{repository} must have one revision");
    }

    assert!(
        !packages
            .iter()
            .any(|package| package["name"].as_str() == Some("serde_json_canonicalizer")),
        "the canonical encoding dependency must have one authoritative owner"
    );
}
