---
id: SR-008
title: "IT-012 model-owner admission gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-integration@1915d6c4e612040c15b78ff871b730e5ae77869a; spec/integration/IT-012-model-owner-admission.md, tests/qsl_model_owner_admission.rs"
review_set: subset
---
# IT-012 model-owner admission gap analysis

## Summary

Ticket: QSL-395 (A3k). PR quire-integration#14. Manual check of IT-012's criteria against tests. The crate is test-only, so there is no untraced production code.

- IT-012-SC-01 (admitted, lock names the selection, wrong-digest document refused): backed by `a_model_bearing_package_emitted_by_qsl_is_admitted_by_irs_reader` and `a_domain_package_supplied_under_another_digest_is_refused`.
- IT-012-SC-02 (model node ids equal ids computed from the content-only owner preimage): backed by `model_nodes_are_owned_by_the_content_only_model_owner`. The oracle is computed independently.
- IT-012-SC-03 (version change: selection differs, model node ids equal): backed by `a_version_only_change_of_the_domain_package_keys_the_same_model_nodes`.

Bindings: all four tests carry the correct `#[trace]`.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Every criterion is backed and every binding is correct. This holds only once SR-007 FND-001 is fixed. Until then, SC-01 and SC-03 are proven against a lock shape that QSL-395 removed.
