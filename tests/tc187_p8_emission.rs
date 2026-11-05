// SPDX-License-Identifier: AGPL-3.0-or-later
//! U33 original literal and admitted model operation parameter types.

#[path = "support/tc187_p8_source.rs"]
mod source;

use ix_trace_rs::trace;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::{
    accounting::ModelNormalizationLimits,
    intake::{admit_unit, package_input, SelectedModels},
};
use quire_exact::ValueType;
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};

fn u33() {
    source::vector(
        "U33",
        "Lower `f(3 ms)` and a domain-package operation parameter `typeRef` naming `ms` (FR-142-AC-14)",
        "a `literal` of `value_kind` `quantity` typed by the node of `ms`, and a member type that is that node; the exact wire vectors are TC-233 QL-01 to QL-04 and TC-280 MU-01 to MU-06",
    );
}

#[trace("TC-107", "QSpec-TC-187", "QSpec-FR-142-AC-14")]
#[test]
fn original_call_literal_is_typed_by_the_declared_ms_node() {
    u33();
    let text = format!(
        "{}{}function f using v(d: Quantity<ms>): Boolean pure {{ true }}\n\
         function invoke using v(): Boolean pure {{ f(3 ms) }}",
        source::HEADER,
        source::UNITS,
    );
    let (raw, unit) = source::parsed("quantity-call", &text);
    let graph = PackageDeclarations::assemble(
        raw,
        "agent-ix/tc187-p8-literal".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("call assembles")
    .check(CheckingLimits::default())
    .expect("f(3 ms) checks");
    let admitted = source::emitted(graph, None);
    let nodes = &admitted.graph().nodes;
    let ms = nodes
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == "unit"
                && node.declaration.as_ref().is_some_and(|decl| {
                    decl.qualified_name
                        .last()
                        .is_some_and(|name| name.as_ref() == "ms")
                })
        })
        .expect("original declared ms node");
    let literals: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.body {
            quire_contract_ir::CheckedTerm::Literal {
                type_id,
                value_kind,
                value,
            } if serde_json::to_value(value_kind).expect("literal kind") == "quantity" => {
                Some((node, type_id, value))
            }
            _ => None,
        })
        .collect();
    let [(literal, type_id, value)] = literals.as_slice() else {
        panic!("one original quantity literal: {nodes:#?}");
    };
    assert_eq!(**value, json!("3"));
    assert_eq!(literal.semantic_type, ms.node_id);
    assert_eq!(**type_id, ms.node_id);
}

const MODEL: &str = "agent-ix/namespace-clock";
const TIMER: &str = "ix://agent-ix/namespace-clock/Timer";

fn origin() -> Value {
    json!({"source": {"sourceIdentity": "ix://agent-ix/namespace-clock/design", "path": "clock.qspec", "startLine": 1, "startColumn": 1}})
}

fn document() -> Vec<u8> {
    let operation = format!("{TIMER}/delay");
    let parameter = format!("{operation}/elapsed");
    serde_json::to_vec(&json!({
        "contractVersion": "2.0.0", "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": "ix://agent-ix/namespace-clock/design", "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": MODEL, "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
        "occurrences": [], "extensions": [], "populations": [],
        "constructs": [{"kind": {"module": MODEL, "name": "timer"}, "moduleVersion": "1.0.0", "construct": {"identity": "none", "shape": "record", "members": {}, "meaning": "quire.meaning.model.object-type/v1"}}],
        "types": [{"identity": TIMER, "displayName": "Timer", "kind": {"module": MODEL, "name": "timer"}, "roles": [], "origin": origin(), "constraints": [], "extensions": [], "unknownPolicy": "reject", "supertypes": [], "fields": [], "relationships": [],
            "operations": [{"identity": operation, "name": "delay", "params": [{"identity": parameter, "name": "elapsed", "typeRef": "ix://quire/unit/ms", "presence": "required", "nullable": false, "defaultKind": "none", "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}, "origin": origin()}], "pre": [], "post": [], "frame": {"modifies": [], "creates": [], "deletes": []}, "origin": origin()}]
        }]
    })).expect("original native domain document")
}

#[trace("TC-107", "QSpec-TC-187", "QSpec-FR-142-AC-14")]
#[test]
fn admitted_operation_parameter_names_the_original_ms_unit() {
    u33();
    let document = document();
    let text = format!(
        "{}profile m = \"quire.model.complete/v1\";\nmodel Clock = \"{MODEL}\";\n{}\
         function retain using v(d: Quantity<ms>): Quantity<ms> pure {{ d }}\n\
         pre Delay using m on Clock::Timer::delay {{ elapsed = elapsed }}",
        source::HEADER,
        source::UNITS,
    );
    let (raw, unit) = source::parsed("model-operation", &text);
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document.as_slice()]).expect("native model identity"),
        ModelNormalizationLimits::default(),
    )
    .expect("native domain intake");
    let owner = models[0]
        .view
        .type_identities()
        .iter()
        .find(|(key, _)| key.node == TIMER)
        .expect("admitted Timer owner")
        .1
        .clone();
    let graph = PackageDeclarations::assemble(
        raw,
        "agent-ix/tc187-p8-member".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("operation parameter resolves")
    .check(CheckingLimits::default())
    .expect("model operation checks");
    let retain = graph.callable("retain").expect("quantity function");
    let [(_, expected @ ValueType::Quantity(_))] = retain.parameters else {
        panic!("retain quantity parameter");
    };
    let operations: Vec<_> = graph
        .scope()
        .operations()
        .declared_by(owner)
        .iter()
        .filter(|operation| operation.name() == "delay")
        .collect();
    let [operation] = operations.as_slice() else {
        panic!("one original model operation: {operations:?}");
    };
    assert_eq!(
        operation.parameters(),
        &[("elapsed".to_owned(), expected.clone())]
    );
    let admitted = source::emitted(graph, Some((MODEL, &document)));
    let nodes = &admitted.graph().nodes;
    let ms = nodes
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == "unit"
                && node.declaration.as_ref().is_some_and(|decl| {
                    decl.qualified_name
                        .last()
                        .is_some_and(|name| name.as_ref() == "ms")
                })
        })
        .expect("source ms unit retained");
    assert!(
        nodes
            .iter()
            .any(|node| node.semantic_type == ms.node_id
                && node.semantic_form.as_ref() == "parameter"),
        "actual emitted model parameter type: {nodes:#?}"
    );
}
