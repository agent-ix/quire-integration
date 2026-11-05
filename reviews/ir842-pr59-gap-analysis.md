---
id: SR-2026110414560057
title: "IR-842 PR 59 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration; tests/generated_oracle_family_scalar_operations.rs, tests/generated_oracle_family_function.rs, tests/support/family_scalar_limits.rs, tests/support/family_function_observation.rs, spec/integration/IT-019-generated-oracle-qsl-agreement.md, spec/test-cases/TC-102-generated-scalar-operation-agreement.md, spec/test-cases/TC-103-generated-operation-family-agreement.md"
review_set: subset
---

## Summary

Ticket: IR-842. Selected eager Boolean operation qualification uses authentic emitted V2 bytes, strict admission, public scalar derivation and actual generated symbols. Independent source checking supplies the same closed-body occurrence; typed authored truth values and exact zero/one-work stop expectations discriminate matching wrong computations.

## Verdict

PASS (scoped manual review). No defects found in the examined changed units. No whole-family or full SC-05 closure is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

No applicable AssuranceProfile was found under spec/. Plan completion: not assessed

Manual narrow trace inspection examined TC-102 and IT-019-SC-05 on the new selected scalar test, and the TC-103 rename on the existing function test. TC-101 remains owned by the unrelated temporal-logic test case. Six selected public operations comprise 22 truth cases at work 0, 1 and 1024, giving 66 observations. Source bodies are closed Boolean expressions, selected through the canonical declaration/body helper. Each generated claim retains its original node and IrConfirmed provenance. Complete typed outcomes, losses, ordered admitted charges and every LimitKind counter are compared by the existing mechanical snapshot helper. QSL losses are asserted empty before extracting their payloads. Exhausted-work stops independently require the exact limit, zero consumption, next charge and charge point, no admitted charges, all counters zero and a real QSL location. Callable session locations are outside this operation contract.

Lazy right_may_stop=true, parameterized Integer operations, remaining scalar groups and composite equality remain Planned under IR-842. Different literal bodies cannot replace original parameterized occurrences or domains. FR-021 retains whole-call FR-485 authority. The parent now reflects the previously merged function cohort without rerunning or expanding its evidence. No SC-04 dispatcher or vector source was changed. The base checklist checked identifiers, changed links, procedure observability, bounded claims and absence of introduced file-tracking machinery. Owner operation provenance is runtime semantic evidence, not a file-tracking record.

Duplicate search examined tests/generated_oracle* and tests/support for existing selectors, observations, lifting and allowance helpers. Canonical selection, native execution and observation are reused; the scalar allowance helper repeats tests/support/family_function_observation.rs.

## Verification limits

Reviewed author logs for quire-integration 0.1.0: focused native test passed 1/1 before dead-branch cleanup; final focused compilation and Clippy passed after cleanup. Format and spec logs contain existing formatter/module advisory warnings. These receipts are not final-head native execution evidence. No reviewer Cargo, compilation, native test, Clippy, Kani, global matrix or advisor run was performed under the dispatch restrictions. Gap analysis uses a manual narrow trace index instead of a recomputed global matrix and makes no repository-wide coverage claim. The lead owns the final serialized scoped execution gate.

## Examined units

