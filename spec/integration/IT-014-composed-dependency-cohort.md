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

The composed Cargo lock for QSL, semantic value, IR, codegen, runtime,
verification contracts, filament core data and quire-rs.

## Preconditions

Cargo metadata reads the integration manifest with `--locked`.

## Inputs

The repository's committed `Cargo.toml` and `Cargo.lock`.

## Test Procedure

1. Resolve the locked dependency graph with Cargo metadata.
   IT-014-SC-01: every named shared type package has exactly one package
   identity and a locked Git source; the three filament core data crates and
   two quire-rs crates each come from their expected repository, with one
   revision per repository; `serde_json_canonicalizer` is absent.

## Expected Results

Cargo resolves one coherent dependency cohort without the retired encoder.
