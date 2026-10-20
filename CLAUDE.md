# quire-integration

Version-locked composition and integration runtime for native Quire V1.

## Hash / digest / pin antipattern: present, slated for removal

This repository still contains hashes, digests, SHAs, pins, checksum catalogs and
records that track files, versions or tools. That is an antipattern and it is slated
for removal. Do not introduce any new use of it. When you touch code or specs that
contain one, remove it as part of the change. The only hash that stays is a canonical
identity digest that binds a proof to the exact content it proved. Package versions
live in Cargo.toml / package.json and their lockfiles only; reports name the app
version they ran.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make kani           # real Kani proof and mutation tests
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + kani + deny + audit-unsafe
```

`make test` runs the composed QSL, contract IR, driver and codegen tests. The
real Kani proof and mutation controls run under `make kani` and `make ci`.

All checks are local. This repository intentionally has no hosted workflow, and
`publish = false` is part of its bootstrap contract.

## Safety scaffolding

- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. Local checks fail on drift.
- `rust-toolchain.toml` selects Rust 1.98.1 (the floor quire-spec-language and quire-contract-codegen require) + rustfmt + clippy + llvm-tools.

## Layout

```
src/lib.rs             # crate root
tests/integration.rs   # end-to-end tests
tests/ir914_bound_export.rs       # QSL -> driver -> CG, with one export refusal (IR-914)
tests/qsl_model_owner_admission.rs # QSL checked-package/v2 -> IR -> CG (IT-012, IT-008, IT-010)
tests/support/         # QSL fixture lookup through Cargo metadata
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
