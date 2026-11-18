use monero_epee_bin_serde::{from_bytes, to_bytes};
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01";
const MAX_DEPTH: usize = 64;

#[derive(Debug, PartialEq)]
struct ByteBlob(Vec<u8>);

impl Serialize for ByteBlob {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for ByteBlob {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BlobVisitor;
        impl<'de> Visitor<'de> for BlobVisitor {
            type Value = ByteBlob;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a byte buffer")
            }

            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Ok(ByteBlob(v))
            }

            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(ByteBlob(v.to_vec()))
            }
        }
        deserializer.deserialize_byte_buf(BlobVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Nested {
    number: u64,
    children: Vec<Leaf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Leaf {
    flag: bool,
    bytes: ByteBlob,
}

#[derive(Debug, Deserialize)]
struct Empty {}

fn nested_bytes() -> Vec<u8> {
    to_bytes(&Nested {
        number: 17,
        children: vec![
            Leaf {
                flag: true,
                bytes: ByteBlob(vec![0, 255, 1]),
            },
            Leaf {
                flag: false,
                bytes: ByteBlob(vec![]),
            },
        ],
    })
    .unwrap()
}

#[test]
fn accepts_nested_objects_arrays_blobs_and_unknown_fields() {
    let bytes = nested_bytes();
    let decoded: Nested = from_bytes(&bytes).unwrap();
    assert_eq!(decoded.number, 17);
    assert_eq!(decoded.children.len(), 2);
    from_bytes::<Empty, _>(&bytes).unwrap();
    // Fixed wire vector: {x: {y: 42u64}}.
    let fixed = hex::decode("0111010101010201010401780c040179052a00000000000000").unwrap();
    from_bytes::<Empty, _>(fixed).unwrap();
}

#[test]
fn every_truncation_returns_an_error_without_panicking() {
    let bytes = nested_bytes();
    for end in 0..bytes.len() {
        assert!(from_bytes::<Empty, _>(&bytes[..end]).is_err(), "end={end}");
    }
    // Each length encoding, including missing first byte and partial u64.
    for marker in [0u8, 1, 2, 3] {
        let mut bytes = HEADER.to_vec();
        bytes.extend_from_slice(&[4, 1, b'x', 10, marker]);
        if marker == 0 {
            bytes.pop();
        }
        assert!(from_bytes::<Empty, _>(bytes).is_err());
    }
}

#[test]
fn rejects_recursive_unknown_objects_and_object_arrays() {
    for array in [false, true] {
        let mut bytes = HEADER.to_vec();
        for _ in 0..MAX_DEPTH + 2 {
            bytes.extend_from_slice(&[4, 1, b'x', if array { 0x8c } else { 12 }]);
            if array {
                bytes.push(4);
            }
        }
        bytes.push(0);
        assert!(from_bytes::<Empty, _>(bytes)
            .unwrap_err()
            .to_string()
            .contains("nesting"));
    }
}

#[test]
fn accepts_objects_nested_up_to_the_depth_limit() {
    fn nested_objects(levels: usize) -> Vec<u8> {
        let mut bytes = HEADER.to_vec();
        for _ in 0..levels {
            bytes.extend_from_slice(&[4, 1, b'x', 12]);
        }
        bytes.push(0);
        bytes
    }

    from_bytes::<Empty, _>(nested_objects(MAX_DEPTH)).unwrap();
    assert!(from_bytes::<Empty, _>(nested_objects(MAX_DEPTH + 1))
        .unwrap_err()
        .to_string()
        .contains("nesting"));
}

#[test]
fn rejects_impossible_lengths() {
    // Above MAX_STRING_LEN_POSSIBLE.
    let mut bytes = HEADER.to_vec();
    bytes.extend_from_slice(&[4, 1, b'x', 10]);
    bytes.extend_from_slice(&[255; 8]);
    assert!(from_bytes::<Empty, _>(&bytes).is_err());

    // Below MAX_STRING_LEN_POSSIBLE, which must fail on the missing bytes without the declared
    // length being allocated.
    let mut huge_len = HEADER.to_vec();
    huge_len.extend_from_slice(&[4, 1, b'x', 10]);
    huge_len.extend_from_slice(&((1_500_000_000u64 << 2) | 3).to_le_bytes());
    assert!(from_bytes::<Empty, _>(&huge_len).is_err());
}

#[test]
fn valid_wire_types_mismatched_with_rust_types_return_without_panicking() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Field<T> {
        x: T,
    }

    for marker in 1u8..=12 {
        let value = match marker {
            1 | 5 | 9 => vec![0; 8],
            2 | 6 => vec![0; 4],
            3 | 7 => vec![0; 2],
            4 | 8 | 11 => vec![0],
            10 => vec![8, 0, 255],         // two arbitrary blob bytes
            12 => vec![4, 1, b'y', 8, 42], // nested {y: 42u8}
            _ => unreachable!(),
        };
        for array in [false, true] {
            for size_tag in 0..4u64 {
                let mut wire = HEADER.to_vec();
                let encoded = (4u64 | size_tag).to_le_bytes();
                wire.extend_from_slice(&encoded[..(1usize << size_tag)]);
                wire.extend_from_slice(&[1, b'x', marker | if array { 128 } else { 0 }]);
                if array {
                    wire.push(8);
                }
                wire.extend_from_slice(&value);
                if array {
                    wire.extend_from_slice(&value);
                }
                from_bytes::<Empty, _>(&wire).unwrap();
                let _ = from_bytes::<Field<u64>, _>(&wire);
                let _ = from_bytes::<Field<bool>, _>(&wire);
                let _ = from_bytes::<Field<String>, _>(&wire);
                let _ = from_bytes::<Field<[u8; 32]>, _>(&wire);
                let _ = from_bytes::<Field<Vec<u8>>, _>(&wire);
                let _ = from_bytes::<Field<Vec<Empty>>, _>(&wire);
                let _ = from_bytes::<Field<Option<Empty>>, _>(&wire);
                let _ = from_bytes::<Field<ByteBlob>, _>(&wire);
            }
        }
    }
}

#[test]
fn rejects_trailing_bytes() {
    let mut trailing = nested_bytes();
    trailing.push(0);
    assert!(from_bytes::<Empty, _>(trailing)
        .unwrap_err()
        .to_string()
        .contains("Trailing"));
}
