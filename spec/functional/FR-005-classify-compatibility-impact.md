---
id: FR-005
title: "Classify compatibility impact and stale evidence"
type: FR
relationships:
  - target: ix://agent-ix/quire-integration/US-003
    type: implements
  - target: ix://agent-ix/quire-integration/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-307
    type: references
  - target: ix://agent-ix/quire-specification/FR-310
    type: references
  - target: ix://agent-ix/quire-specification/FR-311
    type: references
  - target: ix://agent-ix/quire-specification/FR-336
    type: references
  - target: ix://agent-ix/quire-specification/FR-337
    type: references
---
# FR-005: Classify compatibility impact and stale evidence

## Description

When an exact candidate is compared with an exact baseline, the library SHALL
emit one canonical impact envelope whose structural, behavioral, presence,
reachability, evidence-freshness, vector-requirement and vector-availability
axes remain independent.

## Inputs

Validated baseline and candidate locks; canonical bytes plus exact lock-admitted bounded
strict-reader selections for `quire.impact-comparison/v1`,
`quire.impact-evidence-graph/v1`, QSpec FR-336 `quire.conformance-corpus/v1` and
QSpec FR-337 `quire.evidence-intake/v1`, and
`quire.regression-vector-manifest/v1`; canonical bytes plus exact selected bounded
strict readers for the closed versioned `quire.impact-policy/v1` and every owner
contract above; canonical
`quire.unresolved-impact-dependency/v1` overlay bytes; and finite input, graph,
topology, traversal, witness, frontier, evidence-record, policy, vector, output and
duration limits from the closed `Limits` record below.
The readers' constructor-private validated projections are internal results, not caller inputs.

## Outputs

A canonical impact envelope containing the exact validated projections, independent
assessment axes, per-subject reachability/freshness overlays, cycle/unresolved
witnesses, exact usage and frontier, and one record for every catalog vector.

## Behavior

### Identity and owner contracts

- Task 006 owns the versioned feature-local `quire.impact-comparison/v1` projection;
  it is not an FR-307 owner value. Its strict reader SHALL receive caller-lowered
  `max_document_bytes`, `max_identity_bytes`, `max_correspondences`, `max_assessments`
  and `max_changed_roots` before decoding and SHALL build only through a bounded
  coordinator-owned sink. Task 010 adapters populate it from exact
  FR-307 correspondence and real comparison/result-owner values without relabeling them.
- Task 006 likewise owns `quire.impact-evidence-graph/v1`, a canonical feature-local
  projection of the exact FR-310 node/relation/scope/provenance fields, and
  `quire.unresolved-impact-dependency/v1`, the separate unresolved overlay. These are
  integration facade contracts, not new upstream FR-310 owner values. Their strict
  reader receives `max_document_bytes`, `max_identity_bytes`, `max_graph_nodes`,
  `max_graph_edges`, `max_unresolved_entries`, `max_reason_bytes` and
  `max_total_reason_bytes` before decoding and uses a bounded sink.
- The comparison projection SHALL bind the baseline lock identity, candidate lock
  identity, direction-specific subject correspondence, comparison rule identity,
  revision and digest, and change-set identity and digest. Node keys include kind,
  authority, revision and owner-local identity. No package, repository, display
  name, schema bytes or equal local text substitutes for those keys.
- Each correspondence explicitly carries optional baseline and candidate subject keys,
  so addition and removal are directional rather than synthesized by name matching.
  Multiple changes are keyed by correspondence identity and conflicting or duplicate
  assessments refuse.
- A comparable behavioral-result key consists of exact subject, vector, method/provider,
  environment, assumptions/domain and lock identities. Behavioral `absent` means
  that key is required by a closed owner inventory and has no retained result; silence in
  an open collection is not absence.
- Structural assessment is closed over `unchanged`, `additive-compatible`,
  `payload-breaking`, `removed` and `unknown`. Behavioral assessment is closed over
  `not-assessed`, `conforming`, `regressed`, `absent`, `unsupported`, `unavailable`
  and `unknown`. Subject presence is closed over `both-present`, `baseline-only`,
  `candidate-only` and `unknown`. No axis manufactures, suppresses or relabels another.
