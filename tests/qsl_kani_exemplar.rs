// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-010-SC-05 / IT-011: real `cargo kani` runs over codegen's Kani bundles
//! for clauses QSL lowered.
//!
//! Each run builds the bundle and a hand-written subject in a throwaway
//! single-crate workspace with no `rust-version`, so `cargo kani` uses its
//! own toolchain.

// Shared with `qsl_codegen_composition`, which uses every item and so keeps
// the dead-code check; this binary uses a subset.
#[allow(dead_code)]
mod support;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use ix_trace_rs::trace;
use qsl_replay::{ProofCategory, TerminalRecord, TerminalValue};
use quire_contract_codegen as codegen;
use quire_contract_ir as ir;
use quire_spec_language::lowering::{lower_for, LoweringLimits, ProjectionTarget};
use serde_json::json;
use support::{bound_clause, runtime_manifest, GeneratedCrate, CONFIG_DOMAIN, RUNTIME_GIT_URL};

/// Run `bundle` plus `subject` under `cargo kani` with the bundle's own
/// options. `runtime_patch` replaces `quire-contract-runtime` with a local
/// copy.
fn execute_kani(
    bundle: &codegen::KaniArtifactBundle,
    subject: &str,
    runtime_patch: Option<&Path>,
) -> Output {
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&bundle.proof_graph.contents).unwrap();
    // quire-contract-runtime's own `#[cfg(kani)]` proof harness imports
    // `crate::exact`, so a `cargo kani` build needs its "exact" feature.
    let generated = GeneratedCrate::new("composed-kani", &["exact"], "");
    if let Some(local) = runtime_patch {
        generated.write(
            "Cargo.toml",
            &format!(
                "{}\n[patch.\"{RUNTIME_GIT_URL}\"]\nquire-contract-runtime = {{ path = {local:?} }}\n",
                runtime_manifest("composed-kani", &["exact"], "")
            ),
        );
    }
    generated.write(
        "src/lib.rs",
        &format!("{}\n{subject}\n", bundle.rust.contents),
    );
    generated.write(
        "build.rs",
        "fn main() { println!(\"cargo:rustc-check-cfg=cfg(kani)\"); }\n",
    );
    Command::new("cargo")
        .arg("kani")
        .args(&graph.options)
        .env("CARGO_TARGET_DIR", generated.target())
        .env("CARGO_NET_OFFLINE", "true")
        .current_dir(generated.path())
        .output()
        .expect("cargo kani executes")
}

fn transcript(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// ---------------------------------------------------------------------------
// IT-010-SC-05: ConfigVersion VersionUnchanged proves for the identity subject
// and its counterexample replays through native execution.
// ---------------------------------------------------------------------------

/// The first `i64` of Kani's printed concrete playback, decoded exactly.
fn playback_i64(output: &str) -> Option<i64> {
    let tail = output
        .split_once("let concrete_vals: Vec<Vec<u8>> = vec![")?
        .1;
    let row = tail.split_once("vec![")?.1.split_once(']')?.0;
    let bytes = row
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some(i64::from_le_bytes(bytes.try_into().ok()?))
}

#[test]
#[trace("IT-010-SC-02", "IT-010-SC-05")]
fn kani_proves_identity_and_counterexample_replays_natively() {
    let models = [support::config_model()];
    let native = support::config_package(&models);
    let projection = lower_for(
        &native,
        ProjectionTarget::StateScalarIrV1,
        LoweringLimits::default(),
    )
    .unwrap();
    let bound = ir::BoundPackage::from_json_bytes(projection.bytes()).unwrap();
    let clause = bound_clause(&bound, "version_unchanged");
    let bundle = support::absent_precondition_kani_bundle(
        clause,
        "version_unchanged_absent_precondition",
        "it-010-config-version",
    );
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&bundle.proof_graph.contents).unwrap();
    assert_eq!(graph.proof_id, "it-010-config-version");
    assert_eq!(graph.requirement_id, "VersionUnchanged");
    assert_eq!(graph.requirement_revision, 1);
    assert!(graph.dependencies.is_empty());
    assert_eq!(graph.subject_arguments.len(), 1);
    assert_eq!(graph.subject_results.len(), 1);
    for binding in graph.subject_arguments.iter().chain(&graph.subject_results) {
        let bounds = binding.integer_bounds.as_ref().unwrap();
        assert_eq!((bounds.minimum, bounds.maximum), (0, 1000));
    }
    let harness = graph.options[5].clone();
    assert_eq!(
        graph.options,
        [
            "-Z",
            "function-contracts",
            "-Z",
            "concrete-playback",
            "--harness",
            &harness,
            "--exact",
            "--unwind",
            "2",
            "--solver",
            "cadical",
            "--output-format",
            "regular",
            "--concrete-playback",
            "print",
        ]
    );

    let healthy = execute_kani(
        &bundle,
        "/// Identity ConfigVersion subject.\npub fn subject(version_pre: i64) -> i64 { version_pre }",
        None,
    );
    assert!(
        healthy.status.success(),
        "identity proof failed:\n{}",
        transcript(&healthy)
    );

    let changed = execute_kani(
        &bundle,
        "/// Always-changed, still in-domain ConfigVersion subject.\npub fn subject(version_pre: i64) -> i64 { if version_pre == 1000 { 999 } else { version_pre + 1 } }",
        None,
    );
    assert!(
        !changed.status.success(),
        "violating subject unexpectedly proved"
    );
    let output = transcript(&changed);
    let counterexample = playback_i64(&output)
        .unwrap_or_else(|| panic!("Kani emitted no decodable concrete playback:\n{output}"));
    assert!(CONFIG_DOMAIN.contains(&counterexample));
    let post = if counterexample == 1000 {
        999
    } else {
        counterexample + 1
    };
    assert!(CONFIG_DOMAIN.contains(&post));
    assert_eq!(
        support::config_native_verdict(&native, &models[0], counterexample, post),
        Some(false)
    );
    assert_eq!(playback_i64("let concrete_vals = vec![];"), None);
}

