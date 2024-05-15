use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;
use crate::util::{
    byte_to_i8,
    bytes_to_i16,
    bytes_to_i32,
    bytes_to_f32,
    bytes_to_f64,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Rank {
    Scalar,
    // Complex,
}

impl TryFrom<u8> for Rank {
    type Error = Error;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            b'S' => Ok(Rank::Scalar),
            _ => Err(Error::UnknownRankError),
        }
    }
}

pub fn rank_multiple(r: &Rank) -> usize {
    match r {
        Rank::Scalar => 1,
        // Complex => 2,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Byte,
    Int,
    Long,
    Float,
    Double,
}

impl TryFrom<u8> for Format {
    type Error = Error;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            b'B' => Ok(Format::Byte),
            b'I' => Ok(Format::Int),
            b'L' => Ok(Format::Long),
            b'F' => Ok(Format::Float),
            b'D' => Ok(Format::Double),
            _ => Err(Error::UnknownFormatError),
        }
    }
}

pub fn format_size(f: &Format) -> usize {
    match f {
        Format::Byte => 1,
        Format::Int => 2,
        Format::Long => 4,
        Format::Float => 4,
        Format::Double => 8,
    }
}

#[derive(Debug, Clone)]
pub struct DataType {
    pub rank: Rank,
    pub format: Format,
}

impl DataType {
    pub fn size(&self) -> usize {
        rank_multiple(&self.rank) * format_size(&self.format)
    }
}

#[derive(Debug)]
pub enum DataValue {
    SB(i8),
    SI(i16),
    SL(i32),
    SF(f32),
    SD(f64),
}

pub fn bytes_to_data_value(data_type: &DataType, endianness: Endianness, buf: &Vec<u8>) -> Result<DataValue> {
    match data_type {
        DataType{rank: Rank::Scalar, format: Format::Byte} => Ok(DataValue::SB(byte_to_i8(buf[0])?)),
        DataType{rank: Rank::Scalar, format: Format::Int} => Ok(DataValue::SI(bytes_to_i16(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Long} => Ok(DataValue::SL(bytes_to_i32(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Float} => Ok(DataValue::SF(bytes_to_f32(buf, endianness)?)),
        DataType{rank: Rank::Scalar, format: Format::Double} => Ok(DataValue::SD(bytes_to_f64(buf, endianness)?)),
        _ => Err(Error::UnknownDataTypeError),
    }
}
