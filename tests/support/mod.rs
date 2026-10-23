// SPDX-License-Identifier: AGPL-3.0-or-later
//! Locate QSL's authored fixtures from Cargo's resolved dependency graph.

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: PathBuf,
}

pub fn qsl_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let output = std::process::Command::new(env!("CARGO"))
            .args(["metadata", "--format-version", "1", "--manifest-path"])
            .arg(&manifest)
            .output()
            .expect("cargo metadata executes");
        assert!(
            output.status.success(),
            "cargo metadata: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: Metadata =
            serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
        let package = metadata
            .packages
            .into_iter()
            .find(|package| package.name == "quire-spec-language")
            .expect("QSL is in the resolved dependency graph");
        package
            .manifest_path
            .parent()
            .expect("QSL manifest has a parent")
            .to_path_buf()
    })
}
