// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-200 through the composed pipeline: QSL source -> producer plan -> driver `prepare_monitor`
//! -> RT `Program` -> bounded event/watermark executor, on a 20-bit, 1 MHz counter.
//!
//! The expected outcomes are the TC-200 expected results, not the output of
//! any other run.
//!
//! Not covered through the public driver/RT API (so not tagged):
//! - FR-276-AC-5 (sibling identity retained after a limit or fault): in a clause that and-s a
//!   `once[0 ms, 10 ms]` with a longer `eventually`, the producer gives both buffers the same
//!   capacity and they overflow together, so no live sibling survives an overflow to observe.
//! - FR-276-AC-6 (partial): independence of two monitors, no verdict from build, one shared
//!   plan. Not asserted: subject/atom-body/topology retention, and rebinding (RT exposes no
//!   rebind operation to attempt).
//! - FR-276-AC-7 (partial): a foreign source selection and a starved build are covered. Not
//!   covered: altered atom/transition/initializer/interval/buffer/activation/uncertainty/max-gap
//!   and equal subformula ids under two subjects (`prepare_monitor` takes only the producer
//!   root, so these cannot be injected; the driver suites own them), and `Unsupported`.
//! - FR-276-AC-8 (partial): a missing required captures field cannot be offered because the
//!   typed record constructor refuses it first. Not covered: false activation (an `on origin`
//!   clause has no guard) and executor-invariant failure with refusal of later input (no
//!   public injection point).

use std::{collections::BTreeMap, rc::Rc};

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps};
use qsl_eval::temporal::{TickBound, TickMonitorReference, TickMonitorSpec, TickOutcome, TickStep};
use qsl_foundation::SourceIdentity;
use qsl_replay::MonitorTarget;
use qsl_semantics::check::{Activation, CheckedFormula, FormulaNode};
use quire_contract_ir::CheckedNodeId;
use quire_contract_runtime::monitor::{
    build, BuildPlan, Event, InputContext, InputRefusal, Monitor, MonitorFailureCode, Program,
    Reading, StepBudget, Watermark,
};
use quire_driver::monitor::{prepare_monitor, MonitorPreparationLimits};
use quire_exact::{Cancel, FieldValue, Integer, Rational, ScalarLimits, Value};

/// Native PlanRoot support (copied from the driver suite so the producer is the real one).
#[path = "support/monitor_fixture.rs"]
pub mod monitor_fixture;
/// Genuine observation-owner progress admission.
#[path = "support/observation.rs"]
pub mod observation;

const CLAUSE: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}"#;

const LIMITS: MonitorPreparationLimits = MonitorPreparationLimits {
    work: 100_000_000,
    storage: 100_000_000,
};

/// The microsecond tick unit of the 1 MHz counter.
const MICROSECONDS: &str = "unit us : Time = rational(1, 1000000) * second;";

fn ratio(numerator: u64) -> Rational {
    Rational::from_integer(Integer::from(numerator))
}

/// The 20-bit, 1 MHz counter of TC-200 step 1.
fn microsecond_target(uncertainty: u64) -> MonitorTarget {
    MonitorTarget {
        counter: "counter".to_owned(),
        unit: "us".to_owned(),
        width_bits: 20,
        uncertainty: ratio(uncertainty),
        max_event_rate: Some(ratio(2)),
        // Wide enough for the 3001 us reading gap of the scenario.
        max_reading_gap: 10_000,
        max_activations: 8,
    }
}

/// The 16-bit millisecond counter of TC-200 steps 2 and 3, maximum gap 100.
fn millisecond_target(max_reading_gap: u64) -> MonitorTarget {
    MonitorTarget {
        counter: "counter".to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: ratio(0),
        max_event_rate: Some(ratio(2)),
        max_reading_gap,
        max_activations: 8,
    }
}

/// What the QSL reference evaluator needs, owned so it outlives the consumed plan root.
struct Reference {
    formula: CheckedFormula,
    activation: Activation,
    bounds: BTreeMap<usize, TickBound>,
    uncertainty: Rational,
    width_bits: u32,
    max_reading_gap: u64,
    capacities: BTreeMap<usize, u64>,
    activation_capacity: u64,
}

impl Reference {
    fn monitor(&self) -> TickMonitorReference<'_> {
        TickMonitorReference::new(TickMonitorSpec {
            formula: &self.formula,
            activation: self.activation,
            bounds: self.bounds.clone(),
            uncertainty: self.uncertainty.clone(),
            width_bits: self.width_bits,
            max_reading_gap: self.max_reading_gap,
            capacities: self.capacities.clone(),
            activation_capacity: self.activation_capacity,
            unprojectable: Default::default(),
        })
    }
}

/// The producer plan's own subformula ids, read from the plan rather than assumed.
#[derive(Clone, Copy)]
struct Subformulas {
    root: u64,
    /// The first interval operator.
    interval: u64,
}

struct Prepared {
    program: Rc<Program>,
    ids: Subformulas,
    reference: Reference,
    /// The producer's buffer capacity of each buffered operator.
    capacities: Vec<u64>,
    /// The formula nodes of the `holds` atoms in source order (the capture field `a<node>`).
    atoms: Vec<usize>,
}

impl Prepared {
    /// The `n`th `holds` atom of the clause in source order.
    fn atom(&self, n: usize) -> usize {
        self.atoms[n]
    }
}

