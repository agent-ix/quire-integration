// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original AC3 inputs passed through the public QSL producer and strict IR reader.

use qsl_analyze::{
    monitor_entries::{names, render, spec_from_plan, WIRE_FIELD_NAMES},
    monitor_root::{emit_plan_root, PlanRoot, PlanRootRequest},
};
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, emit_monitor_plan_over, Emission, MonitorPlanWire};
use qsl_replay::{
    spine::{check, parse, select, ParseRequest, SpineLimits},
    DependencyInput, MonitorTarget, TimingObligationSettings,
};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_ir::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageReadLimits, CheckedPackageV2,
};
use quire_exact::{Cancel, Integer, Rational};

/// Producer outputs retained intact for driver preparation, including its witnesses.
pub struct Produced {
    pub root: PlanRoot,
    pub package: Box<CheckedPackageV2>,
    source: String,
    document: Vec<u8>,
    identity: String,
    source_identity: SourceIdentity,
    library: LibraryName,
}

pub fn target() -> MonitorTarget {
    MonitorTarget {
        counter: "ac3-counter".to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: Rational::from_integer(Integer::zero()),
        max_event_rate: Some(Rational::from_integer(Integer::from(2_u64))),
        max_reading_gap: 100,
        max_activations: 8,
    }
}

pub fn read(emission: &Emission, identity: &str, document: &[u8]) -> Box<CheckedPackageV2> {
    match read_result(emission, identity, document, emission.package().bytes()) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("the actual producer emission must admit: {other:?}"),
    }
}

fn read_result(
    emission: &Emission,
    identity: &str,
    document: &[u8],
    bytes: &[u8],
) -> CheckedPackageDispatchResult {
    let mut evidence = emission.evidence().clone();
    evidence
        .insert_domain_package_document(identity, document.to_vec())
        .expect("the original model document supplies reader evidence");
    read_checked_package(
        bytes,
        CheckedPackageReadLimits {
            bytes: 8 * 1024 * 1024,
            nodes: 100_000,
            edges: 1_000_000,
            occurrences: 1_000_000,
            diagnostics: 1_000,
            work: 100_000_000,
        },
        &evidence,
    )
}

impl Produced {
    pub fn healthy() -> Self {
        let document = std::fs::read(
            super::support::qsl_root().join("examples/config-version/model.semantic-ir.json"),
        )
        .expect("read the QSL-owned model by reference");
        let domain: serde_json::Value = serde_json::from_slice(&document).unwrap();
        let identity = domain["package"]["identity"].as_str().unwrap().to_owned();
        let source = format!(
            r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit ms;
temporal Construction using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}}
"#
        );
        let source_identity = SourceIdentity::new("agent-ix", "monitor-program-adverse");
        let library = LibraryName::new("agent-ix/monitor-program-adverse").unwrap();
        let packages = package_input([document.as_slice()]).unwrap();
        let root = emit_plan_root(
            &PlanRootRequest {
                source: &source_identity,
                path: "monitor-program-adverse.native",
                bytes: source.as_bytes(),
                packages: &packages,
                package_name: &library,
                limits: SpineLimits::default(),
                clause: "Construction",
                target: &target(),
                settings: &TimingObligationSettings::default(),
            },
            &Cancel::new(),
        )
        .expect("the public producer emits the actual complete plan root");
        let package = read(&root.emission, &identity, &document);
        assert_eq!(package.monitor_plans().len(), 1);
        Self {
            root,
            package,
            source,
            document,
            identity,
            source_identity,
            library,
        }
    }

    /// Read an adverse byte input against the original final producer evidence.
    pub fn read_bytes(&self, bytes: &[u8]) -> CheckedPackageDispatchResult {
        read_result(&self.root.emission, &self.identity, &self.document, bytes)
    }

    /// Recheck owner-generated entry source and let the producer assign all new keys.
    /// With `unsupported` false this must reproduce the healthy emission exactly.
    pub fn entry_emission(&self, unsupported: bool) -> (Emission, Box<CheckedPackageV2>) {
        let mut entries = render(&spec_from_plan(&self.root.plan).unwrap());
        if unsupported {
            const BEFORE: &str = "let initial = State { now: 0, history_start:";
            const AFTER: &str = "let initial = State { now: 0 mod 1, history_start:";
            assert_eq!(
                entries.matches(BEFORE).count(),
                1,
                "one exact producer initializer"
            );
            entries = entries.replacen(BEFORE, AFTER, 1);
        }
        let mut combined = self.source.as_bytes().to_vec();
        combined.push(b'\n');
        combined.extend_from_slice(entries.as_bytes());
        let packages = package_input([self.document.as_slice()]).unwrap();
        let limits = SpineLimits::default();
        let cancel = Cancel::new();
        let parsed = parse(
            &ParseRequest {
                source: &self.source_identity,
                path: "monitor-program-adverse.native",
                bytes: &combined,
            },
            limits.source,
            &cancel,
        )
        .unwrap()
        .into_value();
        let selected = select(&parsed, &packages, limits.model, &cancel)
            .unwrap()
            .into_value();
        let checked = check(
            &parsed,
            self.library.as_str().to_owned(),
            &selected,
            &DependencyInput::default(),
            limits,
            &cancel,
        )
        .unwrap()
        .into_value();
        let emitted = emit_checked(
            checked.package(),
            qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
        )
        .unwrap();
        let mut evidence = emitted.evidence().clone();
        evidence
            .insert_domain_package_document(self.identity.as_str(), self.document.clone())
            .unwrap();
        let plan = &self.root.plan;
        let premises = plan.root_premises();
        let buffers = plan
            .buffers()
            .iter()
            .map(|(id, capacity)| (id.index(), *capacity))
            .collect::<Vec<_>>();
        let emission = emit_monitor_plan_over(
            &emitted,
            self.library.clone(),
            &evidence,
            &MonitorPlanWire {
                clause: plan.subject().clause_node(),
                formula: plan.program().formula(),
                counter: &premises.counter,
                counter_width_bits: premises.counter_width_bits,
                max_reading_gap: premises.max_reading_gap,
                max_event_rate: &premises.max_event_rate,
                uncertainty: &premises.uncertainty,
                buffers: &buffers,
                tick_unit: plan.binding().unit(),
                activation_capacity: premises.activation_capacity,
                state: names::STATE,
                inputs: names::INPUTS,
                scope: names::SCOPE,
                result: names::RESULT,
                entries: [
                    names::INIT,
                    names::ON_EVENT,
                    names::ON_WATERMARK,
                    names::ON_GAP,
                ],
                renamed_records: &["Instance", "Activation", "LimitReached"],
                renames: &WIRE_FIELD_NAMES,
            },
            qsl_package::V2ReadLimits::bounded(qsl_replay::spine::I2_NATIVE_JOIN_STORAGE_DEFAULT),
            qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
        )
        .expect("the producer rekeys the real checked entry and its retained witnesses");
        let package = read(&emission, &self.identity, &self.document);
        (emission, package)
    }
}
