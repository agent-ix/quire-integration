// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

#![forbid(unsafe_code)]

//! TC-008..TC-011: repository-wide resource and authority static verification.

use ix_trace_rs::trace;
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

const INVENTORY: &str = include_str!("../assurance/task-007-resource-authority-inventory.md");
const CAPABILITY: &str = include_str!("../src/capability.rs");
const COMMON: &str = include_str!("../src/common.rs");
const COMPATIBILITY: &str = include_str!("../src/compatibility_impact.rs");
const CONSUMER: &str = include_str!("../src/consumer_views.rs");
const EFFECT: &str = include_str!("../src/effect_reconciliation.rs");
const EXECUTION: &str = include_str!("../src/execution_bundle.rs");
const LIB: &str = include_str!("../src/lib.rs");
const LOCK: &str = include_str!("../src/ecosystem_lock.rs");

const PRODUCTION: [(&str, &str); 8] = [
    ("capability.rs", CAPABILITY),
    ("common.rs", COMMON),
    ("compatibility_impact.rs", COMPATIBILITY),
    ("consumer_views.rs", CONSUMER),
    ("ecosystem_lock.rs", LOCK),
    ("effect_reconciliation.rs", EFFECT),
    ("execution_bundle.rs", EXECUTION),
    ("lib.rs", LIB),
];
const REVIEWED_DIGESTS: [(&str, &str); 8] = [
    (
        "capability.rs",
        "75a3a534aa5c71c7d39e488234764241938b1c68a68d619b7bdf95daa49eea6a",
    ),
    (
        "common.rs",
        "6ed9f39f7fa784a0507100cb2175cd28deddfa836b2b72b0ea4e37b209ddc7f8",
    ),
    (
        "compatibility_impact.rs",
        "42d5f340ada8d3ba73a817ef8509db90472d8d7c372a9c211c141f373b4f2c9a",
    ),
    (
        "consumer_views.rs",
        "8f10a87509d372c7ca9f7e59d1c980e1614cdbc8563fab30beb9b1e96a550d84",
    ),
    (
        "ecosystem_lock.rs",
        "0f6fd654ea9274bdc098bbab45c06a2b22dad6ff573786b274da466e0f57ee9b",
    ),
    (
        "effect_reconciliation.rs",
        "dbaaae6c6ce5e5b2e5085c762240874292529da2a6121dc85795fb1e62781f10",
    ),
    (
        "execution_bundle.rs",
        "16d269b08431932ac24e12062a89e5b8cda33440d53dd90d3b1bc30729f86939",
    ),
    (
        "lib.rs",
        "cfa0a6e7d35e3098c862ed2c3289a41b9cf5f90ac67b290fad870adb8ed194cb",
    ),
];

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let (_, tail) = source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing reviewed boundary start: {start}"));
    let (scope, _) = tail
        .split_once(end)
        .unwrap_or_else(|| panic!("missing reviewed boundary end: {end}"));
    scope
}

