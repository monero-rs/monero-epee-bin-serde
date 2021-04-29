mod de;
mod error;
mod ser;
mod varint;

pub use crate::error::Error;

use crate::de::Deserializer;
use crate::ser::Serializer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;

pub type Result<T> = std::result::Result<T, Error>;

/// Header that needs to be at the beginning of every binary blob that follows
/// this binary serialization format.
const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01";

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

const MARKER_I64: u8 = 1;
const MARKER_I32: u8 = 2;
const MARKER_I16: u8 = 3;
const MARKER_I8: u8 = 4;
const MARKER_U64: u8 = 5;
const MARKER_U32: u8 = 6;
const MARKER_U16: u8 = 7;
const MARKER_U8: u8 = 8;
const MARKER_F64: u8 = 9;
const MARKER_STRING: u8 = 10;
const MARKER_BOOL: u8 = 11;
const MARKER_STRUCT: u8 = 12;
const MARKER_ARRAY_ELEMENT: u8 = 0x80;

#[cfg(test)]
mod tests {
    use super::*;

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
}
