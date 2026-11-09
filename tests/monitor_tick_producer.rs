// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-200 through the composed pipeline: QSL source -> producer plan -> driver `prepare_monitor`
//! -> RT `Program` -> bounded event/watermark executor, on a 20-bit, 1 MHz counter.
//!
//! The expected outcomes are the TC-200 expected results, not the output of
//! any other run.
//!
//! Where each TC-200 step / FR-276 acceptance criterion is covered. "Planned" means the cell
//! needs an API or machinery that does not exist; none is built here, and the missing API is
//! named.
//!
//! Covered in this file, through QSL source -> producer -> `prepare_monitor` -> RT:
//! - Steps 1-3, 6-9: the `step_1_` .. `step_9_` tests (AC-1, AC-3, AC-8 .. AC-11).
//! - Step 2 / AC-2: `step_2_once_overflow_...` (22-slot overflow, `Incomplete`, capacity and
//!   setting text), `step_2_a_shorter_window_buffer_overflows_while_its_sibling_stays_live` (a
//!   2 ms and a 10 ms `once` get capacities 6 and 22; the 2 ms obligation is `Incomplete(6)` from
//!   event 6 while the 10 ms sibling keeps its identity and its own `False` rows until it
//!   overflows at event 22; both limit/setting texts asserted) and
//!   `step_2_an_instance_beyond_the_activation_capacity_names_the_activation_limit` (nine
//!   `on each` activations against `max_activations: 8`: the 9th yields `Incomplete(8)` naming
//!   `activation` / `monitor.max_activations`). AC-2 is Covered.
//! - Step 5 / AC-5: `step_5_the_step_api_carries_five_distinct_outcome_kinds_...` asserts, in
//!   one place and against the oracle, that definite (True/False), Indeterminate, Pending,
//!   Incomplete and Failed all occur and each row carries a known obligation id. The fault test
//!   `step_7_..._rebases_the_next_delta` is tagged AC-9 only and is not cited for AC-5.
//!   AC-5 is Covered for the five kinds; "a stop that consumed nothing" is step 8 (AC-13).
//! - Step 4 / AC-6 (partial): independence of two monitors from one plan, and retention of the
//!   layout, the four entries, the declared State/Inputs/Scope types, one capture per atom and
//!   the buffer capacities after the producer root is dropped
//!   (`step_4_the_admitted_program_retains_...`). Atom bodies are not in the Program (the
//!   observation owner evaluates them; the Program carries one capture field per atom).
//! - Step 5 / AC-7 (partial): a foreign source selection and a starved build here; target
//!   premises in `monitor_refusal.rs::every_target_premise_names_its_actual_admitted_value`
//!   (driver) and `monitor_closure.rs::a_tick_unit_of_another_dimension_is_a_target_mismatch`;
//!   `Unsupported` in `monitor_lowering.rs::excluded_source_forms_stop_at_original_operation_without_partial_tuple`
//!   (driver).
//! - Step 6 / AC-8 (partial): invalid inputs, progress, reversed operands, open/closed endpoints,
//!   and `step_6_a_false_activation_guard_starts_no_root_instance` (an `on each ... when` clause).
//! - Step 10 / AC-12: `monitor_program.rs::state_bound_violations_fail_at_the_entry_site_commit_nothing_and_terminate`
//!   over a controlled arena (the real producer never emits an over-bound entry).
//! - Step 11 / AC-13 (partial): `step_11_each_step_budget_field_one_below_its_use_reports_the_exact_stop`
//!   (build and a real event). The exact set of fields drawn on is asserted per phase
//!   (`EXERCISED`: work, stack_frames, scratch_elements, scratch_bytes, integer_bits,
//!   value_occurrences, work_units, result_units); skipped as never drawn on by this clause:
//!   decimal_digits, scale_expansion, text_input_bytes, text_scalars, normalized_scalars,
//!   unit_edges. The event phase samples only the first event (tick 0), not every event kind.
//!   The bisection over each field takes about 12 s per run. Plus the
//!   TC-098 / IT-018 / interface-002 tests in `monitor_program.rs` for allocator denials over a
//!   controlled arena.
//! - Binder identity (narrow claims, not a binder-semantics test):
//!   `authored_binders_are_not_lowered_into_the_program` (atom bodies with `let`s leave the
//!   Program identical to a baseline: they are not lowered) and
//!   `producer_generated_equal_keyed_locals_live_in_distinct_slots_with_distinct_body_paths`
//!   (producer-generated equal-keyed `Local` reads carry distinct body paths, each led by the
//!   driver's `usize::MAX` binder-scope marker, which this test names as a coupling to the
//!   driver's constant). Slot resolution is asserted by
//!   `every_local_read_resolves_to_the_slot_of_its_innermost_enclosing_binder`: it walks every
//!   function body keeping the binders in scope (let, fold, map, filter), and each `Local` read
//!   must name the slot of the innermost enclosing binder with the read's IR name, in the read's
//!   own function, and its IR level must equal the parameter count plus that binder's depth.
//!   The expectation comes from the Program's own binder declarations and nesting plus the names
//!   and levels in the IR keys of the binders and reads; it is not re-derived from the checked
//!   package, and it does not execute the Program: it is Program self-consistency plus the
//!   IR-key level anchor.
//!
//! Planned (the exact missing API):
//! - AC-4, "equal timestamps without an admitted order": the producer always supplies an admitted
//!   order and the runtime refuses any other, so a plan without one cannot be built. Needs a
//!   producer/plan option to omit the order and a runtime that then accepts none.
//! - AC-6 rebinding and subject/atom-body/topology retention as an object: RT exposes no rebind
//!   operation to attempt and no accessor for atom bodies.
//! - AC-7 altered atom, transition, initializer, interval, buffer or activation binding and equal
//!   subformula ids under two subjects: `prepare_monitor` takes only the producer's own
//!   `PlanRoot`, so an altered plan cannot be offered. Needs a driver entry that takes a re-read
//!   package (bytes) together with the original selection to compare. (IR strict admission alone
//!   accepts a changed `counter_width_bits`, `max_reading_gap`, `max_event_rate`, `counter` or
//!   `activation_capacity` literal, and refuses a changed `uncertainty`.)
//! - AC-8 producer-plan executor-invariant failure: no public injection point; the controlled-arena
//!   equivalent is the AC-12 test.
//! - AC-13 / AC-14 allocator denials with producer-admitted `ProgramParts`, and the separate
//!   parameter / result / expression type-validation denials: `prepare_monitor` returns
//!   `Rc<Program>`, never the `ProgramParts`, so `Program::try_new` cannot be wrapped by the
//!   failing allocator with producer identities. Needs a driver entry that returns `ProgramParts`.
//! - AC-13 `PathArenaCapacity` surfaced in `path_capacity`: no public input sizes the path arena
//!   to exhaustion; the nearby test only asserts it is `None`.
//! - Atoms that call a model function, or that form a sequence for `fold`/`map`/`filter`: an atom
//!   calling a function makes the producer fail with `MonitorPlanInvariant("the base package does
//!   not read back")`, and a sequence literal has no expected type in a collection position.

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
    Reading, StepBudget, StepResource, Watermark,
};
use quire_driver::monitor::{prepare_monitor, MonitorPreparationLimits};
use quire_exact::{Cancel, FieldValue, Integer, Rational, ScalarLimits, Value};