- Only explicit owner tuples are admitted: `unchanged` and `payload-breaking` require
  `both-present`; `removed` requires `baseline-only`; `additive-compatible` permits
  `both-present` or `candidate-only`; and `unknown` permits any presence. `conforming`
  or `regressed` requires `both-present` plus both comparable results; `absent` requires
  `both-present` plus a missing candidate result in the closed comparable inventory.
  Other behavioral states may accompany any structurally admitted tuple. A tuple outside
  this table is an owner contradiction and refuses rather than becoming an impact state.
- Equal lock identities with a nonempty change set or mismatched correspondence
  SHALL refuse as contradictory. An empty owner change set under equal or distinct
  locks SHALL produce a complete no-reported-change envelope, not a compatibility
  claim. Swapping baseline and candidate SHALL create a distinct directional identity.
- The feature-local graph projection SHALL bind the exact FR-310 typed node identities,
  closed relation, scope and provenance values without claiming an upstream canonical
  wire that FR-310 does not define. Any
  feature-local propagation orientation is a separate policy overlay and SHALL NOT
  broaden, rename or reconstruct the FR-310 contract.
- The graph projection SHALL also carry closed authority, relation, scope and provenance
  vocabularies emitted one item at a time through the bounded sink in any order. Each vocabulary is
  exactly the distinct set used by the retained nodes, owner edges and unresolved
  entries: missing, duplicate, foreign or unused members refuse. Vocabulary cardinality
  is therefore bounded by the graph-node, graph-edge and unresolved-entry ceilings plus
  document bytes without changing any limit's accounted unit; an all-empty graph uses
  four empty vocabularies.
- Each evidence node references an exact FR-337 evidence identity. The separately
  strict-read `quire.evidence-intake/v1` inventory supplies its immutable canonical
  owner bytes, rights and limitations; the graph neither embeds a rewritten intake
  value nor invents one. Missing, duplicate, foreign or mismatched references refuse.
  The intake reader receives caller-lowered `max_document_bytes`, `max_identity_bytes`,
  `max_evidence_records`, `max_evidence_rights` and `max_evidence_limitations` and builds
  only through a bounded coordinator-owned sink.

### Canonical impact traversal

- The policy SHALL explicitly name every admitted `(relation, source-kind,
  target-kind, direction)` propagation rule, its exact scope matcher and its effect
  subset of `{affectedness, staleness}`; the subset may be empty or contain both effects.
  The only scope-matcher grammar is `exact(qualified-scope-id)`, compared by complete
  owner identity bytes. Empty, duplicate, unknown, wildcard, regular-expression,
  hierarchical or ambient matchers refuse. Multiple exact rules express a finite union.
- The policy also contains a closed, duplicate-free seed table mapping every admitted
  structural/behavioral/presence tuple to one explicit initial effect subset. Every
  changed root SHALL match exactly one seed entry. For each `(changed-root, effect)` in
  that subset, the library runs canonical breadth-first traversal only across oriented
  edges whose matching rule contains that effect. Equivalently, edge transfer is
  `outgoing_effects = incoming_effects ∩ rule.effects`; an empty result is a barrier and
  is not enqueued. Effects accumulated at a `(changed-root, qualified-node)` are unioned
  monotonically; only a newly acquired effect is enqueued, so converging paths carrying
  different effects both propagate. It SHALL NOT infer edges from reachability outside
  the projection, names, layout, repository membership or equal schemas.
- Roots, nodes and outgoing edges are ordered by their complete qualified identities,
  then effect, relation, scope and provenance. The visited key is
  `(changed-root, effect, qualified-node)`; equivalently, traversal runs independently
  per root and effect. Self-loops, diamonds and shared descendants terminate;
  disconnected nodes remain unaffected. Every reached `(changed-root, subject, effect)`
  retains one canonical shortest witness, with lexicographic full-edge tie-breaking.
  Subject overlays are keyed per `(changed-root, subject)` and retain the independent
  effect witnesses, so distinct roots or converging effects never collapse.
- A cycle encounter SHALL emit a canonical finite cycle witness. A discharge subgraph
  whose sole support is circular remains `unknown`; an independently produced evidence
  node may ground its own path but the cycle never supports itself. Cycles and unresolved
  endpoints SHALL NOT poison unrelated or independently grounded branches.
