use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

const COMMON_HEADER_SIZE: usize = 256;
const ADJUNCT_HEADER_SIZE: usize = 256;
const HEADER_KEYWORD_LENGTH: usize = 92;

#[derive(Debug)]
pub enum Error {
    NotBlueFileError,
    InvalidEndianness,
    ByteConversionError,
    FileOpenError(String),
    FileReadError(String),
    NotEnoughHeaderBytes(usize),
    NotEnoughAdjunctHeaderBytes(usize),
    UnknownFileTypeCode(i32),
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
    Type1000(i32),
    Type2000(i32),
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

pub struct Type1000Adjunct {
    pub xstart: f64,
    pub xdelta: f64,
    pub xunits: i32,
}

pub struct Type2000Adjunct {
    pub xstart: f64,
    pub xdelta: f64,
    pub xunits: i32,
    pub subsize: i32,
    pub ystart: f64,
    pub ydelta: f64,
    pub yunits: i32,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub header_endianness: Endianness,
    pub data_endianness: Endianness,
    pub ext_start: i32,  // in 512 byte blocks
    pub ext_size: i32,  // in bytes
    pub data_start: f64,  // in bytes
    pub data_size: f64,  // in bytes
    pub type_code: TypeCode,
    pub format: Format,
    pub timecode: f64,  // seconds since Jan. 1, 1950
    pub keywords: Vec<HeaderKeyword>,
}

pub struct Type1000 {
    pub header: Header,
    pub adjunct: Type1000Adjunct,
}

pub struct Type2000 {
    file: File,
    pub header: Header,
    pub adjunct: Type2000Adjunct,
}

fn parse_type1000_adjunct(v: &[u8], endianness: Endianness) -> Result<Type1000Adjunct> {
    let xstart: f64 = bytes_to_f64(&v[0..8], endianness)?;
    let xdelta: f64 = bytes_to_f64(&v[8..16], endianness)?;
    let xunits: i32 = bytes_to_i32(&v[16..20], endianness)?;
    Ok(Type1000Adjunct{
        xstart,
        xdelta,
        xunits,
    })
}

fn parse_type2000_adjunct(v: &[u8], endianness: Endianness) -> Result<Type2000Adjunct> {
    let xstart: f64 = bytes_to_f64(&v[0..8], endianness)?;
    let xdelta: f64 = bytes_to_f64(&v[8..16], endianness)?;
    let xunits: i32 = bytes_to_i32(&v[16..20], endianness)?;
    let subsize: i32 = bytes_to_i32(&v[20..24], endianness)?;
    let ystart: f64 = bytes_to_f64(&v[24..32], endianness)?;
    let ydelta: f64 = bytes_to_f64(&v[32..40], endianness)?;
    let yunits: i32 = bytes_to_i32(&v[40..44], endianness)?;
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

fn bytes_to_i32(v: &[u8], endianness: Endianness) -> Result<i32> {
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
    let t = bytes_to_i32(v, endianness)?;

    if t/1000 == 1 {
        Ok(TypeCode::Type1000(t))
    } else if t/1000 == 2 {
        Ok(TypeCode::Type2000(t))
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

    let header_endianness = parse_endianness(&data[4..8])?;
    let data_endianness = parse_endianness(&data[8..12])?;
    let ext_start = bytes_to_i32(&data[24..28], header_endianness)?;
    let ext_size = bytes_to_i32(&data[28..32], header_endianness)?;
    let data_start = bytes_to_f64(&data[32..40], header_endianness)?;
    let data_size = bytes_to_f64(&data[40..48], header_endianness)?;
    let type_code = parse_type_code(&data[48..52], header_endianness)?;
    let format = Format{mode: data[52], ftype: data[53]};
    let timecode = bytes_to_f64(&data[56..64], header_endianness)?;
    let keylength: usize = match bytes_to_i32(&data[160..164], header_endianness).unwrap().try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::HeaderKeywordLengthParseError),
    };
    let mut keywords = Vec::new();
    parse_header_keywords(&mut keywords, &data[164..164+HEADER_KEYWORD_LENGTH], keylength)?;

    let header = Header{
        header_endianness,
        data_endianness,
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

fn open_file(path: &Path) -> Result<File> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path.display().to_string())),
    };
    Ok(file)
}

pub fn init_type2000(path: &Path) -> Result<Type2000> {
    let mut file = open_file(path)?;
    let mut header_data = vec![0_u8; COMMON_HEADER_SIZE];
    let n = match file.read(&mut header_data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError(path.display().to_string())),
    };

    if n < COMMON_HEADER_SIZE {
        return Err(Error::NotEnoughHeaderBytes(n))
    }

    let header = parse_header(&header_data)?;

    let mut adjunct_data = vec![0_u8; ADJUNCT_HEADER_SIZE];
    let n = match file.read(&mut adjunct_data) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileReadError(path.display().to_string())),
    };

    if n < ADJUNCT_HEADER_SIZE {
        return Err(Error::NotEnoughAdjunctHeaderBytes(n))
    }

    let adjunct = parse_type2000_adjunct(&adjunct_data, header.header_endianness)?;

    Ok(Type2000 {
        file,
        header,
        adjunct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_header_test() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/penny.prm");
        let type2000 = init_type2000(d.as_path()).unwrap();
        let header = type2000.header;
        let adjunct = type2000.adjunct;
        assert_eq!(header.header_endianness, Endianness::Little);
        assert_eq!(header.data_endianness, Endianness::Little);
        assert_eq!(header.ext_start, 257);
        assert_eq!(header.ext_size, 320);
        assert_eq!(header.data_start, 512.0);
        assert_eq!(header.data_size, 131072.0);
        assert_eq!(header.type_code, TypeCode::Type2000(2000));
        assert_eq!(header.format.mode, b'S');
        assert_eq!(header.format.ftype, b'D');
        assert_eq!(header.timecode, 0.0);
        assert_eq!(header.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
        assert_eq!(header.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});
        assert_eq!(adjunct.xstart, 0.0);
        assert_eq!(adjunct.xdelta, 1.0);
        assert_eq!(adjunct.xunits, 0);
        assert_eq!(adjunct.subsize, 128);
        assert_eq!(adjunct.ystart, 0.0);
        assert_eq!(adjunct.ydelta, 1.0);
        assert_eq!(adjunct.yunits, 0);
    }
}
