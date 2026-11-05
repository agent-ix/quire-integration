// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 U11 first-defect ordering through QSL intake.

use ix_trace_rs::trace;
use qsl_semantics::check::{
    AssemblyCause, AssemblyRefusal, PackageDeclarations, TopologyFault, TopologyRetained,
};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{Integer, Rational};
use quire_specification::{vectors_of, TestCase};

fn vector(id: &str, inputs: &[&str], expectations: &[&str]) {
    let matches: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = matches.as_slice() else {
        panic!("exactly one public TC-187 row {id}: {matches:?}");
    };
    for input in inputs {
        assert!(row.inputs.contains(input), "{id} input: {}", row.inputs);
    }
    for expectation in expectations {
        assert!(
            row.expected.contains(expectation),
            "{id} expected: {}",
            row.expected
        );
    }
}

fn refusal(declarations: &str) -> (String, AssemblyRefusal) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-order"),
        "tc187-u11-order.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let result = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-order".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    );
    (
        source,
        result.expect_err("a first U11 ordering defect must refuse"),
    )
}

fn unit(name: &str) -> TopologyRetained {
    TopologyRetained::Unit {
        name: name.to_owned(),
        key: None,
    }
}

fn dimension(name: &str) -> TopologyRetained {
    TopologyRetained::Dimension {
        name: name.to_owned(),
        key: None,
    }
}

fn edge(from: &str, to: &str) -> TopologyRetained {
    TopologyRetained::Edge {
        from: from.to_owned(),
        to: to.to_owned(),
    }
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

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. Scale is checked before target.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11h_zero_scale_precedes_unknown_target_in_one_unit() {
    vector(
        "U11h",
        &["unit z : L = 0 * nothing;"],
        &["`invalid_package`/`invalid-value`", "row 1 before row 2"],
    );
    assert_first(
        "U11h",
        "dimension L; unit m : L = rational(1, 1); unit z : L = rational(0, 1) * nothing;",
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::ZeroScale,
            retained: vec![
                unit("z"),
                TopologyRetained::Scale(Rational::from_integer(Integer::zero())),
            ],
        },
        "invalid_package/invalid-value",
        "z",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. Unit order beats row order.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11i_earlier_unknown_target_precedes_later_zero_scale() {
    vector(
        "U11i",
        &[
            "unit a : L = 1 * nothing;",
            "declared before",
            "unit b : L = 0 * m;",
        ],
        &[
            "`missing_declaration`/`missing-name`",
            "naming `a`",
            "earliest unit",
        ],
    );
    assert_first(
        "U11i",
        "dimension L; unit m : L = rational(1, 1); \
         unit a : L = rational(1, 1) * nothing; unit b : L = rational(0, 1) * m;",
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::UnknownTarget,
            retained: vec![
                unit("a"),
                TopologyRetained::TargetName("nothing".to_owned()),
            ],
        },
        "missing_declaration/missing-name",
        "nothing",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. Two roots precede a self-cycle.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11j_two_roots_precede_self_cycle() {
    vector(
        "U11j",
        &[
            "unit m_alt : L = 1;",
            "unit u_self : L = 1 * u_self;",
            "unit m : L = 1;",
        ],
        &[
            "`invalid_package`/`conflicting-definition`",
            "row 4 before row 6",
        ],
    );
    assert_first(
        "U11j",
        "dimension L; unit m : L = rational(1, 1); unit m_alt : L = rational(1, 1); \
         unit u_self : L = rational(1, 1) * u_self;",
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::TwoRoots,
            retained: vec![dimension("L"), unit("m"), unit("m_alt")],
        },
        "invalid_package/conflicting-definition",
        "m_alt",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. Dimension order beats unit row.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11k_earlier_dimension_cycle_precedes_later_zero_scale() {
    vector(
        "U11k",
        &[
            "Dimension `L` declared before `T`",
            "unit u_self : L = 1 * u_self;",
            "with no other unit",
            "unit z : T = 0;",
        ],
        &[
            "`invalid_package`/`definition-cycle`",
            "for `L`",
            "`T` is not examined",
        ],
    );
    assert_first(
        "U11k",
        "dimension L; dimension T; unit u_self : L = rational(1, 1) * u_self; \
         unit z : T = rational(0, 1);",
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::TargetCycle,
            retained: vec![edge("u_self", "u_self")],
        },
        "invalid_package/definition-cycle",
        "u_self",
    );
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-16. Earlier dimension wins.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-16")]
#[test]
fn public_u11m_dimension_order_selects_cycle_or_unanchored_base() {
    vector(
        "U11m",
        &[
            "`dimension Area = L ^ 2;` `dimension L;`",
            "`unit a : Area = 1 * b;`",
            "`unit b : Area = 1 * a;`",
            "then `dimension L;` `dimension Area = L ^ 2;`",
            "the same units",
        ],
        &[
            "`invalid_package`/`definition-cycle` at `Area`",
            "`a -> b`, `b -> a`",
            "`Area` is processed first",
            "`missing_declaration`/`missing-name` at `L`",
            "`L` is processed first",
        ],
    );
    let units = "unit a : Area = rational(1, 1) * b; unit b : Area = rational(1, 1) * a;";
    assert_first(
        "U11m Area first",
        &format!("dimension Area = L ^ 2; dimension L; {units}"),
        AssemblyCause::UnitGraphTopology {
            fault: TopologyFault::TargetCycle,
            retained: vec![edge("a", "b"), edge("b", "a")],
        },
        "invalid_package/definition-cycle",
        "a",
    );
    assert_first(
        "U11m L first",
        &format!("dimension L; dimension Area = L ^ 2; {units}"),
        AssemblyCause::UnanchoredBaseDimension {
            dimension: "L".to_owned(),
            unit: "a".to_owned(),
        },
        "missing_declaration/missing-name",
        "L",
    );
}
