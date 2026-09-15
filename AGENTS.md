# Contributor guidance

Read this file, `README.md`, `LICENSE-DECISION.md`, `CONTENT_RIGHTS.md`, and the
owning issues before editing. This private bootstrap implements Research #34/#74
under integration epics #39/#40.

- Complete the repository-wide `/specify` and owner-selected self
  `/spec-review` cycle for C10 through C14 before production implementation.
- First-party production, test, qualification, generator, validator, adapter,
  and audit code is Rust. Data fixtures and schemas are not executable paths.
- Keep composition separate from domain authority: consume exact versioned QSL,
  TL, QObs, Protocol, Verification, IR, and codegen artifacts without duplicating
  their semantics.
- Preserve every independent domain disposition and provenance axis. Missing,
  incompatible, unsupported, stale, incomplete, refused, and failed are not
  successful Boolean results.
- Require an exact ecosystem lock for every composed run. Do not select ambient
  defaults, infer compatibility from graph reachability, or infer relationships
  from repository layout.
- Evaluators are pure. Only an explicitly authorized adapter may request an
  external effect, using stable request and idempotency identities; receipt and
  observed business effect remain distinct.
- Keep `publish = false`; use local tests only and do not add or dispatch hosted
  workflows without an explicit owner decision.
- Do not copy source material, fixtures, or schemas whose distribution rights are
  unresolved. Follow `CONTENT_RIGHTS.md` before the first commit and every import.
- Run `make fmt-check`, `make lint`, `make test`, `make build`, `make deny`, and
  `make audit-unsafe` before requesting review.