```yaml
scope:
- id: TC-102-unit-0-0
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: '

    # TC-102: Execute generated Boolean scalar operations'
- id: TC-102-unit-2-0
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: 'Qualify selected SC-05 ExactScalar operation rows against independently
    checked

    QSL operation bodies. This operation boundary is distinct from FR-021 whole-call

    sessions. The same original closed checked body supplies CG''s emitted claim and

    public QSL checked-body evaluation after an independent source acquisition.

    Neither evaluator is reproduced.'
- id: TC-102-unit-4-0
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: "1. Check original QSL source, emit its exact checked-package/v2 bytes\
    \ and strictly\n   admit them. Select each closed function's original body using\
    \ the shared\n   declaration selector. Derive scalar items and require genuine\
    \ generated claims\n   with original node identities.\n2. Independently reacquire\
    \ the source through parsing, assembly and checking.\n   Evaluate each corresponding\
    \ closed operation body with fresh global allowances.\n   Run actual emitted ExactScalar\
    \ native artifacts using the same operands and\n   allowances through the existing\
    \ generated-artifact helper.\n3. "
- id: TC-102-unit-4-1
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: "Exercise Boolean Not and full binary truth tables for And, Or, Implies,\n\
    \   Equality/Boolean Equal and NotEqual: 22 authored cases across six public\n\
    \   operation rows. Boolean variants explicitly use `right_may_stop=false`; the\n\
    \   `right_may_stop=true` lazy and supplied-stop controls remain Planned.\n  \
    \ Independently require authored expected truth values rather than merely\n  \
    \ mutual agreement.\n4. Compare complete typed operation outcomes, ordered losses,\
    \ ordered admitted\n   charges and all LimitKind consumed counters at generous\
    \ work and work zero/one.\n   Require genuine inc"
- id: TC-102-unit-4-2
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: "omplete WorkUnits terminal records under exhausted work;\n   never subtract\
    \ call overhead, precharge or reconstruct an evaluator envelope.\n5. Leave Integer\
    \ arithmetic/equality/ordering/division/modulo comparison Planned\n   under IR-842.\
    \ The public `FunctionState.body` is a checked node with slots;\n   `evaluate_closed`\
    \ supplies no parameter bindings. The public parameterized\n   expression evaluator\
    \ requires `CheckedExpression`, and the locked API does not\n   expose a conversion\
    \ from the original function body to that expression.\n   Comparing a different\
    \ closed literal body wo"
- id: TC-102-unit-4-3
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: "uld change the admitted occurrence\n   and Integer domains. This is missing\
    \ integration comparison evidence, not a\n   measured derivation refusal or an\
    \ upstream API blocker. Other scalar families\n   and unexecuted boundary controls\
    \ remain Planned; never claim a whole family."
- id: TC-102-unit-6-0
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: 'Every selected generated symbol executes natively and agrees with its
    independent

    QSL operation authority on the complete exposed observation. Independent Boolean

    truth values distinguish matching wrong computations. Exhausted

    work preserves the typed stop and all accounting without normalization.

    At work zero, connectives stop at BooleanResultRetain with next charge 1;

    equality stops at EqualityPlanForm with next charge 2. Work one completes every

    connective and still stops equality at EqualityPlanForm. Each stopped body has

    its real location, no losses, no admitted charge'
- id: TC-102-unit-6-1
  path: spec/test-cases/TC-102-generated-scalar-operation-agreement.md
  role: examined
  excerpt: 's and zero consumption in every

    LimitKind counter. All 22 cases execute at work 0, 1 and 1024: 66 complete

    operation observations. Any row

    that does not genuinely derive is reported separately and receives no agreement

    credit. Full IT-019-SC-05 and family closure remain Planned under IR-842.

    '
- id: IT-019-unit-0-0
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "QSL public parse, check and checked-package emission \u2192 exact emitted\
    \ V2 bytes\n\u2192 Contract IR public strict admission \u2192 CG public scalar,\
    \ composite equality\nor exact-function generator \u2192 compile and call the\
    \ actual generated native\noracle symbol. Independently reacquire the original\
    \ QSL source, select the\nchecker-issued owner and occurrence, and evaluate it\
    \ through fresh public QSL\nauthority. FR-021 uses whole-call FR-485 sessions;\
    \ FR-014 and FR-018 may use\npublic checked-expression evaluation for complete\
    \ operation observations, as\ndefined in SC-05 below. For TC-187, obtain "
- id: IT-019-unit-0-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'authored vector IDs and prose through

    `quire_specification::vectors_of(TestCase::Tc187)` and

    `QUALIFICATION_VECTORS`, by reference. Integration owns executable dispatch

    for those IDs; it neither copies the corpus nor implements QSL, IR or CG

    semantics. The public vector API''s prose is not itself an executable request.'
- id: IT-019-unit-1-0
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "1. Compile the `combine` source through QSL's public checked-package path,\n\
    \   admit its exact emitted bytes with Contract IR's public reader, derive the\n\
    \   real function, body and call roots, generate through CG's public exact\n \
    \  function API, compile the produced crate, and invoke its actual\n   `GeneratedExactFunctionClaim::oracle_symbol`.\
    \ In a separately acquired\n   FR-485 session for the original source, look up\
    \ the checker-issued\n   callable and apply it with the same arguments and a fresh\
    \ meter at work=1.\n   Compare the complete outcome variant and payload, `Location`,"
- id: IT-019-unit-1-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: " ordered\n   admitted charges, every exposed consumed counter and denied-charge\
    \ fields\n   directly, without normalizing or subtracting the function-call envelope.\n\
    \   IT-019-SC-01: both paths return the same `Incomplete` with\n   `WorkUnits`,\
    \ limit 1, consumed 1, next charge 1 and\n   `IntegerArithmeticOperands`; their\
    \ one admitted `FunctionCall` charge,\n   all counters and `Some(Location { Body\
    \ { function: \"combine\", index: 0 },\n   depth: 0, last_index: None })` agree\
    \ exactly. A missing generated location,\n   a synthetic QSL evaluator, or a nonexecuted\
    \ oracle fails this criter"
- id: IT-019-unit-1-2
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "ion.\n2. Re-run that identical selected function and arguments with fresh\
    \ sessions\n   and meters at work=0 and work=1024. Compare the entire outcome,\
    \ location,\n   ordered charges and every counter for each limit; retain any located\
    \ losses\n   in the completed envelope.\n   IT-019-SC-02: work=0 stops at `FunctionCall`\
    \ before body work with no\n   admitted charge and the same real body location;\
    \ work=1024 completes with\n   integer 7 and identical located losses, charge\
    \ sequence and counters.\n   Neither result can compensate for a failure at work=1.\n\
    3. Submit genuine wrong-owner a"
- id: IT-019-unit-1-3
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "nd wrong-occurrence selections through the\n   public QSL/IR/CG boundaries,\
    \ then submit a mixed request containing a\n   refused item and a healthy sibling.\
    \ Preserve each stage's typed refusal,\n   original request index, source owner\
    \ and occurrence; invoke no oracle for\n   a refused item.\n   IT-019-SC-03: a\
    \ mismatch is refused before generated execution or meter\n   charge, the refusal\
    \ is distinguishable from a QSL lookup or input refusal,\n   and the healthy sibling's\
    \ generated symbol, disposition and independent\n   QSL comparison are unchanged.\
    \ No replacement identity or inde"
- id: IT-019-unit-1-4
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "x is minted.\n4. Enumerate TC-187 through QSpec's public vector API. For\
    \ each authored ID,\n   classify its distinct operations as admission-only, eligible\
    \ generated\n   evaluation, or Planned with a named open owner ticket and missing\
    \ path.\n   Dispatch eligible rows to authentic QSL source \u2192 emitted bytes\
    \ \u2192 IR\n   admission \u2192 relevant CG public generation \u2192 executed\
    \ native oracle and\n   independent QSL evaluation. Compare full results, typed\
    \ refusal or\n   incompleteness, locations, exposed ordered charges and counters.\
    \ Check the\n   QSpec-stated expected result as an additional"
- id: IT-019-unit-1-5
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: " oracle where it specifies\n   one; never treat admission-only rows as\
    \ generated value agreement.\n   IT-019-SC-04: every API row and required operation\
    \ has one explicit\n   classification and observed path or ticket-backed Planned\
    \ entry; unknown\n   IDs, silent skips, copied fixtures and ticketless Planned\
    \ entries fail.\n   Only executed two-sided rows with matching owner expectations\
    \ count as\n   generated-oracle/QSL agreement.\n5. Select the required operation\
    \ set from the public owner types resolved by\n   this integration's Cargo.lock,\
    \ rather than a copied catalog or fixed cou"
- id: IT-019-unit-1-6
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "nt:\n   - FR-014 selects every variant of CG `ExactScalarOperation`, expanded\
    \ by\n     every operator and operand-kind variant in its fields. Include all\n\
    \     owner-defined division profiles, IEEE widths, comparisons and rounding\n\
    \     choices, text profiles, and both values of Boolean `right_may_stop` where\n\
    \     meaningful for the connective. Domain values and boundary cases follow\n\
    \     FR-014 and the selected TC-187 expectations, rather than an enumeration\n\
    \     of arbitrary numeric endpoints. Preserve owner-defined invalid\n     combinations\
    \ as typed refusals, not silently om"
- id: IT-019-unit-1-7
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "itted operations.\n   - FR-018 selects both `EqualityOperatorKind` variants\
    \ for every operand\n     declaration and collection shape consumed by the public\
    \ composite\n     equality generator, including recursively reached shapes and\
    \ leaf kinds\n     defined by the owner `CompositeShape` and `ValueType` contracts.\
    \ Expand\n     `EqualityOperandDescriptor` source-only and conversion-target forms\
    \ on\n     each operand wherever the owner permits them; unsupported combinations\n\
    \     retain their own typed dispositions.\n   - FR-021 selects every `ExactFunctionBody`\
    \ variant, expanded by i"
- id: IT-019-unit-1-8
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "ts integer\n     operator or equality operator and owner-permitted operand\
    \ shapes, and\n     the nested-call form with a genuine selected callee. Include\
    \ the\n     argument, result, owner, occurrence and capability boundaries required\n\
    \     by FR-021 for each form.\n   For every selected operation/shape/choice,\
    \ record a distinct row naming its\n   public owner variant, genuine QSL source-to-IR-to-CG\
    \ route and executable\n   test symbol. Each row must contain an executed two-sided\
    \ result, an\n   observed typed nonexecuted generation disposition with sibling\
    \ isolation,\n   or `Planned"
- id: IT-019-unit-1-9
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "` with the exact missing admission/generation/authority path\n   and an\
    \ open owner ticket. Unimplemented integration dispatch is owned by\n   IR-842;\
    \ it is not attributed to an unavailable upstream semantic feature.\n   A new\
    \ owner variant adds a required row; a missing or unknown variant\n   fails inventory\
    \ closure, rather than inheriting another row's result.\n   IT-019-SC-05: every\
    \ operation selected by this owner-derived rule has an\n   explicit disposition.\
    \ Runnable cases compare the full exposed envelope\n   against independent QSL\
    \ authority. FR-014 and FR-018 operation qual"
- id: IT-019-unit-1-10
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "ification\n   may use fresh, independently acquired public checked-expression\
    \ evaluation\n   of the same admitted source occurrence, operands, domain and\
    \ type. Compare\n   every public operation outcome, loss, ordered admitted charge\
    \ and consumed\n   counter without subtraction or precharging. This supplies operation\
    \ evidence,\n   distinct from FR-021's FunctionCall-level FR-485 session envelope;\
    \ it supplies\n   no callable owner or session-location evidence. A missing complete\
    \ public\n   observation leaves that row Planned with the exact missing API path.\
    \ A\n   different source bo"
- id: IT-019-unit-1-11
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "dy with mismatched Integer domains is not a substitute.\n   A typed-refused\
    \ or unsupported selection\n   is never converted to a Boolean QSL outcome, and\
    \ a passing scalar sample\n   cannot mark the composite or function family complete.\n\
    6. Publish the per-vector and per-family evidence table with status\n   `Passed`,\
    \ `Failed` or `Planned`, test symbol, observed boundary, and owner\n   ticket\
    \ for each Planned entry. Mark a criterion passed only when all of its\n   required\
    \ executable controls pass; keep the overall broad agreement\n   Planned while\
    \ any required row or family remain"
- id: IT-019-unit-1-12
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "s Planned or Failed.\n   IT-019-SC-06: the report distinguishes shared-kernel\
    \ value equality from\n   independent admission, selection and actual generated\
    \ execution evidence;\n   it makes no whole-family or whole-TC-187 claim from\
    \ a minimal sample."
- id: IT-019-unit-2-0
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| Scope | Status | Evidence or remaining work |

    | --- | --- | --- |

    | `combine` work=1, 0 and 1024 | Passed | `function_agreement::actual_generated_function_preserves_the_whole_qsl_call_envelope`:
    actual generated native symbol and independent FR-485 whole-envelope equality,
    including explicit work=1 body location, all three limits, exact ordered charges
    and every counter |

    | Foreign-source owner mismatch followed by original invocation (SC-03 owner control)
    | Passed | The same test observes typed `OwnerIdentityMismatch`, no admitted charge
    and zero counters, then the healt'
- id: IT-019-unit-2-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'hy original call |

    | Owner and occurrence mismatch; refused item and healthy sibling (SC-03) | Passed
    | `generated_oracle_sc03::wrong_occurrence_refuses_without_changing_the_healthy_native_call`
    uses two real QSL call occurrences and an admitted three-root contract: wrong-occurrence
    arguments yield a typed per-item `BodyMismatch`, while the healthy sibling retains
    the same generated claim, symbol and emitted library as standalone generation.
    A wrong full request position yields typed `AccountingIdentity` with the original
    accounting index and exact IDs; a wrong item node at'
- id: IT-019-unit-2-2
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: ' a valid position yields typed `Identity` with the original request index
    and exact IDs. Both fail before a claim map exists. The healthy public symbol
    executes against a fresh QSL session with whole-envelope and value agreement at
    work=0, 1 and 1024. The existing foreign-source owner test observes typed `OwnerIdentityMismatch`
    and zero charge. |

    | TC-187 U06 zero-scale and U11a two-root admission-only cells | Passed | `tc187_vector_dispatch::public_tc187_ids_fail_closed_and_real_admission_controls_refuse`
    reads both IDs and owner expectations from `vectors_of(TestCase::Tc1'
- id: IT-019-unit-2-3
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '87)`, then observes each real QSL source''s typed topology refusal, retained
    identity/value and selected source span before a checked package or generated
    oracle exists |

    | TC-187 remaining public vector IDs and mixed-row operations | Planned | IR-842:
    the same test fails on an unknown public ID and records grouped, ticket-backed
    missing-route placeholders for every other currently exported ID. These placeholders
    do not classify the operations inside mixed rows. Operation-level dispositions,
    generated execution and QSL comparison remain unimplemented. IR-669 owns separate
    QS'
- id: IT-019-unit-2-4
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'L/runtime scalar evidence |

    | FR-014 Boolean Not/And/Or/Implies with `right_may_stop=false`; Equality/Boolean
    Equal and NotEqual | Passed | TC-102 / `generated_oracle_family_scalar_operations::selected_scalar_operations_match_independently_checked_qsl_bodies`:
    22 truth cases at work 0, 1 and 1024 compare complete operation outcomes, losses,
    ordered charges and every counter against fresh public evaluation of the same
    closed source body, with independent typed truth and stop expectations |

    | FR-014 remaining operations and Boolean lazy controls | Planned | IR-842: `right_may'
- id: IT-019-unit-2-5
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '_stop=true`, parameterized Integer comparisons and the other scalar operation
    groups need their own complete observations. The public closed-body entry cannot
    bind function parameters; different literal-body Integer domains receive no agreement
    credit. IR-669 QSL/runtime results supply no generated-oracle credit |

    | FR-018 composite equality generated oracles | Planned | IR-842: per-operation
    executed native/QSL agreement remains unimplemented |

    | FR-021 Subtract, Multiply and genuine subtraction-callee Call cohort | Passed
    | TC-103 / `generated_oracle_family_function::subt'
- id: IT-019-unit-2-6
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'raction_multiplication_and_nested_call_match_fresh_qsl_sessions`: nine
    whole-call envelopes, twelve argument refusals and three alternate-owner refusals;
    exact results, locations, located losses, charges and every counter |

    | FR-021 remaining function operations and boundaries | Planned | IR-842: other
    operations and required boundaries need their own executable evidence; the passed
    cohorts supply no whole-family credit |'
- id: IT-019-unit-3-0
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: 'This test composes CG FR-014 and FR-018 with public QSL operation authority,

    CG FR-021 with QSL FR-485 whole-call authority, and the QSpec TC-187 vector source.
    Test symbols trace to the applicable

    `IT-019-SC-NN` criterion and owner family or vector ID. IT-017 and IT-013

    remain separate evidence for their narrower paths.

    '
- id: scalar-limits-helper
  path: tests/generated_oracle_family_scalar_operations.rs
  role: examined
  excerpt: 'fn limits(work: u64) -> ScalarLimits {'
```

## Disposition round 1 coverage

Expanded examination includes tests/support/family_scalar_limits.rs, tests/support/family_function_observation.rs and the generated driver injection in tests/generated_oracle_family_function.rs. The new canonical limits(work) preserves all ten original fields and values. The scalar host imports it directly; the function observer includes the same canonical source, which its host injects before native compilation. Both duplicate helper definitions are removed. Existing operation selection, independently acquired authority, truth expectations, terminal assertions and complete accounting comparisons are unchanged. No new findings.

Author focused two-target formatting, Clippy and compilation receipts passed after this refactor; formatter advisory warnings persist. No native execution after this refactor has yet been supplied. No reviewer builds or tests ran. The lead retains the final serialized gate for both affected native cases.
