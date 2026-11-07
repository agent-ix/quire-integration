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

## Dispositions

Round 1, verified at be2d39d against IT-022 and TC-115.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Objective now qualifies the complete bridge and every scenario is required; Expected Results matches |
| FND-002 | fixed | Single Test Procedure and Expected Results; appended sections, stray blank lines and long lines removed |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | TC-115 has no blank line between the trace table and the next paragraph, so GFM renders "Assert complete typed owner values..." as a table row; IT-022 SC-02 and SC-13 gained unwrapped long lines | spec/test-cases/TC-115-native-temporal-projection.md:35-36; spec/integration/IT-022-native-temporal-projection.md:104,177 |

Round 2, verified at 279faba.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | still-open | new this round; insert a blank line after the TC-115 table and rewrap the two IT-022 lines |

Round 3, verified at d5168a2.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | Blank line added after the TC-115 table; IT-022 SC-02 and SC-13 rewrapped; the two remaining long lines are TC-115 table rows, which cannot wrap |
