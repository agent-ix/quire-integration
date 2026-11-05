// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original quantity source acquisition and public closed-body observations.

use std::collections::BTreeMap;

use qsl_package::CheckedPackage;
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::intake::SelectedModels;
use quire_contract_codegen::{derive_exact_scalar_items, generate_exact_scalar_oracles};
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm,
};
use quire_exact::{Meter, NodeKey, ScalarLimits};
use quire_semantic_value::checking::CheckingLimits;
use quire_semantic_value::unit::UnitGraph;
use quire_specification::{vectors_of, TestCase};

const UNITS: &str = "dimension Distance; unit m : Distance = rational(1, 1); unit cm : Distance = rational(1, 100) * m; dimension Duration; unit s : Duration = rational(1, 1);";

/// Borrow and bind the selected public vector facts.
pub fn bind(id: &str, inputs: &[&str], expected: &[&str]) {
    let mut rows = vectors_of(TestCase::Tc187).filter(|row| row.id == id);
    let row = rows.next().expect("public quantity vector");
    assert!(rows.next().is_none(), "unique vector {id}");
    for fact in inputs {
        assert!(row.inputs.contains(fact), "{id}: {}", row.inputs);
    }
    for fact in expected {
        assert!(row.expected.contains(fact), "{id}: {}", row.expected);
    }
}

/// Checked original source with its admitted nominal unit graph.
pub struct Original {
    functions: String,
    /// Original checked package; never reconstructed from a test body.
    pub package: CheckedPackage,
    /// Unit graph admitted from the original source declarations.
    pub units: UnitGraph,
    /// Genuine declaration keys indexed by their original source names.
    pub keys: BTreeMap<String, NodeKey>,
}

/// Parse, assemble and check the complete original source.
pub fn original(functions: &str) -> Original {
    let source = format!("language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{UNITS}\n{functions}");
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "quantity-operations"),
        "quantity-operations.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original quantity source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let forms = qsl_forms::build_unit(&parsed).expect("original quantity source forms");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/quantity-operations".to_owned(),
        forms,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("original units assemble");
    let units = declarations.units.clone();
    let keys = declarations
        .nominal_spans
        .iter()
        .map(|(key, span)| (source[span.start..span.end].to_owned(), key.clone()))
        .collect();
    let package = CheckedPackage::link(
        declarations
            .check(CheckingLimits::default())
            .expect("original quantity source checks"),
    );
    Original {
        functions: functions.to_owned(),
        package,
        units,
        keys,
    }
}

/// Small quantity limits with independently selected integer and work ceilings.
pub fn limits(integer_bits: u64, work_units: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits,
        decimal_digits: 0,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences: 2,
        work_units,
        result_units: 1,
    }
}

/// Evaluate the original closed body or its actual left checked child.
pub fn evaluate(
    original: &Original,
    name: &str,
    equality_child: bool,
    limits: ScalarLimits,
) -> (quire_eval::Evaluation, Meter) {
    let graph = original.package.graph();
    let function = graph
        .function_by_identity(
            graph
                .function_identity(name)
                .expect("original function identity"),
        )
        .expect("original function");
    let node = if equality_child {
        let children = function.body.children();
        let [left, _] = children.as_slice() else {
            panic!("original equality has two children");
        };
        *left
    } else {
        function.body
    };
    let mut meter = Meter::new(limits);
    let evaluation =
        qsl_eval::value::evaluate_closed(&original.package, node, function.slots, &mut meter)
            .expect("original closed body evaluates");
    (evaluation, meter)
}

