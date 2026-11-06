// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration-authored one-record QObs producer inputs.

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

pub struct Fixture {
    pub qualified: Box<QualifiedObservation>,
    pub owner: AuthoritySelection,
    pub subject: SubjectSelection,
    pub documents: Vec<(bundle::ComponentRole, Document)>,
}

impl Fixture {
    pub fn new() -> Self {
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
            anchor: Anchor::TimestampNanos(5),
            event_time_nanos: 5,
            ingestion_time_nanos: 6,
            causal_relationship_identity: None,
            clock_identity: id("clock"),
            clock_revision: id("clock-version"),
            clock_uncertainty_nanos: 0,
        };
        record.identity = a::observation::record_identity(&record, limits()).unwrap();
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
                required_member_identities: vec![id("member")],
                observation_sources: vec![id("source")],
                completeness_dependencies: vec![id("completeness")],
                progress_dependencies: vec![id("progress")],
                clock_identity: id("clock"),
                clock_revision: id("clock-version"),
                membership_complete: true,
                closure_identity: Some(id("closure")),
                closure_digest: Some(Digest::new([2; 32])),
                kind: ScopeKind::Window {
                    window_identity: id("window"),
                },
                range: ClockRange::Timestamp {
                    start_nanos: 0,
                    end_nanos: 10,
                },
                members: vec![Member {
                    object_identity: id("member"),
                    record_identity: record.identity.clone(),
                    anchor: record.anchor.clone(),
                }],
            },
            records: vec![record],
            limits: ResourceLimits {
                max_records: 1,
                max_members: 1,
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
            definition_identity: id("observation-owner"),
            definition_revision: id("owner-version"),
            definition_digest: Digest::new([3; 32]),
        };
        let subject = SubjectSelection {
            scope_identity: id("window"),
            population_identity: qualified.scope().population_identity.clone(),
        };
        let mut fixture = Self {
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
        a::observation::Selection::new(self.qualified.records()[0].identity.clone(), self.cutoff())
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
    fn cutoff(&self) -> a::observation::CutoffSelection {
        a::observation::CutoffSelection::new(
            id("clock"),
            id("clock-version"),
            20,
            a::observation::CutoffRule::IngestionTimeAtOrBefore,
            id("cutoff-version"),
        )
    }
    fn interval(&self, start: i128, end: i128) -> a::partial::EventTimeInterval {
        a::partial::EventTimeInterval::new(
            id("clock"),
            id("clock-version"),
            id("unit"),
            start,
            end,
            limits(),
        )
        .unwrap()
    }
    fn boundary(&self) -> TemporalBoundary {
        TemporalBoundary::TimestampedEvent {
            lower_nanos: 0,
            upper_inclusive_nanos: 9,
            carrier_end_exclusive_nanos: 10,
            watermark_nanos: 10,
        }
    }
    fn derive(&self) -> Vec<(bundle::ComponentRole, Document)> {
        use bundle::ComponentRole as R;
        let c = self.context(1, None);
        let record = &self.qualified.records()[0];
        let capture_selection = a::capture::Selection::new(
            id("trigger"),
            Anchor::TimestampNanos(5),
            vec![a::capture::Binding::new(
                id("capture"),
                id("binding"),
                record.identity.clone(),
            )],
        );
        let capture = a::capture::derive(c, &capture_selection, limits()).unwrap();
        let capture_view =
            a::capture::read(capture.bytes(), c, &capture_selection, limits()).unwrap();
        let progress_selection = a::progress::Selection::new(
            a::clock::Selection::new(id("clock"), id("clock-version")),
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
            id("clock"),
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
            vec![a::completeness::Fact::new(
                id("member"),
                Some(record.identity.clone()),
                a::completeness::FactStatus::Available,
            )],
        );
        let completeness = a::completeness::derive(c, &completeness_selection, limits()).unwrap();
        let completeness_view =
            a::completeness::read(completeness.bytes(), c, &completeness_selection, limits())
                .unwrap();
        let proofs = a::activation::AuthorityProofs::new(
            &capture_view,
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
                self.interval(5, 5),
                vec![a::activation::Capture::new(
                    id("capture"),
                    record.identity.clone(),
                    id("boolean"),
                    "true".into(),
                    id("source"),
                )],
            ),
            a::activation::ProgressSelection::new(self.interval(0, 9), self.interval(10, 10)),
            self.interval(5, 5),
            self.interval(5, 9),
            proofs,
        );
        vec![
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
                        id("clock"),
                        id("clock-version"),
                        vec![a::position::Position::new(0, record.identity.clone())],
                    ),
                    limits(),
                )
                .unwrap(),
            ),
            (
                R::Clock,
                a::clock::derive(
                    c,
                    &a::clock::Selection::new(id("clock"), id("clock-version")),
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
                        self.interval(5, 5),
                        6,
                    ),
                    limits(),
                )
                .unwrap(),
            ),
            (R::Capture, capture),
            (R::Progress, progress),
            (
                R::Activation,
                a::activation::derive(c, &activation_selection, limits()).unwrap(),
            ),
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
        ]
    }
}
