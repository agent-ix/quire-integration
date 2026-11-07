---
id: SR-2026110705000804
title: "IR-1005 PR 88 IT-022 spec-review-integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-integration; IR-1005; PR 88 head f07cb34; spec/integration/IT-022-native-temporal-projection.md"
review_set: subset
---

## Summary

Ticket: IR-1005. Review of agent-ix/quire-integration PR 88 (one added file, IT-022). No applicable AssuranceProfile exists under spec/. Owner APIs read at origin/main of quire-spec-language, quire-mltl, quire-observation and quire-contract-ir. quire validate over spec passes. No build, test or gate was run.

## Verdict

FAIL. The document contradicts itself about what it covers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Objective and Expected Results say correspondence reading, joining, supersession and seven-counter qualification remain open, while later sections specify them as required behaviour; a reader cannot tell which obligations this IT qualifies | spec/integration/IT-022-native-temporal-projection.md:21-24,107-114,117-278 |
| FND-002 | low | Sections appended after Expected Results with stray blank lines and unwrapped long lines; status and contract prose are mixed in Remaining owner prerequisites | spec/integration/IT-022-native-temporal-projection.md:115-116,140,144,215-216 |

## Coverage

Checked internal consistency, section structure and terminology across all eleven sections.

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
