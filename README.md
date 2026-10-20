# quire-integration

quire-integration is the cross-repository integration test crate for Quire, a
toolchain that writes a system's rules in a formal language, checks them, and
turns them into generated code and proofs. Each stage lives in its own
repository and tests itself in isolation. Two stages can pass their own tests
and still disagree about the bytes passed between them.

The crate pins the stages to exact commits in one `Cargo.lock`, so every stage
resolves to a single copy. Its tests take a rule through QSL, contract IR and
the code generator, then compile and execute the result and compare it with
QSL's own runtime. Maintainers of the stage repositories use it to find
disagreements between stages.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="The integration tests take a QSL source and its domain model through QSL's compiler into a checked package, read it with contract IR and feed it to the code generator, then compile and run what was generated. QSL's own runtime evaluates the same input, and the test passes only when the two verdicts agree." src="docs/images/hero-light.svg">
</picture>

> **Status: prerelease, private repository.** The library crate is a
> placeholder with one function (`src/lib.rs`) and is not published. The
> integration tests hold all current functionality. The composed runtime
> described in the repository's early charter (ordered execution bundles,
> effect-adapter coordination, model views) is not implemented.

## Tests

Each test file drives real producer bytes through real consumers. None mocks a
neighboring stage.

| Test file | Path it exercises | What must hold |
| --- | --- | --- |
| `tests/qsl_codegen_composition.rs` | QSL source and model, lowered to IR bytes, read by contract IR, handed to the code generator; generated Rust and property-test strategies are compiled and run | Native verdicts, independently specified expectations and generated code agree on a complete small population; generated clause activation matches native source events; property strategies run in-domain with no discards |
| `tests/qsl_kani_exemplar.rs` | The same path, ending in a real `cargo kani` run over the generated bundle | The proof discharges real checks in every generated module; mutating the generated logic turns the proof into a failure; a falsified run's counterexample replays through QSL |
| `tests/qsl_model_owner_admission.rs` | A QSL unit that selects a domain-package model, emitted as `checked-package/v2` bytes and read by contract IR | The package is admitted, a document supplied under another digest is refused, and model nodes are keyed by content only, so a version-only change keeps every node id |
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

The tests fail when a tool is missing and never dispatch a hosted run.

| Needed by | Tool |
| --- | --- |
| all tests | Rust 1.98 (`rust-toolchain.toml`) |
| code-generation tests | Rust with its LLVM tools, and `cargo-llvm-cov` |
| proof tests | `cargo kani`, with `-Z unstable-options --export-json` support |

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
make test        # cargo test (needs the tools above)
make build       # release build
make deny        # licence check
make audit-unsafe
make ci          # all of the gates
```

## Contributing and license

Contributions need a signed Contributor License Agreement; see
[CONTRIBUTING.md](CONTRIBUTING.md) and [CLA.md](CLA.md). Original content is
licensed under `AGPL-3.0-or-later`; see [LICENSE](LICENSE),
[LICENSE-DECISION.md](LICENSE-DECISION.md) and
[CONTENT_RIGHTS.md](CONTENT_RIGHTS.md).
