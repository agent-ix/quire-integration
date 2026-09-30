---
id: IT-008
title: "Execute native Boolean projections through the existing backend"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-009
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-029
    type: verifies
---
# IT-008: Execute native Boolean projections through the existing backend

## Objective

Expose frontend/backend truth or implication-activation drift on actual compiled
output. This bounded Boolean milestone does not close the full ConfigVersion
workflow in QSL's IT-002 (`ix://agent-ix/quire-spec-language/IT-002`).

## Target Integration

Native parse/link/check/package/lower → contract IR wire reader → quire-contract-codegen
→ generated Rust, generated proptest strategy and runtime operators. Native
validation/evaluation supplies the reference result; actual LLVM coverage
supplies generated activation for the exact admitted fixture profile.

## Preconditions

Rust with its LLVM tools and cargo-llvm-cov are available locally. Missing
prerequisites fail the test rather than silently skip it. No hosted run is
dispatched.

## Inputs

An admitted source model, authored clause bindings, and all assignments of a
small Boolean fixture containing nested implication, conjunction, disjunction
and negation. Use independently specified truth and activation expectations.

## Test Procedure

1. Produce and lower the actual checked native package, including through QSL's
   `lower` command path, which exports exactly the lowered bytes.
   IT-008-SC-01: every authored clause and native source locus survives binding.
2. Consume the exact emitted wire through the contract IR reader and invoke
   codegen's public bound generation API.
   IT-008-SC-02: complete generated clauses retain identities and source maps.
3. Validate and evaluate each native runtime assignment; compile and execute the
   generated Rust and generated proptest strategy for the same assignments.
   IT-008-SC-03: all native, independent and generated truth results agree.
4. Measure generated execution with actual coverage and resolve the LLVM 3.1.0
   export segments through the generated source-map probes. Pass the same
   measurement to codegen's bound-coverage analyzer and retain its
   `foreign_generated_file` refusal: the analyzer knows only the bound-oracle
   bundle's own files, not the strategy file measured beside them.
   IT-008-SC-04: generated consequent activation agrees with native source events,
   including skipped and entered nested consequents.

## Expected Results

Every step passes using actual producer bytes and executions. Unsupported
numeric/reference features retain explicit refusal. Generated attestation bodies
are not described as sealed or retained proof records. This fixture does not
claim that codegen's bound-coverage analyzer accepts a measured package that
also contains strategy-generated files.
