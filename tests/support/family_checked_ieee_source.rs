// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independently check original IEEE operation source through public QSL APIs.
use qsl_package::CheckedPackage;
use qsl_semantics::{
    check::PackageDeclarations,
    model::intake::SelectedModels,
    value::{admit_ieee_profile, DefinitionRole},
};
use quire_semantic_value::checking::CheckingLimits;

/// Check the authored IEEE source with the selected IEEE profile admitted.
pub fn checked(source: &str) -> CheckedPackage {
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
    declarations.ieee_profile = Some(
        admit_ieee_profile(&[DefinitionRole::IeeeProfile.reference()], &[])
            .expect("the selected IEEE profile admits"),
    );
    CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("original source checks"),
    )
}
