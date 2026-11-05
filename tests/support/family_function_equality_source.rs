// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original equality source and public acquisition shared by host and native driver.

use super::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use qsl_replay::{
    acquire_checked_closure, CheckedClosureCompilation, CompileUnit, DependencyInput, ReplayLimits,
    SourceIdentity, StageLimits,
};

/// Original declarations, selected call occurrences and closed input constructors.
pub const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "tuple Pair(Boolean, Boolean);\n",
    "record Flag { flag: Boolean; }\n",
    "union Choice { First, Second }\n",
    "function boolEqual using v(left: Boolean, right: Boolean): Boolean pure { left = right }\n",
    "function boolNotEqual using v(left: Boolean, right: Boolean): Boolean pure { left != right }\n",
    "function intEqual using v(left: Int[0,9], right: Int[0,9]): Boolean pure { left = right }\n",
    "function intNotEqual using v(left: Int[0,9], right: Int[0,9]): Boolean pure { left != right }\n",
    "function pairEqual using v(left: Pair, right: Pair): Boolean pure { left = right }\n",
    "function pairNotEqual using v(left: Pair, right: Pair): Boolean pure { left != right }\n",
    "function flagEqual using v(left: Flag, right: Flag): Boolean pure { left = right }\n",
    "function flagNotEqual using v(left: Flag, right: Flag): Boolean pure { left != right }\n",
    "function choiceEqual using v(left: Choice, right: Choice): Boolean pure { left = right }\n",
    "function choiceNotEqual using v(left: Choice, right: Choice): Boolean pure { left != right }\n",
    "function exerciseBoolEqual using v(x: Boolean, y: Boolean): Boolean pure { boolEqual(x,y) }\n",
    "function exerciseBoolNotEqual using v(x: Boolean, y: Boolean): Boolean pure { boolNotEqual(x,y) }\n",
    "function exerciseIntEqual using v(x: Int[0,9], y: Int[0,9]): Boolean pure { intEqual(x,y) }\n",
    "function exerciseIntNotEqual using v(x: Int[0,9], y: Int[0,9]): Boolean pure { intNotEqual(x,y) }\n",
    "function exercisePairEqual using v(x: Pair, y: Pair): Boolean pure { pairEqual(x,y) }\n",
    "function exercisePairNotEqual using v(x: Pair, y: Pair): Boolean pure { pairNotEqual(x,y) }\n",
    "function exerciseFlagEqual using v(x: Flag, y: Flag): Boolean pure { flagEqual(x,y) }\n",
    "function exerciseFlagNotEqual using v(x: Flag, y: Flag): Boolean pure { flagNotEqual(x,y) }\n",
    "function exerciseChoiceEqual using v(x: Choice, y: Choice): Boolean pure { choiceEqual(x,y) }\n",
    "function exerciseChoiceNotEqual using v(x: Choice, y: Choice): Boolean pure { choiceNotEqual(x,y) }\n",
    "function negate using v(left: Int[0,9], right: Int[0,9]): Integer pure { -left }\n",
    "function exerciseNegate using v(x: Int[0,9], y: Int[0,9]): Integer pure { negate(x,y) }\n",
    "function makePairA using v(): Pair pure { Pair(true,false) }\n",
    "function makePairB using v(): Pair pure { Pair(false,true) }\n",
    "function makeFlagA using v(): Flag pure { Flag { flag: true } }\n",
    "function makeFlagB using v(): Flag pure { Flag { flag: false } }\n",
    "function makeChoiceA using v(): Choice pure { Choice::First }\n",
    "function makeChoiceB using v(): Choice pure { Choice::Second }\n",

);

/// Independently check, emit and strictly admit the provided original source.
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
    (acquisition, *package)
}

/// Selected equality declarations in original source order.
pub const NAMES: [&str; 10] = [
    "boolEqual",
    "boolNotEqual",
    "intEqual",
    "intNotEqual",
    "pairEqual",
    "pairNotEqual",
    "flagEqual",
    "flagNotEqual",
    "choiceEqual",
    "choiceNotEqual",
];
/// Distinct original call occurrences for each selected equality declaration.
pub const EXERCISES: [&str; 10] = [
    "exerciseBoolEqual",
    "exerciseBoolNotEqual",
    "exerciseIntEqual",
    "exerciseIntNotEqual",
    "exercisePairEqual",
    "exercisePairNotEqual",
    "exerciseFlagEqual",
    "exerciseFlagNotEqual",
    "exerciseChoiceEqual",
    "exerciseChoiceNotEqual",
];