- Cycle classification SHALL use strongly connected components independently over each
  effect-specific policy-oriented graph; an edge whose rule omits that effect cannot
  create its cycle. An SCC is cyclic only when it contains multiple nodes or a self-loop;
  a diamond edge to an already visited node in an acyclic SCC is not a cycle. For each
  `(changed-root, effect, reachable-cyclic-SCC)`, retain exactly one shortest simple
  directed cycle; among ties, choose the lexicographically smallest full-edge sequence
  after rotating it to its lexicographically smallest edge. Direction is never reversed
  for normalization.
- A missing endpoint inside the strict FR-310 projection SHALL refuse before traversal.
  Valid unresolved dependencies, when required by the impact policy, SHALL be separate
  feature-local overlay entries carrying exact root, predecessor, absent target identity,
  relation, scope, provenance, reason and a digest over those fields; duplicates refuse.
  They produce item-local `unknown`. A malformed
  or authority-foreign node, endpoint, relation, scope or provenance SHALL refuse.
- Reachability is closed over `affected`, `unaffected` and `unknown`. Aggregate retained
  envelope state is closed over `complete` and `incomplete`; structural refusal and
  admitted strict-reader failure return typed `refused`/`failed` errors with no envelope.

### Bounds and phase outcomes

- The caller SHALL supply exactly this closed `Limits` record. Every field is a nonzero
  `u64`, is lowered into the applicable strict reader before invocation and has no ambient
  fallback. `max_document_bytes` applies independently to each canonical input document;
  `max_total_input_bytes` applies to their checked sum. Per-value fields apply to each
  retained value; every other field is aggregate for one impact request.

  | Field | Exact unit |
  |---|---|
  | `max_document_bytes`, `max_total_input_bytes` | bytes per document; bytes across all documents |
  | `max_identity_bytes`, `max_reason_bytes` | bytes per identity; bytes per bounded reason |
  | `max_correspondences`, `max_assessments`, `max_changed_roots` | retained comparison entries of each kind |
  | `max_graph_nodes`, `max_graph_edges` | retained graph nodes; retained owner edges |
  | `max_evidence_records`, `max_evidence_rights`, `max_evidence_limitations` | retained FR-337 records, rights and limitations |
  | `max_reader_invocations` | total selected strict-reader calls |
  | `max_policy_seed_rules`, `max_policy_propagation_rules`, `max_policy_vector_rules` | retained policy seed, propagation and vector-selection rules |
  | `max_vector_definitions`, `max_manifest_providers`, `max_manifest_entries` | retained FR-336 definitions; retained complete provider selections; retained provider-manifest entries |
  | `max_unresolved_entries`, `max_total_reason_bytes` | unresolved overlay entries; bytes across every reason |
  | `max_topology_items` | graph nodes plus distinct oriented edge/effect pairs |
  | `max_traversal_steps`, `max_traversed_edges` | dequeued root/effect/node entries; examined effect-carrying edges |
  | `max_subject_records` | retained root/subject overlays |
  | `max_path_edges`, `max_witnesses`, `max_witness_bytes` | retained edge references, witness records and encoded witness bytes |
  | `max_frontier_entries` | retained traversal frontier entries |
  | `max_output_bytes`, `max_accounting_bytes` | canonical body bytes; complete envelope bytes |
  | `max_duration_ms` | operational monotonic milliseconds |

  Collection and byte limits are checked with checked arithmetic before retaining the
  next value. Exceeding any input/reader/policy/vector/topology field refuses before
  traversal. The complete canonical subject-overlay inventory is checked before graph work;
  exceeding it is incomplete with every actual seeded root/effect unit in the frontier.
  Exceeding traversal/path/witness fields is incomplete with the exact
  traversal frontier when that frontier fits; frontier or encoding overflow refuses.
- Topology demand is computed atomically before SCC analysis as the number of qualified
  graph nodes plus distinct policy-oriented edge/effect pairs after exact-scope matching.
  If it exceeds the topology-item ceiling, the request refuses before SCC analysis and
  emits no topology or traversal frontier. This cardinality preflight bounds any complete
  SCC/shortest-cycle implementation without exposing algorithm-specific stack, index or
  lowlink state. One traversal step dequeues and expands one
  `(changed-root, effect, qualified-node)` entry. One traversed edge examines one
  effect-carrying oriented edge for that root. Path-edge and witness-byte usage count all
  retained edge references and encoded witness bytes.
- Before expanding the next canonical edge, the boundary SHALL charge its work and
  retention dimensions. Traversal SHALL stop before a ceiling is exceeded and emit
  aggregate `incomplete`, exact usage and the canonical immediate unexpanded edge
  frontier while preserving completed independent siblings. If that frontier or the
  accounting envelope cannot fit, the request SHALL refuse without a partial
  completeness claim.
