---
id: SR-2026110414560063
title: IR-842 equality gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-integration; TC-104, IT-019 SC05 equality evidence and the nine
  changed Rust integration/support files
review_set: subset
---

## Summary

Reviewed quire-integration app version 0.1.0 for IR-842 PR #61. All eleven changed files and all TC-104 procedure and expected-result units were examined. No applicable AssuranceProfile was found under spec. The peer SC04 and TC-187 files are outside this change. No dependencies, lock entries, hosted gates, or production semantics change.

The reviewed cohort has 26 input pairs at three allowances, hence 78 whole-call envelopes: eight Boolean pairs, six Integer pairs and twelve nominal pairs. Three invalid-input cases for ten declarations plus two Integer upper-bound cases give 32 input refusals; one independently checked changed owner per declaration gives ten owner refusals. Eight distinct source-backed generation controls retain exact healthy sibling claims. Unary Negate retains UnsupportedOperator. Eight Boolean operation cells at three allowances give 24 FR-018 comparisons; six Integer cells remain typed OperandTypeMismatch nonexecuted dispositions.

FR-021 uses original source acquisition, strict V2 admission, source-authenticated actual lowering, public generation, and returned native symbols. The independent FR-485 registry has no generated factory. Nominal Values come from actual independent source constructor calls outside the compared meters and the same Values reach both sides. Whole-call comparisons preserve outcome, source location, located losses, ordered charges, and every counter, with independent Boolean and stop assertions. FR-018 uses fresh independent public evaluate_closed over the original closed occurrence, original types and domains. Its snapshot is mechanical transport of full operation outcome, losses, ordered charges and all counters. Constructor subtraction, precharging, invented TypeEnvironment, evaluator copies and alternate Integer domains are absent.

Supplied final author evidence shows focused Clippy and compilation succeeded; both new native tests passed (2 passed, 0 failed), and the affected prior function and scalar cohorts each passed (1 passed, 0 failed). Spec validation reported advisory module warnings. These are author results inspected by this reviewer; no reviewer build, test, native execution, Clippy, Kani, global matrix, advisor or repository census was run. The lead owns the single final scoped gate.

## Verdict

PASS for the reviewed subset. No findings. Final scoped gate remains pending with the lead; IR-842 and broad family completion remain open.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

The placeholder row and its required Escape Cause enum are schema scaffolding only; no defect or missing evidence is asserted.

## Coverage

Manual scoped trace review substitutes for a global matrix audit by dispatch instruction. Both new executable host tests carry IT-019-SC-05 and TC-104 trace attributes and reach real generated native symbols; embedded native driver tests execute under those traced host tests. TC-102 and TC-103 affected shared-helper paths retain their focused author regressions. Reverse ownership is IT-019-SC-05/TC-104 for new integration behavior; no new production behavior is introduced. No stubs or inflated completion claims were found in this subset. This is a scoped conclusion, not a repository-wide coverage or inventory-closure claim.

Plan completion: not assessed

## Examined units

