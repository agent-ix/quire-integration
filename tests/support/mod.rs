// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared setup for the composed QSL -> contract IR -> codegen tests (QSL-334).
//!
//! Everything here reaches QSL through its public API. QSL's authored fixture
//! files are read from the `quire-spec-language` checkout Cargo resolved for
//! this crate; none is copied into this repository.

use std::{
    fs,
    num::NonZeroU64,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
    time::Duration,
};

use qsl_foundation::{Source, SourceIdentity};
use quire_contract_codegen as codegen;
use quire_contract_ir as ir;
use quire_spec_language::{
    checking::{check, CheckBindings, CheckLimits, ClauseBinding},
    formal_source::FormalSource,
    link_native,
    model_source::{self, ModelSourceLimits},
    native_model::{ModelLimits, NativeModel},
    package::{NativePackage, PackageLimits},
    parse,
    runtime::{
        self, ArtifactLimits, ExecutionLimits, ExecutionOutcome, ExecutionSelection, FieldBinding,
        Invocation, InvocationDraft, ModelBinding, ObjectEntry, ObjectIdentity,
        ObservationSelection, Population, QualifiedName, RuntimeInput, Snapshot, SnapshotDraft,
        ValueBinding, ValueId, ValueNode,
    },
    Limits, LinkLimits,
};
use serde::Deserialize;
use serde_json::Value;

/// The nominal `versionNumber` domain of the ConfigVersion model.
pub const CONFIG_DOMAIN: std::ops::RangeInclusive<i64> = 0..=1000;

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: PathBuf,
}

/// Root directory of package `name` in the dependency graph of `manifest`,
/// as `cargo metadata` resolves it (fetching it when needed).
pub fn package_root(manifest: &Path, name: &str) -> PathBuf {
    let output = run(Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(manifest));
    let metadata: Metadata = serde_json::from_slice(&output.stdout).unwrap();
    let package = metadata
        .packages
        .into_iter()
        .find(|package| package.name == name)
        .unwrap_or_else(|| panic!("{name} is not in the dependency graph of {manifest:?}"));
    package
        .manifest_path
        .parent()
        .expect("a manifest path has a parent directory")
        .to_path_buf()
}

/// Root directory of the resolved `quire-spec-language` package.
pub fn qsl_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        package_root(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
            "quire-spec-language",
        )
    })
}

/// Run `command` and require success, reporting its output otherwise.
pub fn run(command: &mut Command) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("cannot run {command:?}: {error}"));
    assert!(
        output.status.success(),
        "{command:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn source_identity(identity: &str) -> SourceIdentity {
    SourceIdentity {
        authority: "agent-ix".into(),
        identity: identity.into(),
        revision_namespace: "test".into(),
        revision: "1".into(),
    }
}

fn formal(source: Source, document: &str, revision: u64) -> FormalSource {
    FormalSource::new(
        source,
        ir::SourceIdentity::new(
            ir::SourceDocumentId::new(document).unwrap(),
            ir::SourceRevision::new(revision).unwrap(),
        ),
    )
}

fn admit_model(text: &str, identity: &str, document: &str) -> NativeModel {
    let source = Source::read(
        source_identity(identity),
        "model.json",
        text.as_bytes(),
        qsl_foundation::source::MAX_SOURCE_BYTES,
    )
    .expect("model source is within its intake limits");
    model_source::read(
        formal(source, document, 1),
        model_source::FORMAT,
        ModelSourceLimits::default(),
    )
    .expect("model source decodes")
    .admit(ModelLimits::default())
    .expect("model admits")
}

fn check_program<'m>(
    models: &'m [NativeModel],
    identity: &str,
    text: &str,
    document: &str,
    clauses: Vec<ClauseBinding>,
) -> NativePackage<'m> {
    let unit = parse(
        source_identity(identity),
        "program.native",
        text.as_bytes(),
        Limits::default(),
    )
    .expect("program parses");
    let source = formal(unit.source().clone(), document, 1);
    let linked = link_native(unit, models, LinkLimits::default()).expect("program links");
    let checked = check(
        linked,
        CheckBindings { source, clauses },
        CheckLimits::default(),
    )
    .expect("program checks");
    NativePackage::new(checked, PackageLimits::default()).expect("package builds")
}

