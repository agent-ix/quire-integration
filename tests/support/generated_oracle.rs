// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native-only execution of actual CG artifacts for IT-019.

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use quire_contract_codegen::{Artifact, ArtifactBundle};

#[path = "cargo_command.rs"]
mod cargo_command;

static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(0);

/// Compile the actual generated crate plus the integration-owned observation
/// driver. The driver calls its public generated functions; it supplies no
/// replacement implementation of an operation.
pub fn execute(artifacts: &[Artifact], driver: &str) -> Vec<String> {
    let directory = tempfile::tempdir().expect("owned generated crate directory");
    // All generated crates share one Cargo target directory. Give each
    // observation its own package and test target identity so Cargo cannot
    // reuse another driver's generated library or observation binary.
    let test_name = format!(
        "observation_{}_{}",
        std::process::id(),
        NEXT_OBSERVATION.fetch_add(1, Ordering::Relaxed)
    );
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
    let crate_name = [
        quire_contract_codegen::EXACT_SCALAR_CRATE_NAME,
        quire_contract_codegen::EXACT_FUNCTION_CRATE_NAME,
        quire_contract_codegen::COMPOSITE_EQUALITY_CRATE_NAME,
    ]
    .into_iter()
    .find(|name| manifest.contains(&format!("name = \"{name}\"")))
    .expect("the artifact has a known generated package name");
    manifest = manifest.replacen(
        &format!("name = \"{crate_name}\""),
        &format!("name = \"{crate_name}-{test_name}\""),
        1,
    );
    assert!(manifest.contains("[lib]\n"), "generated library target");
    manifest = manifest.replacen(
        "[lib]\n",
        &format!("[lib]\nname = \"{}\"\n", crate_name.replace('-', "_")),
        1,
    );
    manifest.push_str(&format!(
        "\n[dev-dependencies]\nquire-exact = {{ {}, features = [\"test-support\"] }}\n",
        quire_contract_codegen::EXACT_DEPENDENCY_SOURCE,
    ));
    manifest.push_str(&format!(
        "\n[[test]]\nname = \"{test_name}\"\npath = \"tests/observation.rs\"\n"
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
    // The outer native test is intentionally launched outside Cargo's lock.
    // Keep its selected compiler when the child manifest lives in a temporary
    // directory; a rustup shim must not reselect a compiler per crate directory.
    let cargo = Path::new(env!("CARGO"));
    let compiler = cargo.with_file_name(format!("rustc{}", std::env::consts::EXE_SUFFIX));
    assert!(
        compiler.is_file(),
        "the selected Cargo toolchain contains rustc"
    );
    let output = cargo_command::command()
        .args(["test", "-j2", "--quiet", "--manifest-path"])
        .arg(directory.path().join("Cargo.toml"))
        .args([
            "--test",
            &test_name,
            "--",
            "--nocapture",
            "--test-threads=1",
            "--format=terse",
        ])
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target"),
        )
        .env("RUSTC", compiler)
        .output()
        .expect("native artifact compilation and execution starts");
    if output.status.success() && !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
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
