// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U11 dimension-term first refusals through QSL intake.

use ix_trace_rs::trace;
use qsl_semantics::check::{AssemblyCause, AssemblyRefusal, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use quire_specification::{vectors_of, TestCase};

fn vector(id: &str, inputs: &str, expected: &str) -> String {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    assert_eq!(row.inputs, inputs, "{id} authored inputs");
    assert_eq!(row.expected, expected, "{id} authored first refusal");
    row.inputs
        .replace('`', "")
        .replace(" = 1;", " = rational(1, 1);")
}

fn refusal(declarations: &str) -> (String, AssemblyRefusal) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-dimension-terms"),
        "tc187-u11-dimension-terms.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let result = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-dimension-terms".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    );
    (
        source,
        result.expect_err("a dimension-term defect must refuse"),
    )
}

fn assert_first(id: &str, declarations: &str, cause: AssemblyCause, code: &str, selected: &str) {
    let (source, refusal) = refusal(declarations);
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: exactly one first assembly refusal: {refusal:?}");
    };
    assert_eq!(error.cause, cause, "{id} retained first defect");
    assert_eq!(error.cause.catalog_code().to_string(), code, "{id} code");
    assert_eq!(
        &source[error.span.start..error.span.end],
        selected,
        "{id} span"
    );
}

fn edge(from: &str, to: &str) -> (String, String) {
    (from.to_owned(), to.to_owned())
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. A's term cycle beats its unit.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11n_two_dimension_cycle_retains_ordered_term_edges() {
    let declarations = vector(
        "U11n",
        "`dimension A = B;` `dimension B = A ^ 2;` `unit a : A = 1;`",
        "`invalid_package`/`definition-cycle` at `A`'s turn, retaining the term edges `A -> B`, `B -> A` from `A`, declared first; `a` is not examined (FR-142-AC-17)",
    );
    assert_first(
        "U11n",
        &declarations,
        AssemblyCause::QuantityCycle {
            edges: vec![edge("A", "B"), edge("B", "A")],
        },
        "invalid_package/definition-cycle",
        "A",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. D's own term is a self-cycle.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11o_self_cycle_refuses_at_owning_dimension() {
    let declarations = vector(
        "U11o",
        "`dimension L;` `dimension D = D * L;` `unit m : L = 1;`",
        "`invalid_package`/`definition-cycle` at `D`, retaining `D -> D`; `L` has a unit, so it is not refused (it is processed first and admitted) (FR-142-AC-17)",
    );
    assert_first(
        "U11o",
        &declarations,
        AssemblyCause::QuantityCycle {
            edges: vec![edge("D", "D")],
        },
        "invalid_package/definition-cycle",
        "D",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. D's unknown term wins at D.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11p_unknown_term_refuses_after_anchored_l() {
    let declarations = vector(
        "U11p",
        "`dimension L;` `dimension D = L * K;` `unit m : L = 1;`",
        "`L` is admitted at its turn; `D` then refuses `missing_declaration`/`missing-name` retaining `D` and the term `K` (FR-142-AC-17)",
    );
    assert_first(
        "U11p",
        &declarations,
        AssemblyCause::UnresolvedDimensionTerm {
            dimension: "D".to_owned(),
            term: "K".to_owned(),
        },
        "missing_declaration/missing-name",
        "K",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. Unknown term beats A/B cycle.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11w_unknown_term_precedes_two_dimension_cycle() {
    let declarations = vector(
        "U11w",
        "`dimension A = B * K;` `dimension B = A;`",
        "`missing_declaration`/`missing-name` retaining `A` and `K` (the unknown term is checked before the cycle) (FR-142-AC-17)",
    );
    assert_first(
        "U11w",
        &declarations,
        AssemblyCause::UnresolvedDimensionTerm {
            dimension: "A".to_owned(),
            term: "K".to_owned(),
        },
        "missing_declaration/missing-name",
        "K",
    );
}
