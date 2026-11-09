// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared authentic timed PlanRoot production, admission and request support.

use std::{path::PathBuf, process::Command};

use qsl_analyze::monitor_root::{emit_plan_root, PlanRoot, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_package::Emission;
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageReadLimits,
    CheckedPackageV2,
};
use quire_exact::{Cancel, Integer, Rational};

/// A second authored claim with the legacy identical clause body.
pub const SAME_CLAUSE_TWIN: &str = r#"temporal TwinMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}"#;

/// A distinct second clause for original-subject mapping controls.
pub const DISTINCT_CLAUSE: &str = r#"temporal OtherMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    eventually[0 ms, 3 ms] holds(candidate.versionNumber >= 1)
}"#;

/// Native producer output, strict admission and original witness holders.
pub struct Fixture {
    /// Strictly admitted native package.
    pub package: Box<CheckedPackageV2>,
    /// Native evidence extended with the selected domain document.
    pub evidence: quire_contract_ir::CheckedPackageEvidence,
    /// Independently supplied hardware counter expectation.
    pub counter: &'static str,
    /// Original paired producer plan and final emission.
    root: PlanRoot,
    /// Exact producer capacities joined to final admitted subformula IDs.
    pub buffers: Vec<(CheckedNodeId, Box<str>)>,
}

impl Fixture {
    /// Borrow the original producer subject from its retained plan.
    pub fn subject(&self) -> &qsl_replay::MonitorSubject {
        self.root.plan().subject()
    }

    /// Take the original producer root, consuming the fixture.
    pub fn into_root(self) -> PlanRoot {
        self.root
    }

    /// Borrow the original final emission paired with the retained plan.
    pub fn emission(&self) -> &Emission {
        self.root.emission()
    }
}

/// Produce and strictly admit a selected clause with the requested companion source.
pub fn fixture(selected: &str, counter: &'static str, additional_clause: &str) -> Fixture {
    let target = MonitorTarget {
        counter: counter.to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: Rational::from_integer(Integer::zero()),
        max_event_rate: Some(Rational::from_integer(Integer::from(2_u64))),
        max_reading_gap: 100,
        max_activations: 8,
    };
    fixture_for(selected, counter, additional_clause, "", "ms", &target)
}

/// As [`fixture`], with extra unit declarations, the declared time unit of
/// the model's time source, and an explicit hardware target.
pub fn fixture_for(
    selected: &str,
    counter: &'static str,
    additional_clause: &str,
    extra_units: &str,
    time_unit: &str,
    target: &MonitorTarget,
) -> Fixture {
    try_fixture_for(
        selected,
        counter,
        additional_clause,
        extra_units,
        time_unit,
        target,
    )
    .expect("actual final PlanRoot producer")
}

/// As [`fixture_for`], returning the producer's refusal instead of panicking.
pub fn try_fixture_for(
    selected: &str,
    counter: &'static str,
    additional_clause: &str,
    extra_units: &str,
    time_unit: &str,
    target: &MonitorTarget,
) -> Result<Fixture, qsl_analyze::monitor_root::PlanRootRefusal> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("resolved dependency metadata");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == "qsl-replay")
        .unwrap()["manifest_path"]
        .as_str()
        .unwrap();
    let path = PathBuf::from(manifest);
    let document = std::fs::read(
        path.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/config-version/model.semantic-ir.json"),
    )
    .unwrap();
    let document_value: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = document_value["package"]["identity"].as_str().unwrap();
    let packages = package_input([document.as_slice()]).unwrap();
    let text = format!(
        r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit {time_unit};
{extra_units}
temporal SelectedMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}}
{additional_clause}
"#
    );
    let source = SourceIdentity::new("agent-ix", "monitor-preparation");
    let name = LibraryName::new("agent-ix/monitor-preparation").unwrap();
    let root = emit_plan_root(
        &PlanRootRequest {
            source: &source,
            path: "monitor-preparation.native",
            bytes: text.as_bytes(),
            packages: &packages,
            package_name: &name,
            limits: SpineLimits::default(),
            clause: selected,
            target,
            settings: &TimingObligationSettings::default(),
        },
        &Cancel::new(),
    )?;
    let mut evidence = root.emission().evidence().clone();
    evidence
        .insert_domain_package_document(identity, document)
        .unwrap();
    let package =
        match read_checked_package(root.emission().package().bytes(), read_limits(), &evidence) {
            CheckedPackageDispatchResult::AdmittedV2(package) => package,
            other => panic!("strict final package admission: {other:?}"),
        };
    // Capacity values are selected from the producer-owned plan rather than
    // recomputed by the driver. Retain its independently produced values and
    // use the final emission's exact subformula identities.
    let plan = package.monitor_plans().next().unwrap();
    assert_eq!(plan.buffers().len(), root.plan().buffers().len());
    let buffers = plan
        .buffers()
        .zip(root.plan().buffers())
        .map(|(buffer, (_, capacity))| {
            (
                buffer.subformula().clone(),
                capacity.to_string().into_boxed_str(),
            )
        })
        .collect();
    Ok(Fixture {
        package,
        evidence,
        counter,
        root,
        buffers,
    })
}

/// Explicit native read allowances shared by fixture construction and controls.
pub fn read_limits() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits {
        bytes: 8 * 1024 * 1024,
        nodes: 100_000,
        edges: 1_000_000,
        occurrences: 1_000_000,
        diagnostics: 1_000,
        work: 100_000_000,
    }
}
