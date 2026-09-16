---
id: FR-004
title: "Compose bounded deterministic consumer views"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-003
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-304
    type: references
  - target: ix://agent-ix/quire-specification/FR-309
    type: references
  - target: ix://agent-ix/quire-specification/FR-269
    type: references
  - target: ix://agent-ix/quire-specification/FR-297
    type: references
  - target: ix://agent-ix/quire-specification/FR-298
    type: references
  - target: ix://agent-ix/quire-specification/FR-299
    type: references
  - target: ix://agent-ix/quire-specification/FR-333
    type: references
  - target: ix://agent-ix/quire-specification/FR-334
    type: references
---
# FR-004: Compose bounded deterministic consumer views

## Description

When a model view, renderer, analyzer, or code generator is requested, the
integration boundary SHALL account for every requested fact or obligation.

## Inputs

Exact versioned checked/IR owner artifact and its validated immutable projection,
including package identity/digest, qualified semantic/source identities and typed
edges; a closed typed query containing the exact package digest, subject root,
relation, pure filter and projection; exact feature-local consumer capability selection,
profile binding and requested obligation identities; and finite input,
traversal, result, frontier, accounting, invocation, output and duration limits.

## Outputs

Canonical semantic view and identity-to-region map, or analysis/generated artifact
accounting with per-item disposition, provenance, mapping loss, usage and omitted
frontier. A structurally admitted request always returns an accounting envelope;
only input too malformed or over-bound to retain safely returns a bare refusal.

## State model

- Aggregate coordination state is exactly `complete`, `incomplete`, `refused`, or
  `failed`.
- Facade item state is exactly `displayed`, `omitted`, `generated`, `unsupported`,
  `requires-bound`, `invalid`, or `failed`. These labels describe coordination and
  presentation, not semantic truth.
- Mapping loss retains the exact FR-269 owner value `preserved`, `conditional`,
  `unrepresented`, or `refused`. Mapping loss is a separate axis and SHALL NOT be
  inferred from, collapsed into, or substituted for aggregate or item state.
- Unsupported, requires-bound, invalid and failed items carry no output region or
  substitute artifact. Supported siblings may remain in an otherwise valid aggregate;
  global refusal or output exhaustion exposes no plausible partial output package.

## Behavior

- The boundary SHALL revalidate the exact owner artifact/projection, package digest,
  query, requested-obligation inventory, feature-local consumer admission tuple and complete
  lock-admitted capability tuple before traversal or invocation. An untrusted
  request SHALL NOT select its implementation.
- A conforming selected `quire.semantic-consumer/v1` implementation SHALL be pure and
  deterministic over the admitted immutable view, obligation inventory, profile, exact
  lock and finite limits. It SHALL NOT observe undeclared clock, filesystem, network,
  locale, path or mutable ambient state; repeated equal inputs SHALL produce equal results.
- Filter and projection are closed, versioned, pure declarative values. An unknown
  member refuses before traversal; callbacks, locale, path, clock and ambient defaults
  are not admitted inputs.
- The boundary SHALL perform canonical breadth-first traversal from the included
  exact root. Each depth is ordered by qualified semantic identity; outgoing selected
  edges are ordered by target identity and source region; visited identity is the
  qualified semantic identity. Cycles and shared descendants are visited once. Valid
  non-selected relations are ignored; a foreign or unknown selected relation refuses.
- The filter is applied after discovery and does not prune reachability. Every visited
  fact is accounted exactly once as displayed or omitted before projection.
- Before expanding the next canonical frontier node, the boundary SHALL charge one
  traversal step. The already validated canonical adjacency permits the unexpanded
  queue to be retained without further semantic discovery.
- The boundary SHALL stop before any finite work, result or retention bound is
  exceeded. The omitted frontier is the canonical immediate queue of unexpanded
  selected-relation edges, including predecessor, target, relation and exact source
  identity; it is not a claim about the undiscovered transitive closure.
- When traversal is truncated and that queue fits its independent frontier/accounting
  limits, the boundary SHALL report `incomplete`, `frontier_complete = true`, exact
  usage and the exact omitted frontier. If the frontier/accounting envelope itself
  cannot fit, the boundary SHALL refuse without a partial completeness claim.
