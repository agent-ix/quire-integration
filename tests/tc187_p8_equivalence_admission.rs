// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-111: original public TC-187 U37 native equivalence admission only.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_foundation::{SourceIdentity, Span};
use qsl_semantics::{
    check::{
        wire_id::structural_id, AssemblyCause, AssemblyError, AssemblyRefusal, EquivalenceClasses,
        EquivalenceFault, EquivalenceRetained, PackageDeclarations,
    },
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input},
    },
};
use quire_exact::{NodeKey, Rational};
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const PACKAGE: &str = "agent-ix/tc111-equivalence";
const HEADER: &str = "language \"ix:native\" edition \"1-draft\"; profile v = \"quire.value.complete/v1\"; model R = \"acme/rpc\"; model F = \"acme/flow\"; dimension Time; unit s : Time = rational(1, 1);";
const TIME: &str = "dimension R::Time ~ F::Time by 1 R::s = 1 F::s;";
const LENGTH: &str = "dimension R::Length ~ F::Length by 1 R::m = 1 F::m;";
const DUR: &str = "dimension Dur = R::Time; unit dsec : Dur = rational(1, 1);";
const DUR_PAIR: &str = "dimension Dur ~ F::Time by 1 dsec = 1 F::s;";
const BASES: &str = "dimension A; unit ua : A = rational(1, 1); dimension B; unit ub : B = rational(1, 1); dimension C; unit uc : C = rational(1, 1);";

fn exact(value: i64) -> Rational {
    Rational::from_integer(value.into())
}

// Original domain input: the anchor makes the document a real nonempty model.
fn document(identity: &str, area: bool) -> Vec<u8> {
    let origin = json!({"generated": {"generatorIdentity": format!("ix://{identity}/equivalence-input"), "generatorVersion": "1.0.0", "inputIdentities": [format!("ix://{identity}/equivalence-input")]}});
    let dim = |name: &str| format!("ix://{identity}/dimension/{name}");
    let mut dimensions = Vec::new();
    for (name, terms) in [
        ("Time", vec![]),
        ("Length", vec![]),
        ("Speed", vec![("Length", "1"), ("Time", "-1")]),
    ]
    .into_iter()
    .chain(area.then_some(("Area", vec![("Length", "2")])))
    {
        dimensions.push(json!({"identity": dim(name), "name": name, "base": terms.is_empty(), "terms": terms.into_iter().map(|(name, exponent)| json!({"dimension": dim(name), "exponent": exponent})).collect::<Vec<_>>(), "origin": origin}));
    }
    let mut units = Vec::new();
    for (name, dimension) in [("s", "Time"), ("m", "Length"), ("mps", "Speed")]
        .into_iter()
        .chain(area.then_some(("m2", "Area")))
    {
        units.push(json!({"identity": format!("ix://{identity}/unit/{name}"), "name": name, "dimension": dim(dimension), "scale": "1", "offset": "0", "origin": origin}));
    }
    if !area {
        // The authored local and an independent two-edge composed-offset control.
        for (name, target, offset) in [
            ("local", "s", "5"),
            ("shift", "s", "2"),
            ("composed", "shift", "3"),
        ] {
            units.push(json!({"identity": format!("ix://{identity}/unit/{name}"), "name": name, "dimension": dim("Time"), "target": format!("ix://{identity}/unit/{target}"), "scale": "1", "offset": offset, "origin": origin}));
        }
    }
    serde_json::to_vec(&json!({
        "contractVersion": "2.0.0", "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": format!("ix://{identity}/equivalence-input"), "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": identity, "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
        "occurrences": [], "extensions": [],
        "constructs": [{"kind": {"module": identity, "name": "equivalence_anchor"}, "moduleVersion": "1.0.0", "construct": {"identity": "none", "shape": "record", "members": {}, "meaning": "quire.meaning.model.object-type/v1"}}],
        "types": [{"identity": format!("ix://{identity}/EquivalenceAnchor"), "displayName": "EquivalenceAnchor", "kind": {"module": identity, "name": "equivalence_anchor"}, "roles": [], "origin": origin, "constraints": [], "extensions": [], "unknownPolicy": "reject", "supertypes": [], "fields": [], "operations": []}],
        "dimensions": dimensions, "units": units,
    })).expect("original domain serializes")
}

