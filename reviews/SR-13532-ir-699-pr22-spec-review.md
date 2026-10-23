---
id: SR-13532
title: "Spec review — IR-699 IT-014 cohort criterion"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; PR #22 abeefe8c477b43fb267f03c039507af08a74eb67; spec/integration/IT-014-composed-dependency-cohort.md, tests/dependency_cohort.rs, Cargo.toml"
review_set: subset
---

## Summary

Independent base spec review of new IT-014 and IT-014-SC-01, with the traced Rust test as context. The scenario is observable and avoids an exact commit pin, but leaves the required shared-type package membership outside the criterion.

## Verdict

**CONDITIONAL** — the criterion's “named shared type package” set is undefined in the spec, so different implementations can omit a package and still claim to meet it.

## Examined

- IT-014 objective (examined): one shared type dependency cohort, one revision per multi-crate source, no retired encoder.
- IT-014-SC-01 (examined): one identity and locked Git source for every named shared type package; three FCD and two quire-rs crates from their expected repositories, one revision per repository, canonicalizer absent.
- `tests/dependency_cohort.rs` (context_only): current traced test enumerates nine shared type packages and five FCD/quire-rs members.
- No applicable AssuranceProfile, new FR/NFR/StR statement, dependency edge or domain object was found for the diff; base review applies, integrity and EARS lenses do not target this IT artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | IT-014-SC-01 says “every named shared type package” but the spec names only component families. It does not identify the `qsl-foundation`/`qsl-replay` and `quire-contract-model`/`quire-contract-ir` package members that its test enumerates. An implementation could omit one and still satisfy the written scenario. Name the required package set or define an authoritative derivation of that set. | spec/integration/IT-014-composed-dependency-cohort.md:30 |

## Validation

`quire validate --scope . spec/integration/IT-014-composed-dependency-cohort.md` passed. `quire matrix --scope . --format json` returned engine metadata only, so no computed criterion binding could be verified. The spec adds no exact commit hash, vendored content or compatibility rule.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | IT-014 Inputs now enumerates the nine named shared type packages and maps alias package names to their owning Git repositories. |
