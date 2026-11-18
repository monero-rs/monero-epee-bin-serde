//! A [`serde`](https://docs.rs/serde) library that implements Monero's epee binary encoding [[0],
//! [1]].
//!
//! [0]: https://github.com/monero-project/monero/blob/0a1ddc2eff854f3e932203a95b65a9f1efd60eef/contrib/epee/include/storages/portable_storage_from_bin.h
//! [1]: https://github.com/monero-project/monero/blob/0a1ddc2eff854f3e932203a95b65a9f1efd60eef/contrib/epee/include/storages/portable_storage_to_bin.h

#![forbid(unsafe_code)]

#[cfg(feature = "container_as_blob")]
pub mod container_as_blob;
mod de;
mod error;
mod ser;
mod varint;

pub use crate::error::Error;

use crate::de::Deserializer;
use crate::ser::Serializer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use std::io::Read;

/// A specialized [`Result`] type for serde operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Header that needs to be at the beginning of every binary blob that follows
/// this binary serialization format.
const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01";
/// The maximum length a byte array (marked as a string) can be.
const MAX_STRING_LEN_POSSIBLE: usize = 2000000000;

/// Serialize the given object to binary.
///
/// This function will prepend the magic header bytes to the serialized object.
/// Additionally, the passed in object MUST be a struct. Monero's RPC interface assumes that the root element is a struct without tagging it as such.
pub fn to_bytes<T>(object: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    buffer.extend_from_slice(HEADER);

    let mut serializer = Serializer::new_root(&mut buffer);
    object.serialize(&mut serializer)?;

    Ok(buffer)
}

/// Deserialize the provided bytes.
///
/// This function assumes that the bytes are prepended with the magic header and will fail otherwise.
pub fn from_bytes<T, B>(bytes: B) -> Result<T>
where
    T: DeserializeOwned,
    B: AsRef<[u8]>,
{
    let mut bytes = bytes.as_ref();

    let mut header = [0u8; 9];
    bytes.read_exact(&mut header)?;

    let has_header = header == HEADER;

    if !has_header {
        return Err(Error::missing_header_bytes());
    }

    let mut deserializer = Deserializer::new(&mut bytes);

    T::deserialize(&mut deserializer)
}

const MARKER_SINGLE_I64: Marker = Marker::Single { value: 1 };
const MARKER_SINGLE_I32: Marker = Marker::Single { value: 2 };
const MARKER_SINGLE_I16: Marker = Marker::Single { value: 3 };
const MARKER_SINGLE_I8: Marker = Marker::Single { value: 4 };
const MARKER_SINGLE_U64: Marker = Marker::Single { value: 5 };
const MARKER_SINGLE_U32: Marker = Marker::Single { value: 6 };
const MARKER_SINGLE_U16: Marker = Marker::Single { value: 7 };
const MARKER_U8: u8 = 8;
const MARKER_SINGLE_U8: Marker = Marker::Single { value: MARKER_U8 };
const MARKER_SINGLE_F64: Marker = Marker::Single { value: 9 };
const MARKER_SINGLE_STRING: Marker = Marker::Single { value: 10 };
const MARKER_SINGLE_BOOL: Marker = Marker::Single { value: 11 };
const MARKER_SINGLE_STRUCT: Marker = Marker::Single { value: 12 };
const MARKER_ARRAY_ELEMENT: u8 = 0x80;

#[derive(Debug, PartialEq, Eq)]
enum Marker {
    Single { value: u8 },
    Sequence { element: u8 },
}

impl Marker {
    fn from_byte(value: u8) -> Self {
        let is_sequence = value & MARKER_ARRAY_ELEMENT > 0;

        if is_sequence {
            return Self::Sequence {
                element: value ^ MARKER_ARRAY_ELEMENT,
            };
        }

        Self::Single { value }
    }

    fn to_sequence(&self) -> Self {
        match *self {
            Marker::Single { value } => Marker::Sequence { element: value },
            Marker::Sequence { element } => Marker::Sequence { element },
        }
    }

    fn to_byte(&self) -> u8 {
        match *self {
            Marker::Single { value } => value,
            Marker::Sequence { element } => element | MARKER_ARRAY_ELEMENT,
        }
    }
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single { value } => write!(f, "Single({:x})", value),
            Self::Sequence { element } => write!(f, "Sequence({:x})", element),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize)]
    struct RootStruct {
        foo: u64,
        bars: Vec<Bar>,
    }

    #[derive(Serialize)]
    struct Bar {
        number: u64,
    }

    #[test]
    fn nested_struct_has_struct_marker() {
        let bytes = to_bytes(&RootStruct {
            foo: 100,
            bars: vec![Bar { number: 1 }, Bar { number: 2 }],
        })
        .unwrap();

        let payload = &bytes[9..]; // remove magic bytes

        assert_eq!(&payload, b"\x08\x03foo\x05\x64\x00\x00\x00\x00\x00\x00\x00\x04bars\x8c\x08\x04\x06number\x05\x01\x00\x00\x00\x00\x00\x00\x00\x04\x06number\x05\x02\x00\x00\x00\x00\x00\x00\x00")
    }

    #[test]
    fn root_element_must_be_struct() {
        to_bytes(&1u64).unwrap_err();
        to_bytes(&[1u64]).unwrap_err();
        to_bytes(&true).unwrap_err();
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct AccountBase {
        m_creation_timestamp: u64,
        m_keys: AccountKeys,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct AccountKeys {
        m_account_address: AccountAddress,
        #[serde(with = "serde_bytes")]
        m_spend_secret_key: Vec<u8>,
        #[serde(with = "serde_bytes")]
        m_view_secret_key: Vec<u8>,
        #[serde(with = "serde_bytes")]
        m_encryption_iv: Vec<u8>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct AccountAddress {
        #[serde(with = "serde_bytes")]
        m_spend_public_key: Vec<u8>,
        #[serde(with = "serde_bytes")]
        m_view_public_key: Vec<u8>,
    }

    #[test]
    fn nested_struct_roundtrip() {
        let original = AccountBase {
            m_creation_timestamp: 1402600416,
            m_keys: AccountKeys {
                m_account_address: AccountAddress {
                    m_spend_public_key: vec![0xaa; 32],
                    m_view_public_key: vec![0xbb; 32],
                },
                m_spend_secret_key: vec![0x11; 32],
                m_view_secret_key: vec![0x22; 32],
                m_encryption_iv: vec![0x00; 8],
            },
        };

        let bytes = to_bytes(&original).unwrap();
        let restored: AccountBase = from_bytes(&bytes).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn nested_struct_deser_consumes_marker() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Outer {
            value: u64,
            inner: Inner,
        }
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Inner {
            x: u32,
            y: u32,
        }

        let original = Outer {
            value: 42,
            inner: Inner { x: 1, y: 2 },
        };

        let bytes = to_bytes(&original).unwrap();

        let payload = &bytes[9..];
        let inner_name = b"\x05inner";
        let pos = payload
            .windows(inner_name.len())
            .position(|w| w == inner_name)
            .unwrap();
        assert_eq!(payload[pos + inner_name.len()], 0x0c);

        let restored: Outer = from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }
}
