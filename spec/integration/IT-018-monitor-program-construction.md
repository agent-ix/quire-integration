---
id: IT-018
title: "Retain monitor construction parts with producer-origin identities"
type: IT
relationships:
  - target: ix://agent-ix/quire-contract-runtime/interface_002
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_006
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_005
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_004
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_003
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/FR-012
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_007
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_008
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_009
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_010
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_012
    type: verifies
  - target: ix://agent-ix/quire-contract-runtime/interface_011
    type: verifies
---
# IT-018: Retain monitor construction parts with producer-origin identities

## Objective

Exercise the RT monitor construction boundary while preserving the producer ownership
of semantic identity. These are controlled executable-part construction and ordinary
initializer tests. They do not qualify a temporal plan producer, temporal entry
semantics, or the complete FR-276 monitor.

## Target Integration

Live QSL source emission → IR strict reader → retained structural identity → shared
value/type declarations → RT `monitor::Program::try_new` → bounded ordinary initializer.

## Preconditions

Run the emitter and reader selected by Cargo.lock. Missing tools, refused source,
refused package, missing declarations, and incompatible dependencies fail the target.
No checked-package bytes or derived identity values are copied as fixtures. RT retains
its dependency boundary and does not import QSL or IR.

## Inputs

Author a source containing the declaration identities needed by the construction
controls. Obtain every key from its uniquely selected admitted declaration. Construct
controlled ordinary expression arenas and shared nominal values, then vary references,
types, ownership, source locations and exact premises. These controlled arenas are not
represented as the temporal program emitted by QSL. Their ordinary diagnostic records
use `born` and `amount` fields and do not instantiate FR-487 activation/result records.
A genuinely admitted derived sequence key supplies the nested-key storage probe; its
placement in the controlled layout grants no temporal-root admission or execution credit.

## Test Procedure

1. Drop construction preparation after successful retention. IT-018-SC-01: all four
   entry associations, parameter/result types, expression locations, shared declarations,
   initial values and independently selected metadata remain available through the owner.
2. Vary exact premises. IT-018-SC-02: width 64, nonnegative activation capacity including
   zero and values above host width, and admitted affine/negative-scale Time units retain
   their values; invalid widths, out-of-width gaps, negative uncertainty/capacity and a
   foreign dimension refuse construction. Retain the required final Integer
   event_position with initial literal value 0 under QSpec #595, including after
   two ordered buffers. Missing, Boolean/text, optional, extra or misordered State
   fields, noncanonical buffer names/types and buffer-count mismatches refuse at
   State before later entry-signature checks. A complete otherwise valid closure
   without event_position also refuses; no compatibility branch supplies it.
   These are construction observations, not event-count execution acceptance.
3. Vary executable edges and argument ownership. IT-018-SC-03: duplicate keys/locations,
   unknown addresses, cycles, foreign arguments and missing or mistyped fourth entries
   return the expected typed refusal before execution. Shared DAGs and distinct inline
   locations remain valid; a 20,000-level inline DAG validates on a 512 KiB native stack.
4. Share compatible entry functions. IT-018-SC-04: equivalent event/watermark signatures
   may retain one common function address without inventing another semantic key.

5. Execute ordinary diagnostic initializers. IT-018-SC-05: two builds retain the
   same immutable owner while owning distinct state replacement slots. Zero work,
   stack or operand limits stop with the matching resource. An impossible scratch
   reservation preserves state, owner and the old budget; successful replacement
   changes only operating limits. A lazy branch does not invoke its unselected
   recursive call. Nested helpers preserve argument order and owning scopes;
   selected recursion stops at the explicit continuation ceiling. Native argument
   admission preserves its original work/allocation cause and resolvable path,
   including combined accounting of live operand slots and native reservations.
   A 2,000-level lazy chain executes on a 512 KiB native stack,
   while a smaller explicit continuation budget stops it. A Boolean initializer
   body returns a typed invariant with its native invalid-membership cause and
   original source key retained. These controls cover the initial executor interface only;
   they do not qualify temporal on_event, on_watermark or on_gap semantics.

6. Execute lexical diagnostics. IT-018-SC-06: compact function-qualified slots
   support scoped reads, exact continuation counts and lazy branches. Missing,
   duplicate, sparse and foreign declarations refuse. A recursive call restores
   its caller's binding and cannot capture an inactive binding from that caller.
   A mismatched local annotation preserves native membership failure. A deep
   chain on a small native stack charges every active binding operand. A tight
   recursive-shadow budget stops at four slots and succeeds at five, retaining
   the caller's operand while the recursive slot is active. These
   diagnostic arenas do not qualify temporal entries or the real monitor producer.

