---
id: SR-2026110414560043
title: "IR-1005 formula component code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; IR-1005; formula construction slice; IT-020; TC-101"
review_set: subset
---

## Summary

Independent review of the eight changed files for IR-1005 formula construction. No applicable AssuranceProfile is declared. Review artifacts contain original analysis and repository-relative references only.

## Verdict

CONDITIONAL within this component scope. Full FR-350 and IR-1005 remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The only unbounded-source control selects infinite-trace, so it exits through owner Unsupported before node_kind can produce UnboundedInterval. IT-020 requires an explicit typed premise stop for otherwise-supported unbounded intervals. Add a real-producer event-position (and preferably fixed-sample) unbounded formula control that asserts UnboundedInterval, its exact original node and interval, its stable code, and the unchanged source classification/premises. Returning Representation or constructing a formula for this input would currently leave the suite green. | tests/temporal_formula.rs:527; spec/integration/IT-020-tl-formula-construction.md:32 |

## Coverage

Rust code and all eight integration tests inspected. Owner constructors, strict reader, complete input reclassification, left-to-right occurrence traversal, original predicate handles and capped owner serialization are used directly. No evaluator or schema mirror is introduced. One required adverse branch lacks a discriminating control.

Reviewed the complete changed Cargo manifest/lock, README, IT-020, TC-101, temporal re-export, formula implementation and test target. Existing receipts report eight formula tests and one binding test passing, plus focused Clippy, formatting and specification validation. Reviewer ran no build, test, Kani or gate. Final scoped gate belongs to the lead. No global coverage or completion verdict is claimed.

## Examined units

```yaml
scope:
- id: IT-020-SC-01
  path: spec/integration/IT-020-tl-formula-construction.md
  role: examined
  excerpt: "IT-020-SC-01: Construct all available bounded future and past operators,\
    \ constants\n   and Boolean connectives. Visit occurrence edges in left-to-right\
    \ postorder,\n   preserving repeated uses without node deduplication. Future targets\
    \ select V1 and\n   their closed/prefix profile; past targets select V2 origin-complete-history\
    \ under\n   both source false-extension profiles, including previous as once[1,1].\
    \ Compare\n   exact operator/operand/interval rows and strict owner-read equality."
- id: IT-020-SC-02
  path: spec/integration/IT-020-tl-formula-construction.md
  role: examined
  excerpt: "IT-020-SC-02: Reject a foreign instance or altered classification before\
    \ formula\n   construction. Retain Unsupported dimensions unchanged and original\
    \ outstanding\n   premises on successful formula components; return no successful\
    \ full projection."
- id: IT-020-SC-03
  path: spec/integration/IT-020-tl-formula-construction.md
  role: examined
  excerpt: "IT-020-SC-03: Charge each component graph-entry scheduling visit and each\
    \ emission\n   visit before its action, and charge each occurrence node before\
    \ scheduling. Defaults\n   are max_nodes 65,536, max_work 4,294,967,296, max_bytes\
    \ 16,777,216. Node limits count\n   occurrences. Check nodes before work when\
    \ both would fail at scheduling. Encode the\n   actual owner Serialize implementation\
    \ through a capped writer before invoking its\n   canonical writer; reject before\
    \ accepting the first excess byte. Exact limits admit,\n   one less stops at setting+1,\
    \ counters reset per call and errors expose no artifact.\n   Serialization, owner\
    \ constructor and strict-reader internal work remain owner work,\n   not component\
    \ visit work. No unbounded intermediate JSON encoding precedes the cap."
- id: IT-020-SC-04
  path: spec/integration/IT-020-tl-formula-construction.md
  role: examined
  excerpt: "IT-020-SC-04: Preserve typed owner constructor/strict-reader/encoding\
    \ refusals.\n   Error codes are stable Copy catalog entries with a public code\
    \ accessor."
- id: TC-101
  path: spec/test-cases/TC-101-tl-formula-construction.md
  role: examined
  excerpt: Every IT-020 criterion passes or retains its exact typed owner refusal.
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The initial proposed Supported-unbounded control was based on an incorrect reachability assumption. Actual QSL bounded profiles reject missing intervals before checked-package emission. The fix removes the unreachable public UnboundedInterval contract/code and replaces it with real-producer MissingInterval controls under both false-extension profiles, while retaining infinite-trace Unsupported. |

After excerpt from spec/integration/IT-020-tl-formula-construction.md:

> For event-position and fixed-sample false-extension source, QSL requires a closed
> interval: `eventually holds(true)` returns native `TemporalCause::MissingInterval`
> before any checked package, subject, classification or component exists.

## Latest verdict

PASS within the bounded formula component. Round 1 source changes were inspected directly: both EVENT and SAMPLE native checks must return exactly one typed MissingInterval refusal before emission; the existing infinite-trace test still asserts complete owner Unsupported retention. Defensive invalid interval representations return the existing typed Representation stop. No new findings. Original findings remain unchanged above. Existing focused receipts report nine formula tests and one binding test passing, Clippy, formatting and spec validation passing. No reviewer build, test or gate was run. Full projection, valuation/observation/history/correspondence/join/request readers and whole FR-350 limits remain Planned.