fn assemble(body: &str) -> (String, Result<PackageDeclarations, AssemblyRefusal>) {
    let source = format!("{HEADER}\n{body}");
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc111-equivalence"),
        "equivalence.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original parse");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = qsl_forms::build_unit(&parsed).expect("original forms");
    let documents = [document("acme/rpc", true), document("acme/flow", false)];
    let input = package_input(documents.iter().map(Vec::as_slice)).expect("original package input");
    let models = admit_unit(
        &unit.selections().models,
        &input,
        ModelNormalizationLimits::default(),
    )
    .expect("genuine model intake must succeed before equivalence decision");
    let result = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        PACKAGE.to_owned(),
        unit,
        models,
        Vec::new(),
    );
    (source, result)
}

fn span(source: &str, text: &str) -> Span {
    let starts: Vec<_> = source.match_indices(text).map(|(start, _)| start).collect();
    let [start] = starts.as_slice() else {
        panic!("one original occurrence of {text}: {starts:?}")
    };
    Span {
        start: *start,
        end: start + text.len(),
    }
}

fn nominal(package: &PackageDeclarations, source: &str, name: &str) -> NodeKey {
    let keys: Vec<_> = package
        .nominal_spans
        .iter()
        .filter(|(_, span)| &source[span.start..span.end] == name)
        .map(|(key, _)| key.clone())
        .collect();
    let [key] = keys.as_slice() else {
        panic!("one original {name}: {keys:?}")
    };
    key.clone()
}

struct Keys {
    retained: BTreeMap<String, NodeKey>,
    admitted: BTreeMap<String, NodeKey>,
}

impl Keys {
    fn control() -> Self {
        let aliases = [
            ("rt", "R::Time", "R::s"),
            ("ft", "F::Time", "F::s"),
            ("rl", "R::Length", "R::m"),
            ("fl", "F::Length", "F::m"),
            ("rs", "R::Speed", "R::mps"),
            ("fs", "F::Speed", "F::mps"),
            ("ra", "R::Area", "R::m2"),
        ];
        let mut body = format!("{DUR} {BASES}");
        for (alias, dimension, unit) in aliases {
            body.push_str(&format!(
                " unit {alias} : {dimension} = rational(1, 1) * {unit};"
            ));
        }
        let (source, result) = assemble(&body);
        let package = result.expect("healthy native key control");
        let mut retained = BTreeMap::new();
        let mut admitted = BTreeMap::new();
        for (alias, dimension, unit) in aliases {
            let native = package
                .units
                .unit(nominal(&package, &source, alias))
                .expect("genuine alias unit")
                .dimension_node();
            retained.insert(dimension.to_owned(), native.clone());
            let (model_alias, unit_name) = unit.split_once("::").expect("qualified control unit");
            let identity = if model_alias == "R" {
                "acme/rpc"
            } else {
                "acme/flow"
            };
            let imported = package
                .units
                .units()
                .find(|unit| {
                    serde_json::to_value(structural_id(&unit.key())).expect("actual unit ID")
                        ["path"]
                        == json!([
                            "scalar_type",
                            "unit",
                            format!("ix://{identity}/unit/{unit_name}")
                        ])
                })
                .expect("actual imported unit");
            admitted.insert(
                dimension.to_owned(),
                if matches!(dimension, "R::Time" | "F::Time" | "R::Length" | "F::Length") {
                    imported.dimension_node()
                } else {
                    native
                },
            );
        }
        for name in ["Time", "Dur", "A", "B", "C"] {
            let key = nominal(&package, &source, name);
            retained.insert(name.to_owned(), key.clone());
            admitted.insert(name.to_owned(), key);
        }
        Self { retained, admitted }
    }
    fn dimension(&self, name: &str) -> EquivalenceRetained {
        EquivalenceRetained::Dimension {
            name: name.to_owned(),
            key: Some(self.retained[name].clone()),
        }
    }
}

fn declaration(source: &str, text: &str, names: [&str; 2]) -> EquivalenceRetained {
    EquivalenceRetained::Declaration {
        dimensions: names.map(str::to_owned),
        span: span(source, text),
    }
}

fn refused(body: &str, primary: &str, expected: impl FnOnce(&str) -> AssemblyCause, code: &str) {
    let (source, result) = assemble(body);
    let refusal = result.expect_err("authored equivalence refuses");
    let [error]: &[AssemblyError; 1] = refusal
        .errors
        .as_slice()
        .try_into()
        .expect("exactly one native first refusal");
    assert_eq!(error.cause, expected(&source), "{body}");
    assert_eq!(error.cause.catalog_code().to_string(), code, "{body}");
    assert_eq!(
        error.span,
        span(&source, primary),
        "original primary locus: {body}"
    );
}