fn symbol(name: &str) -> ir::SymbolName {
    ir::SymbolName::new(name).unwrap()
}

fn push(arena: &mut Vec<ValueNode>, value: ValueNode) -> ValueId {
    let id = ValueId::new(u32::try_from(arena.len()).unwrap());
    arena.push(value);
    id
}

/// QSL's authored rule model fixture (`tests/fixtures/native-rule-model.json`).
fn rule_fixture() -> &'static Value {
    static FIXTURE: OnceLock<Value> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let path = qsl_root().join("tests/fixtures/native-rule-model.json");
        let text = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        serde_json::from_str(&text).unwrap()
    })
}

/// QSL's authored rule model with state values added to it.
///
/// `values` pairs a value name with its model type, for example
/// `("flag", json!({"kind": "boolean"}))`.
pub fn rule_model(values: &[(&str, Value)]) -> NativeModel {
    let mut model = rule_fixture().clone();
    let declared = model["values"].as_array_mut().unwrap();
    for (name, value_type) in values {
        declared.push(serde_json::json!({"name": name, "kind": "state", "type": value_type}));
    }
    admit_rule_model(&serde_json::to_string_pretty(&model).unwrap())
}

/// Admit a rule-model source text under the identity [`rule_model`] uses.
pub fn admit_rule_model(text: &str) -> NativeModel {
    admit_model(text, "ix://quire-integration/rule-model", "RuleModelSource")
}

/// The requirement every rule-program clause is bound to.
pub fn rule_owner() -> ir::RequirementRef {
    ir::RequirementRef::parse("example/runtime-rules", "PopulationRule", 7).unwrap()
}

/// A checked two-clause invariant program over `models[0]`'s `Node`:
/// `Rule` (clause `population_rule`) holds `expression`, `Other`
/// (clause `other_rule`) holds `true`.
pub fn rule_package<'m>(models: &'m [NativeModel], expression: &str) -> NativePackage<'m> {
    rule_package_with(models, expression, "true")
}

/// [`rule_package`] with `other` as the later `Other` clause's expression.
pub fn rule_package_with<'m>(
    models: &'m [NativeModel],
    expression: &str,
    other: &str,
) -> NativePackage<'m> {
    let owner = models[0].environment().owner();
    let text = format!(
        "language \"ix:native\" edition \"0-draft\";\nprofile \"state-finite/0-draft\";\nmodel M = {} version \"{}\" digest \"{}\";\ninvariant Rule on M::Node at current {{ {expression} }}\ninvariant Other on M::Node at current {{ {other} }}\n",
        serde_json::to_string(owner.package().as_str()).unwrap(),
        owner.revision().get(),
        models[0].digest(),
    );
    let clauses = [("Rule", "population_rule"), ("Other", "other_rule")]
        .into_iter()
        .map(|(name, clause)| ClauseBinding {
            name: name.into(),
            requirement: rule_owner(),
            clause: ir::ClauseId::new(clause).unwrap(),
            execution_point: ir::ExecutionPoint::Handler {
                name: ir::AnchorName::new("validate").unwrap(),
            },
        })
        .collect();
    check_program(
        models,
        "ix://quire-integration/rule-program",
        &text,
        "RuleProgramSource",
        clauses,
    )
}

/// The record every rule-program clause is written on.
const NODE: &str = "Node";

/// The first element of fixture array `array` whose `key` equals `value`.
fn fixture_entry(array: &str, key: &str, value: &str) -> &'static Value {
    rule_fixture()[array]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry[key] == value)
        .unwrap_or_else(|| panic!("rule model has no {array} entry with {key} = {value}"))
}

/// The universe the rule model declares for `Node` objects.
fn node_universe() -> &'static str {
    fixture_entry("objects", "record", NODE)["universe"]
        .as_str()
        .unwrap()
}

