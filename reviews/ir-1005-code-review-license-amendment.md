---
id: SR-2026110414560016
title: IR-1005 same-review license amendment
type: SpecReview
analysis: code-review
review_set: subset
scope: agent-ix/quire-integration; deny.toml; IR-1005 coordinated input-binding review
  amendment
scope_units:
- id: deny.toml:ar_archive_writer
  path: deny.toml
  role: examined
  excerpt: '{ name = "ar_archive_writer", allow = ["Apache-2.0 WITH LLVM-exception"]
    },'
- id: deny.toml:borrow-or-share
  path: deny.toml
  role: examined
  excerpt: '{ name = "borrow-or-share", allow = ["MIT-0"] },'
---

## Summary

Ticket: IR-1005. Same independent reviewer carry for quire-integration#44, coordinated with quire-contract-ir#897. This amendment examines only the six added deny.toml lines after final qualification exposed production reachability of existing transitive crates. Original review artifacts remain unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The two additional license expressions are restricted to their named owning crates. The global allow list, advisory and source policies, and confidence threshold are unchanged. No license bypass, filter or dependency pin is introduced.

## Coverage

Inspected both exception entries, their TOML placement, the unchanged package entries against main, and QSL's existing license policy permitting both expressions. Moving checked-view dependencies into production makes these existing dependencies relevant to this gate; this amendment declares their actual licenses without granting either expression to unrelated crates. The supplied make deny amendment log runs cargo deny check licenses and ends with licenses ok. The lead reports the baseline deny gate passed and the amended gate exits zero. No Cargo or heavy build was run by the reviewer.

This amendment does not change binding behavior or increase FR-350 coverage. Projection, native/TL joins and limits remain Planned. Full release qualification and the pending build remain the lead's responsibility. No AssuranceProfile applies to this license-only amendment.
