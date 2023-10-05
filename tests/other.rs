use monero_epee_bin_serde::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, PartialEq, Debug)]
struct TestSeq {
    seq: Vec<u32>,
}

#[test]
fn empty_sequence() {
    let obj = TestSeq::default();
    let data = to_bytes(&obj).unwrap();
    assert_eq!(obj, from_bytes(data).unwrap());
}

#[derive(Default, Deserialize, Serialize, PartialEq, Debug)]
struct TestOptional {
    #[serde(skip_serializing_if = "Option::is_none")]
    val: Option<u8>,
}

#[test]
fn optional_val_present() {
    let val = TestOptional { val: Some(1) };
    let buf = to_bytes(&val).unwrap();
    let val2 = from_bytes(&buf).unwrap();
    assert_eq!(val, val2);
}

#[test]
fn optional_val_not_present() {
    let val = TestOptional::default();
    let buf = to_bytes(&val).unwrap();
    let val2 = from_bytes(&buf).unwrap();
    assert_eq!(val, val2);
}
