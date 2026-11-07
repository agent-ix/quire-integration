---
id: SR-2026110705000803
title: "IR-1005 PR 88 IT-022 spec-review-dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-integration; IR-1005; PR 88 head f07cb34; spec/integration/IT-022-native-temporal-projection.md"
review_set: subset
---

## Summary

Ticket: IR-1005. Review of agent-ix/quire-integration PR 88 (one added file, IT-022). No applicable AssuranceProfile exists under spec/. Owner APIs read at origin/main of quire-spec-language, quire-mltl, quire-observation and quire-contract-ir. quire validate over spec passes. No build, test or gate was run.

## Verdict

FAIL on the input contract; edges otherwise acceptable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | project_tl inputs diverge from the FR-350 signature (separate occurrence, valuation_instance and selected_observation versus request-supplied occurrence and trigger plus a revision); FR-350-AC-9 supplied-occurrence and trigger-instance mismatch refusals have no scenario, and no operation signatures or output types are stated | spec/integration/IT-022-native-temporal-projection.md:54-62,73-81 |
| FND-002 | low | Prerequisites IR-1377, IR-1378, QSL-1795 and TL-301 and the owner contracts (quire-mltl, QObs, native report) appear only in prose; no relationship edge records them | spec/integration/IT-022-native-temporal-projection.md:5-11,62,136-146,265 |
| FND-003 | low | Point-in-time status prose (the selected IR owner now contains the IR-1378 correction) will go stale inside a contract | spec/integration/IT-022-native-temporal-projection.md:139-140 |

## Coverage

Compared IT-022 against FR-350 at quire-contract-ir origin/main and the owner APIs it names; all named owner functions exist at origin/main (validate_mapping_binding, selected_model_packages, TraceBinding::authenticate, ActivationInput::contradiction, ActivationFault, CheckedSubject::with_predecessor, Report::to_owner_bytes_bounded, validate_predecessor, ReportPredecessorLimits, NativeJoinBudget, RequestInput).

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
| FND-004 | medium | IT-022 states its inputs are exactly FR-350's NativeProjectionInput and TlProjectionSelection and SC-13 calls validate_supplied_predecessor; these exist only in unmerged quire-contract-ir#944 and quire-mltl#18, so merging #88 first leaves IT-022 contradicting the FR-350 it verifies and naming an absent owner operation | spec/integration/IT-022-native-temporal-projection.md:65-75,165-178 |

## Dispositions

Round 1, verified at be2d39d against IT-022, TC-115 and FR-350 in quire-contract-ir#944 (40ac6f9).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Inputs name the FR-350 #944 signature; SC-02 covers wrong node/role/ordinal, request key, foreign control and instance refusals |
| FND-002 | fixed | depends_on edges added for QSL FR-043/052/516/527, QObs FR-004 and quire-mltl FR-001/003 |
| FND-003 | fixed | Stale owner status prose removed |
| FND-004 | still-open | new this round; not yet addressed |

Round 2, verified at 279faba.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | Inputs cite FR-350 Boundary and interface; SC-13 names validate_supplied_predecessor and direct_predecessor_bytes as an owner prerequisite; criterion-level depends_on edges are an accepted form (quire-contract-ir FR-065 uses one) and quire validate passes. FR-350-AC-11 exists only in quire-contract-ir#944 and quire-mltl FR-001-AC-11/AC-12 only in quire-mltl#18, so #88 merges after both |
