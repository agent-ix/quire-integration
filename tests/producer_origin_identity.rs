// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-016 / TC-096: runtime identities originate in QSL source admitted by IR.

mod support;

use std::{collections::BTreeMap, fs, process::Command, sync::OnceLock};

use ix_trace_rs::trace;
use qsl_foundation::node_wire::node_key_from_json;
use quire_contract_ir::{
    read_checked_package, CheckedNodeId, CheckedNodeOwner, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedSemanticNodeV2,
    CheckedTerm,
};
use quire_exact::{
    ChargePoint, IllTypedCause, Integer, IntegerInterval, LimitKind, Meter, NodeKey, Outcome,
    Quantity, Rational, RoundingMode, ScalarLimits, Undefined, UnitId,
};
use quire_semantic_value::{
    enumeration::EnumDeclaration,
    quantity::{
        convert_quantity, evaluate_quantity, ConvertedValue, HandoffRefusal, QuantityOperation,
        QuantityRefusal, QuantityTarget, QuantityUnit, UnitTable,
    },
    semantic_node::SemanticGraphCause,
    unit::{CompoundUnitCause, DimensionNode, NominalDeclaration, UnitGraph, UnitNode},
};
use serde_json::Value;

/// The path given to the producer is fixed and relative to its working directory.
const SOURCE_PATH: &str = "program.native";

/// Source authored for this integration test; no checked package or semantic
/// identity is stored as a fixture.
const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    enum Phase { OPEN, SHUT }\n\
    dimension Length;\n\
    unit m : Length = rational(1, 1);\n\
    unit cm : Length = rational(1, 100) * m;\n\
    unit km : Length = rational(1000, 1) * m;\n\
    dimension Area = Length^2;\n\
    unit m2 : Area = rational(1, 1);\n\
    dimension Time;\n\
    unit s : Time = rational(1, 1);\n\
    dimension Temperature;\n\
    unit K : Temperature = rational(1, 1);\n\
    unit degC : Temperature = rational(1, 1) * K + rational(5463, 20);\n\
    function keep using v(x: Int[0, 9]): Int[0, 9] pure { x }\n";

/// Run the lock-selected QSL producer and the IR reader. Either missing tool
/// or refused package fails the test at the source-to-reader seam.
fn admitted() -> &'static CheckedPackageV2 {
    static PACKAGE: OnceLock<CheckedPackageV2> = OnceLock::new();
    PACKAGE.get_or_init(|| emit_and_read(SOURCE))
}

fn emit_and_read(source_text: &str) -> CheckedPackageV2 {
    let directory = tempfile::tempdir().expect("test source directory");
    let source = directory.path().join(SOURCE_PATH);
    fs::write(&source, source_text).expect("write authored source");
    let manifest = support::qsl_root().join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .current_dir(directory.path())
        .env(
            "CARGO_TARGET_DIR",
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/qsl-producer"),
        )
        .arg("run")
        .arg("--locked")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest)
        .args([
            "--example",
            "emit_checked_v2",
            "--",
            "--package-name",
            "agent-ix/it-016",
        ])
        .arg(SOURCE_PATH)
        .output()
        .expect("the lock-selected QSL producer runs");
    assert!(
        output.status.success(),
        "emit_checked_v2 refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let wire: Value = serde_json::from_slice(&output.stdout).expect("producer emits JSON");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("producer lock has feature list")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    let package = match read_checked_package(
        &output.stdout,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("IR refused QSL's package: {other:?}"),
    };
    eprintln!(
        "producer=cargo run --locked --example emit_checked_v2; application={}; \
         source-revision=Cargo.lock; outcome=AdmittedV2({})",
        env!("CARGO_PKG_VERSION"),
        package.name()
    );
    package
}

/// Select the exact admitted declaration, refusing absence or ambiguity.
fn declaration_node<'a>(package: &'a CheckedPackageV2, name: &str) -> &'a CheckedSemanticNodeV2 {
    let matches: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .last()
                    .is_some_and(|part| &**part == name)
            })
        })
        .collect();
    let [node] = matches.as_slice() else {
        panic!(
            "expected one admitted declaration {name}, found {}",
            matches.len()
        );
    };
    assert!(
        matches!(
            &node.owner,
            Some(CheckedNodeOwner::Source { authority, identity })
                if authority.as_ref() == "local" && identity.as_ref() == "source:program.native"
        ),
        "{name} retains the producer's source owner: {:?}",
        node.owner
    );
    node
}

fn declaration_id(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    declaration_node(package, name).node_id.clone()
}

/// Carry a key that came from the admitted IR graph into QSV's structural
/// identity representation. This does not construct an identity from parts.
fn admitted_key(id: &CheckedNodeId) -> NodeKey {
    let wire = serde_json::to_value(id).expect("admitted node ID serializes");
    node_key_from_json(&wire).expect("admitted structural node ID is a NodeKey")
}

fn nominal_wire(node: &CheckedSemanticNodeV2) -> Value {
    assert!(matches!(&node.body, CheckedTerm::NominalBody(_)));
    serde_json::to_value(&node.body).expect("admitted nominal body serializes")
}

fn body_id(body: &Value, member: &str) -> CheckedNodeId {
    serde_json::from_value(body[member].clone())
        .unwrap_or_else(|error| panic!("admitted {member} ID: {error}"))
}

