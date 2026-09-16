# Content Rights Policy

Code and documentation authored for this project may be committed under the repository
license. External material is prohibited by default.

## Allowed

- Original project code, tests, schemas, examples, and documentation.
- Short factual citations or links that identify an external source without reproducing
  its protected expression.
- Material covered by a compatible license when its license, attribution, and provenance
  are recorded and a human has approved its inclusion.

## Not allowed in Git or GitHub

- Source text, tables, clause inventories, or close paraphrases whose distribution rights
  have not been confirmed.
- Counsel communications, legal-review packets, or material awaiting a rights decision.
- Private research files, internal scratch paths, or local workstation paths.
- Protected source documents or exports, including PDF, office-document, and spreadsheet
  files, unless a documented human approval explicitly permits the exact file.

These exclusions apply to ignored files, Git LFS, branches, issues, pull requests, commit
messages, CI artifacts, and private repositories as well as the default branch.

The project preflight rejects unreviewed file types by default. Add a format to its text
allowlist only after deciding how its contents will be inspected; separately review any
original binary asset before admitting its exact format.

## Review

Project owner: Agent IX

Record each approved external item with its source, license or permission basis,
attribution requirement, approver, approval date, and allowed distribution scope. Keep
unapproved source material outside the Git working tree.

## Approved dependency record

The project owner-approved dependency policy in `deny.toml` applies to the exact
versions and source revisions retained by `Cargo.lock`. Dependencies remain
unmodified upstream works; their own license notices and attribution terms apply.

| Direct dependency | Exact selection | License basis | Attribution | Approver / date | Allowed scope |
| --- | --- | --- | --- | --- | --- |
| `serde` / `serde_json` | `1.0.228` / `1.0.151` from crates.io | MIT OR Apache-2.0 | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-15 | private source, local build, test and qualification |
| `sha2` | `0.10.9` from crates.io | MIT OR Apache-2.0 | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-15 | private source, local build, test and qualification |
| `semver` | `1.0.28` from crates.io | MIT OR Apache-2.0 | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-15 | private source, local build, test and qualification |
| `jsonschema` | `0.17.1` from crates.io, test-only with default features disabled | MIT | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-15 | private local schema conformance tests |
| `proptest` | `1.11.0` from crates.io, test-only with only the `std` feature | MIT OR Apache-2.0 | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-15 | private local generated property tests |
| `fs2` | `0.4.3` from crates.io, test-only | MIT OR Apache-2.0 | upstream package metadata and license files | Agent IX repository dependency policy / 2026-09-16 | private local crash/restart and durable-store integration tests |
| `ix-trace-rs` | git revision `2ce4ebf47f726b9d76388220545cd0abda8a5cfb`, version `0.1.0` | AGPL-3.0-or-later | Agent IX project identity retained in source and lock | Agent IX repository dependency policy / 2026-09-15 | private local trace-tagged tests |

Transitive packages are limited to the exact `Cargo.lock` closure of these
selections. `cargo deny check` enforces the recorded license/source policy before
review; changing the lock requires a fresh rights and dependency-policy review.