fn node(model: &NativeModel, key: &str) -> ObjectIdentity {
    ObjectIdentity {
        model: model.environment().owner().clone(),
        record: symbol(NODE),
        universe: symbol(node_universe()),
        key: key.into(),
    }
}

/// A value for a `Node` field of `value_type`, chosen by type alone: `0`
/// clamped into an integer scalar's bounds, empty text, `false`, absent,
/// an empty sequence, or a reference to the `self` node.
fn field_value(model: &NativeModel, value_type: &Value) -> ValueNode {
    match value_type["kind"].as_str() {
        Some("boolean") => ValueNode::Boolean { value: false },
        Some("option") => ValueNode::Absent,
        Some("sequence") => ValueNode::Sequence { values: Vec::new() },
        Some("scalar") => {
            let scalar = fixture_entry("scalars", "name", value_type["name"].as_str().unwrap());
            match scalar["kind"].as_str() {
                Some("integer") => ValueNode::Integer {
                    value: 0_i64.clamp(
                        scalar["minimum"].as_i64().unwrap(),
                        scalar["maximum"].as_i64().unwrap(),
                    ),
                },
                Some("text") => ValueNode::Text {
                    value: String::new(),
                },
                _ => panic!("no type-derived value for scalar {scalar}"),
            }
        }
        Some("record") => {
            let object =
                fixture_entry("objects", "reference", value_type["name"].as_str().unwrap());
            assert_eq!(
                object["record"], NODE,
                "a type-derived reference can only name the one `self` node"
            );
            ValueNode::Reference {
                identity: node(model, "self"),
            }
        }
        _ => panic!("no type-derived value for field type {value_type}"),
    }
}

/// A current snapshot holding one complete `Node` named `self` and the given
/// state values. The node's fields are every field the rule model declares
/// for `Node`, each with a value chosen by its type; the tests' assertions
/// depend only on the explicitly given state values.
pub fn node_snapshot(model: &NativeModel, values: &[(&str, ValueNode)]) -> Snapshot {
    let mut arena = Vec::new();
    let fields = fixture_entry("records", "name", NODE)["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| FieldBinding {
            name: symbol(field["name"].as_str().unwrap()),
            value: push(&mut arena, field_value(model, &field["type"])),
        })
        .collect();
    let values = values
        .iter()
        .map(|(name, value)| ValueBinding {
            declaration: QualifiedName {
                model: model.environment().owner().clone(),
                name: symbol(name),
            },
            value: push(&mut arena, value.clone()),
        })
        .collect();
    Snapshot::new(
        source_identity("ix://quire-integration/node-snapshot"),
        SnapshotDraft {
            observation: ir::StateObservation::Current,
            models: vec![ModelBinding {
                model: model.environment().owner().clone(),
                digest: model.digest(),
            }],
            populations: vec![Population {
                model: model.environment().owner().clone(),
                record: symbol(NODE),
                universe: symbol(node_universe()),
                complete: true,
                objects: vec![ObjectEntry {
                    key: "self".into(),
                    fields,
                }],
            }],
            values,
            arena,
        },
        ArtifactLimits::default(),
    )
    .expect("node snapshot is structurally valid")
}

/// Select the `population_rule` clause against `snapshot`'s `self` node.
pub fn rule_selection(model: &NativeModel, snapshot: &Snapshot) -> ExecutionSelection {
    ExecutionSelection {
        requirement: rule_owner(),
        clause: ir::ClauseId::new("population_rule").unwrap(),
        observation: ObservationSelection::Current {
            snapshot: snapshot.reference(),
            self_object: node(model, "self"),
        },
    }
}

/// The clause a generated artifact set was produced for, by clause id.
pub fn bound_clause<'p>(bound: &'p ir::BoundPackage, clause: &str) -> &'p ir::BoundClause {
    bound
        .clauses()
        .iter()
        .find(|candidate| candidate.identity().clause().as_str() == clause)
        .unwrap_or_else(|| panic!("bound package has no clause {clause}"))
}

/// QSL's authored ConfigVersion model (`examples/config-version/model.json`).
pub fn config_model() -> NativeModel {
    let path = qsl_root().join("examples/config-version/model.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    admit_model(
        &text,
        "ix://example/config-version/model",
        "ConfigVersionModelSource",
    )
}