fn assert_ordered(scope: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = scope[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing ordered resource guard: {marker}"));
        cursor = cursor
            .checked_add(offset)
            .and_then(|value| value.checked_add(marker.len()))
            .expect("test-source cursor is representable");
    }
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn external_module(line: &str) -> Option<&str> {
    let line = line.trim();
    if !line.ends_with(';') {
        return None;
    }
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let module_index = tokens.iter().position(|token| *token == "mod")?;
    (module_index + 2 == tokens.len()).then(|| tokens[module_index + 1].trim_end_matches(';'))
}

#[test]
#[trace("TC-010", "NFR-001-AC-1")]
fn tc010_inventory_covers_every_production_module_and_guarded_boundary() {
    for id in 1..=32 {
        assert!(
            INVENTORY.contains(&format!("RB-{id:02}")),
            "resource boundary RB-{id:02} must remain inventoried"
        );
    }
    for (name, _) in PRODUCTION {
        assert!(
            INVENTORY.contains(name.trim_end_matches(".rs")),
            "{name} must remain represented in the static inventory"
        );
    }
    let declared_modules = LIB
        .lines()
        .filter_map(external_module)
        .collect::<BTreeSet<_>>();
    let inventoried_modules = PRODUCTION
        .iter()
        .filter_map(|(name, _)| name.strip_suffix(".rs"))
        .filter(|name| *name != "lib")
        .collect::<BTreeSet<_>>();
    assert_eq!(
        declared_modules, inventoried_modules,
        "every module declared by lib.rs must be scanned and inventoried"
    );
    for (name, source) in PRODUCTION {
        if name != "lib.rs" {
            assert!(
                !source.lines().any(|line| external_module(line).is_some()),
                "{name} introduced an unscanned external nested module"
            );
        }
        assert!(
            !source.contains("include!("),
            "{name} introduced an unscanned source include"
        );
    }
    let production_names = PRODUCTION
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    let reviewed_names = REVIEWED_DIGESTS
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        PRODUCTION.len(),
        production_names.len(),
        "production inventory names must be unique"
    );
    assert_eq!(
        REVIEWED_DIGESTS.len(),
        reviewed_names.len(),
        "review digest names must be unique"
    );
    assert_eq!(
        production_names, reviewed_names,
        "every scanned production source needs exactly one review digest"
    );
    for (name, expected) in REVIEWED_DIGESTS {
        let source = PRODUCTION
            .iter()
            .find_map(|(candidate, source)| (*candidate == name).then_some(*source))
            .expect("reviewed source is in the production inventory");
        let actual = format!("{:x}", Sha256::digest(source.as_bytes()));
        assert_eq!(
            actual, expected,
            "{name} changed after its complete resource/authority audit"
        );
        assert!(
            INVENTORY.contains(expected),
            "{name} review digest must be published in the inventory"
        );
    }

    assert_ordered(
        between(COMMON, "fn canonical_bytes", "fn checked_length_sum"),
        &[
            "canonical_len()?",
            "length > max_output_bytes",
            "usize::try_from(length)",
            "Vec::with_capacity",
        ],
    );
    assert_ordered(
        between(COMMON, "pub fn new(value: impl AsRef<str>", "pub fn as_str"),
        &[
            "u64::try_from(value.len())",
            "length > max_identity_bytes",
            "value.trim().is_empty()",
            "value.to_owned()",
        ],
    );
    assert_ordered(
        between(COMMON, "pub fn try_from_entries", "pub fn len"),
        &[
            "u64::try_from(entries.len())",
            "count > max_items",
            "BTreeMap::new()",
        ],
    );
    assert_ordered(
        between(COMMON, "pub fn try_from_values", "pub fn len"),
        &[
            "u64::try_from(values.len())",
            "count > max_items",
            "BTreeSet::new()",
        ],
    );
    assert_ordered(
        between(LOCK, "fn validate_with_timer", "pub fn validate_then"),
        &[
            "preflight(selection, limits, timer)?",
            "preflight_output(selection, limits, timer)?",
            "validate_repositories",
            "serialize_wire",
        ],
    );
    assert_ordered(
        between(
            LOCK,
            "pub fn read_with_monotonic_clock",
            "fn preflight(selection",
        ),
        &[
            "count(bytes.len())?",
            "limits.max_document_bytes",
            "validate_with_timer(selection, limits, timer)?",
        ],
    );
    assert_ordered(
        between(LOCK, "fn serialize_wire", "fn selection_input_bytes"),
        &[
            "usize::try_from(exact_length)",
            "Vec::with_capacity(capacity)",
            "writer.bytes.len() != capacity",
        ],
    );
    assert_ordered(
        between(EXECUTION, "impl<'a> Coordinator<'a>", "pub fn execute"),
        &["check_count", "validate_request_limits", "Ok(Self"],
    );
    assert_ordered(
        between(EXECUTION, "pub fn execute", "pub fn mark_unavailable"),
        &[
            "max_artifact_bytes",
            "consume_input_bytes",
            "reader.read",
            "invoke_producers",
            "producer.evaluate",
        ],
    );
    assert_ordered(
        between(EXECUTION, "pub fn read(bytes", "fn parse_request"),
        &[
            "limits.max_output_bytes",
            "preflight_document(bytes, limits)?",
            "serde_json::from_slice",
            "total > limits.max_items",
            "usize::try_from(total)",
            "Vec::with_capacity(capacity)",
        ],
    );
    assert_ordered(
        between(EFFECT, "pub fn coordinate<", "pub fn observe<"),
        &["self.validate(", "store", "reserve_or_load", "self.resume"],
    );
    assert_ordered(
        between(EFFECT, "fn prepare_and_dispatch<", "fn invoke_adapter<"),
        &[
            "required_history",
            "max_history_events",
            "max_attempts",
            "max_identity_bytes",
            "prepare_attempt",
            "State::Open",
            "self.invoke_adapter",
        ],
    );
    assert_ordered(
        between(CONSUMER, "pub fn query<R", "fn validate_owner<R"),
        &[
            "validate_owner",
            "max_reader_invocations",
            "admit_capability",
            ".read(",
            "CallBudget",
        ],
    );
    assert_ordered(
        between(CONSUMER, "pub fn submit", "fn fail_invalid"),
        &[
            "validate_local_identity",
            "next_results",
            "borrowed_output_usage",
            "next_bytes > self.limits.max_output_bytes",
            "next_mappings > self.limits.max_mappings",
            "clone_borrowed_output",
            "self.completions.push",
        ],
    );
    assert_ordered(
        between(CONSUMER, "pub fn consume<C", "fn finish_run"),
        &[
            "manifest.selection != request.selection",
            "manifest.supported_obligations.len()",
            "request.obligations.len()",
            "for obligation in &request.obligations",
            "validate_source",
            "let mut obligations = request.obligations.clone()",
            "let result = consumer.consume",
        ],
    );
    assert_ordered(
        between(COMPATIBILITY, "fn push_inner", "fn preflight_item"),
        &[
            "encoded_projection_item_len",
            "self.preflight_item",
            ".push(item)",
        ],
    );
    assert_ordered(
        between(COMPATIBILITY, "pub fn classify", "fn map_reader_error"),
        &[
            "self.limits.validate()?",
            "validate_reader_inputs",
            "max_reader_invocations",
            "admit_capability",
            "Vec::with_capacity(ordered_inputs.len())",
            "reader.read",
            "self.classify_bundle",
        ],
    );
    assert_ordered(
        between(
            COMPATIBILITY,
            "fn validate_reader_inputs",
            "macro_rules! take_projection",
        ),
        &[
            "count(input.document.canonical_bytes.len())?",
            "length > limits.max_document_bytes",
            "total = checked_add(total, length)?",
            "total > limits.max_total_input_bytes",
            "inputs.to_vec()",
        ],
    );

    for (name, source) in PRODUCTION {
        for unchecked in [
            " as usize",
            " as u8",
            " as u16",
            " as u32",
            " as u64",
            " as u128",
        ] {
            assert!(
                !source.contains(unchecked),
                "{name} introduced unchecked numeric conversion `{unchecked}`"
            );
        }
    }
}

