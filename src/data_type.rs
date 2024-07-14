use num::complex::Complex;

use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;
use crate::util::{
    byte_to_i8,
    bytes_to_i16,
    bytes_to_i32,
    bytes_to_i64,
    bytes_to_f32,
    bytes_to_f64,
    bytes_to_complex_i8,
    bytes_to_complex_i16,
    bytes_to_complex_i32,
    bytes_to_complex_i64,
    bytes_to_complex_f32,
    bytes_to_complex_f64,
};

/// Defines the rank of the data.
#[derive(Debug, Clone, PartialEq)]
pub enum Rank {
    Scalar,
    Complex,
}

/// Converts raw bytes to a Rank enum type.
impl TryFrom<u8> for Rank {
    type Error = Error;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            b'S' => Ok(Rank::Scalar),
            b'C' => Ok(Rank::Complex),
            _ => Err(Error::UnknownRankError),
        }
    }
}

/// Defines the number of elements required by each Rank enum type.
pub fn rank_multiple(r: &Rank) -> usize {
    match r {
        Rank::Scalar => 1,
        Rank::Complex => 2,
    }
}

/// Defines the format of the data.
#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Byte,
    Int,
    Long,
    LongLong,
    Float,
    Double,
}

/// Converts raw bytes to a Format enum type.
impl TryFrom<u8> for Format {
    type Error = Error;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            b'B' => Ok(Format::Byte),
            b'I' => Ok(Format::Int),
            b'L' => Ok(Format::Long),
            b'X' => Ok(Format::LongLong),
            b'F' => Ok(Format::Float),
            b'D' => Ok(Format::Double),
            _ => Err(Error::UnknownFormatError),
        }
    }
}

/// Defines the number of bytes required by each Format enum type.
pub fn format_size(f: &Format) -> usize {
    match f {
        Format::Byte => 1,
        Format::Int => 2,
        Format::Long => 4,
        Format::LongLong => 8,
        Format::Float => 4,
        Format::Double => 8,
    }
}

/// Combines the Rank and Format into a single struct so they can be easily passed around together.
#[derive(Debug, Clone)]
pub struct DataType {
    pub rank: Rank,
    pub format: Format,
}

impl DataType {
    /// Returns the total size in bytes for the DataType.
    pub fn size(&self) -> usize {
        rank_multiple(&self.rank) * format_size(&self.format)
    }
}

/// Wraps the Rust data value in the equivalent bluefile data type.
#[derive(Debug)]
pub enum DataValue {
    SB(i8),
    SI(i16),
    SL(i32),
    SX(i64),
    SF(f32),
    SD(f64),
    CB(Complex<i8>),
    CI(Complex<i16>),
    CL(Complex<i32>),
    CX(Complex<i64>),
    CF(Complex<f32>),
    CD(Complex<f64>),
}

/// Converts raw bytes to a bluefile data type.
pub fn bytes_to_data_value(data_type: &DataType, endianness: Endianness, buf: &Vec<u8>) -> Result<DataValue> {
    match data_type {
        DataType{rank: Rank::Scalar, format: Format::Byte} => Ok(DataValue::SB(byte_to_i8(buf[0])?)),
        DataType{rank: Rank::Scalar, format: Format::Int} => Ok(DataValue::SI(bytes_to_i16(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Long} => Ok(DataValue::SL(bytes_to_i32(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::LongLong} => Ok(DataValue::SX(bytes_to_i64(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Float} => Ok(DataValue::SF(bytes_to_f32(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Double} => Ok(DataValue::SD(bytes_to_f64(buf, endianness)?)),
        DataType{rank: Rank::Complex, format: Format::Byte} => Ok(DataValue::CB(bytes_to_complex_i8(buf)?)),
        DataType{rank: Rank::Complex, format: Format::Int} => Ok(DataValue::CI(bytes_to_complex_i16(buf, endianness)?)),
        DataType{rank: Rank::Complex, format: Format::Long} => Ok(DataValue::CL(bytes_to_complex_i32(buf, endianness)?)),
        DataType{rank: Rank::Complex, format: Format::LongLong} => Ok(DataValue::CX(bytes_to_complex_i64(buf, endianness)?)),
        DataType{rank: Rank::Complex, format: Format::Float} => Ok(DataValue::CF(bytes_to_complex_f32(buf, endianness)?)),
        DataType{rank: Rank::Complex, format: Format::Double} => Ok(DataValue::CD(bytes_to_complex_f64(buf, endianness)?)),
        _ => Err(Error::UnknownDataTypeError),
    }
}