fn try_prepare(
    clause: &str,
    units: &str,
    time_unit: &str,
    target: &MonitorTarget,
    selection: impl FnOnce(&mut SourceIdentity),
) -> Result<Prepared, quire_driver::monitor::MonitorPreparationError> {
    let fixture = monitor_fixture::fixture_for("C", "counter", clause, units, time_unit, target);
    prepare_fixture(fixture, selection)
}

fn prepare_fixture(
    fixture: monitor_fixture::Fixture,
    selection: impl FnOnce(&mut SourceIdentity),
) -> Result<Prepared, quire_driver::monitor::MonitorPreparationError> {
    // The base Time dimension comes from the strictly admitted package's
    // own dimension declaration, not from the closure under test.
    let time: CheckedNodeId = {
        let mut dimensions = fixture
            .package
            .resolved_graph()
            .nodes
            .iter()
            .filter(|node| &*node.semantic_form == "dimension");
        let node = dimensions.next().expect("the declared Time dimension");
        assert!(dimensions.next().is_none(), "one declared dimension");
        node.node_id.clone()
    };
    let root = fixture.into_root();
    let plan = root.plan();
    let reference = Reference {
        formula: plan.program().formula().clone(),
        activation: plan.program().activation(),
        bounds: plan
            .intervals()
            .iter()
            .map(|(id, interval)| {
                (
                    id.index(),
                    TickBound {
                        lower: interval.lower().clone(),
                        lower_closed: interval.lower_closed(),
                        upper: interval.upper().clone(),
                        upper_closed: interval.upper_closed(),
                    },
                )
            })
            .collect(),
        uncertainty: plan.uncertainty().clone(),
        width_bits: plan.binding().width_bits(),
        max_reading_gap: plan.max_reading_gap(),
        capacities: plan
            .buffers()
            .iter()
            .map(|(id, capacity)| (id.index(), *capacity))
            .collect(),
        activation_capacity: plan.activation_capacity(),
    };
    let ids = Subformulas {
        root: plan.program().root().index() as u64,
        interval: plan.intervals().first().map_or(0, |i| i.0.index() as u64),
    };
    let capacities = plan.buffers().iter().map(|(_, c)| *c).collect();
    let mut holds: Vec<(usize, usize)> = reference
        .formula
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| match node {
            FormulaNode::Holds { span } => Some((span.start, index)),
            _ => None,
        })
        .collect();
    holds.sort();
    let atoms = holds.into_iter().map(|(_, index)| index).collect();
    let mut source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let (progress, subject) = observation::admitted(Some(vec![observation::row(&root)]));
    selection(&mut source);
    let clock = observation::clock();
    let observation_source = observation::source();
    let program = prepare_monitor(
        root,
        PlanAuthoritySelection {
            source: &source,
            claim: &claim,
            observation_subject: &subject,
            observation_clock: &clock,
            observation_source: &observation_source,
        },
        Some(progress),
        PlanAuthorityValidationCaps {
            read: monitor_fixture::read_limits(),
            order: quire_canonical::Limits::new(1 << 20),
        },
        &time,
        LIMITS,
        &Cancel::new(),
    )?;
    Ok(Prepared {
        program,
        ids,
        reference,
        capacities,
        atoms,
    })
}

fn prepare_with(clause: &str, units: &str, time_unit: &str, target: &MonitorTarget) -> Prepared {
    try_prepare(clause, units, time_unit, target, |_| {}).expect("the clause prepares")
}

fn prepare(uncertainty: u64) -> Prepared {
    prepare_with(CLAUSE, MICROSECONDS, "us", &microsecond_target(uncertainty))
}

fn budget() -> StepBudget {
    let big = 1_000_000;
    StepBudget {
        work: 100_000_000,
        scalar: ScalarLimits {
            integer_bits: big,
            decimal_digits: big,
            scale_expansion: big,
            text_input_bytes: big,
            text_scalars: big,
            normalized_scalars: big,
            unit_edges: big,
            value_occurrences: big,
            work_units: big,
            result_units: big,
        },
        stack_frames: 100_000,
        scratch_elements: 1_000_000,
        scratch_bytes: 100_000_000,
    }
}

/// One obligation's identity and settled state.
#[derive(Debug, Eq, PartialEq)]
struct Settled {
    subformula: Integer,
    origin_tick: Integer,
    outcome: String,
    /// The integer fields of the outcome's payload record, in declared order.
    payload: Vec<Integer>,
    /// The text fields of the outcome's payload record, in declared order.
    texts: Vec<String>,
}

/// The value of the named field of a composite, by its declared name.
fn field<'v>(program: &Program, value: &'v Value, name: &str) -> &'v Value {
    let Value::Composite(composite) = value else {
        panic!("a composite value");
    };
    let declaration = program
        .types()
        .composite(composite.declaration().clone())
        .expect("a declared composite");
    let quire_semantic_value::declaration::CompositeShape::Record(fields) = declaration.shape()
    else {
        panic!("a record");
    };
    let position = fields
        .iter()
        .position(|declared| declared.name() == name)
        .unwrap_or_else(|| {
            panic!(
                "field {name} in {:?}",
                fields
                    .iter()
                    .map(|f| f.name().to_owned())
                    .collect::<Vec<_>>()
            )
        });
    let FieldValue::Present(present) = &composite.slots()[position] else {
        panic!("field {name} is present");
    };
    present
}

