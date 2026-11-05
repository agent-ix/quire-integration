// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 imported base dimensions through selected-model assembly.

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
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const P: &str = "acme/orders";
const Q: &str = "acme/clock";
const P_L: &str = "ix://acme/orders/dimension/L";
const P_AREA: &str = "ix://acme/orders/dimension/Area";
const Q_T: &str = "ix://acme/clock/dimension/T";
const P_SELECTION: &str = "model P = \"acme/orders\";";

fn vector(id: &str, inputs: &str, expected: &str) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    assert_eq!(row.inputs, inputs, "{id} exact authored input");
    assert_eq!(row.expected, expected, "{id} exact expected first result");
}

fn model_document(identity: &str, dimensions: Value, units: Value) -> Vec<u8> {
    let path = support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json");
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let fixture = String::from_utf8(bytes).expect("the QSL model fixture is UTF-8");
    let mut model: Value =
        serde_json::from_str(&fixture.replace(P, identity)).expect("QSL model fixture is JSON");
    assert_eq!(model["package"]["identity"], identity);
    let origin = model["types"][0]["origin"].clone();
    let mut dimensions = dimensions;
    for dimension in dimensions.as_array_mut().expect("dimension list") {
        dimension["origin"] = origin.clone();
    }
    let mut units = units;
    for unit in units.as_array_mut().expect("unit list") {
        unit["origin"] = origin.clone();
    }
    model["dimensions"] = dimensions;
    model["units"] = units;
    serde_json::to_vec(&model).expect("selected model serializes")
}

fn p_document(with_area: bool) -> Vec<u8> {
    let mut dimensions = vec![json!({
        "identity": P_L, "name": "L", "base": true, "terms": [],
    })];
    if with_area {
        dimensions.push(json!({
            "identity": P_AREA, "name": "Area", "base": false,
            "terms": [{"dimension": P_L, "exponent": "2"}],
        }));
    }
    model_document(P, json!(dimensions), json!([]))
}

fn q_document(with_root: bool) -> Vec<u8> {
    model_document(
        Q,
        json!([{"identity": Q_T, "name": "T", "base": true, "terms": []}]),
        if with_root {
            json!([{
                "identity": "ix://acme/clock/unit/s", "name": "s", "dimension": Q_T,
                "scale": "1", "offset": "0",
            }])
        } else {
            json!([])
        },
    )
}

fn assemble_refusal(
    declarations: &str,
    p_area: bool,
    q_root: Option<bool>,
) -> (String, AssemblyRefusal) {
    let q_selection = if q_root.is_some() {
        "model Q = \"acme/clock\";\n"
    } else {
        ""
    };
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         {P_SELECTION}\n{q_selection}{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-imported-base"),
        "tc187-u11-imported-base.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let p = p_document(p_area);
    let q = q_root.map(q_document);
    let documents = std::iter::once(p.as_slice()).chain(q.iter().map(Vec::as_slice));
    let models = admit_unit(
        &unit.selections().models,
        &package_input(documents).expect("selected model identities"),
        ModelNormalizationLimits::default(),
    )
    .expect("P and optional Q admit through public model intake");
    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-imported-base".to_owned(),
        unit,
        models,
        Vec::new(),
    );
    (
        source,
        assembled.expect_err("the unanchored imported base refuses"),
    )
}

fn first(id: &str, source: &str, refusal: &AssemblyRefusal, carrier: &str) {
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: exactly one first QSL refusal: {refusal:?}");
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnanchoredBaseDimension {
            dimension: P_L.to_owned(),
            unit: carrier.to_owned(),
        },
        "{id}: imported P.L and first local carrier"
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "missing_declaration/missing-name",
        "{id}: first refusal code"
    );
    assert_eq!(
        &source[error.span.start..error.span.end],
        P_SELECTION,
        "{id}: P model selection owns the imported base"
    );
}

