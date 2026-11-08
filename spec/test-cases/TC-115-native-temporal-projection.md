---
id: TC-115
title: "Native temporal projection, correspondence and join through genuine owners"
type: TC
relationships:
  - target: ix://agent-ix/quire-integration/IT-022
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
---
# TC-115: Native temporal projection, correspondence and join through genuine owners

## Description

Execute every IT-022 scenario, which together realize Contract IR TC-461, with
owner-produced inputs only: QSL-emitted packages, opaque `EvaluatedEvidence` carriers issued by
`EvidencePackage::evaluate_temporal_evidence`,
quire-mltl evaluated and mapped results, and QObs owner views. Expected
correspondence and join bytes are computed by the test from the inputs using
IT-022's current FR-526 successor encoding rules; no frozen vector, digest or copied owner document is
used.

## Test Procedure

Run these test targets. Each test carries, in its `#[trace(...)]`
attribute, the IT-022 scenario ids it implements together with that
scenario's FR-350 criterion ids:

| Test target | IT-022 trace tags | FR-350 trace tags |
| --- | --- | --- |
| `temporal_mapping_binding` | `IT-022-SC-01` | `FR-350-AC-2` |
| `temporal_projection` | `IT-022-SC-02`, `IT-022-SC-03`, `IT-022-SC-04`, `IT-022-SC-05`, `IT-022-SC-06`, `IT-022-SC-07`, `IT-022-SC-08`, `IT-022-SC-11`, `IT-022-SC-16` | `FR-350-AC-1`, `FR-350-AC-2`, `FR-350-AC-3`, `FR-350-AC-4`, `FR-350-AC-6`, `FR-350-AC-9` |
| `temporal_bridge_definition` | `IT-022-SC-09` | `FR-350-AC-3`, `FR-350-AC-11` |
| `temporal_binding_axes` | `IT-022-SC-10` | `FR-350-AC-3`, `FR-350-AC-10`, `FR-350-AC-11` |
| `temporal_join` | `IT-022-SC-11`, `IT-022-SC-12`, `IT-022-SC-13`, `IT-022-SC-15` | `FR-350-AC-5`, `FR-350-AC-6`, `FR-350-AC-10` |
| `temporal_limits` | `IT-022-SC-14` | `FR-350-AC-8` |
| `temporal_owner_agreement` | Direct `TC-115` assertions | `FR-350-AC-1`, `FR-350-AC-5`, `FR-350-AC-6` |

Assert complete typed owner values and typed refusals, never only an error
string or a trace tag. Run the IT-020 formula and IT-021 observation revision
regressions. The direct `TC-115` tests also carry one exact FR-350 criterion
per trace attribute. They project two distinct true/false atom-map values through
conjunction and disjunction, evaluate each owner separately, and strictly read
the joined Boolean with its original immutable native carrier and TL owner bytes. Independently authored formulas
that both evaluate true must still refuse a foreign native or TL result; equal
Boolean values alone never authorize a join.

## Expected Results

Every IT-022 scenario passes. Each FR-350 criterion FR-350-AC-1 to AC-6 and
AC-8 to AC-11 is asserted by at least one tagged test. Every refusal leaves no
partial projection or join, each limit admits at its measured count and
refuses one below it, and no bridge operation invokes an evaluator.

## Unreachable through genuine owners

- R4 (`unsupported`) and any pair containing it: quire-mltl reports
  `Unsupported` only for formula and lane mismatches (`src/report.rs:735-760`),
  which a bridge-built request for a Supported subject never yields, and a
  result for any other request is `refused` (R1) first.
- A TL correction paired with an original native result over the same
  projection: the native owner refuses to evaluate an original result against a
  request that carries a predecessor relation.
- Caller-mutated native truth/support under one issued identity: the public
  read-data `EvidenceResult` can be copied, but cannot construct or replace the
  private result inside the evaluator-issued carrier required by `join_tl`.
- Multi-stamp bundles and duplicate bundle observation identities: see
  IT-022, "Checks verified at unit level".
