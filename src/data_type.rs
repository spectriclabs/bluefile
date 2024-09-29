use std::fmt;

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

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rank::Scalar => write!(f, "scalar"),
            Rank::Complex => write!(f, "complex"),
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

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Byte => write!(f, "byte"),
            Format::Int => write!(f, "int"),
            Format::Long => write!(f, "long"),
            Format::LongLong => write!(f, "long long"),
            Format::Float => write!(f, "float"),
            Format::Double => write!(f, "double"),
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

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataValue::SB(x) => write!(f, "SB({})", x),
            DataValue::SI(x) => write!(f, "SI({})", x),
            DataValue::SL(x) => write!(f, "SL({})", x),
            DataValue::SX(x) => write!(f, "SX({})", x),
            DataValue::SF(x) => write!(f, "SF({})", x),
            DataValue::SD(x) => write!(f, "SD({})", x),
            DataValue::CB(x) => write!(f, "CB({})", x),
            DataValue::CI(x) => write!(f, "CI({})", x),
            DataValue::CL(x) => write!(f, "CL({})", x),
            DataValue::CX(x) => write!(f, "CX({})", x),
            DataValue::CF(x) => write!(f, "CF({})", x),
            DataValue::CD(x) => write!(f, "CD({})", x),
        }
    }
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
    }
}
