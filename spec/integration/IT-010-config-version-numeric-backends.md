---
id: IT-010
title: "Execute ConfigVersion through numeric verification backends"
type: IT
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-032
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-033
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-034
    type: verifies
  - target: ix://agent-ix/quire-contract-codegen/FR-001
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-003
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-008
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-013
    type: references
---
# IT-010: Execute ConfigVersion through numeric verification backends

## Current Status

The V1 `lowering::lower_for` and command export inputs used by the former
composition and proof tests were removed from the locked QSL API. Both test
binaries were retired. IT-010-SC-02 has only a partial V2 scalar-claim
control in IT-013-SC-02; IT-010-SC-03 through IT-010-SC-05 have no current
test. This document retains the desired ConfigVersion integration behavior,
not a claim that the historical procedure remains executable.

## Objective

Verify the first complete numeric formal-specification path using actual checked native source,
native execution, executable IR, generated Rust oracles, model-domain proptest strategies and Kani
contracts. The test exposes any disagreement without converting invalid, unsupported or incomplete
input into a Boolean result.

## Target Integration

The public Rust path is native model/source parsing, linking, checking and packaging →
`lowering::lower_for` with `integer-ir/v1` or `state-scalar-ir/v1` → the contract IR reader →
quire-contract-codegen's public generation APIs → compiled generated Rust, proptest and
`cargo kani`. `runtime::execute` supplies the native verdict for the same immutable input.

## Preconditions

QSL, contract IR and quire-contract-codegen each resolve to one copy in this crate's dependency
graph, so QSL's lowering and codegen's generation share one set of IR types. Generated code builds
against the `quire-contract-runtime` that codegen's generated manifests name. Rust, proptest and
`cargo kani` are available locally. Missing prerequisites fail the test. No hosted workflow is
dispatched.

## Inputs

Use QSL's ConfigVersion model (read from the resolved `quire-spec-language` checkout) with this
repository's own inputs: a `VersionUnchanged`-shaped native postcondition,
`self.versionNumber = pre(self.versionNumber)` on `attemptUpdate`, over
`versionNumber: 0..1000`, run against a two-object root/child version history; plus a plain
obligation-free integer comparison over the same domain.
The shared deterministic corpus contains `-1`, `0`, `1000` and `1001`; `-1` and `1001` are
deliberately outside the model domain. In-domain state pairs include unchanged and changed values.
The violating Kani subject returns `1000 - version`, an in-domain value that differs from its input
everywhere except 500, so its counterexample can be replayed as a valid native invocation. Because VersionUnchanged has no
authored precondition, its Kani adapter uses an explicitly identified, zero-dependency Boolean true
precondition; this is the exact logical identity for absence of a precondition, not an authored
source clause or an approximation of unsupported semantics.

## Test Procedure

The following was the V1 procedure. Its scenario statements remain unmet on
the current cohort until replacement tests exercise equivalent V2 inputs.

1. Compile the ConfigVersion postcondition and plain-integer native programs, lower them through their
   explicit targets and pass the emitted bytes through the public strict IR reader to codegen.
   The plain-integer bytes are exported through QSL's `lower` command path.
   IT-010-SC-02: the numeric oracle, strategy bundle and Kani bundle retain the authored clause,
   requirement, model bounds, pre/post observation identities and original source spans; a
   later unsupported clause exports nothing.
2. Evaluate every shared-corpus assignment through the bounded backend adapter and
   `runtime::execute`.
   IT-010-SC-03: in-domain assignments produce equal Boolean verdicts from native execution and
   compiled generated oracles; `-1` and `1001` are classified outside the model domain by both
   paths, produce no Boolean verdict and never invoke the Boolean oracle.
3. Compile and run the generated satisfying, violating, broad and boundary strategy surfaces.
   IT-010-SC-04: every sampled value is within `0..=1000`, every tagged expectation matches the
   generated oracle and native verdict, framework discards are zero, and the accepted/rejected/
   discarded counts are reported.
4. Run codegen's Kani obligation harness for the postcondition through codegen's executor against
   the identity subject and a changed, in-domain subject (`1000 - version`), with concrete
   playback enabled.
   IT-010-SC-05: codegen classifies the identity run `Verified` and the changed run `Falsified`;
   codegen's `decode_falsification` decodes the counterexample against the obligation's own
   argument schema; codegen's `replay_counterexample` replays it through `qsl_replay::replay`
   against a QSL function mirroring the clause over the changed subject and QSL settles it
   `Reproduced`, while the same counterexample against the identity mirror is evidence failure;
   and `runtime::execute` of the authored clause over the decoded pre/post returns false.

## Expected Results

Every success criterion passes using actual public Rust APIs, emitted bytes, compiled generated
Rust and real `cargo kani` runs. Valid model-domain values agree across native execution, oracle,
proptest and Kani. Outside-domain, unsupported, incomplete and tool-unavailable outcomes remain
distinct from false. A Kani counterexample is accepted only when the exact decoded input replays
through native execution with the same verdict.

## Metadata

- Priority: P0
- Target Integration: native compiler/runtime → contract IR → codegen/proptest/cargo-kani
- Automation: Automated local Rust integration test

## Dependencies

**Upstream:** QSL FR-032, FR-033 and FR-034; quire-contract-codegen's numeric oracle, strategy and
Kani generation.

## Notes

The Boolean-oracle result type cannot represent invalid arithmetic or an out-of-domain model value.
This test therefore admits only obligation-free comparisons, performs domain admission before a
Boolean oracle call and requires explicit refusal elsewhere. It does not publish a crate or
dispatch hosted CI.

## Traceability

IT-010 verifies the state-scalar runtime workflow over the ConfigVersion model and both
numeric/state lowering requirements
against the separately owned codegen contracts. The two former Rust test
binaries were retired when their V1 input API disappeared. None of the
`IT-010-SC-NN` scenarios currently has a complete test; IT-013-SC-02 tests
only the narrower V2 integer claim.
