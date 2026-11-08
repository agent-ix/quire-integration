// SPDX-License-Identifier: AGPL-3.0-or-later
//! Serialize test-owned Cargo children independently of the test executable.

use std::process::Command;

/// Build a Cargo child under the shared lock and finite thermal settings.
/// Launch the owning test executable outside Cargo and the shared lock.
pub(super) fn command() -> Command {
    let mut command = Command::new("flock");
    command
        .arg("/tmp/titan-cargo.lock")
        .args(["nice", "-n", "10", "ionice", "-c", "3"])
        .arg(env!("CARGO"))
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_TEST_DEBUG", "0");
    command
}