/// Compare real generated symbols to fresh original checked operations.
pub fn generation(
    original: &Original,
    cases: &[(&str, bool, ScalarLimits)],
    integer_bound_refusals: &[&str],
    compound_result_refusals: &[&str],
) -> usize {
    use quire_contract_codegen::{ClaimDisposition, ExactScalarRefusal, OperationProvenance};
    let emission =
        qsl_package::emit_checked(&original.package).expect("original quantity source emits");
    assert!(emission.omitted().is_empty(), "{:?}", emission.omitted());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("original quantity IR admits");
    };
    let mut nodes = BTreeMap::new();
    let mut items = Vec::new();
    for (name, child, _) in cases {
        if nodes.contains_key(name) {
            continue;
        }
        let body = super::family_function_selection::body(&package, name);
        let node = if *child {
            let Some(CheckedTerm::Application { arguments, .. }) = package.term(&body) else {
                panic!("original equality application");
            };
            let [CheckedTerm::Reference { target }, _] = arguments.as_slice() else {
                panic!("original quantity child reference: {arguments:?}");
            };
            target.clone()
        } else {
            body
        };
        let mut results = derive_exact_scalar_items(&package, std::slice::from_ref(&node));
        assert_eq!(results.len(), 1);
        match results.pop().expect("one derived operation") {
            Ok(item) => {
                assert!(!integer_bound_refusals.contains(name));
                items.push(item);
            }
            Err(ExactScalarRefusal::RequiresBound { unbounded_type }) => {
                assert!(
                    integer_bound_refusals.contains(name),
                    "unexpected bound refusal: {name}"
                );
                let refused = package
                    .graph()
                    .nodes
                    .iter()
                    .find(|node| node.node_id == unbounded_type)
                    .expect("actual refused graph type");
                assert_eq!(refused.node_tag.as_ref(), "scalar_type");
                assert_eq!(refused.semantic_form.as_ref(), "integer");
                eprintln!("{name}: RequiresBound on genuine scalar_type/integer exponent; generated agreement Planned");
            }
            Err(other) => panic!("{name}: unexpected derivation refusal: {other:?}"),
        }
        nodes.insert(*name, node);
    }
    let authority = self::original(&original.functions);
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("quantity generation boundary");
    assert!(output.claim_map.blocked.is_empty());
    assert_eq!(output.claim_map.items.len(), items.len());
    let mut driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Meter,Outcome,Quantity,Value,ScalarLimits}};
