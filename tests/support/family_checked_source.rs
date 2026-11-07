// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independently check original operation source through public QSL APIs.
use qsl_package::CheckedPackage;
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_exact::DivisionProfile;
use quire_semantic_value::checking::CheckingLimits;

/// Check the authored operation source without reusing producer evaluation state.
pub fn checked(source: &str) -> CheckedPackage {
    checked_with_division(source, None)
}

/// Check source under one authoritative integer division law.
pub fn checked_with_division(source: &str, profile: Option<DivisionProfile>) -> CheckedPackage {
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "scalar-operations"),
        "scalar-operations.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original source forms");
    let mut declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/scalar-operations".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("original source assembles");
    if let Some(profile) = profile {
        use qsl_semantics::value::{admit_integer_division, DefinitionRole};
        let role = match profile {
            DivisionProfile::Truncating => DefinitionRole::IntegerDivisionTruncating,
            DivisionProfile::Floor => DefinitionRole::IntegerDivisionFloor,
            DivisionProfile::Euclidean => DefinitionRole::IntegerDivisionEuclidean,
        };
        declarations.integer_division = Some(
            admit_integer_division(&[role.reference()], None)
                .expect("original selected division definition admits"),
        );
    }
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("original source checks"),
    )
}
