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
    fs,
    path::Path,
    process::{Command, Output},
    time::Duration,
};

use ix_trace_rs::trace;
use qsl_replay::{
    ByteDigest, DigestDomain, DigestRecord, ProofCategory, ScalarLimits, StageLimits,
    WitnessSettlement, WitnessValue, MAX_ENCODED_BYTES,
};
use quire_contract_codegen as codegen;
use quire_contract_ir as ir;
use quire_spec_language::lowering::{lower_for, LoweringLimits, ProjectionTarget};
use serde_json::json;
use support::{
    bound_clause, config, package_root, parameter_names, runtime_manifest, GeneratedCrate,
    CONFIG_DOMAIN, RUNTIME_GIT_URL,
};

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
    let generated = kani_crate(&bundle.rust.contents, subject, runtime_patch);
    Command::new("cargo")
        .arg("kani")
        .args(&graph.options)
        .env("CARGO_TARGET_DIR", generated.target())
        .current_dir(generated.path())
        .output()
        .expect("cargo kani executes")
}

/// A throwaway crate holding generated Kani `source` followed by `subject`.
/// `runtime_patch` replaces `quire-contract-runtime` with a local copy.
fn kani_crate(source: &str, subject: &str, runtime_patch: Option<&Path>) -> GeneratedCrate {
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
    generated.write("src/lib.rs", &format!("{source}\n{subject}\n"));
    generated.write(
        "build.rs",
        "fn main() { println!(\"cargo:rustc-check-cfg=cfg(kani)\"); }\n",
    );
    generated
}

