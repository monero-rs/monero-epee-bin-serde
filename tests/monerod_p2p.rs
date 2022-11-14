use hex_literal::hex;
use monero_epee_bin_serde::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TryFromInto;
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BasicNodeData {
    pub my_port: u32,
    pub network_id: [u8; 16],
    pub peer_id: u64,
    pub support_flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_credits_per_hash: Option<u32>,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CoreSyncData {
    pub cumulative_difficulty: u64,
    pub cumulative_difficulty_top64: u64,
    pub current_height: u64,
    pub pruning_seed: u32,
    #[serde_as(as = "TryFromInto<[u8; 32]>")]
    pub top_id: monero::Hash,
    pub top_version: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HandshakeR {
    pub node_data: BasicNodeData,
    pub payload_data: CoreSyncData,
}

#[test]
fn received_handshake() {
    let bytes = hex!("01110101010102010108096e6f64655f646174610c10076d795f706f727406a04600000a6e6574776f726b5f69640a401230f171610441611731008216a1a11007706565725f6964053eb3c096c4471c340d737570706f72745f666c61677306010000000c7061796c6f61645f646174610c181563756d756c61746976655f646966666963756c7479053951f7a79aab4a031b63756d756c61746976655f646966666963756c74795f746f7036340500000000000000000e63757272656e745f68656967687405fa092a00000000000c7072756e696e675f73656564068001000006746f705f69640a806cc497b230ba57a95edb370be8d6870c94e0992937c89b1def3a4cb7726d37ad0b746f705f76657273696f6e0810");
    let decoded_handshake = from_bytes::<HandshakeR, _>(bytes).unwrap();

    let handshake = HandshakeR {
        node_data: BasicNodeData {
            network_id: [
                18, 48, 241, 113, 97, 4, 65, 97, 23, 49, 0, 130, 22, 161, 161, 16,
            ],
            my_port: 18080,
            rpc_port: None,
            rpc_credits_per_hash: None,
            peer_id: 3754955098988524350,
            support_flags: 1,
        },
        payload_data: CoreSyncData {
            current_height: 2755066,
            cumulative_difficulty: 237190611121688889,
            cumulative_difficulty_top64: 0,
            top_id: monero::Hash {
                0: hex!("6cc497b230ba57a95edb370be8d6870c94e0992937c89b1def3a4cb7726d37ad"),
            },
            top_version: 16,
            pruning_seed: 384,
        },
    };
    assert_eq!(decoded_handshake, handshake);
    let encoded_handshake = to_bytes(&handshake).unwrap();
    assert_eq!(encoded_handshake, bytes);
}
