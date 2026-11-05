// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source checking and separate native value refusal observations for TC-187.

mod support;

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_semantics::{
    check::{CheckCause, CheckRefusal, PackageDeclarations},
    model::{
        accounting::ModelNormalizationLimits,
        intake::{admit_unit, package_input, SelectedModels},
    },
};
use quire_exact::{
    IllTyped, IllTypedCause, Integer, LimitKind, Meter, Quantity, Rational, ScalarLimits,
};
use quire_semantic_value::{
    checking::CheckingLimits,
    location::{Location, Origin},
    quantity::{evaluate_quantity_unit, QuantityOperation, QuantityRefusal, UnitTable},
    unit::{NominalUnitForm, Unit},
};
use quire_specification::{vectors_of, TestCase};
use serde_json::{json, Value};

const LT: &str =
    "dimension L; dimension T; unit m : L = rational(1, 1); unit s : T = rational(1, 1);";
const TEMPERATURE: &str = "dimension Temperature; unit K : Temperature = rational(1, 1); unit degC : Temperature = rational(1, 1) * K + decimal(27315, 2); unit degF : Temperature = rational(5, 9) * K + rational(45967, 180);";

fn vector(id: &str, inputs: &str, expected: &str) {
    let rows: Vec<_> = vectors_of(TestCase::Tc187)
        .filter(|row| row.id == id)
        .collect();
    let [row] = rows.as_slice() else {
        panic!("exactly one public row {id}: {rows:?}")
    };
    assert_eq!(row.inputs, inputs, "{id} complete input");
    assert_eq!(row.expected, expected, "{id} complete expectation");
}

fn model_document() -> Vec<u8> {
    let bytes =
        std::fs::read(support::qsl_root().join("tests/fixtures/spine-model.semantic-ir.json"))
            .expect("owner model fixture");
    let mut model: Value = serde_json::from_slice(&bytes).expect("owner model JSON");
    assert_eq!(model["package"]["identity"], "acme/orders");
    let origin = model["types"][0]["origin"].clone();
    model["dimensions"] = json!([{"identity":"ix://acme/orders/dimension/L", "name":"L", "base":true, "terms":[], "origin":origin}]);
    model["units"] = json!([{"identity":"ix://acme/orders/unit/m", "name":"m", "dimension":"ix://acme/orders/dimension/L", "scale":"1", "offset":"0", "origin":origin}]);
    serde_json::to_vec(&model).expect("model serialization")
}

fn assemble(declarations: &str, expression: &str, imported: bool) -> (String, PackageDeclarations) {
    let selection = if imported {
        "model M = \"acme/orders\";"
    } else {
        ""
    };
    let text = format!("language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\n{selection}\n{declarations}\nfunction bad using v(): Boolean pure {{ ({expression}) = 1 m }}");
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "tc187-p8-refusals"),
        "tc187-p8-refusals.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("source parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let forms = qsl_forms::build_unit(&parsed).expect("source forms");
    let models = if imported {
        let document = model_document();
        admit_unit(
            &forms.selections().models,
            &package_input([document.as_slice()]).expect("model identity"),
            ModelNormalizationLimits::default(),
        )
        .expect("selected model admission")
    } else {
        SelectedModels::default()
    };
    let assembled = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "agent-ix/tc187-p8-refusals".to_owned(),
        forms,
        models,
        Vec::new(),
    )
    .expect("source assembly");
    (text, assembled)
}

fn declared<'a>(package: &'a PackageDeclarations, text: &str, name: &str) -> &'a Unit {
    let keys: Vec<_> = package
        .nominal_spans
        .iter()
        .filter(|(_, span)| &text[span.start..span.end] == name)
        .map(|(key, _)| key.clone())
        .collect();
    let [key] = keys.as_slice() else {
        panic!("one declared unit {name}: {keys:?}")
    };
    package
        .units
        .unit(key.clone())
        .expect("original native unit")
}

fn check(text: &str, package: PackageDeclarations, expression: &str, cause: IllTypedCause) {
    let regions = package.regions();
    let source = package.source.clone();
    let refusals = package
        .check(CheckingLimits::default())
        .expect_err("invalid source must refuse checking");
    assert_eq!(
        refusals,
        vec![CheckRefusal {
            location: Location::at(
                Origin::Body {
                    function: "bad".to_owned(),
                    index: 0
                },
                &[0]
            ),
            cause: CheckCause::IllTyped(cause)
        }]
    );
    let refusal = &refusals[0];
    let expected = if cause == IllTypedCause::OperatorIneligible {
        "ill_typed/operator-ineligible"
    } else {
        "ill_typed/type-mismatch"
    };
    assert_eq!(
        refusal.cause.catalog_code().map(|code| code.to_string()),
        Some(expected.to_owned())
    );
    let region = regions
        .region(&refusal.location)
        .expect("original expression region");
    assert_eq!(region.source(), &source);
    let start = text.rfind(expression).expect("original expression");
    assert_eq!(
        (region.start(), region.end()),
        (
            u64::try_from(start).expect("start"),
            u64::try_from(start + expression.len()).expect("end")
        )
    );
}

fn limits(allowance: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: allowance,
        decimal_digits: allowance,
        scale_expansion: allowance,
        text_input_bytes: allowance,
        text_scalars: allowance,
        normalized_scalars: allowance,
        unit_edges: allowance,
        value_occurrences: allowance,
        work_units: allowance,
        result_units: allowance,
    }
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

