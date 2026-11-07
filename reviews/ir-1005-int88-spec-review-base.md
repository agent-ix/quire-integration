---
id: SR-2026110705000801
title: "IR-1005 PR 88 IT-022 spec-review-base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-1005; PR 88 head f07cb34; spec/integration/IT-022-native-temporal-projection.md"
review_set: subset
---

## Summary

Ticket: IR-1005. Review of agent-ix/quire-integration PR 88 (one added file, IT-022). No applicable AssuranceProfile exists under spec/. Owner APIs read at origin/main of quire-spec-language, quire-mltl, quire-observation and quire-contract-ir. quire validate over spec passes. No build, test or gate was run.

## Verdict

FAIL. IT-022 is meant to be the single canonical full-chain IT, but only the projection half carries scenario ids, and the projection half narrows FR-350-AC-5 and FR-350-AC-8. No hash, digest, pin or vendored owner schema is introduced; content comparison of complete canonical bytes is used throughout, which is correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Correspondence reading, join, join reading, bridge definition admission, retained activation and native full-predecessor sections carry obligations but no IT-022-SC id, so no test can trace them and none is required | spec/integration/IT-022-native-temporal-projection.md:117-278 |
| FND-002 | high | Test procedure narrows FR-350: SC-06 excludes support and cause counters and Expected Results leaves join, supersession and seven-counter qualification open, so AC-5 and AC-8 have no scenario in the canonical IT | spec/integration/IT-022-native-temporal-projection.md:104-114 |
| FND-003 | medium | No TC artifact verifies IT-022 (IT-020 has TC-101, IT-021 has TC-114); no test target or procedure is named | spec/test-cases/ |
| FND-004 | medium | SC-06 names no limit, setting, default, cause shape or multi-limit tie-break; FR-350 defines seven TlLimits with defaults and limit-reached{limit,value,setting} | spec/integration/IT-022-native-temporal-projection.md:98-105 |

## Coverage

Checked ID format, relationships, scenario coverage of FR-350-AC-1..AC-9, error paths, limit boundaries and the hash/pin antipattern.

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
| FND-005 | low | TC-115 says tests carry FR-350 criterion trace tags, departing from this repo's IT-0xx-SC-NN tag convention (tests/temporal_formula.rs); the SC-to-test link then rests on TC prose only | spec/test-cases/TC-115-native-temporal-projection.md:28-29 |

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | FR-350-AC-6 (e0477e6) requires every binding-step failure to return its assigned code and premise, but SC-02, SC-07 and SC-08 enumerate only some; unlisted are subject-name, rational epoch/period and non-u64 range bound (clock-representation), out-of-range, repeated and above-completeness stamps (position), request.anchor matching no or several positions (anchor), owner anchor of the wrong form (clock), valuation eviction (complete-input), capture-count and delivery-anchor mismatches, surrounding closure unequal to the supplied closure, absent progress, unparsable watermark and timestamped boundary (progress), timed interval and several source entries | spec/integration/IT-022-native-temporal-projection.md:94-158 |

## Dispositions

Round 1, verified at be2d39d against IT-022, TC-115 and FR-350 in quire-contract-ir#944 (40ac6f9).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | SC-08 to SC-15 now carry activation, bridge definition, correspondence, malformed bytes, join rows, corrections, limits and no-evaluator obligations |
| FND-002 | fixed | SC-12 reaches R1 to R8 and row pairs, SC-13 reaches R7 for a correction, SC-14 covers max_support and max_causes; open wording removed |
| FND-003 | fixed | TC-115 verifies IT-022 and names its test targets |
| FND-004 | fixed | SC-14 names all seven settings, defaults, limit-reached{limit, value, setting} and table-order tie-break |
| FND-005 | still-open | new this round; not yet addressed |

Round 2, verified at 279faba.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | fixed | TC-115 table maps each test target to its IT-022-SC and FR-350 trace tags in #[trace(...)] |

Round 4, verified at 880a385 against FR-350 in quire-contract-ir#944 (e0477e6).

| FND | outcome | reason |
| --- | --- | --- |
| FND-006 | still-open | new this round; list every FR-350 binding-step failure in SC-02, SC-07 and SC-08, or state that each step failure is tested and name the omitted ones |