fn body_rational(body: &Value, member: &str) -> Rational {
    let ratio = &body[member];
    let parse = |part: &str| {
        ratio[part]
            .as_str()
            .unwrap_or_else(|| panic!("{member}.{part} is a canonical integer"))
            .parse::<Integer>()
            .unwrap_or_else(|error| panic!("{member}.{part}: {error}"))
    };
    Rational::new(parse("numerator"), parse("denominator"))
        .expect("admitted rational has nonzero denominator")
}

/// TC-033's enum declaration checks consume a producer-origin declaration key.
#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-4")]
fn admitted_enum_declaration_retains_member_order_and_typed_refusals() {
    let package = admitted();
    let phase_node = declaration_node(package, "Phase");
    let phase = admitted_key(&phase_node.node_id);
    let CheckedTerm::NominalBody(body) = &phase_node.body else {
        panic!("admitted Phase has a nominal body: {:?}", phase_node.body);
    };
    let (ordered, cases) = body.enum_details().expect("Phase is an admitted enum");
    let declaration = EnumDeclaration::new(
        phase.clone(),
        ordered,
        cases.iter().map(|case| case.to_string()).collect(),
    )
    .expect("the admitted enum cases are canonical");
    assert_eq!(declaration.key(), phase);
    assert_eq!(declaration.members(), ["OPEN", "SHUT"]);
    let mut member_cases = Vec::new();
    for node in &package.graph().nodes {
        if node.semantic_form.as_ref() != "enum_value" || node.semantic_type != phase_node.node_id {
            continue;
        }
        let CheckedTerm::Literal { value, .. } = &node.body else {
            panic!("admitted enum member is a literal: {:?}", node.body);
        };
        let case = value.as_str().expect("admitted case name");
        let member_key = admitted_key(&node.node_id);
        let member = declaration
            .member(case, member_key.clone())
            .expect("reader-admitted case belongs to its declaration");
        assert_eq!(member.declaration(), phase);
        assert_eq!(member.member(), member_key);
        member_cases.push(case);
    }
    member_cases.sort();
    assert_eq!(member_cases, ["OPEN", "SHUT"]);
    let refusal = declaration
        .member("ABSENT", phase)
        .expect_err("a case outside the declaration refuses");
    assert_eq!(refusal.cause, SemanticGraphCause::UndeclaredCase);
}

/// TC-033's unit consumer uses the exact dimension and unit identities IR
/// admitted from emitted source, including the target edge between two units.
struct AdmittedUnits {
    graph: UnitGraph,
    dimension: NodeKey,
    metre: NodeKey,
    centimetre: NodeKey,
    kilometre: NodeKey,
    second: NodeKey,
    kelvin: NodeKey,
    celsius: NodeKey,
    square_metre: NodeKey,
}

fn admitted_units() -> AdmittedUnits {
    admitted_units_from_package(admitted())
}

