---
id: SR-2026110414560043
title: "IR-842 PR 57 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-integration; spec/test-cases/TC-101-generated-operation-family-agreement.md, tests/generated_oracle_family_function.rs, tests/support/family_function_observation.rs, tests/support/family_function_selection.rs, tests/generated_oracle_sc03.rs, tests/support/function_agreement.rs, tests/support/function_observation.rs"
review_set: subset
---

## Summary

Ticket: IR-842. Reviewed the three-file function cohort, Rust conventions, native execution seam and author receipts. The native assertions cover the selected operations and boundaries; duplicated checked-graph selectors violate the required single-home rule.

## Verdict

FAIL. One high duplication finding requires consolidation before approval.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new declaration/body selectors duplicate the existing declared/body helpers in generated_oracle_sc03.rs and support/function_agreement.rs; graph selection and body-reference decoding now have additional independent implementations that must change together. | tests/generated_oracle_family_function.rs:34; tests/generated_oracle_family_function.rs:53 |

## Coverage

No applicable AssuranceProfile was found under spec/. Examined the TC description, each of its six procedure steps and all Expected Results paragraphs. The outer test binds TC-101 and IT-019-SC-05; this binding supports only the named initial cohort. The authentic source routes through checking, emitted V2 strict admission, authenticated lowering, public function generation and actual generated native exports. Fresh authority acquisition and registries register no local body factory. Integer 10 distinguishes actual bounded source admission from scalar Integer classifier metadata. The original owner mismatch is followed by successful original-source calls.

The driver compares full Integer/incomplete outcomes, locations, ordered located losses, admitted charge records and all consumed counters. Independent expectations require values 1/12/1, direct versus nested work-one stop points and genuine Body ordinals 0/1/2 at depth zero. Invalid arguments require exact native arity/kind precedence and zero accounting. Other FR-021 operations and result/capability/occurrence/sibling boundaries, plus FR-014/FR-018 inventories, remain Planned under IR-842.

Duplicate search: searched declaration/body/selected helper definitions in tests/generated_oracle* and tests/support. Existing implementations are tests/generated_oracle_sc03.rs:33 and :56, and tests/support/function_agreement.rs:20 and :41. The new observer also defines its own declaration selector. Consolidate checked-package selection in one reusable test-support module; preserve the fresh acquisition and real native execution paths.

## Verification limits

Author evidence inspected: quire-integration 0.1.0 native cohort run, 1 passed / 0 failed / 0 ignored in 1.49 seconds; host compilation and focused Clippy succeeded; format and documentation checks succeeded with advisory module/formatter warnings. Native generated compilation also reports existing cfg(kani) and unused-import warnings; these are not represented as warning-free evidence. No Cargo, test, Clippy, build, Kani or final gate was rerun by the reviewer, per the dispatch scope. The lead owns the final serialized gate.


## Examined units