fn transcript(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// codegen's classification of one raw `cargo kani` run.
fn classify(output: &Output) -> codegen::KaniRunOutcome {
    codegen::classify_kani_run(output.status.success(), &transcript(output))
}

// ---------------------------------------------------------------------------
// IT-010-SC-05: the VersionUnchanged postcondition's Kani obligation proves for the identity
// subject, and the changed subject's counterexample is decoded by codegen and replayed through
// QSL's `qsl_replay::replay` facade.
// ---------------------------------------------------------------------------

/// Wall-clock budget for one real Kani run.
const KANI_TIMEOUT: Duration = Duration::from_secs(600);

/// The changed subject: in-domain, and equal to its input only at 500. The
/// replay twin mirrors its `1000 - version`.
const CHANGED_SUBJECT: &str =
    "/// Changed ConfigVersion subject.\npub fn subject(version_pre: i64) -> i64 { 1000 - version_pre }";

/// The selected function of the QSL replay twin.
const TWIN_FUNCTION: &str = "version_unchanged";

/// The header of a standalone QSL function unit, selecting the profile
/// `qsl_replay::call_site` compiles against.
const TWIN_HEADER: &str = "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\" version \"1-draft.2\" digest \"sha256:c8c7ae9fbe783286369ecc83f006190f83be4c3c8fc585766617c90f27a25b16\";\n";

/// No proving run carries limits, so the replay is charged against
/// unlimited stand-ins.
const UNLIMITED: ScalarLimits = ScalarLimits {
    integer_bits: u64::MAX,
    decimal_digits: u64::MAX,
    scale_expansion: u64::MAX,
    text_input_bytes: u64::MAX,
    text_scalars: u64::MAX,
    normalized_scalars: u64::MAX,
    unit_edges: u64::MAX,
    value_occurrences: u64::MAX,
    work_units: u64::MAX,
    result_units: u64::MAX,
};

/// The QSL function `qsl_replay::replay` evaluates for one counterexample:
/// the VersionUnchanged clause over the subject `post_of` computes, with
/// the harness argument `argument` as its one parameter. Equality is written
/// as two `<=`, because QSL's `=` requires both sides to have the same
/// integer range and `post_of` may widen it.
fn replay_twin(argument: &str, post_of: &str) -> codegen::ReplayPackage {
    let source = format!(
        "{TWIN_HEADER}function {TWIN_FUNCTION} using v({argument}: Int[0, 1000]): Boolean pure {{ {post_of} <= {argument} and {argument} <= {post_of} }}\n"
    );
    let s1 = ScalarLimits {
        text_input_bytes: u64::try_from(MAX_ENCODED_BYTES).unwrap(),
        ..UNLIMITED
    };
    codegen::ReplayPackage::new(
        codegen::ReplayInputs {
            source: codegen::LockedSource {
                authority: "agent-ix".to_owned(),
                identity: "ix://quire-integration/config-version/replay-twin".to_owned(),
                namespace: "test".to_owned(),
                revision: "1".to_owned(),
                bytes: source.into_bytes(),
            },
            dependencies: Vec::new(),
            backend_manifest: DigestRecord::mint(
                DigestDomain::ToolManifestJcsV1,
                ByteDigest::of(b"kani").as_bytes(),
            ),
            accounting_limits: UNLIMITED,
            stage_limits: StageLimits {
                s1,
                s2: UNLIMITED,
                s3: UNLIMITED,
                s4: UNLIMITED,
            },
        },
        TWIN_FUNCTION,
    )
    .unwrap_or_else(|refusal| panic!("replay twin does not compile: {refusal}"))
}

/// codegen's one Kani obligation harness for the VersionUnchanged clause.
fn config_obligation(bound: &ir::BoundPackage) -> codegen::KaniObligationHarness {
    let clause = bound_clause(bound, config::CLAUSE).identity().clone();
    let items = [codegen::ObligationItem::BoundClause {
        package: bound,
        clause: &clause,
    }];
    let outcome = codegen::negotiate_kani_obligations(&codegen::KaniObligationRequest {
        items: &items,
        subject_path: "crate::subject",
        unwind: 2,
    })
    .unwrap();
    let codegen::KaniObligationOutcome::Emitted {
        records,
        mut harnesses,
        ..
    } = outcome
    else {
        panic!("VersionUnchanged obligation was rejected: {outcome:#?}")
    };
    assert!(
        matches!(
            records.as_slice(),
            [codegen::ObligationRecord {
                disposition: codegen::ObligationDisposition::Supported { .. },
                ..
            }]
        ),
        "VersionUnchanged obligation is not supported: {records:#?}"
    );
    assert_eq!(harnesses.len(), 1);
    harnesses.remove(0)
}

/// Run `subject` under `harness` through codegen's executor and classifier.
fn prove(harness: &codegen::KaniObligationHarness, subject: &str) -> codegen::KaniRunOutcome {
    let installation = codegen::KaniInstallation::discover().expect("cargo-kani is installed");
    let generated = kani_crate(&harness.rust.contents, subject, None);
    codegen::execute_kani_obligation(&codegen::KaniExecutionRequest {
        installation: &installation,
        harness: harness.into(),
        crate_directory: generated.path(),
        target_directory: &generated.target(),
        timeout: KANI_TIMEOUT,
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
    .outcome
}

#[test]
#[trace("IT-010-SC-02", "IT-010-SC-05")]
fn kani_proves_identity_and_counterexample_replays_through_qsl_replay() {
    let models = [support::config_model()];
    let native = support::config_package(&models);
    let projection = lower_for(
        &native,
        ProjectionTarget::StateScalarIrV1,
        LoweringLimits::default(),
    )
    .unwrap();
    let bound = ir::BoundPackage::from_json_bytes(projection.bytes()).unwrap();
    let clause = bound_clause(&bound, config::CLAUSE);
    let bundle = support::absent_precondition_kani_bundle(
        clause,
        "version_unchanged_absent_precondition",
        "it-010-config-version",
    );
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&bundle.proof_graph.contents).unwrap();
    assert_eq!(graph.proof_id, "it-010-config-version");
    assert_eq!(graph.requirement_id, config::REQUIREMENT);
    assert_eq!(graph.requirement_revision, config::REQUIREMENT_REVISION);
    assert!(graph.dependencies.is_empty());
    assert_eq!(graph.subject_arguments.len(), 1);
    assert_eq!(graph.subject_results.len(), 1);
    for binding in graph.subject_arguments.iter().chain(&graph.subject_results) {
        let bounds = binding.integer_bounds.as_ref().unwrap();
        assert_eq!((bounds.minimum, bounds.maximum), (0, 1000));
    }

    let harness = config_obligation(&bound);
    let identity = &harness.identity;
    assert_eq!(identity.clause.clause().as_str(), config::CLAUSE);
    let [argument] = identity.arguments.as_slice() else {
        panic!("one symbolic pre-state argument: {:#?}", identity.arguments)
    };
    assert_eq!(identity.results.len(), 1);
    for binding in identity.arguments.iter().chain(&identity.results) {
        let bounds = binding.integer_bounds.as_ref().unwrap();
        assert_eq!((bounds.minimum, bounds.maximum), (0, 1000));
    }
    assert!(identity
        .options
        .windows(2)
        .any(|pair| pair == ["--concrete-playback", "print"]));

    assert_eq!(
        prove(
            &harness,
            "/// Identity ConfigVersion subject.\npub fn subject(version_pre: i64) -> i64 { version_pre }",
        ),
        codegen::KaniRunOutcome::Verified
    );

    let outcome = prove(&harness, CHANGED_SUBJECT);
    let codegen::KaniRunOutcome::Falsified { counterexample } = &outcome else {
        panic!("the changed subject must be falsified: {outcome:?}")
    };
    let decoded = codegen::decode_falsification(
        &identity.harness_symbol,
        &identity.module_symbol,
        &identity.arguments,
        counterexample,
    )
    .unwrap_or_else(|failure| panic!("counterexample does not decode: {failure:?}"));
    let [(name, WitnessValue::Integer(pre))] = decoded.as_slice() else {
        panic!("one decoded integer: {decoded:?}")
    };
    assert_eq!(name, &argument.identifier);
    assert!(CONFIG_DOMAIN.contains(pre));
    assert_ne!(*pre, 500, "the changed subject keeps 500 unchanged");

    // QSL evaluates the clause over the changed subject at the decoded input and settles the
    // same violation Kani reported.
    let changed_twin = replay_twin(
        &argument.identifier,
        &format!("1000 - {}", argument.identifier),
    );
    assert_eq!(
        codegen::replay_counterexample(identity, counterexample, &changed_twin)
            .expect("QSL settles the replay"),
        codegen::ReplayVerdict::Reproduced
    );
    // Against the identity subject the same counterexample is not reproduced: the verdict is
    // QSL's evaluation, not a value this test supplies.
    let identity_twin = replay_twin(&argument.identifier, &argument.identifier);
    assert_eq!(
        codegen::replay_counterexample(identity, counterexample, &identity_twin)
            .expect("QSL settles the replay"),
        codegen::ReplayVerdict::EvidenceFailure(codegen::EvidenceFailureCause::Verdict {
            settlement: WitnessSettlement::Inconclusive,
            category: ProofCategory::Success,
        })
    );
    // The authored clause itself, executed natively over the same pre/post, is false too.
    assert_eq!(
        support::config_native_verdict(&native, &models[0], *pre, 1000 - pre),
        Some(false)
    );
}

// ---------------------------------------------------------------------------
// IT-011: the function-application exemplar `amount < 1000 implies amount + 1
// <= 1000` -- its proof discharges SUCCESS checks, and mutating the connective
// helper its compiled postcondition calls falsifies the proof.
// ---------------------------------------------------------------------------

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

/// Byte offset of a `proc-macro2` line (1-based) / column (0-based, in
/// characters) position in `source`.
fn byte_offset(source: &str, position: proc_macro2::LineColumn) -> usize {
    let line_start: usize = source
        .split_inclusive('\n')
        .take(position.line - 1)
        .map(str::len)
        .sum();
    line_start
        + source[line_start..]
            .chars()
            .take(position.column)
            .map(char::len_utf8)
            .sum::<usize>()
}

/// `source` with `fn implies_short_circuit`'s body replaced by
/// `R::from(false)`, its parameters ignored.
fn mutate_connective_helper(source: &str) -> String {
    let file = syn::parse_file(source).expect("operators.rs is Rust");
    let function = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == "implies_short_circuit" => {
                Some(function)
            }
            _ => None,
        })
        .expect("quire-contract-runtime's operators.rs defines fn implies_short_circuit");
    let ignored: String = parameter_names(function)
        .iter()
        .map(|name| format!("let _ = {name}; "))
        .collect();
    let body = function.block.brace_token.span.join();
    let (start, end) = (
        byte_offset(source, body.start()),
        byte_offset(source, body.end()),
    );
    format!(
        "{}{{ {ignored}R::from(false) }}{}",
        &source[..start],
        &source[end..]
    )
}

