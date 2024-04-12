use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

const HEADER_SIZE: usize = 512;
const HEADER_KEYWORD_LENGTH: usize = 92;

#[derive(Debug)]
pub enum Error {
    NotBlueFileError,
    InvalidEndianness,
    ByteConversionError,
    FileOpenError(String),
    FileReadError(String),
    NotEnoughHeaderBytes(usize),
    UnknownFileTypeCode(u32),
    InvalidHeaderKeywordLength(usize),
    HeaderKeywordParseError,
    HeaderKeywordLengthParseError,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TypeCode {
    T1000,
    T2000,
}

#[derive(Debug, Clone)]
pub struct Format {
    pub mode: u8,
    pub ftype: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderKeyword {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub header_rep: Endianness,
    pub data_rep: Endianness,
    pub ext_start: u32,  // in 512 byte blocks
    pub ext_size: u32,  // in bytes
    pub data_start: f64,  // in bytes
    pub data_size: f64,  // in bytes
    pub type_code: TypeCode,
    pub format: Format,
    pub timecode: f64,  // seconds since Jan. 1, 1950
    pub keywords: Vec<HeaderKeyword>,
}

fn is_blue(v: &[u8]) -> bool {
    v[0] == b'B' && v[1] == b'L' && v[2] == b'U' && v[3] == b'E'
}

fn parse_endianness(v: &[u8]) -> Result<Endianness> {
    if v[0] == b'E' && v[1] == b'E' && v[2] == b'E' && v[3] == b'I' {
        Ok(Endianness::Little)
    } else if v[0] == b'I' && v[1] == b'E' && v[2] == b'E' && v[3] == b'E' {
        Ok(Endianness::Big)
    } else {
        Err(Error::InvalidEndianness)
    }
}

fn bytes_to_u32(v: &[u8], endianness: Endianness) -> Result<u32> {
    let b: [u8; 4] = match v.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::ByteConversionError),
    };

    if endianness == Endianness::Little {
        Ok(u32::from_le_bytes(b))
    } else {
        Ok(u32::from_be_bytes(b))
    }
}

fn bytes_to_f64(v: &[u8], endianness: Endianness) -> Result<f64> {
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

fn parse_type_code(v: &[u8], endianness: Endianness) -> Result<TypeCode> {
    let t = bytes_to_u32(v, endianness)?;

    if t == 1000 {
        Ok(TypeCode::T1000)
    } else if t == 2000 {
        Ok(TypeCode::T2000)
    } else {
        Err(Error::UnknownFileTypeCode(t))
    }
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
            term = b'\0'
        } else if *b == term && term == b'\0' && name.len() > 0 {
            keywords.push(HeaderKeyword{
                name: from_utf8(&name).unwrap().to_string(),
                value: from_utf8(&value).unwrap().to_string(),
            });
            count += 1;
            term = b'=';
            name = Vec::new();
            value = Vec::new();
        } else if *b != term && term == b'=' {
            name.push(*b);
        } else if *b != term && term == b'\0' {
            value.push(*b);
        } else {
            return Err(Error::HeaderKeywordParseError);
        }
    }

    Ok(count)
}

pub fn parse_header(data: &[u8]) -> Result<Header> {
    if !is_blue(&data[0..4]) {
        return Err(Error::NotBlueFileError);
    }

    let header_rep = parse_endianness(&data[4..8])?;
    let data_rep = parse_endianness(&data[8..12])?;
    let ext_start = bytes_to_u32(&data[24..28], header_rep)?;
    let ext_size = bytes_to_u32(&data[28..32], header_rep)?;
    let data_start = bytes_to_f64(&data[32..40], header_rep)?;
    let data_size = bytes_to_f64(&data[40..48], header_rep)?;
    let type_code = parse_type_code(&data[48..52], header_rep)?;
    let format = Format{mode: data[52], ftype: data[53]};
    let timecode = bytes_to_f64(&data[56..64], header_rep)?;
    let keylength: usize = match bytes_to_u32(&data[160..164], header_rep).unwrap().try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::HeaderKeywordLengthParseError),
    };
    let mut keywords = Vec::new();
    parse_header_keywords(&mut keywords, &data[164..164+HEADER_KEYWORD_LENGTH], keylength)?;

    let header = Header{
        header_rep,
        data_rep,
        ext_start,
        ext_size,
        data_start,
        data_size,
        type_code,
        format,
        timecode,
        keywords,
    };

    Ok(header)
}

pub fn read_header(path: &Path) -> Result<Header> {
    let mut file = match File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path.display().to_string())),
    };

    let mut data = vec![0_u8; HEADER_SIZE];
    let n = match file.read(&mut data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError(path.display().to_string())),
    };

    if n < HEADER_SIZE {
        return Err(Error::NotEnoughHeaderBytes(n))
    }

    let header = parse_header(&data)?;
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_header_test() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/penny.prm");
        let header = read_header(d.as_path());
        let href = header.as_ref().unwrap();
        assert_eq!(href.header_rep, Endianness::Little);
        assert_eq!(href.data_rep, Endianness::Little);
        assert_eq!(href.ext_start, 257);
        assert_eq!(href.ext_size, 320);
        assert_eq!(href.data_start, 512.0);
        assert_eq!(href.data_size, 131072.0);
        assert_eq!(href.type_code, TypeCode::T2000);
        assert_eq!(href.format.mode, b'S');
        assert_eq!(href.format.ftype, b'D');
        assert_eq!(href.timecode, 0.0);
        assert_eq!(href.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
        assert_eq!(href.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});
    }
}
