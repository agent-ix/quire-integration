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
    declarations: String,
    /// Original parsed expression trees, retained for public binder evaluation.
    #[allow(
        dead_code,
        reason = "only binder-boundary tests inspect original forms"
    )]
    pub expressions: BTreeMap<String, qsl_forms::Expression>,
    /// Original checked package; never reconstructed from a test body.
    pub package: CheckedPackage,
    /// Unit graph admitted from the original source declarations.
    pub units: UnitGraph,
    /// Genuine declaration keys indexed by their original source names.
    pub keys: BTreeMap<String, NodeKey>,
}

/// Parse, assemble and check the complete original source.
pub fn original(functions: &str) -> Original {
    original_with("", functions)
}

/// Check original functions with additional original unit declarations.
pub fn original_with(extra: &str, functions: &str) -> Original {
    let source = source_text(extra, functions);
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "quantity-operations"),
        "quantity-operations.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("original quantity source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let forms = qsl_forms::build_unit(&parsed).expect("original quantity source forms");
    let expressions = forms
        .forms()
        .iter()
        .filter_map(|form| match form.form() {
            qsl_forms::DeclarationForm::Function(function) => {
                Some((function.name.clone(), function.body.clone()))
            }
            _ => None,
        })
        .collect();
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
        declarations: extra.to_owned(),
        expressions,
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
    let evaluation = qsl_eval::value::evaluate_closed_with_frame(
        &original.package,
        node,
        function.slots,
        Vec::new(),
        function.formed_units,
        &mut meter,
    )
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
    generation_with(
        original,
        cases,
        integer_bound_refusals,
        compound_result_refusals,
        &[],
    )
}

