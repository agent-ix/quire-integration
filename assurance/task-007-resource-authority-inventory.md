# Task 007 resource and authority inventory

This inventory is the static evidence for TC-009 through TC-011. It covers every production
module currently reachable from `src/lib.rs`. “Before” means the named guard dominates the
retention, traversal, persistence, or invocation in the same function; the executable static
suite checks the listed source ordering and rejects new ambient or unchecked numeric surfaces.

## Untrusted resource boundaries (TC-010)

| ID | Module / boundary | Untrusted value | Checked guard and caller-lowered ceiling | Protected operation |
|---|---|---|---|---|
| RB-01 | `common::identity_type::new` | identity text bytes | checked `usize`→`u64`, `max_identity_bytes`, nonblank | `to_owned` retention |
| RB-02 | `common::CanonicalEncode::canonical_bytes` | measured encoded length | `canonical_len`, `max_output_bytes`, checked `u64`→`usize` | exact-capacity output allocation |
| RB-03 | `common::append_bytes` / `append_field` | appended field length | checked conversions and additions, `max_output_bytes` | buffer extension |
| RB-04 | `common::Meter::consume` | work increment | `checked_add`, dimension-specific `Budget` member | counter mutation |
| RB-05 | `common::CanonicalMap::try_from_entries` | entry inventory | checked length, `max_items` | sorted-map retention |
| RB-06 | `common::CanonicalSet::try_from_values` | value inventory | checked length, `max_items` | sorted-set retention |
| RB-07 | `ecosystem_lock` value constructors | revisions, remotes, versions, features and contracts | checked field bytes, feature count and exact SemVer/revision grammar | owned lock-selection fields |
| RB-08 | `ecosystem_lock::validate` | complete selection counts, fields and work duration | `preflight(selection, limits, timer)` plus per-item monotonic checks | graph traversal and canonical wire construction |
| RB-09 | `ecosystem_lock::validate` output | canonical lock bytes | allocation-free `preflight_output`, `max_output_bytes`, monotonic writer checks | exact-capacity serialization |
| RB-10 | `ecosystem_lock::read` | lock document bytes | checked count, `max_document_bytes` | independent-selection validation/comparison |
| RB-11 | `ecosystem_lock::BoundedWriter::write` | serializer chunk bytes | checked current + incoming, writer maximum | output extension |
| RB-12 | `execution_bundle::Coordinator::new` | request item inventory and field bytes | `max_items`, `validate_request_limits` | request retention and later traversal |
| RB-13 | `execution_bundle::Coordinator::execute` | artifact bytes and producer work | `max_artifact_bytes`, meter input/invocation budgets | strict read and producer evaluation |
| RB-14 | `execution_bundle::OwnerResult::new` | owner bytes and independent axes | per-result and field ceilings | owned result retention |
| RB-15 | `execution_bundle::read` | canonical bundle document | `max_output_bytes`, allocation-free `preflight_document` | Serde allocation/parsing |
| RB-16 | `execution_bundle::read` groups | result group total | checked group sum, `max_items`, checked `u64`→`usize` | result-vector capacity and traversal |
| RB-17 | `effect_reconciliation` identity constructors | effect identity text | checked byte conversion and `max_identity_bytes` | owned identity retention |
| RB-18 | `EffectRequest::from_persisted` / request validation | arguments, owner artifact, canonical request and history | named `Limits` fields and checked arithmetic | request reconstruction/persistence |
| RB-19 | `EffectCoordinator::coordinate` | submission, limits and deadline | full validation, exact lock admission, remaining limits | store access and dispatch path entry |
| RB-20 | `EffectCoordinator::prepare_and_dispatch` | history, attempts and derived attempt id | checked additions, history/attempt/identity ceilings | attempt-id allocation, persistence, adapter invocation |
| RB-21 | `EffectCoordinator::observe` | observation identity and payload | identity and `max_observation_bytes` checks, remaining duration | store read, authority verification and link |
| RB-22 | persisted effect decoder / canonical writer | length-prefixed fields and output chunks | checked `u64`→`usize`, remaining input, request/output ceilings | slicing, owned decoding and output extension |
| RB-23 | `consumer_views::ViewCoordinator::query` | owner/query and reader output | owner validation, exact lock admission, lowered `CallBudget` | reader invocation and graph traversal |
| RB-24 | semantic projection validation | nodes, edges, sources and traversal work | named `Limits`, checked totals and timer | canonical retention and frontier traversal |
| RB-25 | `ConsumerOutputSink::submit` | borrowed consumer bytes, regions and codes | item/aggregate bytes, mappings, count and identity bounds | cloning into coordinator-owned output |
| RB-26 | `ConsumerCoordinator::consume` | manifest and obligation inventory | exact capability/profile checks, `max_obligations`, per-item output bounds | obligation cloning and consumer invocation |
| RB-27 | output-region validation / canonical writer | region offsets and accounting chunks | payload length, checked `u64`→`usize`, UTF-8 boundaries, output maximum | indexing and buffer extension |
| RB-28 | `compatibility_impact::ProjectionSink` | streamed headers/items and reasons | per-kind counts, document/retained/reason limits and checked totals | projection retention |
| RB-29 | `ImpactCoordinator::classify` | six owner documents/readers | complete inventory, exact lock admission, lowered `ReaderBudget`, timer | reader invocation and semantic validation |
| RB-30 | compatibility graph/policy/corpus/evidence validation | nodes, edges, rules, records, vectors and providers | named `Limits`, exact/one-over preflight and checked totals | sets/maps, traversal, SCC and witness work |
| RB-31 | compatibility canonical/count writers | envelope fields and output chunks | count-only length pass, checked additions/conversions, output ceilings | canonical body/envelope allocation |
| RB-32 | every monotonic timer | elapsed duration | `checked_sub`, checked `u128`→`u64`, nonzero remaining caller budget | reader/producer/consumer/adapter work |

