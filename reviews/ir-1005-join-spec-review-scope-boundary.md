---
id: SR-2026110414560127
title: "spec-review/scope-boundary — partial observation revision admission"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-integration; IR-1005; PR 82; Cargo.toml, Cargo.lock, README.md, src/temporal.rs, src/temporal/observation_revision.rs, tests/support/observation_revision/mod.rs, tests/temporal_observation_revision.rs, IT-021, TC-114"
review_set: subset
---

## Summary

Ticket: IR-1005. Review context: agent-ix/quire-integration PR 82. No applicable AssuranceProfile exists under spec/. This independent review examines only the partial IT-021/TC-114 observation-input component. Full FR-350 acceptance, complete ValidatedTlJoin/join_tl/read_tl_join, selected observation-document lineage, QSL capture, native Report binding, paired native/TL predecessors and full owner-work accounting remain Planned.

## Verdict

PASS for the reviewed source/specification component: no new source defects identified. Adoption remains BLOCKED by the independently tracked unchanged dependency cohort failure. This source verdict grants no merge clearance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Integration owns only exact selected observation membership, bounded component input bytes and scan visits, typed composition refusals and immutable value retention (core responsibility). QObs owns canonical parsing, all eleven component semantics and historical bundle direct-predecessor authentication; that guarantee is exercised through the real owner contract and adverse controls. Owner internal work is separately bounded by native Limits and excluded from component work. Independently supplied Context/Selection remain caller expectations. Historical bundle lineage does not authenticate selected observation-document, QSL capture, native Report or native/TL result lineage. No current-head or no-fork claim is made. Future QSL-1732/1733/1734 work is outside this review and remains Planned.

Existing focused evidence was inspected, not rerun: six component tests, nine formula tests and one mapping-binding regression passed; scoped warnings-denied Clippy, formatting and IT/TC schema validation passed. Required unchanged dependency cohort evidence fails: canonical package identities actual 2, required 1 at tests/dependency_cohort.rs:89. Upstream normal dependency convergence remains required; no override, guard weakening, waiver or merge authorization is provided. No aggregate or final scoped gate ran. Gate execution was withheld by the dispatch, which overrides skill gate defaults.

## Examined units