fn admitted_units_from_package(package: &CheckedPackageV2) -> AdmittedUnits {
    let dimension_id = declaration_id(package, "Length");
    let dimension = admitted_key(&dimension_id);
    let time_id = declaration_id(package, "Time");
    let time = admitted_key(&time_id);
    let metre_id = declaration_id(package, "m");
    let metre = admitted_key(&metre_id);
    let centimetre_id = declaration_id(package, "cm");
    let centimetre = admitted_key(&centimetre_id);
    let kilometre_id = declaration_id(package, "km");
    let kilometre = admitted_key(&kilometre_id);
    let second_id = declaration_id(package, "s");
    let second = admitted_key(&second_id);
    let temperature_id = declaration_id(package, "Temperature");
    let temperature = admitted_key(&temperature_id);
    let kelvin_id = declaration_id(package, "K");
    let kelvin = admitted_key(&kelvin_id);
    let celsius_id = declaration_id(package, "degC");
    let celsius = admitted_key(&celsius_id);
    let area_id = declaration_id(package, "Area");
    let area = admitted_key(&area_id);
    let square_metre_id = declaration_id(package, "m2");
    let square_metre = admitted_key(&square_metre_id);
    let dimension_body = nominal_wire(declaration_node(package, "Length"));
    assert_eq!(dimension_body["term"], "dimension");
    assert_eq!(dimension_body["terms"], serde_json::json!([]));
    let metre_body = nominal_wire(declaration_node(package, "m"));
    let centimetre_body = nominal_wire(declaration_node(package, "cm"));
    let kilometre_body = nominal_wire(declaration_node(package, "km"));
    let second_body = nominal_wire(declaration_node(package, "s"));
    let kelvin_body = nominal_wire(declaration_node(package, "K"));
    let celsius_body = nominal_wire(declaration_node(package, "degC"));
    let area_body = nominal_wire(declaration_node(package, "Area"));
    let square_metre_body = nominal_wire(declaration_node(package, "m2"));
    assert_eq!(metre_body["term"], "unit");
    assert_eq!(centimetre_body["term"], "unit");
    assert_eq!(body_id(&metre_body, "dimension_node_id"), dimension_id);
    assert_eq!(body_id(&centimetre_body, "dimension_node_id"), dimension_id);
    assert_eq!(body_id(&kilometre_body, "dimension_node_id"), dimension_id);
    assert_eq!(body_id(&second_body, "dimension_node_id"), time_id);
    assert_eq!(body_id(&kelvin_body, "dimension_node_id"), temperature_id);
    assert_eq!(body_id(&celsius_body, "dimension_node_id"), temperature_id);
    assert_eq!(body_id(&square_metre_body, "dimension_node_id"), area_id);
    assert!(metre_body["target_unit_node_id"].is_null());
    assert_eq!(body_id(&centimetre_body, "target_unit_node_id"), metre_id);
    assert_eq!(body_id(&kilometre_body, "target_unit_node_id"), metre_id);
    assert!(second_body["target_unit_node_id"].is_null());
    assert!(kelvin_body["target_unit_node_id"].is_null());
    assert_eq!(body_id(&celsius_body, "target_unit_node_id"), kelvin_id);
    assert!(square_metre_body["target_unit_node_id"].is_null());
    let area_terms = area_body["terms"]
        .as_array()
        .expect("admitted Area has dimension terms")
        .iter()
        .map(|term| {
            let id: CheckedNodeId = serde_json::from_value(term["dimension_node_id"].clone())
                .expect("admitted base dimension ID");
            let exponent = term["exponent"]
                .as_str()
                .expect("canonical exponent")
                .parse::<Integer>()
                .expect("admitted integer exponent");
            (admitted_key(&id), exponent)
        })
        .collect();
    let dimension_pairs = [
        (
            dimension.clone(),
            DimensionNode::checked(Vec::new(), NominalDeclaration::default())
                .expect("base dimension"),
        ),
        (
            time.clone(),
            DimensionNode::checked(Vec::new(), NominalDeclaration::default())
                .expect("second base dimension"),
        ),
        (
            temperature.clone(),
            DimensionNode::checked(Vec::new(), NominalDeclaration::default())
                .expect("temperature base dimension"),
        ),
        (
            area.clone(),
            DimensionNode::checked(area_terms, NominalDeclaration::default())
                .expect("admitted derived area"),
        ),
    ];
    let dimensions = BTreeMap::from(dimension_pairs);
    let unit_pairs = [
        (
            metre.clone(),
            UnitNode::checked(
                dimension.clone(),
                None,
                body_rational(&metre_body, "scale"),
                body_rational(&metre_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("canonical root"),
        ),
        (
            centimetre.clone(),
            UnitNode::checked(
                dimension.clone(),
                Some(metre.clone()),
                body_rational(&centimetre_body, "scale"),
                body_rational(&centimetre_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("derived unit"),
        ),
        (
            kilometre.clone(),
            UnitNode::checked(
                dimension.clone(),
                Some(metre.clone()),
                body_rational(&kilometre_body, "scale"),
                body_rational(&kilometre_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("kilometre edge"),
        ),
        (
            second.clone(),
            UnitNode::checked(
                time,
                None,
                body_rational(&second_body, "scale"),
                body_rational(&second_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("second canonical root"),
        ),
        (
            kelvin.clone(),
            UnitNode::checked(
                temperature.clone(),
                None,
                body_rational(&kelvin_body, "scale"),
                body_rational(&kelvin_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("kelvin canonical root"),
        ),
        (
            celsius.clone(),
            UnitNode::checked(
                temperature,
                Some(kelvin.clone()),
                body_rational(&celsius_body, "scale"),
                body_rational(&celsius_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("affine Celsius unit"),
        ),
        (
            square_metre.clone(),
            UnitNode::checked(
                area,
                None,
                body_rational(&square_metre_body, "scale"),
                body_rational(&square_metre_body, "offset"),
                NominalDeclaration::default(),
            )
            .expect("area canonical root"),
        ),
    ];
    let units = BTreeMap::from(unit_pairs);
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units)
        .expect("the emitted source's unit topology admits");
    AdmittedUnits {
        graph,
        dimension,
        metre,
        centimetre,
        kilometre,
        second,
        kelvin,
        celsius,
        square_metre,
    }
}

#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-5", "FR-011-AC-7")]
fn admitted_unit_graph_preserves_declared_owners_and_target_order() {
    let admitted = admitted_units();
    assert!(admitted.graph.dimension(admitted.dimension).is_some());
    assert!(admitted.graph.unit(admitted.metre).is_some());
    assert!(admitted.graph.unit(admitted.centimetre).is_some());
    assert!(admitted.graph.unit(admitted.kilometre).is_some());
    assert!(admitted.graph.unit(admitted.second).is_some());
    assert!(admitted.graph.unit(admitted.kelvin).is_some());
    assert!(admitted.graph.unit(admitted.celsius).is_some());
    assert!(admitted.graph.unit(admitted.square_metre).is_some());
    assert_eq!(admitted.graph.units().count(), 7);
}

#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-5")]
fn admitted_graph_has_the_same_canonical_dimensions_in_both_insertion_orders() {
    let time_block = "dimension Time;\nunit s : Time = rational(1, 1);\n";
    let temperature_block = "dimension Temperature;\nunit K : Temperature = rational(1, 1);\nunit degC : Temperature = rational(1, 1) * K + rational(5463, 20);\n";
    let original = format!("{time_block}{temperature_block}");
    assert!(
        SOURCE.contains(&original),
        "the source order being varied exists"
    );
    let reversed = SOURCE.replacen(&original, &format!("{temperature_block}{time_block}"), 1);
    let reordered_package = emit_and_read(&reversed);
    let forward = admitted_units();
    let reverse = admitted_units_from_package(&reordered_package);
    for admitted in [&forward, &reverse] {
        let length = admitted
            .graph
            .dimension(admitted.dimension.clone())
            .expect("admitted Length dimension");
        assert_eq!(
            length.exponents().collect::<Vec<_>>(),
            vec![(admitted.dimension.clone(), &Integer::from(1_i64))]
        );
        let square = admitted
            .graph
            .unit(admitted.square_metre.clone())
            .expect("admitted square metre unit");
        assert_eq!(
            square.dimension().exponents().collect::<Vec<_>>(),
            vec![(admitted.dimension.clone(), &Integer::from(2_i64))]
        );
        let keys: Vec<_> = admitted.graph.units().map(|unit| unit.key()).collect();
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    }
}

#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-5")]
fn admitted_root_units_have_canonical_compound_terms_and_order_refusal() {
    let admitted = admitted_units();
    let mut roots = [admitted.metre, admitted.second];
    roots.sort();
    let terms = [
        (roots[0].clone(), Integer::from(1_i64)),
        (roots[1].clone(), Integer::from(-1_i64)),
    ];
    let compound = admitted
        .graph
        .compound_unit(&terms)
        .expect("admitted roots form a compound unit");
    let actual: Vec<_> = compound
        .terms()
        .map(|(key, exponent)| (key, exponent.clone()))
        .collect();
    assert_eq!(actual, terms);
    let refused = admitted
        .graph
        .compound_unit(&[terms[1].clone(), terms[0].clone()])
        .expect_err("unsorted terms refuse");
    assert_eq!(refused.cause, CompoundUnitCause::UnsortedTerms);
}

fn meter() -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: 1024,
        result_units: 1024,
    })
}

/// The former TC-201 consumer path reads the package's declared unit IDs.
/// The expected centimetre conversion is an independent rational oracle.
#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-20", "FR-007-AC-21")]
fn admitted_units_convert_exactly_and_incompatible_dimensions_do_not_charge() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(250_i64)),
        UnitId::declared(admitted.centimetre.clone()),
    );
    let target = table
        .get(UnitId::declared(admitted.metre.clone()))
        .expect("admitted metre");
    let mut observed = meter();
    let Outcome::Completed(conversion) = convert_quantity(
        table.resolve(&source).expect("admitted centimetre"),
        target,
        &QuantityTarget::Exact,
        &mut observed,
    )
    .expect("same dimension") else {
        panic!("exact conversion completes");
    };
    assert_eq!(
        conversion.value(),
        &ConvertedValue::Exact(
            Rational::new(Integer::from(5_i64), Integer::from(2_i64)).expect("two is nonzero")
        )
    );
    assert_eq!(conversion.unit(), target);
    assert!(observed.admission_count() > 0);

    let incompatible = table
        .get(UnitId::declared(admitted.second))
        .expect("admitted second");
    let mut refused_meter = meter();
    let refusal = convert_quantity(
        table.resolve(&source).expect("admitted centimetre"),
        incompatible,
        &QuantityTarget::Exact,
        &mut refused_meter,
    )
    .expect_err("length does not convert to time");
    assert_eq!(refusal.cause, IllTypedCause::IncompatibleDimensions);
    assert_eq!(refused_meter.admission_count(), 0);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-20")]
fn admitted_centimetres_match_an_independent_fraction_oracle() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let target = table
        .get(UnitId::declared(admitted.metre))
        .expect("admitted metre");
    for numerator in -20_i64..=20 {
        for denominator in 1_i64..=20 {
            let value = Rational::new(Integer::from(numerator), Integer::from(denominator))
                .expect("positive denominator");
            let source = Quantity::new(value, UnitId::declared(admitted.centimetre.clone()));
            let Outcome::Completed(conversion) = convert_quantity(
                table.resolve(&source).expect("admitted centimetre"),
                target,
                &QuantityTarget::Exact,
                &mut meter(),
            )
            .expect("same dimension") else {
                panic!("fraction conversion completes");
            };
            let expected =
                Rational::new(Integer::from(numerator), Integer::from(denominator * 100))
                    .expect("positive denominator");
            assert_eq!(
                conversion.value(),
                &ConvertedValue::Exact(expected),
                "source={numerator}/{denominator} cm"
            );
        }
    }
}

/// TC-201's old i128 oracle varied both edge scales and offsets. One authored
/// source declares each edge once, so all 256 source/target pairs retain keys
/// from one emitted and IR-admitted package.
#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-20")]
fn admitted_scale_and_offset_matrix_matches_independent_i128_oracle() {
    const SCALES: [(i128, i128); 4] = [(-3, 7), (1, 2), (2, 3), (3, 1)];
    const OFFSETS: [(i128, i128); 4] = [(-1, 2), (0, 1), (1, 3), (7, 10)];
    const VALUES: [i128; 6] = [-2, -1, 0, 1, 2, i64::MAX as i128 + 1_000_003];
    let mut source = String::from(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         dimension Length;\n\
         unit root : Length = rational(1, 1);\n",
    );
    for (scale_index, &(numerator, denominator)) in SCALES.iter().enumerate() {
        for (offset_index, &(offset, divisor)) in OFFSETS.iter().enumerate() {
            source.push_str(&format!(
                "unit u{scale_index}{offset_index} : Length = \
                 rational({numerator}, {denominator}) * root + \
                 rational({offset}, {divisor});\n"
            ));
        }
    }
    source.push_str("function keep using v(x: Int[0, 9]): Int[0, 9] pure { x }\n");
    let package = emit_and_read(&source);
    let dimension_id = declaration_id(&package, "Length");
    let dimension = admitted_key(&dimension_id);
    let root_id = declaration_id(&package, "root");
    let root = admitted_key(&root_id);
    let root_body = nominal_wire(declaration_node(&package, "root"));
    assert_eq!(body_id(&root_body, "dimension_node_id"), dimension_id);
    assert!(root_body["target_unit_node_id"].is_null());
    let dimensions = BTreeMap::from([(
        dimension.clone(),
        DimensionNode::checked(Vec::new(), NominalDeclaration::default()).expect("base dimension"),
    )]);
    let mut units = BTreeMap::from([(
        root.clone(),
        UnitNode::checked(
            dimension.clone(),
            None,
            body_rational(&root_body, "scale"),
            body_rational(&root_body, "offset"),
            NominalDeclaration::default(),
        )
        .expect("canonical root"),
    )]);
    let mut keys = Vec::new();
    for scale_index in 0..SCALES.len() {
        for offset_index in 0..OFFSETS.len() {
            let name = format!("u{scale_index}{offset_index}");
            let id = declaration_id(&package, &name);
            let key = admitted_key(&id);
            let body = nominal_wire(declaration_node(&package, &name));
            assert_eq!(body_id(&body, "dimension_node_id"), dimension_id);
            assert_eq!(body_id(&body, "target_unit_node_id"), root_id);
            units.insert(
                key.clone(),
                UnitNode::checked(
                    dimension.clone(),
                    Some(root.clone()),
                    body_rational(&body, "scale"),
                    body_rational(&body, "offset"),
                    NominalDeclaration::default(),
                )
                .expect("admitted scale and offset"),
            );
            keys.push(key);
        }
    }
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units)
        .expect("the emitted scale/offset graph admits");
    let table = UnitTable::declared(&graph);
    let multiply = |(an, ad): (i128, i128), (bn, bd): (i128, i128)| (an * bn, ad * bd);
    let subtract = |(an, ad): (i128, i128), (bn, bd): (i128, i128)| (an * bd - bn * ad, ad * bd);
    let add = |(an, ad): (i128, i128), (bn, bd): (i128, i128)| (an * bd + bn * ad, ad * bd);
    for (source_scale_index, &(source_scale_n, source_scale_d)) in SCALES.iter().enumerate() {
        for (source_offset_index, &(source_offset_n, source_offset_d)) in OFFSETS.iter().enumerate()
        {
            let source_unit = &keys[source_scale_index * OFFSETS.len() + source_offset_index];
            for (target_scale_index, &(target_scale_n, target_scale_d)) in SCALES.iter().enumerate()
            {
                for (target_offset_index, &(target_offset_n, target_offset_d)) in
                    OFFSETS.iter().enumerate()
                {
                    let target_unit =
                        &keys[target_scale_index * OFFSETS.len() + target_offset_index];
                    let target = table
                        .get(UnitId::declared(target_unit.clone()))
                        .expect("admitted target unit");
                    for value in VALUES {
                        let quantity = Quantity::new(
                            Rational::from_integer(Integer::from(value)),
                            UnitId::declared(source_unit.clone()),
                        );
                        let Outcome::Completed(converted) = convert_quantity(
                            table.resolve(&quantity).expect("admitted source unit"),
                            target,
                            &QuantityTarget::Exact,
                            &mut meter(),
                        )
                        .expect("same dimension") else {
                            panic!("admitted scale/offset conversion completes");
                        };
                        // Independent i128 fraction arithmetic: map through the
                        // canonical root, then invert the target edge.
                        let canonical = add(
                            multiply((value, 1), (source_scale_n, source_scale_d)),
                            (source_offset_n, source_offset_d),
                        );
                        let shifted = subtract(canonical, (target_offset_n, target_offset_d));
                        let (expected_n, expected_d) =
                            multiply(shifted, (target_scale_d, target_scale_n));
                        let expected =
                            Rational::new(Integer::from(expected_n), Integer::from(expected_d))
                                .expect("nonzero target scale");
                        assert_eq!(
                            converted.value(),
                            &ConvertedValue::Exact(expected),
                            "source {source_scale_n}/{source_scale_d} + \
                             {source_offset_n}/{source_offset_d}, target \
                             {target_scale_n}/{target_scale_d} + \
                             {target_offset_n}/{target_offset_d}, value {value}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-23")]
fn admitted_conversion_charges_in_read_edge_arithmetic_target_retain_order() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(100_i64)),
        UnitId::declared(admitted.centimetre),
    );
    let mut meter = Meter::new(ScalarLimits {
        integer_bits: 8,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 1,
        value_occurrences: 1,
        work_units: 6,
        result_units: 1,
    });
    let Outcome::Completed(converted) = convert_quantity(
        table.resolve(&source).expect("admitted centimetre"),
        table
            .get(UnitId::declared(admitted.metre))
            .expect("admitted metre"),
        &QuantityTarget::Exact,
        &mut meter,
    )
    .expect("same dimension") else {
        panic!("centimetres to metres complete");
    };
    assert_eq!(
        converted.value(),
        &ConvertedValue::Exact(Rational::from_integer(Integer::from(1_i64)))
    );
    assert_eq!(
        meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect::<Vec<_>>(),
        [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitEdge,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitRationalArithmetic,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain,
        ]
    );
    assert_eq!(meter.consumed(LimitKind::UnitEdges), 1);
    assert_eq!(meter.consumed(LimitKind::IntegerBits), 8);
    assert_eq!(meter.consumed(LimitKind::ValueOccurrences), 1);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 6);
    assert_eq!(meter.consumed(LimitKind::ResultUnits), 1);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-22", "FR-007-AC-24")]
fn admitted_affine_unit_converts_exactly_and_refuses_arithmetic_before_charge() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(0_i64)),
        UnitId::declared(admitted.celsius),
    );
    let target = table
        .get(UnitId::declared(admitted.kelvin))
        .expect("admitted kelvin");
    let Outcome::Completed(conversion) = convert_quantity(
        table.resolve(&source).expect("admitted Celsius"),
        target,
        &QuantityTarget::Exact,
        &mut meter(),
    )
    .expect("same dimension") else {
        panic!("Celsius to Kelvin completes");
    };
    assert_eq!(
        conversion.value(),
        &ConvertedValue::Exact(
            Rational::new(Integer::from(5463_i64), Integer::from(20_i64))
                .expect("twenty is nonzero")
        )
    );
    let mut refused_meter = meter();
    let refusal = evaluate_quantity(
        QuantityOperation::Add(
            table.resolve(&source).expect("admitted Celsius"),
            table.resolve(&source).expect("admitted Celsius"),
        ),
        &mut refused_meter,
    )
    .expect_err("affine addition refuses before evaluation");
    assert_eq!(
        refusal,
        QuantityRefusal::IllTyped(quire_exact::IllTyped {
            cause: IllTypedCause::AffineUnitArithmetic,
        })
    );
    assert_eq!(refused_meter.admission_count(), 0);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-25")]
fn admitted_conversion_retention_denial_has_exact_incomplete_fields() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(100_i64)),
        UnitId::declared(admitted.centimetre),
    );
    let mut limits = *meter().limits();
    limits.result_units = 0;
    let mut meter = Meter::new(limits);
    let Outcome::Incomplete(record) = convert_quantity(
        table.resolve(&source).expect("admitted centimetre"),
        table
            .get(UnitId::declared(admitted.metre))
            .expect("admitted metre"),
        &QuantityTarget::Exact,
        &mut meter,
    )
    .expect("same dimension") else {
        panic!("retention denial returns Incomplete");
    };
    assert_eq!(record.limit_kind, LimitKind::ResultUnits);
    assert_eq!(record.limit, 0);
    assert_eq!(record.consumed, 0);
    assert_eq!(record.next_charge, Integer::from(1_i64));
    assert_eq!(record.charge_point, ChargePoint::UnitResultRetain);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-007-AC-25")]
fn admitted_two_edge_conversion_denial_reports_the_second_edge() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(1_i64)),
        UnitId::declared(admitted.kilometre),
    );
    let mut limits = *meter().limits();
    limits.unit_edges = 1;
    let mut meter = Meter::new(limits);
    let Outcome::Incomplete(record) = convert_quantity(
        table.resolve(&source).expect("admitted kilometre"),
        table
            .get(UnitId::declared(admitted.centimetre))
            .expect("admitted centimetre"),
        &QuantityTarget::Exact,
        &mut meter,
    )
    .expect("same dimension") else {
        panic!("second edge denial returns Incomplete");
    };
    assert_eq!(record.limit_kind, LimitKind::UnitEdges);
    assert_eq!(record.limit, 1);
    assert_eq!(record.consumed, 1);
    assert_eq!(record.next_charge, Integer::from(2_i64));
    assert_eq!(record.charge_point, ChargePoint::UnitEdge);
    assert_eq!(meter.consumed(LimitKind::UnitEdges), 1);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-275-AC-23")]
