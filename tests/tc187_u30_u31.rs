// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U30/U31 quantity type admission through QSL intake.

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_semantics::check::{AssemblyCause, AssemblyRefusal, CheckCause, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{UnitId, ValueType};
use quire_semantic_value::checking::CheckingLimits;
use quire_specification::{vectors_of, TestCase};

const TIME_UNITS: &str = "dimension T;\n\
    unit s : T = rational(1, 1);\n\
    unit ms : T = rational(1, 1000) * s;\n";

fn public_cell(id: &str, inputs: &str, expected: &str) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    assert_eq!(row.inputs, inputs, "{id} complete authored input");
    assert_eq!(row.expected, expected, "{id} complete expected cell");
}

fn source(declarations: &str) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         {TIME_UNITS}{declarations}"
    )
}

fn assemble(text: &str) -> Result<PackageDeclarations, AssemblyRefusal> {
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc187-u30-u31"),
        "tc187-u30-u31.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the selected public case parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("the selected public case forms");
    PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u30-u31".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
}

fn declared_ms(package: &PackageDeclarations, text: &str) -> UnitId {
    let keys: Vec<_> = package
        .nominal_spans
        .iter()
        .filter(|(_, span)| &text[span.start..span.end] == "ms")
        .map(|(key, _)| key.clone())
        .collect();
    let [key] = keys.as_slice() else {
        panic!("one declared ms unit node: {keys:?}");
    };
    let unit = package
        .units
        .unit(key.clone())
        .expect("the declared ms is in the admitted unit graph");
    assert_ne!(unit.root(), unit.key(), "ms is derived from s");
    unit.id().clone()
}

enum MissingNamespace {
    Unit,
    Type,
}

fn assert_assembly_missing(declarations: &str, name: &str, namespace: MissingNamespace) {
    let text = source(declarations);
    let refusal = assemble(&text).expect_err("the unit namespace miss refuses assembly");
    let [error] = refusal.errors.as_slice() else {
        panic!("one typed missing-name refusal for {name}: {refusal:?}");
    };
    match (namespace, &error.cause) {
        (
            MissingNamespace::Unit,
            AssemblyCause::UnresolvedUnitName {
                name: found,
                packages,
            },
        ) => {
            assert_eq!(found, name);
            assert!(packages.is_empty());
        }
        (MissingNamespace::Type, AssemblyCause::UnresolvedTypeName { name: found }) => {
            assert_eq!(found, name)
        }
        _ => panic!("original missing-name namespace for {name}: {error:?}"),
    }
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "missing_declaration/missing-name"
    );
    let start = text
        .rfind(name)
        .expect("the selected name is in the source");
    assert_eq!(
        (error.span.start, error.span.end),
        (start, start + name.len())
    );
}

fn assert_check_mismatch(declarations: &str, selected: &str) {
    let text = source(declarations);
    let package = assemble(&text).expect("the mismatched expression assembles");
    let regions = package.regions();
    let refusals = package
        .check(CheckingLimits::default())
        .expect_err("a different unit type refuses checking");
    let [refusal] = refusals.as_slice() else {
        panic!("one typed mismatch at {selected}: {refusals:?}");
    };
    assert!(matches!(&refusal.cause, CheckCause::IllTyped(_)));
    assert_eq!(
        refusal.cause.catalog_code().map(|code| code.to_string()),
        Some("ill_typed/type-mismatch".to_owned())
    );
    let region = regions
        .region(&refusal.location)
        .expect("the mismatch names a source region");
    let start = text
        .rfind(selected)
        .expect("the selected expression is in the source");
    assert_eq!(
        (region.start(), region.end()),
        (start as u64, (start + selected.len()) as u64)
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-11. Unit types and namespace misses.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-11")]
#[test]
fn public_u30_quantity_parameter_uses_ms_and_refuses_other_units_or_names() {
    public_cell(
        "U30",
        "Check `f(d: Quantity<ms>)` called as `f(3 ms)` and as `f(3 s)`, then `d: Quantity<T>`, `d: Quantity<nothing>` and a bare `d: ms` (FR-142-AC-11)",
        "`f(3 ms)` admitted; `f(3 s)` refused `ill_typed`/`type-mismatch` at the argument; each of the three unit-namespace misses refused `missing_declaration`/`missing-name` at the name; no charge on any check",
    );
    let f = "function f using v(d: Quantity<ms>): Boolean pure { true }\n";
    let accepted = source(&format!(
        "{f}function good using v(): Boolean pure {{ f(3 ms) }}"
    ));
    let package = assemble(&accepted).expect("f(3 ms) assembles");
    let ms = declared_ms(&package, &accepted);
    let checked = package
        .check(CheckingLimits::default())
        .expect("f(3 ms) checks without evaluation or charge");
    let callable = checked.callable("f").expect("f is publicly callable");
    match callable.parameters {
        [(name, ValueType::Quantity(unit))] if name == "d" => {
            assert_eq!(unit, &ms, "f's parameter names the declared ms node");
        }
        other => panic!("f's parameter is Quantity<ms>: {other:?}"),
    }

    assert_check_mismatch(
        &format!("{f}function bad using v(): Boolean pure {{ f(3 s) }}"),
        "3 s",
    );
    assert_assembly_missing(
        "function f using v(d: Quantity<T>): Boolean pure { true }",
        "T",
        MissingNamespace::Unit,
    );
    assert_assembly_missing(
        "function f using v(d: Quantity<nothing>): Boolean pure { true }",
        "nothing",
        MissingNamespace::Unit,
    );
    assert_assembly_missing(
        "function f using v(d: ms): Boolean pure { true }",
        "ms",
        MissingNamespace::Type,
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-12. The result retains ms.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-12")]
#[test]
fn public_u31_result_names_ms_and_refuses_a_converted_s_result() {
    public_cell(
        "U31",
        "Check `f(d: Quantity<ms>): Quantity<ms>` returning `d`, then returning `convert<Quantity<s>>(d)` (FR-142-AC-12)",
        "the first checks and names the node of `ms` as the type of `d` and the result; the second refused `ill_typed`/`type-mismatch` at the returned expression",
    );
    let returning =
        |body: &str| format!("function f using v(d: Quantity<ms>): Quantity<ms> pure {{ {body} }}");
    let accepted = source(&returning("d"));
    let package = assemble(&accepted).expect("returning d assembles");
    let ms = declared_ms(&package, &accepted);
    let checked = package
        .check(CheckingLimits::default())
        .expect("returning d checks");
    let callable = checked.callable("f").expect("f is publicly callable");
    let [(_, ValueType::Quantity(parameter_unit))] = callable.parameters else {
        panic!(
            "f's parameter resolves to a quantity: {:?}",
            callable.parameters
        );
    };
    let ValueType::Quantity(result_unit) = callable.result else {
        panic!("f's result resolves to a quantity: {:?}", callable.result);
    };
    assert_eq!(
        parameter_unit, result_unit,
        "parameter and result name one ms node"
    );
    assert_eq!(result_unit, &ms, "the result names the declared ms node");

    assert_check_mismatch(
        &returning("convert<Quantity<s>>(d)"),
        "convert<Quantity<s>>(d)",
    );
}