```yaml
scope:
- id: TC-104-procedure-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Author ten pure equality functions and ten separate exercise calls.\n   Acquire the original source\
    \ through QSL, strictly admit its emission, and\n   select actual declaration, body, call, argument and scalar\
    \ type handles.\n   Lower the actual requested roots and authenticate the contract against its\n   source. Generate\
    \ public CompositeEquality Equal and NotEqual declarations\n   with the real Boolean result type and source\
    \ parameter descriptors."
- id: TC-104-procedure-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Compile and invoke the returned actual native symbols. Independently\n   reacquire the same source into\
    \ an unregistered FR-485 registry. Compare\n   the complete Evaluation or CallFailure, result location, ordered\
    \ located\n   losses, admitted charges and every consumed counter with identical fresh\n   global allowances\
    \ at work 0, 1 and 1024. Do not subtract, precharge,\n   reconstruct outcomes or install generated factories\
    \ into the authority."
- id: TC-104-procedure-3-part-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Require independent Boolean truth tables for all four Boolean pairs and\n   explicit equal, less and\
    \ greater Integer examples. Successful calls have\n   the required Boolean and no location or loss. Work 0 must\
    \ stop at\n   FunctionCall; work 1 must stop at EqualityPlanForm after exactly the outer\n   FunctionCall. Stops\
    \ retain the source body origin, WorkUnits allowance,\n   consumed work and next charge 1 for FunctionCall.\
    \ EqualityPlanForm's next\n   charge is 2 for primitives and nullary union values, 6 for the Boolean pair\n\
    \   tuple and 4 for the single-field Boolean record. Ev"
- id: TC-104-procedure-3-part-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: ery other counter is zero.
- id: TC-104-procedure-4
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Compare actual argument arity and kind refusals, including combined invalid\n   count/kind precedence,\
    \ with zero charges and counters. For Integer inputs\n   require the actual Int[0,9] source bound to refuse\
    \ Integer 10. Independently\n   check and admit a changed valid source, require its original owner refusal\n\
    \   before charges, then execute the original healthy symbols."
- id: TC-104-procedure-5
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Keep deeper nominal closures, alternate union payloads and per-form\n   operational boundaries Planned\
    \ under IR-842 until their own actual source\n   cohorts and complete observations exist. Whole-function nominal\
    \ rows use\n   FR-485 sessions and have no dependency on operation-only binding support."
- id: TC-104-procedure-6-part-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Separately qualify the public FR-018 composite equality generator for\n   closed Boolean literal Equal/NotEqual\
    \ truth tables and Integer equal,\n   less and greater literal pairs. Derive operand semantic types and expression\n\
    \   IDs from original QSL emission, authenticate actual lowered contract bytes,\n   and invoke returned native\
    \ environment/oracle symbols. Evaluate each same\n   source body through a fresh independently checked public\
    \ QSL evaluator.\n   Compare complete typed outcome, all losses, ordered charges and all counters\n   at work\
    \ 0, 1 and 1024 without constructor-c"
- id: TC-104-procedure-6-part-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "harge subtraction. Independently\n   require explicit Boolean answers and exact EqualityPlanForm stops\
    \ (next\n   charge 2 and zero consumed work). Keep nominal shapes and conversions Planned\n   until their own\
    \ actual complete public observations exist."
- id: TC-104-procedure-7-part-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Add original tuple Pair(Boolean, Boolean), record Flag { flag: Boolean; }\n   and union Choice { First,\
    \ Second } declarations with Equal/NotEqual functions\n   and separate exercise calls. Obtain authentic argument\
    \ Values by running\n   independently selected closed source constructor functions through public\n   FR-485\
    \ sessions. These completed Values are external inputs to fresh whole\n   equality calls; constructor meters\
    \ are never reused or subtracted. Require\n   explicit equal/different Boolean answers and the same full whole-call\n\
    \   observations. Actual nominal parameter d"
- id: TC-104-procedure-7-part-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "escriptors come from the emitted\n   parameter nodes. No fabricated owner key, TypeEnvironment or copied\
    \ renderer\n   assembly is permitted."
- id: TC-104-procedure-8
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: "Measure equality descriptor result mismatch, requested unsupported\n   capability, parameter-kind mismatch,\
    \ argument-count mismatch, duplicate\n   requests, ambiguous names, duplicate declaring nodes and wrong actual\
    \ call occurrence through\n   public generation. Each retains a typed refusal and a healthy generated\n   sibling.\
    \ Measure Scalar Negate using an actual source unary body and its\n   actual call occurrence; retain the observed\
    \ unsupported disposition without\n   granting execution credit."
- id: TC-104-results-1-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '`generated_oracle_family_function_equality::primitive_and_nominal_function_equality_matches_fresh_qsl_sessions`

    passes 78 whole-call comparisons, 32 input refusals and ten changed-owner

    refusals. Each work stop and completed Boolean has an independent required

    shape, and both sides compare full typed outcome, location, located loss,

    ordered admitted charges and every counter. The original QSL constructors

    supply external nominal input Values; their separate session meters contribute

    nothing to the compared equality calls. The same test observes eight typed

    generation-bounda'
- id: TC-104-results-1-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: 'ry refusals, each retaining a healthy nominal sibling, and

    actual unary Scalar Negate''s UnsupportedOperator with sibling isolation.'
- id: TC-104-results-2-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '`generated_oracle_family_function_equality::primitive_composite_generator_matches_independent_checked_bodies`

    passes 24 FR-018 Boolean operation observations: both operators and all four

    truth pairs at work 0, 1 and 1024. Six closed nonnegative Integer literal

    candidates check and emit, but the generator retains OperandTypeMismatch at

    position 0, with the actual emitted Integer type expected and no found type.

    Their operands are inline typed Literal terms; the public source operand

    confirmation route does not admit them as reference-backed occurrences. These

    six are measure'
- id: TC-104-results-2-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: 'd nonexecuted dispositions with healthy Boolean siblings in the

    same request, never Boolean agreement credit. Integer native agreement remains

    Planned under IR-842 pending a genuine supported original-source operand route.'
- id: TC-104-result-row-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal / Boolean | Passed | Four truth pairs, three allowances; actual native
    whole-call envelope |'
- id: TC-104-result-row-2
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality NotEqual / Boolean | Passed | Four truth pairs, three allowances; actual
    native whole-call envelope |'
- id: TC-104-result-row-3
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal / Int[0,9] | Passed | Equal, less and greater examples, three allowances;
    Integer 10 source guard |'
- id: TC-104-result-row-4
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality NotEqual / Int[0,9] | Passed | Equal, less and greater examples, three allowances;
    Integer 10 source guard |'
- id: TC-104-result-row-5
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal / Pair(Boolean,Boolean) | Passed | Same and different authentic source
    tuples, three allowances |'
- id: TC-104-result-row-6
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality NotEqual / Pair(Boolean,Boolean) | Passed | Same and different authentic
    source tuples, three allowances |'
- id: TC-104-result-row-7
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal / Flag{flag:Boolean} | Passed | Same and different authentic source
    records, three allowances |'
- id: TC-104-result-row-8
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality NotEqual / Flag{flag:Boolean} | Passed | Same and different authentic source
    records, three allowances |'
- id: TC-104-result-row-9
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal / Choice{First,Second} | Passed | Same and different nullary source
    union values, three allowances |'
- id: TC-104-result-row-10
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality NotEqual / Choice{First,Second} | Passed | Same and different nullary source
    union values, three allowances |'
- id: TC-104-result-row-11
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 Scalar Negate | Passed (nonexecuted) | Actual unary source body returns UnsupportedOperator;
    healthy nominal sibling generates |'
- id: TC-104-result-row-12
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality result descriptor | Passed (nonexecuted) | SignatureMismatch and healthy sibling |'
- id: TC-104-result-row-13
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality parameter descriptor | Passed (nonexecuted) | SignatureMismatch and healthy sibling
    |'
- id: TC-104-result-row-14
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality capability request | Passed (nonexecuted) | UnsupportedCapability retains requested
    capability; healthy sibling |'
- id: TC-104-result-row-15
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality argument-count request | Passed (nonexecuted) | ArityMismatch expected 2, found 1;
    healthy sibling |'
- id: TC-104-result-row-16
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality wrong call occurrence | Passed (nonexecuted) | BodyMismatch and healthy sibling |'
- id: TC-104-result-row-17
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality duplicate request | Passed (nonexecuted) | DuplicateRequest collapses identical request;
    healthy sibling retained |'
- id: TC-104-result-row-18
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality ambiguous declaration name | Passed (nonexecuted) | AmbiguousFunctionName retains
    boolEqual; healthy sibling |'
- id: TC-104-result-row-19
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 equality duplicate declaring node | Passed (nonexecuted) | DuplicateDeclaringNode retains actual
    body ID; healthy sibling |'
- id: TC-104-result-row-20
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 Equal / closed Boolean literals | Passed | Four truth pairs and three operation allowances
    |'
- id: TC-104-result-row-21
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 NotEqual / closed Boolean literals | Passed | Four truth pairs and three operation allowances
    |'
- id: TC-104-result-row-22
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 Equal / closed Integer literals | Passed (nonexecuted) | Three genuine emitted inline-literal
    cells retain OperandTypeMismatch; healthy Boolean siblings |'
- id: TC-104-result-row-23
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 NotEqual / closed Integer literals | Passed (nonexecuted) | Three genuine emitted inline-literal
    cells retain OperandTypeMismatch; healthy Boolean siblings |'
- id: TC-104-result-row-24
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 parameterized Integer operations | Planned | IR-842: original-source local bindings are unavailable
    to public evaluate_closed; QSL-1736 tracks the bound operation observation need |'
- id: TC-104-result-row-25
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 constructed nominal/collection operations | Planned | IR-842: genuine constructors add child
    charges; need fresh bound-expression evaluation or per-operation full outcome/loss/charge/counter observation,
    without subtraction |'
- id: TC-104-result-row-26
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 options, recursive leaves, conversion-target combinations and remaining refusal boundaries
    | Planned | IR-842: each original-source admission, generator descriptor and complete public operation observation
    remains to be authored; no blanket constructor claim applies to contextual absent literals |'
- id: TC-104-result-row-27
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-021 deeper nominal shapes, union payloads and remaining per-form boundaries | Planned | IR-842:
    author additional actual source declarations/constructors, derive admitted nominal inputs and execute their
    public whole-call cohorts; no missing QSL operation ledger blocks these sessions |'
- id: TC-104-results-4-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: 'Full FR-018, FR-021 and IT-019-SC-05 remain Planned while their owner-derived

    inventories contain unexecuted rows. Typed nonexecuted generation dispositions

    remain distinct from native two-sided agreement. Existing TC-102 and TC-103

    retain only their own operation and boundary evidence.

    '
- id: IT-019-SC-05-part-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "   IT-019-SC-05: every operation selected by this owner-derived rule has an\n   explicit disposition.\
    \ Runnable cases compare the full exposed envelope\n   against independent QSL authority. FR-014 and FR-018\
    \ operation qualification\n   may use fresh, independently acquired public checked-expression evaluation\n \
    \  of the same admitted source occurrence, operands, domain and type. Compare\n   every public operation outcome,\
    \ loss, ordered admitted charge and consumed\n   counter without subtraction or precharging. This supplies operation\
    \ evidence,\n   distinct from FR-021's FunctionC"
- id: IT-019-SC-05-part-2
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: "all-level FR-485 session envelope; it supplies\n   no callable owner or session-location evidence. A\
    \ missing complete public\n   observation leaves that row Planned with the exact missing API path. A\n   different\
    \ source body with mismatched Integer domains is not a substitute.\n   A typed-refused or unsupported selection\n\
    \   is never converted to a Boolean QSL outcome, and a passing scalar sample\n   cannot mark the composite or\
    \ function family complete."
- id: IT-019-SC-05-evidence-line-219
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-018 Equal and NotEqual / closed Boolean literals | Passed | TC-104 / `generated_oracle_family_function_equality::primitive_composite_generator_matches_independent_checked_bodies`:
    24 complete native operation observations against independently checked original bodies, with independent Boolean
    truths and exact stops |'
- id: IT-019-SC-05-evidence-line-220
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-018 Equal and NotEqual / closed Integer literals | Passed (nonexecuted) | The same test retains
    six actual emitted inline-literal OperandTypeMismatch dispositions with genuine expected Integer type and found
    None, alongside healthy generated/executed Boolean siblings; no Integer agreement credit |'
- id: IT-019-SC-05-evidence-line-221
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-018 remaining shapes, conversions and operation boundaries | Planned | IR-842: inline Integer operation
    agreement needs a genuine supported operand occurrence route; parameterized operations need public original-source
    binding evaluation (QSL-1736 for Integer). Constructed shapes need an independent operation meter or complete
    per-operation outcome/loss/charges/counters; constructor charges cannot be subtracted. Other source shapes/descriptors
    and refusal controls require their own authored native cohorts |'
- id: IT-019-SC-05-evidence-line-222
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-021 Subtract, Multiply and genuine subtraction-callee Call cohort | Passed | TC-103 / `generated_oracle_family_function::subtraction_multiplication_and_nested_call_match_fresh_qsl_sessions`:
    nine whole-call envelopes, twelve argument refusals and three alternate-owner refusals; exact results, locations,
    located losses, charges and every counter |'
- id: IT-019-SC-05-evidence-line-223
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-021 CompositeEquality Equal and NotEqual / Boolean, Int[0,9], Boolean Pair tuple, Boolean Flag
    record and nullary Choice union | Passed | TC-104 / `generated_oracle_family_function_equality::primitive_and_nominal_function_equality_matches_fresh_qsl_sessions`:
    78 whole-call envelopes, 32 input refusals and ten changed-owner refusals; authentic source-constructor nominal
    inputs, independent Boolean answers, exact locations/losses/charges and every counter |'
- id: IT-019-SC-05-evidence-line-224
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-021 Scalar Negate and equality result/parameter/capability/arity/occurrence/duplicate/ambiguity/declaring-node
    requests | Passed (nonexecuted) | TC-104: actual unary source UnsupportedOperator plus eight typed generation-boundary
    controls, each retaining a healthy nominal generated sibling; no execution credit for refused items |'
- id: IT-019-SC-05-evidence-line-225
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-021 remaining shapes and per-form boundaries | Planned | IR-842: deeper nominal closures, nonempty
    union payloads and remaining per-form operating controls need their own genuine source constructors, admitted
    nominal arguments and executed whole-call evidence. FR-485 supports this route independently of the FR-018 operation-ledger
    gap; passed cohorts supply no whole-family credit |'
- id: tests/generated_oracle_family_function.rs
  path: tests/generated_oracle_family_function.rs
  role: examined
  excerpt: //! Selected FR-021 operation rows; other SC-05 operations remain unqualified.
- id: tests/generated_oracle_family_function_equality.rs
  path: tests/generated_oracle_family_function_equality.rs
  role: examined
  excerpt: //! Selected FR-021 operation rows; other SC-05 operations remain unqualified.
- id: tests/generated_oracle_family_scalar_operations.rs
  path: tests/generated_oracle_family_scalar_operations.rs
  role: examined
  excerpt: //! Selected scalar operation rows, independently checked and executed natively.
- id: tests/support/family_checked_source.rs
  path: tests/support/family_checked_source.rs
  role: examined
  excerpt: //! Independently check original operation source through public QSL APIs.
- id: tests/support/family_function_envelope.rs
  path: tests/support/family_function_envelope.rs
  role: examined
  excerpt: //! Exact observed whole-call envelope comparison shared by function cohorts.
- id: tests/support/family_function_equality_observation.rs
  path: tests/support/family_function_equality_observation.rs
  role: examined
  excerpt: //! Native caller and independent FR-485 observation, with no local body factory.
- id: tests/support/family_function_equality_source.rs
  path: tests/support/family_function_equality_source.rs
  role: examined
  excerpt: //! Original equality source and public acquisition shared by host and native driver.
- id: tests/support/family_function_observation.rs
  path: tests/support/family_function_observation.rs
  role: examined
  excerpt: //! Native caller and independent FR-485 observation, with no local body factory.
- id: tests/support/generated_oracle.rs
  path: tests/support/generated_oracle.rs
  role: examined
  excerpt: //! Native-only execution of actual CG artifacts for IT-019.
bindings:
- test_id: primitive_and_nominal_function_equality_matches_fresh_qsl_sessions
  ac_id: IT-019-SC-05
  trace: correct
- test_id: primitive_composite_generator_matches_independent_checked_bodies
  ac_id: IT-019-SC-05
  trace: correct
```