/// A throwaway copy of the `quire-contract-runtime` checkout at `source`
/// with `operators::implies_short_circuit` mutated to return `false`. The
/// copy keeps the manifest's features and dependencies and drops only its
/// `[workspace]` table, so it builds as a plain crate.
fn mutated_runtime_copy(source: &Path) -> tempfile::TempDir {
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
    let mutated = mutate_connective_helper(&original);
    assert_ne!(mutated, original);
    fs::write(&operators, mutated).unwrap();
    directory
}

#[test]
#[trace("IT-011-SC-01")]
fn healthy_run_discharges_success_checks() {
    let bundle = function_application_bundle();
    let healthy = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    let output = transcript(&healthy);
    assert!(
        healthy.status.success(),
        "function-application proof did not verify:\n{output}"
    );
    assert!(
        discharged_success_checks(&output) > 0,
        "the function-application proof discharged zero SUCCESS checks:\n{output}"
    );
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
    // The runtime checkout the generated Kani crate resolves, fetched if needed.
    let locator = GeneratedCrate::new("composed-kani", &["exact"], "");
    locator.write("src/lib.rs", "");
    let runtime = package_root(&locator.path().join("Cargo.toml"), "quire-contract-runtime");
    let mutated_runtime = mutated_runtime_copy(&runtime);
    let mutated = execute_kani(&bundle, HEALTHY_SUBJECT, Some(mutated_runtime.path()));
    let outcome = classify(&mutated);
    assert!(
        matches!(outcome, codegen::KaniRunOutcome::Falsified { .. }),
        "mutating implies_short_circuit did not falsify the proof -- the connective helper is not \
         load-bearing for this exemplar: {outcome:?}\n{}",
        transcript(&mutated)
    );
}

