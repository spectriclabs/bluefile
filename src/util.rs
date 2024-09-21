use std::fs::File;
use std::path::PathBuf;

use num::complex::Complex;

use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;

pub fn open_file(path: &PathBuf) -> Result<File> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path.display().to_string())),
    };
    Ok(file)
}

pub(crate) fn byte_to_i8(v: u8) -> Result<i8> {
    match i8::try_from(v) {
        Ok(x) => Ok(x),
        Err(_) => return Err(Error::ByteConversionError),
    }
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

pub(crate) fn bytes_to_i64(v: &[u8], endianness: Endianness) -> Result<i64> {
    let b: [u8; 8] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(i64::from_le_bytes(b))
    } else {
        Ok(i64::from_be_bytes(b))
    }
}

pub(crate) fn bytes_to_f32(v: &[u8], endianness: Endianness) -> Result<f32> {
    let b: [u8; 4] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(f32::from_le_bytes(b))
    } else {
        Ok(f32::from_be_bytes(b))
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

pub(crate) fn bytes_to_complex_i8(v: &[u8]) -> Result<Complex<i8>> {
    let real: i8 = byte_to_i8(v[0])?;
    let imag: i8 = byte_to_i8(v[1])?;
    Ok(Complex::<i8>::new(real, imag))
}

pub(crate) fn bytes_to_complex_i16(v: &[u8], endianness: Endianness) -> Result<Complex<i16>> {
    let real: i16 = bytes_to_i16(&v[0..2], endianness)?;
    let imag: i16 = bytes_to_i16(&v[2..4], endianness)?;
    Ok(Complex::<i16>::new(real, imag))
}

pub(crate) fn bytes_to_complex_i32(v: &[u8], endianness: Endianness) -> Result<Complex<i32>> {
    let real: i32 = bytes_to_i32(&v[0..4], endianness)?;
    let imag: i32 = bytes_to_i32(&v[4..8], endianness)?;
    Ok(Complex::<i32>::new(real, imag))
}

pub(crate) fn bytes_to_complex_i64(v: &[u8], endianness: Endianness) -> Result<Complex<i64>> {
    let real: i64 = bytes_to_i64(&v[0..8], endianness)?;
    let imag: i64 = bytes_to_i64(&v[8..16], endianness)?;
    Ok(Complex::<i64>::new(real, imag))
}

pub(crate) fn bytes_to_complex_f32(v: &[u8], endianness: Endianness) -> Result<Complex<f32>> {
    let real: f32 = bytes_to_f32(&v[0..4], endianness)?;
    let imag: f32 = bytes_to_f32(&v[4..8], endianness)?;
    Ok(Complex::<f32>::new(real, imag))
}

pub(crate) fn bytes_to_complex_f64(v: &[u8], endianness: Endianness) -> Result<Complex<f64>> {
    let real: f64 = bytes_to_f64(&v[0..8], endianness)?;
    let imag: f64 = bytes_to_f64(&v[8..16], endianness)?;
    Ok(Complex::<f64>::new(real, imag))
}