## Protocol-slot addendum

PASS for the current reviewed subset, including the protocol-slot addendum; no new findings. The initial PASS above remains a historical review of the original cohort. This addendum reviews all four changed files in the extension, including all TC-105 procedure and expected-result units and the changed TC-104 and IT-019 clauses. Current total scope is thirteen changed files. App version remains 0.1.0.

The actual model-backed source contains a Boolean Pair constructor and an original finite-global protocol finish body that compares one outcome binder with itself. Equal and NotEqual each run at work 0, 1 and 1024, giving six complete FR-018 operation comparisons. This is reflexive equality evidence, not distinct-tuple or general nominal-family agreement. Other shapes and operating boundaries remain Planned under IR-842.

Manual tracing confirms the strict admitted V2 binary node is selected from the original source, the generated contract is source-authenticated, and the public composite generator returns the symbols actually invoked. The emitted protocol-location correspondence selects the same application and agrees with the checked finish root origin and empty path. The original checked finish slot node, slot_names and CheckedSlots binder indices determine the outcome frame; the unbound model input has no readers. Public evaluate_protocol_slot runs that original body directly without a FunctionCall prefix or reconstructed stage wrapper.

The host independently checks the same source and obtains its nominal Value through its original closed constructor outside the compared meters. The native caller reacquires the authentic source, selects the real constructor declaration, resolves its owner key through wire_graph().key, obtains the public callable signature and nominal_declaration metadata, and uses the public tuple constructor with actual TuplePosition metadata. Complete input Value debug snapshots, including nominal declaration identity and both Boolean fields, must agree before operation observations. Host binder admission is asserted. No local TypeEnvironment, fabricated declaration key, copied fixture, new dependency, precharge or meter subtraction is added. The owner model document is read by reference through the existing Cargo-resolved fixture locator, not copied into the repository.

