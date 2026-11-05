// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U11 local values and derived dimensions through QSL intake.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_semantics::check::{
    AssemblyCause, AssemblyRefusal, PackageDeclarations, TopologyFault, TopologyRetained,
};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{Integer, NodeKey, Rational};
use quire_specification::{vectors_of, TestCase};

fn assemble(declarations: &str) -> (String, Result<PackageDeclarations, AssemblyRefusal>) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-local-values"),
        "tc187-u11-local-values.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-local-values".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    );
    (source, assembled)
}

fn vector(id: &str, input: &str, expectation: &str) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    assert!(row.inputs.contains(input), "{id} input: {}", row.inputs);
    assert!(
        row.expected.contains(expectation),
        "{id} expected: {}",
        row.expected
    );
}

fn rational(integer: i64) -> Rational {
    Rational::from_integer(Integer::from(integer))
}

fn assert_non_identity_root(public_input: &str, declarations: &str, scale: i64, offset: i64) {
    vector("U11e", public_input, "invalid-value");
    let (source, result) = assemble(declarations);
    let refusal = result.expect_err("a targetless nonidentity root must refuse");
    let [error] = refusal.errors.as_slice() else {
        panic!("one first root error: {refusal:?}");
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::NonIdentityRoot,
            retained: vec![
                TopologyRetained::Unit {
                    name: "m".to_owned(),
                    key: None,
                },
                TopologyRetained::Scale(rational(scale)),
                TopologyRetained::Offset(rational(offset)),
            ],
        }
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "invalid_package/invalid-value"
    );
    assert_eq!(&source[error.span.start..error.span.end], "m");
}

fn named_keys(package: &PackageDeclarations, source: &str) -> BTreeMap<String, NodeKey> {
    package
        .nominal_spans
        .iter()
        .map(|(key, span)| (source[span.start..span.end].to_owned(), key.clone()))
        .collect()
}

fn assert_derived_dimension(
    id: &str,
    declarations: &str,
    unit_name: &str,
    derived_name: &str,
    exponent: &str,
) {
    let (source, result) = assemble(declarations);
    let refusal = result.expect_err("the unanchored base dimension must refuse");
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: one first base-dimension error: {refusal:?}");
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnanchoredBaseDimension {
            dimension: "L".to_owned(),
            unit: unit_name.to_owned(),
        },
        "{id}"
    );
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "missing_declaration/missing-name",
        "{id}"
    );
    assert_eq!(&source[error.span.start..error.span.end], "L", "{id}");

    let anchored = declarations.replacen(
        "dimension L;",
        "dimension L; unit m : L = rational(1, 1);",
        1,
    );
    let (source, result) = assemble(&anchored);
    let package = result.expect("an anchored base dimension admits the same derived map");
    let keys = named_keys(&package, &source);
    let dimension = package
        .units
        .dimension(keys[derived_name].clone())
        .expect("the derived dimension is retained");
    let exponents: Vec<_> = dimension
        .exponents()
        .map(|(base, power)| (base, power.to_string()))
        .collect();
    assert_eq!(
        exponents,
        vec![(keys["L"].clone(), exponent.to_owned())],
        "{id}"
    );
    let derived_unit = package
        .units
        .unit(keys[unit_name].clone())
        .expect("the derived unit is admitted");
    assert_eq!(derived_unit.dimension(), dimension, "{id}");
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. Both authored U11e forms refuse.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11e_nonidentity_roots_retain_scale_and_offset() {
    assert_non_identity_root(
        "unit m : L = 2;",
        "dimension L; unit m : L = rational(2, 1);",
        2,
        0,
    );
    assert_non_identity_root(
        "unit m : L = 1 + 1;",
        "dimension L; unit m : L = rational(1, 1) + rational(1, 1);",
        1,
        1,
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-16. The base check precedes units.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-16")]
#[test]
fn public_u11g_derived_maps_refuse_unanchored_l_and_admit_anchored_controls() {
    vector("U11g1", "m2", "L^2");
    assert_derived_dimension(
        "U11g1",
        "dimension L; dimension Area = L ^ 2; unit m2 : Area = rational(1, 1);",
        "m2",
        "Area",
        "2",
    );

    vector("U11g2", "m3", "L^3");
    assert_derived_dimension(
        "U11g2",
        "dimension L; dimension Area = L ^ 2; dimension Vol = Area * L; unit m3 : Vol = rational(1, 1);",
        "m3",
        "Vol",
        "3",
    );

    vector("U11g4", "m4", "L^4");
    assert_derived_dimension(
        "U11g4",
        "dimension L; dimension Area = L ^ 2; dimension Vol = Area * Area; unit m4 : Vol = rational(1, 1);",
        "m4",
        "Vol",
        "4",
    );
}