/// The compiled postcondition oracle of the function-application exemplar.
const POSTCONDITION_ORACLE: &str = "oracle_populationrule_7_population_rule";

/// Every `+` expression in a function body.
#[derive(Default)]
struct Additions<'a>(Vec<&'a syn::ExprBinary>);

impl<'a> syn::visit::Visit<'a> for Additions<'a> {
    fn visit_expr_binary(&mut self, expression: &'a syn::ExprBinary) {
        if matches!(expression.op, syn::BinOp::Add(_)) {
            self.0.push(expression);
        }
        syn::visit::visit_expr_binary(self, expression);
    }
}

/// `source` with the one `+` in `fn {POSTCONDITION_ORACLE}` replaced by its
/// left operand: an `add` that returns its left operand.
fn mutate_postcondition_addition(source: &str) -> String {
    let file = syn::parse_file(source).expect("generated Kani source is Rust");
    let function = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == POSTCONDITION_ORACLE => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the Kani bundle defines fn {POSTCONDITION_ORACLE}"));
    let mut additions = Additions::default();
    syn::visit::Visit::visit_item_fn(&mut additions, function);
    let [addition] = additions.0.as_slice() else {
        panic!(
            "fn {POSTCONDITION_ORACLE} has {} additions, not the exemplar's one `amount + 1`",
            additions.0.len()
        )
    };
    let offsets = |span: proc_macro2::Span| {
        (
            byte_offset(source, span.start()),
            byte_offset(source, span.end()),
        )
    };
    let (start, end) = offsets(syn::spanned::Spanned::span(*addition));
    let (left_start, left_end) = offsets(syn::spanned::Spanned::span(&*addition.left));
    format!(
        "{}{}{}",
        &source[..start],
        &source[left_start..left_end],
        &source[end..]
    )
}

/// An `add` that returns its left operand turns the postcondition into
/// `amount < 1000 implies amount <= 1000`, which every in-domain `amount`
/// satisfies. A proof that still verifies under that mutation has not
/// exercised the arithmetic. Expected to fail until quire-contract-codegen
/// IR-458 lands: the mutated postcondition is still a valid property over
/// the domain, so Kani proves it.
#[test]
#[trace("IT-011-SC-03")]
fn mutating_the_compiled_postconditions_addition_falsifies_the_proof() {
    let bundle = function_application_bundle();
    let unmutated = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    assert!(
        unmutated.status.success(),
        "control run must verify before mutating:\n{}",
        transcript(&unmutated)
    );
    let mut mutated_bundle = bundle.clone();
    mutated_bundle.rust.contents = mutate_postcondition_addition(&bundle.rust.contents);
    assert_ne!(mutated_bundle.rust.contents, bundle.rust.contents);
    let mutated = execute_kani(&mutated_bundle, HEALTHY_SUBJECT, None);
    let outcome = classify(&mutated);
    assert!(
        matches!(outcome, codegen::KaniRunOutcome::Falsified { .. }),
        "an `add` returning its left operand still proves -- the exemplar's arithmetic is not \
         load-bearing: {outcome:?}\n{}",
        transcript(&mutated)
    );
}
