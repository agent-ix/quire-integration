# quire-integration

quire-integration is the cross-repository integration test crate for the Quire
toolchain. It checks that the stages, each in its own repository, agree on the
bytes they pass to each other.

It pins every stage to an exact commit in one `Cargo.lock`. Its tests admit
checked QSL packages through Contract IR, pass supported scalar operations to
the code generator, and exercise real proof and native paths. The table below
states which paths each test composes.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="Diagram of the intended full composition chain from a QSL source through contract IR, code generation, and native execution. The current test scope is listed below." src="docs/images/hero-light.svg">
</picture>

> **Status: prerelease, private repository.** The library crate is a
> placeholder with one function (`src/lib.rs`) and is not published. The
> integration tests hold all current functionality. The composed runtime
> described in the repository's early charter (ordered execution bundles,
> effect-adapter coordination, model views) is not implemented.

## Tests

The composition tests use the locked producer and consumer APIs. The table
lists the path each test exercises.
The IR-914 export tests cover the natural relationless skip and a real QSL
`export_bindings` result passed through driver and CG. The original partition
control also injects one refusal to test exact driver transport. The real
export test forces the driver's relation seam because its checked source does
not author an abstraction relation.

| Test file | Path it exercises | What must hold |
| --- | --- | --- |
| `tests/ir914_bound_export.rs` | QSL compilation, the driver's relationless skip and real per-item binding export, CG generation, and a separate real Kani scalar path | The skip sends every item to CG; a real QSL refusal is withheld with its original index; the integer proof verifies and its arithmetic mutant falsifies |
| `tests/qsl_model_owner_admission.rs` | A QSL unit emitted as `checked-package/v2` bytes, read by contract IR and passed to CG's scalar claim API | Model ownership stays content based; integer addition and Boolean negation produce checked CG claims |
| `tests/integration.rs` | The placeholder library | It links |

The plain-language test descriptions are in `spec/integration/`, one file per
scenario, each with its objective, preconditions, procedure and expected result.

### Example

The model-owner test checks that contract IR refuses a package whose domain
document does not match the digest the unit selected
(`tests/qsl_model_owner_admission.rs`):

```rust
#[test]
#[trace("IT-012-SC-01")]
fn a_domain_package_supplied_under_another_digest_is_refused() {
    let document = domain_package(AUTHORED_VERSION);
    let bytes = emit(&unit_selecting(AUTHORED_VERSION, &document), &document);
    let evidence = evidence(&bytes, &"0".repeat(64), &document);
    let CheckedPackageDispatchResult::Refused(refusal) = read(&bytes, &evidence) else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        refusal.path.map(|path| path.to_string()).as_deref(),
        Some("/lock/model_selections/0/digest")
    );
}
```

## Version lock

The lock is `Cargo.lock`. `Cargo.toml` names each first-party repository by
branch `main`, and the lock fixes that branch to one commit, so all the crates
of a repository resolve to the same revision. QSL's lowering and the
code generator must share one set of contract IR types. The
table is generated from the lock file.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/lock-dark.svg">
  <img alt="The version lock is Cargo.lock. It pins quire-spec-language, filament-core-data, quire-canonical, quire-contract-ir, ix-trace-rs, quire-contract-codegen, quire-contract-runtime, quire-rs, quire-verification-contracts each to one exact commit, so every crate of a repository resolves to a single copy." src="docs/images/lock-light.svg">
</picture>

To move the lock, run `cargo update -p <crate>` for the repository you want and
commit the new `Cargo.lock` together with a passing test run.

## Requirements

The tests fail when a required tool is missing and never dispatch a hosted run.

| Needed by | Tool |
| --- | --- |
| all tests | Rust toolchain selected by `rust-toolchain.toml` |
| `make kani` and `make ci` | Installed Kani and CBMC |

## Quire ecosystem

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Quire repositories. The main chain runs from the standard, quire-specification, to the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and quire-contract-runtime. quire-spec-language also feeds quire-protocol, and quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and quire-verification plans verification. Below it, shared libraries used throughout: quire-exact, quire-semantic-value, quire-walk, quire-canonical and quire-verification-contracts." src="docs/images/ecosystem-light.svg">
</picture>

quire-integration sits at the top of the ecosystem. It depends on the language and backend
repositories (quire-spec-language, quire-contract-ir, quire-contract-codegen,
and through them quire-contract-runtime and the foundation crates) as
development dependencies only. Nothing depends on it, so a disagreement between
stages shows up here. The locked
repositories are listed in the version lock section.

## Building and testing

```bash
make fmt-check   # rustfmt
make lint        # clippy, warnings are errors
make test        # cargo test
make kani        # real Kani proof and mutation
make build       # release build
make deny        # licence check
make audit-unsafe # Clippy unsafe-block documentation gate
make ci          # all of the gates
```

## Contributing and license

Contributions need a signed Contributor License Agreement; see
[CONTRIBUTING.md](CONTRIBUTING.md) and [CLA.md](CLA.md). Original content is
licensed under `AGPL-3.0-or-later`; see [LICENSE](LICENSE),
[LICENSE-DECISION.md](LICENSE-DECISION.md) and
[CONTENT_RIGHTS.md](CONTENT_RIGHTS.md).
