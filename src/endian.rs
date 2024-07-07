use std::convert::TryFrom;

use crate::error::Error;

/// Defines endianness type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endianness {
    Big,
    Little,
}

/// Converts raw bytes to an Endianness enum type.
impl TryFrom<&[u8]> for Endianness {
    type Error = Error;

    fn try_from(v: &[u8]) -> std::result::Result<Self, Self::Error> {
        if v[0] == b'E' && v[1] == b'E' && v[2] == b'E' && v[3] == b'I' {
            Ok(Endianness::Little)
        } else if v[0] == b'I' && v[1] == b'E' && v[2] == b'E' && v[3] == b'E' {
            Ok(Endianness::Big)
        } else {
            Err(Error::InvalidEndianness)
        }
    }
}
