// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independently check original operation source through public QSL APIs.
use qsl_package::CheckedPackage;
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_semantic_value::checking::CheckingLimits;

/// Check the authored operation source without reusing producer evaluation state.
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
    CheckedPackage::link(
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "agent-ix/scalar-operations".to_owned(),
            unit,
            SelectedModels::default(),
            Vec::new(),
        )
        .expect("original source assembles")
        .check(CheckingLimits::default())
        .expect("original source checks"),
    )
}