/// Trace: QSpec-TC-187, QSpec-FR-142-AC-16.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-16")]
#[test]
fn public_u11y_imported_l_turn_precedes_local_area() {
    vector(
        "U11y",
        "Imported `dimension L;` with no unit in P; local `dimension Area = L ^ 2;` and `unit m2 : Area = 1;`",
        "`missing_declaration`/`missing-name` retaining `L` and `m2`, at `L`'s turn placed before `Area`'s declaration (FR-142-AC-16)",
    );
    let (source, refusal) = assemble_refusal(
        "dimension Area = P::L ^ 2; unit m2 : Area = rational(1, 1);",
        false,
        None,
    );
    first("U11y", &source, &refusal, "m2");
}

/// Trace: QSpec-TC-187, QSpec-FR-142-AC-16.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-16")]
#[test]
fn public_u11z1_imported_area_turn_precedes_imported_l() {
    vector(
        "U11z1",
        "Imported package P declares `dimension L;` and `dimension Area = L ^ 2;` with no unit; local `unit m2 : P.Area = 1;`",
        "`P.Area` takes its turn (reached by the unit's type) and passes; `P.L` takes its turn next (reached through `Area`'s terms): `missing_declaration`/`missing-name` retaining `P.L` and `m2` (FR-142-AC-16)",
    );
    let (source, refusal) = assemble_refusal("unit m2 : P::Area = rational(1, 1);", true, None);
    first("U11z1", &source, &refusal, "m2");

    let (source, refusal) = assemble_refusal(
        "unit m2 : P::Area = rational(1, 1); \
         unit m2_alt : P::Area = rational(1, 1);",
        true,
        None,
    );
    let [error] = refusal.errors.as_slice() else {
        panic!("U11z1 adverse P.Area must be first: {refusal:?}");
    };
    let AssemblyCause::UnitGraphTopology {
        fault: TopologyFault::TwoRoots,
        retained,
    } = &error.cause
    else {
        panic!("U11z1 adverse P.Area must precede unanchored P.L: {error:?}");
    };
    let [TopologyRetained::Dimension {
        name: dimension,
        key: Some(_),
    }, TopologyRetained::Unit {
        name: first_root,
        key: None,
    }, TopologyRetained::Unit {
        name: second_root,
        key: None,
    }] = retained.as_slice()
    else {
        panic!("U11z1 P.Area roots must be retained: {retained:?}");
    };
    assert_eq!(dimension, P_AREA);
    assert_eq!(first_root, "m2");
    assert_eq!(second_root, "m2_alt");
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/conflicting-definition"
    );
    assert_eq!(&source[error.span.start..error.span.end], P_SELECTION);
}

/// Trace: QSpec-TC-187, QSpec-FR-142-AC-18.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-18")]
#[test]
fn public_u11z2_imported_l_precedes_q_t_and_later_zero_scale() {
    vector(
        "U11z2",
        "`dimension X = P.L * Q.T;` `unit x : X = 1;` and, declared later, `unit t : Q.T = 0 * Q.s;`, with `P.L` unitless in P, `Q.T` a dimension of Q with unit `Q.s`",
        "the walk reaches `P.L` then `Q.T` at `X`'s declaration, before the later unit: `P.L`'s turn comes first, `missing_declaration`/`missing-name` retaining `P.L` and `x`; the zero scale of `t` is not reported (FR-142-AC-18)",
    );
    let (source, refusal) = assemble_refusal(
        "dimension X = P::L * Q::T; unit x : X = rational(1, 1); \
         unit t : Q::T = rational(0, 1) * Q::s;",
        false,
        Some(true),
    );
    first("U11z2", &source, &refusal, "x");

    let (source, refusal) = assemble_refusal(
        "dimension X = P::L * Q::T; unit x : X = rational(1, 1);",
        false,
        Some(false),
    );
    first("U11z2 competing unitless bases", &source, &refusal, "x");
}
