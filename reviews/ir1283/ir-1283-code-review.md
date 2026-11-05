---
id: SR-2026110414560040
title: "IR-1283 IEEE source agreement code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; PR #52 8d9c4e76f9a57de8f1867574eaf3da9d91a34ef0; Cargo.lock and tests/ieee_source_agreement.rs; IT-017-SC-01/02/03"
review_set: subset
---

# IR-1283 IEEE source agreement code and Rust review

## Summary

Ticket: IR-1283. The review covers the frozen PR #52 diff against integration main. The Rust lane applied repository guidance and the Rust style checklist. `Cargo.lock` changes only the resolved QSL main cohort and its required IR/Exact transitive entries. The tests use the admitted checked QSL source and emitted operation carrier, call QSL evaluation and the owned Exact operation, and compare values, flags, losses, typed outcomes, and charge sequences. No copied upstream fixture file, compatibility path, unsafe code, new production API, or CI gate change was found. The QSpec row API supplies the ID dispatch; its prose is not a typed executable request, as IT-017 states. No applicable AssuranceProfile was found. Focused test, fmt, and Clippy results in the PR body were read as coder evidence; the code lead owns the single final gate.

## Verdict

**CONDITIONAL.** One assertion gap leaves QSL's denied-charge amount and some Incomplete limits unchecked. Fix it in the same PR before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new incomplete branches assert native `next_charge` but omit QSL `next_charge` in `assert_float_conversion` and F33; rational conversion omits the Incomplete `limit`. A QSL regression that reports the right point and consumed count with a wrong next amount or limit can pass despite IT-017-SC-03 requiring every denial field. Assert both sides' limit and next charge against the authored expectation, with the function-call envelope applied only to WorkUnits. | tests/ieee_source_agreement.rs:432-452,517-558,1902-1918; spec/integration/IT-017-runtime-scalar-agreement.md:111-120 |

## Examined scope

```yaml
scope:
  - id: IT-017-SC-01
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every row offered by QSpec's scalar-vector API reaches its specified evaluation or admission path, or is named PENDING with an open upstream owner ticket and the exact missing path."
  - id: IT-017-SC-02
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every runnable evaluation case has a real two-sided observation and the same specified value or noncompleted result."
  - id: IT-017-SC-03
    path: spec/integration/IT-017-runtime-scalar-agreement.md
    role: examined
    excerpt: "every runnable authority-metered evaluation and required sweep agrees in charge order, amounts, counters, denial fields, and stop prefix."
  - id: tests/ieee_source_agreement.rs
    path: tests/ieee_source_agreement.rs
    role: examined
    excerpt: "fn tc193_f24_f25_f26_f31_width_conversions_agree() {"
  - id: Cargo.lock
    path: Cargo.lock
    role: examined
    excerpt: "source = \"git+https://github.com/agent-ix/quire-spec-language?branch=main#3e8193af4968b68835d138458fb302d0535b71d7\""
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: code-review
    check: test-intent
    artifact_id: IT-017-SC-03
    related_ids: [IT-017-SC-02]
    path: tests/ieee_source_agreement.rs
    lines: "432-452,517-558,1902-1918"
    excerpt: "every runnable authority-metered evaluation and required sweep agrees in charge order, amounts, counters, denial fields, and stop prefix."
    finding: "The new incomplete branches omit QSL next_charge or Incomplete limit assertions, so wrong denied-charge fields can pass."
bindings:
  - {test_id: tc193_f24_f25_f26_f31_width_conversions_agree, ac_id: IT-017-SC-03, trace: correct}
  - {test_id: tc193_f24_f27_f28_exact_conversions_agree, ac_id: IT-017-SC-03, trace: correct}
  - {test_id: tc193_f33_negation_meter_agrees, ac_id: IT-017-SC-03, trace: correct}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both QSL and runtime Incomplete branches now compare limits against their respective configured ScalarLimits and next_charge against the authored expectation for width, rational, and F33 cases. The QSL function-call envelope applies only to WorkUnits. Verified at PR #52 head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`. |

## Final disposition

**PASS.** FND-001 is fixed at PR #52 head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`; no new finding in the fix diff.

## Lock refresh verification

The later dependency refresh at PR #52 head `b4ca5a2e81218630429acba25652183c6c23dca4` changes only the locked QSL cohort from `3e8193af` to `4f12efb7` and the required transitive IR model from `d9e17c77` to `1cd5857f`; Exact stays at `7e06eba7`. The reviewed `tests/ieee_source_agreement.rs` is byte-identical to the previously reviewed head `4803e50b7d332f47d55561ecb1d0000f2c4b44b6`. The asserted denial fields and rendering checks remain in place. The existing FND-001 disposition remains fixed; no new finding in this lock delta. Coder-reported focused evidence: IEEE 23/23, both generated_boolean controls, fmt, all-target Clippy, and diff check pass. **PASS** for this confined refresh review; code lead owns the aggregate gate.