#[test]
#[trace("TC-009", "FR-003-AC-6")]
fn tc009_adapter_dispatch_has_one_private_persisted_boundary() {
    let compact_sources = PRODUCTION
        .iter()
        .map(|(name, source)| (*name, without_whitespace(source)))
        .collect::<Vec<_>>();
    let call_count: usize = PRODUCTION
        .iter()
        .map(|(_, source)| without_whitespace(source).matches(".dispatch(").count())
        .sum();
    assert_eq!(
        call_count, 1,
        "production must have one adapter dispatch call"
    );
    assert_eq!(without_whitespace(EFFECT).matches(".dispatch(").count(), 1);
    for (name, source) in compact_sources {
        if name != "effect_reconciliation.rs" {
            assert!(
                !source.contains("AuthorizedEffectAdapter"),
                "{name} acquired effect-adapter capability"
            );
        }
    }

    let prepared = between(EFFECT, "fn prepare_and_dispatch<", "fn invoke_adapter<");
    assert_ordered(
        prepared,
        &["prepare_attempt", "State::Open", "self.invoke_adapter"],
    );
    let invocation = between(EFFECT, "fn invoke_adapter<", "fn persist_delivery_unknown");
    assert_ordered(
        invocation,
        &[
            "require_current_authority",
            "DispatchPermit {",
            "adapter.dispatch",
        ],
    );
    let permit = between(EFFECT, "pub struct DispatchPermit", "impl DispatchPermit");
    for field in [
        "request_id",
        "attempt_id",
        "lock_identity",
        "max_receipt_bytes",
        "remaining_duration_ms",
    ] {
        assert!(
            !permit.contains(&format!("pub {field}")),
            "permit field became forgeable: {field}"
        );
    }
    assert!(!EFFECT.contains("pub fn invoke_adapter"));
}

