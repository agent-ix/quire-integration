// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 imported unit topology through QSL model intake and assembly.

mod support;

use std::fs;

use ix_trace_rs::trace;
use qsl_semantics::{
    check::{AssemblyCause, AssemblyRefusal, PackageDeclarations, TopologyFault, TopologyRetained},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_exact::{Integer, Rational};
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const MODEL_IDENTITY: &str = "acme/orders";
const MODEL_SELECTION: &str = "model P = \"acme/orders\";";

fn vector(id: &str, inputs: &[&str], expectation: &str) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    for input in inputs {
        assert!(row.inputs.contains(input), "{id} input: {}", row.inputs);
    }
    assert!(
        row.expected.contains(expectation),
        "{id} expected: {}",
        row.expected
    );
}

fn model_document() -> Vec<u8> {
    let path = support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json");
    let document = fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let mut model: Value = serde_json::from_slice(&document).expect("QSL model fixture is JSON");
    assert_eq!(model["package"]["identity"], MODEL_IDENTITY);
    let origin = model["types"][0]["origin"].clone();
    let dimension = format!("ix://{MODEL_IDENTITY}/dimension/L");
    model["dimensions"] = json!([{
        "identity": dimension,
        "name": "L",
        "base": true,
        "terms": [],
        "origin": origin,
    }]);
    model["units"] = json!([{
        "identity": format!("ix://{MODEL_IDENTITY}/unit/m"),
        "name": "m",
        "dimension": dimension,
        "scale": "1",
        "offset": "0",
        "origin": origin,
    }]);
    serde_json::to_vec(&model).expect("the model document serializes")
}

fn refusal(declarations: &str) -> (String, AssemblyRefusal) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         {MODEL_SELECTION}\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-imported-topology"),
        "tc187-u11-imported-topology.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let document = model_document();
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document.as_slice()]).expect("the QSL model document has an identity"),
        ModelNormalizationLimits::default(),
    )
    .expect("the selected QSL model admits");
    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-imported-topology".to_owned(),
        unit,
        models,
        Vec::new(),
    );
    (
        source,
        assembled.expect_err("the imported unit topology must refuse"),
    )
}

fn first<'a>(id: &str, refusal: &'a AssemblyRefusal) -> &'a qsl_semantics::check::AssemblyError {
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: one first assembly refusal: {refusal:?}");
    };
    error
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-18. The imported root is not
/// rechecked; the local two-edge cycle is the selected defect.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-18")]
#[test]
fn public_u11s_local_cycle_over_imported_dimension_retains_both_edges() {
    vector("U11s", &["unit a", "unit b"], "definition-cycle");
    let (source, refusal) = refusal(
        "unit a : P::L = rational(1, 1) * b; \
         unit b : P::L = rational(1, 1) * a;",
    );
    let error = first("U11s", &refusal);
    assert_eq!(
        error.cause,
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::TargetCycle,
            retained: vec![
                TopologyRetained::Edge {
                    from: "a".to_owned(),
                    to: "b".to_owned(),
                },
                TopologyRetained::Edge {
                    from: "b".to_owned(),
                    to: "a".to_owned(),
                },
            ],
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/definition-cycle"
    );
    assert_eq!(&source[error.span.start..error.span.end], MODEL_SELECTION);
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-18. The two-root refusal retains
/// the imported root identity and the new local root independently.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-18")]
#[test]
fn public_u11t_local_root_conflicts_with_imported_root() {
    vector("U11t", &["m2"], "conflicting-definition");
    let (source, refusal) = refusal("unit m2 : P::L = rational(1, 1);");
    let error = first("U11t", &refusal);
    let AssemblyCause::UnitGraphTopology {
        fault: TopologyFault::TwoRoots,
        retained,
    } = &error.cause
    else {
        panic!("U11t must retain both roots: {error:?}");
    };
    let [TopologyRetained::Dimension {
        name: dimension,
        key: Some(dimension_key),
    }, TopologyRetained::Unit {
        name: imported,
        key: Some(imported_key),
    }, TopologyRetained::Unit {
        name: local,
        key: None,
    }] = retained.as_slice()
    else {
        panic!("U11t exact imported/local roots: {retained:?}");
    };
    assert_eq!(dimension, "ix://acme/orders/dimension/L");
    assert_eq!(imported, "ix://acme/orders/unit/m");
    assert_ne!(dimension_key, imported_key);
    assert_eq!(local, "m2");
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/conflicting-definition"
    );
    assert_eq!(&source[error.span.start..error.span.end], MODEL_SELECTION);
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-18. A local zero scale refuses
/// before the imported root can make the dimension look healthy.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-18")]
#[test]
fn public_u11u_zero_scale_precedes_imported_root() {
    vector("U11u", &["unit z"], "invalid-value");
    let (source, refusal) = refusal("unit z : P::L = rational(0, 1) * P::m;");
    let error = first("U11u", &refusal);
    assert_eq!(
        error.cause,
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::ZeroScale,
            retained: vec![
                TopologyRetained::Unit {
                    name: "z".to_owned(),
                    key: None,
                },
                TopologyRetained::Scale(Rational::from_integer(Integer::zero())),
            ],
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/invalid-value"
    );
    assert_eq!(&source[error.span.start..error.span.end], "z");
}
