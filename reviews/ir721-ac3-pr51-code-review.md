---
id: SR-2026110414560036
title: IR-721 PR51 source code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-integration; PR51; TC-100; tests/monitor_program_adverse.rs;
  tests/support/monitor_program_adverse.rs; Cargo.lock
review_set: subset
---

## Summary

IR-721 integration draft PR51; frozen 666680d63c18b6f30cdbf0cfff582e4cf6f9b951, base integration main 851237f. Source/evidence review only, not runtime acceptance or merge readiness. No applicable AssuranceProfile found. No driver preparer, IR implementation, production support policy, target comparator or RT construction change is included.

## Verdict

PASS for source review only: no source findings. Runtime acceptance is BLOCKED, not passed; this draft is not merge-ready.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Rust source review: public emit_plan_root is used; generated entry source is obtained by public render(spec_from_plan), rerendered unmodified first and required to reproduce exact final bytes. The adverse edit replaces one checked initializer expression in source, not identities or emitted term payloads; rechecking/emission assign genuine new keys and retain clause/witness inputs. Reachability walks original term references with a visited set; modulo must be absent from healthy and uniquely present under changed init. Driver diagnostics retain original selected keys, Generated0 or caller None, exact uncertainty value path, operation/kind/occurrence and typed invalid-path/occurrence failures. These payload constructors are not presented as target mismatch policy or Unsupported preparation outcomes. Missing-body mutation removes exactly the actual selected node body and re-reads unchanged bytes afterward. No copied evaluator/temporal algorithm, fake preparer, hand-built ProgramParts, semantic identity minting, unsafe or new production panic path exists. Expected terminal decode oracle is source-grounded in locked IR d81d81d: body is a required CheckedSemanticNodeV2 field; decode_closed_bytes/classify_closed_error yields MalformedWire for a missing known field, decoder_pointer preserves semantic_graph/nodes/index; no admitted identity/cause is created at that stage. Its exact runtime result remains unobserved.

## Verification limits

ACCEPTANCE BLOCKED on QSL-1722. Existing driver-diagnostic-test.log compiled successfully then exited 101: zero tests passed and two failed at public emit_plan_root Write(MissingEqualityIdentityContext) for generated quire.op.boolean.ne. This occurs before final emission and every new diagnostic/reader assertion. All new downstream assertions are unexecuted. Existing focused Clippy, fmt and changed-TC validation receipts pass; the earlier-cohort producer checkpoint grants no new credit. Reviewer ran no Cargo/tests/gates/census/advisor/Kani and made no branch edits. No retry, pin or owner patch was attempted. Resume only after the relayed merged QSL-1722 notice; lead owns one final scoped gate at the eventual final head.

## Examined units