impl Operator {
    fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Power => "^",
        }
    }
}

fn native(
    package: &PackageDeclarations,
    left: &Unit,
    right: &Unit,
    operator: Operator,
    cause: IllTypedCause,
) {
    let table = UnitTable::declared(&package.units);
    let one = Rational::from_integer(Integer::one());
    let a = Quantity::new(one.clone(), left.id());
    let b = Quantity::new(one, right.id());
    let exponent = Integer::from(2);
    for allowance in [u64::MAX, 0] {
        let a = table.resolve(&a).expect("original left unit ID");
        let b = table.resolve(&b).expect("original right unit ID");
        let operation = match operator {
            Operator::Add => QuantityOperation::Add(a, b),
            Operator::Subtract => QuantityOperation::Subtract(a, b),
            Operator::Multiply => QuantityOperation::Multiply(a, b),
            Operator::Divide => QuantityOperation::Divide(a, b),
            Operator::Power => QuantityOperation::Power(a, &exponent),
        };
        let mut meter = Meter::new(limits(allowance));
        assert_eq!(
            evaluate_quantity_unit(operation, &mut meter),
            Err(QuantityRefusal::IllTyped(IllTyped { cause })),
            "{operator:?}, allowance {allowance}"
        );
        assert!(
            meter.admitted_charges().is_empty(),
            "{operator:?}: ordered charges"
        );
        assert_eq!(LimitKind::ALL.len(), 10);
        for kind in LimitKind::ALL {
            assert_eq!(meter.consumed(kind), 0, "{operator:?}: {kind:?}");
        }
    }
}

#[trace("QSpec-TC-187", "TC-108")]
#[test]
fn incompatible_length_and_time_refuse_at_original_addition() {
    vector(
        "U03",
        "Add `1 m` to `1 s` where `s` has time dimension `T`",
        "`refused { code: ill_typed }` for incompatible dimensions before arithmetic",
    );
    let expression = "1 m + 1 s";
    let (text, package) = assemble(LT, expression, false);
    native(
        &package,
        declared(&package, &text, "m"),
        declared(&package, &text, "s"),
        Operator::Add,
        IllTypedCause::IncompatibleDimensions,
    );
    check(&text, package, expression, IllTypedCause::TypeMismatch);
}

#[trace("QSpec-TC-187", "TC-108")]
#[test]
fn all_ten_affine_arithmetic_cases_refuse_before_value_charges() {
    vector("U05", "Add, subtract, multiply, divide or raise a `degC`/`degF` value to a power", "`refused { code: ill_typed }` because nonzero-offset points have no implicit difference-unit arithmetic");
    for name in ["degC", "degF"] {
        for operator in [
            Operator::Add,
            Operator::Subtract,
            Operator::Multiply,
            Operator::Divide,
            Operator::Power,
        ] {
            let expression = if matches!(operator, Operator::Power) {
                format!("1 {name} ^ 2")
            } else {
                format!("1 {name} {} 1 {name}", operator.symbol())
            };
            let (text, package) = assemble(&format!("{LT}{TEMPERATURE}"), &expression, false);
            let point = declared(&package, &text, name);
            assert_ne!(point.offset(), &Rational::from_integer(Integer::zero()));
            native(
                &package,
                point,
                point,
                operator,
                IllTypedCause::AffineUnitArithmetic,
            );
            let cause = if matches!(operator, Operator::Power) {
                IllTypedCause::OperatorIneligible
            } else {
                IllTypedCause::TypeMismatch
            };
            check(&text, package, &expression, cause);
        }
    }
}

#[trace("QSpec-TC-187", "TC-108")]
#[test]
fn same_named_base_dimensions_keep_distinct_actual_owners() {
    vector(
        "U09",
        "Use a base-dimension name/shape equal to `L` but a different I04 semantic-node key",
        "`refused { code: ill_typed }` as incompatible despite structural resemblance",
    );
    let expression = "1 m + 1 M::m";
    let (text, package) = assemble(
        "dimension L; unit m : L = rational(1, 1);",
        expression,
        true,
    );
    let local = declared(&package, &text, "m");
    let imported: Vec<_> = package
        .units
        .units()
        .filter(|unit| unit.id() != local.id())
        .collect();
    let [imported] = imported.as_slice() else {
        panic!("one imported root: {imported:?}")
    };
    assert_ne!(local.id(), imported.id());
    assert_ne!(local.dimension_node(), imported.dimension_node());
    let table = UnitTable::declared(&package.units);
    for unit in [local, *imported] {
        let node = table
            .nominal_node(&unit.dimension_node())
            .expect("actual dimension declaration");
        assert_eq!(
            node.declaration
                .qualified_declaration
                .last()
                .map(String::as_str),
            Some("L")
        );
        assert_eq!(node.form, NominalUnitForm::Dimension { terms: Vec::new() });
        assert_eq!(
            unit.dimension().exponents().collect::<Vec<_>>(),
            vec![(unit.dimension_node(), &Integer::one())]
        );
        assert_eq!(unit.scale(), &Rational::from_integer(Integer::one()));
        assert_eq!(unit.offset(), &Rational::from_integer(Integer::zero()));
        assert_eq!(unit.root(), unit.key());
    }
    native(
        &package,
        local,
        imported,
        Operator::Add,
        IllTypedCause::IncompatibleDimensions,
    );
    check(&text, package, expression, IllTypedCause::TypeMismatch);
}