- Every traversal frontier and cycle entry SHALL carry changed root, effect, predecessor,
  target, relation, scope, provenance and depth so independent per-root/effect traversal
  can resume without reconstruction.
- Duration is an operational wall-clock ceiling measured by an injected monotonic clock.
  Expiry returns typed `failed(duration)` with no impact envelope or partial siblings;
  elapsed time and cutoff-dependent partial state never enter canonical bytes. Successful
  canonical envelopes are determined solely by validated inputs and deterministic work
  ceilings.
- Checks use one precedence. Preflight checks input bytes, identity lengths, collection
  counts, duplicates/rebindings/digests, lock/capability admission, reader-invocation
  ceilings and cross-field rules, in that order. At each canonical work boundary,
  the deterministic subject inventory is checked first; traversal ceilings are then checked
  in this order: traversal steps, traversed edges, path edges, witness count, witness bytes
  and frontier.
  Duration is checked at public entry, before/after each reader, before each topology or
  traversal expansion after same-boundary deterministic checks, and before encoding.
  Output/accounting overflow refuses during encoding with no envelope.
- Invalid lock/correspondence/owner graph/rule/policy identity refuses before traversal;
  an admitted strict-reader failure or duration expiry is `failed` with no envelope;
  valid unresolved or unsupported facts are item-local; and traversal exhaustion is
  `incomplete`. None defaults to compatible, incompatible, current, supported, available
  or passed.

### Evidence freshness and regression vectors

- Evidence freshness is a non-authoritative overlay keyed by exact baseline subject
  revision and evidence identity. It is closed over `current`, `stale` and `unknown`
  and preserves the original FR-310 `supports`/`contradicts` relation, owner bytes,
  scope and provenance. A policy staleness path changes current evidence exactly as the
  table specifies; stale
  evidence is not current support, counterevidence, semantic falsehood or evidence for
  the candidate. Freshness follows the complete table below, where staleness propagation
  is independently `reached`, `not-reached`, `unknown` or `incomplete`. A witness is
  required only when this run changes `current` to `stale`; previously stale evidence
  remains stale without manufacturing a new changed-root witness:

  | Prior freshness | reached | not-reached | unknown/incomplete |
  |---|---|---|---|
  | current | stale | current | unknown |
  | stale | stale | stale | stale |
  | unknown | unknown | unknown | unknown |
- The impact policy SHALL be an immutable pure value with identity, revision, digest
  and duplicate-free closed rules over the independent axes. It SHALL contain no
  callbacks or ambient defaults. Its vector catalog becomes immutable after structural
  validation. The `quire.impact-policy/v1` strict reader SHALL receive caller-lowered
  `max_document_bytes`, `max_identity_bytes`, `max_policy_seed_rules`,
  `max_policy_propagation_rules` and `max_policy_vector_rules` before decoding and SHALL
  build only through a bounded coordinator-owned sink. The FR-336 corpus reader receives
  `max_document_bytes`, `max_identity_bytes` and `max_vector_definitions` and likewise
  builds through a bounded sink.
- Every FR-336 catalog vector SHALL retain an independent requirement axis
  (`required`/`not-applicable`) and availability axis
  (`available`/`unsupported`/`unavailable`/`unknown`). Every catalog vector SHALL occur
  exactly once in canonical output; required vectors are the exact selected subset. A
  missing referenced definition, unknown policy
  member or conflicting duplicate SHALL refuse the policy; an admitted definition with
  no exact candidate provider is `unavailable`; a selected provider contract that
  explicitly lacks the vector is `unsupported`; an admitted manifest may explicitly
  report `unknown` with a bounded reason. Availability SHALL be derived only from the
  exact candidate lock's complete capability selection plus strict-read unchanged provider,
  build, contract, options and vector-manifest values; an untrusted request cannot assert it.
  Execution/pass/waiver dispositions are
  owned by Tasks 010/011 and SHALL NOT be invented here.
- The `quire.regression-vector-manifest/v1` strict reader SHALL receive lowered
  `max_document_bytes`, `max_manifest_providers`, `max_manifest_entries`, `max_reason_bytes` and
  `max_total_reason_bytes` before decoding and SHALL build through a bounded sink.
  A vector key contains vector definition, subject/scope, method, environment,
  requested provider-role and policy-profile identities; selected provider identity is
  a separate optional field. Converging rules produce one record per complete key
  retaining every contributing rule identity; equal local vector text under a distinct
  subject or scope remains a distinct record.