fn admitted_compound_to_declared_handoff_survives_second_conversion() {
    let admitted = admitted_units();
    let mut table = UnitTable::declared(&admitted.graph);
    let compound = admitted
        .graph
        .compound_unit(&[(admitted.metre, Integer::from(2_i64))])
        .expect("admitted metre forms square metres");
    let compound_id = table
        .insert(QuantityUnit::Compound(compound))
        .expect("compound identity is bounded");
    let source = Quantity::new(
        Rational::from_integer(Integer::from(4_i64)),
        compound_id.clone(),
    );
    assert_ne!(compound_id, UnitId::declared(admitted.square_metre.clone()));
    let mut first_meter = meter();
    let Outcome::Completed(first) = convert_quantity(
        table.resolve(&source).expect("formed compound unit"),
        table
            .get(UnitId::declared(admitted.square_metre))
            .expect("admitted area root"),
        &QuantityTarget::Exact,
        &mut first_meter,
    )
    .expect("same normalized dimension") else {
        panic!("compound converts to declared area");
    };
    assert_eq!(
        first.value(),
        &ConvertedValue::Exact(Rational::from_integer(Integer::from(4_i64)))
    );
    let before_handoff = first_meter.admission_count();
    let (second_source, identified) = first
        .identified_quantity()
        .expect("exact result is a quantity");
    assert_eq!(first_meter.admission_count(), before_handoff);
    assert_eq!(second_source.unit(), &identified.id());
    assert_eq!(
        second_source.unit(),
        &UnitId::declared(admitted.square_metre)
    );
    let Outcome::Completed(second) = convert_quantity(
        table.resolve(&second_source).expect("declared area unit"),
        table.get(compound_id).expect("formed compound target"),
        &QuantityTarget::Exact,
        &mut meter(),
    )
    .expect("same normalized dimension") else {
        panic!("declared area converts back to compound unit");
    };
    assert_eq!(
        second.value(),
        &ConvertedValue::Exact(Rational::from_integer(Integer::from(4_i64)))
    );
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-201", "FR-275-AC-23")]
fn admitted_nonexact_conversion_refuses_quantity_handoff_without_extra_charge() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let source = Quantity::new(
        Rational::from_integer(Integer::from(1_i64)),
        UnitId::declared(admitted.metre.clone()),
    );
    let mut meter = meter();
    let Outcome::Completed(converted) = convert_quantity(
        table.resolve(&source).expect("admitted metre"),
        table
            .get(UnitId::declared(admitted.metre))
            .expect("same admitted metre"),
        &QuantityTarget::Integer {
            domain: IntegerInterval::spanning(Integer::from(0_i64), Integer::from(10_i64)),
            rounding: RoundingMode::Exact,
        },
        &mut meter,
    )
    .expect("same dimension") else {
        panic!("integer conversion completes");
    };
    let before_handoff = meter.admission_count();
    assert_eq!(
        converted.identified_quantity(),
        Err(HandoffRefusal::NotExact)
    );
    assert_eq!(meter.admission_count(), before_handoff);
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-032", "FR-011-AC-2", "FR-011-AC-4")]
fn admitted_unit_identity_read_precedes_other_denials_and_zero_stops() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let unit = UnitId::declared(admitted.metre);
    let one = Quantity::new(Rational::from_integer(Integer::from(5_i64)), unit.clone());
    let zero = Quantity::new(Rational::from_integer(Integer::from(0_i64)), unit);
    let mut limits = *meter().limits();
    limits.integer_bits = 0;
    limits.value_occurrences = 0;
    let mut denied = Meter::new(limits);
    let Outcome::Incomplete(record) = evaluate_quantity(
        QuantityOperation::Add(
            table.resolve(&one).expect("admitted unit"),
            table.resolve(&one).expect("admitted unit"),
        ),
        &mut denied,
    )
    .expect("well typed") else {
        panic!("identity-read denial returns Incomplete");
    };
    assert_eq!(record.limit_kind, LimitKind::IntegerBits);
    assert_eq!(record.limit, 0);
    assert_eq!(record.consumed, 0);
    assert_eq!(record.next_charge, Integer::from(3_i64));
    assert_eq!(record.charge_point, ChargePoint::UnitIdentityRead);
    assert_eq!(denied.admission_count(), 0);

    let outcome = evaluate_quantity(
        QuantityOperation::Divide(
            table.resolve(&one).expect("admitted unit"),
            table.resolve(&zero).expect("admitted unit"),
        ),
        &mut meter(),
    )
    .expect("same unit");
    assert_eq!(outcome, Outcome::Undefined(Undefined::DivisionByZero));
}

