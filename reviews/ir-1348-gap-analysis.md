---
id: SR-2026110414560132
title: "IR-1348 PR 84 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; IR-1348 PR 84; TC-106 U12 and IT-019-SC-04"
review_set: subset
---
## Summary

Manually checked the changed U12 acceptance claim against its executable test and dispatcher. The computed Quire matrix returned no coverage records for this repository, so this is a scoped manual trace check.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the scoped U12 generated-agreement claim. No whole-repository matrix verdict is inferred.

## Coverage

The test carries TC-106, IT-019-SC-04, QSpec-TC-187, QSpec-FR-142 and QSpec-FR-142-AC-6 trace tags. Its four completed cases assert magnitudes 6, 3, 4 and 1 plus canonical compound terms; four one-below cases assert exact UnitResultRetain stop, unpaid ledger, empty losses and original QSL operation locus. The shared generation helper emits the original checked package, strictly reads it through IR, generates four IR-confirmed claims, executes their native symbols, and compares typed outcome, losses, ordered charges and all counters against independent QSL observations. The dispatcher changes U12 alone to a generated row. Quire matrix --scope . --format json returned coverage_matrix null; manual method used.

## Examined scope

```yaml
scope:
- {id: TC-106-U12, path: spec/test-cases/TC-106-quantity-static-admission.md, role: examined, excerpt: "U12 product, quotient, square and dimensionless cancellation generated native agreement at full and one-below-retention limits"}
- {id: IT-019-SC-04, path: spec/integration/IT-019-generated-oracle-qsl-agreement.md, role: examined, excerpt: "TC-187 U12 generated arithmetic agreement"}
- {id: QSpec-FR-142-AC-6, path: quire-specification/spec/functional/type-model/FR-142-evaluate-quantities-and-units.md, role: examined, excerpt: "Multiplication, division and integer power produce exact canonical compound-unit values including dimensionless cancellation"}
- {id: tests/tc187_u12_arithmetic.rs, path: tests/tc187_u12_arithmetic.rs, role: examined, excerpt: "Eight QSL and generated native U12 observations"}
- {id: tests/support/tc187_p9_source.rs, path: tests/support/tc187_p9_source.rs, role: examined, excerpt: "Original source to strict IR to CG to generated native agreement"}
- {id: tests/tc187_vector_dispatch.rs, path: tests/tc187_vector_dispatch.rs, role: examined, excerpt: "U12 generated arithmetic dispatch cell"}
```

## Disposition pass 1

No findings required disposition. The scoped manual trace verdict remains PASS at head 61549fc2; the fix touched only TC-106 procedure wording and PR description.

## Post-review scope extension at f001185a

The U22 original conversion source-value claim is backed by the authentic cut expression, inferred parameter type, QSL-produced area value, exact 4 m² and conversion-only ledger assertions. The U28 generated agreement claim is backed by `source::generation` for the unchanged product at its completed limit; the helper requires one IR-confirmed CG claim and compares actual native output to the independently rechecked QSL operation snapshot. The two smaller U28 integer-limit observations remain source-only. No new scoped acceptance gap was found; computed matrix availability is unchanged.
