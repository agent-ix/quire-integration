// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-350: genuine native occurrence, evidence and TL-owner projection.

#[path = "../temporal_projection/observation.rs"]
mod observation;
use crate::support;

use qsl_eval::temporal::{
    evidence::{
        CompletenessRecord, EvidencePosition, EvidenceRef, EvidenceRequest, ExecutionDisposition,
        ExecutionOrigin, Payload, Relation, SemanticTriggerIdentity, TemporalEvidenceLimits,
        TraceBinding, TriggerEvidence,
    },
    ClockCover, ProgressClosure, ProgressCompleteness,
};
use qsl_package::checked_views::{
    mapping_support, temporal_subject, Classification, Closure, TemporalSubjectView,
};
use qsl_semantics::{
    check::{CheckRefusal, CheckedGraph, PackageDeclarations},
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
    project_tl, read_observation_revision, NativeProjectionInput, ObservationAdmissionLimits,
    TlLimits, TlProjectionError, TlProjectionSelection, ValidatedTlProjection,
};
use quire_mltl::request::ObservationInputs;
use quire_observation::authority::{self as a, bundle, Limits, OpenClosed};
use quire_semantic_value::checking::CheckingLimits;
use quire_spec_language::temporal::{
    self as native, ObservationRef, ValuationEvidence, ValuationInstance, ValuationRow,
};
use std::collections::BTreeMap;
use tl_mltl::wire::OwnerLimits;

