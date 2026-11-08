// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public U32 original-source namespace selection and exact refusals.

#[path = "support/tc187_p8_source.rs"]
mod source;

use ix_trace_rs::trace;
use qsl_foundation::Code;
use qsl_semantics::check::{AssemblyCause, AssemblyLimits, PackageDeclarations};
use qsl_semantics::model::intake::SelectedModels;
use quire_exact::{Cancel, ValueType};
use quire_semantic_value::checking::CheckingLimits;

fn u32() {
    source::vector(
        "U32",
        "Declare a unit `ms` and a record `ms` and use `ms` in a type position and as `Quantity<ms>`; declare two units named `ms` in one linked package and use `Quantity<ms>`; declare `record Quantity` (FR-142-AC-13)",
        "the first checks, the type position meaning the record and `Quantity<ms>` the unit; the second refused `ambiguous_declaration`/`ambiguous-name` at `Quantity<ms>`; the third refused `invalid_syntax` at the name",
    );
}

#[trace("TC-107", "QSpec-TC-187", "QSpec-FR-142-AC-13")]
#[test]
fn ordinary_ms_type_and_quantity_ms_select_distinct_original_nodes() {
    u32();
    let text = format!(
        "{}{}record ms {{ ready: Boolean; }}\n\
         function inspect using v(row: ms, delay: Quantity<ms>): Boolean pure {{ row.ready }}",
        source::HEADER,
        source::UNITS,
    );
    let (raw, unit) = source::parsed("namespace", &text);
    let checked = PackageDeclarations::assemble(
        raw,
        "agent-ix/tc187-p8".to_owned(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("separate namespaces assemble")
    .check(CheckingLimits::default())
    .expect("record and quantity parameters check");
    let callable = checked.callable("inspect").expect("original function");
    assert!(matches!(
        callable.parameters,
        [(_, ValueType::Composite(_)), (_, ValueType::Quantity(_))]
    ));
    let admitted = source::emitted(checked, None);
    let nodes = &admitted.resolved_graph().nodes;
    let named: Vec<_> = nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>()
                    == ["ms"]
            })
        })
        .collect();
    assert_eq!(named.len(), 2, "both original ms declarations emitted");
    let record = named
        .iter()
        .find(|node| node.semantic_form.as_ref() == "record")
        .expect("ms record node");
    let unit = named
        .iter()
        .find(|node| node.semantic_form.as_ref() == "unit")
        .expect("ms unit node");
    assert_ne!(record.node_id, unit.node_id);
    let function = nodes
        .iter()
        .find(|node| {
            node.declaration.as_ref().is_some_and(|decl| {
                decl.qualified_name
                    .last()
                    .is_some_and(|name| name.as_ref() == "inspect")
            })
        })
        .expect("inspect emitted");
    let quire_contract_ir::CheckedTerm::Aggregate { members } = admitted
        .term(&function.node_id)
        .expect("original function term")
    else {
        panic!("function aggregate");
    };
    let quire_contract_ir::CheckedTerm::Binding { name, value } = &members[0] else {
        panic!("parameters binding");
    };
    assert_eq!(name.as_ref(), "parameters");
    let quire_contract_ir::CheckedTerm::Aggregate {
        members: parameters,
    } = value.as_ref()
    else {
        panic!("parameter group: {value:?}");
    };
    assert_eq!(parameters.len(), 2);
    for (parameter, expected) in parameters.iter().zip([record, unit]) {
        let quire_contract_ir::CheckedTerm::Reference { target } = parameter else {
            panic!("original parameter reference");
        };
        let parameter = nodes
            .iter()
            .find(|node| &node.node_id == target)
            .expect("original parameter node");
        assert_eq!(parameter.semantic_type, expected.node_id);
    }
}

#[trace("TC-107", "QSpec-TC-187", "QSpec-FR-142-AC-13")]
#[test]
fn linked_duplicate_ms_units_expose_the_quantity_type_location_blocker() {
    u32();
    let first = format!(
        "{}dimension A; unit ms : A = rational(1, 1);",
        source::HEADER
    );
    let second = format!("{}dimension B; unit ms : B = rational(1, 1);\nfunction f using v(d: Quantity<ms>): Boolean pure {{ true }}", source::HEADER);
    let refusal = PackageDeclarations::assemble_units(
        "agent-ix/tc187-p8-linked".to_owned(),
        vec![
            source::parsed("first", &first),
            source::parsed("second", &second),
        ],
        SelectedModels::default(),
        Vec::new(),
        AssemblyLimits::default(),
        &Cancel::new(),
    )
    .expect_err("duplicate units refuse the linked package");
    let [error, unresolved] = refusal.errors.as_slice() else {
        panic!("duplicate declaration and unresolved use: {refusal:?}");
    };
    assert_eq!(
        error.cause.catalog_code().to_string(),
        "ambiguous_declaration/ambiguous-name"
    );
    let AssemblyCause::DuplicateQuantityName { name, spans } = &error.cause else {
        panic!("native duplicate-name payload: {error:?}");
    };
    assert_eq!(name, "ms");
    let first_name = first.find("ms :").expect("first unit name");
    let second_name = second.find("ms :").expect("second unit name");
    assert_eq!(
        spans
            .iter()
            .map(|span| (span.start, span.end))
            .collect::<Vec<_>>(),
        [(first_name, first_name + 2), (second_name, second_name + 2)]
    );
    assert_eq!(
        (error.span.start, error.span.end),
        (second_name, second_name + 2)
    );
    let required = second.find("Quantity<ms>").expect("quantity use");
    let AssemblyCause::UnresolvedUnitName { name, packages } = &unresolved.cause else {
        panic!("native unresolved quantity use: {unresolved:?}");
    };
    assert_eq!(name, "ms");
    assert!(packages.is_empty());
    assert_eq!(
        (unresolved.span.start, unresolved.span.end),
        (required + "Quantity<".len(), required + "Quantity<ms".len())
    );
    assert_ne!((error.span.start, error.span.end), (required, required + "Quantity<ms>".len()), "U32 remains incomplete: owner refuses the duplicate declaration before resolving the quantity type");
}

#[trace("TC-107", "QSpec-TC-187", "QSpec-FR-142-AC-13")]
#[test]
fn quantity_is_reserved_at_the_record_name() {
    u32();
    let text = format!("{}record Quantity {{ ready: Boolean; }}", source::HEADER);
    let parsed = qsl_cst::parse(
        qsl_foundation::SourceIdentity::new("agent-ix", "reserved"),
        "reserved.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("parser returns located diagnostics");
    assert!(!parsed.is_admissible());
    let [diagnostic] = parsed.diagnostics() else {
        panic!("one reserved-name refusal: {:?}", parsed.diagnostics());
    };
    assert_eq!(diagnostic.code, Code::InvalidSyntax);
    let region = diagnostic.region.as_ref().expect("reserved name location");
    let start = text.find("Quantity").expect("record name");
    assert_eq!(
        (region.start(), region.end()),
        (start as u64, (start + "Quantity".len()) as u64)
    );
}
