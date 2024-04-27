use std::fs::File;
use std::path::Path;

use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;

pub(crate) fn open_file(path: &Path) -> Result<File> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path.display().to_string())),
    };
    Ok(file)
}

pub(crate) fn bytes_to_i16(v: &[u8], endianness: Endianness) -> Result<i16> {
    let b: [u8; 2] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(i16::from_le_bytes(b))
    } else {
        Ok(i16::from_be_bytes(b))
    }
}

pub(crate) fn bytes_to_i32(v: &[u8], endianness: Endianness) -> Result<i32> {
    let b: [u8; 4] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(i32::from_le_bytes(b))
    } else {
        Ok(i32::from_be_bytes(b))
    }
}

pub(crate) fn bytes_to_f64(v: &[u8], endianness: Endianness) -> Result<f64> {
    let b: [u8; 8] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(f64::from_le_bytes(b))
    } else {
        Ok(f64::from_be_bytes(b))
    }
}