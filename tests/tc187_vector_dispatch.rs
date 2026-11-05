// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public TC-187 vector dispatch and real QSL admission controls for IT-019.

use std::collections::BTreeSet;

use ix_trace_rs::trace;
use qsl_semantics::check::{
    AssemblyCause, AssemblyRefusal, PackageDeclarations, TopologyFault, TopologyRetained,
};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{Integer, Rational};
use quire_specification::{vectors_of, QualificationVector, TestCase};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionCase {
    ZeroScale,
    TwoRoots,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowDispatch {
    Admission(AdmissionCase),
    Topology(&'static str),
    Admitted(&'static str),
    Planned {
        tickets: &'static [&'static str],
        missing: &'static str,
    },
}

// This is dispatch over public IDs, not a copy of QSpec's inputs or expected
// prose. A newly exported ID must get its own disposition instead of silently
// inheriting a passing row. Mixed rows remain Planned until split by operation.
fn dispatch(id: &str) -> Option<RowDispatch> {
    let pending = |missing| {
        Some(RowDispatch::Planned {
            tickets: &["IR-842"],
            missing,
        })
    };
    match id {
        "U06" => Some(RowDispatch::Admission(AdmissionCase::ZeroScale)),
        "U11a" => Some(RowDispatch::Admission(AdmissionCase::TwoRoots)),
        "U11b" => Some(RowDispatch::Topology("definition-cycle")),
        "U11c" => Some(RowDispatch::Topology("missing-name")),
        "U11d" => Some(RowDispatch::Topology("incompatible-definition")),
        "U11e" => Some(RowDispatch::Topology("invalid-value")),
        "U11f" => Some(RowDispatch::Topology("definition-cycle")),
        "U11g1" | "U11g2" | "U11g4" => Some(RowDispatch::Topology("missing-name")),
        "U11h" => Some(RowDispatch::Topology("invalid-value")),
        "U11i" => Some(RowDispatch::Topology("missing-name")),
        "U11j" => Some(RowDispatch::Topology("conflicting-definition")),
        "U11k" | "U11m" => Some(RowDispatch::Topology("definition-cycle")),
        "U11l" => Some(RowDispatch::Topology("definition-cycle")),
        "U11s" => Some(RowDispatch::Topology("definition-cycle")),
        "U11t" => Some(RowDispatch::Topology("conflicting-definition")),
        "U11u" => Some(RowDispatch::Topology("invalid-value")),
        "U11r" => Some(RowDispatch::Admitted("admitted")),
        "U11n" | "U11o" => Some(RowDispatch::Topology("definition-cycle")),
        "U11p" | "U11w" => Some(RowDispatch::Topology("missing-name")),
        "U11q" | "U11z3" => Some(RowDispatch::Topology("invalid-value")),
        "U11x" => Some(RowDispatch::Topology("missing-name")),
        "U01" => Some(RowDispatch::Planned {
            tickets: &["QSL-1737", "IR-1345"],
            missing: "public checked-package BoundedQuantity emission and source-to-CG agreement for admitted carried_quantity operations",
        }),
        "U11g3" => Some(RowDispatch::Planned {
            tickets: &["QSL-1738"],
            missing: "semantic-value derived dimension N = L / L must have an empty map, not N^1",
        }),
        "U02" | "U03" | "U04" | "U05" | "U07" | "U08" | "U09" | "U09b" | "U10" => {
            pending("QSL/IR/CG/native per-operation dispatch and comparison")
        }
        "U11y" | "U11z1" | "U11z2" => {
            pending("QSL admission and mixed-operation disposition")
        }
        "U12" | "U13" | "U14" | "U15" | "U16" | "U17" | "U18" | "U19" | "U20" | "U21" | "U22"
        | "U23" | "U24" | "U25" | "U26" | "U27" | "U28" | "U29" => {
            pending("QSL/IR/CG/native quantity operation and limit dispatch")
        }
        "U30" | "U31" | "U32" | "U33" | "U34" | "U35" | "U36" | "U37" | "U38" | "U39" | "U40" => {
            pending("QSL model/namespace admission and generated operation dispatch")
        }
        _ => None,
    }
}

fn assemble_refusal(declarations: &str) -> (String, AssemblyRefusal) {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{declarations}"
    );
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "tc187-admission"),
        "tc187-admission.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the authored source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("the authored source forms");
    let refusal = match PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-admission".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    ) {
        Ok(_) => panic!("the selected TC-187 admission control must refuse"),
        Err(refusal) => refusal,
    };
    (source, refusal)
}

