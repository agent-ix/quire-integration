---
id: SR-16370
title: "IR-669 PR 37 F-row gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; Cargo.lock, tests/ieee_source_agreement.rs, tests/scalar_agreement_ieee_text.rs; IT-017-SC-01 through IT-017-SC-05, TC-097"
review_set: subset
---

## Summary

Ticket: IR-669. Planless changed-test trace and evidence audit for draft PR #37. The QSpec F-row API is referenced by the tests, but the reviewed family does not yet close every runnable row or make an unknown row fail.

## Verdict

FAIL. The missing F-row inventory gate allows coverage inflation even if every named test passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | No exhaustive TC-193 vector dispatch maps every published F ID to a runnable test or an exact ticket-backed PENDING cause; unknown and omitted runnable rows are invisible to the suite. | tests/ieee_source_agreement.rs:34-47 |
| FND-002 | medium | The new F10 and F17 tests do not verify all named limit and denial fields in TC-193; F13 lacks its operation-local flag sequence, and F20 omits multiple admission-only subcases. These trace-tagged tests therefore do not yet establish all claimed SC-03/SC-04 evidence. | tests/ieee_source_agreement.rs:717-784 |

## Coverage

Plan completion: not assessed. `quoin matrix --repo ... --json` returned no coverage_matrix criteria because the installed module declares no `obligations:` source. Fell back to a manual scan of IT-017-SC-01 through SC-05, TC-097, the changed `#[trace]` tests, and QSpec TC-193 F01-F34. No reverse ownership gap, stub, ignored test, vendored fixture, or copied corpus was found in the changed files. Optional semantic review was not separately requested; the concrete test-to-criterion checks above are the scoped reviewer pass.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `f_case` provides exhaustive TC-193 ID routing with an unknown-ID panic and ticket-backed PENDING entries. |
| FND-002 | fixed | F10/F17 named limit checks, F13 locality sequence, and F20 admission branches now have trace-tagged source/runtime observations; the separate FND-006 records the blocked Partial runner regression. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The newly runnable F02 and F05 paths omit authored subcases: F02 never checks the second qNaN's bit identity to itself, and F05 never checks that each exchanged ascending pair fails `<=`. QSpec TC-193 F02 says each qNaN is bit-identical to itself; QSL FR-397-AC-24 explicitly requires reverse `<` and `<=` for each F05 pair. | tests/ieee_source_agreement.rs:1915-1930,2044-2057 |

## Current review pass

At the reviewed draft head against QSpec TC-193, the new F04b/F11, F01/F02/F02b, F05, F20 and F34 trace claims were checked manually because the installed matrix has no obligations source. The two omitted authored subcases above are the only new coverage finding. IR-1283 conversion/negation cases remain ticket-backed PENDING; IR-1282 equality charge assertions remained retained but unrun. No passing focused test or final gate was claimed while IR-1277 blocked compilation.

## Disposition pass 3

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | F02 now checks second_nan bit identity to itself; each F05 pair now checks reverse LessOrEqual is false. |

## Disposition pass 4

No new findings in the reviewed delta. The F dispatcher now runs F01/F02/F17 strict metering; IR-1283 conversion and negation subcases remain ticket-backed PENDING. No false whole-F completion claim was found. The focused test evidence is 19 passed and zero failed; the final aggregate gate is reserved for the code lead.

## Disposition pass 5

No new trace or runnable F-row gap in the driver forwarding and lockfile relock. The committed artifact status correction is recorded as SR-16369 FND-009.

## Current status

The initial FAIL verdict and earlier IR-1282/IR-1277 blockers above describe historical review passes. The reviewed runnable F rows now execute strict F01/F02/F17 metering, F20 admission refusal, FMA, ordering, identity, arithmetic, and named meter assertions. `cargo test --test ieee_source_agreement` passes (19 passed, zero failed or ignored), and the strict IR-914 export regression passes (1 passed). IR-1283 conversion and negation subcases remain ticket-backed PENDING. IR-1301's transitive QSL filament-core-data split holds the final aggregate gate; no full-gate result is claimed.

## Disposition pass 6

No new trace or F-row coverage gap at the frozen PR head. Six F and three IR-914 SourceIdentity constructor changes preserve test routes and assertions. The full F dispatcher remains fail-closed for unknown IDs; F24/F25/F27/F28/F31/F32/F33 and the F26 conversion portion remain ticket-backed PENDING under IR-1283. The focused F and IR-914 test results were supplied by the coding lane; the lead owns the final aggregate gate.

## Disposition pass 7

The incremental `SourceIdentity::new` constructor migration at head `f429337` changes nine fixture calls across four existing tests without changing their trace tags, assertions, or oracle paths. Scoped gap disposition: PASS, no new finding. Final non-proof gate evidence remains pending on the next head; `make kani` is unrun under the standing 16 GiB proof restriction.

## New findings (disposition pass 8)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | D13, D18 and D24 now execute only RT while their QSL composition is marked PENDING under IR-1312. IT-017-SC-01/05 and TC-097 require an open upstream owner ticket for each PENDING path; structured IR-1312 is the integration-owned D04/D05 fixture ticket, whose acceptance criteria do not own these three source-composition decisions. Their QSL observations are therefore not validly ticket-backed. | tests/scalar_agreement_decimal.rs:359-376, 888-927, 1055-1089, 1316-1358; spec/integration/IT-017-runtime-scalar-agreement.md:81-86,124-130 |

## Disposition pass 8

The unknown-ID panic and QSpec TC-185 dispatch remain exhaustive. D13/D18/D24 retain the RT-only values, losses and charge assertions, and D25 retains an exact S4 type-mismatch assertion. Those three rows cannot count as two-sided agreement while their QSL paths are absent. FND-004 remains open until each missing path has an actual upstream owner ticket and the PENDING routing names it; an integration-owned D04/D05 fixture ticket does not satisfy IT-017's owner requirement.

## Disposition pass 9

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | D13, D18 and D24 now cite QSL-1619, an open upstream QSL/QSpec ticket whose structured scope and acceptance criteria own all three missing source-composition paths. Their RT oracles remain active, and the rows are not claimed as QSL/RT agreement. |

The repaired dispatcher names QSL-1393 PENDING for the division-source rows D04/D05/D06/D09/D12/D14/D15/D17 and QSL-1619 PENDING for D13/D18/D24. D25 remains partly PENDING under QSL-1394 while its mixed-profile refusal and explicit conversion admission run. Unknown QSpec IDs still panic. Titan's focused decimal run passed 1, failed 0, ignored 3 at the reviewed head. These PENDING paths are unqualified and do not complete IT-017 or TC-097; no new trace or gap finding in the repair delta.