```yaml
scope:
- id: TC-100-Description
  path: spec/test-cases/TC-100-monitor-producer-adverse-construction.md
  role: examined
  excerpt: 'Qualify the IR-721 AC3 construction handoff from genuine QSL production
    through

    strict IR admission and the driver to RT. The first executable contribution prepares
    the changed-target and

    unsupported-instruction controls. Full IR-721 AC3 still requires the healthy

    FR-276-AC-6 construction and all FR-276-AC-7 adverse cases listed below. A

    producer/diagnostic preflight alone supplies no construction or target-policy

    refusal credit.

    Event/watermark execution is a separate qualification.'
- id: TC-100-Test Procedure
  path: spec/test-cases/TC-100-monitor-producer-adverse-construction.md
  role: examined
  excerpt: "1. Read the QSL-owned ConfigVersion model unchanged by reference. Author\
    \ a\n   bounded timed clause and explicit target, and invoke public\n   `qsl_analyze::monitor_root::emit_plan_root`.\
    \ Strictly admit the exact final\n   emission with its original evidence and model\
    \ document. Preserve the plan,\n   root and four entry content keys, authentic\
    \ occurrence and producer\n   witnesses. No synthetic plan or executable arena\
    \ supplies the positive case.\n2. Generate the same entry source with the public\
    \ `monitor_entries` generator,\n   check it through QSL and emit with `emit_monitor_plan_over`.\
    \ Require tha"
- id: TC-100-Outstanding Construction Controls
  path: spec/test-cases/TC-100-monitor-producer-adverse-construction.md
  role: examined
  excerpt: 'All rows below remain Planned under IR-721. The owner column identifies
    the

    boundary that must supply the observed decision, not a newly asserted blocker.

    The driver signature and error variants must come from its production owner;

    this test invents neither an API nor a diagnostic type.


    | Required control | Observed boundary and ownership | Status |

    | --- | --- | --- |

    | Healthy final plan after preparation objects are dropped; two independent builds
    | Driver FR-015 returns the complete owned program; RT FR-276 runs its real initializer
    | Planned |

    | Foreign atom or operand and equal numeric'
- id: TC-100-Expected Results
  path: spec/test-cases/TC-100-monitor-producer-adverse-construction.md
  role: examined
  excerpt: 'The positive and both adverse boundaries execute through the real public

    producer, reader, driver and RT construction APIs and retain their exact typed

    results and source loci. No callback, reimplemented temporal transition,

    hand-built `ProgramParts`, unavailable-seam skip or test-only evaluator supplies

    missing meaning.


    Status: Planned for the complete construction and adverse intake handoff under

    IR-721. The two `monitor_program_adverse` tests contain the genuine producer

    rekeying, merged-driver diagnostic and missing-entry IR controls described

    above. The refreshed focused target compiles,'
- id: tests/monitor_program_adverse.rs
  path: tests/monitor_program_adverse.rs
  role: examined
  excerpt: 'Complete 241-line test source: actual producer rekey/driver diagnostic
    assertions and missing initializer body ingress refusal.'
- id: tests/support/monitor_program_adverse.rs
  path: tests/support/monitor_program_adverse.rs
  role: examined
  excerpt: 'Complete 222-line source: public emit_plan_root; original evidence; public
    render/recheck/emission route; strict IR read.'
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: Ordinary coherent first-party main cohort refresh including merged driver
    monitor_refusal API; no Cargo.toml edits.
- id: tests/support/mod.rs
  path: tests/support/mod.rs
  role: context_only
  excerpt: Existing unchanged Cargo metadata helper locates QSL model document by
    locked dependency context.
```

## Resumed evidence review

Resumed evidence review at frozen 83660638e33050e74a6069ceebf5729020f1c7a4: PASS for the qualified producer/diagnostic/malformed-ingress slice; no open source findings and no new findings. User-relayed QSL-1722/QSL-2367 merged notice authorized the ordinary main refresh. Cargo.lock now resolves the QSL cohort coherently at 14427bd3; both test source files are unchanged from the source-reviewed tree. The actual qsl1722-resumed-test.log records 2 passed, 0 failed/ignored/filtered in 11.59 seconds after compilation. No unavailable-boundary skip or replacement producer occurred.

Executed assertions cover genuine final PlanRoot and exact unmodified entry-source reproduction; reachable changed initializer modulo with fresh original root/init keys and unchanged clause; exact original uncertainty [14,0], structural key, Generated0/None/nonmember occurrence behavior; complete original modulo UnsupportedOperation diagnostic at root path [], source kind/key/occurrence and invalid path; exact missing-body MalformedWire/no-cause/no-admitted-locus/actual-node-pointer refusal followed by healthy original readmission. The previously source-grounded decode oracle is now observed through actual strict IR admission. The earlier blocked history remains unchanged above and supplies no extra credit.

## Latest verdict

PASS for the qualified slice; no open or new review findings.

TC100 now reports precisely those observations and explicitly preserves the complete missing-entry preparation control and every outstanding construction row as Planned. No real prepare_monitor, independent target comparison, support-policy decision, runtime Program/build or full FR276 criterion has executed. This is qualified-slice review PASS, not full construction acceptance or permission to merge before the lead completes the final scoped gate. Reviewer performed no Cargo/tests/gates/global census rerun or branch edit.
