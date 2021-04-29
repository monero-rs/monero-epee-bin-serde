use hex_literal::hex;
use monero_epee_bin_serde::from_bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::Bytes;
use std::fmt::Debug;

#[test]
fn get_o_indexes() {
    test_from_bytes(&[
        (&hex!("011101010101020101140763726564697473050000000000000000096f5f696e64657865738504a900000000000000067374617475730a084f4b08746f705f686173680a0009756e747275737465640b00"), GetOIndexesResponse {
            base: BaseResponse {
                credits: 0,
                status: "OK".to_owned(),
                top_hash: "".to_string(),
                untrusted: false,
            },
            o_indexes: vec![169],
        }),
        (&hex!("011101010101020101100763726564697473050000000000000000067374617475730a184661696c656408746f705f686173680a0009756e747275737465640b00"), GetOIndexesResponse {
            base: BaseResponse {
                credits: 0,
                status: "Failed".to_owned(),
                top_hash: "".to_string(),
                untrusted: false,
            },
            o_indexes: vec![],
        })
    ])
}

#[test]
fn get_outs() {
    test_from_bytes(&[
        (&hex!("011101010101020101140763726564697473050000000000000000046f7574738c04140668656967687405a100000000000000036b65790a802d392d0be38eb4699c17767e62a063b8d2f989ec15c80e5d2665ab06f8397439046d61736b0a805e8b863c5b267deda13f4bc5d5ec8e59043028380f2431bc8691c15c83e1fea404747869640a80c0646e065a33b849f0d9563673ca48eb0c603fe721dd982720dba463172c246f08756e6c6f636b65640b00067374617475730a084f4b08746f705f686173680a0009756e747275737465640b00"), GetOutsResponse {
            base: BaseResponse {
                credits: 0,
                status: "OK".to_owned(),
                top_hash: "".to_string(),
                untrusted: false
            },
            outs: vec![
                OutKey {
                    height: 161,
                    key: hex!("2d392d0be38eb4699c17767e62a063b8d2f989ec15c80e5d2665ab06f8397439"),
                    mask: hex!("5e8b863c5b267deda13f4bc5d5ec8e59043028380f2431bc8691c15c83e1fea4"),
                    txid: hex!("c0646e065a33b849f0d9563673ca48eb0c603fe721dd982720dba463172c246f"),
                    unlocked: false
                }
            ]
        }),
    ])
}

fn test_from_bytes<T>(cases: &[(&[u8], T)])
where
    T: DeserializeOwned + PartialEq + Debug,
{
    for (bytes, expected) in cases {
        let response = from_bytes::<T, _>(bytes).unwrap();

        assert_eq!(&response, expected)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct BaseResponse {
    credits: u64,
    status: String,
    top_hash: String,
    untrusted: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct GetOIndexesResponse {
    #[serde(flatten)]
    base: BaseResponse,
    #[serde(default)]
    o_indexes: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct GetOutsResponse {
    #[serde(flatten)]
    base: BaseResponse,
    outs: Vec<OutKey>,
}

// We currently don't support tuples outside of byte slices via `deserialize_bytes`. Need to use `serde_as` to make sure we opt into the supported behaviour.
#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
struct OutKey {
    height: u64,
    #[serde_as(as = "Bytes")]
    key: [u8; 32],
    #[serde_as(as = "Bytes")]
    mask: [u8; 32],
    #[serde_as(as = "Bytes")]
    txid: [u8; 32],
    unlocked: bool,
}