/// Native PlanRoot support: the real producer's PlanRoot, owned by this crate.
#[path = "support/monitor_fixture.rs"]
pub mod monitor_fixture;
/// Genuine observation-owner progress admission.
#[path = "support/observation.rs"]
pub mod observation;

const CLAUSE: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}"#;

/// The capture field of an `on each ... when (..)` clause's activation guard; every other
/// capture field is `a<formula node index>`.
const GUARD_CAPTURE: &str = "guard";

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
                let holds = if name == GUARD_CAPTURE {
                    true
                } else {
                    holding.contains(&name[1..].parse::<usize>().expect("a<node index>"))
                };
                (name, FieldValue::Present(Value::Boolean(holds)))
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
        self.event_guarded(tick, holding, true)
    }

    /// As `event_atoms`, with the truth of the activation guard of an `on each` clause.
    fn event_guarded(
        &mut self,
        tick: u64,
        holding: &[usize],
        guard: bool,
    ) -> Result<Vec<Settled>, quire_contract_runtime::monitor::MonitorFailure> {
        let program = Rc::clone(&self.program);
        let values = self
            .capture_fields()
            .into_iter()
            .map(|name| {
                let holds = if name == GUARD_CAPTURE {
                    guard
                } else {
                    holding.contains(&name[1..].parse::<usize>().expect("a<node index>"))
                };
                (name, FieldValue::Present(Value::Boolean(holds)))
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
                // The two limits differ in the payload names RT reports (asserted on the
                // typed result where it matters); both carry the capacity reached.
                TickOutcome::Incomplete { capacity }
                | TickOutcome::ActivationLimit { capacity } => Out::Incomplete(capacity),
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
    /// As `Event`, but the clause's activation guard is false at this event.
    UnguardedEvent(u64, Vec<usize>),
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
        .filter(|name| *name != GUARD_CAPTURE)
        .map(|name| name[1..].parse().expect("a<node index>"))
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
            In::UnguardedEvent(tick, holding) => (
                run.event_guarded(*tick, holding, false)
                    .expect("admitted event"),
                TickStep::Reading {
                    reading: *tick,
                    atoms: atoms.iter().map(|a| (*a, holding.contains(a))).collect(),
                    guard: false,
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

// Partial: AC-2's sibling-live clause is not covered (see the header); only the overflow
// result, its capacity and its setting text are asserted.
#[trace("TC-200", "FR-276-AC-2")]
#[test]
fn step_2_once_overflow_is_incomplete_naming_the_producer_capacity() {
    // 2 events per millisecond is 2/1000 events per microsecond tick of the 1 MHz counter.
    let target = MonitorTarget {
        max_event_rate: Some(Rational::new(Integer::from(2_u64), Integer::from(1000_u64)).unwrap()),
        ..microsecond_target(0)
    };
    let prepared = prepare_with(ONCE, MICROSECONDS, "us", &target);
    let [capacity] = prepared.capacities[..] else {
        panic!("one buffered operator");
    };
    // TC-200 step 2 / FR-276-AC-2: on the 1 MHz counter the producer's buffer is 22 slots
    // (10 ms upper bound plus one tick of cover at 2 events/ms).
    assert_eq!(capacity, 22);
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
    // `capacity` events inside one 10 ms window (100 us apart), then one more: the extra
    // event exceeds the fixed buffer. `p` never holds, so each result depends on the history.
    let inputs: Vec<In> = (0..=capacity)
        .map(|i| In::Event(i * 100, Vec::new()))
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

// An admitted order is in play (the producer's model-step order, carried in the program layout
// and required by the runtime), so this does NOT exercise FR-276-AC-4's "equal timestamps
// without an admitted order" clause. That clause is not covered: the fixture cannot omit the
// order and the runtime refuses any other order. The different outcomes of the two arrival
// orders are therefore expected, not a defect. No FR tag is claimed.
#[test]
fn step_3_equal_tick_events_in_both_arrival_orders_under_the_admitted_order_agree_with_the_reference(
) {
    let prepared = sixteen_bit();
    let (req, ack) = (prepared.atom(0), prepared.atom(1));
    for (first, second) in [(req, ack), (ack, req)] {
        let rows = assert_agree(
            &prepared,
            &[
                In::Event(7, vec![first]),
                In::Event(7, vec![second]),
                In::Watermark(7 + 3500),
            ],
        );
        assert!(
            rows.iter().any(|step| !step.is_empty()),
            "something settles"
        );
    }
}

// ---------------------------------------------------------------------------
// Step 4: two monitors from one plan.

#[trace("TC-200", "FR-276-AC-6")]
#[test]
fn step_4_two_monitors_from_one_plan_have_independent_state() {
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
    // Progress past the 3 ms deadline settles different obligations in the two monitors: only
    // the first has the request at tick 0, so only its root `always` is violated.
    let mut second = second;
    let missed = |node: u64| settled_at_zero(node, "False");
    assert!(second.event(0, false, false).is_empty());
    assert_eq!(
        second.watermark(3500),
        vec![
            settled_at_zero(prepared.ids.root, "Pending"),
            missed(prepared.ids.interval)
        ]
    );
    assert_eq!(
        first.watermark(3500),
        vec![missed(prepared.ids.root), missed(prepared.ids.interval)]
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
    let eventually = ids.interval;
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
    const R: u64 = 0;
    const E: u64 = 1;
    // Per case: the settled rows of the acknowledgement event and of the watermark, for the
    // root (`R`) and the eventually (`E`), mapped to the plan's own ids below. A boundary hit is Indeterminate whichever
    // side is closed (step 1); an open lower endpoint hit by the same tick decides nothing
    // until progress closes it False.
    for (interval, ack, at_ack, at_watermark) in [
        (
            "[0 ms, 3 ms]",
            3000,
            vec![
                (0, R, Indeterminate),
                (0, E, Indeterminate),
                (3000, E, True),
            ],
            vec![],
        ),
        (
            "[0 ms, 3 ms)",
            3000,
            vec![
                (0, R, Indeterminate),
                (0, E, Indeterminate),
                (3000, E, True),
            ],
            vec![],
        ),
        (
            "(0 ms, 3 ms]",
            0,
            vec![(0, R, Indeterminate), (0, E, Indeterminate)],
            vec![(0, E, False)],
        ),
        (
            "(0 ms, 3 ms]",
            3000,
            vec![(0, R, Indeterminate), (0, E, Indeterminate)],
            vec![(3000, E, False)],
        ),
        (
            "[0 ms, 3 ms)",
            2998,
            vec![(0, E, True), (2998, E, True)],
            vec![(0, R, Pending)],
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
        let ids = prepared.ids;
        let plan_ids = |expected: Vec<(i128, u64, Out)>| -> Vec<(i128, u64, Out)> {
            expected
                .into_iter()
                .map(|(tick, node, out)| {
                    (tick, if node == R { ids.root } else { ids.interval }, out)
                })
                .collect()
        };
        assert_eq!(rows[1], plan_ids(at_ack), "{interval} ack at {ack}");
        assert_eq!(
            rows[2],
            plan_ids(at_watermark),
            "{interval} ack at {ack} watermark"
        );
    }
}

// ---------------------------------------------------------------------------
// Steps 7-9: counter fault handling, step budgets and event position.

impl Run {
    /// The named Integer field of the monitor's State.
    fn state_integer(&self, name: &str) -> Integer {
        let Value::Integer(value) = field(&self.program, self.monitor.state(), name) else {
            panic!("integer State field {name}");
        };
        value.clone()
    }
}

#[trace("TC-200", "FR-276-AC-9")]
#[test]
fn step_7_a_gap_fault_drops_only_the_open_instance_and_rebases_the_next_delta() {
    let prepared = sixteen_bit();
    let (req, ack, ids) = (prepared.atom(0), prepared.atom(1), prepared.ids);
    let inputs = [
        // The first reading, whatever its value, is delta 0; its own instance is decided.
        In::Event(65520, vec![ack]),
        // The instance that stays open across the fault.
        In::Event(65530, vec![req]),
        // 65500 is 65506 behind 65530: the gap fault.
        In::Event(65500, vec![]),
        // Differenced from 65500 (delta 10), not from 65530; a third instance starts and decides.
        In::Event(65510, vec![ack]),
    ];
    let rows = assert_agree(&prepared, &inputs);
    let failed = Out::Failed {
        gap: 65506,
        max_reading_gap: 100,
    };
    assert_eq!(
        rows[0],
        vec![(0, ids.interval, Out::True)],
        "decided before"
    );
    assert!(rows[1].is_empty(), "the request is open");
    assert_eq!(
        rows[2],
        vec![(0, ids.root, failed.clone()), (10, ids.interval, failed)],
        "the open `always` and the open request fail; the decided instance is untouched"
    );
    let mut run = Run::new(&prepared);
    let mut nows = Vec::new();
    for input in &inputs {
        let In::Event(tick, holding) = input else {
            unreachable!()
        };
        run.event_atoms(*tick, holding).expect("admitted event");
        nows.push(run.state_integer("now"));
    }
    assert_eq!(nows[0], Integer::zero(), "first reading leaves init's time");
    assert_eq!(nows[1], Integer::from(10_u64));
    assert_eq!(
        nows[3].to_u64().unwrap() - nows[2].to_u64().unwrap(),
        10,
        "65510 is differenced from 65500"
    );
    assert_eq!(rows[3].len(), 1);
    assert_eq!(
        rows[3][0].2,
        Out::True,
        "the instance started after the fault settles"
    );
}

#[trace("TC-200", "FR-276-AC-10")]
#[test]
fn step_8_a_starved_step_budget_stops_unconsumed_and_the_same_event_then_runs() {
    let prepared = sixteen_bit();
    let ack = prepared.atom(1);
    // The independent expectation: a fresh monitor's outcome for the same event.
    let expected = {
        let mut fresh = Run::new(&prepared);
        fresh.event_atoms(65530, &[ack]).expect("admitted")
    };
    assert!(!expected.is_empty());
    let mut run = Run::new(&prepared);
    let before = fingerprint(run.monitor.state());
    run.monitor
        .set_step_budget(StepBudget {
            work: 1,
            ..budget()
        })
        .expect("budget replacement");
    let stop = run
        .event_atoms(65530, &[ack])
        .expect_err("one on_event does not fit in one unit of work");
    assert_eq!(stop.code, MonitorFailureCode::Resource);
    assert!(stop.resource.is_some() && stop.invariant.is_none());
    assert_eq!(fingerprint(run.monitor.state()), before, "state unchanged");
    assert_eq!(run.monitor.baseline(), None, "the event is unconsumed");
    // A resource stop is not terminal: raise the budget and offer the same event.
    run.monitor
        .set_step_budget(budget())
        .expect("budget replacement");
    let actual = run
        .event_atoms(65530, &[ack])
        .expect("accepted after a stop");
    assert_eq!(actual, expected);
    assert_eq!(run.monitor.baseline(), Some(65530));
}

#[trace("TC-200", "FR-276-AC-11")]
#[test]
fn step_9_event_position_counts_ingested_events_through_the_producer_plan() {
    let prepared = sixteen_bit();
    let mut run = Run::new(&prepared);
    let position = |run: &Run| run.state_integer("event_position");
    assert_eq!(position(&run), Integer::zero(), "build");
    run.event_atoms(65530, &[]).expect("baseline event");
    assert_eq!(position(&run), Integer::from(1_u64));
    run.watermark(5);
    run.poll(7).expect("poll");
    assert_eq!(position(&run), Integer::from(1_u64), "watermark and poll");
    run.event_atoms(65500, &[]).expect("gap fault");
    assert_eq!(position(&run), Integer::from(1_u64), "the faulting reading");
    run.event_atoms(65510, &[]).expect("following event");
    assert_eq!(position(&run), Integer::from(2_u64));
}

// ---------------------------------------------------------------------------
// AC-5: five distinct outcome kinds, each carrying its obligation's identity.

fn kind(out: &Out) -> &'static str {
    match out {
        Out::True | Out::False => "definite",
        Out::Pending => "pending",
        Out::Indeterminate => "indeterminate",
        Out::Incomplete(_) => "incomplete",
        Out::Failed { .. } => "failed",
        Out::Other(_) => "other",
    }
}

#[trace("TC-200", "FR-276-AC-5")]
#[test]
fn step_5_the_step_api_carries_five_distinct_outcome_kinds_with_obligation_identities() {
    let mut seen: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut note = |prepared: &Prepared, rows: &[Vec<Row>]| {
        for (_, id, out) in rows.iter().flatten() {
            assert!(
                *id == prepared.ids.root
                    || *id == prepared.ids.interval
                    || prepared.reference.capacities.contains_key(&(*id as usize)),
                "outcome for an unknown obligation {id}"
            );
            *seen.entry(kind(out)).or_default() += 1;
        }
    };
    // Definite and indeterminate: the 3 ms obligation at acknowledgements 2998 and 3000.
    for ack in [2998, 3000] {
        let prepared = prepare(0);
        let rows = assert_agree(
            &prepared,
            &[
                In::Event(0, vec![prepared.atom(0)]),
                In::Event(ack, vec![prepared.atom(1)]),
            ],
        );
        note(&prepared, &rows);
    }
    // Pending: a half-open window whose acknowledgement came in time, until progress closes
    // the root.
    let half_open = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms) holds(candidate.versionNumber = 3))
}"#;
    let prepared = prepare_with(half_open, MICROSECONDS, "us", &microsecond_target(0));
    let rows = assert_agree(
        &prepared,
        &[
            In::Event(0, vec![prepared.atom(0)]),
            In::Event(2998, vec![prepared.atom(1)]),
            In::Watermark(2998 + 3500),
        ],
    );
    note(&prepared, &rows);
    // Incomplete: a buffer past its capacity.
    let target = MonitorTarget {
        max_event_rate: Some(Rational::new(Integer::from(2_u64), Integer::from(1000_u64)).unwrap()),
        ..microsecond_target(0)
    };
    let prepared = prepare_with(ONCE, MICROSECONDS, "us", &target);
    let inputs: Vec<In> = (0..=22).map(|i| In::Event(i * 100, Vec::new())).collect();
    let rows = assert_agree(&prepared, &inputs);
    note(&prepared, &rows);
    // Failed: a reading that goes backward on the 16-bit counter.
    let prepared = sixteen_bit();
    let rows = assert_agree(
        &prepared,
        &[
            In::Event(65530, vec![prepared.atom(0)]),
            In::Event(65500, vec![prepared.atom(1)]),
        ],
    );
    note(&prepared, &rows);
    assert_eq!(
        seen.keys().copied().collect::<Vec<_>>(),
        [
            "definite",
            "failed",
            "incomplete",
            "indeterminate",
            "pending"
        ],
        "{seen:?}"
    );
}

// ---------------------------------------------------------------------------
// Binder identity: do equal-keyed IR variables reach the runtime, and what do they do there?

/// Every lowered expression of a Program, as a comparable record.
fn program_fingerprint(program: &Program) -> Vec<(String, Vec<usize>, String)> {
    (0..)
        .map(quire_contract_runtime::monitor::ExpressionId)
        .map_while(|id| program.expression(id))
        .map(|e| {
            (
                format!("{:?}", e.key),
                e.body_path.clone(),
                format!("{:?}", e.value_type),
            )
        })
        .collect()
}

/// Atom bodies that bind the same names more than once: sibling lets over different values,
/// sibling lets of different types, nested lets, and one expression used both inside and
/// outside a binder. (Folds, maps and filters cannot be written in an atom here: a sequence
/// literal has no expected type in a collection position, and an atom that calls a model
/// function makes the producer fail with `MonitorPlanInvariant("the base package does not
/// read back")`.)
const ADVERSARIAL_ATOMS: [&str; 5] = [
    "holds((let t = candidate.versionNumber + 1 in t * 2) + (let t = candidate.versionNumber + 5 in t * 2) = 18)",
    "holds((let t = candidate.versionNumber > 2 in t) and (let t = candidate.versionNumber + 1 in t > 3))",
    "holds(let t = candidate.versionNumber + 1 in (let u = t * 10 in u + 1) + t = 23)",
    "holds((candidate.versionNumber * 2) + (let t = candidate.versionNumber * 2 in t * t) + (candidate.versionNumber * 2) = 48)",
    "holds((let t = candidate.versionNumber in t) * (let t = candidate.versionNumber + 1 in t) = 6)",
];

fn binder_clause(atom: &str) -> String {
    format!(
        r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    always ({atom} implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}}"#
    )
}

fn binder_prepared(atom: &str) -> Prepared {
    prepare_with(
        &binder_clause(atom),
        MICROSECONDS,
        "us",
        &microsecond_target(0),
    )
}

#[test]
fn authored_binders_are_not_lowered_into_the_program() {
    // The atom bodies are not evaluated here, and the literals in them (18, 23, ...) are dead:
    // the claim is only that the producer's closure which the driver lowers holds its own
    // generated entries, so the Program is identical (key, body path, type of every
    // expression) to the one for a binder-free atom, whatever binders the atom writes.
    let baseline = binder_prepared("holds(candidate.versionNumber = 2)");
    let expected = program_fingerprint(&baseline.program);
    assert!(expected.len() > 100, "a real generated program");
    for atom in ADVERSARIAL_ATOMS {
        let prepared = binder_prepared(atom);
        assert_eq!(
            program_fingerprint(&prepared.program),
            expected,
            "the Program differs for {atom}"
        );
    }
}

#[test]
fn producer_generated_equal_keyed_locals_live_in_distinct_slots_with_distinct_body_paths() {
    use quire_contract_runtime::monitor::Operation;
    // Group the generated Program's local reads by (IR key, owning function). More than one
    // slot under one key is the IR collision (same name, level and type) that the driver
    // separates with BINDER_SCOPED body paths; every read stays distinct and the runtime
    // accepted the Program (it refuses a repeated (key, body_path)).
    let prepared = prepare(0);
    let program = &prepared.program;
    // (slot, body path) of each read, per (IR key, owning function).
    type Reads = Vec<(usize, Vec<usize>)>;
    let mut groups: BTreeMap<(String, usize), Reads> = BTreeMap::new();
    let mut id = 0;
    while let Some(e) = program.expression(quire_contract_runtime::monitor::ExpressionId(id)) {
        if let Operation::Local(local) = &e.operation {
            groups
                .entry((format!("{:?}", e.key), local.function.0))
                .or_default()
                .push((local.position, e.body_path.clone()));
        }
        id += 1;
    }
    let collided: Vec<_> = groups
        .values()
        .filter(|reads| {
            let mut slots: Vec<_> = reads.iter().map(|(slot, _)| *slot).collect();
            slots.sort();
            slots.dedup();
            slots.len() > 1
        })
        .collect();
    for reads in &collided {
        let mut paths: Vec<_> = reads.iter().map(|(_, path)| path.clone()).collect();
        paths.sort();
        paths.dedup();
        assert_eq!(
            paths.len(),
            reads.len(),
            "equal-keyed reads are told apart: {reads:?}"
        );
        // Coupled to the driver's private `BINDER_SCOPED` marker (quire-driver
        // src/monitor/lowering/program.rs), the value `usize::MAX` that leads the body path of
        // an expression reading binder slots. That each read resolves to the slot of its own
        // binder is asserted separately, below.
        assert!(reads
            .iter()
            .all(|(_, path)| path.first() == Some(&usize::MAX)));
    }
    assert!(
        !collided.is_empty(),
        "the generated entries bind the same variable more than once"
    );
}

const EACH_EVENT: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on each (w: Selected::ConfigVersion) {
    eventually holds(candidate.versionNumber = 3)
}"#;

#[trace("TC-200", "FR-276-AC-2")]
#[test]
fn step_2_an_instance_beyond_the_activation_capacity_names_the_activation_limit() {
    // `on each` starts an unbounded `eventually` at every event; the target allows 8 live
    // instances and none is satisfied, so the 9th event's instance is the first beyond it.
    let prepared = prepare_with(EACH_EVENT, MICROSECONDS, "us", &microsecond_target(0));
    assert_eq!(prepared.reference.activation_capacity, 8);
    let inputs: Vec<In> = (0..9).map(|i| In::Event(i * 1000, Vec::new())).collect();
    let rows = assert_agree(&prepared, &inputs);
    assert!(
        rows[..8].iter().all(Vec::is_empty),
        "eight instances start quietly"
    );
    assert_eq!(
        rows[8],
        vec![(8000, prepared.ids.root, Out::Incomplete(8))],
        "the 9th instance is not started and yields the limit"
    );
    // The complete typed result of the same RT run: limit `activation`, setting
    // `monitor.max_activations`, the capacity as the amount.
    let mut run = Run::new(&prepared);
    let mut last = Vec::new();
    for input in &inputs {
        let In::Event(tick, holding) = input else {
            unreachable!()
        };
        last = run.event_atoms(*tick, holding).expect("admitted event");
    }
    let [limited] = &last[..] else {
        panic!("one settled obligation: {last:?}");
    };
    assert_eq!(limited.outcome, "Incomplete");
    assert_eq!(limited.texts, ["activation", "monitor.max_activations"]);
    assert_eq!(limited.payload, [Integer::from(8_u64)]);
}

// ---------------------------------------------------------------------------
// Gaps closed in the follow-up: AC-2 sibling-live and AC-5, false activation, retention,
// and per-field step budgets.

const TWO_WINDOWS: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    once[0 ms, 10 ms] holds(candidate.versionNumber = 1) and once[0 ms, 2 ms] holds(candidate.versionNumber = 2)
}"#;

#[trace("TC-200", "FR-276-AC-2")]
#[trace("TC-200", "FR-276-AC-5")]
#[test]
fn step_2_a_shorter_window_buffer_overflows_while_its_sibling_stays_live() {
    // 2 events per millisecond on the 1 MHz counter; the two `once` windows get different
    // producer capacities, so the 2 ms buffer overflows well before the 10 ms one.
    let target = MonitorTarget {
        max_event_rate: Some(Rational::new(Integer::from(2_u64), Integer::from(1000_u64)).unwrap()),
        ..microsecond_target(0)
    };
    let prepared = prepare_with(TWO_WINDOWS, MICROSECONDS, "us", &target);
    let by_capacity: BTreeMap<u64, u64> = prepared
        .reference
        .capacities
        .iter()
        .map(|(id, capacity)| (*capacity, *id as u64))
        .collect();
    assert_eq!(by_capacity.len(), 2, "two buffers of different capacity");
    let (small, large) = (by_capacity[&6], by_capacity[&22]);
    // Neither atom ever holds, 100 us apart: every window keeps filling.
    let inputs: Vec<In> = (0..30).map(|i| In::Event(i * 100, Vec::new())).collect();
    let rows = assert_agree(&prepared, &inputs);
    for (index, step) in rows.iter().enumerate() {
        let origin = index as i128 * 100;
        let outcome = |id: u64| {
            step.iter()
                .find(|(tick, subformula, _)| *tick == origin && *subformula == id)
                .map(|(_, _, out)| out.clone())
        };
        // The sibling's identity is on every row, with its own outcome, never the other's.
        let large_out = outcome(large).expect("the 10 ms obligation settles at every event");
        let small_out = outcome(small).expect("the 2 ms obligation settles at every event");
        assert_eq!(
            small_out,
            if index < 6 {
                Out::False
            } else {
                Out::Incomplete(6)
            },
            "2 ms buffer at event {index}"
        );
        assert_eq!(
            large_out,
            if index < 22 {
                Out::False
            } else {
                Out::Incomplete(22)
            },
            "10 ms buffer at event {index}"
        );
    }
    // The complete typed results of the same RT run at the last event: each buffer names its
    // own capacity with the same limit and setting.
    let mut run = Run::new(&prepared);
    let mut last = Vec::new();
    for input in &inputs {
        let In::Event(tick, holding) = input else {
            unreachable!()
        };
        last = run.event_atoms(*tick, holding).expect("admitted event");
    }
    for (id, capacity) in [(small, 6_u64), (large, 22)] {
        let limited = last
            .iter()
            .find(|settled| settled.subformula == Integer::from(id))
            .expect("both buffers settle at the last event");
        assert_eq!(limited.outcome, "Incomplete");
        assert_eq!(limited.texts, ["event_buffer", "monitor.max_event_rate"]);
        assert_eq!(limited.payload, [Integer::from(capacity)]);
    }
}

const GUARDED: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on each (w: Selected::ConfigVersion) when (w.versionNumber = 1) {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}"#;

#[trace("TC-200", "FR-276-AC-8")]
#[test]
fn step_6_a_false_activation_guard_starts_no_root_instance() {
    let prepared = prepare_with(GUARDED, MICROSECONDS, "us", &microsecond_target(0));
    let (req, root, interval) = (prepared.atom(0), prepared.ids.root, prepared.ids.interval);
    assert_ne!(root, interval);
    let rows = assert_agree(
        &prepared,
        &[
            // Guard false at tick 0: the `always` is not activated here, though the request
            // holds. (A bounded `eventually` keeps its own buffer and runs at every event.)
            In::UnguardedEvent(0, vec![req]),
            In::Event(1000, vec![req]),
            In::Watermark(1000 + 3500),
        ],
    );
    let flat: Vec<&Row> = rows.iter().flatten().collect();
    assert!(
        flat.iter()
            .all(|(origin, id, _)| !(*origin == 0 && *id == root)),
        "no root instance for the false activation: {flat:?}"
    );
    assert!(
        flat.iter()
            .any(|(origin, id, out)| *origin == 0 && *id == interval && *out == Out::False),
        "the interval operator still ran at tick 0: {flat:?}"
    );
    assert!(
        flat.iter()
            .any(|(origin, id, out)| *origin == 1000 && *id == root && *out == Out::False),
        "the true-guard activation at tick 1000 starts and fails: {flat:?}"
    );
}

#[trace("TC-200", "FR-276-AC-6")]
#[test]
fn step_4_the_admitted_program_retains_its_layout_entries_and_declared_types() {
    // `prepare` has dropped the producer root and every driver object; only the Program stays.
    let prepared = prepare(0);
    let program = &prepared.program;
    let layout = program.layout();
    assert!(!layout.subject.is_empty() && !layout.profile.is_empty());
    assert!(!layout.package_lock.is_empty() && !layout.clock_binding.is_empty());
    assert!(!layout.admitted_order.is_empty() && !layout.progress_context.is_empty());
    assert!(!layout.counter.is_empty());
    let entries = [
        layout.entries.init,
        layout.entries.on_event,
        layout.entries.on_watermark,
        layout.entries.on_gap,
    ];
    for entry in entries {
        assert!(
            program.function(entry).is_some(),
            "entry {entry:?} is retained"
        );
    }
    for binding in [&layout.state, &layout.inputs, &layout.scope] {
        assert!(
            program.types().composite(binding.key.clone()).is_some(),
            "a declared composite for each layout type"
        );
    }
    // One capture field per `holds` atom of the clause, in the plan's own naming.
    let run = Run::new(&prepared);
    let fields = run.capture_fields();
    for atom in &prepared.atoms {
        assert!(
            fields.contains(&format!("a{atom}")),
            "atom {atom} in {fields:?}"
        );
    }
    // The buffers keep the producer's capacities in the plan's order.
    let capacities: Vec<u64> = layout
        .buffers
        .iter()
        .map(|buffer| buffer.capacity.to_u64().unwrap())
        .collect();
    assert_eq!(capacities, prepared.capacities);
}

/// The StepBudget fields: how to set one, and the stop its exhaustion names.
type BudgetField = (&'static str, fn(&mut StepBudget, u64), StepResource);

fn budget_fields() -> Vec<BudgetField> {
    use quire_exact::LimitKind;
    vec![
        ("work", |b, v| b.work = v, StepResource::Work),
        (
            "stack_frames",
            |b, v| b.stack_frames = v as usize,
            StepResource::Stack,
        ),
        (
            "scratch_elements",
            |b, v| b.scratch_elements = v as usize,
            StepResource::ScratchElements,
        ),
        (
            "scratch_bytes",
            |b, v| b.scratch_bytes = v as usize,
            StepResource::ScratchBytes,
        ),
        (
            "integer_bits",
            |b, v| b.scalar.integer_bits = v,
            StepResource::Scalar(LimitKind::IntegerBits),
        ),
        (
            "decimal_digits",
            |b, v| b.scalar.decimal_digits = v,
            StepResource::Scalar(LimitKind::DecimalDigits),
        ),
        (
            "scale_expansion",
            |b, v| b.scalar.scale_expansion = v,
            StepResource::Scalar(LimitKind::ScaleExpansion),
        ),
        (
            "text_input_bytes",
            |b, v| b.scalar.text_input_bytes = v,
            StepResource::Scalar(LimitKind::TextInputBytes),
        ),
        (
            "text_scalars",
            |b, v| b.scalar.text_scalars = v,
            StepResource::Scalar(LimitKind::TextScalars),
        ),
        (
            "normalized_scalars",
            |b, v| b.scalar.normalized_scalars = v,
            StepResource::Scalar(LimitKind::NormalizedScalars),
        ),
        (
            "unit_edges",
            |b, v| b.scalar.unit_edges = v,
            StepResource::Scalar(LimitKind::UnitEdges),
        ),
        (
            "value_occurrences",
            |b, v| b.scalar.value_occurrences = v,
            StepResource::Scalar(LimitKind::ValueOccurrences),
        ),
        (
            "work_units",
            |b, v| b.scalar.work_units = v,
            StepResource::Scalar(LimitKind::WorkUnits),
        ),
        (
            "result_units",
            |b, v| b.scalar.result_units = v,
            StepResource::Scalar(LimitKind::ResultUnits),
        ),
    ]
}

/// Far past anything the producer's program draws on.
const CEILING: u64 = 100_000_000;

/// Build under `step_budget`, then offer one event; the first resource stop either returns.
fn stop_under(
    prepared: &Prepared,
    step_budget: StepBudget,
    during_build: bool,
) -> Option<quire_contract_runtime::monitor::StepResourceStop> {
    let failure = if during_build {
        match build(BuildPlan {
            program: Rc::clone(&prepared.program),
            budget: step_budget,
        }) {
            Ok(_) => return None,
            Err(failure) => failure,
        }
    } else {
        let mut run = Run::new(prepared);
        run.monitor
            .set_step_budget(step_budget)
            .expect("the budget is installable");
        let before = fingerprint(run.monitor.state());
        match run.event_atoms(0, &[prepared.atom(0)]) {
            Ok(_) => return None,
            Err(failure) => {
                assert_eq!(
                    fingerprint(run.monitor.state()),
                    before,
                    "a resource stop commits nothing"
                );
                failure
            }
        }
    };
    assert_eq!(failure.code, MonitorFailureCode::Resource);
    Some(
        *failure
            .resource
            .expect("a resource stop carries its detail"),
    )
}

/// The fields both phases draw on; the others of `budget_fields` are never drawn on by this clause.
const EXERCISED: &[&str] = &[
    "work",
    "stack_frames",
    "scratch_elements",
    "scratch_bytes",
    "integer_bits",
    "value_occurrences",
    "work_units",
    "result_units",
];

#[trace("TC-200", "FR-276-AC-13")]
#[test]
fn step_11_each_step_budget_field_one_below_its_use_reports_the_exact_stop() {
    let prepared = prepare(0);
    for during_build in [true, false] {
        let mut exercised = Vec::new();
        for (name, set, resource) in budget_fields() {
            let with = |value: u64| {
                let mut budget = budget();
                set(&mut budget, value);
                budget
            };
            let ceiling = CEILING;
            assert!(
                stop_under(&prepared, with(ceiling), during_build).is_none(),
                "{name} generous"
            );
            // The smallest value of this field under which the work completes.
            let (mut low, mut high) = (0_u64, ceiling);
            while low < high {
                let middle = low + (high - low) / 2;
                if stop_under(&prepared, with(middle), during_build).is_none() {
                    high = middle;
                } else {
                    low = middle + 1;
                }
            }
            let used = low;
            if used == 0 {
                continue; // this field is never drawn on by this phase
            }
            let stop = stop_under(&prepared, with(used - 1), during_build)
                .unwrap_or_else(|| panic!("{name} one below {used} must stop"));
            assert_eq!(stop.resource, resource, "{name}");
            assert_eq!(stop.limit, Some(u128::from(used - 1)), "{name} limit");
            let requested = u128::from(stop.requested.to_u64().expect("a small request"));
            // The denied request does not fit under the configured limit...
            assert!(stop.spent + requested > u128::from(used - 1), "{name}");
            // ...and, for the cumulative counters, it is exactly the last unit of the use.
            if [
                "work",
                "stack_frames",
                "scratch_elements",
                "scratch_bytes",
                "work_units",
            ]
            .contains(&name)
            {
                assert_eq!(stop.spent + requested, u128::from(used), "{name}");
            }
            exercised.push(name);
        }
        // Exactly these fields are drawn on (the event phase samples only the first event,
        // at tick 0). A field that stops being drawn on, or a new one that starts, fails here.
        assert_eq!(exercised, EXERCISED, "build={during_build}");
    }
}

fn top_level(key: &impl std::fmt::Debug) -> Vec<String> {
    let text = format!("{key:?}");
    let start = text.find('[').expect("a key list") + 1;
    let (mut depth, mut quoted, mut current, mut out) = (0_i32, false, String::new(), Vec::new());
    for c in text[start..].chars() {
        match c {
            '"' => {
                quoted = !quoted;
                current.push(c);
            }
            '(' | '[' if !quoted => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' if !quoted => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                current.push(c);
            }
            ',' if !quoted && depth == 0 => out.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    out
}

/// The name a binder or read declares in its IR key (read from the key's own segments).
fn key_names(key: &impl std::fmt::Debug) -> Vec<String> {
    top_level(key)
        .into_iter()
        .map(|segment| segment.trim_matches('"').to_owned())
        .collect()
}

const LET: &str = r#"["expression","let",<name>,..]"#;
const MAP_FILTER: &str = r#"["expression","collection",<seq>,<item>,..]"#;
const FOLD: &str = r#"["expression","collection",<seq>,<acc>,<item>,..]"#;
const READ: &str = r#"["value","parameter","name",<type>,<name>,"level",<type>,<level>,..]"#;

/// The `index`th segment of a binder or read key, or a failure naming the expected layout.
///
/// The layouts are the IR key shapes of the producer's generated closure, as observed in this
/// Program (not defined in this crate): let `["expression","let",<name>,..]`; map/filter
/// `["expression","collection",<seq>,<item>,..]`; fold
/// `["expression","collection",<seq>,<acc>,<item>,..]`; read
/// `["value","parameter","name",<text type>,<name>,"level",<int type>,<level>,..]`.
fn segment(names: &[String], index: usize, layout: &str) -> String {
    names
        .get(index)
        .unwrap_or_else(|| {
            panic!("key segment {index} missing; expected layout {layout}: {names:?}")
        })
        .clone()
}

/// Every operand of an operation with the binders (name, slot) it brings into scope for that
/// operand. Binder names come from the IR keys of the binding expressions: a `let` key is
/// `["expression","let",<name>,..]`, a map/filter key `["expression","collection",<seq>,<item>,..]`
/// and a fold key `["expression","collection",<seq>,<acc>,<item>,..]`.
fn operands(
    expression: &quire_contract_runtime::monitor::Expression,
) -> Vec<(
    quire_contract_runtime::monitor::ExpressionId,
    Vec<(String, usize)>,
)> {
    use quire_contract_runtime::monitor::Operation as Op;
    let names = key_names(&expression.key);
    let plain = |ids: &[quire_contract_runtime::monitor::ExpressionId]| {
        ids.iter().map(|id| (*id, Vec::new())).collect::<Vec<_>>()
    };
    match &expression.operation {
        Op::Literal(_) | Op::Argument { .. } | Op::Local(_) => Vec::new(),
        Op::Let {
            binding,
            value,
            body,
        } => vec![
            (*value, Vec::new()),
            (*body, vec![(segment(&names, 2, LET), binding.position)]),
        ],
        Op::SequenceFold {
            sequence,
            initial,
            accumulator,
            item,
            body,
        } => vec![
            (*sequence, Vec::new()),
            (*initial, Vec::new()),
            (
                *body,
                vec![
                    (segment(&names, 3, FOLD), accumulator.position),
                    (segment(&names, 4, FOLD), item.position),
                ],
            ),
        ],
        Op::SequenceMap {
            sequence,
            item,
            body,
        } => vec![
            (*sequence, Vec::new()),
            (*body, vec![(segment(&names, 3, MAP_FILTER), item.position)]),
        ],
        Op::SequenceFilter {
            sequence,
            item,
            predicate,
        } => vec![
            (*sequence, Vec::new()),
            (
                *predicate,
                vec![(segment(&names, 3, MAP_FILTER), item.position)],
            ),
        ],
        Op::Call { arguments, .. } => plain(arguments),
        Op::Record { fields } => fields.iter().map(|(_, id)| (*id, Vec::new())).collect(),
        Op::Tuple { positions } | Op::Union { positions, .. } => plain(positions),
        Op::Sequence { elements } => plain(elements),
        Op::SequenceSize { value }
        | Op::SequenceFlatten { value }
        | Op::UnionIs { value, .. }
        | Op::UnionPayload { value, .. }
        | Op::TupleElement { value, .. }
        | Op::RecordField { value, .. }
        | Op::IntegerNegate { value }
        | Op::BooleanNot { value } => plain(&[*value]),
        Op::Equality { left, right, .. }
        | Op::IntegerAdd { left, right }
        | Op::IntegerSubtract { left, right }
        | Op::IntegerMultiply { left, right }
        | Op::IntegerOrder { left, right, .. }
        | Op::BooleanAnd { left, right }
        | Op::BooleanOr { left, right } => plain(&[*left, *right]),
        Op::If {
            condition,
            when_true,
            when_false,
        } => plain(&[*condition, *when_true, *when_false]),
    }
}

#[test]
fn every_local_read_resolves_to_the_slot_of_its_innermost_enclosing_binder() {
    use quire_contract_runtime::monitor::{ExpressionId, FunctionId, Operation};
    let prepared = prepare(0);
    let program = &prepared.program;
    // Walk every function body keeping the binders in scope. The expectation comes from the
    // Program's own binder declarations and nesting, matched to each read by the name its IR
    // key carries (the IR key of a read is name, level and type; the driver's slot assignment
    // is what is under test). This checks the Program against itself plus those IR key
    // names; it does not re-derive scoping from the checked package.
    let mut visited = std::collections::BTreeSet::new();
    let mut resolved = 0_usize;
    // (function, name) -> distinct slots that name resolved to.
    let mut slots: BTreeMap<(usize, String), std::collections::BTreeSet<usize>> = BTreeMap::new();
    let mut function = 0;
    while let Some(owner) = program.function(FunctionId(function)) {
        let mut stack = vec![(owner.body, Vec::<(String, usize)>::new())];
        while let Some((id, scope)) = stack.pop() {
            visited.insert(id.0);
            let expression = program.expression(id).expect("a retained expression");
            if let Operation::Local(local) = &expression.operation {
                let names = key_names(&expression.key);
                let name = &segment(&names, 4, READ);
                let level = segment(&names, 7, READ)
                    .parse::<usize>()
                    .expect("the read key's level segment is an integer");
                let (_, slot) = scope
                    .iter()
                    .rev()
                    .find(|(declared, _)| declared == name)
                    .unwrap_or_else(|| {
                        panic!("read of {name} (expression {}) has no binder", id.0)
                    });
                assert_eq!(local.function, FunctionId(function), "read {} owner", id.0);
                assert_eq!(
                    local.position, *slot,
                    "read {} of {name} (level {level}) resolves to slot {} but its binder declared {slot}",
                    id.0, local.position
                );
                // The IR level of the read is the depth of that binder: parameters, then one
                // per enclosing binder.
                let index = scope
                    .iter()
                    .rposition(|(declared, _)| declared == name)
                    .unwrap();
                assert_eq!(
                    level,
                    owner.parameters.len() + index,
                    "read {} of {name}: IR level against binder depth",
                    id.0
                );
                slots
                    .entry((function, name.clone()))
                    .or_default()
                    .insert(*slot);
                resolved += 1;
            }
            for (child, introduced) in operands(expression) {
                let mut inner = scope.clone();
                inner.extend(introduced);
                stack.push((child, inner));
            }
        }
        function += 1;
    }
    // Non-vacuity: every Local in the arena was reached through a function body and checked,
    // and some name is declared by more than one binder of one function (distinct slots).
    let total = (0..)
        .map_while(|id| program.expression(ExpressionId(id)))
        .filter(|e| matches!(e.operation, Operation::Local(_)))
        .count();
    let reached = visited
        .iter()
        .filter(|id| {
            matches!(
                program.expression(ExpressionId(**id)).unwrap().operation,
                Operation::Local(_)
            )
        })
        .count();
    assert!(
        total > 20 && reached == total,
        "{reached} of {total} reads reached"
    );
    assert!(resolved >= reached);
    assert!(
        slots.values().any(|set| set.len() > 1),
        "some name is bound to different slots in one function"
    );
}
