//! Tests that deserialize captured EPEE binary responses from real monerod nodes.
//!
//! The binary fixtures in `tests/vectors/` were captured from a stagenet node.

use monero_epee_bin_serde::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Base fields present in all monerod binary RPC responses.
#[derive(Clone, Debug, Deserialize, PartialEq)]
struct BaseResponse {
    credits: u64,
    status: String,
    top_hash: String,
    untrusted: bool,
}

/// Response from `get_o_indexes.bin` — uses `#[serde(flatten)]` for deserialization.
#[derive(Clone, Debug, Deserialize, PartialEq)]
struct GetOIndexesResponse {
    #[serde(flatten)]
    base: BaseResponse,
    #[serde(default)]
    o_indexes: Vec<u64>,
}

/// Flat version of `GetOIndexesResponse` without `#[serde(flatten)]`.
/// EPEE's serializer does not support serde enums / flatten, so we use
/// this for round-trip serialization tests.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct GetOIndexesResponseFlat {
    credits: u64,
    #[serde(default)]
    o_indexes: Vec<u64>,
    status: String,
    top_hash: String,
    untrusted: bool,
}

/// Load the captured binary file and deserialize it.
#[test]
fn deserialize_captured_get_o_indexes_bin() {
    let bin = include_bytes!("vectors/get_o_indexes_bin_0.bin");

    let response: GetOIndexesResponse = from_bytes(bin).expect("deserialization should succeed");

    // Verify base fields
    assert_eq!(response.base.status, "OK");
    assert_eq!(response.base.credits, 0);
    assert_eq!(response.base.top_hash, "");
    assert!(!response.base.untrusted);

    // Verify o_indexes — the captured TX had exactly 2 output indexes
    assert_eq!(response.o_indexes.len(), 2);
    assert_eq!(response.o_indexes[0], 9541662);
    assert_eq!(response.o_indexes[1], 9541663);
}

/// Verify that the meta JSON agrees with the binary payload.
#[test]
fn meta_json_matches_binary() {
    let meta_json = include_str!("vectors/get_o_indexes_bin_0_meta.json");
    let meta: serde_json::Value = serde_json::from_str(meta_json).expect("meta JSON should parse");

    // The meta file records the raw hex encoding of the binary payload.
    let raw_hex = meta["raw_hex"].as_str().expect("raw_hex field");
    let from_hex = hex::decode(raw_hex).expect("valid hex");

    let from_file = include_bytes!("vectors/get_o_indexes_bin_0.bin");
    assert_eq!(
        from_hex.as_slice(),
        from_file.as_slice(),
        "binary file should match the hex recorded in the meta JSON"
    );

    // The meta file also records the expected length.
    let expected_len = meta["length"].as_u64().expect("length field") as usize;
    assert_eq!(from_file.len(), expected_len);
}

/// Serialize the deserialized structure back to bytes and verify round-trip.
///
/// Uses the flat struct variant because EPEE's serializer does not support
/// `#[serde(flatten)]`. We compare by deserializing both and checking field
/// equality rather than byte-equality (field ordering may differ from monerod).
#[test]
fn round_trip_get_o_indexes_bin() {
    let original_bytes = include_bytes!("vectors/get_o_indexes_bin_0.bin");

    // Deserialize using the flat struct
    let response: GetOIndexesResponseFlat =
        from_bytes(original_bytes).expect("deserialization should succeed");

    assert_eq!(response.status, "OK");
    assert_eq!(response.o_indexes, vec![9541662, 9541663]);

    // Re-serialize
    let re_serialized = to_bytes(&response).expect("re-serialization should succeed");

    // Deserialize the re-serialized bytes
    let round_tripped: GetOIndexesResponseFlat =
        from_bytes(&re_serialized).expect("round-trip deserialization should succeed");

    assert_eq!(
        response, round_tripped,
        "round-trip should preserve all fields"
    );
}

/// Verify that an inline hex-encoded "Failed" response also deserializes correctly,
/// confirming we handle both success and failure cases from captured data.
#[test]
fn deserialize_failed_response_has_empty_o_indexes() {
    // This is the "Failed" response from the existing test, kept as a cross-check.
    let failed_hex = "011101010101020101100763726564697473050000000000000000067374617475730a184661696c656408746f705f686173680a0009756e747275737465640b00";
    let failed_bytes = hex::decode(failed_hex).expect("valid hex");

    let response: GetOIndexesResponse =
        from_bytes(&failed_bytes).expect("deserialization should succeed");

    assert_eq!(response.base.status, "Failed");
    assert!(
        response.o_indexes.is_empty(),
        "failed response should have no o_indexes"
    );
}

/// Helper: deserialize, compare to expected, then round-trip via the flat struct.
fn assert_round_trip(bytes: &[u8], expected: &GetOIndexesResponseFlat) {
    let parsed: GetOIndexesResponseFlat =
        from_bytes(bytes).expect("deserialization should succeed");
    assert_eq!(&parsed, expected, "parsed value should match expected");

    let re_serialized = to_bytes(&parsed).expect("re-serialization should succeed");
    let round_tripped: GetOIndexesResponseFlat =
        from_bytes(&re_serialized).expect("round-trip deserialization should succeed");
    assert_eq!(&parsed, &round_tripped, "round-trip should preserve value");
}

#[test]
fn round_trip_helper_works_on_captured_vector() {
    let bin = include_bytes!("vectors/get_o_indexes_bin_0.bin");

    let expected = GetOIndexesResponseFlat {
        credits: 0,
        o_indexes: vec![9541662, 9541663],
        status: "OK".to_owned(),
        top_hash: "".to_string(),
        untrusted: false,
    };

    assert_round_trip(bin.as_slice(), &expected);
}
