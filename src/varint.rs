use byteorder::{LittleEndian, ReadBytesExt};
use std::io;

pub fn encode(number: usize) -> Vec<u8> {
    const BITS_FOR_SIZE: u32 = 2;

    const FITS_IN_ONE_BYTE: u64 = 64; // 2u64.pow(8 - BITS_FOR_SIZE);
    const FITS_IN_TWO_BYTES: u64 = 16_384; // 2u64.pow(16 - BITS_FOR_SIZE);
    const FITS_IN_FOUR_BYTES: u64 = 1_073_741_824; // 2u64.pow(32 - BITS_FOR_SIZE);

    let n = number as u64;
    let size_marker: u64 = if n < FITS_IN_ONE_BYTE {
        0
    } else if n < FITS_IN_TWO_BYTES {
        1
    } else if n < FITS_IN_FOUR_BYTES {
        2
    } else {
        3
    };

    let encoded = (n << BITS_FOR_SIZE) | size_marker;

    match size_marker {
        0 => vec![encoded as u8],
        1 => (encoded as u16).to_le_bytes().to_vec(),
        2 => (encoded as u32).to_le_bytes().to_vec(),
        3 => encoded.to_le_bytes().to_vec(),
        _ => unreachable!(),
    }
}

pub fn decode(stream: &mut impl io::BufRead) -> Result<usize, io::Error> {
    let v = *stream.fill_buf()?.first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "unexpected EOF reading varint",
        )
    })?;

    let raw: u64 = match v & 0x03 {
        0 => u64::from(stream.read_u8()?),
        1 => u64::from(stream.read_u16::<LittleEndian>()?),
        2 => u64::from(stream.read_u32::<LittleEndian>()?),
        3 => stream.read_u64::<LittleEndian>()?,
        _ => unreachable!(),
    };

    usize::try_from(raw >> 2)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "varint value exceeds usize"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_var_int() {
        let test_cases: &[(usize, &str)] = &[
            (0, "00"),
            (1, "04"),
            (2, "08"),
            (3, "0c"),
            (32, "80"),
            (64, "0101"),
            (100, "9101"),
            (123123123, "ced65a1d"),
            (6472923, "6e138b01"),
            (1_073_741_824, "0300000001000000"),
        ];

        for (number, expected) in test_cases {
            let actual = hex::encode(encode(*number));

            assert_eq!(&actual, expected);

            let encoded = encode(*number);
            let mut reader = encoded.as_slice();
            assert_eq!(decode(&mut reader).unwrap(), *number);
            assert!(reader.is_empty());
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_encode_decode_64bit_var_int() {
        let number = 9_999_999_999_999_999usize;
        let encoded = encode(number);
        assert_eq!(hex::encode(encoded.as_slice()), "ffff03bfc91b8e00");
        assert_eq!(decode(&mut encoded.as_slice()).unwrap(), number);
    }

    #[test]
    fn test_decode_empty_stream_returns_unexpected_eof() {
        let mut empty: &[u8] = &[];
        let err = decode(&mut empty).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_decode_truncated_varints_return_unexpected_eof() {
        for bytes in [
            &[0x01u8][..],
            &[0x02][..],
            &[0x02, 0x00, 0x00][..],
            &[0x03][..],
            &[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..],
        ] {
            let mut reader = bytes;
            let err = decode(&mut reader).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
        }
    }

    #[test]
    fn test_decode_overwide_encodings() {
        // Value 1 encoded with 1-, 2-, 4-, and 8-byte tags.
        for encoded in [
            &[0x04u8][..],
            &[0x05, 0x00][..],
            &[0x06, 0x00, 0x00, 0x00][..],
            &[0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..],
        ] {
            let mut reader = encoded;
            assert_eq!(decode(&mut reader).unwrap(), 1);
            assert!(reader.is_empty());
        }
    }
}
