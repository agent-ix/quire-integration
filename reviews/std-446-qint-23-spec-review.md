---
id: SR-16353
title: "spec-review of IT-015 and TC-095 negotiate FR-345 layer witnesses over the real backends (quire-integration PR 23)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration@c6844d6f; spec/integration/IT-015-negotiate-layer-witnesses-over-real-backends.md; spec/test-cases/TC-095-negotiate-layer-witnesses-over-real-backends.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-integration/IT-015
    type: references
  - target: ix://agent-ix/quire-integration/TC-095
    type: references
---

# Spec review of IT-015 and TC-095 negotiate FR-345 layer witnesses over the real backends (quire-integration PR 23)

## Summary

Ticket: STD-446. One spec-review pass over the PR's own delta, checked for the no-drop/no-narrowing/no-deferral rule, hand-derived vectors, id uniqueness and mergeability.

Examined, clean:

- Ownership: FR-280-AC-3 requires the `cargo tree` of every QSL crate to list no CG, RT or driver crate, and `xtask` is a QSL crate, so QSL cannot run `negotiate_*` itself. quire-integration is the repo that already depends on both QSL and CG, so IT-015 there is the only place the real-backend half can live. QSL FR-345-AC-12 links to it, and the AC-9 witness half and AC-6 real-backend half have an owner and a TC.
- Steps hand-checked against QSL FR-345: SC-01 matches AC-6 (once per candidate by backend name, `supported` under one), SC-02 holds because cases and edges are independent of negotiation, SC-03 matches AC-6 regression text and AC-8 (removed `witness` entry gives a tool failure naming the layer and `witness`). IT-015 and TC-095 are unused on origin/main and on the other open quire-integration heads (#22 uses IT-014).
- No deferral or narrowing wording. Both files validate (`quire validate --scope .`, exit 0).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The `references` edge targets `ix://agent-ix/quire-contract-codegen/FR-331`, but FR-331 (the backend-provider envelope whose `negotiate_*` arm the test uses) is a quire-specification artifact; the quire-contract-codegen clone holds no FR-331 (searched artifact ids on origin/main). `quire validate` does not resolve cross-repo targets, so it passes, but the edge dangles. Fix: target `ix://agent-ix/quire-specification/FR-331`. | spec/integration/IT-015-negotiate-layer-witnesses-over-real-backends.md | wrong-requirement |

## Verdict

Mergeable after FND-001 (low) is fixed or dispositioned by the lead.

Merge-tree: new files only, no conflict with main or other open heads. Gate: spec-only; `quire validate --scope .` on both files exit 0 (quire-integration `make ci` is a Rust gate and was not run for a spec-only PR).