```yaml
{
  "scope": [
    {
      "id": "IT-021:Objective",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Qualify a partial observation-input prerequisite for FR-350 using genuine owner\nvalues. The component SHALL admit only canonical v1 bundle bytes accepted by the\nQObs historical reader and containing the exact independently admitted observation.\nIt SHALL retain the admitted bundle, selected observation and actual prior receipt."
    },
    {
      "id": "IT-021:Target Integration",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Real QObs admission, document derivation, publication and strict readers \u2192 integration\n`read_observation_revision` \u2192 constructor-private `AdmittedObservationRevision`.\nNeither temporal evaluator is invoked."
    },
    {
      "id": "IT-021:Preconditions",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "The caller supplies independent QObs Context, bundle Selection and genuine observation\nView. A correction supplies its exact previous component receipt. QObs owns bundle\nlineage validation; historical reading establishes neither global current-head nor\nabsence of unknown forks. An unchanged observation may remain in a corrected bundle."
    },
    {
      "id": "IT-021:Inputs",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Canonical bundle bytes, Context, Selection, selected observation View, optional prior\nreceipt, native QObs Limits and component limits. Defaults are 16,777,216 bytes and\n65,536 scan visits. Component bytes count the supplied current bundle and selected\nobservation bytes before owner reading. Each record and component-stamp scan visit\ncosts one work unit, charged before inspecting that item. Owner derivation, parsing,\nallocation and internal comparisons use native limits and are excluded from component\nwork claims. Previous receipts are borrowed without recursive copying or traversal."
    },
    {
      "id": "IT-021:Expected Results",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "The public component returns only a partial observation revision receipt. Selected\nobservation-document lineage, QSL capture/native binding, duplicate valuation evidence,\ncomplete correspondence, native/TL result predecessors, `ValidatedTlJoin`, `join_tl`,\n`read_tl_join` and complete FR-350 resource accounting remain Planned. No final Boolean\nor agreement is established. This evidence does not complete any whole FR-350 criterion."
    },
    {
      "id": "IT-021:Dependencies",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Adoption is blocked by the unchanged IT-014 shared dependency cohort guard: the\nnormal QObs main dependency currently selects its upstream tagged canonical encoder\nalongside the existing main encoder. This component uses genuine QObs-owned types\nonly, but does not waive the one-copy cohort requirement. Upstream normal dependency\nconvergence is required before merge; no local override or policy exception is added."
    },
    {
      "id": "IT-021-SC-01",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Admit genuine initial and corrected v1 bundles through\n   `bundle::read_revision`, supplying the previous receipt's actual bundle View.\n   Compare complete returned typed owner values and bytes; retain the exact previous\n   receipt. Reject missing, foreign and non-direct predecessors using native errors."
    },
    {
      "id": "IT-021-SC-02",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Require exact canonical observation-document bytes in the bundle's\n   Observation record and its exact role, identity, contract and revision component\n   stamp. Reject foreign selected observations and equal-content envelope revision\n   substitutions. Admit a corrected bundle retaining the unchanged observation."
    },
    {
      "id": "IT-021-SC-03",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Delegate malformed, noncanonical and trailing-byte refusal to QObs;\n   preserve the original typed owner error. Expose stable Copy component codes for\n   owner refusal, absent exact observation and limit exhaustion, with no partial receipt."
    },
    {
      "id": "IT-021-SC-04",
      "path": "spec/integration/IT-021-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Check current-input bytes before owner reading and each scan visit\n   before inspection. Exact settings admit; one less refuses at setting+1. Each call\n   has fresh counters. Byte limits precede owner reads; scan limits precede selection\n   acceptance. Preserve all genuine owner values and prior bytes unchanged."
    },
    {
      "id": "TC-114:Description",
      "path": "spec/test-cases/TC-114-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Exercise the public partial observation admission component using integration-authored\ninputs passed through real QObs owner APIs, without copied owner fixture documents."
    },
    {
      "id": "TC-114:Test Procedure",
      "path": "spec/test-cases/TC-114-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "Run `temporal_observation_revision`. Compare genuine initial/corrected bundle and\nselected observation values and retained prior bytes. Test unchanged observation in\ncorrected bundles, foreign and revised observation substitutions, missing/foreign/\nnon-direct predecessors, malformed/noncanonical/trailing bytes and exact/one-over\ncomponent byte and scan-work limits. Assert native typed refusals and stable component\nerror codes. Run temporal mapping binding and formula construction regressions."
    },
    {
      "id": "TC-114:Expected Results",
      "path": "spec/test-cases/TC-114-observation-revision-admission.md",
      "role": "examined",
      "excerpt": "All IT-021 component criteria pass. Every rejection yields no receipt. No temporal\nevaluator runs and no full projection, correspondence or native/TL join is claimed."
    },
    {
      "id": "Cargo.toml",
      "path": "Cargo.toml",
      "role": "examined",
      "excerpt": "[package]\nname = \"quire-integration\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.98\"\ndescription = \"Version-locked composition and integration runtime for native Quire V1.\"\nauthors = [\"Agent IX <agent@agent-ix.dev>\"]\nlicense = \"AGPL-3.0-or-later\"\nrepository = \"https://github.com/agent-ix/quire-integration\"\npublish = false\n\n[lints.clippy]\nundocumented_unsafe_blocks = \"deny\"\n\n[lib]\nname = \"quire_integration\"\npath = \"src/lib.rs\"\n\n[dependencies]\nquire-observation = { git = \"https://github.com/agent-ix/quire-observation\", branch = \"main\" }\nqsl-package = { git = \"https://github.com/agent-ix"
    },
    {
      "id": "Cargo.lock",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "name = \"quire-observation\""
    },
    {
      "id": "README.md",
      "path": "README.md",
      "role": "examined",
      "excerpt": "# quire-integration\n\nquire-integration is the cross-repository integration test crate for the Quire\ntoolchain. It checks that the stages, each in its own repository, agree on the\nbytes they pass to each other.\n\nIt pins every stage to an exact commit in one `Cargo.lock`. Its tests admit\nchecked QSL packages through Contract IR, pass supported scalar operations to\nthe code generator, and exercise real proof and native paths. The table below\nstates which paths each test composes.\n\n<picture>\n  <source media=\"(prefers-color-scheme: dark)\" srcset=\"docs/images/hero-dark.svg\">\n  <img alt=\"Diagram of t"
    },
    {
      "id": "src/temporal.rs",
      "path": "src/temporal.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! The admitted subject/classification boundary of the FR-350 temporal bridge.\n//!\n//! This check admits no TL projection and invokes neither evaluator.\n\nuse qsl_package::checked_views::{\n    mapping_support, Classification, Closure, MappingRefusal, TemporalSubjectView,\n};\nuse quire_contract_ir::CheckedPackageV2;\n\n/// Why a supplied classification cannot be consumed for this admitted subject.\n#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum MappingBindingRefusal {\n    /// The subject borrows a different admitted package instance.\n    ForeignS"
    },
    {
      "id": "src/temporal/observation_revision.rs",
      "path": "src/temporal/observation_revision.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! IT-021: exact observation selection in a historical QObs bundle.\n\nuse quire_observation::authority::{bundle, observation, Context, Error, Limits};\n\n/// Ceilings for component input bytes and record/component scan visits.\n/// Owner-internal work remains subject to the separately supplied native limits.\n#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub struct ObservationAdmissionLimits {\n    /// Sum of current bundle and selected observation canonical bytes.\n    pub max_bytes: u64,\n    /// One unit before each embedded-record or component-stamp in"
    },
    {
      "id": "tests/support/observation_revision/mod.rs",
      "path": "tests/support/observation_revision/mod.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Integration-authored one-record QObs producer inputs.\n\nuse quire_observation::authority::{\n    self as a, bundle, AuthoritySelection, Context, Document, History, Limits, OpenClosed,\n    SubjectSelection, TemporalBoundary,\n};\nuse quire_observation::{\n    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,\n    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, QualifiedSubject,\n    ResourceLimits, ScopeKind, ScopeSelection, StaticProducerBundle, SubjectIdentity, SubjectKind,\n    ValueS"
    },
    {
      "id": "tests/temporal_observation_revision.rs",
      "path": "tests/temporal_observation_revision.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! TC-114: genuine QObs historical bundle and exact observation admission.\n\n#[path = \"support/observation_revision/mod.rs\"]\nmod support;\n\nuse ix_trace_rs::trace;\nuse quire_integration::temporal::{\n    read_observation_revision, ObservationAdmissionCode, ObservationAdmissionError,\n    ObservationAdmissionLimit, ObservationAdmissionLimits,\n};\nuse quire_observation::authority::{self as a, bundle, ErrorCode, Limits};\nuse support::Fixture;\n\nfn limits() -> ObservationAdmissionLimits {\n    ObservationAdmissionLimits::default()\n}\nfn owner_limits() -> Limi"
    }
  ]
}
```

## Dependency adoption disposition — round 1

The earlier source PASS and historical red dependency evidence above remain unchanged. The known adoption blocker is now resolved: The normal QObs main dependency now uses the existing canonical main encoder with its production serde_json feature. The lock removes the separate tagged canonical identity and updates only QObs; all other package identities remain unchanged. The actual unchanged IT-014 named cohort test passed one test with zero failures. No consumer override, local pin, guard weakening or policy exception was introduced.

The same reviewer inspected the normal dependency adoption and README/IT-021 status delta. Production component, fixture/tests, manifest and cohort guard are unchanged. No new source or specification findings were identified. Current review clearance is PASS for this partial component and dependency adoption, conditional only on the still-pending final scoped integration gate. No final gate, build, retest or census was performed by this reviewer. Full FR-350/native-TL join and QSL-1732/1733/1734 prerequisites remain Planned; this disposition grants no whole-criterion acceptance.

## Dispositions

There were no real FND rows to disposition; the original placeholder remains a placeholder. The separately tracked dependency adoption blocker is resolved by the evidence described above. Original findings, scope excerpts and review comments are preserved.
