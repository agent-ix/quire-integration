// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350 input binding through the public QSL producer and strict IR reader.

mod support;

use ix_trace_rs::trace;
use qsl_package::checked_views::{mapping_support, temporal_subject, Closure, Support, Unmatched};
use qsl_semantics::{
    check::{CheckCause, CheckRefusal, CheckedGraph, PackageDeclarations, TemporalCause},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedNodeKind, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, ClockConfiguration,
    ClockExactNumber, TemporalForm,
};
use quire_integration::temporal::{
    construct_tl_formula, MappingBindingRefusal, TlFormulaCode, TlFormulaComponent, TlFormulaError,
    TlFormulaLimit, TlFormulaLimits,
};
use quire_semantic_value::checking::CheckingLimits;
use tl_syntax::{
    FormulaDocument, FormulaSchemaVersion, Interval, NodeId, NodeKind, PropositionId,
    SemanticProfile, SyntaxArtifactLimits,
};

/// Parse and check original source without bypassing native temporal admission.
fn checked_formula(
    formula: &str,
    profile: &str,
    document: &[u8],
) -> Result<CheckedGraph, Vec<CheckRefusal>> {
    // A formula marked `@value` ranges over an integer subject, the only subject a
    // window aggregate atom accepts.
    let source = if let Some(body) = formula.strip_prefix("@value ") {
        format!(
            r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile t = "{profile}";
temporal Due using t over (e: Int[0, 9]) clock "events" on origin {{
    {body}
}}
"#
        )
    } else {
        format!(
            r#"language "ix:native" edition "1-draft";
profile v = "quire.model.complete/v1";
profile t = "{profile}";
model M = "acme/orders";
temporal Due using t over (g: M::Gadget) clock "events" on origin {{
    {formula}
}}
"#
        )
    };
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "test:temporal-binding"),
        "temporal-binding.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the temporal source parses");
    assert!(parsed.diagnostics().is_empty());
    let raw = parsed.source().reference().clone();
    let unit = qsl_forms::build_unit(&parsed).expect("the temporal unit forms");
    let models = admit_unit(
        &unit.selections().models,
        &package_input([document]).expect("the domain document reads"),
        ModelNormalizationLimits::default(),
    )
    .expect("the selected model admits");
    PackageDeclarations::assemble(
        raw,
        "agent-ix/temporal-binding".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("the package assembles")
    .check(CheckingLimits::default())
}

/// Emit once and admit twice: the packages have equal contents and distinct owners.
fn packages(formula: &str, profile: &str) -> (CheckedPackageV2, CheckedPackageV2, CheckedNodeId) {
    let document =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("the selected QSL owner fixture exists");
    let graph = checked_formula(formula, profile, &document).expect("the temporal package checks");
    let selections: Vec<_> = graph
        .temporal_clauses()
        .iter()
        .filter(|_| !profile.contains("infinite-trace"))
        .map(|clause| {
            qsl_package::TemporalClockSelection::new(
                graph.source().clone(),
                clause.clause().span(),
                if profile.contains("fixed-sample") {
                    ClockConfiguration::FixedSample {
                        epoch: ClockExactNumber::Integer {
                            decimal: "0".into(),
                        },
                        period: ClockExactNumber::Integer {
                            decimal: "1".into(),
                        },
                        unit: "tick".into(),
                    }
                } else if profile.contains("timestamped-event") {
                    ClockConfiguration::TimestampedEvent {
                        timestamp_unit: "tick".into(),
                    }
                } else {
                    ClockConfiguration::EventPosition {
                        sequence_authority: "events".into(),
                    }
                },
            )
        })
        .collect();
    let emitted = qsl_package::emit_checked_with_temporal_selections(
        &qsl_package::CheckedPackage::link(graph),
        &selections,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("the producer emits the temporal package");
    assert!(emitted.omitted().is_empty());
    let bytes = emitted.package().bytes();
    let wire: serde_json::Value = serde_json::from_slice(bytes).expect("emitted JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature identity"));
    }
    evidence
        .insert_domain_package_document("acme/orders", document.as_slice())
        .expect("the original model document is reader evidence");
    let admit = || match read_checked_package(bytes, CheckedPackageReadLimits::bounded(), &evidence)
    {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("the strict reader admits the producer output: {other:?}"),
    };
    let first = admit();
    let second = admit();
    let clauses: Vec<_> = first
        .graph()
        .nodes
        .iter()
        .zip(first.node_kinds())
        .filter(|(_, kind)| **kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
        .map(|(node, _)| node.node_id.clone())
        .collect();
    let [clause] = clauses.as_slice() else {
        panic!("exactly one temporal clause");
    };
    (first, second, clause.clone())
}