fn assert_admission(vector: &QualificationVector, case: AdmissionCase) {
    let (declarations, fault, retained, cause, expected_fragment, selected_name) = match case {
        AdmissionCase::ZeroScale => (
            "dimension L; unit m : L = rational(1, 1); \
             unit zero : L = rational(0, 1) * m;",
            TopologyFault::ZeroScale,
            vec![
                TopologyRetained::Unit {
                    name: "zero".to_owned(),
                    key: None,
                },
                TopologyRetained::Scale(Rational::from_integer(Integer::zero())),
            ],
            "invalid_package/invalid-value",
            "invalid-value",
            "zero",
        ),
        AdmissionCase::TwoRoots => (
            "dimension L; unit m : L = rational(1, 1); \
             unit m_alt : L = rational(1, 1);",
            TopologyFault::TwoRoots,
            vec![
                TopologyRetained::Dimension {
                    name: "L".to_owned(),
                    key: None,
                },
                TopologyRetained::Unit {
                    name: "m".to_owned(),
                    key: None,
                },
                TopologyRetained::Unit {
                    name: "m_alt".to_owned(),
                    key: None,
                },
            ],
            "invalid_package/conflicting-definition",
            "conflicting-definition",
            "m_alt",
        ),
    };
    assert!(vector.inputs.contains(selected_name), "{} input", vector.id);
    assert!(
        vector.expected.contains(expected_fragment),
        "{} owner expectation",
        vector.id
    );
    let (source, refusal) = assemble_refusal(declarations);
    let [error] = refusal.errors.as_slice() else {
        panic!("{} must have one typed QSL refusal: {refusal:?}", vector.id);
    };
    assert_eq!(
        error.cause.catalog_code().to_string(),
        cause,
        "{} code",
        vector.id
    );
    assert_eq!(
        error.cause,
        AssemblyCause::UnitGraphTopology { fault, retained },
        "{} retained identity and value",
        vector.id
    );
    assert_eq!(&source[error.span.start..error.span.end], selected_name);
}

/// Trace: QSpec-TC-187. This partial slice does not back IT-019-SC-04 yet.
#[trace("QSpec-TC-187")]
#[test]
fn public_tc187_ids_fail_closed_and_real_admission_controls_refuse() {
    let mut seen = BTreeSet::new();
    let mut observed = BTreeSet::new();
    let mut planned = 0;
    for vector in vectors_of(TestCase::Tc187) {
        assert!(
            seen.insert(vector.id),
            "duplicate public TC-187 ID {}",
            vector.id
        );
        assert!(
            !vector.inputs.is_empty(),
            "{} has no authored input",
            vector.id
        );
        assert!(
            !vector.expected.is_empty(),
            "{} has no authored expectation",
            vector.id
        );
        match dispatch(vector.id)
            .unwrap_or_else(|| panic!("unknown public TC-187 ID {}", vector.id))
        {
            RowDispatch::Admission(case) => {
                assert_admission(vector, case);
                observed.insert(vector.id);
            }
            RowDispatch::Topology(expected) => {
                // Each of these IDs has a value-asserting public QSL control in
                // tc187_u11_topology, tc187_u11_local_values, tc187_u11_order or
                // tc187_u11_imported_topology, tc187_u11_dimension_terms or
                // tc187_u11_multidoc_transitive. The dispatcher guards their public
                // row identity and expectation; the separate tests execute them.
                assert!(
                    vector.expected.contains(expected),
                    "{} expectation",
                    vector.id
                );
                observed.insert(vector.id);
            }
            RowDispatch::Admitted(expected) => {
                // The positive U11r source passes the real selected-model
                // QSL check and emission in tc187_u11_imported_positive.
                assert!(
                    vector.inputs.contains("unit cm"),
                    "{} imported unit input",
                    vector.id
                );
                assert!(
                    vector.expected.contains(expected),
                    "{} positive expectation",
                    vector.id
                );
                observed.insert(vector.id);
            }
            RowDispatch::Planned { tickets, missing } => {
                let expected_tickets: &[&str] = match vector.id {
                    "U01" => &["QSL-1737", "IR-1345"],
                    "U11g3" => &["QSL-1738"],
                    _ => &["IR-842"],
                };
                assert_eq!(tickets, expected_tickets, "{} owners", vector.id);
                assert!(!missing.is_empty(), "{} missing route", vector.id);
                if vector.id == "U01" {
                    assert!(missing.contains("BoundedQuantity"));
                    assert!(missing.contains("source-to-CG agreement"));
                }
                if vector.id == "U11g3" {
                    assert!(missing.contains("empty map"));
                }
                planned += 1;
            }
        }
    }
    assert_eq!(
        observed,
        BTreeSet::from([
            "U06", "U11a", "U11b", "U11c", "U11d", "U11e", "U11f", "U11g1", "U11g2", "U11g4",
            "U11h", "U11i", "U11j", "U11k", "U11l", "U11m", "U11n", "U11o", "U11p", "U11q", "U11r",
            "U11s", "U11t", "U11u", "U11w", "U11x", "U11z3",
        ])
    );
    assert!(planned > 0, "SC-04 is not yet complete");
    assert!(
        dispatch("U41").is_none(),
        "carrier rows are outside public Vector API"
    );
    assert!(dispatch("unknown").is_none(), "new IDs fail closed");
}
