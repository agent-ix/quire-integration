// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected public TC-187 U11 topology rows through QSL's public intake.

use ix_trace_rs::trace;
use qsl_semantics::check::{AssemblyCause, PackageDeclarations, TopologyFault, TopologyRetained};
use qsl_semantics::model::intake::SelectedModels;
use quire_specification::{vectors_of, TestCase};

fn assemble(
    declarations: &str,
) -> (
    String,
    Result<PackageDeclarations, qsl_semantics::check::AssemblyRefusal>,
) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-u11-topology"),
        "tc187-u11.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("U11 source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("U11 source forms");
    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-u11-topology".to_owned(),
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

fn edge(from: &str, to: &str) -> TopologyRetained {
    TopologyRetained::Edge {
        from: from.to_owned(),
        to: to.to_owned(),
    }
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

fn assert_refusal(
    id: &str,
    declarations: &str,
    fault: TopologyFault,
    retained: Vec<TopologyRetained>,
    code: &str,
    span: &str,
) {
    let (source, result) = assemble(declarations);
    let refusal = result.expect_err("the U11 topology must refuse before checking");
    let [error] = refusal.errors.as_slice() else {
        panic!("{id}: exactly one typed assembly error: {refusal:?}");
    };
    assert_eq!(
        error.cause,
        AssemblyCause::UnitGraphTopology { fault, retained },
        "{id}"
    );
    assert_eq!(error.cause.catalog_code().to_string(), code, "{id}");
    assert_eq!(&source[error.span.start..error.span.end], span, "{id}");
}

/// Trace: QSpec-TC-187 and QSpec-FR-142-AC-15. This proves only these five rows.
#[trace("QSpec-TC-187", "QSpec-FR-142-AC-15")]
#[test]
fn public_u11_target_refusals_retain_identity_and_source_order() {
    vector("U11b", "m : L", "m -> cm");
    assert_refusal(
        "U11b",
        "dimension L; unit m : L = rational(1, 1) * cm; unit cm : L = rational(1, 100) * m;",
        TopologyFault::TargetCycle,
        vec![edge("m", "cm"), edge("cm", "m")],
        "invalid_package/definition-cycle",
        "m",
    );

    vector("U11c", "nothing", "missing-name");
    assert_refusal(
        "U11c",
        "dimension L; unit m : L = rational(1, 1); unit x : L = rational(1, 1) * nothing;",
        TopologyFault::UnknownTarget,
        vec![
            unit("x"),
            TopologyRetained::TargetName("nothing".to_owned()),
        ],
        "missing_declaration/missing-name",
        "nothing",
    );

    vector("U11d", "cm : L", "incompatible-definition");
    assert_refusal(
        "U11d",
        "dimension L; dimension T; unit m : L = rational(1, 1); unit s : T = rational(1, 1); unit cm : L = rational(1, 100) * s;",
        TopologyFault::CrossDimensionTarget,
        vec![unit("cm"), TopologyRetained::TargetName("s".to_owned()), dimension("L"), dimension("T")],
        "invalid_package/incompatible-definition",
        "cm",
    );

    vector("U11f", "u_self", "u_self -> u_self");
    assert_refusal(
        "U11f",
        "dimension L; unit m : L = rational(1, 1); unit u_self : L = rational(1, 1) * u_self;",
        TopologyFault::TargetCycle,
        vec![edge("u_self", "u_self")],
        "invalid_package/definition-cycle",
        "u_self",
    );

    vector("U11l", "cm : L", "cm -> m");
    assert_refusal(
        "U11l",
        "dimension L; unit cm : L = rational(1, 100) * m; unit m : L = rational(1, 1) * cm;",
        TopologyFault::TargetCycle,
        vec![edge("cm", "m"), edge("m", "cm")],
        "invalid_package/definition-cycle",
        "cm",
    );
}
