// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration-authored QObs producer inputs, from one timestamped record to
//! several event-position or fixed-sample records.

#![allow(dead_code)] // Each target uses a different part of the fixture.

use quire_observation::authority::{
    self as a, bundle, AuthoritySelection, Context, Document, History, Limits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, QualifiedSubject,
    ResourceLimits, ScopeKind, ScopeSelection, StaticProducerBundle, SubjectIdentity, SubjectKind,
    ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};
use quire_observation::{ProducerConfiguration, Revision};

pub fn id(text: &str) -> Identity {
    Identity::new(text)
}
fn limits() -> Limits {
    Limits::owner_max()
}

/// How the observation records are anchored on their clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Form {
    /// Nanosecond timestamps on the revision clock.
    Timestamp,
    /// Event positions on the sequence clock.
    Event,
    /// Fixed samples (epoch 0, period 1 ns) on the sequence clock.
    Sample,
}

/// The records a fixture admits: their anchor form, count and first stamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Shape {
    pub form: Form,
    pub count: u64,
    pub start: u64,
    /// Also place the first observation document in the bundle a second time.
    pub duplicate_first: bool,
    /// Stamps the clock range covers beyond the admitted records.
    pub range_extra: u64,
}

impl Shape {
    pub const fn timestamp() -> Self {
        Self {
            form: Form::Timestamp,
            count: 1,
            start: 0,
            duplicate_first: false,
            range_extra: 0,
        }
    }

    /// `count` consecutive stamps from `start`.
    pub const fn positions(form: Form, start: u64, count: u64) -> Self {
        Self {
            form,
            count,
            start,
            duplicate_first: false,
            range_extra: 0,
        }
    }

    fn clock(self) -> &'static str {
        if self.form == Form::Timestamp {
            "clock"
        } else {
            "events"
        }
    }

    fn anchor(self, offset: u64) -> Anchor {
        let stamp = self.start + offset;
        match self.form {
            Form::Timestamp => Anchor::TimestampNanos(5),
            Form::Event => Anchor::EventPosition(stamp),
            Form::Sample => Anchor::FixedSample {
                index: stamp,
                epoch_nanos: 0,
                period_nanos: 1,
            },
        }
    }
}

