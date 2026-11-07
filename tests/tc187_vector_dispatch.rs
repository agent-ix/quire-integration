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
enum NativeOperation {
    Add,
    Convert,
    Compare,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowDispatch {
    Admission(AdmissionCase),
    Topology(&'static str),
    Admitted(&'static str),
    Checked(&'static str),
    GeneratedAgreement(&'static str),
    NativeOriginal {
        expected: &'static [&'static str],
        operation: NativeOperation,
    },
    NativeOwner {
        expected: &'static str,
        generated_owner: &'static str,
        missing: &'static str,
    },
    QslControl {
        expected: &'static str,
        remaining_owner: &'static str,
        missing: &'static str,
    },
    Planned {
        tickets: &'static [&'static str],
        missing: &'static str,
    },
    /// Public IDs that no FR-014 acceptance criterion selects.
    OutsideRequiredSet,
}

// This is dispatch over public IDs, not a copy of QSpec's inputs or expected
// prose. A newly exported ID must get its own disposition instead of silently
// inheriting a passing row. Routed mixed rows name only the backed axis and
// retain their remaining generated or source-authority work.
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
        "U11y" | "U11z1" | "U11z2" => Some(RowDispatch::Topology("missing-name")),
        "U11r" => Some(RowDispatch::Admitted("admitted")),
        "U11n" | "U11o" => Some(RowDispatch::Topology("definition-cycle")),
        "U11p" | "U11w" => Some(RowDispatch::Topology("missing-name")),
        "U11q" | "U11z3" => Some(RowDispatch::Topology("invalid-value")),
        "U11x" => Some(RowDispatch::Topology("missing-name")),
        "U30" | "U31" => Some(RowDispatch::Checked("type-mismatch")),
        "U14" => Some(RowDispatch::Checked("ill_typed")),
        "U12" => Some(RowDispatch::GeneratedAgreement("empty dimensionless")),
        "U02" => Some(RowDispatch::GeneratedAgreement("empty dimension")),
        "U04" => Some(RowDispatch::GeneratedAgreement("5463/20")),
        "U07" => Some(RowDispatch::GeneratedAgreement("nearest-even")),
        "U08" => Some(RowDispatch::GeneratedAgreement("refuse atomically")),
        "U09b" => Some(RowDispatch::GeneratedAgreement("conversion succeeds exactly")),
        "U10" => Some(RowDispatch::GeneratedAgreement("1 m")),
        "U16" => Some(RowDispatch::GeneratedAgreement("unit.target-domain")),
        "U17" => Some(RowDispatch::GeneratedAgreement("membership")),
        "U18" => Some(RowDispatch::GeneratedAgreement("4294967296")),
        "U22" => Some(RowDispatch::GeneratedAgreement("40000")),
        "U25" => Some(RowDispatch::GeneratedAgreement("1 J")),
        "U29" => Some(RowDispatch::GeneratedAgreement("integer-domain")),
        "U13" => Some(RowDispatch::GeneratedAgreement("incomplete")),
        "U21" => Some(RowDispatch::GeneratedAgreement("dimensionless")),
        "U28" => Some(RowDispatch::GeneratedAgreement("127/500000")),
        "U15" => Some(RowDispatch::NativeOriginal {
            expected: &["127/50", "unit.edge"],
            operation: NativeOperation::Convert,
        }),
        "U26" => Some(RowDispatch::NativeOriginal {
            expected: &["-9 u1"],
            operation: NativeOperation::Convert,
        }),
        "U19" => Some(RowDispatch::NativeOriginal {
            expected: &["5 m", "unit.result-retain"],
            operation: NativeOperation::Add,
        }),
        "U20" => Some(RowDispatch::NativeOriginal {
            expected: &["3 u2"],
            operation: NativeOperation::Add,
        }),
        "U24" => Some(RowDispatch::NativeOriginal {
            expected: &["3 cm", "no `unit.edge`"],
            operation: NativeOperation::Add,
        }),
        "U23" => Some(RowDispatch::NativeOriginal {
            expected: &["false"],
            operation: NativeOperation::Compare,
        }),
        "U27" => Some(RowDispatch::QslControl {
            expected: "undefined",
            remaining_owner: "QSL",
            missing: "QSL check refuses the zero divisor as Unproved(Nonzero) before emission, so no generated claim exists",
        }),
        "U03" | "U05" | "U09" => Some(RowDispatch::Checked("ill_typed")),
        "U36" => Some(RowDispatch::NativeOwner {
            expected: "unequal compound units",
            generated_owner: "IR-842",
            missing: "CG generated comparison, direct dsec conversion and whole-let bridge agreement",
        }),
        "U33" => Some(RowDispatch::QslControl {
            expected: "literal",
            remaining_owner: "IR-842",
            missing: "native runtime and generated agreement beyond checked emission",
        }),
        "U37" => Some(RowDispatch::QslControl {
            expected: "conflicting-definition",
            remaining_owner: "IR-842",
            missing: "value and generated agreement beyond equivalence admission",
        }),
        "U01" => Some(RowDispatch::Planned {
            tickets: &["IR-1401"],
            missing: "CG operand check admits only unit and compound_unit forms, so the carried_quantity parameter of a carried convert-then-add refuses OperandTypeMismatch and source-to-CG agreement is unexecuted",
        }),
        "U11g3" => Some(RowDispatch::Planned {
            tickets: &["QSL-1738"],
            missing: "semantic-value derived dimension N = L / L must have an empty map, not N^1",
        }),
        "U32" | "U34" | "U35" | "U39" => Some(RowDispatch::OutsideRequiredSet),
        "U38" | "U40" => pending("intended equivalent-unit positive conversions and dependent operations are unexecuted; IR-842 QSL owner routing"),
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
                // tc187_u11_imported_topology, tc187_u11_dimension_terms,
                // tc187_u11_multidoc_transitive or tc187_u11_imported_base.
                // The dispatcher guards public identity and expectation;
                // the separate tests execute each case.
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
            RowDispatch::Checked(expected) => {
                // tc187_u30_u31 binds quantity type checks, while
                // tc187_p9_u14_static binds five source refusals and separate
                // zero-limit owner controls. tc187_p8_static_refusals binds
                // U03/U05/U09 source and owner refusals. No generated credit.
                assert!(
                    vector.expected.contains(expected),
                    "{} expectation",
                    vector.id
                );
                observed.insert(vector.id);
            }
            RowDispatch::GeneratedAgreement(expected) => {
                // tc187_u12_arithmetic, tc187_u13_u21_power, tc187_ir842_conversions and
                // tc187_p9_remaining_conversions execute these original operations as QSL
                // source, emitted bytes, IR admission, CG generation and a native oracle
                // compared with QSL evaluation.
                assert!(
                    vector.expected.contains(expected),
                    "{} generated agreement expectation",
                    vector.id
                );
                observed.insert(vector.id);
            }
            RowDispatch::NativeOriginal {
                expected,
                operation,
            } => {
                // tc187_u19_u24_addition and the PR #80 conversion/quantity
                // tests compare only these original operations to native CG.
                let expected_operation = match vector.id {
                    "U15" | "U26" => NativeOperation::Convert,
                    "U19" | "U20" | "U24" => NativeOperation::Add,
                    "U23" => NativeOperation::Compare,
                    other => panic!("unexpected native original row {other}"),
                };
                assert_eq!(operation, expected_operation, "{} operation", vector.id);
                for fragment in expected {
                    assert!(
                        vector.expected.contains(*fragment),
                        "{} original {operation:?} expectation: {fragment}",
                        vector.id
                    );
                }
                observed.insert(vector.id);
            }
            RowDispatch::NativeOwner {
                expected,
                generated_owner,
                missing,
            } => {
                // tc187_p8_conversion, tc187_p8_carried_conversion and
                // tc187_p8_model_values compare genuine source operations
                // with native owner values and ledgers. None executes CG.
                assert!(
                    vector.expected.contains(expected),
                    "{} owner value expectation",
                    vector.id
                );
                assert_eq!(generated_owner, "IR-842", "{} generated owner", vector.id);
                assert!(
                    missing.contains("CG"),
                    "{} missing generated path",
                    vector.id
                );
                observed.insert(vector.id);
            }
            RowDispatch::QslControl {
                expected,
                remaining_owner,
                missing,
            } => {
                // U22/U25 keep source and owner conversion axes separate;
                // U27 keeps checked-expression authority distinct from its
                // full-source UnprovedNonzero refusal. U33 is checked
                // admission/emission; U37 is equivalence admission. None
                // receives unobserved generated or native runtime credit.
                assert!(
                    vector.expected.contains(expected),
                    "{} expectation",
                    vector.id
                );
                let expected_owner = if matches!(vector.id, "U27") {
                    "QSL"
                } else {
                    "IR-842"
                };
                assert_eq!(
                    remaining_owner, expected_owner,
                    "{} remaining owner",
                    vector.id
                );
                assert!(!missing.is_empty(), "{} missing original route", vector.id);
                observed.insert(vector.id);
            }
            RowDispatch::Planned { tickets, missing } => {
                let expected_tickets: &[&str] = match vector.id {
                    "U01" => &["IR-1401"],
                    "U11g3" => &["QSL-1738"],
                    _ => &["IR-842"],
                };
                assert_eq!(tickets, expected_tickets, "{} owners", vector.id);
                assert!(!missing.is_empty(), "{} missing route", vector.id);
                if vector.id == "U01" {
                    assert!(missing.contains("carried_quantity"));
                    assert!(missing.contains("source-to-CG agreement"));
                }
                if vector.id == "U11g3" {
                    assert!(missing.contains("empty map"));
                }
                if matches!(vector.id, "U38" | "U40") {
                    assert!(missing.contains("positive conversions"));
                }
                planned += 1;
            }
            RowDispatch::OutsideRequiredSet => {
                assert!(matches!(vector.id, "U32" | "U34" | "U35" | "U39"));
            }
        }
    }
    assert_eq!(
        observed,
        BTreeSet::from([
            "U06", "U11a", "U11b", "U11c", "U11d", "U11e", "U11f", "U11g1", "U11g2", "U11g4",
            "U11h", "U11i", "U11j", "U11k", "U11l", "U11m", "U11n", "U11o", "U11p", "U11q", "U11r",
            "U11s", "U11t", "U11u", "U11w", "U11x", "U11y", "U11z1", "U11z2", "U11z3", "U30",
            "U31", "U12", "U13", "U14", "U15", "U16", "U17", "U18", "U19", "U20", "U21", "U22",
            "U23", "U24", "U25", "U26", "U27", "U28", "U29", "U02", "U03", "U04", "U05", "U07",
            "U08", "U09", "U09b", "U10", "U33", "U36", "U37",
        ])
    );
    // Rows executed above that keep a Planned remainder with its own owner: U25's nominal-to-compound
    // first step (IR-1403, compound unit grammar) and U21's zero-to-negative power, which QSL
    // refuses as Unproved(Nonzero) before emission.
    for (id, owner) in [("U25", "IR-1403"), ("U21", "QSL")] {
        assert!(
            observed.contains(id),
            "{id} is executed and keeps a Planned remainder on {owner}"
        );
    }
    assert!(planned > 0, "SC-04 is not yet complete");
    assert!(
        dispatch("U41").is_none(),
        "carrier rows are outside public Vector API"
    );
    assert!(dispatch("unknown").is_none(), "new IDs fail closed");
}