// ---------------------------------------------------------------------------
// IT-011: the function-application exemplar `amount < 1000 implies amount + 1
// <= 1000` -- its claimed module discharges a SUCCESS check, and mutating the
// connective helper its compiled postcondition calls fails the proof.
// ---------------------------------------------------------------------------

/// The one claimed module for this exemplar's one proposition: the compiled
/// postcondition oracle Kani discharges checks against.
const CLAIMED_MODULES: [&str; 1] = ["PopulationRule@7/population_rule"];

/// `quire_contract_runtime::operators::implies_short_circuit`'s current
/// source, matched before mutating it so an upstream edit fails loudly
/// instead of silently mutating nothing.
const CONNECTIVE_HELPER_HEALTHY: &str = "pub fn implies_short_circuit<R: From<bool>>(antecedent: bool, consequent: impl FnOnce() -> R) -> R {\n    if antecedent {\n        consequent()\n    } else {\n        R::from(true)\n    }\n}";

/// The mutation control: unconditionally false, ignoring both operands.
const CONNECTIVE_HELPER_MUTATED: &str = "pub fn implies_short_circuit<R: From<bool>>(_antecedent: bool, _consequent: impl FnOnce() -> R) -> R {\n    R::from(false)\n}";

/// This clause's ABI derives no subject result: the postcondition is a pure
/// fact over the input.
const HEALTHY_SUBJECT: &str = "pub fn subject(_amount_current: i64) {}";

fn function_application_bundle() -> codegen::KaniArtifactBundle {
    let models = [support::rule_model(&[(
        "amount",
        json!({"kind": "scalar", "name": "Version"}),
    )])];
    let native = support::rule_package(&models, "amount < 1000 implies amount + 1 <= 1000");
    let projection = lower_for(
        &native,
        ProjectionTarget::IntegerIrV1,
        LoweringLimits::default(),
    )
    .unwrap();
    let bound = ir::BoundPackage::from_json_bytes(projection.bytes()).unwrap();
    support::absent_precondition_kani_bundle(
        bound_clause(&bound, "population_rule"),
        "amount_absent_precondition",
        "qsl-326-function-application",
    )
}

/// The SUCCESS-check discharge count from Kani's
/// `** <failed> of <total> failed[ (<n> unreachable)]` summary line.
/// UNREACHABLE and UNDETERMINED checks are excluded.
fn discharged_success_checks(output: &str) -> u64 {
    for line in output.lines() {
        let Some(rest) = line.trim().strip_prefix("** ") else {
            continue;
        };
        let (counts, qualifier) = match rest.split_once(" (") {
            Some((counts, qualifier)) => (counts, qualifier.trim_end_matches(')')),
            None => (rest, ""),
        };
        let Some((failed, total)) = counts
            .strip_suffix(" failed")
            .and_then(|counts| counts.split_once(" of "))
        else {
            continue;
        };
        let (Ok(failed), Ok(total)) = (failed.parse::<u64>(), total.parse::<u64>()) else {
            continue;
        };
        let excluded = if qualifier.contains("unreachable") || qualifier.contains("undetermined") {
            qualifier
                .split_whitespace()
                .next()
                .and_then(|count| count.parse::<u64>().ok())
                .unwrap_or(0)
        } else {
            0
        };
        return total.saturating_sub(failed).saturating_sub(excluded);
    }
    panic!("Kani transcript has no '** <failed> of <total> failed' summary line:\n{output}");
}