#[test]
#[trace("TC-096", "IT-016-SC-02", "TC-034", "FR-007-AC-16", "FR-007-AC-17")]
fn admitted_affine_power_refuses_and_distinct_length_units_multiply_canonically() {
    let admitted = admitted_units();
    let table = UnitTable::declared(&admitted.graph);
    let celsius = Quantity::new(
        Rational::from_integer(Integer::from(1_i64)),
        UnitId::declared(admitted.celsius),
    );
    for exponent in [0_i64, -1] {
        let exponent = Integer::from(exponent);
        let mut refused_meter = meter();
        assert_eq!(
            evaluate_quantity(
                QuantityOperation::Power(
                    table.resolve(&celsius).expect("admitted Celsius"),
                    &exponent,
                ),
                &mut refused_meter,
            ),
            Err(QuantityRefusal::IllTyped(quire_exact::IllTyped {
                cause: IllTypedCause::AffineUnitArithmetic,
            }))
        );
        assert_eq!(refused_meter.admission_count(), 0);
    }

    let metre = Quantity::new(
        Rational::from_integer(Integer::from(2_i64)),
        UnitId::declared(admitted.metre.clone()),
    );
    let centimetre = Quantity::new(
        Rational::from_integer(Integer::from(50_i64)),
        UnitId::declared(admitted.centimetre),
    );
    let Outcome::Completed(product) = evaluate_quantity(
        QuantityOperation::Multiply(
            table.resolve(&metre).expect("admitted metre"),
            table.resolve(&centimetre).expect("admitted centimetre"),
        ),
        &mut meter(),
    )
    .expect("non-affine multiplication") else {
        panic!("distinct length units multiply exactly");
    };
    assert_eq!(
        product.magnitude(),
        &Rational::from_integer(Integer::from(1_i64))
    );
    let expected_unit = admitted
        .graph
        .compound_unit(&[(admitted.metre, Integer::from(2_i64))])
        .expect("metre squared is canonical")
        .id()
        .expect("compound ID fits the configured bound");
    assert_eq!(product.unit(), &expected_unit);
}

