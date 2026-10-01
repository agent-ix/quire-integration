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
    path::{Path, PathBuf},
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

/// One raw `cargo kani` run over a bundle: its process output, the report
/// Kani exported for it, and the crate it ran in, kept so that check
/// locations relative to that crate still resolve.
struct BundleRun {
    output: Output,
    report: Option<Vec<u8>>,
    generated: GeneratedCrate,
}

impl BundleRun {
    fn transcript(&self) -> String {
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr)
        )
    }

    /// codegen's classification of the run, from the report Kani exported.
    fn classify(&self) -> codegen::ClassifiedRun {
        codegen::classify_kani_run(
            self.output.status.success(),
            self.report.as_deref(),
            &self.transcript(),
            None,
        )
        .unwrap_or_else(|refusal| panic!("{refusal}\n{}", self.transcript()))
    }
}

/// Where Kani writes the run's report, relative to the generated crate.
const REPORT_FILE: &str = "kani-report.json";

/// Run `bundle` plus `subject` under `cargo kani` with the bundle's own
/// options, exporting Kani's report. `runtime_patch` replaces
/// `quire-contract-runtime` with a local copy.
fn execute_kani(
    bundle: &codegen::KaniArtifactBundle,
    subject: &str,
    runtime_patch: Option<&Path>,
) -> BundleRun {
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&bundle.proof_graph.contents).unwrap();
    let generated = kani_crate(&bundle.rust.contents, subject, runtime_patch);
    let report_path = generated.path().join(REPORT_FILE);
    let output = Command::new("cargo")
        .arg("kani")
        .args(&graph.options)
        .args(["-Z", "unstable-options", "--export-json"])
        .arg(&report_path)
        .env("CARGO_TARGET_DIR", generated.target())
        .current_dir(generated.path())
        .output()
        .expect("cargo kani executes");
    let report = match fs::read(&report_path) {
        Ok(report) => Some(report),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("cannot read the Kani report {report_path:?}: {error}"),
    };
    BundleRun {
        output,
        report,
        generated,
    }
}

/// A throwaway crate whose `src/lib.rs` holds generated Kani `source` and
/// whose `src/subject.rs` holds `subject`, so a check Kani attributes to
/// `src/lib.rs` is in the generated code. `runtime_patch` replaces
/// `quire-contract-runtime` with a local copy.
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
    generated.write(
        "src/lib.rs",
        &format!("{source}\nmod subject;\npub use subject::subject;\n"),
    );
    generated.write("src/subject.rs", &format!("{subject}\n"));
    generated.write(
        "build.rs",
        "fn main() { println!(\"cargo:rustc-check-cfg=cfg(kani)\"); }\n",
    );
    generated
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

/// The modules this exemplar claims, as files of the generated crate.
/// ADR-011 §2.3 requires each claimed module to discharge at least one
/// SUCCESS check and to hold a mutation control that turns the proof red:
/// `src/lib.rs` is codegen's generated bundle, whose postcondition oracle
/// IT-011-SC-03 mutates. The runtime's `operators.rs`, which IT-011-SC-02
/// mutates, is not claimed here: runtime code is claimed by the runtime's own
/// ADR-011 §2.3 gate, `implies_short_circuit` only selects between its
/// operands and holds no checkable operation, and the `+` it evaluates is in
/// a closure defined in the generated bundle.
const CLAIMED_MODULES: [&str; 1] = ["src/lib.rs"];

/// The canonical path of each claimed module's file in the generated crate
/// at `crate_root`.
fn claimed_module_files(crate_root: &Path) -> Vec<PathBuf> {
    CLAIMED_MODULES
        .iter()
        .map(|module| {
            let file = crate_root.join(module);
            fs::canonicalize(&file)
                .unwrap_or_else(|error| panic!("claimed module {file:?} does not exist: {error}"))
        })
        .collect()
}

/// The file a Kani check location names, canonicalized. Kani prints a file
/// in the crate it ran in relative to that crate (`src/lib.rs`) and a file in
/// a dependency as an absolute path; both resolve against `crate_root`. A
/// location naming no existing file (Kani's `unknown`) resolves to nothing.
fn check_file(location: &codegen::KaniCheckLocation, crate_root: &Path) -> Option<PathBuf> {
    fs::canonicalize(crate_root.join(&location.file)).ok()
}

/// How many SUCCESS checks each of `modules` discharged: a check is
/// attributed to a module only when its location names exactly that file.
/// Every status other than SUCCESS is not counted (ADR-011 §2.3).
fn success_census<'m>(
    checks: &[codegen::KaniCheckResult],
    crate_root: &Path,
    modules: &'m [PathBuf],
) -> Vec<(&'m Path, usize)> {
    let discharged: Vec<PathBuf> = checks
        .iter()
        .filter(|check| check.status == codegen::KaniCheckStatus::Success)
        .filter_map(|check| check_file(&check.location, crate_root))
        .collect();
    modules
        .iter()
        .map(|module| {
            let count = discharged.iter().filter(|file| *file == module).count();
            (module.as_path(), count)
        })
        .collect()
}

