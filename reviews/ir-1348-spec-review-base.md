---
id: SR-2026110414560133
title: "IR-1348 PR 84 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-integration; IR-1348 PR 84; TC-106 U12 and IT-019 generated agreement"
review_set: base
---
## Summary

Reviewed the changed disposition against the TC-106 procedure and actual public QSL API. The generated-agreement status is supported, but one procedure step still names a removed evaluator API.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-106 step 4 still instructs evaluation of the selected U12 checked subexpression with `evaluate_closed`, although the test now correctly uses QSL `evaluate_closed_with_frame` and the retained checked function's formed-unit frame. Update the procedure to name the frame-aware public API and its authentic frame. | spec/test-cases/TC-106-quantity-static-admission.md:39; tests/support/tc187_p9_source.rs:149 |

## Verdict

CONDITIONAL. Correct TC-106 step 4. The changed IT-019 and TC-106 U12 status statements match the observed eight native cases.

## Coverage

Static spec review only; no tests rerun. No new requirement statement, object schema, or relationship edge requires a separate sub-analysis. The recorded U12 values, compound identities, losses, charges, counters and bounded retention agree with the test's asserted cases. The original QSL refusal location is asserted separately; native snapshots do not claim to contain it.

## Examined scope

```yaml
scope:
- {id: TC-106-Step-4, path: spec/test-cases/TC-106-quantity-static-admission.md, role: examined, excerpt: "Evaluate that original node directly with evaluate_closed; do not cut or reconstruct a body, evaluate the equality wrapper, or subtract any charge."}
- {id: TC-106-U12, path: spec/test-cases/TC-106-quantity-static-admission.md, role: examined, excerpt: "Original package emits, strictly admits through IR and yields four ir_confirmed CG claims whose generated native symbols agree with independently rechecked original operation nodes."}
- {id: IT-019-SC-04, path: spec/integration/IT-019-generated-oracle-qsl-agreement.md, role: examined, excerpt: "TC-187 U12 generated arithmetic agreement Passed (four completed and four unpaid-retention native observations)."}
- {id: tests/support/tc187_p9_source.rs, path: tests/support/tc187_p9_source.rs, role: context_only, excerpt: "evaluate_closed_with_frame uses the checked function's retained formed_units"}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-106 step 4 now names `evaluate_closed_with_frame` and passes the checked function's retained `formed_units` with its genuine zero-argument frame. |

### Round 1 verdict

PASS. Static inspection of commit 61549fc2 confirms the procedure matches the tested public QSL API. No Cargo or wider review rerun.

## Post-review scope extension at f001185a

Focused static review found the updated IT-019 and TC-106 U22/U28 disposition rows consistent with the added test evidence. U22 now claims an isolated original QSL conversion result and conversion-only accounting, while explicitly retaining generated conversion as Planned. U28 claims one completed generated native observation; the two size-stop source observations remain source-only. U25 and U27 remain Planned with their measured source boundaries. No new specification defect or additional sub-analysis was identified.