/// The malformed operation varies after admission. Every term key still came
/// from a real source declaration that IR accepted.
#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-4", "FR-011-AC-7")]
fn admitted_dimension_keys_expose_first_typed_term_cause() {
    let package = admitted();
    let length = admitted_key(&declaration_id(package, "Length"));
    let time = admitted_key(&declaration_id(package, "Time"));
    let ascending = if length < time {
        [length, time]
    } else {
        [time, length]
    };
    let checked = |terms| DimensionNode::checked(terms, NominalDeclaration::default());
    assert_eq!(
        checked(vec![
            (ascending[1].clone(), Integer::from(1_i64)),
            (ascending[0].clone(), Integer::from(0_i64)),
            (ascending[0].clone(), Integer::from(1_i64)),
        ]),
        Err(SemanticGraphCause::ZeroExponent)
    );
    assert_eq!(
        checked(vec![
            (ascending[0].clone(), Integer::from(1_i64)),
            (ascending[0].clone(), Integer::from(1_i64)),
        ]),
        Err(SemanticGraphCause::DuplicateTerm)
    );
    assert_eq!(
        checked(vec![
            (ascending[1].clone(), Integer::from(1_i64)),
            (ascending[0].clone(), Integer::from(1_i64)),
        ]),
        Err(SemanticGraphCause::UnsortedTerms)
    );

    let area = admitted_key(&declaration_id(package, "Area"));
    let length = admitted_key(&declaration_id(package, "Length"));
    let time = admitted_key(&declaration_id(package, "Time"));
    let dimensions = BTreeMap::from([
        (length.clone(), checked(Vec::new()).expect("base length")),
        (
            area.clone(),
            checked(vec![(length, Integer::from(2_i64))]).expect("derived area"),
        ),
        (
            time,
            checked(vec![(area, Integer::from(1_i64))]).expect("per-node valid term"),
        ),
    ]);
    assert_eq!(
        UnitGraph::from_checked_nodes(&dimensions, &BTreeMap::new())
            .expect_err("a derived term is not a base dimension")
            .cause,
        SemanticGraphCause::NonBaseDimensionTerm
    );
}