use quire_semantic_value::quantity::UnitTable;
fn lift(outcome: Outcome<Quantity>)->Outcome<Value> {{ match outcome {{ Outcome::Completed(v)=>Outcome::Completed(Value::Quantity(v)),Outcome::Undefined(v)=>Outcome::Undefined(v),Outcome::Refused(v)=>Outcome::Refused(v),Outcome::Incomplete(v)=>Outcome::Incomplete(v) }} }}
#[test]
fn observe() {{ let graph={}; let table=UnitTable::declared(&graph);
"#,
        native_units(original)
    );
    let mut expected = Vec::new();
    for (name, child, limits) in cases {
        if integer_bound_refusals.contains(name) {
            continue;
        }
        let claims = output
            .claim_map
            .items
            .iter()
            .filter(|claim| claim.node_id == nodes[name])
            .collect::<Vec<_>>();
        let [claim] = claims.as_slice() else {
            panic!("one exact original claim: {name}");
        };
        if compound_result_refusals.contains(name) {
            assert!(
                matches!(&claim.result, ClaimDisposition::Refused {
                refusal: ExactScalarRefusal::ResultTypeMismatch {
                    expected: quire_contract_codegen::ScalarForm::Unit, found,
                }
            } if found.as_deref()==Some("compound_unit")),
                "{name}: {:?}",
                claim.result
            );
            eprintln!("{name}: ResultTypeMismatch expected Unit, found compound_unit; generated agreement Planned");
            continue;
        }
        let ClaimDisposition::Generated(generated) = &claim.result else {
            panic!("{name}: {:?}", claim.result);
        };
        assert_eq!(claim.operation.provenance, OperationProvenance::IrConfirmed);
        let graph = authority.package.graph();
        let function = graph
            .function_by_identity(graph.function_identity(name).expect("original identity"))
            .expect("original function");
        let operation = if *child {
            let children = function.body.children();
            assert_eq!(children.len(), 2);
            children[0]
        } else {
            function.body
        };
        let operands = operation.children();
        assert_eq!(operands.len(), 2);
        let mut arguments = Vec::new();
        driver.push_str("{\n");
        for (index, operand) in operands.into_iter().enumerate() {
            let mut input_meter = Meter::new(self::limits(1024, 1024));
            let input = qsl_eval::value::evaluate_closed(
                &authority.package,
                operand,
                function.slots,
                &mut input_meter,
            )
            .expect("authentic literal operand");
            assert!(input.losses.is_empty());
            let quire_eval::FamilyOutcome::Evaluated(quire_exact::Outcome::Completed(value)) =
                input.outcome
            else {
                panic!("closed operand completes");
            };
            match value {
                quire_exact::Value::Quantity(quantity) => {
                    let key = quantity
                        .unit()
                        .declared_key()
                        .expect("original literal uses declared unit");
                    // The emitted declared quantity type must join the original QSL key.
                    assert!(package
                        .graph()
                        .nodes
                        .iter()
                        .any(|node| same_key(&node.node_id, key)));
                    driver.push_str(&format!("let input{index}=Quantity::new({},quire_exact::UnitId::declared({})); let arg{index}=table.resolve(&input{index}).expect(\"original operand unit\");\n",native_rational(quantity.magnitude()),native_key(key)));
                    arguments.push(format!("arg{index}"));
                }
                quire_exact::Value::Integer(integer) => {
                    driver.push_str(&format!("let arg{index}:quire_exact::Integer={:?}.parse().expect(\"original exponent\");\n",integer.to_string()));
                    arguments.push(format!("&arg{index}"));
                }
                other => panic!("original quantity operand: {other:?}"),
            }
        }
        let (evaluation, meter) = evaluate(&authority, name, *child, *limits);
        let quire_eval::FamilyOutcome::Evaluated(outcome) = evaluation.outcome else {
            panic!("original quantity operation family");
        };
        let losses = evaluation
            .losses
            .into_iter()
            .map(|loss| loss.loss)
            .collect::<Vec<_>>();
        assert!(losses.is_empty());
        expected.push(super::oracle_observation::snapshot(
            &outcome, &losses, &meter,
        ));
        driver.push_str(&format!(r#"let mut meter=Meter::new({limits:?});
let (actual,identified)=quire_exact_scalar_oracles::{}({}, &mut meter).expect("actual generated quantity operation");
if let Outcome::Completed(ref quantity)=actual {{ assert_eq!(quantity.unit(), &identified.id()); }}
let actual=lift(actual); let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new();
println!("IT019_OBSERVATION {{}}",oracle_observation::snapshot(&actual,&losses,&meter)); }}
"#,generated.symbol,arguments.join(",")));
    }
    driver.push_str("}\n");
    if expected.is_empty() {
        return 0;
    }
    let actual = super::generated_oracle::execute(&output.artifacts, &driver);
    assert_eq!(
        actual, expected,
        "complete original QSL/native quantity observations"
    );
    actual.len()
}

