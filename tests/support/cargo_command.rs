// SPDX-License-Identifier: AGPL-3.0-or-later
//! Serialize test-owned Cargo children independently of the test executable.

use std::{path::Path, process::Command};

/// Build a Cargo child under the shared lock and finite thermal settings.
/// Launch the owning test executable outside Cargo and the shared lock.
pub(crate) fn command() -> Command {
    // A temporary child directory must not reselect an ambient rustup compiler.
    let cargo = Path::new(env!("CARGO"));
    let compiler = cargo.with_file_name(format!("rustc{}", std::env::consts::EXE_SUFFIX));
    assert!(
        compiler.is_file(),
        "the selected Cargo toolchain contains rustc"
    );
    let mut command = Command::new("flock");
    command
        .arg("/tmp/titan-cargo.lock")
        .args(["nice", "-n", "10", "ionice", "-c", "3"])
        .arg(cargo)
        .env("RUSTC", compiler)
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_TEST_DEBUG", "0");
    command
}
