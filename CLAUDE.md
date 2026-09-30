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
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + deny + audit-unsafe
```

`make test` runs the composed QSL/codegen tests, which need `cargo kani` and
`cargo llvm-cov` on `PATH`; a missing tool fails the test.

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
tests/qsl_codegen_composition.rs  # QSL -> contract IR -> codegen, compiled and executed (IT-008, IT-010)
tests/qsl_kani_exemplar.rs        # real `cargo kani` runs over codegen Kani bundles (IT-010, IT-011)
tests/support/         # shared setup over QSL's public API
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
