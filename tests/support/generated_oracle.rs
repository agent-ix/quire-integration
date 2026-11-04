// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native-only execution of actual CG artifacts for IT-019.

use std::{fs, path::Path, process::Command};

use quire_contract_codegen::{Artifact, ArtifactBundle};

/// Compile the actual generated crate plus the integration-owned observation
/// driver. The driver calls its public generated functions; it supplies no
/// replacement implementation of an operation.
pub fn execute(artifacts: &[Artifact], driver: &str) -> Vec<String> {
    let directory = tempfile::tempdir().expect("owned generated crate directory");
    let bundle = ArtifactBundle::new(artifacts.to_vec()).expect("CG artifacts validate");
    for artifact in bundle.artifacts() {
        let destination = directory.path().join(&artifact.path);
        fs::create_dir_all(destination.parent().expect("artifact parent"))
            .expect("create artifact directory");
        fs::write(destination, &artifact.contents).expect("write actual generated artifact");
    }
    assert!(directory.path().join("src/lib.rs").is_file());
    fs::create_dir(directory.path().join("tests")).expect("create native test directory");
    fs::write(directory.path().join("tests/observation.rs"), driver)
        .expect("write integration-owned native observation driver");
    fs::write(
        directory.path().join("tests/oracle_observation.rs"),
        include_str!("oracle_observation.rs"),
    )
    .expect("write the same mechanical observation definition");
    // Ordered charge logs are an owner-supported dev-only observation. The
    // generated production manifest and library retain their own dependencies.
    let manifest_path = directory.path().join("Cargo.toml");
    let mut manifest = fs::read_to_string(&manifest_path).expect("generated manifest");
    manifest.push_str(&format!(
        "\n[dev-dependencies]\nquire-exact = {{ {}, features = [\"test-support\"] }}\n",
        quire_contract_codegen::EXACT_DEPENDENCY_SOURCE,
    ));
    fs::write(manifest_path, manifest).expect("enable the dev-only native observer");
    // Cargo alone maintains the child lock. Starting from the actual consumer
    // lock keeps first-party dependency choices coherent without a second
    // revision record or a rewritten dependency declaration.
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        directory.path().join("Cargo.lock"),
    )
    .expect("supply the real consumer lock to Cargo");
    let output = Command::new("flock")
        .arg("/tmp/titan-cargo.lock")
        .args(["nice", "-n", "10", "ionice", "-c2", "-n7"])
        .arg(env!("CARGO"))
        .args(["test", "-j2", "--quiet", "--manifest-path"])
        .arg(directory.path().join("Cargo.toml"))
        .args([
            "--test",
            "observation",
            "--",
            "--nocapture",
            "--test-threads=1",
            "--format=terse",
        ])
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target"),
        )
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .output()
        .expect("native artifact compilation and execution starts");
    assert!(
        output.status.success(),
        "generated native execution failed: {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .expect("native observation is UTF-8")
        .lines()
        .filter_map(|line| line.strip_prefix("IT019_OBSERVATION ").map(str::to_owned))
        .collect()
}
