// SPDX-License-Identifier: AGPL-3.0-or-later
//! Prepare the authored package and runtime mutation for IT-011-SC-02.

mod support;

use std::{fs, ops::Range};

use qsl_foundation::SourceIdentity;
use qsl_semantics::model::intake::package_input;
use quire_contract_ir::{
    read_checked_package, ApplicationOperator, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedTerm,
};
use quire_exact::Cancel;
use serde_json::{json, Value};

const MUTANT_BODY: &str = "{ R::from(false) }";

fn source_offset(source: &str, line: usize, column: usize) -> Option<usize> {
    let mut offset = 0;
    for (index, text) in source.split_inclusive('\n').enumerate() {
        if index + 1 == line {
            return (column <= text.len()).then_some(offset + column);
        }
        offset += text.len();
    }
    None
}

fn mutate_runtime_helper_body(source: &str) -> Result<(String, Range<usize>), String> {
    let file =
        syn::parse_file(source).map_err(|error| format!("runtime Rust does not parse: {error}"))?;
    let helpers: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == "implies_short_circuit" => {
                Some(function)
            }
            _ => None,
        })
        .collect();
    let [helper] = helpers.as_slice() else {
        return Err(format!(
            "expected one implies_short_circuit helper, found {}",
            helpers.len()
        ));
    };
    if helper.block.stmts.is_empty() {
        return Err("implies_short_circuit has no parsed body".into());
    }
    syn::parse_str::<syn::Block>(MUTANT_BODY)
        .map_err(|error| format!("mutant body does not parse: {error}"))?;
    let start = helper.block.brace_token.span.open().start();
    let end = helper.block.brace_token.span.close().end();
    let start = source_offset(source, start.line, start.column)
        .ok_or("helper body start is outside runtime source")?;
    let end = source_offset(source, end.line, end.column)
        .ok_or("helper body end is outside runtime source")?;
    let body = source
        .get(start..end)
        .ok_or("helper body spans an invalid runtime source range")?;
    if !body.starts_with('{') || !body.ends_with('}') {
        return Err("parsed helper body does not match source braces".into());
    }
    let mut changed = source.to_owned();
    changed.replace_range(start..end, MUTANT_BODY);
    Ok((changed, start..end))
}

const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
profile v = \"quire.value.complete/v1\";\n\
model Config = \"example/config-version\" version \"1.0.0\" digest \"sha256-jcs:{digest}\";\n\
post AmountBounded using v on Config::ConfigVersion::attemptUpdate { \
amount < 1000 implies amount + 1 <= 1000 }";

fn model_document() -> Vec<u8> {
    let path = support::qsl_root().join("examples/config-version/model.semantic-ir.json");
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let mut document: Value = serde_json::from_slice(&bytes).expect("QSL example model is JSON");
    let owner = document["types"]
        .as_array_mut()
        .expect("model types")
        .iter_mut()
        .find(|item| item["identity"] == "ix://example/config-version/ConfigVersion")
        .expect("ConfigVersion owner type");
    owner["operations"][0]["params"] = json!([{
        "identity":"ix://example/config-version/ConfigVersion/attemptUpdate/amount",
        "name":"amount",
        "typeRef":"ix://example/config-version/VersionNumber",
        "presence":"required",
        "nullable":false,
        "defaultKind":"none",
        "origin":{"source":{"sourceIdentity":"ix://example/config-version/spec", "path":"spec.qspec", "startLine":1, "startColumn":1}},
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    serde_json::to_vec(&document).expect("derived document serializes")
}

struct AdmittedClause {
    package: CheckedPackageV2,
}

fn admitted_clause() -> AdmittedClause {
    let document = model_document();
    let packages = package_input([document.as_slice()]);
    let [(digest, _)] = packages.iter().collect::<Vec<_>>()[..] else {
        panic!("one authored domain document");
    };
    let source = SOURCE.replace("{digest}", &qsl_semantics::model::key::hex(digest));
    let source_id = SourceIdentity::new("agent-ix", "test:it011-clause", "fixture", "fixture:1");
    let limits = qsl_replay::spine::SpineLimits::default();
    let cancel = Cancel::new();
    let parsed = qsl_replay::spine::parse(
        &qsl_replay::spine::ParseRequest {
            source: &source_id,
            path: "program.native",
            bytes: source.as_bytes(),
        },
        limits.source,
        &cancel,
    )
    .expect("QSL parses authored postcondition")
    .into_value();
    let selected = qsl_replay::spine::select(&parsed, &packages, limits.model, &cancel)
        .expect("QSL admits model selection")
        .into_value();
    let checked = qsl_replay::spine::check(
        &parsed,
        &selected,
        &qsl_replay::spine::DependencyInput::default(),
        &qsl_replay::spine::LockEvidence::default(),
        limits,
        &cancel,
    )
    .expect("QSL checks authored postcondition")
    .into_value();
    let emitted = qsl_replay::spine::package(
        &checked,
        qsl_replay::spine::PackageLimits::default(),
        &cancel,
    )
    .expect("QSL emits exact checked package")
    .into_value();
    let bytes = emitted.package().bytes();
    let mut evidence = emitted.evidence().clone();
    evidence
        .insert_domain_package_document(qsl_semantics::model::key::hex(digest), document)
        .expect("exact model document evidence");
    match read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => AdmittedClause { package: *package },
        other => panic!("IR must admit exact QSL emitted package: {other:?}"),
    }
}

#[test]
fn qsl_authored_parameter_only_postcondition_emits_one_admitted_clause() {
    let admitted = admitted_clause();
    let package = &admitted.package;
    let clauses: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "state"
                && matches!(
                    package.term(&node.node_id),
                    Some(CheckedTerm::Application {
                        operator: ApplicationOperator::StateClause,
                        operation,
                        ..
                    }) if operation["member"]["clause"] == "postcondition"
                )
        })
        .collect();
    assert_eq!(clauses.len(), 1, "one QSL-authored postcondition clause");
    assert_eq!(clauses[0].semantic_form.as_ref(), "state_clause");
    assert!(
        !clauses[0].node_id.digest.is_empty(),
        "the clause has a checked identity"
    );
}

#[test]
fn runtime_helper_mutation_changes_only_one_parsed_body() {
    let root = support::package_root("quire-contract-runtime");
    let source = fs::read_to_string(root.join("src/operators.rs"))
        .expect("resolved authoritative runtime helper source");
    let (mutant, body) = mutate_runtime_helper_body(&source).expect("one parsed helper body");
    assert_eq!(&source[..body.start], &mutant[..body.start]);
    assert_eq!(
        &source[body.end..],
        &mutant[body.start + MUTANT_BODY.len()..]
    );
    assert_eq!(
        &mutant[body.start..body.start + MUTANT_BODY.len()],
        MUTANT_BODY
    );
    let absent = source.replace("fn implies_short_circuit", "fn absent_helper");
    assert!(mutate_runtime_helper_body(&absent).is_err());
    assert!(mutate_runtime_helper_body(&format!("{source}\n{source}")).is_err());
}