const EVENT: &str = "quire.temporal.event-position.false-extension/v1";
const SAMPLE: &str = "quire.temporal.fixed-sample.false-extension/v1";

fn component(formula: &str, profile: &str, closure: Closure) -> TlFormulaComponent {
    let (package, _, clause) = packages(formula, profile);
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, closure).expect("classification");
    let result = construct_tl_formula(
        &package,
        &subject,
        &classification,
        closure,
        TlFormulaLimits::default(),
    )
    .expect("formula component");
    assert_eq!(result.classification(), &classification);
    let Support::Supported { premises, .. } = &result.classification().support else {
        panic!("supported component")
    };
    assert!(premises.contains(&"total Boolean predicate projection"));
    assert_eq!(
        FormulaDocument::from_json_bytes(result.bytes(), SyntaxArtifactLimits::default())
            .expect("strict owner reader"),
        *result.formula()
    );
    assert_eq!(
        result
            .formula()
            .canonical_json_bytes()
            .expect("canonical owner writer"),
        result.bytes()
    );
    result
}

#[trace("IT-020-SC-01", "IT-020-SC-02")]
#[test]
fn asymmetric_postorder_preserves_repeated_checked_leaf_occurrences() {
    let (package, _, clause) = packages(
        "(not holds(true)) implies (holds(false) or holds(true))",
        EVENT,
    );
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let result = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect("formula");
    assert_eq!(
        result
            .formula()
            .nodes()
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            NodeKind::Proposition {
                proposition: PropositionId(0)
            },
            NodeKind::Not { operand: NodeId(0) },
            NodeKind::Proposition {
                proposition: PropositionId(1)
            },
            NodeKind::Proposition {
                proposition: PropositionId(2)
            },
            NodeKind::Or {
                left: NodeId(2),
                right: NodeId(3)
            },
            NodeKind::Implies {
                left: NodeId(1),
                right: NodeId(4)
            },
        ]
    );
    assert_eq!(result.formula().root(), NodeId(5));
    assert_eq!(
        result
            .leaves()
            .iter()
            .map(|leaf| &leaf.predicate)
            .collect::<Vec<_>>(),
        subject.predicate_leaves().iter().collect::<Vec<_>>()
    );
    assert_eq!(result.leaves()[0].predicate, result.leaves()[2].predicate);
    assert_ne!(
        result.leaves()[0].proposition,
        result.leaves()[2].proposition
    );
    assert_eq!(
        result
            .leaves()
            .iter()
            .map(|leaf| leaf.node)
            .collect::<Vec<_>>(),
        vec![NodeId(0), NodeId(2), NodeId(3)]
    );
    assert_eq!(result.work(), 12);
}

