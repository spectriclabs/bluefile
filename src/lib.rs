//! Experimental Rust library for handling X-Midas Bluefiles.
//!
//! ```no_run
//! use std::fs::File;
//! use bluefile::read_header;
//!
//! let file = File::open("/path/to/bluefile").unwrap();
//! let header = read_header(&file).unwrap();
//! println!("{}", header.type_code);
//! println!("{}", header.data_type);
//! ```

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::str::from_utf8;

use num::complex::Complex;

const ADJUNCT_HEADER_OFFSET: usize = 256;
const ADJUNCT_HEADER_SIZE: usize = 256;
const EXT_KEYWORD_LENGTH: usize = 4;

const COMMON_HEADER_OFFSET: usize = 0;  // in bytes
const COMMON_HEADER_SIZE: usize = 256;  // in bytes
const HEADER_KEYWORD_OFFSET: usize = 164;  // in bytes
const HEADER_KEYWORD_LENGTH: usize = 92;  // in bytes

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotBlueFileError,
    TypeCodeMismatchError,
    UnknownRankError,
    UnknownFormatError,
    UnknownDataTypeError,
    InvalidEndianness,
    ByteConversionError,
    FileOpenError(String),
    FileReadError,
    NotEnoughHeaderBytes(usize),
    NotEnoughAdjunctHeaderBytes(usize),
    UnknownFileTypeCode(i32),
    InvalidHeaderKeywordLength(usize),
    HeaderSeekError,
    AdjunctHeaderSeekError,
    ExtHeaderSeekError,
    HeaderKeywordParseError,
    HeaderKeywordLengthParseError,
    ExtHeaderKeywordLengthParseError,
    ExtHeaderKeywordReadError,
    DataSeekError,
    BluejayConfigError,
}


/// Defines endianness type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endianness {
    Big,
    Little,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endianness::Big => write!(f, "big"),
            Endianness::Little => write!(f, "little"),
        }
    }
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

/// Represents the bluefile type, such as 1000, 2000, etc.
pub type TypeCode = i32;

/// Two bytes that represent rank and format
#[derive(Clone, Debug, PartialEq)]
pub struct DataType {
    pub rank: u8,
    pub format: u8,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match from_utf8(&[self.rank, self.format]) {
            Ok(v) => write!(f, "{}", v),
            Err(_) => write!(f, "??"),
        }
    }
}

impl DataType {
    pub fn num_bytes(&self) -> Result<usize> {
        let mult = match self.rank {
            b'S' => 1,
            b'C' => 2,
            _ => return Err(Error::UnknownRankError),
        };

        let size = match self.format {
            b'B' => 1,
            b'I' => 2,
            b'L' => 4,
            b'X' => 8,
            b'F' => 4,
            b'D' => 8,
            _ => return Err(Error::UnknownFormatError),
        };

        Ok(mult * size)
    }
}

/// Reads the extended header keywords.
pub fn read_ext_header(mut file: &File, header: &Header) -> Result<Vec<ExtKeyword>> {
    match file.seek(SeekFrom::Start(header.ext_start as u64)) {
        Ok(x) => x,
        Err(_) => return Err(Error::ExtHeaderSeekError),
    };

    let mut keywords: Vec<ExtKeyword> = vec![];
    let mut consumed: usize = 0;

    while consumed < header.ext_size {
        let mut key_length_buf = vec![0_u8; EXT_KEYWORD_LENGTH];
        consumed += match file.read_exact(&mut key_length_buf) {
            Ok(_) => EXT_KEYWORD_LENGTH,
            Err(_) => break,
        };

        // entire length of keyword block: tag, data, kwhdr & padding
        let key_length = bytes_to_i32(&key_length_buf, header.header_endianness).unwrap() as usize;
        let mut key_buf = vec![0_u8; key_length-EXT_KEYWORD_LENGTH];
        consumed += match file.read_exact(&mut key_buf) {
            Ok(_) => key_length-EXT_KEYWORD_LENGTH,
            Err(_) => break,
        };
        let keyword = parse_ext_keyword(&key_buf, key_length, header.header_endianness).unwrap();
        keywords.push(keyword);
    }

    Ok(keywords)
}

/// Represents an extended header keyword value with the necessary information to render it from
/// raw bytes.
pub struct ExtKeywordValue {
    pub format: char,
    pub endianness: Endianness,
    pub raw_value: Vec<u8>,
}