/// Execute eligible original quantity families and preserve carried-target derivation stops.
pub fn generation_with(
    original: &Original,
    cases: &[(&str, bool, ScalarLimits)],
    integer_bound_refusals: &[&str],
    compound_result_refusals: &[&str],
    carried_refusals: &[&str],
) -> usize {
    use quire_contract_codegen::{ClaimDisposition, ExactScalarRefusal, OperationProvenance};
    let emission = qsl_package::emit_checked(
        &original.package,
        qsl_package::EmitLimits::new(qsl_replay::spine::EMIT_OUTPUT_BYTES_DEFAULT),
    )
    .expect("original quantity source emits");
    assert!(emission.omitted().is_empty(), "{:?}", emission.omitted());
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let read_result = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let package = match read_result {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        CheckedPackageV2ReadResult::Refused(refusal) if !carried_refusals.is_empty() => {
            assert!(cases
                .iter()
                .all(|(name, _, _)| carried_refusals.contains(name)));
            assert_eq!(
                refusal.code,
                quire_contract_ir::CheckedPackageRefusalCode::InvalidPackage
            );
            assert_eq!(
                refusal.cause,
                Some(quire_contract_ir::CheckedPackageRefusalCause::InvalidDerivedShape)
            );
            let locus = refusal.locus.as_ref().expect("actual carried type locus");
            assert_eq!(locus.path.len(), 2);
            assert!(
                matches!(&locus.path[..],[quire_contract_ir::CheckedNodePathSegment::Name(a),quire_contract_ir::CheckedNodePathSegment::Name(b)] if a.as_ref()=="scalar_type" && b.as_ref()=="carried_quantity")
            );
            eprintln!("CARRIED_READER {:?}: InvalidPackage/InvalidDerivedShape at original scalar_type/carried_quantity; no CG claim",carried_refusals);
            return 0;
        }
        other => panic!("original quantity IR admits: {other:?}"),
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
                assert!(
                    !carried_refusals.contains(name),
                    "carried source unexpectedly became derivable: {name}"
                );
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
            Err(other) if carried_refusals.contains(name) => {
                assert!(
                    matches!(&other,ExactScalarRefusal::NoDerivableClaim { reason: quire_contract_codegen::ClaimDerivationRefusal::OperandFormsNotDerivable {operation_identity} | quire_contract_codegen::ClaimDerivationRefusal::ModeNotDerivable {operation_identity} } if operation_identity=="quire.op.quantity.convert"),
                    "{name}: {other:?}"
                );
                eprintln!("CARRIED_DERIVATION {name}: {other:?}");
            }
            Err(other) => panic!("{name}: unexpected derivation refusal: {other:?}"),
        }
        nodes.insert(*name, node);
    }
    let authority = self::original_with(&original.declarations, &original.functions);
    let output =
        generate_exact_scalar_oracles(&package, &items).expect("quantity generation boundary");
    assert!(output.claim_map.blocked.is_empty());
    assert_eq!(output.claim_map.items.len(), items.len());
    let mut driver = format!(
        r#"
mod oracle_observation;
use quire_exact::{{Meter,Outcome,Quantity,Value,ScalarLimits}};
use quire_semantic_value::quantity::UnitTable;
fn lift<T>(outcome: Outcome<T>,value:impl FnOnce(T)->Value)->Outcome<Value> {{ match outcome {{ Outcome::Completed(v)=>Outcome::Completed(value(v)),Outcome::Undefined(v)=>Outcome::Undefined(v),Outcome::Refused(v)=>Outcome::Refused(v),Outcome::Incomplete(v)=>Outcome::Incomplete(v) }} }}
#[test]
fn observe() {{ let graph={}; let table=UnitTable::declared(&graph);
"#,
        native_units(original)
    );
    let mut expected = Vec::new();
    for (name, child, limits) in cases {
        if integer_bound_refusals.contains(name) || carried_refusals.contains(name) {
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
        let item = items
            .iter()
            .find(|item| item.node_id == nodes[name])
            .expect("derived original item");
        let conversion = matches!(
            item.operation,
            quire_contract_codegen::ExactScalarOperation::QuantityConversion
        );
        let comparison = matches!(
            item.operation,
            quire_contract_codegen::ExactScalarOperation::QuantityComparison { .. }
        );
        assert_eq!(operands.len(), if conversion { 1 } else { 2 });
        let mut arguments = Vec::new();
        driver.push_str("{\n");
        for (index, operand) in operands.into_iter().enumerate() {
            let mut input_meter = Meter::new(self::limits(1024, 1024));
            let input = qsl_eval::value::evaluate_closed_with_frame(
                &authority.package,
                operand,
                function.slots,
                Vec::new(),
                function.formed_units,
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
        if conversion {
            let quire_exact::ValueType::Quantity(unit_id) = operation.value_type() else {
                panic!("exact conversion result type");
            };
            let key = unit_id
                .declared_key()
                .expect("exact declared conversion target");
            driver.push_str(&format!("let target=quire_semantic_value::quantity::IdentifiedUnit::new(table.get(quire_exact::UnitId::declared({})).expect(\"original target\").clone()).expect(\"admitted target identity\");\n",native_key(key)));
            arguments.push("&target".to_owned());
        }
        driver.push_str(&format!("let mut meter=Meter::new({limits:?});\n"));
        if comparison {
            driver.push_str(&format!("let actual=quire_exact_scalar_oracles::{}({},&mut meter).expect(\"generated comparison\"); let actual=lift(actual,Value::Boolean);\n",generated.symbol,arguments.join(",")));
        } else if conversion {
            driver.push_str(&format!("let actual=quire_exact_scalar_oracles::{}({},&mut meter).expect(\"generated conversion\"); let actual=lift(actual,Value::Quantity);\n",generated.symbol,arguments.join(",")));
        } else {
            driver.push_str(&format!("let (actual,identified)=quire_exact_scalar_oracles::{}({},&mut meter).expect(\"generated arithmetic\"); if let Outcome::Completed(ref quantity)=actual {{ assert_eq!(quantity.unit(),&identified.id()); }} let actual=lift(actual,Value::Quantity);\n",generated.symbol,arguments.join(",")));
        }
        driver.push_str("let losses=Vec::<quire_semantic_value::loss::ValueLoss>::new(); println!(\"IT019_OBSERVATION {}\",oracle_observation::snapshot(&actual,&losses,&meter)); }\n");
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
            NominalUnitForm::Dimension { base, terms } => {
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
                let constructor = if *base { "checked" } else { "checked_derived" };
                driver.push_str(&format!("dimensions.insert({key},DimensionNode::{constructor}(vec![{terms}],{declaration}).expect(\"original dimension metadata\"));"));
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
    use quire_exact::ChargePoint;
    let charge = |point, integer_bits, value_occurrences| {
        self::charge(point, integer_bits, None, None, value_occurrences)
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
    accounting(
        meter,
        &expected,
        [
            4,
            0,
            0,
            u64::try_from(read_bits.len()).expect("small reads"),
            u64::try_from(expected.len()).expect("small ledger"),
            u64::from(retained),
        ],
    );
}

fn native_rational(value: &quire_exact::Rational) -> String {
    format!("quire_exact::Rational::new({:?}.parse().expect(\"original numerator\"),{:?}.parse().expect(\"original denominator\")).expect(\"original rational\")",value.numerator().to_string(),value.denominator().to_string())
}

/// Compare complete owner charge records without projecting away any amount.
pub fn assert_charges(meter: &Meter, expected: &[quire_exact::AdmittedCharge]) {
    assert_eq!(meter.admitted_charges(), expected);
}

/// Construct a complete independent event expectation for the quantity ledger.
pub fn charge(
    point: quire_exact::ChargePoint,
    bits: Option<u64>,
    digits: Option<u64>,
    edges: Option<u64>,
    occurrences: Option<u64>,
) -> quire_exact::AdmittedCharge {
    quire_exact::AdmittedCharge {
        point,
        integer_bits: bits,
        decimal_digits: digits,
        unit_edges: edges,
        value_occurrences: occurrences,
        scale_expansion: None,
        text_input_bytes: None,
        text_scalars: None,
        normalized_scalars: None,
    }
}
/// Check ordered records and all ten counters against independently selected totals.
pub fn accounting(meter: &Meter, events: &[quire_exact::AdmittedCharge], totals: [u64; 6]) {
    use quire_exact::LimitKind;
    assert_charges(meter, events);
    for kind in LimitKind::ALL {
        let expected = match kind {
            LimitKind::IntegerBits => totals[0],
            LimitKind::DecimalDigits => totals[1],
            LimitKind::UnitEdges => totals[2],
            LimitKind::ValueOccurrences => totals[3],
            LimitKind::WorkUnits => totals[4],
            LimitKind::ResultUnits => totals[5],
            LimitKind::ScaleExpansion
            | LimitKind::TextInputBytes
            | LimitKind::TextScalars
            | LimitKind::NormalizedScalars => 0,
        };
        assert_eq!(meter.consumed(kind), expected, "{kind:?}");
    }
}

/// Original source text shared by admitted and statically refused operation controls.
pub fn source_text(extra: &str, functions: &str) -> String {
    let source = format!("language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{UNITS}\n{extra}\n{functions}");
    source
}