fn settled(program: &Program, results: &Value) -> Vec<Settled> {
    let Value::Collection(results) = results else {
        panic!("results are a sequence");
    };
    results
        .elements()
        .iter()
        .map(|result| {
            let instance = field(program, result, "instance");
            let Value::Integer(subformula) = field(program, instance, "subformula") else {
                panic!("integer subformula");
            };
            let Value::Integer(origin_tick) = field(program, instance, "origin") else {
                panic!("integer origin");
            };
            let Value::Union(outcome) = field(program, result, "outcome") else {
                panic!("an outcome union");
            };
            let texts = outcome
                .payload()
                .iter()
                .flat_map(|value| match value {
                    Value::Composite(record) => record
                        .slots()
                        .iter()
                        .filter_map(|slot| match slot {
                            FieldValue::Present(Value::String(text)) => Some(text.to_string()),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .collect();
            let payload = outcome
                .payload()
                .iter()
                .flat_map(|value| match value {
                    Value::Composite(record) => record
                        .slots()
                        .iter()
                        .filter_map(|slot| match slot {
                            FieldValue::Present(Value::Integer(integer)) => Some(integer.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .collect();
            Settled {
                subformula: subformula.clone(),
                origin_tick: origin_tick.clone(),
                outcome: outcome.member().identifier().to_owned(),
                payload,
                texts,
            }
        })
        .collect()
}

fn context(program: &Program) -> InputContext<'_> {
    let layout = program.layout();
    InputContext {
        package_lock: &layout.package_lock,
        clause: &layout.clause,
        subject: &layout.subject,
        occurrence: &layout.occurrence,
        clock_binding: &layout.clock_binding,
        counter: &layout.counter,
        tick_unit: &layout.tick_unit,
        counter_width_bits: layout.counter_width_bits,
        profile: &layout.profile,
        admitted_order: &layout.admitted_order,
    }
}

struct Run {
    program: Rc<Program>,
    monitor: Monitor,
    /// The request and acknowledgement atoms in source order.
    atoms: Vec<usize>,
}

impl Run {
    fn new(prepared: &Prepared) -> Self {
        let program = &prepared.program;
        let monitor = build(BuildPlan {
            program: Rc::clone(program),
            budget: budget(),
        })
        .expect("initialization yields a monitor");
        Self {
            program: Rc::clone(program),
            monitor,
            atoms: prepared.atoms.clone(),
        }
    }

    /// The declared capture field names, one per `holds` atom (`a<formula node index>`).
    fn capture_fields(&self) -> Vec<String> {
        let program = &self.program;
        let declaration = program
            .types()
            .composite(program.layout().inputs.key.clone())
            .expect("declared captures");
        let quire_semantic_value::declaration::CompositeShape::Record(fields) = declaration.shape()
        else {
            panic!("captures are a record");
        };
        fields.iter().map(|f| f.name().to_owned()).collect()
    }

    /// Captures where exactly the atoms with the given formula node indices hold.
    fn captures(&self, holding: &[usize]) -> Value {
        let program = &self.program;
        let values = self
            .capture_fields()
            .into_iter()
            .map(|name| {
                let node: usize = name[1..].parse().expect("a<node index>");
                (
                    name,
                    FieldValue::Present(Value::Boolean(holding.contains(&node))),
                )
            })
            .collect::<Vec<_>>();
        program
            .types()
            .record(
                program.layout().inputs.key.clone(),
                values
                    .iter()
                    .map(|(n, v)| (n.as_str(), v.clone()))
                    .collect(),
            )
            .expect("captures")
    }

    /// An event at `tick` where exactly the atoms with the given formula node indices hold.
    fn event_atoms(
        &mut self,
        tick: u64,
        holding: &[usize],
    ) -> Result<Vec<Settled>, quire_contract_runtime::monitor::MonitorFailure> {
        let program = Rc::clone(&self.program);
        let values = self
            .capture_fields()
            .into_iter()
            .map(|name| {
                let node: usize = name[1..].parse().expect("a<node index>");
                (
                    name,
                    FieldValue::Present(Value::Boolean(holding.contains(&node))),
                )
            })
            .collect::<Vec<_>>();
        let captures = program
            .types()
            .record(
                program.layout().inputs.key.clone(),
                values
                    .iter()
                    .map(|(n, v)| (n.as_str(), v.clone()))
                    .collect(),
            )
            .expect("captures");
        let step = self.monitor.event(Event {
            reading: Reading {
                context: context(&program),
                value: tick,
            },
            inputs: &captures,
        })?;
        Ok(settled(&program, step.results()))
    }

    /// An event at counter reading `tick` where the clause's first (request) and second
    /// (acknowledgement) atoms in source order hold as given.
    fn event(&mut self, tick: u64, req: bool, ack: bool) -> Vec<Settled> {
        let holding: Vec<usize> = [(self.atoms[0], req), (self.atoms[1], ack)]
            .into_iter()
            .filter_map(|(atom, holds)| holds.then_some(atom))
            .collect();
        self.event_atoms(tick, &holding).expect("admitted event")
    }

    /// An admitted progress assertion covering ticks through `boundary`.
    fn watermark(&mut self, boundary: u64) -> Vec<Settled> {
        let program = Rc::clone(&self.program);
        let layout = program.layout();
        let scope = program
            .types()
            .record(
                layout.scope.key.clone(),
                vec![(
                    "subject",
                    FieldValue::Present(Value::Integer(Integer::zero())),
                )],
            )
            .expect("scope");
        let boundary = Integer::from(boundary);
        let step = self
            .monitor
            .watermark(Watermark {
                context: context(&program),
                progress_context: &layout.progress_context,
                assertion: b"admitted progress assertion",
                inclusive_boundary: &boundary,
                scope: &scope,
            })
            .expect("admitted watermark");
        settled(&program, step.results())
    }
}

/// A settled obligation from the request at tick 0.
fn settled_at_zero(subformula: u64, outcome: &str) -> Settled {
    Settled {
        subformula: Integer::from(subformula),
        origin_tick: Integer::zero(),
        outcome: outcome.to_owned(),
        payload: Vec::new(),
        texts: Vec::new(),
    }
}

/// Request at tick 0, first acknowledgement at `ack` ticks.
fn after_ack(uncertainty: u64, ack: u64) -> (Vec<Settled>, Subformulas) {
    let prepared = prepare(uncertainty);
    let ids = prepared.ids;
    let mut run = Run::new(&prepared);
    assert!(
        run.event(0, true, false).is_empty(),
        "the request alone settles nothing"
    );
    // TC-200 names the obligations of the request at tick 0; the later
    // event starts its own, unrelated instances.
    let mut results = run.event(ack, false, true);
    results.retain(|result| result.origin_tick == Integer::zero());
    (results, ids)
}

#[trace("TC-200", "FR-276-AC-1")]
#[test]
fn ack_at_2998_is_definitely_true() {
    let (results, ids) = after_ack(0, 2998);
    assert_eq!(results, vec![settled_at_zero(ids.interval, "True")]);
}

#[trace("TC-200", "FR-276-AC-1")]
#[test]
fn ack_at_3000_is_indeterminate() {
    let (results, ids) = after_ack(0, 3000);
    assert_eq!(
        results,
        vec![
            settled_at_zero(ids.root, "Indeterminate"),
            settled_at_zero(ids.interval, "Indeterminate")
        ]
    );
}

#[trace("TC-200", "FR-276-AC-1")]
#[test]
fn ack_at_3001_is_definitely_false() {
    let (results, ids) = after_ack(0, 3001);
    assert_eq!(
        results,
        vec![
            settled_at_zero(ids.root, "False"),
            settled_at_zero(ids.interval, "False")
        ]
    );
}

#[trace("TC-200", "FR-276-AC-1")]
#[test]
fn ack_at_2998_with_two_microseconds_of_uncertainty_is_indeterminate() {
    let (results, ids) = after_ack(2, 2998);
    assert_eq!(
        results,
        vec![
            settled_at_zero(ids.root, "Indeterminate"),
            settled_at_zero(ids.interval, "Indeterminate")
        ]
    );
}

#[trace("TC-200", "FR-276-AC-1")]
#[test]
fn pending_stays_distinct_from_indeterminate_while_the_window_is_open() {
    let prepared = prepare(0);
    let ids = prepared.ids;
    let mut run = Run::new(&prepared);
    run.event(0, true, false);
    let pending = run.watermark(2998);
    assert_eq!(
        pending,
        vec![
            settled_at_zero(ids.root, "Pending"),
            settled_at_zero(ids.interval, "Pending")
        ]
    );
    assert!(pending
        .iter()
        .all(|result| result.outcome != "Indeterminate"));
}

// ---------------------------------------------------------------------------
// The QSL reference evaluator as the independent oracle (TC-200 steps 2, 3 and 6).

/// One obligation's outcome with its payload, comparable across RT and the reference.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Out {
    True,
    False,
    Pending,
    Indeterminate,
    /// The buffer capacity that was reached.
    Incomplete(u64),
    Failed {
        gap: u64,
        max_reading_gap: u64,
    },
    Other(String),
}

type Row = (i128, u64, Out);

fn runtime_rows(results: &[Settled]) -> Vec<Row> {
    let small = |value: &Integer| value.to_u64().expect("fits");
    let mut rows: Vec<Row> = results
        .iter()
        .map(|settled| {
            let origin = i128::from(settled.origin_tick.to_u64().expect("origin fits"));
            let subformula = small(&settled.subformula);
            let out = match settled.outcome.as_str() {
                "True" => Out::True,
                "False" => Out::False,
                "Pending" => Out::Pending,
                "Indeterminate" => Out::Indeterminate,
                "Incomplete" => Out::Incomplete(small(&settled.payload[0])),
                "Failed" => Out::Failed {
                    gap: small(&settled.payload[0]),
                    max_reading_gap: small(&settled.payload[1]),
                },
                other => Out::Other(other.to_owned()),
            };
            (origin, subformula, out)
        })
        .collect();
    rows.sort();
    rows
}

fn reference_rows(results: Vec<qsl_eval::temporal::TickResult>) -> Vec<Row> {
    let mut rows: Vec<Row> = results
        .into_iter()
        .map(|result| {
            let out = match result.outcome {
                TickOutcome::True => Out::True,
                TickOutcome::False => Out::False,
                TickOutcome::Pending => Out::Pending,
                TickOutcome::Indeterminate => Out::Indeterminate,
                TickOutcome::Incomplete { capacity } => Out::Incomplete(capacity),
                TickOutcome::Failed {
                    gap,
                    max_reading_gap,
                } => Out::Failed {
                    gap,
                    max_reading_gap,
                },
                other => Out::Other(format!("{other:?}")),
            };
            (result.origin, result.subformula as u64, out)
        })
        .collect();
    rows.sort();
    rows
}

/// One input of a scenario, offered to both RT and the reference.
#[derive(Clone, Debug)]
enum In {
    /// An event at the counter reading where exactly these formula nodes' atoms hold.
    Event(u64, Vec<usize>),
    /// A bare counter poll.
    Poll(u64),
    /// An admitted progress assertion through the unwrapped boundary.
    Watermark(u64),
}

/// Drive RT and the QSL reference evaluator through the same inputs, returning the
/// per-input rows of each.
fn run_both(prepared: &Prepared, inputs: &[In]) -> (Vec<Vec<Row>>, Vec<Vec<Row>>) {
    let mut run = Run::new(prepared);
    let mut reference = prepared.reference.monitor();
    let atoms: Vec<usize> = run
        .capture_fields()
        .iter()
        .map(|name| name[1..].parse().unwrap())
        .collect();
    let mut runtime = Vec::new();
    let mut oracle = Vec::new();
    for input in inputs {
        let (rt, step) = match input {
            In::Event(tick, holding) => (
                run.event_atoms(*tick, holding).expect("admitted event"),
                TickStep::Reading {
                    reading: *tick,
                    atoms: atoms.iter().map(|a| (*a, holding.contains(a))).collect(),
                    guard: true,
                },
            ),
            In::Poll(tick) => {
                run.poll(*tick).expect("admitted poll");
                (Vec::new(), TickStep::Poll { reading: *tick })
            }
            In::Watermark(boundary) => (
                run.watermark(*boundary),
                TickStep::Watermark {
                    boundary: *boundary,
                },
            ),
        };
        runtime.push(runtime_rows(&rt));
        oracle.push(reference_rows(
            reference.step(&step).expect("reference step"),
        ));
    }
    (runtime, oracle)
}

fn assert_agree(prepared: &Prepared, inputs: &[In]) -> Vec<Vec<Row>> {
    let (runtime, oracle) = run_both(prepared, inputs);
    assert_eq!(
        runtime, oracle,
        "RT and the QSL reference evaluator agree per input"
    );
    runtime
}

impl Run {
    /// A counter poll: validated against the plan, never an event or progress.
    fn poll(&self, tick: u64) -> Result<(), quire_contract_runtime::monitor::MonitorFailure> {
        self.monitor.poll(Reading {
            context: context(&self.program),
            value: tick,
        })
    }
}

// ---------------------------------------------------------------------------
// Step 1 against the oracle.

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_1_boundary_cases_agree_with_the_reference_evaluator() {
    for (uncertainty, ack, expected) in [
        (0, 2998, vec![Out::True]),
        (0, 3000, vec![Out::Indeterminate; 2]),
        (0, 3001, vec![Out::False; 2]),
        (2, 2998, vec![Out::Indeterminate; 2]),
    ] {
        let prepared = prepare_with(CLAUSE, MICROSECONDS, "us", &microsecond_target(uncertainty));
        let (req, ackn) = (prepared.atom(0), prepared.atom(1));
        let rows = assert_agree(
            &prepared,
            &[In::Event(0, vec![req]), In::Event(ack, vec![ackn])],
        );
        // The request's own obligations, with the TC-200 expected outcomes.
        let own: Vec<Out> = rows[1]
            .iter()
            .filter(|(origin, _, _)| *origin == 0)
            .map(|(_, _, out)| out.clone())
            .collect();
        assert_eq!(own, expected, "uncertainty {uncertainty}, ack at {ack}");
    }
}

// ---------------------------------------------------------------------------
// Step 2: the producer's buffer capacity, fixed upper bound.

const ONCE: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    once[0 ms, 10 ms] holds(candidate.versionNumber = 1)
}"#;

#[trace("TC-200", "FR-276-AC-2")]
#[test]
fn step_2_once_overflow_is_incomplete_naming_the_producer_capacity() {
    let prepared = prepare_with(ONCE, "", "ms", &millisecond_target(100));
    let [capacity] = prepared.capacities[..] else {
        panic!("one buffered operator");
    };
    // FR-251 sizes the buffer as ceil(rate * R) + 1 for the widened tick window R, at QSL pin
    // 64cc29bd: 2 events/ms over the 10 ms window and its tick cover gives 23. The RT spec text
    // that says 21 is stale.
    assert_eq!(capacity, 23);
    // The producer, not the runtime or this test, supplies the capacity.
    assert_eq!(
        prepared
            .reference
            .capacities
            .values()
            .copied()
            .collect::<Vec<_>>(),
        [capacity]
    );
    // `capacity` events inside one 10 ms window, then one more: the extra event exceeds the
    // fixed buffer. `p` never holds, so each result depends on the retained history.
    let inputs: Vec<In> = (0..=capacity)
        .map(|i| In::Event(i / 3, Vec::new()))
        .collect();
    let rows = assert_agree(&prepared, &inputs);
    let last = usize::try_from(capacity).unwrap();
    for (index, step) in rows.iter().enumerate() {
        let limited: Vec<_> = step
            .iter()
            .filter(|(_, _, out)| matches!(out, Out::Incomplete(_)))
            .collect();
        if index < last {
            assert!(limited.is_empty(), "no limit before event {index}");
        } else {
            assert!(
                !limited.is_empty(),
                "the overflowing event settles Incomplete"
            );
            assert!(limited
                .iter()
                .all(|(_, _, out)| *out == Out::Incomplete(capacity)));
        }
    }
    // The complete typed payload of the same RT run: the producer's limit and the event-rate
    // setting of 2 events/ms; no event was refused or dropped on the way.
    let mut run = Run::new(&prepared);
    let mut last = Vec::new();
    for input in &inputs {
        let In::Event(tick, holding) = input else {
            unreachable!()
        };
        last = run
            .event_atoms(*tick, holding)
            .expect("every event is admitted");
    }
    let [limited] = &last[..] else {
        panic!("one settled obligation: {last:?}");
    };
    assert_eq!(limited.outcome, "Incomplete");
    assert_eq!(limited.texts, ["event_buffer", "monitor.max_event_rate"]);
    assert_eq!(limited.payload, [Integer::from(capacity)]);
}

// ---------------------------------------------------------------------------
// Step 3: counter wrap, reading gap and producer refusal.

fn sixteen_bit() -> Prepared {
    prepare_with(CLAUSE, "", "ms", &millisecond_target(100))
}

#[trace("TC-200", "FR-276-AC-3")]
#[test]
fn step_3_wrapped_readings_advance_by_the_modular_difference() {
    let prepared = sixteen_bit();
    let (req, ack, ids) = (prepared.atom(0), prepared.atom(1), prepared.ids);
    let rows = assert_agree(
        &prepared,
        &[In::Event(65530, vec![req]), In::Event(4, vec![ack])],
    );
    // 4 - 65530 modulo 2^16 is 10, outside the request's [0 ms, 3 ms]: the request's own
    // obligations are False, the acknowledgement settles its own at unwrapped tick 10, and no
    // reading-gap fault is reported.
    assert_eq!(
        rows[1],
        vec![
            (0, ids.root, Out::False),
            (0, ids.interval, Out::False),
            (10, ids.interval, Out::True),
        ]
    );
}

#[trace("TC-200", "FR-276-AC-3")]
#[test]
fn step_3_backward_reading_fails_with_the_gap_and_no_temporal_verdict() {
    let prepared = sixteen_bit();
    let (req, ack, ids) = (prepared.atom(0), prepared.atom(1), prepared.ids);
    let rows = assert_agree(
        &prepared,
        &[In::Event(65530, vec![req]), In::Event(65500, vec![ack])],
    );
    let failed = Out::Failed {
        gap: 65506,
        max_reading_gap: 100,
    };
    // Exactly the request's open obligations fail; no True or False is emitted.
    assert_eq!(
        rows[1],
        vec![(0, ids.root, failed.clone()), (0, ids.interval, failed)]
    );
}

#[trace("TC-200", "FR-276-AC-3")]
#[test]
fn step_3_gap_equal_to_the_counter_modulus_is_refused_before_runtime_construction() {
    use qsl_analyze::monitor_root::PlanRootRefusal;
    use qsl_replay::MonitorPlanRefusal;
    let refusal = monitor_fixture::try_fixture_for(
        "C",
        "counter",
        CLAUSE,
        "",
        "ms",
        &millisecond_target(65536),
    )
    .err()
    .expect("the producer refuses");
    assert!(
        matches!(
            refusal,
            PlanRootRefusal::Plan(MonitorPlanRefusal::GapExceedsWidth {
                max_reading_gap: 65536,
                width_bits: 16
            })
        ),
        "{refusal:?}"
    );
}

/// An input association differing from the plan's in exactly one component.
fn refused_with(prepared: &Prepared, change: impl FnOnce(&mut Components)) -> InputRefusal {
    let mut run = Run::new(prepared);
    let before = fingerprint(run.monitor.state());
    let layout = prepared.program.layout();
    let mut components = Components {
        counter: layout.counter.to_string(),
        tick_unit: layout.tick_unit.clone(),
        width: layout.counter_width_bits,
        profile: layout.profile.to_string(),
        order: layout.admitted_order.to_vec(),
    };
    change(&mut components);
    let context = InputContext {
        counter: &components.counter,
        tick_unit: &components.tick_unit,
        counter_width_bits: components.width,
        profile: &components.profile,
        admitted_order: &components.order,
        ..context(&prepared.program)
    };
    let captures = run.captures(&[]);
    let failure = run
        .monitor
        .event(Event {
            reading: Reading { context, value: 0 },
            inputs: &captures,
        })
        .err()
        .expect("a foreign association settles nothing");
    assert_eq!(failure.code, MonitorFailureCode::Input);
    assert_eq!(
        fingerprint(run.monitor.state()),
        before,
        "refusal leaves state unchanged"
    );
    failure.input.expect("typed input refusal")
}

/// `Value` has no equality; its debug rendering is a faithful witness of the whole state.
fn fingerprint(value: &Value) -> String {
    format!("{value:?}")
}

struct Components {
    counter: String,
    tick_unit: quire_exact::NodeKey,
    width: u8,
    profile: String,
    order: Vec<u8>,
}

#[trace("TC-200", "FR-276-AC-4")]
#[test]
fn step_3_each_foreign_binding_component_is_refused_before_settling_anything() {
    let prepared = sixteen_bit();
    assert_eq!(
        refused_with(&prepared, |c| c.counter.push_str("-other")),
        InputRefusal::Counter
    );
    assert_eq!(
        refused_with(&prepared, |c| c.tick_unit =
            prepared.program.layout().clause.clone()),
        InputRefusal::Unit
    );
    assert_eq!(
        refused_with(&prepared, |c| c.width = 15),
        InputRefusal::Width
    );
    assert_eq!(
        refused_with(&prepared, |c| c.profile.push_str("-other")),
        InputRefusal::Profile
    );
    assert_eq!(
        refused_with(&prepared, |c| c.order.push(0)),
        InputRefusal::Order
    );
}

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_3_equal_ticks_gain_no_causality_from_ingestion_order() {
    let prepared = sixteen_bit();
    let (req, ack) = (prepared.atom(0), prepared.atom(1));
    // Both ingestion orders of two events at one tick agree with the reference evaluator.
    for (first, second) in [(req, ack), (ack, req)] {
        let rows = assert_agree(
            &prepared,
            &[In::Event(7, vec![first]), In::Event(7, vec![second])],
        );
        assert!(rows.iter().any(|step| !step.is_empty()));
    }
}

// ---------------------------------------------------------------------------
// Step 4: two monitors from one plan.

#[trace("TC-200", "FR-276-AC-6")]
#[test]
fn step_4_two_monitors_from_one_plan_are_independent_and_keep_the_plan() {
    let prepared = prepare(0);
    let program = Rc::clone(&prepared.program);
    // `prepare` consumed the producer root and the driver's preparation objects; only the
    // closed plan remains.
    let mut first = Run::new(&prepared);
    let second = Run::new(&prepared);
    let initial = fingerprint(second.monitor.state());
    assert_eq!(
        fingerprint(first.monitor.state()),
        initial,
        "initialization emits no verdict"
    );
    assert!(first.event(0, true, false).is_empty());
    assert_ne!(fingerprint(first.monitor.state()), initial);
    assert_eq!(
        fingerprint(second.monitor.state()),
        initial,
        "a request in the first changes nothing in the second"
    );
    // Both stay bound to the one admitted plan; the monitor offers no way to rebind.
    assert!(Rc::ptr_eq(first.monitor.program(), &program));
    assert!(Rc::ptr_eq(second.monitor.program(), &program));
    // A later request in the second still sees only its own history.
    let mut second = second;
    let alone = second.event(0, false, true);
    assert!(
        alone.iter().all(|r| r.outcome == "True"),
        "the second monitor never saw the first's request: {alone:?}"
    );
    let together = first.event(1, false, true);
    assert!(
        together
            .iter()
            .any(|r| r.outcome == "True" && r.origin_tick == Integer::zero()),
        "the first monitor settles its own request at tick 0: {together:?}"
    );
}

// ---------------------------------------------------------------------------
// Step 5: admission and build failures.

#[trace("TC-200", "FR-276-AC-7")]
#[test]
fn step_5_a_foreign_source_selection_is_refused_at_driver_admission() {
    let result = try_prepare(
        CLAUSE,
        MICROSECONDS,
        "us",
        &microsecond_target(0),
        |source| *source = SourceIdentity::new("agent-ix", "another-source"),
    );
    // Refused before any RT program exists: there is no Program to build a monitor from.
    let error = result.err().expect("a foreign selected source is refused");
    assert!(
        matches!(
            error,
            quire_driver::monitor::MonitorPreparationError::Authority(_)
                | quire_driver::monitor::MonitorPreparationError::Selection { .. }
        ),
        "{error:?}"
    );
}

#[trace("TC-200", "FR-276-AC-7")]
#[test]
fn step_5_a_starved_initialization_fails_typed_with_no_monitor() {
    let program = prepare(0).program;
    let starved = StepBudget {
        work: 0,
        ..budget()
    };
    let failure = build(BuildPlan {
        program,
        budget: starved,
    })
    .err()
    .expect("no monitor from a starved initialization");
    assert_eq!(failure.code, MonitorFailureCode::Resource);
}

// ---------------------------------------------------------------------------
// Step 6: invalid inputs, progress, and the reference evaluator.

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_6_invalid_event_and_progress_inputs_change_nothing() {
    let prepared = prepare(0);
    let program = Rc::clone(&prepared.program);
    let mut run = Run::new(&prepared);
    run.event(0, true, false);
    let before = fingerprint(run.monitor.state());
    // An event whose captures record omits a required admitted atom value.
    let first = run.capture_fields().remove(0);
    let partial = program.types().record(
        program.layout().inputs.key.clone(),
        vec![(first.as_str(), FieldValue::Present(Value::Boolean(true)))],
    );
    // The typed record constructor itself refuses the omitted field, so a captures value
    // lacking an atom cannot be offered to RT at all; the remaining refusal is a value that
    // is not the admitted captures record.
    assert!(
        matches!(
            &partial,
            Err(refusal) if format!("{refusal:?}").contains("MissingField")
        ),
        "{partial:?}"
    );
    let wrong = Value::Boolean(true);
    let failure = run
        .monitor
        .event(Event {
            reading: Reading {
                context: context(&program),
                value: 5,
            },
            inputs: &wrong,
        })
        .err()
        .expect("missing admitted captures are refused");
    assert_eq!(failure.code, MonitorFailureCode::Input);
    assert_eq!(failure.input, Some(InputRefusal::Values));
    assert_eq!(fingerprint(run.monitor.state()), before);
    // Progress lacking its admitted assertion.
    let layout = program.layout();
    let scope = program
        .types()
        .record(
            layout.scope.key.clone(),
            vec![(
                "subject",
                FieldValue::Present(Value::Integer(Integer::zero())),
            )],
        )
        .expect("scope");
    let boundary = Integer::from(3001_u64);
    let failure = run
        .monitor
        .watermark(Watermark {
            context: context(&program),
            progress_context: &layout.progress_context,
            assertion: b"",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .err()
        .expect("progress without an assertion is refused");
    assert_eq!(failure.input, Some(InputRefusal::MissingProgress));
    assert_eq!(fingerprint(run.monitor.state()), before);
}

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_6_only_admitted_progress_settles_a_deadline_never_a_counter_poll() {
    let prepared = prepare_with(CLAUSE, MICROSECONDS, "us", &microsecond_target(0));
    let rows = assert_agree(
        &prepared,
        &[
            In::Event(0, vec![prepared.atom(0)]),
            // A poll far past the 3000 us deadline decides nothing.
            In::Poll(3500),
            In::Watermark(3500),
        ],
    );
    let ids = prepared.ids;
    assert!(rows[0].is_empty());
    assert!(rows[1].is_empty(), "a poll settles nothing");
    assert_eq!(
        rows[2],
        vec![(0, ids.root, Out::False), (0, ids.interval, Out::False)],
        "admitted progress settles the missed deadline"
    );
}

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_6_reversed_implication_operands_agree_with_the_reference_evaluator() {
    let reversed = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3) implies holds(candidate.versionNumber = 2))
}"#;
    let prepared = prepare_with(reversed, MICROSECONDS, "us", &microsecond_target(0));
    // In source order the acknowledgement atom comes first and the request atom second.
    let (ack_atom, req_atom) = (prepared.atom(0), prepared.atom(1));
    // The root `always` is violated at the acknowledgement event, which holds the eventually
    // but not the request; the request's own `eventually` follows the step-1 boundaries.
    let ids = prepared.ids;
    let eventually: u64 = 2;
    for (ack, request) in [
        (2998, Out::True),
        (3000, Out::Indeterminate),
        (3001, Out::False),
    ] {
        let rows = assert_agree(
            &prepared,
            &[
                In::Event(0, vec![req_atom]),
                In::Event(ack, vec![ack_atom]),
                In::Watermark(ack + 3500),
            ],
        );
        assert_eq!(
            rows[1],
            vec![
                (0, ids.root, Out::False),
                (0, eventually, request),
                (i128::from(ack), eventually, Out::True),
            ],
            "ack at {ack}"
        );
        assert!(rows[0].is_empty() && rows[2].is_empty());
    }
}

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_6_open_and_closed_interval_endpoints_agree_with_the_reference_evaluator() {
    use Out::{False, Indeterminate, Pending, True};
    // Per case: the settled rows of the acknowledgement event and of the watermark, for the
    // root (node 0) and the eventually (node 3). A boundary hit is Indeterminate whichever
    // side is closed (step 1); an open lower endpoint hit by the same tick decides nothing
    // until progress closes it False.
    for (interval, ack, at_ack, at_watermark) in [
        (
            "[0 ms, 3 ms]",
            3000,
            vec![
                (0, 0, Indeterminate),
                (0, 3, Indeterminate),
                (3000, 3, True),
            ],
            vec![],
        ),
        (
            "[0 ms, 3 ms)",
            3000,
            vec![
                (0, 0, Indeterminate),
                (0, 3, Indeterminate),
                (3000, 3, True),
            ],
            vec![],
        ),
        (
            "(0 ms, 3 ms]",
            0,
            vec![(0, 0, Indeterminate), (0, 3, Indeterminate)],
            vec![(0, 3, False)],
        ),
        (
            "(0 ms, 3 ms]",
            3000,
            vec![(0, 0, Indeterminate), (0, 3, Indeterminate)],
            vec![(3000, 3, False)],
        ),
        (
            "[0 ms, 3 ms)",
            2998,
            vec![(0, 3, True), (2998, 3, True)],
            vec![(0, 0, Pending)],
        ),
    ] {
        let clause = format!(
            r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    always (holds(candidate.versionNumber = 2) implies eventually{interval} holds(candidate.versionNumber = 3))
}}"#
        );
        let prepared = prepare_with(&clause, MICROSECONDS, "us", &microsecond_target(0));
        let rows = assert_agree(
            &prepared,
            &[
                In::Event(0, vec![prepared.atom(0)]),
                In::Event(ack, vec![prepared.atom(1)]),
                In::Watermark(ack + 3500),
            ],
        );
        assert_eq!(rows[1], at_ack, "{interval} ack at {ack}");
        assert_eq!(rows[2], at_watermark, "{interval} ack at {ack} watermark");
    }
}
