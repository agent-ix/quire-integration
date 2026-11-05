---
id: SR-2026110414560042
title: "IR-1344 spec-review-base of Decimal D29 qualification"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; tests/scalar_agreement_decimal.rs; IR-1344"
review_set: subset
---

## Summary

Ticket: IR-1344. The frozen PR changes only tests/scalar_agreement_decimal.rs: the D29 dispatch arm, its real native-source controls, and the admission trace tag. The local candidate and GitHub PR head were independently confirmed to match the frozen candidate. No repository edits were made by this reviewer.

All three omitted-mode forms call qsl_cst::parse, require inadmissibility and nonempty typed Parse/InvalidSyntax diagnostics, and never enter forms/check/evaluation. Replacing the one omitted-mode occurrence yields a source that must parse and pass real S2/S3/S4. The Quantity source declares dimension Length and unit m. Each corresponding exact Decimal carrier executes the public native round operation and checks representation and no loss. The conversion counterpart additionally links the checked graph and calls f with Rational 1/2; native division must complete with (50,2) and no loss before QSL/native agreement compares value, loss, ordered charges and all counters, separately accounting for function.call. The guarded conversion check executes for exactly the table's second entry.

No RT quantity identity is constructed and no quantity evaluation agreement is claimed. The public QSpec row API remains the dispatch source; no Markdown parser, copied corpus, new pin, dependency change, oracle change, new ignore, or catch-all success was introduced. Existing qsl_convert and carrier/meter helpers were inspected; qsl_convert cannot serve the malformed-source parser control because it generates an always-explicit target in both source positions. Existing helpers are reused for native operation and agreement.

## Verdict

PASS for scoped contract fit. Existing IT-017 admission/evaluation separation and IT-016 producer-origin constraints already cover this case; the implementation requires no specification changes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Scope is the changed D29 behavior only, not closure of IT-017 or all TC-185 pending cases. Author logs were inspected: untouched-main baseline panics on unimplemented D29; focused target reports 1 passed and 2 preexisting QSL-1393 ignored probes; scoped Clippy exits successfully; fmt log contains only existing stable-toolchain configuration warnings. These are supplied runtime results, not reruns by the reviewer. The lead owns the final scoped gate. No Cargo/build/test/Clippy, global matrix/trace/coverage/properties/advisor, Kani, or additional agents were run, as instructed.

The read-only fresh-main vector source and inventory identify D01..D29 versus baseline D01..D28, only D29 missing. The diff retains fail-closed unknown-ID panic and duplicate-ID check. No applicable AssuranceProfile was found under spec. AGENTS.md, CLAUDE.md, CONTENT_RIGHTS.md and README.md were read; LICENSE-DECISION.md is absent from this checkout. All new test text is authored integration source rather than imported upstream source material.

Plan completion: not assessed

Examined units and context (role and exact excerpt):

- IT-017-SC-04 (examined, spec/integration/IT-017-runtime-scalar-agreement.md): every runnable admission case has its own observed decision and is never counted as two-sided scalar evaluation.
- IT-017-SC-02 (examined, spec/integration/IT-017-runtime-scalar-agreement.md): every runnable evaluation case has a real two-sided observation and the same specified value or noncompleted result. A case that cannot execute remains PENDING under SC-01 and is not agreement.
- IT-017-SC-03 (examined, spec/integration/IT-017-runtime-scalar-agreement.md): every runnable authority-metered evaluation and required sweep agrees in charge order, amounts, counters, denial fields, and stop prefix.
- IT-017-SC-01 (context_only, spec/integration/IT-017-runtime-scalar-agreement.md): every row offered by QSpec's scalar-vector API reaches its specified evaluation or admission path, or is named PENDING with an open upstream owner ticket and the exact missing path.
- IT-016 (context_only, spec/integration/IT-016-producer-origin-runtime-identity.md): The source of each semantic key is the admitted package, including for negative tests whose *operation* is malformed after admission.

Selected lenses: code-review with Rust lane, scoped manual gap-analysis, base contract-fit review. Further spec analyses are inapplicable to the unchanged requirements. Evidence advisor was prohibited by the dispatch and was not run; no catalog recommendation is claimed. No prior review findings exist for this PR, so no dispositions are needed.