fn fault(fault: EquivalenceFault, retained: Vec<EquivalenceRetained>) -> AssemblyCause {
    AssemblyCause::DimensionEquivalence { fault, retained }
}

fn admitted(body: &str, pairs: &[(&str, [&str; 2], Rational)], keys: &Keys) -> EquivalenceClasses {
    let (source, result) = assemble(body);
    let package = result.expect("authored equivalence admits");
    assert_eq!(package.dimension_equivalences.len(), pairs.len(), "{body}");
    let mut expected: Vec<_> = pairs.iter().collect();
    expected.sort_by_key(|(text, _, _)| span(&source, text).start);
    for (row, (text, names, ratio)) in package.dimension_equivalences.iter().zip(expected) {
        let mut dimensions = names.map(|name| keys.admitted[name].clone());
        let scale = if dimensions[0] < dimensions[1] {
            ratio.clone()
        } else {
            dimensions.swap(0, 1);
            exact(1).div(ratio).expect("positive reciprocal")
        };
        assert_eq!(row.dimensions, dimensions, "{text}");
        assert_eq!(row.root_scale, scale, "{text}");
        assert_eq!(row.span, span(&source, text));
        let id: Value =
            serde_json::to_value(structural_id(&row.key)).expect("native equivalence ID");
        assert_eq!(id["package"], PACKAGE);
        assert_eq!(id["path"][0], "scalar_type");
        assert_eq!(id["path"][1], "dimension_equivalence");
    }
    let classes = EquivalenceClasses::new(&package.dimension_equivalences);
    for (_, names, ratio) in pairs {
        let (first_root, first_size) = classes.class(keys.admitted[names[0]].clone());
        let (second_root, second_size) = classes.class(keys.admitted[names[1]].clone());
        assert_eq!(first_root, second_root);
        assert_eq!(
            first_size.div(&second_size),
            Some(ratio.clone()),
            "exact class size relation {names:?}"
        );
    }
    classes
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn public_u37_is_bound_by_reference() {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == "U37")
        .collect();
    let [row] = rows.as_slice() else {
        panic!("one original public U37: {rows:?}")
    };
    for fragment in [
        "R::Time ~ F::Time",
        "R::Time ~ Time",
        "Z::Time",
        "Nothing",
        "F::second",
        "R::Time ~ R::Time",
        "2 R::s",
        "1 R::m",
        "0 R::s",
        "rational(-1, 1)",
        "F::local",
        "R::Area",
        "Dur ~ F::Time",
        "R::Speed ~ F::Speed",
        "2 F::mps",
        "R::Length ~ F::Length",
        "B ~ C",
        "1000 uc",
        "reverse source order",
    ] {
        assert!(
            row.inputs.contains(fragment),
            "public U37 input: missing {fragment}: {}",
            row.inputs
        );
    }
    for fragment in [
        "missing-name",
        "definition-cycle",
        "conflicting-definition",
        "incompatible-definition",
        "invalid-value",
        "composed offset",
        "retaining all three in source order",
        "within a phase changes no decision",
    ] {
        assert!(
            row.expected.contains(fragment),
            "public U37 outcome: missing {fragment}: {}",
            row.expected
        );
    }
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn basic_pairs_names_and_self_pair_keep_native_identity_and_locus() {
    let keys = Keys::control();
    for (text, names) in [
        (TIME, ["R::Time", "F::Time"]),
        (
            "dimension R::Time ~ Time by 1 R::s = 1 s;",
            ["R::Time", "Time"],
        ),
    ] {
        let classes = admitted(text, &[(text, names, exact(1))], &keys);
        let least = names
            .map(|name| keys.admitted[name].clone())
            .into_iter()
            .min()
            .expect("pair");
        for name in names {
            assert_eq!(
                classes.class(keys.admitted[name].clone()),
                (least.clone(), exact(1))
            );
        }
        assert_ne!(keys.admitted[names[0]], keys.admitted[names[1]]);
    }
    for (text, name) in [
        ("dimension R::Time ~ Z::Time by 1 R::s = 1 Z::s;", "Z::Time"),
        ("dimension R::Time ~ Nothing by 1 R::s = 1 s;", "Nothing"),
        (
            "dimension R::Time ~ F::Time by 1 R::s = 1 F::second;",
            "F::second",
        ),
    ] {
        let (source, result) = assemble(text);
        let refusal = result.expect_err("missing name");
        let [error] = refusal.errors.as_slice() else {
            panic!("one unresolved name: {refusal:?}")
        };
        assert_eq!(
            error.cause,
            AssemblyCause::UnresolvedQuantityName {
                name: name.to_owned()
            }
        );
        assert_eq!(
            error.cause.catalog_code().to_string(),
            "missing_declaration/missing-name"
        );
        assert_eq!(error.span, span(&source, name));
    }
    let text = "dimension R::Time ~ R::Time by 1 R::s = 1 R::s;";
    refused(
        text,
        text,
        |_| AssemblyCause::QuantityCycle {
            edges: vec![("R::Time".to_owned(), "R::Time".to_owned())],
        },
        "invalid_package/definition-cycle",
    );
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn duplicate_pairs_keep_both_original_declarations_in_either_order() {
    for reverse in [
        "dimension F::Time ~ R::Time by 1 F::s = 1 R::s;",
        "dimension F::Time ~ R::Time by 1 F::s = 2 R::s;",
    ] {
        for rows in [
            [
                (TIME, ["R::Time", "F::Time"]),
                (reverse, ["F::Time", "R::Time"]),
            ],
            [
                (reverse, ["F::Time", "R::Time"]),
                (TIME, ["R::Time", "F::Time"]),
            ],
        ] {
            let body = format!("{} {}", rows[0].0, rows[1].0);
            refused(
                &body,
                rows[1].0,
                |source| {
                    fault(
                        EquivalenceFault::Conflicting,
                        rows.iter()
                            .map(|(text, names)| declaration(source, text, *names))
                            .collect(),
                    )
                },
                "invalid_package/conflicting-definition",
            );
        }
    }
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn wrong_units_values_affine_offset_and_area_keep_full_typed_payloads() {
    let keys = Keys::control();
    for (text, unit) in [
        ("dimension R::Time ~ F::Time by 1 R::m = 1 F::s;", "R::m"),
        ("dimension R::Time ~ F::Time by 1 F::s = 1 R::s;", "F::s"),
    ] {
        refused(
            text,
            text,
            |_| {
                fault(
                    EquivalenceFault::Incompatible,
                    vec![
                        EquivalenceRetained::Unit {
                            name: unit.to_owned(),
                        },
                        keys.dimension("R::Time"),
                    ],
                )
            },
            "invalid_package/incompatible-definition",
        );
    }
    for (text, value) in [
        ("dimension R::Time ~ F::Time by 0 R::s = 1 F::s;", 0),
        (
            "dimension R::Time ~ F::Time by rational(-1, 1) R::s = 1 F::s;",
            -1,
        ),
    ] {
        refused(
            text,
            text,
            |source| {
                fault(
                    EquivalenceFault::InvalidValue,
                    vec![
                        declaration(source, text, ["R::Time", "F::Time"]),
                        EquivalenceRetained::Value(exact(value)),
                    ],
                )
            },
            "invalid_package/invalid-value",
        );
    }
    for unit in ["F::local", "F::composed"] {
        let affine = format!("dimension R::Time ~ F::Time by 1 R::s = 1 {unit};");
        refused(
            &affine,
            &affine,
            |_| {
                fault(
                    EquivalenceFault::InvalidValue,
                    vec![
                        EquivalenceRetained::Unit {
                            name: unit.to_owned(),
                        },
                        EquivalenceRetained::Value(exact(5)),
                    ],
                )
            },
            "invalid_package/invalid-value",
        );
    }
    let area = "dimension R::Time ~ R::Area by 1 R::s = 1 R::m2;";
    refused(
        area,
        area,
        |_| {
            fault(
                EquivalenceFault::Incompatible,
                vec![keys.dimension("R::Time"), keys.dimension("R::Area")],
            )
        },
        "invalid_package/incompatible-definition",
    );
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn dur_uses_base_authority_before_derived_phase_in_both_source_orders() {
    let keys = Keys::control();
    refused(
        &format!("{DUR} {DUR_PAIR}"),
        DUR_PAIR,
        |_| {
            fault(
                EquivalenceFault::Incompatible,
                vec![keys.dimension("Dur"), keys.dimension("F::Time")],
            )
        },
        "invalid_package/incompatible-definition",
    );
    for declarations in [format!("{TIME} {DUR_PAIR}"), format!("{DUR_PAIR} {TIME}")] {
        let classes = admitted(
            &format!("{DUR} {declarations}"),
            &[
                (TIME, ["R::Time", "F::Time"], exact(1)),
                (DUR_PAIR, ["Dur", "F::Time"], exact(1)),
            ],
            &keys,
        );
        let least = ["Dur", "R::Time", "F::Time"]
            .map(|name| keys.admitted[name].clone())
            .into_iter()
            .min()
            .expect("three nodes");
        for name in ["Dur", "R::Time", "F::Time"] {
            assert_eq!(
                classes.class(keys.admitted[name].clone()),
                (least.clone(), exact(1))
            );
        }
    }
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn speeds_require_both_base_pairs_and_exact_ratio_in_all_phase_orders() {
    let keys = Keys::control();
    for ratio in [1, 2] {
        let speed = format!("dimension R::Speed ~ F::Speed by 1 R::mps = {ratio} F::mps;");
        for bases in ["", TIME, LENGTH] {
            for body in [format!("{bases} {speed}"), format!("{speed} {bases}")] {
                refused(
                    &body,
                    &speed,
                    |_| {
                        fault(
                            EquivalenceFault::Incompatible,
                            vec![keys.dimension("R::Speed"), keys.dimension("F::Speed")],
                        )
                    },
                    "invalid_package/incompatible-definition",
                );
            }
        }
        for bases in [format!("{TIME} {LENGTH}"), format!("{LENGTH} {TIME}")] {
            for body in [format!("{bases} {speed}"), format!("{speed} {bases}")] {
                if ratio == 1 {
                    let classes = admitted(
                        &body,
                        &[
                            (TIME, ["R::Time", "F::Time"], exact(1)),
                            (LENGTH, ["R::Length", "F::Length"], exact(1)),
                            (&speed, ["R::Speed", "F::Speed"], exact(1)),
                        ],
                        &keys,
                    );
                    for names in [
                        ["R::Time", "F::Time"],
                        ["R::Length", "F::Length"],
                        ["R::Speed", "F::Speed"],
                    ] {
                        let least = names
                            .map(|name| keys.admitted[name].clone())
                            .into_iter()
                            .min()
                            .expect("pair");
                        for name in names {
                            assert_eq!(
                                classes.class(keys.admitted[name].clone()),
                                (least.clone(), exact(1))
                            );
                        }
                    }
                } else {
                    refused(
                        &body,
                        &speed,
                        |_| {
                            fault(
                                EquivalenceFault::Incompatible,
                                vec![keys.dimension("R::Speed"), keys.dimension("F::Speed")],
                            )
                        },
                        "invalid_package/incompatible-definition",
                    );
                }
            }
        }
    }
}

#[trace("TC-111", "QSpec-TC-187", "QSpec-FR-142-AC-22")]
#[test]
fn abc_cycles_preserve_sizes_and_actual_closing_locus_when_reversed() {
    let keys = Keys::control();
    let ab = "dimension A ~ B by 1 ua = 1 ub;";
    let bc = "dimension B ~ C by 1 ub = 1000 uc;";
    let ca = "dimension C ~ A by 1000 uc = 1 ua;";
    let bad_ca = "dimension C ~ A by 1 uc = 1 ua;";
    let pairs = [
        (ab, ["A", "B"], exact(1)),
        (bc, ["B", "C"], exact(1000)),
        (
            ca,
            ["C", "A"],
            exact(1).div(&exact(1000)).expect("thousandth"),
        ),
    ];
    let mut original = None;
    for rows in [[ab, bc, ca], [ca, bc, ab]] {
        let classes = admitted(&format!("{BASES} {}", rows.join(" ")), &pairs, &keys);
        let least = ["A", "B", "C"]
            .map(|name| keys.admitted[name].clone())
            .into_iter()
            .min()
            .expect("three nodes");
        for name in ["A", "B", "C"] {
            assert_eq!(classes.class(keys.admitted[name].clone()).0, least);
        }
        if let Some(previous) = &original {
            assert_eq!(&classes, previous, "source order preserves full classes");
        }
        original = Some(classes);
    }
    for rows in [
        [(ab, ["A", "B"]), (bc, ["B", "C"]), (bad_ca, ["C", "A"])],
        [(bad_ca, ["C", "A"]), (bc, ["B", "C"]), (ab, ["A", "B"])],
    ] {
        let body = format!("{BASES} {}", rows.map(|(text, _)| text).join(" "));
        refused(
            &body,
            rows[2].0,
            |source| {
                fault(
                    EquivalenceFault::Conflicting,
                    rows.iter()
                        .map(|(text, names)| declaration(source, text, *names))
                        .collect(),
                )
            },
            "invalid_package/conflicting-definition",
        );
    }
}