#[trace("IT-020-SC-01")]
#[test]
fn every_bounded_future_operator_uses_exact_owner_rows_and_closure_profiles() {
    let interval = Interval::new(1, 3).expect("interval");
    let rows = [
        (
            "eventually[1,3] holds(true)",
            NodeKind::Future {
                interval,
                operand: NodeId(0),
            },
        ),
        (
            "always[1,3] holds(true)",
            NodeKind::Globally {
                interval,
                operand: NodeId(0),
            },
        ),
        (
            "holds(true) until[1,3] holds(false)",
            NodeKind::Until {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "holds(true) release[1,3] holds(false)",
            NodeKind::Release {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
    ];
    for profile in [EVENT, SAMPLE] {
        for (closure, expected) in [
            (Closure::Complete, SemanticProfile::ClosedTraceV1),
            (Closure::Open, SemanticProfile::OnlinePrefixV1),
        ] {
            for (source, root) in &rows {
                let result = component(source, profile, closure);
                assert_eq!(result.formula().nodes().last().expect("root").kind, *root);
                assert_eq!(result.formula().schema_version(), FormulaSchemaVersion::V1);
                assert_eq!(result.formula().semantic_profile(), expected);
                assert!(
                    matches!(result.classification().support, Support::Supported { total_sample_valuation, .. } if total_sample_valuation == (profile == SAMPLE))
                );
            }
        }
    }
    let result = component("true and false", EVENT, Closure::Complete);
    assert_eq!(
        result
            .formula()
            .nodes()
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            NodeKind::True,
            NodeKind::False,
            NodeKind::And {
                left: NodeId(0),
                right: NodeId(1)
            }
        ]
    );
    assert!(result.leaves().is_empty());
}

#[trace("IT-020-SC-01", "IT-020-SC-02")]
#[test]
fn every_bounded_past_operator_and_previous_retains_history_premises() {
    let interval = Interval::new(1, 3).expect("interval");
    let rows = [
        (
            "once[1,3] holds(true)",
            NodeKind::Once {
                interval,
                operand: NodeId(0),
            },
        ),
        (
            "historically[1,3] holds(true)",
            NodeKind::Historically {
                interval,
                operand: NodeId(0),
            },
        ),
        (
            "holds(true) since[1,3] holds(false)",
            NodeKind::Since {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "holds(true) triggered[1,3] holds(false)",
            NodeKind::Triggered {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "once[1,1] holds(true)",
            NodeKind::Once {
                interval: Interval::new(1, 1).expect("previous"),
                operand: NodeId(0),
            },
        ),
    ];
    for profile in [EVENT, SAMPLE] {
        for closure in [Closure::Complete, Closure::Open] {
            for (source, root) in &rows {
                let result = component(source, profile, closure);
                assert_eq!(result.formula().nodes().last().expect("root").kind, *root);
                assert_eq!(result.formula().schema_version(), FormulaSchemaVersion::V2);
                assert_eq!(
                    result.formula().semantic_profile(),
                    SemanticProfile::OriginCompleteHistoryV1
                );
                let Support::Supported { premises, .. } = result.classification().support else {
                    panic!("supported")
                };
                assert!(premises.contains(&"closure and history premises"));
            }
        }
    }
}

#[trace("IT-020-SC-02")]
#[test]
fn binding_and_unsupported_stops_never_produce_a_component() {
    let (package, foreign, clause) = packages("eventually[0,2] holds(true)", EVENT);
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    assert!(matches!(
        construct_tl_formula(
            &foreign,
            &subject,
            &classification,
            Closure::Complete,
            TlFormulaLimits::default()
        ),
        Err(TlFormulaError::Binding(
            MappingBindingRefusal::ForeignSubject
        ))
    ));
    let mut altered = classification.clone();
    altered.support = Support::Unsupported { dimensions: vec![] };
    assert!(matches!(
        construct_tl_formula(
            &package,
            &subject,
            &altered,
            Closure::Complete,
            TlFormulaLimits::default()
        ),
        Err(TlFormulaError::Binding(
            MappingBindingRefusal::ClassificationMismatch
        ))
    ));
    assert!(matches!(
        construct_tl_formula(
            &package,
            &subject,
            &classification,
            Closure::Open,
            TlFormulaLimits::default()
        ),
        Err(TlFormulaError::Binding(
            MappingBindingRefusal::ClassificationMismatch
        ))
    ));
    let (package, _, clause) = packages("eventually[0,2] (once[0,1] holds(true))", EVENT);
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let error = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect_err("mixed directions");
    let TlFormulaError::Unsupported(retained) = error else {
        panic!("unsupported classification")
    };
    assert_eq!(*retained, classification);
    assert_eq!(
        retained.support,
        Support::Unsupported {
            dimensions: vec![Unmatched::MixedDirection]
        }
    );
}

#[trace("IT-020-SC-03")]
#[test]
fn exact_node_work_and_canonical_byte_settings_admit_and_first_excess_stops() {
    let (package, _, clause) = packages("holds(true) and holds(false)", EVENT);
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let run = |limits| {
        construct_tl_formula(
            &package,
            &subject,
            &classification,
            Closure::Complete,
            limits,
        )
    };
    let baseline = run(TlFormulaLimits::default()).expect("baseline");
    let exact = TlFormulaLimits {
        max_bytes: u64::try_from(baseline.bytes().len()).expect("length"),
        max_nodes: 3,
        max_work: 6,
    };
    for _ in 0..2 {
        assert_eq!(
            run(exact).expect("exact settings reset per call").bytes(),
            baseline.bytes()
        );
    }
    for (limit, setting, limits) in [
        (
            TlFormulaLimit::Bytes,
            exact.max_bytes - 1,
            TlFormulaLimits {
                max_bytes: exact.max_bytes - 1,
                ..exact
            },
        ),
        (
            TlFormulaLimit::Nodes,
            2,
            TlFormulaLimits {
                max_nodes: 2,
                ..exact
            },
        ),
        (
            TlFormulaLimit::Work,
            5,
            TlFormulaLimits {
                max_work: 5,
                ..exact
            },
        ),
        (
            TlFormulaLimit::Nodes,
            0,
            TlFormulaLimits {
                max_nodes: 0,
                max_work: 0,
                ..exact
            },
        ),
        (
            TlFormulaLimit::Work,
            0,
            TlFormulaLimits {
                max_work: 0,
                ..exact
            },
        ),
        (
            TlFormulaLimit::Bytes,
            0,
            TlFormulaLimits {
                max_bytes: 0,
                ..exact
            },
        ),
    ] {
        let error = run(limits).expect_err("first excess unit");
        assert!(
            matches!(error, TlFormulaError::LimitReached { limit: actual, value, setting: actual_setting } if actual == limit && actual_setting == setting && value == setting + 1),
            "{error:?}"
        );
        assert_eq!(error.code(), TlFormulaCode::LimitReached);
    }
}

#[trace("IT-020-SC-02", "IT-020-SC-04")]
#[test]
fn unbounded_source_retains_original_infinite_trace_classification_and_codes_roundtrip() {
    let (package, _, clause) =
        packages("eventually holds(true)", "quire.temporal.infinite-trace/v1");
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let error = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect_err("unbounded infinite profile remains unsupported");
    let TlFormulaError::Unsupported(retained) = &error else {
        panic!("original owner disposition: {error:?}")
    };
    assert_eq!(retained.as_ref(), &classification);
    assert_eq!(
        retained.support,
        Support::Unsupported {
            dimensions: vec![Unmatched::InfiniteTrace]
        }
    );
    assert_eq!(error.code(), TlFormulaCode::Unsupported);
    for &code in TlFormulaCode::all() {
        assert_eq!(TlFormulaCode::from_code(code.as_str()), Some(code));
    }
    assert_eq!(TlFormulaCode::from_code("foreign-code"), None);
}

#[trace("IT-020-SC-02")]
#[test]
fn all_owner_unsupported_dimensions_are_retained_in_order() {
    let (package, _, clause) = packages(
        "eventually[0,2] (once[0,1] holds(true))",
        "quire.temporal.timestamped-event.finite-window/v1",
    );
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let error = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect_err("two unsupported dimensions");
    let TlFormulaError::Unsupported(retained) = error else {
        panic!("original unsupported classification")
    };
    assert_eq!(*retained, classification);
    assert_eq!(
        retained.support,
        Support::Unsupported {
            dimensions: vec![Unmatched::FiniteWindow, Unmatched::MixedDirection]
        }
    );
}

#[trace("IT-020-SC-01", "IT-020-SC-04")]
#[test]
fn bounded_endpoint_conversion_is_exact_and_out_of_owner_range_refuses() {
    let result = component(
        "eventually[4294967295,4294967295] holds(true)",
        EVENT,
        Closure::Complete,
    );
    assert_eq!(
        result.formula().nodes().last().expect("root").kind,
        NodeKind::Future {
            interval: Interval::new(u32::MAX, u32::MAX).expect("endpoint"),
            operand: NodeId(0)
        }
    );
    let (package, _, clause) = packages("eventually[4294967296,4294967296] holds(true)", EVENT);
    let subject = temporal_subject(&package, &clause).expect("subject");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let error = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect_err("owner endpoint range");
    assert_eq!(error.code(), TlFormulaCode::Representation);
    let TlFormulaError::Representation(node) = error else {
        panic!("typed representation refusal")
    };
    assert_eq!(&node, subject.formula_graph()[0].node);
}

#[trace("IT-020-SC-02", "IT-020-SC-04")]
#[test]
fn bounded_profiles_refuse_missing_intervals_before_any_checked_package_exists() {
    let document =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("selected owner model fixture");
    for profile in [EVENT, SAMPLE] {
        let refusals = checked_formula("eventually holds(true)", profile, &document)
            .expect_err("bounded profiles require an explicit closed interval");
        let [refusal] = refusals.as_slice() else {
            panic!("exactly one native missing-interval refusal: {refusals:?}")
        };
        let CheckCause::Temporal(cause) = &refusal.cause else {
            panic!("original typed native temporal refusal: {refusal:?}")
        };
        let TemporalCause::MissingInterval { span } = cause.as_ref() else {
            panic!("original typed native MissingInterval refusal: {refusal:?}")
        };
        assert_eq!(span.end - span.start, "eventually".len());
    }
}

#[trace("IT-020-SC-04")]
#[test]
fn an_aggregate_atom_is_a_representation_refusal() {
    let (package, _, clause) = packages(
        "@value capture n: Int[0, 9] = e; always[0,5] (sum of n over past[0,2] <= 15)",
        EVENT,
    );
    let subject = temporal_subject(&package, &clause).expect("aggregate is a readable atom");
    let classification = mapping_support(&subject, Closure::Complete).expect("classification");
    let error = construct_tl_formula(
        &package,
        &subject,
        &classification,
        Closure::Complete,
        TlFormulaLimits::default(),
    )
    .expect_err("an aggregate atom has no proposition");
    assert_eq!(error.code(), TlFormulaCode::Representation);
}
