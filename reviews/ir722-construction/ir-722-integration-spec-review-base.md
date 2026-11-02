---
id: SR-20261102072212
title: "IR-722 integration spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-722; tests/monitor_program.rs, IT-018, TC-098, Cargo.lock"
review_set: subset
---

## Summary

Reviewed quire-integration companion PR35, frozen 859976698b6bdafda0ae2a1d506146d4a42b4fce against origin/main 80cc87e7. Same IR-722 construction review carry, not execution acceptance. The live QSL emitter and strict IR reader are real and fail on absence/refusal; admitted-id conversion matches existing producer_origin_identity.rs. Controlled arenas are explicitly diagnostic, but declaration meaning substitution and unit provenance require the findings below. No RT/QSL/IR dependency boundary is changed by the lock update. No source edits or Cargo/Kani commands run by reviewer. Supplied evidence: 11 tests pass, all-target Clippy and changed spec validation pass; old RT negative unit control fails as intended. Full repository gates remain outstanding. No applicable AssuranceProfile found. EARS is inapplicable to this IT/TC addition; other lenses not selected. RT FND-004 remains open until the RT fixture is actually removed.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — Findings require disposition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The live source declares tick at scale 1 and offset 0, while the unit control rebuilds its graph under the same key with scale -1 or offset 7. The tested unit definitions were never emitted and strictly admitted, contrary to IT-018-SC-02 wording. Author and admit the actual unit variants before using their identities. | tests/monitor_program.rs:778-839 |

## Examined scope

```yaml
scope:
- id: IT-018
  path: spec/integration/IT-018-monitor-program-construction.md
  role: examined
  excerpt: 'These are controlled executable-part construction tests. They do

    not qualify a temporal plan producer, entry execution, or the complete FR-276
    monitor.'
- id: TC-098
  path: spec/test-cases/TC-098-monitor-program-construction.md
  role: examined
  excerpt: no test skips an unavailable boundary or remints an identity from constituent
    parts.
- id: tests/monitor_program.rs
  path: tests/monitor_program.rs
  role: examined
  excerpt: RT construction controls using identities from a live QSL producer and
    strict IR reader.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "quire-contract-runtime"
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified source and assertions at f9a59bda; exact after_excerpt below. |

Disposition round 1 inspected f9a59bda76a780434ce9f4d09596976994f9ba13. The source now declares matching record/tuple/union/function forms and shared diagnostic declarations match the authored field types. Semantic-form assertions reject enum-to-record/function substitution. The retained nested key is the actual admitted Step second-position sequence identity; an explicit nested PathSegment::Node assertion exercises storage without claiming temporal-root admission. Both special Time units are genuinely authored, emitted and strictly admitted, and their unit graph references/rationals are carried from their actual admitted bodies. The missing/mistyped-entry trace now names SC-03. No remaining defect found in this construction-only fix diff.

The born/amount diagnostic records and controlled arenas are explicitly not FR-487 monitor execution qualification. The actual full temporal producer is not required to test storage of an admitted nested key. Supplied evidence: eleven authentic construction controls and all-target Clippy pass; the authentic unit control fails against old RT with intended InvalidLayout and passes against RT728. Quire changed-doc validation passes. No Cargo/Kani or source edits performed by reviewer. Original RT FND-004 remains open until its fixture is actually removed; no full FR-276 credit is granted.

Current construction carry verdict: **PASS**. Repository aggregate gates remain outstanding.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: 'unit negative_tick : Time = rational(-1, 1) * root_tick;

    unit affine_tick : Time = rational(1, 1) * root_tick + rational(7, 1);'
```

## QSpec 595 amendment carry

QSpec #595 amendment carry inspected RT 8e0ff8fb57d5d24f7fe88d3619149e011de8e95d and integration 7ee2876a342446bf4a9b963c9cfa764a10e6b908 against the prior reviewed construction slice. Canonical source ruling now appends event_position to State; seventeen root bindings and public Layout/API remain unchanged. The outer State validator borrows the shared record, requires exact ordered required fields, Integer now/history_start/event_position, canonical buffer_0..n Sequence fields matching capacity count, and Sequence activations. It returns TypeMismatch at the State key; no compatibility default or host counter mirror appears. Source admission still owns activation/result internals.

Reviewed actual QSL State variants and matching shared declarations, valid zero/two-buffer cohorts, typed missing-position closure, malformed outer fields, State-specific refusal assertions, and retained literal zero. Supplied evidence is 14 passing controls, all-target Clippy pass, and intended red/green for the complete missing-position closure. These establish construction only. FR-276-AC-11 event increments and watermark/poll/gap preservation remain open execution obligations, together with AC-1..10. No genuine temporal producer or execution acceptance is granted. Previous code findings remain fixed; full-delivery gap remains deferred/open. Pending consumer acknowledgement, strict coverage authorization and incomplete heavy IEEE proof remain explicit; no merge-ready or all-gates-green claim is made. Reviewer ran no Cargo/Kani and changed no source.

No new defect in this method. A separate EARS method records the newly compound SHALL sentence; its disposition is still required.

## Integration link fix (round 4)

Inspected efe9b3c6b67f80e25e476de633ded1c19b86a084 against 7ee2876a342446bf4a9b963c9cfa764a10e6b908: the only product/spec delta is IT-018 verifies target changing from interface-002 to the exact declared artifact id interface_002. Criterion IDs and all code/tests are unchanged. Dependency FND-002 is fixed; all earlier integration findings remain fixed. Supplied changed-doc Quire validation passes. Construction amendment review is PASS; final ordinary aggregate remains for the lead to run. RT EARS finding remains pending a separate RT fix. No temporal producer/execution credit, Cargo/Kani invocation, source edit or merge performed by reviewer.

Current method verdict: **PASS** for construction carry.

```yaml
dispositions: []
```
