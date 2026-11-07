// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict admission to the original integration-owned bridge definition.

use ix_trace_rs::trace;
use quire_integration::temporal::{
    read_tl_bridge_definition, TlBridgeDefinitionError as Refusal, TlLimits, TL_BRIDGE_DEFINITION,
};

fn limits(max_bytes: u64) -> TlLimits {
    TlLimits {
        max_bytes,
        ..TlLimits::default()
    }
}

#[trace("IT-022-SC-09", "FR-350-AC-3", "FR-350-AC-11")]
#[test]
fn authored_definition_selects_exact_members_and_canonical_bytes() {
    let exact = u64::try_from(TL_BRIDGE_DEFINITION.len()).unwrap();
    let admitted = read_tl_bridge_definition(TL_BRIDGE_DEFINITION, limits(exact)).unwrap();
    assert_eq!(admitted.authority(), "agent-ix/quire-integration");
    assert_eq!(admitted.identity(), "native-temporal-correspondence/v1");
    assert_eq!(
        admitted.profile(),
        "quire.native-temporal-correspondence/v1"
    );
    assert_eq!(admitted.version(), 1);
    assert_eq!(admitted.bytes(), TL_BRIDGE_DEFINITION);
    assert_eq!(admitted.usage().bytes, exact);
    assert_eq!(
        read_tl_bridge_definition(TL_BRIDGE_DEFINITION, limits(exact - 1)),
        Err(Refusal::LimitReached {
            value: exact,
            setting: exact - 1
        })
    );
    for (field, replacement, expected) in [
        (
            "authority",
            serde_json::json!("foreign"),
            Refusal::Authority,
        ),
        (
            "identity",
            serde_json::json!("another-definition"),
            Refusal::Identity,
        ),
        (
            "profile",
            serde_json::json!("another-profile"),
            Refusal::Profile,
        ),
        ("version", serde_json::json!(2), Refusal::Version),
    ] {
        let mut changed: serde_json::Value = serde_json::from_slice(TL_BRIDGE_DEFINITION).unwrap();
        changed[field] = replacement;
        let bytes = quire_canonical::to_vec(&changed, quire_canonical::Limits::new(1024)).unwrap();
        assert_eq!(
            read_tl_bridge_definition(&bytes, limits(1024)),
            Err(expected),
            "{field}"
        );
    }
    let mut trailing = TL_BRIDGE_DEFINITION.to_vec();
    trailing.push(b' ');
    assert_eq!(
        read_tl_bridge_definition(&trailing, limits(1024)),
        Err(Refusal::Canonical)
    );
    let duplicate = [
        b"{\"authority\":\"agent-ix/quire-integration\",".as_slice(),
        &TL_BRIDGE_DEFINITION[1..],
    ]
    .concat();
    assert_eq!(
        read_tl_bridge_definition(&duplicate, limits(1024)),
        Err(Refusal::Shape)
    );
    let mut value: serde_json::Value = serde_json::from_slice(TL_BRIDGE_DEFINITION).unwrap();
    value.as_object_mut().unwrap().remove("identity");
    assert_eq!(
        read_tl_bridge_definition(&serde_json::to_vec(&value).unwrap(), limits(1024)),
        Err(Refusal::Shape)
    );
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), true.into());
    assert_eq!(
        read_tl_bridge_definition(&serde_json::to_vec(&value).unwrap(), limits(1024)),
        Err(Refusal::Shape)
    );
}
