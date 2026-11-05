// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original equality source and public acquisition shared by host and native driver.

use super::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use qsl_replay::{
    acquire_checked_closure, CheckedClosureCompilation, CompileUnit, DependencyInput, ReplayLimits,
    SourceIdentity, StageLimits,
};

pub const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "function boolEqual using v(left: Boolean, right: Boolean): Boolean pure { left = right }\n",
    "function boolNotEqual using v(left: Boolean, right: Boolean): Boolean pure { left != right }\n",
    "function intEqual using v(left: Int[0,9], right: Int[0,9]): Boolean pure { left = right }\n",
    "function intNotEqual using v(left: Int[0,9], right: Int[0,9]): Boolean pure { left != right }\n",
    "function exerciseBoolEqual using v(x: Boolean, y: Boolean): Boolean pure { boolEqual(x,y) }\n",
    "function exerciseBoolNotEqual using v(x: Boolean, y: Boolean): Boolean pure { boolNotEqual(x,y) }\n",
    "function exerciseIntEqual using v(x: Int[0,9], y: Int[0,9]): Boolean pure { intEqual(x,y) }\n",
    "function exerciseIntNotEqual using v(x: Int[0,9], y: Int[0,9]): Boolean pure { intNotEqual(x,y) }\n",
);

pub fn acquire(bytes: &[u8]) -> (CheckedClosureCompilation, CheckedPackageV2) {
    let acquisition = acquire_checked_closure(
        CompileUnit {
            source: SourceIdentity::new("agent-ix", "function-equality"),
            package_name: "function-equality".to_owned(),
            path: "function-equality.native",
            bytes,
        },
        [],
        &DependencyInput::default(),
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("original equality source checks and emits");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        acquisition.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("exact equality source emission admits");
    };
    (acquisition, package)
}