- Availability follows one table: exactly one candidate-lock-admitted provider whose
  strict manifest supports the vector is `available` with that provider; exactly one
  admitted provider whose manifest explicitly excludes it is `unsupported` with that
  provider; no admitted matching provider is `unavailable` with no selected provider;
  an explicitly indeterminate admitted manifest entry is `unknown` with its bounded
  reason; and multiple matches, foreign/mismatched identities or malformed manifests
  refuse. Requirement `not-applicable` does not remove the catalog record or rewrite
  availability.

### Canonical envelope

- Exact duplicate keys and conflicting rebindings in owner projections, policy or
  vector inputs SHALL refuse. Roots, records, witnesses, frontier entries and vector
  records use the canonical orders above.
- The domain-separated envelope identity SHALL be the digest of the canonical envelope
  body with its identity field excluded. The wire carries that digest and strict reading
  recomputes/rejects it. The digest preimage includes baseline/candidate lock IDs,
  complete owner projection bytes and identities, change-set/graph/rule/policy/vector
  identities, all deterministic limits, every independent state and canonical body byte. Equal
  permutations reproduce bytes; any meaning-affecting mutation changes identity.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | For every tuple admitted by the structural × behavioral × subject-presence table, output retains exactly the input tuple and no value on one axis changes, suppresses or synthesizes another; every tuple outside the table refuses as an owner contradiction. | property-based-testing (TC-005) |
| FR-005-AC-2 | For every finite resolved graph and input permutation, each affected subject and each evidence record newly changed from current to stale contains the canonical shortest typed witness from its exact changed root and effect; converging paths with different effects both propagate, with byte-identical ordering and owner identities. | property-based-testing (TC-005) |
| FR-005-AC-3 | For all finite admitted graphs and bounds, the phase mapping is invariant: malformed/foreign or one-over input, reader, policy, vector, topology and encoding dimensions refuse; cycle-only or unresolved premises are item-local unknown; exact work completes; one-over traversal/path/witness/subject retention is incomplete with its exact frontier when that frontier fits; frontier overflow refuses; and topology never writes a compatibility axis. | property-based-testing (TC-005) |
| FR-005-AC-4 | For all owner-relation × prior-freshness × staleness-propagation combinations, output preserves the owner relation/provenance and applies exactly the freshness table; stale support is never current support, contradiction, candidate truth or semantic falsehood. | property-based-testing (TC-005) |
| FR-005-AC-5 | For every valid policy and input permutation, each catalog vector appears once with independent requirement and availability states; duplicate/conflicting/missing definitions refuse and unavailable/unsupported providers never become executed or passed. | property-based-testing (TC-005) |
| FR-005-AC-6 | For every permutation of equal set-valued inputs, canonical body bytes and identity are equal; changing direction, any qualified identity, owner value, rule, policy, vector, deterministic limit or state changes identity. | property-based-testing (TC-005) |

## Error Conditions

An invalid baseline/candidate lock, correspondence, owner projection, foreign graph
value, unknown rule/policy member, conflicting duplicate, missing vector definition or
malformed path refuses before traversal. An admitted strict-reader failure is failed.
Duration expiry is `failed(duration)`. Either failure returns no impact envelope or
partial sibling state.
Valid cycles without independent evidence and unresolved endpoints are item-local unknown;
valid missing providers are unavailable, explicit provider exclusions are unsupported,
and traversal exhaustion is incomplete with exact usage/frontier. None defaults to
compatible, incompatible, current, supported, available, executed or passed.

## Dependencies

- [FR-001](./FR-001-validate-exact-ecosystem-lock.md) supplies baseline and
  candidate lock identities.
- QSpec FR-310 owns typed semantic-subject/evidence relationships.
- QSpec FR-307 owns package compatibility/migration correspondence, FR-336 owns immutable
  conformance-corpus/vector definitions, FR-311 owns qualification execution/accounting,
  and FR-337 owns immutable evidence-intake bytes. Task 006 owns its feature-local
  comparison/graph/manifest projections and overlays only; Task 010 owns strict real-owner
  adapters/pass-through and Task 011 owns qualification and release decisions.