#[test]
#[trace("TC-008", "TC-011", "NFR-002-AC-1", "NFR-002-AC-2", "NFR-002-AC-3")]
fn tc008_tc011_pure_capabilities_have_no_ambient_or_effect_authority_edge() {
    for id in 1..=9 {
        assert!(
            INVENTORY.contains(&format!("AB-{id:02}")),
            "authority boundary AB-{id:02} must remain inventoried"
        );
    }

    for (name, source) in PRODUCTION {
        for forbidden in [
            "unsafe {",
            "std::env",
            "std::fs",
            "std::net",
            "current_dir(",
            "home_dir(",
            "Command::new",
            "reqwest",
            "tokio::",
            "OnceLock",
            "OnceCell",
            "lazy_static!",
            "thread_local!",
            "static mut",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} introduced ambient/unsafe authority token `{forbidden}`"
            );
        }
    }

    for (name, source) in [
        ("execution_bundle.rs", EXECUTION),
        ("consumer_views.rs", CONSUMER),
        ("compatibility_impact.rs", COMPATIBILITY),
    ] {
        for forbidden in ["EffectStore", "DispatchPermit", "PersistenceCapability"] {
            assert!(
                !source.contains(forbidden),
                "{name} acquired persistence/effect authority `{forbidden}`"
            );
        }
    }

    assert!(EXECUTION.contains("admission: AdmittedCapability<'a>"));
    assert!(EXECUTION.contains("limits: ExecutionLimits"));
    assert!(CONSUMER.contains("admission: &AdmittedCapability<'_>"));
    assert!(CONSUMER.contains("budget: CallBudget"));
    assert!(COMPATIBILITY.contains("admission: &AdmittedCapability<'_>"));
    assert!(COMPATIBILITY.contains("budget: ReaderBudget"));
    let compact_execution = without_whitespace(EXECUTION);
    let compact_consumer = without_whitespace(CONSUMER);
    let compact_compatibility = without_whitespace(COMPATIBILITY);
    assert_eq!(compact_execution.matches("reader.read(").count(), 1);
    assert_eq!(compact_execution.matches("producer.evaluate(").count(), 1);
    assert_eq!(compact_consumer.matches("reader.read(").count(), 1);
    assert_eq!(compact_consumer.matches("consumer.consume(").count(), 1);
    assert_eq!(compact_compatibility.matches("reader.read(").count(), 1);

    let decision_validation = between(EFFECT, "fn validate<V", "fn resume<");
    assert_ordered(
        decision_validation,
        &[
            "admit_capability(authority_verifier.decision_selection())",
            "DECISION_AUTHORITY_CONTRACT",
            "admit_capability(authority_verifier.grant_selection())",
            "GRANT_AUTHORITY_CONTRACT",
            "authority_verifier",
            ".verify(",
            "timer.call_budget()?",
        ],
    );
    let observation = between(EFFECT, "pub fn observe<", "fn validate<V");
    assert_ordered(
        observation,
        &[
            "admit_capability(authority.selection())",
            "OBSERVATION_AUTHORITY_CONTRACT",
            "authority",
            ".verify(observation, context.timer.call_budget()?)",
        ],
    );
    assert!(EFFECT.contains("pub trait DecisionAuthority"));
    assert!(EFFECT.contains("pub trait ObservationAuthority"));
    let observation_trait = between(
        EFFECT,
        "pub trait ObservationAuthority",
        "pub trait DecisionAuthority",
    );
    let decision_trait = between(EFFECT, "pub trait DecisionAuthority", "pub enum Outcome");
    assert_eq!(
        without_whitespace(observation_trait)
            .matches("budget:CallBudget")
            .count(),
        1
    );
    assert_eq!(
        without_whitespace(decision_trait)
            .matches("budget:CallBudget")
            .count(),
        1
    );

    for trait_name in [
        "ProducerCapability",
        "ConsumerCapability",
        "PersistenceCapability",
        "EvidenceIntakeCapability",
        "DecisionCapability",
    ] {
        assert!(CAPABILITY.contains(&format!("pub trait {trait_name}")));
        assert!(
            !PRODUCTION
                .iter()
                .any(|(_, source)| source.contains(&format!("{trait_name} for"))),
            "production added an unreviewed blanket authority implementation for {trait_name}"
        );
    }
    assert!(!CAPABILITY.contains("impl<"));
    assert!(!CAPABILITY.contains("From<"));
    assert!(!CAPABILITY.contains("Into<"));
    assert_eq!(
        CAPABILITY
            .matches("fn selection(&self) -> &CapabilitySelection")
            .count(),
        5
    );
    for forbidden_operation in [
        "fn evaluate",
        "fn consume",
        "fn persist",
        "fn ingest",
        "fn decide",
    ] {
        assert!(
            !CAPABILITY.contains(forbidden_operation),
            "shared capability traits must remain selection markers, not invocation authority"
        );
    }

    macro_rules! assert_not_impl {
        ($subject:ty: $authority:path) => {{
            trait AmbiguousIfImplemented<Marker> {
                fn check() {}
            }
            struct AuthorityMarker;
            impl<T: ?Sized> AmbiguousIfImplemented<()> for T {}
            impl<T: ?Sized + $authority> AmbiguousIfImplemented<AuthorityMarker> for T {}
            let _ = <$subject as AmbiguousIfImplemented<_>>::check;
        }};
    }

    macro_rules! deny_source_and_effect_authority {
        ($subject:ty) => {{
            assert_not_impl!($subject: quire_integration::ProducerCapability);
            assert_not_impl!($subject: quire_integration::ConsumerCapability);
            assert_not_impl!($subject: quire_integration::PersistenceCapability);
            assert_not_impl!($subject: quire_integration::EvidenceIntakeCapability);
            assert_not_impl!($subject: quire_integration::DecisionCapability);
            assert_not_impl!($subject: quire_integration::execution_bundle::StrictReader);
            assert_not_impl!($subject: quire_integration::execution_bundle::ExecutionProducer);
            assert_not_impl!($subject: quire_integration::consumer_views::CheckedPackageReader);
            assert_not_impl!($subject: quire_integration::compatibility_impact::StrictImpactReader);
            assert_not_impl!($subject: quire_integration::effect_reconciliation::AuthorizedEffectAdapter);
            assert_not_impl!($subject: quire_integration::effect_reconciliation::DecisionAuthority);
            assert_not_impl!($subject: quire_integration::effect_reconciliation::ObservationAuthority);
        }};
    }

    deny_source_and_effect_authority!(quire_integration::consumer_views::ConsumerRun);
    deny_source_and_effect_authority!(quire_integration::consumer_views::AccountingRecord);
    deny_source_and_effect_authority!(quire_integration::consumer_views::ConsumerRequest);
    deny_source_and_effect_authority!(quire_integration::consumer_views::ConsumerAdmission);
    deny_source_and_effect_authority!(quire_integration::compatibility_impact::EvidenceRecord);
    deny_source_and_effect_authority!(quire_integration::compatibility_impact::EvidenceProjection);
    deny_source_and_effect_authority!(quire_integration::effect_reconciliation::AuthorityGrant);
    deny_source_and_effect_authority!(quire_integration::effect_reconciliation::Decision);
    deny_source_and_effect_authority!(quire_integration::effect_reconciliation::Observation);
    deny_source_and_effect_authority!(quire_integration::effect_reconciliation::Recommendation);
    deny_source_and_effect_authority!(quire_integration::effect_reconciliation::OwnerArtifact);
}