impl fmt::Display for ExtKeywordValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            'A' | 'S' | 'Z' => write!(f, "\"{}\"", from_utf8(&self.raw_value).unwrap().replace('\"', "\\\"")),
            'B' => write!(f, "{}", byte_to_i8(self.raw_value[0]).unwrap()),
            'O' => write!(f, "{}", self.raw_value[0]),
            'I' => write!(f, "{}", bytes_to_i16(&self.raw_value[0..2], self.endianness).unwrap()),
            'L' => write!(f, "{}", bytes_to_i32(&self.raw_value[0..4], self.endianness).unwrap()),
            'X' => write!(f, "{}", bytes_to_i64(&self.raw_value[0..8], self.endianness).unwrap()),
            'F' => write!(f, "{}", bytes_to_f32(&self.raw_value[0..4], self.endianness).unwrap()),
            'D' => write!(f, "{}", bytes_to_f64(&self.raw_value[0..8], self.endianness).unwrap()),
            _ => write!(f, "\"?\""),
        }
    }
}

/// Extended header keyword.
pub struct ExtKeyword {
    pub length: usize,
    pub tag: String,
    pub value: ExtKeywordValue,
}

fn parse_ext_keyword(v: &[u8], key_length: usize, endianness: Endianness) -> Result<ExtKeyword> {
    // Note that 4 is subtracted from the offsets because key_length was already read
    let extra_length = bytes_to_i16(&v[0..2], endianness)? as usize;  // length of the keyword header, tag & padding
    let tag_length = v[2] as usize;  // length of just the tag
    let format = v[3] as char;

    let value_offset: usize = 4;
    let value_length: usize = key_length - extra_length;
    let tag_offset: usize = value_offset + value_length;

    let tag = from_utf8(&v[tag_offset..tag_offset+tag_length]).unwrap().to_string();
    let raw_value = v[value_offset..value_offset+value_length].to_vec();
    let value = ExtKeywordValue{
        format,
        endianness,
        raw_value,
    };

    Ok(ExtKeyword{
        length: key_length,
        tag,
        value,
    })
}

/// Represents a from the main header (not extended header).
#[derive(Debug, Clone, PartialEq)]
pub struct HeaderKeyword {
    pub name: String,
    pub value: String,
}

/// Represents the main header.
#[derive(Debug, Clone)]
pub struct Header {
    /// Endianness of the values in the header.
    pub header_endianness: Endianness,

    /// Endianness of the values in the data.
    pub data_endianness: Endianness,

    /// Extended header start location in bytes.
    pub ext_start: usize,

    /// Size of the extended header in bytes.
    pub ext_size: usize,

    /// Data start location in bytes.  The format specifies this as a 64 bit float because there was
    /// no 64 bit integer type at the time.
    pub data_start: f64,

    /// Data size in bytes.  The format specifies this as a 64 bit float because there was
    /// no 64 bit integer type at the time.
    pub data_size: f64,

    /// The type code of the file (1000, 2000, 3000, etc.).
    pub type_code: TypeCode,

    /// The rank and format of the data (SD, CF, NH, etc.).
    pub data_type: DataType,

    /// The start time of the data in seconds since January 1, 1950.
    pub timecode: f64,

    /// Keywords from the main header (not extended header keywords).
    pub keywords: Vec<HeaderKeyword>,
}

/// Represents the adjunct header fields for type 1000 files.
#[derive(Clone, Debug)]
pub struct Type1000Adjunct {
    pub xstart: f64,
    pub xdelta: f64,
    pub xunits: i32,
}

/// Represents the adjunct header fields for type 2000 files.
#[derive(Clone, Debug)]
pub struct Type2000Adjunct {
    pub xstart: f64,
    pub xdelta: f64,
    pub xunits: i32,
    pub subsize: i32,
    pub ystart: f64,
    pub ydelta: f64,
    pub yunits: i32,
}

fn is_blue(v: &[u8]) -> bool {
    v[0] == b'B' && v[1] == b'L' && v[2] == b'U' && v[3] == b'E'
}