/// This repository's own ConfigVersion inputs: a `VersionUnchanged`-shaped
/// postcondition over QSL's ConfigVersion model, and the two-object version
/// history it runs against.
pub mod config {
    /// Requirement package the postcondition is bound to.
    pub const REQUIREMENT_PACKAGE: &str = "quire-integration/config-version";
    /// Requirement the postcondition is bound to.
    pub const REQUIREMENT: &str = "VersionUnchanged";
    /// Revision of [`REQUIREMENT`].
    pub const REQUIREMENT_REVISION: u64 = 1;
    /// Clause id of the postcondition.
    pub const CLAUSE: &str = "version_unchanged";
    /// The postcondition: `attemptUpdate` leaves the version unchanged.
    pub const EXPRESSION: &str = "self.versionNumber = pre(self.versionNumber)";
    /// Version of the history's root object; each case varies the child's.
    pub const ROOT_VERSION: i64 = 3;
}

/// The checked `VersionUnchanged`-shaped postcondition over `models[0]`.
pub fn config_package(models: &[NativeModel]) -> NativePackage<'_> {
    let owner = models[0].environment().owner();
    let text = format!(
        "language \"ix:native\" edition \"0-draft\";\nprofile \"state-finite/0-draft\";\nmodel Config = {} version \"{}\" digest \"{}\";\npost {} on Config::ConfigVersion::attemptUpdate {{ {} }}\n",
        serde_json::to_string(owner.package().as_str()).unwrap(),
        owner.revision().get(),
        models[0].digest(),
        config::REQUIREMENT,
        config::EXPRESSION,
    );
    let clause = ClauseBinding {
        name: config::REQUIREMENT.into(),
        requirement: ir::RequirementRef::parse(
            config::REQUIREMENT_PACKAGE,
            config::REQUIREMENT,
            config::REQUIREMENT_REVISION,
        )
        .unwrap(),
        clause: ir::ClauseId::new(config::CLAUSE).unwrap(),
        execution_point: ir::ExecutionPoint::Post {
            operation: ir::AnchorName::new("attemptUpdate").unwrap(),
        },
    };
    check_program(
        models,
        "ix://quire-integration/config-version/program",
        &text,
        "ConfigVersionProgram",
        vec![clause],
    )
}

fn config_object(model: &NativeModel, key: &str) -> ObjectIdentity {
    ObjectIdentity {
        model: model.environment().owner().clone(),
        record: symbol("ConfigVersion"),
        universe: symbol("config_history"),
        key: key.into(),
    }
}

/// A `root` / `child` ConfigVersion history whose child has `version`.
fn config_snapshot(
    model: &NativeModel,
    observation: ir::StateObservation,
    version: i64,
    identity: &str,
) -> Snapshot {
    let mut arena = Vec::new();
    let mut entry = |key: &str, version: i64, parent: Option<&str>| {
        let version = push(&mut arena, ValueNode::Integer { value: version });
        let parent = match parent {
            Some(parent) => {
                let value = push(
                    &mut arena,
                    ValueNode::Reference {
                        identity: config_object(model, parent),
                    },
                );
                push(&mut arena, ValueNode::Present { value })
            }
            None => push(&mut arena, ValueNode::Absent),
        };
        ObjectEntry {
            key: key.into(),
            fields: vec![
                FieldBinding {
                    name: symbol("versionNumber"),
                    value: version,
                },
                FieldBinding {
                    name: symbol("parent"),
                    value: parent,
                },
            ],
        }
    };
    let objects = vec![
        entry("root", config::ROOT_VERSION, None),
        entry("child", version, Some("root")),
    ];
    Snapshot::new(
        source_identity(identity),
        SnapshotDraft {
            observation,
            models: vec![ModelBinding {
                model: model.environment().owner().clone(),
                digest: model.digest(),
            }],
            populations: vec![Population {
                model: model.environment().owner().clone(),
                record: symbol("ConfigVersion"),
                universe: symbol("config_history"),
                complete: true,
                objects,
            }],
            values: Vec::new(),
            arena,
        },
        ArtifactLimits::default(),
    )
    .expect("ConfigVersion snapshot is structurally valid")
}

