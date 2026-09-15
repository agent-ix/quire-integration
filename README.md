# quire-integration

Private Rust owner for version-locked composition and integration of native
Quire V1.

## Charter

This repository will implement the AD-008/AD-011 composed-execution boundary,
including exact ecosystem locks, ordered execution bundles, authorized
decision/effect-adapter coordination, deterministic model views, and the real
IT-070 through IT-078 qualification harness. It consumes immutable versioned
artifacts from QSL, TL, QObs, Protocol, Verification, Contract IR, and codegen.

## Non-goals

- No source grammar, parser, type system, temporal semantics, protocol semantics,
  proof engine, evidence store, or generated-code semantics are reimplemented
  here.
- No evaluator directly invokes an external business adapter.
- No dependency is selected from an ambient registry or compatible-looking
  current version.
- No public package, hosted CI, public repository, or release claim is authorized
  by this bootstrap.

## Current status

Repository scaffold only. Research #34/#74 owns creation and the required major
`/specify` plus self `/spec-review` cycle for C10 through C14. No composed runtime
or qualification capability is implemented yet, and the placeholder crate API
makes no semantic claim.

## Local checks

```bash
make fmt-check
make lint
make test
make build
make deny
make audit-unsafe
```

## License and content rights

Original project content is licensed under `AGPL-3.0-or-later`. See `LICENSE`,
`LICENSE-DECISION.md`, and `CONTENT_RIGHTS.md`. External material is prohibited
unless its exact provenance, license or permission, attribution, approver, date,
and distribution scope are recorded before inclusion.