/// Parses the main header from raw bytes.
pub fn parse_header(data: &[u8]) -> Result<Header> {
    if !is_blue(&data[0..4]) {
        return Err(Error::NotBlueFileError);
    }

    let header_endianness = Endianness::try_from(&data[4..8])?;
    let data_endianness = Endianness::try_from(&data[8..12])?;
    let ext_start = (bytes_to_i32(&data[24..28], header_endianness)? as usize) * 512;
    let ext_size = bytes_to_i32(&data[28..32], header_endianness)? as usize;
    let data_start = bytes_to_f64(&data[32..40], header_endianness)?;
    let data_size = bytes_to_f64(&data[40..48], header_endianness)?;
    let type_code = parse_type_code(&data[48..52], header_endianness)?;
    let data_type = DataType{rank: data[52], format: data[53]};
    let timecode = bytes_to_f64(&data[56..64], header_endianness)?;
    let keylength: usize = match bytes_to_i32(&data[160..164], header_endianness).unwrap().try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::HeaderKeywordLengthParseError),
    };
    let mut keywords = Vec::new();
    parse_header_keywords(&mut keywords, &data[HEADER_KEYWORD_OFFSET..HEADER_KEYWORD_OFFSET+HEADER_KEYWORD_LENGTH], keylength)?;

    let header = Header{
        header_endianness,
        data_endianness,
        ext_start,
        ext_size,
        data_start,
        data_size,
        type_code,
        data_type,
        timecode,
        keywords,
    };

    Ok(header)
}

/// Reads the main header from a file.
pub fn read_header(mut file: &File) -> Result<Header> {
    match file.seek(SeekFrom::Start(COMMON_HEADER_OFFSET as u64)) {
        Ok(x) => x,
        Err(_) => return Err(Error::HeaderSeekError),
    };

    let mut header_data = vec![0_u8; COMMON_HEADER_SIZE];
    let n = match file.read(&mut header_data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError),
    };

    if n < COMMON_HEADER_SIZE {
        return Err(Error::NotEnoughHeaderBytes(n))
    }

    let header = parse_header(&header_data)?;
    Ok(header)
}

fn parse_header_keywords(keywords: &mut Vec<HeaderKeyword>, v: &[u8], keylength: usize) -> Result<usize> {
    if keylength > HEADER_KEYWORD_LENGTH {
        return Err(Error::InvalidHeaderKeywordLength(keylength));
    }

    let mut count: usize = 0;
    let mut name = Vec::new();
    let mut value = Vec::new();
    let mut term = b'=';

    for b in &v[0..keylength] {
        if *b == term && term == b'=' {
            // found equal, now look for null terminator
            term = b'\0'
        } else if *b == term && term == b'\0' && !name.is_empty() {
            // found null terminator, add new keyword
            keywords.push(HeaderKeyword{
                name: from_utf8(&name).unwrap().to_string(),
                value: from_utf8(&value).unwrap().to_string(),
            });
            count += 1;
            term = b'=';
            name = Vec::new();
            value = Vec::new();
        } else if term == b'=' && *b == b'\0' {
            // encountered null terminator when looking for equal
            return Err(Error::HeaderKeywordParseError);
        } else if *b != term && term == b'=' {
            // add character to name until we find equal
            name.push(*b);
        } else if *b != term && term == b'\0' {
            // add character to value until we find null terminator
            value.push(*b);
        } else {
            // unexpected state
            return Err(Error::HeaderKeywordParseError);
        }
    }

    Ok(count)
}

fn parse_type_code(v: &[u8], endianness: Endianness) -> Result<TypeCode> {
    let t = bytes_to_i32(v, endianness)?;

    match t / 1000 {
        #![allow(clippy::manual_range_patterns)]
        1 | 2 | 3 | 4 | 5 | 6 => Ok(t as TypeCode),
        _ => Err(Error::UnknownFileTypeCode(t)),
    }
}

/// Reads the adjunct header from a type 1000 file.
pub fn read_type1000_adjunct_header(mut file: &File, header: &Header) -> Result<Type1000Adjunct> {
    match file.seek(SeekFrom::Start(ADJUNCT_HEADER_OFFSET as u64)) {
        Ok(x) => x,
        Err(_) => return Err(Error::AdjunctHeaderSeekError),
    };

    let mut data = vec![0_u8; ADJUNCT_HEADER_SIZE];
    let n = match file.read(&mut data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError),
    };

    if n < ADJUNCT_HEADER_SIZE {
        return Err(Error::NotEnoughAdjunctHeaderBytes(n))
    }

    let endianness = header.header_endianness;
    let xstart: f64 = bytes_to_f64(&data[0..8], endianness)?;
    let xdelta: f64 = bytes_to_f64(&data[8..16], endianness)?;
    let xunits: i32 = bytes_to_i32(&data[16..20], endianness)?;

    Ok(Type1000Adjunct{
        xstart,
        xdelta,
        xunits,
    })
}