/// The claimed modules of `census` that discharged no SUCCESS check.
fn unreached<'m>(census: &[(&'m Path, usize)]) -> Vec<&'m Path> {
    census
        .iter()
        .filter(|(_, discharged)| *discharged == 0)
        .map(|(module, _)| *module)
        .collect()
}

/// Fail, naming them, when any claimed module of `census` discharged no
/// SUCCESS check (ADR-011 §2.3 `unreached`).
fn assert_every_module_reached(census: &[(&Path, usize)], checks: &[codegen::KaniCheckResult]) {
    let unreached = unreached(census);
    assert!(
        unreached.is_empty(),
        "claimed modules discharged zero SUCCESS checks (unreached): {unreached:?}\n\
         census: {census:?}\nchecks: {checks:#?}"
    );
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
#[trace("IT-011-SC-01", "IT-011-SC-04")]
fn healthy_run_discharges_success_checks_in_every_claimed_module() {
    let bundle = function_application_bundle();
    let healthy = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    assert!(
        healthy.output.status.success(),
        "function-application proof did not verify:\n{}",
        healthy.transcript()
    );
    let run = healthy.classify();
    assert!(
        run.success_checks > 0,
        "the function-application proof discharged zero SUCCESS checks: {:#?}",
        run.checks
    );
    let modules = claimed_module_files(healthy.generated.path());
    let census = success_census(&run.checks, healthy.generated.path(), &modules);
    assert_every_module_reached(&census, &run.checks);
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
        unmutated.output.status.success(),
        "control run must verify before mutating:\n{}",
        unmutated.transcript()
    );
    // The runtime checkout the generated Kani crate resolves, fetched if needed.
    let locator = GeneratedCrate::new("composed-kani", &["exact"], "");
    locator.write("src/lib.rs", "");
    let runtime = package_root(&locator.path().join("Cargo.toml"), "quire-contract-runtime");
    let mutated_runtime = mutated_runtime_copy(&runtime);
    let mutated = execute_kani(&bundle, HEALTHY_SUBJECT, Some(mutated_runtime.path()));
    let outcome = mutated.classify().outcome;
    assert!(
        matches!(outcome, codegen::KaniRunOutcome::Falsified { .. }),
        "mutating implies_short_circuit did not falsify the proof -- the connective helper is not \
         load-bearing for this exemplar: {outcome:?}\n{}",
        mutated.transcript()
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

/// `source` with the one `+` in `fn {POSTCONDITION_ORACLE}` returning
/// `left + right + 1`: `amount_current + 1_i64` becomes
/// `amount_current + 1_i64 + 1_i64`.
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
    let end = byte_offset(source, syn::spanned::Spanned::span(*addition).end());
    format!("{} + 1_i64{}", &source[..end], &source[end..])
}

/// The decode schema of a bundle's symbolic arguments, in the order the
/// bundle's harness calls `kani::any()`.
fn bundle_arguments(graph: &codegen::ProofDependencyGraph) -> Vec<codegen::ObligationBinding> {
    graph
        .subject_arguments
        .iter()
        .map(|binding| codegen::ObligationBinding {
            identifier: binding.identifier.clone(),
            role: binding.role,
            primitive_type: binding.primitive_type,
            integer_bounds: binding.integer_bounds.clone(),
            dependencies: vec![binding.dependency.clone()],
        })
        .collect()
}

/// IT-011-SC-03: the mutation control for the exemplar's arithmetic.
///
/// The mutant must be non-equivalent: it has to make the claim false. An
/// equivalent mutant such as dropping the `+ 1` leaves
/// `amount < 1000 implies amount <= 1000`, which every in-domain `amount`
/// satisfies, so Kani proving it is the correct verdict, not a vacuous one.
/// Here the `+` returns `left + right + 1`, so the rule reads
/// `amount < 1000 implies amount + 2 <= 1000`, which is false exactly at
/// `amount = 999`.
#[test]
#[trace("IT-011-SC-03")]
fn mutating_the_compiled_postconditions_addition_falsifies_the_proof() {
    let bundle = function_application_bundle();
    let unmutated = execute_kani(&bundle, HEALTHY_SUBJECT, None);
    assert!(
        unmutated.output.status.success(),
        "control run must verify before mutating:\n{}",
        unmutated.transcript()
    );
    let mut mutated_bundle = bundle.clone();
    mutated_bundle.rust.contents = mutate_postcondition_addition(&bundle.rust.contents);
    assert_ne!(mutated_bundle.rust.contents, bundle.rust.contents);
    let mutated = execute_kani(&mutated_bundle, HEALTHY_SUBJECT, None);
    let outcome = mutated.classify().outcome;
    let codegen::KaniRunOutcome::Falsified { counterexample } = &outcome else {
        panic!(
            "an `add` returning `left + right + 1` still proves -- the exemplar's arithmetic is \
             not load-bearing: {outcome:?}\n{}",
            mutated.transcript()
        )
    };
    let graph: codegen::ProofDependencyGraph =
        serde_json::from_str(&bundle.proof_graph.contents).unwrap();
    let harness = graph
        .options
        .windows(2)
        .find_map(|pair| (pair[0] == "--harness").then_some(pair[1].as_str()))
        .expect("the bundle's options name its harness");
    let (module, proof) = harness
        .split_once("::")
        .expect("the bundle's harness is module-qualified");
    let decoded =
        codegen::decode_falsification(proof, module, &bundle_arguments(&graph), counterexample)
            .unwrap_or_else(|failure| panic!("counterexample does not decode: {failure:?}"));
    assert_eq!(
        decoded,
        [("amount_current".to_owned(), WitnessValue::Integer(999))]
    );
}

/// A Kani check of class `assertion` at `file` with `status`.
fn check_at(id: u64, file: &str, status: codegen::KaniCheckStatus) -> codegen::KaniCheckResult {
    codegen::KaniCheckResult {
        id,
        class: codegen::KaniCheckClass::from("assertion".to_owned()),
        location: codegen::KaniCheckLocation {
            file: file.to_owned(),
            line: Some(1),
        },
        status,
    }
}

/// IT-011-SC-04's attribution rule, without a Kani run: a check is
/// attributed by its exact file, whether Kani printed it relative or
/// absolute. A file whose path a claimed module's path is a prefix of, or a
/// same-named file in another crate, is not attributed, and only SUCCESS
/// counts.
#[test]
#[trace("IT-011-SC-04")]
fn success_census_attributes_checks_by_exact_file() {
    use codegen::KaniCheckStatus::{Satisfied, Success, Unreachable};

    let generated = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let mutated = tempfile::tempdir().unwrap();
    for root in [generated.path(), runtime.path(), mutated.path()] {
        fs::create_dir(root.join("src")).unwrap();
        for file in ["src/lib.rs", "src/lib.rs.orig", "src/operators.rs"] {
            fs::write(root.join(file), "").unwrap();
        }
    }
    let modules = [
        fs::canonicalize(generated.path().join("src/lib.rs")).unwrap(),
        fs::canonicalize(runtime.path().join("src/operators.rs")).unwrap(),
    ];
    let absolute = |root: &Path, file: &str| root.join(file).display().to_string();

    let checks = [
        // Relative: Kani's spelling for the crate it ran in.
        check_at(1, "src/lib.rs", Success),
        // Absolute: Kani's spelling for a dependency.
        check_at(2, &absolute(runtime.path(), "src/operators.rs"), Success),
        check_at(3, &absolute(runtime.path(), "src/operators.rs"), Success),
        // `src/lib.rs` is a prefix of `src/lib.rs.orig`.
        check_at(4, "src/lib.rs.orig", Success),
        // Same crate-relative path, another crate.
        check_at(5, &absolute(mutated.path(), "src/operators.rs"), Success),
        // The generated crate's own `src/operators.rs`, printed relative.
        check_at(6, "src/operators.rs", Success),
        check_at(7, "unknown", Success),
        // Not SUCCESS.
        check_at(8, "src/lib.rs", Unreachable),
        check_at(9, &absolute(runtime.path(), "src/operators.rs"), Satisfied),
    ];
    let census = success_census(&checks, generated.path(), &modules);
    assert_eq!(
        census,
        [(modules[0].as_path(), 1), (modules[1].as_path(), 2)]
    );
    assert!(unreached(&census).is_empty());
    assert_every_module_reached(&census, &checks);

    // A claimed module no SUCCESS check names is counted zero, and so unreached.
    let census = success_census(&checks[3..], generated.path(), &modules);
    assert_eq!(
        census,
        [(modules[0].as_path(), 0), (modules[1].as_path(), 0)]
    );
    assert_eq!(
        unreached(&census),
        [modules[0].as_path(), modules[1].as_path()]
    );
    let partial = [(modules[0].as_path(), 1), (modules[1].as_path(), 0)];
    assert_eq!(unreached(&partial), [modules[1].as_path()]);

    // The failure names the unreached module.
    let failure = std::panic::catch_unwind(|| assert_every_module_reached(&partial, &[]))
        .expect_err("an unreached claimed module fails the test");
    let message = failure
        .downcast_ref::<String>()
        .expect("the assertion message is formatted");
    assert!(
        message.contains(&format!("(unreached): [{:?}]", modules[1])),
        "{message}"
    );
}