The authority independently requires true for Equal and false for NotEqual, no successful location or loss, and exact WorkUnits stops at EqualityPlanForm with next charge 6, consumed zero, original finish-body location, no admitted charges and all counters zero. Mechanical snapshots compare complete typed outcomes, losses, ordered charges and all counters to the returned native oracle for every allowance. Loss emptiness is asserted before comparison. The final array equality also requires the complete native input to match the authentic source input.

The protocol route corrects the earlier broad description of missing operation binding support: QSL-1736 concerns original function-body local bindings; public original protocol-slot frames are already usable. Remaining unimplemented protocol cohorts are integration work, not a blanket missing owner API. This refinement does not change the initial function/Boolean cohort results or confer full SC05 closure.

Supplied final protocol evidence was inspected: focused Clippy and compilation succeeded; the protocol native test passed 1 test, 0 failed; formatter check completed with existing stable-channel configuration warnings; specification validation completed with module advisory warnings. Failed exploratory probes are not treated as passing evidence. This reviewer ran no builds, Cargo, native tests, Clippy, Kani, global matrix, advisor or repository census. The lead still owns the one final scoped gate. No applicable AssuranceProfile was added by this extension.

Manual scoped trace extension only; no repository-wide coverage or inventory-closure claim.

Plan completion: not assessed