All externally supplied integers remain `u64` until a platform index/capacity is required. The
production tree contains no unchecked numeric `as` conversion. Exact and one-over executable
evidence is dual-tagged TC-007 in the common, lock, execution, effect, consumer, and impact
suites; deadline-derived outcomes are asserted separately from canonical equal-input output.

## Authority and call graph (TC-009 / TC-011)

| ID | Edge | Static result |
|---|---|---|
| AB-01 | `src/*` → external effect adapter | exactly one `.dispatch(` call, inside private `EffectCoordinator::invoke_adapter` |
| AB-02 | persistence → dispatch | `prepare_attempt` and persisted `State::Open` validation dominate `invoke_adapter`; runtime history asserts `RequestPersisted`, `AttemptPrepared`, then receipt |
| AB-03 | caller → dispatch permit | `DispatchPermit` fields are private and there is no public constructor |
| AB-04 | execution strict reader/producer | requires constructor-private `AdmittedCapability`; receives caller meter/limits; owns no store or adapter handle |
| AB-05 | view and impact strict readers | require exact `AdmittedCapability` plus lowered `CallBudget`/`ReaderBudget`; own no store or adapter handle |
| AB-06 | renderer/analyzer/code generator | `ConsumerKind` is closed; outputs remain facade/accounting values and implement no producer, persistence, evidence, decision, or adapter authority |
| AB-07 | shared capability declarations | five distinct selection-only marker traits; all invocation methods live on feature-specific coordinator traits with opaque admissions, and no production blanket implementation or conversion edge exists |
| AB-08 | ambient/unsafe surface | no environment, filesystem, network, command, global registry, default provider lookup, or unsafe block in production |
| AB-09 | effect decision/observation authorities | exact lock admission precedes each pure verification call and each call receives a lowered `CallBudget`; neither authority receives store or adapter capability |

The static suite scans all eight production source files, not only public exports. A new source module,
adapter invocation, ambient lookup token, unchecked cast, or capability import therefore requires
an explicit inventory update and review rather than silently widening authority.

## Reviewed source snapshot

The inventory is pinned to the complete production source bytes below. Any production edit fails
TC-010 until the whole affected module is re-audited and its digest deliberately advanced.

| Source | SHA-256 |
|---|---|
| `src/capability.rs` | `75a3a534aa5c71c7d39e488234764241938b1c68a68d619b7bdf95daa49eea6a` |
| `src/common.rs` | `6ed9f39f7fa784a0507100cb2175cd28deddfa836b2b72b0ea4e37b209ddc7f8` |
| `src/compatibility_impact.rs` | `42d5f340ada8d3ba73a817ef8509db90472d8d7c372a9c211c141f373b4f2c9a` |
| `src/consumer_views.rs` | `8f10a87509d372c7ca9f7e59d1c980e1614cdbc8563fab30beb9b1e96a550d84` |
| `src/ecosystem_lock.rs` | `0f6fd654ea9274bdc098bbab45c06a2b22dad6ff573786b274da466e0f57ee9b` |
| `src/effect_reconciliation.rs` | `dbaaae6c6ce5e5b2e5085c762240874292529da2a6121dc85795fb1e62781f10` |
| `src/execution_bundle.rs` | `16d269b08431932ac24e12062a89e5b8cda33440d53dd90d3b1bc30729f86939` |
| `src/lib.rs` | `cfa0a6e7d35e3098c862ed2c3289a41b9cf5f90ac67b290fad870adb8ed194cb` |
