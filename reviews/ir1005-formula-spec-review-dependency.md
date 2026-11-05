---
id: SR-2026110414560047
title: "IR-1005 formula component spec-review-dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-integration; IR-1005; formula construction slice; IT-020; TC-101"
review_set: subset
---

## Summary

Independent review of the eight changed files for IR-1005 formula construction. No applicable AssuranceProfile is declared. Review artifacts contain original analysis and repository-relative references only.

## Verdict

PASS within this component scope. Full FR-350 and IR-1005 remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Cross-repository verifies linkage is qualification context, not a prerequisite completion claim. The component consumes available owner APIs and direct branch-main tl-syntax. Full projection, native binding/valuation authority, report predecessor authority, request/result readers and remaining counters remain subsequent work. No requirement dependency cycle or mixed request-stack adoption is introduced.

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

## Latest verdict

PASS within the bounded formula component. Round 1 source changes were inspected directly: both EVENT and SAMPLE native checks must return exactly one typed MissingInterval refusal before emission; the existing infinite-trace test still asserts complete owner Unsupported retention. Defensive invalid interval representations return the existing typed Representation stop. No new findings. Original findings remain unchanged above. Existing focused receipts report nine formula tests and one binding test passing, Clippy, formatting and spec validation passing. No reviewer build, test or gate was run. Full projection, valuation/observation/history/correspondence/join/request readers and whole FR-350 limits remain Planned.
