---
id: TC-100
title: "Construct a real produced monitor and reject adverse intake"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-015
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: verifies
---
# TC-100: Construct a real produced monitor and reject adverse intake

## Description

Qualify the IR-721 AC3 construction handoff from genuine QSL production through
strict IR admission and the driver to RT. This focused contribution covers
FR-276-AC-6 construction and the changed-target and unsupported-instruction
parts of FR-276-AC-7. Other AC-7 mutations and event/watermark execution remain
outside this contribution; a producer preflight alone supplies no construction
or admission-refusal credit.

## Test Procedure

1. Read the QSL-owned ConfigVersion model unchanged by reference. Author a
   bounded timed clause and explicit target, and invoke public
   `qsl_analyze::monitor_root::emit_plan_root`. Strictly admit the exact final
   emission with its original evidence and model document. Preserve the plan,
   root and four entry content keys, authentic occurrence and producer
   witnesses. No synthetic plan or executable arena supplies the positive case.
2. Generate the same entry source with the public `monitor_entries` generator,
   check it through QSL and emit with `emit_monitor_plan_over`. Require that
   the unmodified route reproduces the original final emission exactly.
   In a separate source, change only the generated initializer's Integer
   zero expression to `0 mod 1`. Check and emit that source through the same
   public producer stages. Require new genuine root and initializer content
   keys, the same original clause key, retained producer witnesses and exactly
   one modulo application reachable from the selected initializer. Do not
   patch emitted IDs, copy a transition implementation, or substitute modulo
   inside an atom that the producer captures as an ordinary Boolean input.
3. Prepare the healthy admitted root using the driver's public owning entry
   and independent authentic selection, exact target and operating allowances.
   Require success before interpreting any negative control as qualifying.
   Drop source, QSL/IR and driver preparation objects while retaining only the
   RT-owned program, then call RT `build` twice with explicit operating budgets.
   Assert the prescribed initialized state and independent state ownership;
   build consumes no event and produces no verdict.
4. Keep the original healthy package, root and selection fixed. Change only
   the independently supplied target uncertainty to another valid value.
   Require the driver's exact typed target/selection mismatch and first locus,
   with no program or initializer execution. Do not regenerate a matching
   plan for this negative or accept an unrelated missing-witness failure.
5. Independently select the separately produced modulo-bearing root using its
   own genuine keys and witnesses. Require the driver's typed `Unsupported`
   preparation result at the actual reachable modulo source locus, before RT
   initialization. A mismatched original selection, IR malformed-key refusal,
   missing type witness or unsupported operation outside the selected closure
   cannot satisfy this control. The original healthy preparation and build
   must remain unaffected by the refused request.

## Expected Results

The positive and both adverse boundaries execute through the real public
producer, reader, driver and RT construction APIs and retain their exact typed
results and source loci. No callback, reimplemented temporal transition,
hand-built `ProgramParts`, unavailable-seam skip or test-only evaluator supplies
missing meaning.

Status: Planned for the complete construction and adverse intake handoff under
IR-721. The isolated `producer_rekeys_a_genuinely_reachable_unsupported_initializer`
preflight asserts producer emission and reachable mutation only. It does not
call driver preparation or RT build, and has no FR-276 criterion trace credit.
Qualification awaits the actual FR-015 production entry and its typed errors;
no speculative driver API is invoked by this preflight.
