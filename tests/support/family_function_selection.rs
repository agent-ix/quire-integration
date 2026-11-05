// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked-package declaration and body selection shared by host and native tests.

use super::{CheckedNodeId, CheckedPackageV2, CheckedTerm};

/// Select exactly one original function declaration, without a first-match fallback.
pub fn declared(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    let selected = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .iter()
                    .map(AsRef::as_ref)
                    .eq([name])
            })
        })
        .collect::<Vec<_>>();
    let [node] = selected.as_slice() else {
        panic!("one exact authored declaration {name}");
    };
    node.node_id.clone()
}

/// Retain the selected declaration's one actual typed body-reference target.
pub fn body(package: &CheckedPackageV2, name: &str) -> CheckedNodeId {
    let declaration = declared(package, name);
    let Some(CheckedTerm::Aggregate { members }) = package.term(&declaration) else {
        panic!("the selected declaration has a typed function header");
    };
    let selected = members
        .iter()
        .filter_map(|member| match member {
            CheckedTerm::Binding { name, value } if name.as_ref() == "body" => match &**value {
                CheckedTerm::Reference { target } => Some(target.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    let [body] = selected.as_slice() else {
        panic!("one original typed body reference");
    };
    body.clone()
}
