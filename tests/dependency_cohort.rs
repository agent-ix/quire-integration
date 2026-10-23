// SPDX-License-Identifier: AGPL-3.0-or-later
//! The composed lock has one owner for each shared type family and canonical encoder.

use std::{collections::BTreeSet, path::Path, process::Command};

use serde_json::Value;

#[test]
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
    let sources = |name: &str| -> BTreeSet<&str> {
        packages
            .iter()
            .filter(|package| package["name"].as_str() == Some(name))
            .filter_map(|package| package["source"].as_str())
            .collect()
    };

    for name in [
        "quire-semantic-value",
        "quire-contract-model",
        "quire-contract-ir",
        "quire-contract-codegen",
        "quire-contract-runtime",
        "quire-spec-language",
        "qsl-foundation",
        "qsl-replay",
        "quire-verification-contracts",
    ] {
        assert_eq!(sources(name).len(), 1, "{name} must have one source");
    }

    for repository in ["filament-core-data", "quire-rs"] {
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