/// The graph's malformed edges are varied after the producer and reader have
/// admitted the keys. Distinct keys always designate real source declarations.
#[test]
#[trace("TC-096", "IT-016-SC-01", "TC-033", "FR-012-AC-4", "FR-011-AC-7")]
fn admitted_unit_keys_expose_first_typed_edge_and_topology_cause() {
    let package = admitted();
    let length = admitted_key(&declaration_id(package, "Length"));
    let time = admitted_key(&declaration_id(package, "Time"));
    let metre = admitted_key(&declaration_id(package, "m"));
    let centimetre = admitted_key(&declaration_id(package, "cm"));
    let kilometre = admitted_key(&declaration_id(package, "km"));
    let second = admitted_key(&declaration_id(package, "s"));
    let ratio = |value: i64| Rational::from_integer(Integer::from(value));
    let unit = |dimension: NodeKey, target: Option<NodeKey>, scale, offset| {
        UnitNode::checked(
            dimension,
            target,
            ratio(scale),
            ratio(offset),
            NominalDeclaration::default(),
        )
        .expect("the per-node edge shape admits")
    };
    assert_eq!(
        UnitNode::checked(
            length.clone(),
            None,
            ratio(0),
            ratio(5),
            NominalDeclaration::default(),
        ),
        Err(SemanticGraphCause::ZeroScale)
    );
    assert_eq!(
        UnitNode::checked(
            length.clone(),
            None,
            ratio(2),
            ratio(0),
            NominalDeclaration::default(),
        ),
        Err(SemanticGraphCause::NonIdentityRoot)
    );
    let base = |key: NodeKey| {
        BTreeMap::from([(
            key,
            DimensionNode::checked(Vec::new(), NominalDeclaration::default())
                .expect("base dimension"),
        )])
    };
    let length_only = base(length.clone());
    let two_dimensions = BTreeMap::from([
        (
            length.clone(),
            DimensionNode::checked(Vec::new(), NominalDeclaration::default()).expect("length"),
        ),
        (
            time.clone(),
            DimensionNode::checked(Vec::new(), NominalDeclaration::default()).expect("time"),
        ),
    ]);
    let cases = [
        (
            &length_only,
            BTreeMap::from([(second.clone(), unit(time.clone(), None, 1, 0))]),
            SemanticGraphCause::UnknownDimension,
        ),
        (
            &length_only,
            BTreeMap::from([
                (metre.clone(), unit(length.clone(), None, 1, 0)),
                (
                    centimetre.clone(),
                    unit(length.clone(), Some(kilometre.clone()), 1, 0),
                ),
            ]),
            SemanticGraphCause::UnknownTarget,
        ),
        (
            &two_dimensions,
            BTreeMap::from([
                (metre.clone(), unit(length.clone(), None, 1, 0)),
                (
                    second.clone(),
                    unit(time.clone(), Some(metre.clone()), 1, 0),
                ),
            ]),
            SemanticGraphCause::CrossDimensionTarget,
        ),
        (
            &length_only,
            BTreeMap::from([
                (
                    metre.clone(),
                    unit(length.clone(), Some(centimetre.clone()), 1, 0),
                ),
                (
                    centimetre.clone(),
                    unit(length.clone(), Some(metre.clone()), 1, 0),
                ),
            ]),
            SemanticGraphCause::MissingRoot,
        ),
        (
            &length_only,
            BTreeMap::from([
                (metre.clone(), unit(length.clone(), None, 1, 0)),
                (centimetre.clone(), unit(length.clone(), None, 1, 0)),
            ]),
            SemanticGraphCause::DuplicateRoot,
        ),
        (
            &length_only,
            BTreeMap::from([
                (metre, unit(length.clone(), None, 1, 0)),
                (
                    centimetre.clone(),
                    unit(length.clone(), Some(kilometre.clone()), 1, 0),
                ),
                (kilometre, unit(length, Some(centimetre), 1, 0)),
            ]),
            SemanticGraphCause::TargetCycle,
        ),
    ];
    for (dimensions, units, cause) in cases {
        assert_eq!(
            UnitGraph::from_checked_nodes(dimensions, &units)
                .expect_err("malformed topology refuses")
                .cause,
            cause
        );
    }
    // `DuplicateNode` is not representable: the graph accepts a BTreeMap
    // keyed by the exact admitted NodeKey, so duplicate insertion replaces.
}