/// The native `VersionUnchanged` verdict for one `attemptUpdate` that moves
/// the child from `pre` to `post`.
///
/// An out-of-domain version must fail runtime validation on its nominal
/// bounds and so yields no verdict.
pub fn config_native_verdict(
    native: &NativePackage<'_>,
    model: &NativeModel,
    pre: i64,
    post: i64,
) -> Option<bool> {
    let stem = format!("ix://quire-integration/config-version/{pre}/{post}");
    let pre_snapshot = config_snapshot(
        model,
        ir::StateObservation::Pre,
        pre,
        &format!("{stem}/pre"),
    );
    let post_snapshot = config_snapshot(
        model,
        ir::StateObservation::Post,
        post,
        &format!("{stem}/post"),
    );
    let invocation = Invocation::new(
        source_identity(&format!("{stem}/invocation")),
        InvocationDraft {
            models: vec![ModelBinding {
                model: model.environment().owner().clone(),
                digest: model.digest(),
            }],
            context: QualifiedName {
                model: model.environment().owner().clone(),
                name: symbol("ConfigVersion"),
            },
            operation: symbol("attemptUpdate"),
            anchor: ir::AnchorName::new("attemptUpdate").unwrap(),
            self_object: config_object(model, "child"),
            pre: pre_snapshot.reference(),
            post: post_snapshot.reference(),
            parameters: Vec::new(),
            result: Some(ValueId::new(0)),
            created: Vec::new(),
            deleted: Vec::new(),
            arena: vec![ValueNode::Boolean { value: true }],
        },
        ArtifactLimits::default(),
    )
    .expect("attemptUpdate invocation is structurally valid");
    let binding = native.checked().clauses()[0].binding();
    let selection = ExecutionSelection {
        requirement: binding.requirement.clone(),
        clause: binding.clause.clone(),
        observation: ObservationSelection::Invocation {
            invocation: invocation.reference(),
        },
    };
    let input = RuntimeInput {
        snapshots: vec![pre_snapshot, post_snapshot],
        invocations: vec![invocation],
    };
    let report = runtime::execute(native, input, selection, ExecutionLimits::default(), || {
        false
    });
    if !CONFIG_DOMAIN.contains(&pre) || !CONFIG_DOMAIN.contains(&post) {
        let ExecutionOutcome::ValidationFailed(failure) = report.outcome() else {
            panic!("out-of-domain input pre={pre} post={post} reached native evaluation")
        };
        assert!(failure.diagnostics.iter().any(|diagnostic| {
            diagnostic.diagnostic.code == qsl_foundation::Code::InvalidRuntimeInput
                && diagnostic.diagnostic.message == "integer is outside its nominal bounds"
        }));
    }
    report.truth()
}

/// The one public function in a generated Rust source file.
pub fn public_function(source: &str) -> syn::ItemFn {
    let mut functions = public_functions(source);
    assert_eq!(functions.len(), 1, "one public generated function");
    functions.remove(0)
}

/// Every public free function in a generated Rust source file.
pub fn public_functions(source: &str) -> Vec<syn::ItemFn> {
    syn::parse_file(source)
        .expect("generated source is Rust")
        .items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) if matches!(function.vis, syn::Visibility::Public(_)) => {
                Some(function)
            }
            _ => None,
        })
        .collect()
}

/// Parameter names of a generated free function.
pub fn parameter_names(function: &syn::ItemFn) -> Vec<String> {
    function
        .sig
        .inputs
        .iter()
        .map(|argument| {
            let syn::FnArg::Typed(argument) = argument else {
                panic!("generated function is a free function")
            };
            let syn::Pat::Ident(name) = argument.pat.as_ref() else {
                panic!("generated function parameter is named")
            };
            name.ident.to_string()
        })
        .collect()
}

