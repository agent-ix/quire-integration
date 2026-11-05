// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real PlanRoot emission to the bounded selected-fact reader (FR-357-AC-9).

mod support;

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_package::{EqualityFactWitness as ProducerFact, EqualityFactWitnessRow};
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_ir::{
    read_checked_package, AdmittedEqualityReadErrorCode, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, EqualityDimensionRole, EqualityFactWitness, EqualityOperandTypeRole,
    EqualityUnitRole, SelectedEqualityApplicationRow, SelectedEqualityFactRow,
    SelectedExactIdentities, SelectedFactEqualityId, SelectedFactFieldId, SelectedFactIds,
    SelectedFactRows, SelectedFactsErrorCode,
};
use quire_exact::{Cancel, Integer, IntegerInterval, Presence, Rational, ValueType};

fn operand_role(role: qsl_package::EqualityOperandTypeRole) -> EqualityOperandTypeRole {
    match role {
        qsl_package::EqualityOperandTypeRole::Source => EqualityOperandTypeRole::Source,
        qsl_package::EqualityOperandTypeRole::ConversionTarget => {
            EqualityOperandTypeRole::ConversionTarget
        }
    }
}

// Translate the existing closed producer/reader vocabularies without deriving
// identities, discarding fields, or reconstructing checker decisions.
fn fact(row: &EqualityFactWitnessRow) -> SelectedEqualityFactRow {
    let witness = match row.fact() {
        ProducerFact::OperandType { position, role } => EqualityFactWitness::OperandType {
            position: *position,
            role: operand_role(*role),
        },
        ProducerFact::TextPlanType { position, role } => EqualityFactWitness::TextPlanType {
            position: *position,
            role: operand_role(*role),
        },
        ProducerFact::Unit {
            unit_id,
            quantity_type,
            operand_position,
            role,
        } => EqualityFactWitness::Unit {
            unit_id: unit_id.clone(),
            quantity_type: quantity_type.clone(),
            operand_position: *operand_position,
            role: match role {
                qsl_package::EqualityUnitRole::Source => EqualityUnitRole::Source,
                qsl_package::EqualityUnitRole::ConversionTarget => {
                    EqualityUnitRole::ConversionTarget
                }
            },
        },
        ProducerFact::DimensionDeclaration {
            unit_id,
            operand_position,
            role,
            ordinal,
        } => EqualityFactWitness::DimensionDeclaration {
            unit_id: unit_id.clone(),
            operand_position: *operand_position,
            ordinal: *ordinal,
            role: match role {
                qsl_semantics::check::EqualityDimensionRole::UnitDimension => {
                    EqualityDimensionRole::UnitDimension
                }
                qsl_semantics::check::EqualityDimensionRole::ConversionDimension => {
                    EqualityDimensionRole::ConversionDimension
                }
                qsl_semantics::check::EqualityDimensionRole::CompoundBase => {
                    EqualityDimensionRole::CompoundBase
                }
                qsl_semantics::check::EqualityDimensionRole::BaseDimension => {
                    EqualityDimensionRole::BaseDimension
                }
            },
        },
        ProducerFact::DimensionTerm {
            unit_id,
            operand_position,
            ordinal,
            base,
            base_node,
            exponent,
        } => EqualityFactWitness::DimensionTerm {
            unit_id: unit_id.clone(),
            operand_position: *operand_position,
            ordinal: *ordinal,
            base: base.clone(),
            base_node: base_node.clone(),
            exponent: exponent.clone(),
        },
        ProducerFact::ConversionEdge {
            unit_id,
            operand_position,
            ordinal,
            target,
            target_node,
            scale,
            offset,
        } => EqualityFactWitness::ConversionEdge {
            unit_id: unit_id.clone(),
            operand_position: *operand_position,
            ordinal: *ordinal,
            target: target.clone(),
            target_node: target_node.clone(),
            scale: scale.clone(),
            offset: offset.clone(),
        },
        ProducerFact::EnumMember {
            source_declaration,
            source_declaration_node,
            declaration,
            declaration_node,
            member,
            member_node,
            case,
            ordinal,
            ordered,
        } => EqualityFactWitness::EnumMember {
            source_declaration: source_declaration.clone(),
            source_declaration_node: source_declaration_node.clone(),
            declaration: declaration.clone(),
            declaration_node: declaration_node.clone(),
            member: member.clone(),
            member_node: member_node.clone(),
            case: case.clone(),
            ordinal: *ordinal,
            ordered: *ordered,
        },
    };
    SelectedEqualityFactRow {
        application: row.application().clone(),
        checked_application: row.checked_application().clone(),
        occurrence: row.occurrence().clone(),
        source_map_entry: row.source_map_entry().clone(),
        structural_key: row.structural_key().clone(),
        structural_node: row.structural_node().clone(),
        owner: row.owner().cloned(),
        value_type: row.value_type().cloned(),
        fact: witness,
    }
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
    let packages = package_input([document.as_slice()]).expect("the authored model admits");
    let text = format!(
        r#"language "ix:native" edition "1-draft";
profile values = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
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
    let package = match read_checked_package(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
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
    )
    .expect("IR joins exact original producer identities");
    let mut matching_fields = fields
        .iter()
        .filter(|row| row.field_original_identity.ends_with("/versionNumber"));
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
    let applications: Vec<_> = emission
        .equality_producer_identity_rows()
        .iter()
        .map(|row| SelectedEqualityApplicationRow {
            application: row.checked_node().clone(),
            occurrence: row.occurrence().clone(),
            source_map_entry: row.source_map_entry().clone(),
            node_key: row.application().clone(),
            location: row.location().clone(),
        })
        .collect();
    let facts: Vec<_> = emission
        .equality_fact_witness_rows()
        .iter()
        .map(fact)
        .collect();
    assert!(!applications.is_empty() && !facts.is_empty());
    let rows = || SelectedFactRows {
        declarations,
        fields,
        source_units: emission.selected_source_unit_rows(),
        equality_applications: &applications,
        equality_facts: &facts,
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
        )
        .expect("representable explicit model facts join");
    assert_eq!(selected.declarations, [selected_declaration.clone()]);
    assert_eq!(selected.fields.len(), 1);
    assert_eq!(selected.fields[0].row, *selected_field);
    assert_eq!(selected.fields[0].name.as_ref(), "versionNumber");
    assert_eq!(selected.fields[0].order, 0);
    assert_eq!(selected.fields[0].presence, Presence::Required);
    assert_eq!(
        selected.fields[0].value_type,
        ValueType::Int(IntegerInterval::spanning(
            Integer::zero(),
            Integer::from(1000_u64)
        ))
    );
    let equality = &applications[0];
    let decision = package.admitted_equality(
        &equality.application,
        &equality.occurrence,
        &applications,
        &facts,
        emission.selected_source_unit_rows(),
        1_000_000,
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
    );
    match decision {
        Ok(decision) => {
            let joined = joined.expect("the native equality also joins the selected read");
            assert_eq!(joined.declarations, selected.declarations);
            assert_eq!(joined.fields, selected.fields);
            assert_eq!(joined.equalities.len(), 1);
            assert_eq!(joined.equalities[0].id, equality_ids[0]);
            assert_eq!(joined.equalities[0].decision, decision);
        }
        Err(native) => {
            assert_eq!(
                native.code,
                AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
            );
            assert_eq!(*native.application, equality.application);
            assert!(
                native.witness.is_some(),
                "the missing witness has a native kind"
            );
            assert_eq!(native.attempted_work, 0);
            let refused = joined.expect_err("a missing native witness yields no partial view");
            assert_eq!(refused.code, SelectedFactsErrorCode::Equality);
            assert_eq!(refused.at, native.at);
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