### Addendum examined units

```yaml
scope:
- id: TC-105-description-1-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: 'Assess a genuine protocol finish equality body, independently of function-body

    binding limitations. The public evaluate_protocol_slot entry accepts the original

    checked slot body and its binder frame on a fresh operation meter.'
- id: TC-105-test-procedure-1-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: "1. Author an ordinary complete-profile Boolean tuple and a finite-global protocol\n   whose finish binder\
    \ has that tuple type and whose original body compares that\n   binder with itself, separately with Equal and\
    \ NotEqual. Supply an ordinary\n   model-backed role from a lock-selected owner model document without copying\n\
    \   that document into integration. Check and emit the original source without reconstructed\n   CheckedBody,\
    \ expression, owner key or TypeEnvironment."
- id: TC-105-test-procedure-2-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: "2. Select the original finish slot and actual emitted equality occurrence. Derive\n   operand descriptors\
    \ from genuine referenced nodes. Strictly admit original IR,\n   lower the actual occurrence and authenticate\
    \ its contract, then invoke public\n   composite equality generation."
- id: TC-105-test-procedure-3-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: "3. Independently check the same original source for authority. Obtain actual\n   host nominal Values\
    \ from its original source constructor outside compared\n   meters. In the native caller use public checked-closure\
    \ nominal declaration\n   metadata, the genuine constructor signature and original wire-graph key lookup\n \
    \  with the public tuple constructor. Do not assemble a TypeEnvironment or\n   invent a declaration key. Derive\
    \ the frame's outcome index and declared type\n   from original CheckedSlots binders and slot_names. Confirm\
    \ the unbound model\n   input has no readers. Requi"
- id: TC-105-test-procedure-3-2
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: "re the emitted protocol-location correspondence\n   to retain the original checked slot's root location\
    \ and actual application. Invoke the original checked slot\n   with its actual binder frame and the returned\
    \ native oracle with identical\n   fresh limits. Compare full typed outcome, losses, ordered charges and every\n\
    \   counter at work 0, 1 and generous allowance; require independent Boolean and\n   stop expectations. Do not\
    \ subtract constructor charges or wrap a function body\n   as a different stage body."
- id: TC-105-test-procedure-4-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: "4. If a real boundary refuses the candidate, preserve its exact measured stage\n   and disposition as\
    \ Planned. Do not attribute that boundary to all protocol\n   slots, all nominal equality or the function-body\
    \ binding issue. Leave no\n   ignored test or fabricated fallback route."
- id: TC-105-expected-results-1-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: '`generated_oracle_family_protocol_equality::original_protocol_finish_equality_retains_its_source_occurrence`

    passes six actual FR-018 native operation comparisons: Equal and NotEqual of

    one original Boolean Pair finish binder with itself, at work 0, 1 and 1024.

    Independent required answers are true and false. At work 0 and 1 the exact

    WorkUnits stop consumes zero and requires EqualityPlanForm charge 6; every

    counter and admitted charge is zero and the authority retains its original

    finish-body location. Successful outcomes have no location or loss. Full typed

    outcomes, loss'
- id: TC-105-expected-results-1-2
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: es, ordered charges and all counters match the generated oracle.
- id: TC-105-expected-results-2-1
  path: spec/test-cases/TC-105-generated-protocol-equality-agreement.md
  role: examined
  excerpt: 'This establishes a public original protocol-stage frame route for this nominal

    operation without a FunctionCall envelope or charge subtraction. It does not

    establish general original function-body binding support. Record, union,

    distinct tuple values, recursive leaves, conversions and other original stage

    cohorts remain Planned under IR-842 pending their authored complete observations;

    there is no blanket owner API blocker inferred from these unimplemented rows.

    Full SC-05 remains Planned.'
- id: TC-protocol-addendum-line-134-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 parameterized Integer operations | Planned | IR-842: original function-body local bindings
    are unavailable to public evaluate_closed; QSL-1736 tracks that function-stage need. Original protocol-slot
    frames have a separate public entry, exercised by TC-105 |'
- id: TC-protocol-addendum-line-135-1
  path: spec/test-cases/TC-104-generated-function-equality-agreement.md
  role: examined
  excerpt: '| FR-018 constructed nominal/collection operations | Planned | IR-842: direct closed function-body constructors
    add child charges; remaining rows need their own matching original protocol-stage or other public operation
    cohort. TC-105 establishes one genuine tuple protocol-slot route without subtraction |'
- id: IT-protocol-addendum-line-221-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-018 Equal and NotEqual / original protocol finish Boolean Pair binder with itself | Passed | TC-105
    / `generated_oracle_family_protocol_equality::original_protocol_finish_equality_retains_its_source_occurrence`:
    six complete native operation comparisons against independently checked original protocol-slot bodies and authentic
    binder frames, including exact original location correspondence and work stops; genuine owner metadata constructs
    native inputs without a TypeEnvironment |'
- id: IT-protocol-addendum-line-222-1
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '| FR-018 remaining shapes, conversions and operation boundaries | Planned | IR-842: inline Integer operation
    agreement needs a genuine supported operand occurrence route. Original function-body local bindings need their
    own public entry (QSL-1736 for Integer); public original protocol-slot frames already exist and TC-105 exercises
    one nominal cohort. Remaining record/union/collection, distinct tuple inputs, recursive/conversion and refusal
    rows need their own authored matching stage/descriptor/native observations. Direct closed constructor-body charges
    cannot be subtracted '
- id: IT-protocol-addendum-line-222-2
  path: spec/integration/IT-019-generated-oracle-qsl-agreement.md
  role: examined
  excerpt: '|'
- id: tests/generated_oracle_family_protocol_equality.rs
  path: tests/generated_oracle_family_protocol_equality.rs
  role: examined
  excerpt: //! Original protocol-stage equality candidate, without function-body reconstruction.
bindings:
- test_id: original_protocol_finish_equality_retains_its_source_occurrence
  ac_id: IT-019-SC-05
  trace: correct
```