7. Execute ordinary Boolean diagnostics. IT-018-SC-07: both unary rows and all
   eight binary truth-table rows yield their independent expected native Boolean.
   False And and true Or skip a recursive right operand; the opposite left values
   reach its explicit stack stop. Non-Boolean operands/results refuse at build,
   and selected operating stops retain the resource channel. These tests do not
   qualify temporal operators or a monitor's temporal outcomes.

8. Execute canonical tuple diagnostics. IT-018-SC-08: declared positions select
   distinct State values, and temporary owner references are released before
   build returns. Record receivers, unknown positions and wrong result types
   refuse. Tight scratch budgets charge retained projection addresses; a deep
   chain executes on a small native stack and releases every temporary tuple
   owner without copying a projected subtree. A separate native conformance
   diagnostic uses QSL/IR declaration keys and shapes with explicitly test-owned
   supertype edges. It exercises dynamic positional meaning and bounded scalar
   slot admission, including its native work refusal. This does not assert that
   QSL emitted those edges. No temporal outcome is qualified.

9. Execute required record diagnostics. IT-018-SC-09: original node-owned member
   identities select distinct State fields despite their declaration order.
   Temporary canonical owners drop, foreign owners and missing members refuse,
   and tuple receivers or wrong result types cannot become record reads. Tight
   scratch budgets stop before returning a state. A conforming dynamic record
   diagnostic reverses physical field order and checks the exact per-field scan
   work, using an explicitly test-owned native edge. These controlled parts do not
   qualify the real temporal producer.

10. Execute exact scalar diagnostics. IT-018-SC-10: independent decimal integer
    expectations beyond host width discriminate addition, subtraction,
    multiplication and negation; all four order relations have explicit
    three-way truth rows. Native result-domain refusal retains its typed cause.
    Required ScalarLimits apply cumulatively, inactive branches do not charge,
    and operating stops retain exact Integer requests and native source evidence.
    Canonical computed values consume scratch. These initializer controls do not
    qualify same-monitor reuse after event dispatch or temporal outcomes.

11. Execute canonical construction diagnostics. IT-018-SC-11: required record
    members run in declaration order, tuples run in position order and shared
    child owners remain canonical. Malformed member sets and result shapes
    refuse. Earlier native or conformance stops prevent later recursive members
    from running; exact scratch units and native retention causes are retained.
    A deep chain uses explicit continuations on a small native stack. These
    initializer controls do not qualify temporal entries or same-monitor replay.

12. Execute sequence and fold diagnostics. IT-018-SC-12: construction, size and
    outer/inner flatten order preserve duplicates; noncommutative folds, empty
    identities, lexical captures and recursive slot restoration have independent
    expectations. Returned elements retain their canonical owners. Wrong kinds
    and lexical addresses refuse, authored bounds remain native bounds, and
    native visits, formation and scratch exhaustion expose no partial value.

13. Execute verified union and checked-equality diagnostics. IT-018-SC-13:
    original sealed member identities and payloads survive temporary-owner drop;
    foreign/unsealed/ill-shaped members refuse and inactive payloads never become
    defaults. Earlier payload stops prevent later native work. Equality uses its
    original native conversion, unit and enum-index metadata after preparation
    drops. Exact native shortages remain operating failures. These controls do
    not qualify temporal entries or same-monitor stopped-input replay.

14. Execute atomic entry-dispatch diagnostics. IT-018-SC-14: independently attached
    associations and raw widths refuse before execution; poll and watermark leave
    the event baseline unchanged; modular wrap has an independent delta oracle.
    A recursive lexical resource stop preserves state and the unconsumed baseline,
    and the same machine and budget successfully process a subsequent input. These
    controls include native scalar exhaustion, stopped fold re-entry and stopped
    watermark progress rollback. Every attached context component has an independent
    negative control; original Results and distinct program associations survive
    preparation and monitor drop. These controlled bodies do not qualify
    real-producer temporal semantics.

## Expected Results

The complete target passes every construction control against the lock-selected RT
candidate. Each control retains its independent positive or refusal oracle. No passing
construction test is reported as real-producer temporal execution or full FR-276 credit.