- The explicit requested-obligation set becomes immutable after structural validation.
  The bounded query result inventory becomes immutable when traversal stops. The
  boundary SHALL account for every member of each applicable inventory exactly once.
- A foreign package/query subject root is a request-level refusal before traversal. A
  structurally retained obligation that names a fact outside the admitted bounded view is
  item-local `invalid`; valid siblings may still invoke exactly once and the aggregate
  cannot claim `complete`.
- The boundary SHALL preserve exact source and semantic identities for every
  output region, finding, and generated artifact.
- Region maps use ordered half-open byte ranges within the immutable output, must obey
  the selected profile's closed `utf8` or `binary` region-encoding rule (including UTF-8
  character boundaries when selected), and may neither duplicate an obligation
  nor cross-wire package, profile, semantic or source identities. Exact FR-298/FR-299
  owner records and packages are not redefined by this feature-local facade. Task 010's
  strict real-consumer adapters own full FR-297/FR-298/FR-299/FR-333/FR-334 validation and
  must retain their canonical owner bytes and identities when connecting this coordinator;
  the local admission, item-state and mapping-loss fields are not substitutes for them.
- An admitted consumer error, missing/extra result, malformed region/map, or foreign
  owner value SHALL become attributable `failed` accounting with no accepted artifact.
- A missing required finite item bound SHALL produce `requires-bound` without
  invocation. An unsupported obligation under the admitted profile SHALL produce `unsupported` without a
  substitute artifact. Output exhaustion SHALL produce `incomplete` accounting and no
  partial generated package.
- The boundary SHALL NOT infer semantic relationships from layout, adjacency,
  labels, heuristic findings, generated syntax, or parser acceptance. Consumer outputs
  are non-authoritative and cannot re-enter the checked-package constructor.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | A conforming selected consumer is pure and deterministic: equal admitted package/query/profile/lock/limit inputs produce byte-identical ordered views, artifacts, mappings, and loss records. | property-based-testing (TC-004) |
| FR-004-AC-2 | Every displayed, omitted, generated, unsupported, requires-bound, invalid or failed item appears exactly once and maps to its exact source and semantic identity; facade state and FR-269 mapping loss remain separate. | property-based-testing (TC-004) |
| FR-004-AC-3 | An exact work/result bound completes, while a one-over reachable input stops before excess and returns incomplete with exact usage, a complete canonical immediate frontier and no completeness claim. | property-based-testing (TC-004) |
| FR-004-AC-4 | Unknown/foreign package or query identity, relation, filter, projection, profile or lock mismatch refuses before invocation; foreign obligation subjects are item-local invalid without a complete aggregate; unsupported and missing-bound items are accounted without artifacts; admitted consumer failure is failed; and output exhaustion is incomplete with no partial package. | negative-abuse-testing (TC-004) |
| FR-004-AC-5 | Changing layout, labels, heuristic text/confidence, generated syntax or downstream parser acceptance cannot change checked source bytes, digest, facts, relations or mapping-loss classifications. | property-based-testing (TC-004) |
| FR-004-AC-6 | No view, finding, region map, generated artifact or parser observation can construct checked semantics or satisfy proof, qualification or release authority. | compile-time-check (TC-004) |

## Error Conditions

An unknown or foreign package/query subject, relation; unknown filter/projection; or
profile/manifest/lock mismatch is a stable typed refusal before invocation.
An unsupported obligation under the admitted profile and missing bounds are item-local
`unsupported` and `requires-bound`. Traversal/result exhaustion is `incomplete`
with an exact frontier; output exhaustion is `incomplete` with no partial package;
an admitted consumer error or malformed response is `failed`. No substitute
artifact, inferred relationship or completeness claim is emitted.

## Dependencies

- QSpec FR-304 owns deterministic model views.
- QSpec FR-309 owns analysis and code-generation consumer accounting.
- QSpec FR-269 owns the closed mapping-loss labels retained by this facade. QSpec
  FR-297 through FR-299 own the full profile, per-obligation record and generated-package
  contracts; Task 010, not Task 005, owns their strict adapter and canonical pass-through.
- QSpec FR-333 owns proof-strength/method outcomes and FR-334 owns full tool/plugin
  manifests. Task 005 neither reconstructs nor substitutes those values; Task 010 retains
  them through real adapters, and they cannot grant source, proof or qualification authority.
