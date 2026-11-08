// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real PlanRoot emission to the bounded selected-fact reader (FR-357-AC-9).

mod support;

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_ir::{
    read_checked_package, AdmittedEqualityReadErrorCode, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, NativeJoinBudget, SelectedExactIdentities, SelectedFactEqualityId,
    SelectedFactFieldId, SelectedFactIds, SelectedFactRows, SelectedFactsErrorCode,
};
use quire_exact::{Cancel, Integer, Presence, Rational, ValueType};

// An original model source for the representable record-field selection.
// The monitor subject remains the separate QSL-owned ConfigVersion model.
fn selected_record_document() -> Vec<u8> {
    let origin = |identity: &str| {
        serde_json::json!({"generated": {
            "generatorIdentity": identity,
            "generatorVersion": "1.0.0",
            "inputIdentities": [identity]
        }})
    };
    serde_json::to_vec(&serde_json::json!({
        "contractVersion": "2.0.0",
        "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": "ix://agent-ix/selected-facts-model/source",
                   "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": "agent-ix/selected-facts-model", "version": "1.0.0",
                    "mappingVersions": [], "profileVersions": []},
        "constructs": [{
            "kind": {"module": "agent-ix/selected-facts-model", "name": "record"},
            "moduleVersion": "1.0.0",
            "construct": {"identity": "none", "shape": "record", "members": {},
                          "meaning": "quire.meaning.model.record-value-type/v1"}
        }],
        "types": [{
            "identity": "ix://agent-ix/selected-facts-model/Snapshot", "displayName": "Snapshot",
            "kind": {"module": "agent-ix/selected-facts-model", "name": "record"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "origin": origin("ix://agent-ix/selected-facts-model/Snapshot"),
            "supertypes": [],
            "fields": [{
                "identity": "ix://agent-ix/selected-facts-model/Snapshot/count", "name": "count",
                "typeRef": "ix://quire/native/Integer", "presence": "required",
                "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
                "origin": origin("ix://agent-ix/selected-facts-model/Snapshot/count")
            }]
        }],
        "occurrences": [], "extensions": []
    }))
    .expect("the original selected model document serializes")
}