/// A Kani bundle for `clause` whose precondition is the canonical
/// zero-dependency Boolean `true`: the exact identity for an absent
/// authored precondition.
pub fn absent_precondition_kani_bundle(
    clause: &ir::BoundClause,
    precondition_clause: &str,
    proof_id: &str,
) -> codegen::KaniArtifactBundle {
    let truth = ir::Expression::new(
        ir::ExpressionKind::BooleanLiteral { value: true },
        clause.source().clone(),
    );
    let precondition = clause
        .environment()
        .check_expression(&truth, &ir::ValueType::Boolean, clause.anchor(), true)
        .expect("canonical absent precondition is a zero-dependency Boolean true");
    assert!(precondition.dependencies().is_empty());
    let precondition_clause = ir::ClauseId::new(precondition_clause).unwrap();
    codegen::generate_kani_bundle(&codegen::KaniRequest {
        ceilings: codegen::ProofCeilings {
            memory_bytes: NonZeroU64::new(4 * 1024 * 1024 * 1024).unwrap(),
            wall_clock: Duration::from_secs(300),
        },
        requirement: clause.identity().requirement(),
        precondition_clause: &precondition_clause,
        postcondition_clause: clause.identity().clause(),
        precondition: &precondition,
        postcondition: clause.expression(),
        proof_id,
        subject_path: "crate::subject",
        unwind: 2,
        solver: codegen::KaniSolver::Cadical,
        dependencies: &[],
    })
    .unwrap()
}

/// The `[dependencies]` line for proptest in generated crates.
pub const PROPTEST_DEPENDENCY: &str =
    "proptest = { version = \"1\", default-features = false, features = [\"std\"] }\n";

/// Git source of `quire-contract-runtime`, as codegen's generated manifests name it.
pub const RUNTIME_GIT_URL: &str = "https://github.com/agent-ix/quire-contract-runtime";

/// A single-crate workspace manifest that builds generated code against the
/// runtime source codegen's own generated manifests name. `dependencies` are extra
/// `[dependencies]` lines; `runtime_features` are enabled on
/// `quire-contract-runtime`.
pub fn runtime_manifest(name: &str, runtime_features: &[&str], dependencies: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\nquire-contract-runtime = {{ git = \"{RUNTIME_GIT_URL}\", branch = \"main\", features = {runtime_features:?} }}\n{dependencies}\n[workspace]\n",
    )
}

/// A throwaway crate for generated code, with its own target directory.
pub struct GeneratedCrate {
    directory: tempfile::TempDir,
    name: String,
}

impl GeneratedCrate {
    /// Create the crate with [`runtime_manifest`].
    pub fn new(name: &str, runtime_features: &[&str], dependencies: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("src")).unwrap();
        let generated = Self {
            directory,
            name: name.to_owned(),
        };
        generated.write(
            "Cargo.toml",
            &runtime_manifest(name, runtime_features, dependencies),
        );
        generated
    }

    /// The crate root.
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    /// Write one file relative to the crate root.
    pub fn write(&self, relative: &str, contents: &str) {
        fs::write(self.path().join(relative), contents).unwrap();
    }

    /// Private target directory of this crate.
    pub fn target(&self) -> PathBuf {
        self.path().join("target")
    }

    /// Run `cargo <subcommand>` with warnings denied; a non-empty `trailing`
    /// is passed after `--`.
    pub fn cargo(&self, subcommand: &str, trailing: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO"));
        command
            .args([subcommand, "--quiet", "--target-dir"])
            .arg(self.target())
            .env("RUSTFLAGS", "-Dwarnings")
            .current_dir(self.path());
        if !trailing.is_empty() {
            command.arg("--").args(trailing);
        }
        command.output().unwrap()
    }

    /// Build the binary and return its path.
    pub fn build_binary(&self) -> PathBuf {
        let output = self.cargo("build", &[]);
        assert!(
            output.status.success(),
            "generated crate {} did not compile:\n{}",
            self.name,
            String::from_utf8_lossy(&output.stderr)
        );
        self.target()
            .join("debug")
            .join(format!("{}{}", self.name, std::env::consts::EXE_SUFFIX))
    }
}
