---
id: SR-2026110705000802
title: "IR-1005 PR 88 IT-022 spec-review-failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-integration; IR-1005; PR 88 head f07cb34; spec/integration/IT-022-native-temporal-projection.md"
review_set: subset
---

## Summary

Ticket: IR-1005. Review of agent-ix/quire-integration PR 88 (one added file, IT-022). No applicable AssuranceProfile exists under spec/. Owner APIs read at origin/main of quire-spec-language, quire-mltl, quire-observation and quire-contract-ir. quire validate over spec passes. No build, test or gate was run.

## Verdict

FAIL. The valuation source contradicts the native owner, and several input states have no stated disposition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | SC-02 builds TL propositions from EvidenceRequest shared and instance-bound cells, but native evaluate_v2 takes valuations only from the admitted six-bound ValuationEvidence relation and replaces request maps; ValuationEvidence is absent from project_tl Inputs | spec/integration/IT-022-native-temporal-projection.md:54-62,78-81,126 |
| FND-002 | medium | Disposition of Valuation::Missing, Valuation::Undefined(cause), aggregate samples and non-Boolean leaves is unstated; FR-350-AC-3 non-Boolean-leaf case has no scenario | spec/integration/IT-022-native-temporal-projection.md:78-81 |
| FND-003 | medium | Duplicate cell is undefined: BTreeMap keys cannot repeat, so the reachable duplicates (shared plus instance-bound for one atom and position, or repeated ValuationRow) must be named | spec/integration/IT-022-native-temporal-projection.md:80-81 |
| FND-004 | medium | SC-04 and the predecessor sections accept a preserved owner refusal as the scenario outcome for authored supported cases (non-nanosecond fixed-sample past, corrected agreement) without naming an owning ticket or stating the scenario stays non-qualifying | spec/integration/IT-022-native-temporal-projection.md:91-92,143-146,163-165 |
| FND-005 | low | Clock family is an undefined term; QObs exposes clock identity and revision | spec/integration/IT-022-native-temporal-projection.md:77 |

## Coverage

Traced each SC failure mode to the owner value types at origin/main: qsl-eval EvidencePosition and Valuation, root-crate CheckedSubject, ValuationRow and evaluate_v2, quire-mltl RequestInput and nanosecond past check.

## Examined units

```yaml
scope:
- {id: IT-022-SC-01, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Emit original source once and admit the bytes twice. Require a local subject and local classification to pass; substitute only the foreign classification ... require a named admission refusal before formula construction."}
- {id: IT-022-SC-02, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Construct every required position/proposition value from the request's shared or selected instance-bound valuation cells. ... Missing cells are incomplete; duplicate, swapped, foreign or conflicting bindings refuse before a TL request exists."}
- {id: IT-022-SC-03, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Construct closed and prefix future requests, and individually supported bounded-past requests with complete origin history. ... A history cutoff is incomplete, not a new origin."}
- {id: IT-022-SC-04, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Retain fixed-sample epoch, period and unit ... If an owner cannot represent an otherwise authored supported selection, preserve its typed refusal and record the exact upstream limitation"}
- {id: IT-022-SC-05, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Supply independent admitted clock, decision and surrounding progress/closure, completeness and availability views. ... No evaluator or result reader is invoked."}
- {id: IT-022-SC-06, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Precharge bridge node, position and cell visits ... Support and cause counters belong to the later join and are not qualified by projection tests."}
- {id: IT-022, path: spec/integration/IT-022-native-temporal-projection.md, role: examined, excerpt: "Sections: Objective, Target Integration, Preconditions, Inputs, Expected Results, Correspondence and strict reading, Remaining owner prerequisites, Result joining and strict join reading, Original bridge definition admission, Retained activation and exact source occurrence, Native full-predecessor admission"}
- {id: FR-350, path: "quire-contract-ir spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md", role: context_only, excerpt: "project_tl/read_tl_projection/join_tl/read_tl_join signatures, seven TlLimits, AC-1..AC-9"}
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | SC-02 requires incomplete for an instance anchor unequal to request.anchor, but FR-350-AC-9 in quire-contract-ir#944 says a mismatched instance refuses with its typed error; a coder following one fails the other | spec/integration/IT-022-native-temporal-projection.md:92-94 |

## Dispositions

Round 1, verified at be2d39d against IT-022, TC-115 and FR-350 in quire-contract-ir#944 (40ac6f9).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Inputs take valuations only from ValuationEvidence; SC-03 proves contradicting request maps leave the projection unchanged |
| FND-002 | fixed | SC-03 covers Missing, Undefined and aggregate request maps as ignored; SC-05 refuses aggregate and non-holds leaves with tl-projection-formula |
| FND-003 | fixed | SC-03 defines a duplicate as two rows with equal (position, operand), with equal and opposite values |
| FND-004 | fixed | SC-06 limits the owner refusal to the refusal clause of FR-350-AC-4 and requires passing positive cases; SC-13 requires a correction to reach R7 |
| FND-005 | fixed | Clock identified by QObs clock identity and clock revision (Inputs, SC-02) |
| FND-006 | still-open | new this round; not yet addressed |
