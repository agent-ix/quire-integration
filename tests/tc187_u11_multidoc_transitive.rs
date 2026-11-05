// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 document-order and transitive-dimension first refusals.

use ix_trace_rs::trace;
use qsl_foundation::source::provenance::RawSourceRef;
use qsl_semantics::check::{
    AssemblyCause, AssemblyLimits, AssemblyRefusal, PackageDeclarations, TopologyFault,
    TopologyRetained,
};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{Cancel, Integer, Rational};
use quire_specification::{vectors_of, TestCase};

const U11Q_INPUTS: &str = "Document 1 declares `dimension T;` and `unit z : T = 0;`; document 2 declares `dimension L;` and `unit u_self : L = 1 * u_self;`; the request lists document 1 before document 2; then the request lists document 2 before document 1";
const U11Q_EXPECTED: &str = "the first run: `invalid_package`/`invalid-value` at `z` (`T` is processed first); the second run: `invalid_package`/`definition-cycle` retaining `u_self -> u_self` (`L` is processed first) (FR-142-AC-17)";
const U11X_INPUTS: &str = "`dimension L;` `unit m : L = 1;` `dimension E = D;` `dimension D = L * K;`; then the same with `unit e : E = 0;` added";
const U11X_EXPECTED: &str = "`missing_declaration`/`missing-name` retaining `D` and `K` at `D`'s turn (`E`'s turn does not look through `D`); then `invalid_package`/`invalid-value` at `e` (`E`'s turn comes first and its unit rows run) (FR-142-AC-17)";
const U11Z3_INPUTS: &str =
    "`dimension E = A;` `unit e : E = 0;` `dimension A = B;` `dimension B = A;`";
const U11Z3_EXPECTED: &str = "`invalid_package`/`invalid-value` at `e` (`E`'s turn is first; `E` is not a member of the cycle, so it is not refused for it, has no map, and its unit rows run); with `unit e` removed, `invalid_package`/`definition-cycle` at `A`, retaining `A -> B`, `B -> A` (FR-142-AC-17)";

fn vector(id: &str, inputs: &str, expected: &str) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    assert_eq!(row.inputs, inputs, "{id} full authored inputs");
    assert_eq!(row.expected, expected, "{id} full authored expectation");
}

fn source(declarations: &str) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    )
}

fn parsed_unit(identity: &str, source: &str) -> (RawSourceRef, qsl_forms::ParsedUnit) {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", identity),
        "tc187-u11.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    (
        parsed.source().reference().clone(),
        qsl_forms::build_unit(&parsed).expect("U11 source forms"),
    )
}

fn refusal(declarations: &str) -> (String, AssemblyRefusal) {
    let source = source(declarations);
    let (reference, unit) = parsed_unit("tc187-u11-transitive", &source);
    let result = PackageDeclarations::assemble(
        reference,
        "agent-ix/tc187-u11-transitive".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    );
    (source, result.expect_err("first U11 defect must refuse"))
}

fn assert_first(
    id: &str,
    source: &str,
    refusal: &AssemblyRefusal,
    cause: AssemblyCause,
    code: &str,
    selected: &str,
) {
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: exactly one first assembly refusal: {refusal:?}");
    };
    assert_eq!(error.cause, cause, "{id} retained first defect");
    assert_eq!(error.cause.catalog_code().to_string(), code, "{id} code");
    assert_eq!(
        &source[error.span.start..error.span.end],
        selected,
        "{id} selected span"
    );
}

fn zero_scale(name: &str) -> AssemblyCause {
    AssemblyCause::UnitGraphTopology {
        fault: TopologyFault::ZeroScale,
        retained: vec![
            TopologyRetained::Unit {
                name: name.to_owned(),
                key: None,
            },
            TopologyRetained::Scale(Rational::from_integer(Integer::zero())),
        ],
    }
}

fn edge(from: &str, to: &str) -> (String, String) {
    (from.to_owned(), to.to_owned())
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. Document order wins over offsets.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11q_document_order_selects_the_first_documents_defect() {
    vector("U11q", U11Q_INPUTS, U11Q_EXPECTED);
    let time = source(&format!(
        "{}dimension T; unit z : T = rational(0, 1);",
        " ".repeat(400)
    ));
    let length = source("dimension L; unit u_self : L = rational(1, 1) * u_self;");
    assert!(time.find("z :").expect("z") > length.find("u_self :").expect("u_self"));

    for (first, second, selected, cause, code) in [
        (
            ("time", &time),
            ("length", &length),
            "z",
            zero_scale("z"),
            "invalid_package/invalid-value",
        ),
        (
            ("length", &length),
            ("time", &time),
            "u_self",
            AssemblyCause::UnitGraphTopology {
                fault: TopologyFault::TargetCycle,
                retained: vec![TopologyRetained::Edge {
                    from: "u_self".to_owned(),
                    to: "u_self".to_owned(),
                }],
            },
            "invalid_package/definition-cycle",
        ),
    ] {
        let assembled = PackageDeclarations::assemble_units(
            "agent-ix/tc187-u11-multidoc".to_owned(),
            vec![
                parsed_unit(first.0, first.1),
                parsed_unit(second.0, second.1),
            ],
            SelectedModels::default(),
            Vec::new(),
            AssemblyLimits::default(),
            &Cancel::new(),
        );
        let refusal = assembled.expect_err("the first listed document must refuse");
        assert_first("U11q", first.1, &refusal, cause, code, selected);
    }
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. E's unit can precede D's term.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11x_transitive_unknown_term_and_earlier_unit_defect() {
    vector("U11x", U11X_INPUTS, U11X_EXPECTED);
    let declarations =
        "dimension L; unit m : L = rational(1, 1); dimension E = D; dimension D = L * K;";
    let (source, found) = refusal(declarations);
    assert_first(
        "U11x without e",
        &source,
        &found,
        AssemblyCause::UnresolvedDimensionTerm {
            dimension: "D".to_owned(),
            term: "K".to_owned(),
        },
        "missing_declaration/missing-name",
        "K",
    );

    let (source, found) = refusal(&format!("{declarations} unit e : E = rational(0, 1);"));
    assert_first(
        "U11x with e",
        &source,
        &found,
        zero_scale("e"),
        "invalid_package/invalid-value",
        "e",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-17. E's unit precedes A's cycle.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-17")]
#[test]
fn public_u11z3_transitive_cycle_follows_the_earlier_unit_defect() {
    vector("U11z3", U11Z3_INPUTS, U11Z3_EXPECTED);
    let declarations =
        "dimension E = A; unit e : E = rational(0, 1); dimension A = B; dimension B = A;";
    let (source, found) = refusal(declarations);
    assert_first(
        "U11z3 with e",
        &source,
        &found,
        zero_scale("e"),
        "invalid_package/invalid-value",
        "e",
    );

    let (source, found) = refusal("dimension E = A; dimension A = B; dimension B = A;");
    assert_first(
        "U11z3 without e",
        &source,
        &found,
        AssemblyCause::QuantityCycle {
            edges: vec![edge("A", "B"), edge("B", "A")],
        },
        "invalid_package/definition-cycle",
        "A",
    );
}
