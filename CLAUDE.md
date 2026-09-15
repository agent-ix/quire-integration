# quire-integration

Version-locked composition and integration runtime for native Quire V1.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # run all cargo-deny supply-chain checks
make audit-unsafe   # compiler-enforce zero unsafe Rust across every target
make ci             # fmt-check + lint + test + deny + audit-unsafe
```

All checks are local. This repository intentionally has no hosted workflow, and
`publish = false` is part of its bootstrap contract.

## Safety foundation

- `clippy.toml` pins MSRV to `1.85` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs a compiler check with `-F unsafe-code`
  across every Rust target; there is no baseline or documented-unsafe exception.
- `rustfmt.toml` uses stable 100-character formatting. Local checks fail on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs                   # crate root and public exports
src/common.rs                # identities, dispositions, budgets, canonical collections
src/capability.rs            # separately typed capability boundaries
tests/common_contracts.rs    # TC-007/TC-008 foundation evidence
spec/                        # reviewed C10-C14 requirements artifacts
plan/                        # accepted implementation plan and task ledger
scripts/                     # local safety tooling
```