```yaml
scope:
- id: TC-101-description
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: 'Refine IT-019-SC-05 for an initial FR-021 whole-function cohort: the public

    `ExactFunctionBody::Scalar` Subtract and Multiply alternatives, and a genuine

    `ExactFunctionBody::Call` whose selected callee performs subtraction. Existing

    Add evidence does not establish these operations. This procedure supplies no

    FR-014 operation-oracle or FR-018 equality-oracle agreement credit.

    '
- id: TC-101-step-1
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "1. Author ordinary pure subtraction, multiplication and relay functions\
    \ with\n   two `Int[0,9]` parameters and an `Integer` result. The relay calls\
    \ the actual\n   subtraction declaration. Separate exercise functions supply genuine\
    \ selected\n   call occurrences. Acquire the source through public QSL checking/emission,\n\
    \   strictly admit its exact V2 bytes, and derive original declarations, body\n\
    \   roots, ordered argument references and scalar type handles. Do not substitute\n\
    \   manually authored IR, source identities or a copied fixture."
- id: TC-101-step-2
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "2. Lower those actual roots and admit the authenticated contract with\
    \ its source\n   package. Generate through the public exact-function API, using\
    \ the original\n   request positions and the corresponding public body variants.\
    \ Require each\n   selected item to generate; retain its original call/function\
    \ IDs and returned\n   public oracle symbol. Compile the actual artifacts through\
    \ the existing native\n   helper. No proof executor or locally supplied body implementation\
    \ completes\n   the generation path."
- id: TC-101-step-3
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "3. Independently reacquire the original source and select its actual callable\n\
    \   under the original owner in a fresh FR-485 registry. Register no generated\n\
    \   factory in that authority registry. Invoke the generated public symbol and\n\
    \   the independent session with Integer arguments 4 and 3 and identical fresh\n\
    \   global allowances at work 0, 1 and 1024. Compare complete typed outcomes and\n\
    \   call failures, locations, ordered located losses, admitted charges and every\n\
    \   exposed consumed counter without subtraction, normalization or envelope\n\
    \   reconstruction."
- id: TC-101-step-4
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "4. Independently require successful results 1, 12 and 1 for subtraction,\n\
    \   multiplication and relay respectively. Success has no result location or\n\
    \   located loss. Work 0 stops at FunctionCall with no admitted charge. Work 1\n\
    \   admits exactly the outer FunctionCall: direct arithmetic stops at\n   IntegerArithmeticOperands;\
    \ relay stops at the callee's FunctionCall. Both\n   stops retain WorkUnits, the\
    \ original limit and consumed work, next charge 1,\n   no value/loss, and the\
    \ real checked body location. All other consumed scalar\n   counters remain zero.\
    \ These required terminal shapes prevent two"
- id: TC-101-step-5
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "5. Exercise argument arity and kind on the real public application boundary,\n\
    \   including an argument list with both a wrong count and wrong kind so the\n\
    \   native precedence is distinguishable. Include Integer 10 as a genuine value\n\
    \   outside the authored Int[0,9] parameter domain, to distinguish the source's\n\
    \   bounded admission from the generator's scalar Integer classification.\n  \
    \ A widened generated input path fails; it is not normalized or credited.\n  \
    \ Compare original native CallFailure\n   payloads against the independent session\
    \ and require zero admitted charges\n   and counters. Separat"
- id: TC-101-step-6
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: "6. Keep the full public FR-021 selection rule explicit. Scalar expands\
    \ the\n   owner's IntegerOperator: Add, Subtract and Multiply have binary execution\n\
    \   paths; Negate is an owner-defined unsupported unary selection requiring its\n\
    \   own actual typed generation observation. CompositeEquality expands Equal\n\
    \   and NotEqual over owner-permitted Boolean, Integer, record, tuple and union\n\
    \   operands. Call requires a genuine selected callee. Each form additionally\n\
    \   needs its prescribed result/signature, capability, owner/occurrence,\n   duplicate/ambiguous\
    \ request, refused-item/sibling and operatin"
- id: TC-101-expected-results
  path: spec/test-cases/TC-101-generated-operation-family-agreement.md
  role: examined
  excerpt: 'The initial cohort has executed two-sided native agreement. Each selected

    generated function and independent source session agrees on the complete exposed
    envelope,

    with the independent required values and stops above. Source authentication and

    argument refusals remain distinct native boundaries and never become values.


    The initial rows are Scalar/Subtract, Scalar/Multiply and Call/subtraction-callee,

    with their named argument and owner controls. Existing Add and SC-03 observations

    retain their own separate evidence. CompositeEquality operations, Negate''s typed

    unsupported generation, unexecu'
```

## Disposition round 1 coverage

Expanded review includes tests/support/family_function_selection.rs, tests/generated_oracle_sc03.rs, tests/support/function_agreement.rs and tests/support/function_observation.rs. Each host caller imports the canonical module. Each native driver receives that same module through source inclusion before generated compilation. The module imports checked-model types from its parent so host dependency naming and generated dependency naming remain correct. Name selection still rejects missing or ambiguous declarations, and body selection still requires exactly one typed reference. Original arithmetic, meter comparisons, ownership refusals and occurrence/sibling assertions remain unchanged.

Focused author Clippy and no-run compilation passed for generated_oracle_family_function, generated_oracle_qsl_agreement and generated_oracle_sc03. These validate the host integration targets; embedded native drivers still require the lead’s final execution gate. The prior native 1.49-second result predates this refactor and is not final-head execution evidence. No reviewer build or execution was run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | One canonical declaration/body selector now serves all affected host and embedded native callers; redundant selectors are removed. |

## Current verdict

PASS after disposition round 1. FND-001 is fixed; no new finding was found in the expanded refactor. Final serialized native execution remains the lead’s gate.
