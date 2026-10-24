---
id: IT-014
title: "Compose one shared type dependency cohort"
type: IT
---
# IT-014: Compose one shared type dependency cohort

## Objective

Check that the locked integration graph resolves each shared type package once,
uses one revision of each multi-crate source repository, and excludes the
retired canonical JSON encoder.

## Target Integration

The composed Cargo lock for canonical encoding, QSL, semantic value, IR,
codegen, runtime, verification contracts, filament core data and quire-rs.

## Preconditions

Cargo metadata reads the integration manifest with `--locked`.

## Inputs

The repository's committed `Cargo.toml` and `Cargo.lock`.

The named shared type packages are `quire-semantic-value`,
`quire-contract-model`, `quire-contract-ir`, `quire-contract-codegen`,
`quire-contract-runtime`, `quire-spec-language`, `qsl-foundation`,
`qsl-replay` and `quire-verification-contracts`. The first-party Git repository
for each is the same as its package name, except `quire-contract-model` and
`quire-contract-ir` both come from `quire-contract-ir`, and `qsl-foundation`
and `qsl-replay` both come from `quire-spec-language`.
The canonical source contributes `quire-canonical` and
`quire-canonical-derive`. The normal graph selects `quire-rs`; its optional
extraction and parser packages need not be present.

## Test Procedure

1. Resolve the locked dependency graph with Cargo metadata.
   IT-014-SC-01: every named shared type package has exactly one package
   identity and a locked Git source; the canonical, QSL, IR and three filament
   core data crates and the selected `quire-rs` package come from their expected
   repositories, with one revision across every selected package from each
   repository; `serde_json_canonicalizer` is absent.

## Expected Results

Cargo resolves one coherent dependency cohort without the retired encoder.
