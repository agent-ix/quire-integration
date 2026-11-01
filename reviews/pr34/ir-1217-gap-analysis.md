---
id: SR-16367
title: "IR-1217 scenario and trace gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; IR-1217 PR34; tests/qsl_model_owner_admission.rs, tests/ir914_bound_export.rs, spec/integration/IT-012-model-owner-admission.md, README.md"
review_set: subset
---
# IR-1217 scenario and trace gap analysis

## Summary

Ticket: IR-1217. PR: quire-integration#34. One independent review with the Rust lane folded into code-review. Initial source c5e7f3e and same-PR documentation fix 6fd79c3 were inspected. No production implementation, dependency, CI or proof-lane change occurs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | IT-012 still requires digest-keyed evidence, a lock model-selection entry and a differing lock selection across versions, while the migrated tests correctly use declared-identity ModelRoot selections and assert that those selections remain equal; the active scenario contradicts its tests. | spec/integration/IT-012-model-owner-admission.md:30-45 | wrong-requirement |

## Verdict

PASS for the reviewed slice after the same-PR fix. Merge remains subject to the parent's confirmed format-check exit status. All discovered defects are fixed; no production or test behavior changed in the documentation fix.

## Coverage

The locked all-target Clippy receipt finishes successfully. The two touched test binaries report ten regular tests passed and the two existing Kani controls ignored. The reviewer ran no Cargo or Kani command. Formatting receipt is parent-owned; its empty output alone is not interpreted as an exit code. The owner must confirm exit zero before merge. No 16 GiB proof was run or credited. No applicable AssuranceProfile was found. Source/test changes preserve real QSL emitter, IR reader, driver export and CG generation boundaries; no shim, mirror semantic type or new vendoring was found.

Plan completion: not assessed

Computed matrix command emitted engine metadata only, without criterion records, so the declared fallback is a manual index of IT-012 SC1–3 and IT-013 SC1–8 against existing trace attributes. SC5/6 remain named ignored backend controls; this slice restores compilation and makes no new installed-Kani claim. Repository-wide completion and optional full semantic audit are not claimed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | IT-012 now keys evidence by declared identity, reads the selected ModelRoot owner, checks exact Gadget/Widget owners and retains selected-root and declaration-node identity equality across document-version changes. |

## Examined Scope

- **IT-012-SC-01** (examined): the reader admits the package, its lock names the selected domain package, and a document supplied under another digest is refused.
- **IT-012-SC-02** (examined): the ids of the model nodes IR admitted are exactly those.
- **IT-012-SC-03** (examined): the lock's selection differs and the model node ids are the same.
- **IT-013-SC-01** (examined): codegen receives two bound envelope items and full positions at indexes 0, 1 and 2; both settlement layers and generated items name 0 and 2, while the driver retains B's exact typed refusal and occurrence at 1.
- **IT-013-SC-02** (examined): the claim is IR-confirmed and generated with a nonempty source map and checked bounds.
- **IT-013-SC-03** (examined): codegen generates the IR-confirmed Boolean negation claim with a nonempty source map. This control makes no Boolean proof claim.
- **IT-013-SC-04** (examined): codegen returns one supported Kani record with a retained scalar harness, and QSL's checked-package call of `step(4)` completes with the integer 5.
- **IT-013-SC-05** (examined): the proof verdict is `Verified`, and the native integer result equals the checked QSL `step(4)` result of 5.
- **IT-013-SC-06** (examined): real Kani returns `Falsified` with a counterexample on the generated soundness assertion, rather than a compilation refusal.
- **IT-013-SC-07** (examined): a package without an abstraction relation skips `export_bindings`; every item reaches codegen and no item is refused.
- **IT-013-SC-08** (examined): QSL itself returns the model item's typed `Unbound` refusal; the driver retains that exact refusal and original index, withholds the refused item from codegen and generates both `Ok` siblings.