fn cargo_home() -> PathBuf {
    env::var_os("CARGO_HOME").map_or_else(
        || PathBuf::from(env::var_os("HOME").expect("HOME is set")).join(".cargo"),
        PathBuf::from,
    )
}

/// Cargo's git checkout of `quire-contract-runtime` at `revision`, which the
/// unmutated Kani run has already fetched.
fn cached_runtime_checkout(revision: &str) -> PathBuf {
    let checkouts = cargo_home().join("git/checkouts");
    let short = &revision[..7];
    fs::read_dir(&checkouts)
        .unwrap_or_else(|error| panic!("cargo git checkouts {checkouts:?}: {error}"))
        .map(|entry| entry.unwrap().path().join(short))
        .find(|candidate| {
            fs::read_to_string(candidate.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.contains("name = \"quire-contract-runtime\""))
        })
        .unwrap_or_else(|| {
            panic!(
                "quire-contract-runtime@{revision} is not in cargo's git checkouts {checkouts:?}"
            )
        })
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// A throwaway copy of `quire-contract-runtime` with
/// `operators::implies_short_circuit` mutated to return `false`. The copy
/// keeps the manifest's features and dependencies and drops only its
/// `[workspace]` table, so it builds as a plain crate.
fn mutated_runtime_copy(revision: &str) -> tempfile::TempDir {
    let source = cached_runtime_checkout(revision);
    let directory = tempfile::tempdir().unwrap();
    let manifest = fs::read_to_string(source.join("Cargo.toml")).unwrap();
    let start = manifest
        .find("[workspace]")
        .expect("quire-contract-runtime's manifest declares [workspace]");
    let end = manifest[start..]
        .find("\n\n")
        .map_or(manifest.len(), |offset| start + offset + 2);
    let manifest = format!("{}{}", &manifest[..start], &manifest[end..]);
    assert!(!manifest.contains("[workspace]"));
    fs::write(directory.path().join("Cargo.toml"), manifest).unwrap();
    copy_tree(&source.join("src"), &directory.path().join("src"));
    // `src/lib.rs` includes its `#[cfg(kani)]` harness from `verification/`.
    copy_tree(
        &source.join("verification"),
        &directory.path().join("verification"),
    );
    let operators = directory.path().join("src/operators.rs");
    let original = fs::read_to_string(&operators).unwrap();
    assert!(
        original.contains(CONNECTIVE_HELPER_HEALTHY),
        "operators::implies_short_circuit no longer matches this mutation control's expected \
         source; update CONNECTIVE_HELPER_HEALTHY/_MUTATED"
    );
    fs::write(
        &operators,
        original.replace(CONNECTIVE_HELPER_HEALTHY, CONNECTIVE_HELPER_MUTATED),
    )
    .unwrap();
    directory
}

#[test]
#[trace("IT-011-SC-01")]
fn healthy_run_discharges_a_success_check_for_every_claimed_module() {
    let bundle = function_application_bundle();
    let healthy = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    let output = transcript(&healthy);
    assert!(
        healthy.status.success() && output.contains("VERIFICATION:- SUCCESSFUL"),
        "function-application proof did not verify:\n{output}"
    );
    let success_checks = u32::try_from(discharged_success_checks(&output)).unwrap();
    for module in CLAIMED_MODULES {
        let record = TerminalRecord::new(module, TerminalValue::Proved { success_checks });
        assert_eq!(
            record.value().category(),
            ProofCategory::Success,
            "claimed module {module}'s discharge count did not read back as success"
        );
        let TerminalValue::Proved {
            success_checks: discharged,
        } = record.value()
        else {
            panic!("record was constructed as Proved")
        };
        assert!(
            discharged > 0,
            "claimed module {module} discharged zero SUCCESS checks"
        );
    }
}

#[test]
#[trace("IT-011-SC-02")]
fn mutating_the_compiled_postconditions_connective_helper_fails_the_proof() {
    let bundle = function_application_bundle();
    assert!(
        bundle
            .rust
            .contents
            .contains("quire_contract_runtime::operators::implies_short_circuit("),
        "the compiled postcondition no longer calls the connective helper this mutation targets"
    );
    let unmutated = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    assert!(
        unmutated.status.success(),
        "control run must verify before mutating:\n{}",
        transcript(&unmutated)
    );
    let mutated_runtime = mutated_runtime_copy(codegen::RUNTIME_REVISION);
    let mutated = execute_kani(&bundle, HEALTHY_SUBJECT, Some(mutated_runtime.path()));
    let output = transcript(&mutated);
    assert!(
        !mutated.status.success() && output.contains("VERIFICATION:- FAILED"),
        "mutating implies_short_circuit did not fail the proof -- the connective helper is not \
         load-bearing for this exemplar:\n{output}"
    );
}
