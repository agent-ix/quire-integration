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
