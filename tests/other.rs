use monero_epee_bin_serde::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, PartialEq, Debug)]
struct TestStruct {
    seq: Vec<u32>,
}

#[test]
fn empty_sequence() {
    let obj = TestStruct::default();
    let data = to_bytes(&obj).unwrap();
    assert_eq!(obj, from_bytes(&data).unwrap());
}
