use byteorder::{LittleEndian, ReadBytesExt};
use std::io;

pub fn encode(number: usize) -> Vec<u8> {
    const BITS_FOR_SIZE: u32 = 2;

    const FITS_IN_ONE_BYTE: usize = 64; // 2usize.pow(8 - BITS_FOR_SIZE);
    const FITS_IN_TWO_BYTES: usize = 16384; // 2usize.pow(16 - BITS_FOR_SIZE);
    const FITS_IN_FOUR_BYTES: usize = 1073741824; // 2usize.pow(32 - BITS_FOR_SIZE);

    let size_marker = if number < FITS_IN_ONE_BYTE {
        0
    } else if number < FITS_IN_TWO_BYTES {
        1
    } else if number < FITS_IN_FOUR_BYTES {
        2
    } else {
        3
    };

    let number = number << BITS_FOR_SIZE; // make space for the size marker
    let number = number | size_marker; // store the size marker in the number

    match size_marker {
        0 => vec![number as u8],
        1 => (number as u16).to_le_bytes().to_vec(),
        2 => (number as u32).to_le_bytes().to_vec(),
        3 => (number as u64).to_le_bytes().to_vec(),
        _ => unreachable!(),
    }
}

pub fn decode(stream: &mut impl io::BufRead) -> Result<usize, io::Error> {
    let v = stream.fill_buf()?[0];

    let mask = v & 0x03;

    let number = match mask {
        0 => stream.read_u8()? as usize,
        1 => stream.read_u16::<LittleEndian>()? as usize,
        2 => stream.read_u32::<LittleEndian>()? as usize,
        3 => stream.read_u64::<LittleEndian>()? as usize,
        mask => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected mask {}", mask),
            ))
        }
    };

    Ok(number >> 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_var_int() {
        let test_cases = &[
            (0, "00"),
            (1, "04"),
            (2, "08"),
            (3, "0c"),
            (32, "80"),
            (64, "0101"),
            (100, "9101"),
            (123123123, "ced65a1d"),
            (9999999999999999, "ffff03bfc91b8e00"),
            (6472923, "6e138b01"),
        ];

        for (number, expected) in test_cases {
            let actual = hex::encode(&encode(*number));

            assert_eq!(&actual, expected)
        }
    }
}