/// Parse and check original source without bypassing native temporal admission.
pub(crate) fn checked_formula(
    formula: &str,
    profile: &str,
    document: &[u8],
) -> Result<CheckedGraph, Vec<CheckRefusal>> {
    let source = format!(
        r#"language "ix:native" edition "1-draft";
profile v = "quire.model.complete/v1";
profile t = "{profile}";
model M = "acme/orders";
temporal Due using t over (g: M::Gadget) clock "events" on origin {{
    {formula}
}}
"#
    );
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

thread_local! {
    /// The fixed-sample period and unit the next packages on this thread declare.
    static SAMPLE_CLOCK: std::cell::Cell<(&'static str, &'static str)> =
        const { std::cell::Cell::new(("1", "nanoseconds")) };
}

thread_local! {
    /// Anchors the observation records as samples (`Some(true)`) or event positions
    /// (`Some(false)`) regardless of the declared clock.
    static OBSERVATION_SAMPLE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Run `test` with observation records of the given anchor form.
#[allow(dead_code)] // Only the clock-binding scenarios mix the forms.
pub(crate) fn with_observation_form<T>(sample: bool, test: impl FnOnce() -> T) -> T {
    let previous = OBSERVATION_SAMPLE.with(|form| form.replace(Some(sample)));
    let result = test();
    OBSERVATION_SAMPLE.with(|form| form.set(previous));
    result
}

/// Run `test` with packages declaring a fixed-sample clock of `period` and `unit`.
#[allow(dead_code)] // Only the clock-binding scenarios vary the declared clock.
pub(crate) fn with_sample_clock<T>(
    period: &'static str,
    unit: &'static str,
    test: impl FnOnce() -> T,
) -> T {
    let previous = SAMPLE_CLOCK.with(|clock| clock.replace((period, unit)));
    let result = test();
    SAMPLE_CLOCK.with(|clock| clock.set(previous));
    result
}

/// Emit once and admit twice: the packages have equal contents and distinct owners.
pub(crate) fn packages(
    formula: &str,
    profile: &str,
) -> (CheckedPackageV2, CheckedPackageV2, CheckedNodeId) {
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
                    let (period, unit) = SAMPLE_CLOCK.with(std::cell::Cell::get);
                    ClockConfiguration::FixedSample {
                        epoch: ClockExactNumber::Integer {
                            decimal: "0".into(),
                        },
                        period: ClockExactNumber::Integer {
                            decimal: period.into(),
                        },
                        unit: unit.into(),
                    }
                } else if profile.contains("timestamped-event") {
                    ClockConfiguration::TimestampedEvent {
                        timestamp_unit: "nanoseconds".into(),
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
        qsl_package::EmitLimits::new(16 * 1024 * 1024),
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

pub(crate) const EVENT: &str = "quire.temporal.event-position.false-extension/v1";
pub(crate) const SAMPLE: &str = "quire.temporal.fixed-sample.false-extension/v1";
pub(crate) const CAPTURE_INSTANCE: &str = "capture-environment:origin";

#[derive(Clone, Copy)]
pub(crate) struct Case<'a> {
    pub(crate) package: &'a CheckedPackageV2,
    pub(crate) subject: &'a TemporalSubjectView<'a>,
    pub(crate) classification: &'a Classification,
    pub(crate) surrounding: Closure,
    pub(crate) native: NativeProjectionInput<'a>,
    pub(crate) selection: TlProjectionSelection<'a>,
}
impl<'a> Case<'a> {
    pub(crate) fn project(
        &self,
        limits: TlLimits,
    ) -> Result<ValidatedTlProjection<'a>, TlProjectionError> {
        project_tl(
            self.package,
            self.subject,
            self.classification,
            self.surrounding,
            self.native,
            self.selection,
            limits,
        )
    }
}

pub(crate) fn with_case(formula: &str, sample: bool, test: impl FnOnce(Case<'_>)) {
    with_case_scopes(formula, sample, Closure::Complete, OpenClosed::Closed, test);
}

pub(crate) fn with_case_scopes(
    formula: &str,
    sample: bool,
    surrounding: Closure,
    decision: OpenClosed,
    test: impl FnOnce(Case<'_>),
) {
    with_case_watermark(formula, sample, surrounding, decision, 1, test);
}

pub(crate) fn with_case_watermark(
    formula: &str,
    sample: bool,
    surrounding: Closure,
    decision: OpenClosed,
    watermark: u64,
    test: impl FnOnce(Case<'_>),
) {
    let surrounding_state = if surrounding == Closure::Complete {
        OpenClosed::Closed
    } else {
        OpenClosed::Open
    };
    let native_closure = |state| {
        if state == OpenClosed::Closed {
            ProgressClosure::Closed
        } else {
            ProgressClosure::Open
        }
    };
    let profile = if sample { SAMPLE } else { EVENT };
    let (package, _, clause) = packages(formula, profile);
    let subject = temporal_subject(&package, &clause).expect("checked temporal subject");
    let classification = mapping_support(&subject, surrounding).expect("classification");
    let occurrence = &subject.occurrences()[0];
    let bridge_definition = quire_integration::temporal::read_tl_bridge_definition(
        quire_integration::temporal::TL_BRIDGE_DEFINITION,
        TlLimits::default(),
    )
    .expect("original integration bridge definition");
    let f = observation::Fixture::new(
        OBSERVATION_SAMPLE
            .with(std::cell::Cell::get)
            .unwrap_or(sample),
    );
    let context = f.context(1, None);
    let owner_limits = Limits::owner_max();
    let selection = f.selection(None);
    let publication =
        bundle::publish(context, &selection, None, owner_limits).expect("owner bundle");
    let observed = f.observation(f.observation_document(), 1, None);
    let observation = read_observation_revision(
        publication.document().bytes(),
        context,
        &selection,
        &observed,
        None,
        owner_limits,
        ObservationAdmissionLimits::default(),
    )
    .expect("strict integration observation admission");
    let clock_selection =
        a::clock::Selection::new(observation::id("events"), observation::id("clock-version"));
    let clock_document =
        a::clock::derive(context, &clock_selection, owner_limits).expect("clock producer");
    let clock = a::clock::read(
        clock_document.bytes(),
        context,
        &clock_selection,
        owner_limits,
    )
    .expect("clock reader");
    let boundary = || {
        if sample {
            a::TemporalBoundary::FixedSample {
                lower: 0,
                upper_inclusive: 0,
                carrier_end_exclusive: 1,
                watermark,
            }
        } else {
            a::TemporalBoundary::EventPosition {
                lower: 0,
                upper_inclusive: 0,
                carrier_end_exclusive: 1,
                watermark,
            }
        }
    };
    let read_progress = |state| {
        let progress_selection = a::progress::Selection::new(
            clock_selection.clone(),
            vec![observation::id("source")],
            boundary(),
            state,
            observation::id("trigger"),
            f.cutoff(),
            observation::id("restoration"),
        );
        let progress_document = a::progress::derive(context, &progress_selection, owner_limits)
            .expect("progress producer");
        let progress = a::progress::read(
            progress_document.bytes(),
            context,
            &progress_selection,
            owner_limits,
        )
        .expect("progress reader");
        progress
    };
    let read_closure = |state| {
        let closure_selection = a::closure::Selection::new(
            observation::id("events"),
            observation::id("clock-version"),
            vec![observation::id("source")],
            boundary(),
            state,
        );
        let closure_document = a::closure::derive(context, &closure_selection, owner_limits)
            .expect("closure producer");
        let closure = a::closure::read(
            closure_document.bytes(),
            context,
            &closure_selection,
            owner_limits,
        )
        .expect("closure reader");
        closure
    };
    let progress = read_progress(decision);
    let closure = read_closure(decision);
    let surrounding_progress = read_progress(surrounding_state);
    let surrounding_closure = read_closure(surrounding_state);
    let completeness_selection = a::completeness::Selection::new(
        observation::id("boundary"),
        vec![a::completeness::Fact::new(
            observation::id("member"),
            Some(f.qualified.records()[0].identity.clone()),
            a::completeness::FactStatus::Available,
        )],
    );
    let completeness_document =
        a::completeness::derive(context, &completeness_selection, owner_limits)
            .expect("completeness producer");
    let completeness = a::completeness::read(
        completeness_document.bytes(),
        context,
        &completeness_selection,
        owner_limits,
    )
    .expect("completeness reader");
    let availability_selection = a::availability::Selection::new(
        vec![observation::id("result")],
        vec![observation::id("result")],
        a::availability::DependencyState::Available,
        a::availability::DependencyState::Available,
    );
    let availability_document =
        a::availability::derive(context, &availability_selection, owner_limits)
            .expect("availability producer");
    let availability = a::availability::read(
        availability_document.bytes(),
        context,
        &availability_selection,
        owner_limits,
    )
    .expect("availability reader");
    let anchor = observed.payload().observation_identity();
    assert!(
        anchor.parse::<u64>().is_err(),
        "the observation identity is an opaque name, never a numeric anchor"
    );
    let instance = ValuationInstance {
        anchor,
        capture_instance: CAPTURE_INSTANCE,
    };
    let revision = ObservationRef {
        reference: observation.bundle().identity().as_str().to_owned(),
        revision: observation.bundle().revision().to_string(),
    };
    let binding = subject.clock_of(occurrence).expect("selected native clock");
    let trigger =
        SemanticTriggerIdentity::new(b"whole-execution:one".as_slice()).expect("origin identity");
    let request = EvidenceRequest {
        native_valuations: Vec::new(),
        trigger: Some(trigger.clone()),
        correspondence: EvidenceRef {
            authority_identity: bridge_definition.authority().to_owned(),
            ..EvidenceRef::named(bridge_definition.identity())
        },
        positions: vec![EvidencePosition {
            observation: EvidenceRef::named(anchor),
            position: 0,
            ..EvidencePosition::default()
        }],
        anchor: Some(anchor.to_owned()),
        deliveries: Vec::new(),
        origins: vec![ExecutionOrigin {
            identity: trigger,
            payload: Payload::new(b"origin".as_slice()),
            captures: Vec::new(),
        }],
        trigger_evidence: TriggerEvidence::Admitted,
        trigger_scope: ProgressClosure::Closed,
        decision_progress: Some(watermark),
        decision_closure: Some(native_closure(decision)),
        surrounding_progress: Some(watermark),
        surrounding_closure: Some(native_closure(surrounding_state)),
        execution: ExecutionDisposition::Completed,
        completeness: Some(CompletenessRecord {
            reference: EvidenceRef::named("complete"),
            state: ProgressCompleteness::Complete { through: 0 },
            facts: Vec::new(),
        }),
        authoritative_origin: Some(true),
        evicted: Vec::new(),
        cover: ClockCover::Exact,
        progress_ledger: None,
        ledger: None,
        relation: Relation::Original,
        control: None,
        binding: TraceBinding {
            profile: subject.profile().definition.identity.clone(),
            clock_name: binding.clock_name.clone(),
            parameters: binding.clock.clone(),
        },
        occurrence: Some(quire_contract_ir::CheckedOccurrence {
            role: occurrence.role.clone(),
            ordinal: occurrence.ordinal,
        }),
        limits: TemporalEvidenceLimits::default(),
    };
    let trace = native_trace(profile, anchor);
    let rows = subject
        .predicate_leaves()
        .iter()
        .enumerate()
        .map(|(index, operand)| ValuationRow {
            subject: occurrence.clone(),
            position: 0,
            anchor: anchor.to_owned(),
            capture_instance: CAPTURE_INSTANCE.to_owned(),
            revision: revision.revision.clone(),
            operand: operand.clone(),
            value: index % 2 == 0,
        })
        .collect();
    let valuations = ValuationEvidence::admit(&subject, &trace, instance, &revision.revision, rows)
        .expect("actual native valuation admission");
    test(Case {
        package: &package,
        subject: &subject,
        classification: &classification,
        surrounding,
        native: NativeProjectionInput {
            request: &request,
            valuations: &valuations,
            occurrence,
            instance,
            observation: &revision,
        },
        selection: TlProjectionSelection {
            native_report_limits: quire_spec_language::temporal::ReportPredecessorLimits::new(
                1 << 30,
                1 << 30,
            ),
            bridge_definition: &bridge_definition,
            observation: &observation,
            clock: &clock,
            observations: ObservationInputs {
                decision_scope_progress: &progress,
                decision_scope_closure: &closure,
                surrounding_execution_progress: &surrounding_progress,
                surrounding_execution_closure: &surrounding_closure,
                completeness: &completeness,
                availability: &availability,
            },
            owner_limits: OwnerLimits::default(),
        },
    });
}

pub(crate) fn native_trace(profile: &str, anchor: &str) -> native::Trace {
    native::Trace {
        clock: native::ClockBinding {
            name: "events".into(),
            profile_identity: profile.into(),
            parameters: BTreeMap::new(),
        },
        positions: vec![native::Position {
            coordinate: 0,
            order: None,
            valuations: BTreeMap::new(),
            instance_valuations: BTreeMap::new(),
        }],
        anchor: anchor.into(),
        triggers: Vec::new(),
        trigger_evidence: native::Evidence::Admitted,
        trigger_scope: native::Closure::Closed,
        decision_scope: native::Closure::Closed,
        surrounding_execution: native::Closure::Closed,
        execution: native::Execution::Completed,
        completeness: native::Completeness::Complete,
        authoritative_origin: true,
        watermark: 1,
        evicted: Vec::new(),
    }
}

/// The native trace shape the owner admits valuation rows against for this case.
#[allow(dead_code)] // Not every target sharing this support admits valuation rows.
pub(crate) fn trace_for(case: &Case<'_>) -> native::Trace {
    let profile: &str = &case.subject.profile().definition.identity;
    native_trace(profile, case.native.instance.anchor)
}
