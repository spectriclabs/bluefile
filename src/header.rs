use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::str::from_utf8;

use crate::bluefile::TypeCode;
use crate::data_type::{DataType, Format, Rank};
use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;
use crate::util::{
    bytes_to_f64,
    bytes_to_i32,
};

const COMMON_HEADER_OFFSET: usize = 0;
const COMMON_HEADER_SIZE: usize = 256;
const HEADER_KEYWORD_OFFSET: usize = 164;
const HEADER_KEYWORD_LENGTH: usize = 92;

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderKeyword {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub header_endianness: Endianness,
    pub data_endianness: Endianness,
    pub ext_start: usize,  // in bytes (multiple of 512 byte blocks)
    pub ext_size: usize,  // in bytes
    pub data_start: f64,  // in bytes
    pub data_size: f64,  // in bytes
    pub type_code: TypeCode,
    pub data_type: DataType,
    pub timecode: f64,  // seconds since Jan. 1, 1950
    pub keywords: Vec<HeaderKeyword>,
}

fn is_blue(v: &[u8]) -> bool {
    v[0] == b'B' && v[1] == b'L' && v[2] == b'U' && v[3] == b'E'
}

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
    let data_type = DataType{rank: Rank::try_from(data[52])?, format: Format::try_from(data[53])?};
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

    if t/1000 == 1 {
        Ok(TypeCode::Type1000(t))
    } else if t/1000 == 2 {
        Ok(TypeCode::Type2000(t))
    } else {
        Err(Error::UnknownFileTypeCode(t))
    }
}