pub struct Fixture {
    pub shape: Shape,
    pub qualified: Box<QualifiedObservation>,
    pub owner: AuthoritySelection,
    pub subject: SubjectSelection,
    pub documents: Vec<(bundle::ComponentRole, Document)>,
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_owner("observation-owner")
    }

    pub fn with_owner(owner_identity: &str) -> Self {
        Self::build(owner_identity, Shape::timestamp())
    }

    /// Admitted records at event positions or fixed samples for temporal projection.
    pub fn projection(shape: Shape) -> Self {
        Self::build("observation-owner", shape)
    }

    fn build(owner_identity: &str, shape: Shape) -> Self {
        let mut producer = StaticProducerBundle {
            bundle_identity: Some("integration:producer".into()),
            bundle_revision: Some(Revision::producer("original")),
            interface_version: Some(PRODUCER_INTERFACE_VERSION.into()),
            ..StaticProducerBundle::default()
        };
        let configuration_digest = producer
            .canonical_digest_selection()
            .expect("configuration value");
        producer.configuration = Some(ProducerConfiguration {
            configuration_identity: "integration:configuration".into(),
            digest: configuration_digest,
        });
        producer.digest = Some(
            producer
                .canonical_digest_selection()
                .expect("producer bytes"),
        );
        let producer = producer.admit().expect("real producer admission");
        let subject = QualifiedSubject::new(
            &producer,
            SubjectKind::new(b"integration:Sensor".to_vec()).unwrap(),
            SubjectIdentity::new(b"integration:sensor:one".to_vec()).unwrap(),
        );
        let clock = shape.clock();
        let records: Vec<AdmittedRecord> = (0..shape.count)
            .map(|offset| {
                let mut record = AdmittedRecord {
                    identity: id("candidate"),
                    binding_identity: id("binding"),
                    source_identity: id("source"),
                    schema_identity: id("schema"),
                    subject: subject.clone(),
                    signal_identity: id("signal"),
                    trigger_identity: id("trigger"),
                    unit: id("unit"),
                    value: ValueState::Present {
                        value_type: id("boolean"),
                        canonical_value: "true".into(),
                    },
                    visibility: Visibility::External,
                    anchor: shape.anchor(offset),
                    // Positions double as event times so intervals follow the stamps.
                    event_time_nanos: if shape.form == Form::Timestamp {
                        5
                    } else {
                        i128::from(shape.start + offset)
                    },
                    ingestion_time_nanos: if shape.form == Form::Timestamp {
                        6
                    } else {
                        i128::from(shape.start + offset) + 1
                    },
                    causal_relationship_identity: None,
                    clock_identity: id(clock),
                    clock_revision: id("clock-version"),
                    clock_uncertainty_nanos: 0,
                };
                record.identity = a::observation::record_identity(&record, limits()).unwrap();
                record
            })
            .collect();
        let (scope_kind, scope_identity) = if shape.form == Form::Timestamp {
            (
                ScopeKind::Window {
                    window_identity: id("window"),
                },
                id("window"),
            )
        } else {
            (
                ScopeKind::Snapshot {
                    snapshot_identity: id("snapshot"),
                },
                id("snapshot"),
            )
        };
        let range = match shape.form {
            Form::Timestamp => ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 10,
            },
            Form::Event => ClockRange::EventPosition {
                start: shape.start,
                end_exclusive: shape.start + shape.count + shape.range_extra,
            },
            Form::Sample => ClockRange::FixedSample {
                start: shape.start,
                end_exclusive: shape.start + shape.count + shape.range_extra,
                epoch_nanos: 0,
                period_nanos: 1,
            },
        };
        let members: Vec<Member> = records
            .iter()
            .enumerate()
            .map(|(index, record)| Member {
                object_identity: id(&if index == 0 {
                    "member".to_owned()
                } else {
                    format!("member-{index}")
                }),
                record_identity: record.identity.clone(),
                anchor: record.anchor,
            })
            .collect();
        let count = usize::try_from(shape.count).unwrap();
        let mut request = AdmissionRequest {
            package: PackageSelection {
                format: NATIVE_LINKED_PACKAGE_FORMAT.into(),
                identity: id("package"),
                revision: id("package-version"),
                digest: Digest::new([1; 32]),
            },
            binding: ObservationBinding {
                identity: id("binding"),
                source_identity: id("source"),
                schema_identity: id("schema"),
                signal_identity: id("signal"),
                trigger_identity: id("trigger"),
                unit: id("unit"),
                subject_kind: subject.kind().clone(),
                required: true,
            },
            expected_subject: subject,
            relationships: vec![],
            required_relationships: vec![],
            scope: ScopeSelection {
                population_identity: id("candidate"),
                membership_rule_identity: id("candidate"),
                membership_digest: Digest::new([0; 32]),
                membership_document: vec![],
                required_member_identities: members
                    .iter()
                    .map(|member| member.object_identity.clone())
                    .collect(),
                observation_sources: vec![id("source")],
                completeness_dependencies: vec![id("completeness")],
                progress_dependencies: vec![id("progress")],
                clock_identity: id(clock),
                clock_revision: id("clock-version"),
                membership_complete: true,
                closure_identity: Some(id("closure")),
                closure_digest: Some(Digest::new([2; 32])),
                kind: scope_kind,
                range,
                members,
            },
            records,
            limits: ResourceLimits {
                max_records: count,
                max_members: count,
                max_relationships: 0,
                max_required_relationships: 0,
            },
            producer,
        };
        a::population::assign_request_identities(&mut request, limits()).unwrap();
        let qualified = match admit(request) {
            AdmissionOutcome::Available { observation } => observation,
            other => panic!("real admission: {other:?}"),
        };
        let owner = AuthoritySelection {
            definition_identity: id(owner_identity),
            definition_revision: id("owner-version"),
            definition_digest: Digest::new([3; 32]),
        };
        let subject = SubjectSelection {
            scope_identity,
            population_identity: qualified.scope().population_identity.clone(),
        };
        let mut fixture = Self {
            shape,
            qualified,
            owner,
            subject,
            documents: vec![],
        };
        fixture.documents = fixture.derive();
        fixture
    }

    pub fn context<'a>(&'a self, revision: u64, predecessor: Option<&'a Document>) -> Context<'a> {
        Context::new(
            History::batch(&self.qualified),
            &self.owner,
            &self.subject,
            revision,
            predecessor,
        )
    }
    pub fn observation_selection(&self) -> a::observation::Selection {
        self.observation_selection_at(0)
    }
    /// The selection of the `index`th admitted record.
    pub fn observation_selection_at(&self, index: usize) -> a::observation::Selection {
        a::observation::Selection::new(
            self.qualified.records()[index].identity.clone(),
            self.cutoff(),
        )
    }
    /// The owner view of the `index`th record's observation document.
    pub fn observation_at(&self, index: usize) -> a::observation::View {
        a::observation::read(
            self.documents[index].1.bytes(),
            self.context(1, None),
            &self.observation_selection_at(index),
            limits(),
        )
        .unwrap()
    }
    pub fn observation(
        &self,
        document: &Document,
        revision: u64,
        prior: Option<&Document>,
    ) -> a::observation::View {
        a::observation::read(
            document.bytes(),
            self.context(revision, prior),
            &self.observation_selection(),
            limits(),
        )
        .unwrap()
    }
    pub fn observation_document(&self) -> &Document {
        &self.documents[0].1
    }
    pub fn selection(&self, replacement: Option<&Document>) -> bundle::Selection {
        let components = self
            .documents
            .iter()
            .map(|(role, document)| {
                bundle::Component::from_document(
                    *role,
                    if *role == bundle::ComponentRole::Observation {
                        replacement.unwrap_or(document)
                    } else {
                        document
                    },
                )
                .unwrap()
            })
            .collect();
        let replacements = replacement
            .map(|new| {
                bundle::Replacement::new(
                    &bundle::Component::from_document(
                        bundle::ComponentRole::Observation,
                        self.observation_document(),
                    )
                    .unwrap(),
                    &bundle::Component::from_document(bundle::ComponentRole::Observation, new)
                        .unwrap(),
                )
                .unwrap()
            })
            .into_iter()
            .collect();
        bundle::Selection::new(components, replacements)
    }
    pub fn availability_document(&self) -> &Document {
        &self.documents.last().unwrap().1
    }
    pub fn replace_availability(
        &self,
        revision: u64,
        prior: &Document,
    ) -> (Document, bundle::Selection) {
        let document = a::availability::derive(
            self.context(revision, Some(prior)),
            &a::availability::Selection::new(
                vec![id("result")],
                vec![id("result")],
                a::availability::DependencyState::Available,
                a::availability::DependencyState::Available,
            ),
            limits(),
        )
        .unwrap();
        let role = bundle::ComponentRole::Availability;
        let components = self
            .documents
            .iter()
            .map(|(item_role, item)| {
                bundle::Component::from_document(
                    *item_role,
                    if *item_role == role { &document } else { item },
                )
                .unwrap()
            })
            .collect();
        let replacement = bundle::Replacement::new(
            &bundle::Component::from_document(role, prior).unwrap(),
            &bundle::Component::from_document(role, &document).unwrap(),
        )
        .unwrap();
        (
            document,
            bundle::Selection::new(components, vec![replacement]),
        )
    }
    pub fn cutoff(&self) -> a::observation::CutoffSelection {
        a::observation::CutoffSelection::new(
            id(self.shape.clock()),
            id("clock-version"),
            20,
            a::observation::CutoffRule::IngestionTimeAtOrBefore,
            id("cutoff-version"),
        )
    }
    fn interval(&self, start: i128, end: i128) -> a::partial::EventTimeInterval {
        a::partial::EventTimeInterval::new(
            id(self.shape.clock()),
            id("clock-version"),
            id("unit"),
            start,
            end,
            limits(),
        )
        .unwrap()
    }
    /// The progress boundary over every record, with the given watermark.
    pub fn boundary_at(&self, watermark: u64) -> TemporalBoundary {
        let (lower, last, end) = (
            self.shape.start,
            self.shape.start + self.shape.count + self.shape.range_extra - 1,
            self.shape.start + self.shape.count + self.shape.range_extra,
        );
        match self.shape.form {
            Form::Timestamp => TemporalBoundary::TimestampedEvent {
                lower_nanos: 0,
                upper_inclusive_nanos: 9,
                carrier_end_exclusive_nanos: 10,
                watermark_nanos: 10,
            },
            Form::Event => TemporalBoundary::EventPosition {
                lower,
                upper_inclusive: last,
                carrier_end_exclusive: end,
                watermark,
            },
            Form::Sample => TemporalBoundary::FixedSample {
                lower,
                upper_inclusive: last,
                carrier_end_exclusive: end,
                watermark,
            },
        }
    }
    fn boundary(&self) -> TemporalBoundary {
        self.boundary_at(self.shape.start + self.shape.count + self.shape.range_extra)
    }
    /// The event time every record's activation is pinned to.
    fn point(&self) -> i128 {
        if self.shape.form == Form::Timestamp {
            5
        } else {
            i128::from(self.shape.start)
        }
    }
    /// The progress interval: first event time, last event time, carrier end.
    fn span(&self) -> (i128, i128, i128) {
        if self.shape.form == Form::Timestamp {
            (0, 9, 10)
        } else {
            let (start, count) = (
                i128::from(self.shape.start),
                i128::from(self.shape.count + self.shape.range_extra),
            );
            (start, start + count - 1, start + count)
        }
    }
    fn derive(&self) -> Vec<(bundle::ComponentRole, Document)> {
        use bundle::ComponentRole as R;
        let c = self.context(1, None);
        let record = &self.qualified.records()[0];
        let capture_selection = a::capture::Selection::new(
            id("trigger"),
            record.anchor,
            self.qualified
                .records()
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    a::capture::Binding::new(
                        id(&if index == 0 {
                            "capture".to_owned()
                        } else {
                            format!("capture-{index}")
                        }),
                        id("binding"),
                        item.identity.clone(),
                    )
                })
                .collect(),
        );
        // A capture environment has one anchor, so only a single-record fixture has one.
        let single = self.shape.count == 1;
        let capture = single.then(|| a::capture::derive(c, &capture_selection, limits()).unwrap());
        let capture_view = capture.as_ref().map(|document| {
            a::capture::read(document.bytes(), c, &capture_selection, limits()).unwrap()
        });
        let progress_selection = a::progress::Selection::new(
            a::clock::Selection::new(id(self.shape.clock()), id("clock-version")),
            vec![id("source")],
            self.boundary(),
            OpenClosed::Closed,
            id("trigger"),
            self.cutoff(),
            id("restoration"),
        );
        let progress = a::progress::derive(c, &progress_selection, limits()).unwrap();
        let progress_view =
            a::progress::read(progress.bytes(), c, &progress_selection, limits()).unwrap();
        let closure_selection = a::closure::Selection::new(
            id(self.shape.clock()),
            id("clock-version"),
            vec![id("source")],
            self.boundary(),
            OpenClosed::Closed,
        );
        let closure = a::closure::derive(c, &closure_selection, limits()).unwrap();
        let closure_view =
            a::closure::read(closure.bytes(), c, &closure_selection, limits()).unwrap();
        let completeness_selection = a::completeness::Selection::new(
            id("boundary"),
            self.qualified
                .records()
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    a::completeness::Fact::new(
                        id(&if index == 0 {
                            "member".to_owned()
                        } else {
                            format!("member-{index}")
                        }),
                        Some(item.identity.clone()),
                        a::completeness::FactStatus::Available,
                    )
                })
                .collect(),
        );
        let completeness = a::completeness::derive(c, &completeness_selection, limits()).unwrap();
        let completeness_view =
            a::completeness::read(completeness.bytes(), c, &completeness_selection, limits())
                .unwrap();
        let activation = capture_view.as_ref().map(|capture_view| {
            let proofs = a::activation::AuthorityProofs::new(
                capture_view,
                Some(&progress_view),
                Some(&closure_view),
                Some(&completeness_view),
            )
            .unwrap();
            let activation_selection = a::activation::Selection::new(
                a::activation::ActivationSelection::new(
                    id("obligation"),
                    id("binding"),
                    id("trigger"),
                    record.identity.clone(),
                    self.interval(self.point(), self.point()),
                    vec![a::activation::Capture::new(
                        id("capture"),
                        record.identity.clone(),
                        id("boolean"),
                        "true".into(),
                        id("source"),
                    )],
                ),
                a::activation::ProgressSelection::new(
                    self.interval(self.span().0, self.span().1),
                    self.interval(self.span().2, self.span().2),
                ),
                self.interval(self.point(), self.point()),
                self.interval(self.point(), self.span().1),
                proofs,
            );
            a::activation::derive(c, &activation_selection, limits()).unwrap()
        });
        let mut documents = vec![
            (
                R::Observation,
                a::observation::derive(c, &self.observation_selection(), limits()).unwrap(),
            ),
            (R::Population, a::population::derive(c, limits()).unwrap()),
            (
                R::Position,
                a::position::derive(
                    c,
                    &a::position::Selection::new(
                        id("ledger"),
                        id(self.shape.clock()),
                        id("clock-version"),
                        self.qualified
                            .records()
                            .iter()
                            .zip(0u64..)
                            .map(|(item, offset)| {
                                // Ledger positions are zero-based and contiguous.
                                a::position::Position::new(offset, item.identity.clone())
                            })
                            .collect(),
                    ),
                    limits(),
                )
                .unwrap(),
            ),
            (
                R::Clock,
                a::clock::derive(
                    c,
                    &a::clock::Selection::new(id(self.shape.clock()), id("clock-version")),
                    limits(),
                )
                .unwrap(),
            ),
            (
                R::Partial,
                a::partial::derive(
                    c,
                    &a::partial::Selection::new(
                        record.identity.clone(),
                        a::partial::PossibilitySet::new(
                            false,
                            true,
                            a::partial::ValueReason::Known,
                        )
                        .unwrap(),
                        self.interval(self.point(), self.point()),
                        self.point() + 1,
                    ),
                    limits(),
                )
                .unwrap(),
            ),
            (R::Progress, progress),
            (R::Closure, closure),
            (R::Completeness, completeness),
            (
                R::Availability,
                a::availability::derive(
                    c,
                    &a::availability::Selection::new(
                        vec![id("result")],
                        vec![id("result")],
                        a::availability::DependencyState::Available,
                        a::availability::DependencyState::Available,
                    ),
                    limits(),
                )
                .unwrap(),
            ),
        ];
        // Every further record contributes its own observation document, and a
        // duplicated first document places one observation identity in the bundle twice.
        for index in 1..self.qualified.records().len() {
            documents.insert(
                index,
                (
                    R::Observation,
                    a::observation::derive(c, &self.observation_selection_at(index), limits())
                        .unwrap(),
                ),
            );
        }
        if let Some(document) = capture {
            let at = documents
                .iter()
                .position(|(role, _)| *role == R::Partial)
                .unwrap()
                + 1;
            documents.insert(at, (R::Capture, document));
        }
        if let Some(document) = activation {
            let at = documents
                .iter()
                .position(|(role, _)| *role == R::Progress)
                .unwrap()
                + 1;
            documents.insert(at, (R::Activation, document));
        }
        if self.shape.duplicate_first {
            let first = documents[0].1.clone();
            documents.insert(1, (R::Observation, first));
        }
        documents
    }
}