#[trace("TC-098", "FR-357-AC-9")]
#[test]
fn real_plan_root_rows_join_selected_model_and_equality_facts() {
    // The producer's own domain document is read from Cargo's resolved QSL
    // checkout by reference, with no copied or rewritten model fixture.
    let document =
        std::fs::read(support::qsl_root().join("examples/config-version/model.semantic-ir.json"))
            .expect("QSL owns the domain fixture");
    let domain: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = domain["package"]["identity"].as_str().unwrap();
    let selected_document = selected_record_document();
    let packages = package_input([document.as_slice(), selected_document.as_slice()])
        .expect("both original model documents admit");
    let text = format!(
        r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
model Facts = "agent-ix/selected-facts-model";
function selected_count using v(snapshot: Facts::Snapshot): Integer pure {{ snapshot.count }}
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit ms;
temporal SelectedFacts using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    always (holds(candidate.versionNumber = 7) implies eventually[0 ms, 4 ms] holds(candidate.versionNumber = 8))
}}
"#
    );
    let source = SourceIdentity::new("agent-ix", "selected-monitor-facts");
    let name = LibraryName::new("agent-ix/selected-monitor-facts").unwrap();
    let target = MonitorTarget {
        counter: "counter".to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: Rational::new(Integer::zero(), Integer::one()).unwrap(),
        max_event_rate: Some(Rational::new(Integer::from(2_u64), Integer::one()).unwrap()),
        max_reading_gap: 100,
        max_activations: 8,
    };
    let root = emit_plan_root(
        &PlanRootRequest {
            source: &source,
            path: "selected-monitor.native",
            bytes: text.as_bytes(),
            packages: &packages,
            package_name: &name,
            limits: SpineLimits::default(),
            clause: "SelectedFacts",
            target: &target,
            settings: &TimingObligationSettings::default(),
        },
        &Cancel::new(),
    )
    .expect("the real QSL producer returns a final PlanRoot emission");
    let emission = &root.emission;
    let mut evidence = emission.evidence().clone();
    evidence
        .insert_domain_package_document(identity, document)
        .expect("selected model evidence");
    evidence
        .insert_domain_package_document("agent-ix/selected-facts-model", selected_document)
        .expect("selected record model evidence");
    let package = match read_checked_package(
        emission.package().bytes(),
        CheckedPackageReadLimits {
            bytes: 8 * 1024 * 1024,
            nodes: 100_000,
            edges: 1_000_000,
            occurrences: 1_000_000,
            diagnostics: 1_000,
            work: 100_000_000,
        },
        &evidence,
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("IR admits the actual PlanRoot bytes: {other:?}"),
    };
    assert_eq!(package.monitor_plans().len(), 1);
    let declarations = emission.selected_exact_declaration_rows();
    let fields = emission.selected_exact_field_rows();
    assert!(!declarations.is_empty() && !fields.is_empty());
    SelectedExactIdentities::from_stage_rows(
        &package,
        declarations.iter().cloned(),
        fields.iter().cloned(),
        &mut NativeJoinBudget::new(1_000_000, 1_000_000),
    )
    .expect("IR joins exact original producer identities");
    let mut matching_fields = fields
        .iter()
        .filter(|row| row.field_original_identity.ends_with("/Snapshot/count"));
    let selected_field = matching_fields
        .next()
        .expect("select the model's original numeric field");
    assert!(
        matching_fields.next().is_none(),
        "the original field is unique"
    );
    let selected_declaration = declarations
        .iter()
        .find(|row| row.structural_node == selected_field.record_node)
        .expect("the selected field retains its producer declaration");
    let selected_declarations = [selected_field.record_node.clone()];
    let selected_fields = [SelectedFactFieldId {
        record: selected_field.record_node.clone(),
        original_identity: selected_field.field_original_identity.clone(),
    }];
    // The producer's public handoff adapter owns the exhaustive translation
    // and retains only witnesses for nodes in this final emitted package.
    let equality_rows = emission.equality_witness(&package);
    let applications = equality_rows.applications();
    let facts = equality_rows.facts();
    assert!(!applications.is_empty() && !facts.is_empty());
    for application in applications {
        let producer = emission
            .equality_producer_identity_rows()
            .iter()
            .find(|row| {
                row.checked_node() == &application.application
                    && row.occurrence() == &application.occurrence
            })
            .expect("the handoff application comes from this final emission");
        assert_eq!(application.source_map_entry, *producer.source_map_entry());
        assert_eq!(application.node_key, *producer.application());
        assert_eq!(application.location, *producer.location());
    }
    for fact in facts {
        let producer = emission
            .equality_fact_witness_rows()
            .iter()
            .find(|row| {
                row.checked_application() == &fact.checked_application
                    && row.occurrence() == &fact.occurrence
                    && row.structural_node() == &fact.structural_node
                    && row.source_map_entry() == &fact.source_map_entry
            })
            .expect("the handoff fact comes from this final emission");
        assert_eq!(fact.application, *producer.application());
        assert_eq!(fact.structural_key, *producer.structural_key());
        assert_eq!(fact.owner.as_ref(), producer.owner());
        assert_eq!(fact.value_type.as_ref(), producer.value_type());
    }
    assert_eq!(
        equality_rows.source_units(),
        emission.selected_source_unit_rows()
    );
    let rows = || SelectedFactRows {
        declarations,
        fields,
        source_units: emission.selected_source_unit_rows(),
        equality_applications: applications,
        equality_facts: facts,
    };
    let selected = package
        .project_selected_facts(
            SelectedFactIds {
                declarations: &selected_declarations,
                fields: &selected_fields,
                equalities: &[],
            },
            rows(),
            1_000_000,
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .expect("representable explicit model facts join");
    assert_eq!(
        selected.declarations.as_slice(),
        std::slice::from_ref(selected_declaration)
    );
    assert_eq!(selected.fields.len(), 1);
    assert_eq!(selected.fields[0].row, *selected_field);
    assert_eq!(selected.fields[0].name.as_ref(), "count");
    assert_eq!(selected.fields[0].order, 0);
    assert_eq!(selected.fields[0].presence, Presence::Required);
    assert_eq!(selected.fields[0].value_type, ValueType::Integer);
    let equality = &applications[0];
    let decision = package.admitted_equality(
        &equality.application,
        &equality.occurrence,
        applications,
        facts,
        emission.selected_source_unit_rows(),
        &mut NativeJoinBudget::new(1_000_000, 1_000_000),
    );
    let equality_ids = [SelectedFactEqualityId {
        application: equality.application.clone(),
        occurrence: equality.occurrence.clone(),
    }];
    let joined = package.project_selected_facts(
        SelectedFactIds {
            declarations: &selected_declarations,
            fields: &selected_fields,
            equalities: &equality_ids,
        },
        rows(),
        1_000_000,
        &mut NativeJoinBudget::new(1_000_000, 1_000_000),
    );
    match decision {
        Ok(decision) => {
            eprintln!("selected equality outcome: opaque native decision");
            let joined = joined.expect("the native equality also joins the selected read");
            assert_eq!(joined.declarations, selected.declarations);
            assert_eq!(joined.fields, selected.fields);
            assert_eq!(joined.equalities.len(), 1);
            assert_eq!(joined.equalities[0].id, equality_ids[0]);
            assert_eq!(joined.equalities[0].decision, decision);
        }
        Err(native) => {
            eprintln!("selected equality outcome: {:?}", native.code);
            assert_eq!(
                native.code,
                AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
            );
            assert_eq!(native.application.as_deref(), Some(&equality.application));
            assert!(
                native.witness.is_some(),
                "the missing witness has a native kind"
            );
            assert_eq!(native.attempted_work, 0);
            let refused = joined.expect_err("a missing native witness yields no partial view");
            assert_eq!(refused.code, SelectedFactsErrorCode::Equality);
            assert_eq!(Some(&refused.at), native.at.as_ref());
            assert_eq!(refused.occurrence.as_ref(), Some(&equality.occurrence));
            assert!(refused.exact.is_none() && refused.exact_code.is_none());
            let cause = refused.equality.expect("retain the native equality cause");
            assert_eq!(cause.code, native.code);
            assert_eq!(cause.application, native.application);
            assert_eq!(cause.at, native.at);
            assert_eq!(cause.witness, native.witness);
            assert_eq!(cause.materialization, native.materialization);
            assert_eq!(cause.attempted_work, native.attempted_work);
        }
    }
}
