// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original model inputs and public intake for this selected-model cohort.

use qsl_package::CheckedPackage;
use qsl_semantics::{
    check::{AssemblyRefusal, PackageDeclarations},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};

pub const HEADER: &str = "language \"ix:native\" edition \"1-draft\"; profile v = \"quire.value.complete/v1\"; model R = \"acme/rpc\"; model F = \"acme/flow\"; dimension Time; unit s : Time = rational(1, 1);";
pub const RF: &str = "dimension R::Time ~ F::Time by 1 R::s = 1000 F::ms;";
pub const RG: &str = "dimension R::Time ~ G::Time by 1 R::s = 1000 G::ms;";

pub fn document(identity: &str, missing_mass_root: bool) -> Vec<u8> {
    let origin = json!({"generated": {"generatorIdentity": format!("ix://{identity}/quantity-input"), "generatorVersion": "1.0.0", "inputIdentities": [format!("ix://{identity}/quantity-input")]}});
    let dim = |name: &str| format!("ix://{identity}/dimension/{name}");
    let dimensions: Vec<_> = ["Time", "Length", "Mass"].into_iter().map(|name| json!({"identity": dim(name), "name": name, "base": true, "terms": [], "origin": origin})).collect();
    let mut units = Vec::new();
    for (name, dimension) in [
        (if identity == "acme/clock" { "ms" } else { "s" }, "Time"),
        ("m", "Length"),
        ("kg", "Mass"),
    ] {
        if missing_mass_root && dimension == "Mass" {
            continue;
        }
        units.push(json!({"identity": format!("ix://{identity}/unit/{name}"), "name": name, "dimension": dim(dimension), "scale": "1", "offset": "0", "origin": origin}));
    }
    if identity != "acme/rpc" {
        let (name, target, scale) = if identity == "acme/clock" {
            ("s", "ms", "1000")
        } else {
            ("ms", "s", "1/1000")
        };
        units.push(json!({"identity": format!("ix://{identity}/unit/{name}"), "name": name, "dimension": dim("Time"), "target": format!("ix://{identity}/unit/{target}"), "scale": scale, "offset": "0", "origin": origin}));
    }
    serde_json::to_vec(&json!({
        "contractVersion": "2.0.0", "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": format!("ix://{identity}/quantity-input"), "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": identity, "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
        "occurrences": [], "extensions": [],
        "constructs": [{"kind": {"module": identity, "name": "quantity_anchor"}, "moduleVersion": "1.0.0", "construct": {"identity": "none", "shape": "record", "members": {}, "meaning": "quire.meaning.model.object-type/v1"}}],
        "types": [{"identity": format!("ix://{identity}/QuantityAnchor"), "displayName": "QuantityAnchor", "kind": {"module": identity, "name": "quantity_anchor"}, "roles": [], "origin": origin, "constraints": [], "extensions": [], "unknownPolicy": "reject", "supertypes": [], "fields": [], "operations": []}],
        "dimensions": dimensions, "units": units,
    })).expect("original domain input serializes")
}

pub fn source(body: &str, clock: bool) -> String {
    let g = if clock {
        "model G = \"acme/clock\";"
    } else {
        ""
    };
    let (selections, declarations) = HEADER
        .split_once(" dimension Time;")
        .expect("original selection boundary");
    format!("{selections} {g} dimension Time;{declarations}\n{body}")
}

pub fn assemble(
    text: &str,
    missing_mass_root: bool,
) -> Result<PackageDeclarations, AssemblyRefusal> {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc113-model-values"),
        "model-values.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original source forms");
    let documents = [
        document("acme/rpc", false),
        document("acme/flow", missing_mass_root),
        document("acme/clock", false),
    ];
    let input =
        package_input(documents.iter().map(Vec::as_slice)).expect("original model identities");
    let selected = admit_unit(
        &unit.selections().models,
        &input,
        ModelNormalizationLimits::default(),
    )
    .expect("original models admit");
    PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc113-model-values".to_owned(),
        unit,
        selected,
        Vec::new(),
    )
}

pub fn checked(body: &str, clock: bool) -> CheckedPackage {
    CheckedPackage::link(
        assemble(&source(body, clock), false)
            .expect("original assembly")
            .check(CheckingLimits::default())
            .expect("original quantity source checks"),
    )
}

pub fn public_row(id: &str) -> &'static quire_specification::QualificationVector {
    let rows: Vec<_> = quire_specification::vectors_of(quire_specification::TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("one public {id}: {rows:?}");
    };
    row
}

pub fn bind(id: &str, inputs: &[&str], expected: &[&str]) {
    let row = public_row(id);
    for text in inputs {
        assert!(
            row.inputs.contains(text),
            "{id} original input {text}: {}",
            row.inputs
        );
    }
    for text in expected {
        assert!(
            row.expected.contains(text),
            "{id} original expected {text}: {}",
            row.expected
        );
    }
}

pub fn imported_unit(
    package: &PackageDeclarations,
    identity: &str,
    name: &str,
) -> quire_semantic_value::unit::Unit {
    package
        .units
        .units()
        .find(|unit| {
            let actual: Value =
                serde_json::to_value(qsl_semantics::check::wire_id::structural_id(&unit.key()))
                    .expect("actual original ID");
            actual["path"]
                == json!([
                    "scalar_type",
                    "unit",
                    format!("ix://{identity}/unit/{name}")
                ])
        })
        .expect("original imported unit")
        .clone()
}