/// Carry each component of an already admitted structural key into the native caller.
fn native_key(key: &NodeKey) -> String {
    let segments = key
        .path()
        .iter()
        .map(|segment| match segment {
            quire_exact::PathSegment::Text(text) => {
                format!("quire_exact::PathSegment::Text({text:?}.to_owned())")
            }
            quire_exact::PathSegment::Node(node) => {
                format!("quire_exact::PathSegment::Node({})", native_key(node))
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "quire_exact::NodeKey::new({:?},vec![{segments}],{}).expect(\"transported admitted key\")",
        key.package(),
        key.ordinal()
    )
}

/// Re-admit genuine producer-owned nominal metadata with the owner's public constructors.
fn native_units(original: &Original) -> String {
    use quire_semantic_value::unit::NominalUnitForm;
    let mut driver = String::from("{ use quire_semantic_value::unit::{NominalDeclaration,DimensionNode,UnitNode,UnitGraph}; let mut dimensions=std::collections::BTreeMap::new(); let mut units=std::collections::BTreeMap::new();");
    let table = quire_semantic_value::quantity::UnitTable::declared(&original.units);
    for node in original
        .keys
        .values()
        .filter_map(|key| table.nominal_node(key))
    {
        let declaration = format!(
            "NominalDeclaration {{ qualified_declaration:vec![{}],preimage:vec!{:?} }}",
            node.declaration
                .qualified_declaration
                .iter()
                .map(|part| format!("{part:?}.to_owned()"))
                .collect::<Vec<_>>()
                .join(","),
            node.declaration.preimage
        );
        let key = native_key(&node.key);
        match &node.form {
            NominalUnitForm::Dimension { terms } => {
                let terms = terms
                    .iter()
                    .map(|(key, power)| {
                        format!(
                            "({}, {:?}.parse().expect(\"original exponent\"))",
                            native_key(key),
                            power.to_string()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                driver.push_str(&format!("dimensions.insert({key},DimensionNode::checked(vec![{terms}],{declaration}).expect(\"original dimension metadata\"));"));
            }
            NominalUnitForm::Unit {
                dimension,
                target,
                scale,
                offset,
            } => {
                let target = target.as_ref().map_or_else(
                    || "None".to_owned(),
                    |key| format!("Some({})", native_key(key)),
                );
                driver.push_str(&format!("units.insert({key},UnitNode::checked({},{target},{},{},{declaration}).expect(\"original unit metadata\"));",native_key(dimension),native_rational(scale),native_rational(offset)));
            }
        }
    }
    driver.push_str(
        "UnitGraph::from_checked_nodes(&dimensions,&units).expect(\"original nominal graph\") }",
    );
    driver
}

fn same_key(id: &quire_contract_ir::CheckedNodeId, key: &NodeKey) -> bool {
    use quire_contract_ir::CheckedNodePathSegment;
    use quire_exact::PathSegment;
    let path = key.path();
    id.package.as_ref() == key.package()
        && id.ordinal == key.ordinal()
        && id.path.len() == path.len()
        && id
            .path
            .iter()
            .zip(&path)
            .all(|(left, right)| match (left, right) {
                (CheckedNodePathSegment::Name(left), PathSegment::Text(right)) => {
                    left.as_ref() == right
                }
                (CheckedNodePathSegment::Node(left), PathSegment::Node(right)) => {
                    same_key(left, right)
                }
                _ => false,
            })
}

/// Check the independently required small root-unit operation schedule.
pub fn assert_ledger(meter: &Meter, read_bits: &[u64], retained: bool) {
    use quire_exact::{AdmittedCharge, ChargePoint, LimitKind};
    let charge = |point, integer_bits, value_occurrences| AdmittedCharge {
        point,
        integer_bits,
        value_occurrences,
        decimal_digits: None,
        scale_expansion: None,
        text_input_bytes: None,
        text_scalars: None,
        normalized_scalars: None,
        unit_edges: None,
    };
    let mut expected = read_bits
        .iter()
        .enumerate()
        .map(|(index, bits)| {
            charge(
                ChargePoint::UnitIdentityRead,
                Some(*bits),
                Some(u64::try_from(index + 1).expect("small operand count")),
            )
        })
        .collect::<Vec<_>>();
    expected.push(charge(ChargePoint::UnitRationalArithmetic, Some(4), None));
    expected.push(charge(ChargePoint::UnitTargetDomain, None, None));
    if retained {
        expected.push(charge(ChargePoint::UnitResultRetain, None, Some(1)));
    }
    assert_eq!(meter.admitted_charges(), expected);
    for kind in LimitKind::ALL {
        let expected = match kind {
            LimitKind::IntegerBits => 4,
            LimitKind::ValueOccurrences => {
                u64::try_from(read_bits.len()).expect("small read count")
            }
            LimitKind::WorkUnits => u64::try_from(expected.len()).expect("small charge count"),
            LimitKind::ResultUnits => u64::from(retained),
            LimitKind::DecimalDigits
            | LimitKind::ScaleExpansion
            | LimitKind::TextInputBytes
            | LimitKind::TextScalars
            | LimitKind::NormalizedScalars
            | LimitKind::UnitEdges => 0,
        };
        assert_eq!(meter.consumed(kind), expected, "{kind:?}");
    }
}

fn native_rational(value: &quire_exact::Rational) -> String {
    format!("quire_exact::Rational::new({:?}.parse().expect(\"original numerator\"),{:?}.parse().expect(\"original denominator\")).expect(\"original rational\")",value.numerator().to_string(),value.denominator().to_string())
}
