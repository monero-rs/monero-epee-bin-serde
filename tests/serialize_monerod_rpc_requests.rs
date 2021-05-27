use monero_epee_bin_serde::to_bytes;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::TryFromInto;

#[test]
fn get_o_indexes() {
    let payload = GetOIndexesPayload {
        txid: "0bdd2418548da386d9594d2c7245fcdbb5212d3136a3e2170fe25d1c663af9ae"
            .parse()
            .unwrap(),
    };

    let serialized = to_bytes(&payload).unwrap();

    assert_eq!(
        serialized,
        vec![
            1, 17, 1, 1, 1, 1, 2, 1, 1, 4, 4, 116, 120, 105, 100, 10, 128, 11, 221, 36, 24, 84,
            141, 163, 134, 217, 89, 77, 44, 114, 69, 252, 219, 181, 33, 45, 49, 54, 163, 226, 23,
            15, 226, 93, 28, 102, 58, 249, 174
        ]
    );
}

#[serde_as]
#[derive(Debug, Serialize)]
struct GetOIndexesPayload {
    #[serde_as(as = "TryFromInto<[u8; 32]>")]
    txid: monero::Hash,
}
