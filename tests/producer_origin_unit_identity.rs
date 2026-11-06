// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-016 / TC-096: the exact consumer refuses an oversized producer unit identity.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_replay::{CompileUnit, ReplayLimits, StageLimits};
use quire_exact::{Integer, Meter, Outcome, Quantity, Rational, ScalarLimits};
use quire_semantic_value::{
    quantity::{convert_quantity, HandoffRefusal, QuantityTarget, QuantityUnit, UnitScope},
    semantic_node::IdentityRefusal,
    unit::{DimensionNode, NominalUnitForm, UnitGraph, UnitNode},
};

/// Thirty-two real roots share one normalized base dimension. Their producer
/// identities each retain the package name, so the combined target crosses
/// the native identity byte limit without a synthetic key.
fn source() -> String {
    let declarations = (0..32)
        .map(|index| {
            format!(
                "dimension Derived{index} = Length;\n\
                 unit root{index} : Derived{index} = rational(1, 1);\n"
            )
        })
        .collect::<String>();
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         dimension Length;\n\
         unit anchor : Length = rational(1, 1);\n\
         {declarations}\
         function keep using v(): Boolean pure {{ true }}\n"
    )
}

fn retained_graph(context: &qsl_replay::AdmittedContext) -> UnitGraph {
    let table = context.unit_table();
    let dimensions = context
        .dimensions()
        .map(|dimension| {
            let nominal = table
                .nominal_node(dimension.node())
                .expect("QSL held dimension");
            let NominalUnitForm::Dimension { terms, .. } = &nominal.form else {
                panic!("QSL dimension has dimension facts")
            };
            (
                nominal.key.clone(),
                DimensionNode::checked(terms.clone(), nominal.declaration.clone())
                    .expect("QSL checked the dimension"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let units = context
        .units()
        .map(|unit| {
            let nominal = table.nominal_node(unit.node()).expect("QSL held unit");
            let NominalUnitForm::Unit {
                dimension,
                target,
                scale,
                offset,
            } = &nominal.form
            else {
                panic!("QSL unit has unit facts")
            };
            (
                nominal.key.clone(),
                UnitNode::checked(
                    dimension.clone(),
                    target.clone(),
                    scale.clone(),
                    offset.clone(),
                    nominal.declaration.clone(),
                )
                .expect("QSL checked the unit"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    UnitGraph::from_checked_nodes(&dimensions, &units).expect("retained QSL graph is valid")
}

fn meter() -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: 100_000,
        result_units: 100_000,
    })
}

#[test]
#[trace("IT-016-SC-03", "IT-016-SC-05", "TC-096", "FR-275-AC-23")]
fn retained_qsl_roots_refuse_only_at_exact_identity_handoff() {
    let package_name = "p".repeat(16_777_216 / 32);
    let source = source();
    let compiled = qsl_replay::compile_admitted(
        CompileUnit {
            source: SourceIdentity::new("test", "it-016-retained-units"),
            package_name,
            path: "it-016-retained-units.native",
            bytes: source.as_bytes(),
        },
        [],
        &qsl_replay::spine::DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("QSL admits the authored source");
    let context = compiled.context();
    let mut roots = context
        .units()
        .filter(|unit| {
            unit.qualified_name()
                .last()
                .is_some_and(|name| name.as_str().starts_with("root"))
        })
        .map(|unit| unit.node().clone())
        .collect::<Vec<_>>();
    roots.sort();
    assert_eq!(roots.len(), 32);
    assert!(roots
        .iter()
        .all(|root| root.view().package() == context.package_name().as_str()));

    let graph = retained_graph(context);
    let terms = roots
        .into_iter()
        .enumerate()
        .map(|(index, root)| (root, Integer::from(if index < 16 { 1_i64 } else { -1_i64 })))
        .collect::<Vec<_>>();
    let target = QuantityUnit::Compound(
        graph
            .compound_unit(&terms)
            .expect("QSL roots form a compound target"),
    );
    assert!(target.dimension().is_dimensionless());
    let source_unit = QuantityUnit::Compound(graph.compound_unit(&[]).expect("empty unit"));
    let source_id = source_unit.id().expect("empty identity fits");
    let mut table = context.unit_table().clone();
    table.insert(source_unit).expect("empty source is held");
    let source = Quantity::new(Rational::from_integer(Integer::one()), source_id);
    let scope = UnitScope::new(&table);
    let mut meter = meter();
    let Outcome::Completed(conversion) = convert_quantity(
        scope.resolve(&source).expect("source resolves"),
        &target,
        &QuantityTarget::Exact,
        &mut meter,
    )
    .expect("normalized dimensions agree") else {
        panic!("exact conversion completes before identity encoding")
    };
    let before = meter.admission_count();
    let Err(HandoffRefusal::Identity(IdentityRefusal::InputBytes { bound, required })) =
        conversion.identified_quantity()
    else {
        panic!("the consumer's actual handoff refuses oversized identity bytes")
    };
    assert_eq!(bound, 16_777_216);
    assert!(required > bound);
    assert_eq!(meter.admission_count(), before);
    assert!(scope.into_formed().ids().next().is_none());
}
