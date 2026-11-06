---
id: SR-2026110414560102
title: "spec-review/base of P8 model namespaces"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; spec/test-cases/TC-110-tc187-model-namespaces.md; tests/tc187_p8_model_names.rs"
review_set: subset
---

## Summary

Ticket: IR-842. PR: quire-integration#75. Scoped review of the two new files only. No actionable defects in this partial qualification change.

## Verdict

**PASS** for the scoped change. Outstanding normative results remain Planned. No whole-criterion acceptance or whole-repository assurance is granted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Original minimal domain documents pass genuine public model intake, assembly, checking, checked-package emission and strict IR admission with original document evidence. No checked keys or owner receipts are fabricated. U34 asserts distinct native source/model unit keys and unit IDs, scales, model root, both parameter references, ownership/path and tick Rate/scale. U35 asserts three dimension keys and parameter types, tenth scale, complete two-term Speed map, three missing names, and both duplicate source spans with the later diagnostic locus.

U34 remains partial: native UnanchoredBaseDimension preserves original Length identity and m2; assembly and public compiler agree on the original model-selection source span, but do not expose the required /semantic_graph/nodes/0/owner pointer. U35 remains partial: duplicate Dimension.identity characterization checks the full actual malformed-declaration payload, repeated identity, later $.dimensions[1].identity member and original /ir/dimensions/1/identity pointer. It deliberately lacks an AC-20 trace tag and grants no conflicting-binding acceptance credit. The original intended failing receipt and owner-search evidence were inspected; the mismatch is routed under IR-842 rather than locally remapped. No dedicated open owner was verified by that search. No generated/runtime or complete SC-04 qualification is claimed.

Base checklist applied to the full new procedure, expected results and seven qualification rows. TC-110 references public FR authority instead of defining domain semantics. Intended obligations are separated from actual qualification status. No requirement statements, domain objects, relationship-edge policy or architecture boundary changed, so further specification lenses were not selected.

Supplied receipts inspected: four focused tests PASS, scoped Clippy with warnings denied PASS, formatting PASS (stable rustfmt reports unsupported import-group options), TC-110 validation PASS. Reviewer ran git diff --check successfully. No builds, tests, census, Kani or full repository gates were rerun: dispatch reserves ONE final scoped gate to the lead. These are inspected execution receipts, not reviewer execution. Dispatcher, IT-019, shared helpers, manifests and lock are unchanged.

No applicable AssuranceProfile was found under the repository spec tree. No enforced review policy applied.

## Examined units

```yaml
scope:
- id: TC-110:U34 source ms and model ms, both parameters, tick
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U34 source ms and model ms, both parameters, tick | Genuine native intake,
    exact distinct unit keys and unit IDs, scales and model root; parameter references
    and model owner/path survive checked emission and strict IR admission; tick''s
    native dimension is Rate | Passed |'
- id: TC-110:U34 bare Rate
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U34 bare Rate | Typed missing-name refusal at the original Rate span
    | Passed |'
- id: TC-110:U34 unitless Length
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U34 unitless Length | Typed missing-name retaining the original imported
    Length identity and m2; assembly and public compiler agree at the model-selection
    source span | Partial: graph owner pointer Planned |'
- id: TC-110:U35 three Time owners, tenth and Speed
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U35 three Time owners, tenth and Speed | Native intake and checking,
    distinct dimension keys and parameter types; tenth scale and complete Speed dimension
    map; emission and strict IR admission | Passed |'
- id: TC-110:U35 missing local Time, Z::Time and R::Speed
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U35 missing local Time, Z::Time and R::Speed | Each typed missing-name
    refusal at its original name span | Passed |'
- id: TC-110:U35 duplicate source Time
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U35 duplicate source Time | Typed ambiguity retaining both original
    name spans and locating the later one | Passed |'
- id: TC-110:U35 duplicate model Dimension.identity
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: '| U35 duplicate model Dimension.identity | Native intake currently returns
    malformed-declaration, retaining the exact repeated identity, later member path
    and original reader pointer | Characterized; required conflicting-binding Planned
    |'
- id: source_and_model_ms_are_independent_and_tick_uses_model_rate
  path: tests/tc187_p8_model_names.rs
  role: examined
  excerpt: fn source_and_model_ms_are_independent_and_tick_uses_model_rate() {
- id: unitless_imported_length_retains_original_first_refusal
  path: tests/tc187_p8_model_names.rs
  role: examined
  excerpt: fn unitless_imported_length_retains_original_first_refusal() {
- id: three_time_owners_compose_and_missing_names_remain_missing
  path: tests/tc187_p8_model_names.rs
  role: examined
  excerpt: fn three_time_owners_compose_and_missing_names_remain_missing() {
- id: duplicate_model_dimension_identity_characterizes_current_malformed_intake
  path: tests/tc187_p8_model_names.rs
  role: examined
  excerpt: fn duplicate_model_dimension_identity_characterizes_current_malformed_intake()
    {
- id: TC-110:Description
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: Exercise the public U34 and U35 vectors by reference through native model
    intake,
- id: TC-110:Test Procedure
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: 4. Present a duplicate native Dimension.identity and require intake refusal
    at
- id: TC-110:Expected Results
  path: spec/test-cases/TC-110-tc187-model-namespaces.md
  role: examined
  excerpt: 'Status: Partial; the exact qualified and outstanding subcases appear below.'
```