/// Reads the adjunct header from a type 2000 file.
pub fn read_type2000_adjunct_header(mut file: &File, header: &Header) -> Result<Type2000Adjunct> {
    match file.seek(SeekFrom::Start(ADJUNCT_HEADER_OFFSET as u64)) {
        Ok(x) => x,
        Err(_) => return Err(Error::AdjunctHeaderSeekError),
    };

    let mut data = vec![0_u8; ADJUNCT_HEADER_SIZE];
    let n = match file.read(&mut data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError),
    };

    if n < ADJUNCT_HEADER_SIZE {
        return Err(Error::NotEnoughAdjunctHeaderBytes(n))
    }

    let endianness = header.header_endianness;
    let xstart: f64 = bytes_to_f64(&data[0..8], endianness)?;
    let xdelta: f64 = bytes_to_f64(&data[8..16], endianness)?;
    let xunits: i32 = bytes_to_i32(&data[16..20], endianness)?;
    let subsize: i32 = bytes_to_i32(&data[20..24], endianness)?;
    let ystart: f64 = bytes_to_f64(&data[24..32], endianness)?;
    let ydelta: f64 = bytes_to_f64(&data[32..40], endianness)?;
    let yunits: i32 = bytes_to_i32(&data[40..44], endianness)?;

    Ok(Type2000Adjunct{
        xstart,
        xdelta,
        xunits,
        subsize,
        ystart,
        ydelta,
        yunits,
    })
}

/// Converts a byte to an i8.
pub fn byte_to_i8(v: u8) -> Result<i8> {
    match i8::try_from(v) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::ByteConversionError),
    }
}

/// Converts bytes to an i16.
pub fn bytes_to_i16(v: &[u8], endianness: Endianness) -> Result<i16> {
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

/// Concerts bytes to an i32.
pub fn bytes_to_i32(v: &[u8], endianness: Endianness) -> Result<i32> {
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

/// Converts bytes to an i64.
pub fn bytes_to_i64(v: &[u8], endianness: Endianness) -> Result<i64> {
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

/// Converts bytes to an f32.
pub fn bytes_to_f32(v: &[u8], endianness: Endianness) -> Result<f32> {
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

/// Converts bytes to an f64.
pub fn bytes_to_f64(v: &[u8], endianness: Endianness) -> Result<f64> {
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

/// Converts bytes to a complex i8 (CB).
pub fn bytes_to_complex_i8(v: &[u8]) -> Result<Complex<i8>> {
    let real: i8 = byte_to_i8(v[0])?;
    let imag: i8 = byte_to_i8(v[1])?;
    Ok(Complex::<i8>::new(real, imag))
}

/// Converts bytes to a complex i16 (CI).
pub fn bytes_to_complex_i16(v: &[u8], endianness: Endianness) -> Result<Complex<i16>> {
    let real: i16 = bytes_to_i16(&v[0..2], endianness)?;
    let imag: i16 = bytes_to_i16(&v[2..4], endianness)?;
    Ok(Complex::<i16>::new(real, imag))
}

/// Converts bytes to a complex i32 (CL).
pub fn bytes_to_complex_i32(v: &[u8], endianness: Endianness) -> Result<Complex<i32>> {
    let real: i32 = bytes_to_i32(&v[0..4], endianness)?;
    let imag: i32 = bytes_to_i32(&v[4..8], endianness)?;
    Ok(Complex::<i32>::new(real, imag))
}

/// Converts bytes to a complex i64 (CX).
pub fn bytes_to_complex_i64(v: &[u8], endianness: Endianness) -> Result<Complex<i64>> {
    let real: i64 = bytes_to_i64(&v[0..8], endianness)?;
    let imag: i64 = bytes_to_i64(&v[8..16], endianness)?;
    Ok(Complex::<i64>::new(real, imag))
}

/// Converts bytes to a complex f32 (CF).
pub fn bytes_to_complex_f32(v: &[u8], endianness: Endianness) -> Result<Complex<f32>> {
    let real: f32 = bytes_to_f32(&v[0..4], endianness)?;
    let imag: f32 = bytes_to_f32(&v[4..8], endianness)?;
    Ok(Complex::<f32>::new(real, imag))
}

/// Converts bytes to a complex f64 (CD).
pub fn bytes_to_complex_f64(v: &[u8], endianness: Endianness) -> Result<Complex<f64>> {
    let real: f64 = bytes_to_f64(&v[0..8], endianness)?;
    let imag: f64 = bytes_to_f64(&v[8..16], endianness)?;
    Ok(Complex::<f64>::new(real, imag))
}
