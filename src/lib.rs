use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::str::from_utf8;

const COMMON_HEADER_OFFSET: usize = 0;
const COMMON_HEADER_SIZE: usize = 256;
const ADJUNCT_HEADER_OFFSET: usize = 256;
const ADJUNCT_HEADER_SIZE: usize = 256;
const HEADER_KEYWORD_OFFSET: usize = 164;
const HEADER_KEYWORD_LENGTH: usize = 92;
const EXT_KEYWORD_LENGTH: usize = 4;

#[derive(Debug)]
pub enum Error {
    NotBlueFileError,
    TypeCodeMismatchError,
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
    ExtSizeParseError,
    ExtStartParseError,
}

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

#[derive(Debug, Clone)]
pub struct Header {
    pub header_endianness: Endianness,
    pub data_endianness: Endianness,
    pub ext_start: usize,  // in bytes (multiple of 512 byte blocks)
    pub ext_size: usize,  // in bytes
    pub data_start: f64,  // in bytes
    pub data_size: f64,  // in bytes
    pub type_code: TypeCode,
    pub format: Format,
    pub timecode: f64,  // seconds since Jan. 1, 1950
    pub keywords: Vec<HeaderKeyword>,
}

type Result<T> = std::result::Result<T, Error>;

pub trait BluefileReader {
    fn new(path: &Path) -> Result<Self> where Self: Sized;
    type AdjunctHeader;
    fn read_adjunct_header(&self) -> Result<Self::AdjunctHeader>;
    fn get_ext_size(&self) -> usize;
    fn get_ext_start(&self) -> usize;
    fn get_file(&self) -> &File;
    fn get_header_endianness(&self) -> Endianness;

    fn read_ext_header(&self) -> Result<Vec<ExtKeyword>> {
        let mut keywords = Vec::new();
        let mut consumed: usize = 0;
        let mut file = self.get_file();

        match file.seek(SeekFrom::Start(self.get_ext_start() as u64)) {
            Ok(x) => x,
            Err(_) => return Err(Error::ExtHeaderSeekError),
        };

        while consumed < self.get_ext_size() {
            let mut key_length_buf = vec![0_u8; EXT_KEYWORD_LENGTH];
            consumed += match file.read(&mut key_length_buf) {
                Ok(x) => x,
                Err(_) => return Err(Error::ExtHeaderKeywordLengthParseError),
            };

            // entire length of keyword block: tag, data, kwhdr & padding
            let key_length = bytes_to_i32(&key_length_buf, self.get_header_endianness())? as usize;
            let mut key_buf = vec![0_u8; key_length-EXT_KEYWORD_LENGTH];
            consumed += match file.read(&mut key_buf) {
                Ok(x) => x,
                Err(_) => return Err(Error::ExtHeaderKeywordReadError),
            };
            let keyword = parse_ext_keyword(&key_buf, key_length, self.get_header_endianness())?;
            keywords.push(keyword);
        }

        Ok(keywords)
    }
}

fn read_header(mut file: &File) -> Result<Header> {
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

pub struct Type1000Reader {
    file: File,
    header: Header,
}

pub struct Type2000Reader {
    file: File,
    header: Header,
}

impl BluefileReader for Type1000Reader {
    fn new(path: &Path) -> Result<Self> {
        let file = open_file(path)?;
        let header = read_header(&file)?;

        match header.type_code {
            TypeCode::Type1000(_) => Ok(Self {file, header}),
            _ => Err(Error::TypeCodeMismatchError),
        }
    }

    type AdjunctHeader = Type1000Adjunct;

    fn read_adjunct_header(&self) -> Result<Self::AdjunctHeader> {
        let mut file = self.get_file();

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

        let endianness = self.get_header_endianness();
        let xstart: f64 = bytes_to_f64(&data[0..8], endianness)?;
        let xdelta: f64 = bytes_to_f64(&data[8..16], endianness)?;
        let xunits: i32 = bytes_to_i32(&data[16..20], endianness)?;

        Ok(Type1000Adjunct{
            xstart,
            xdelta,
            xunits,
        })
    }

    fn get_ext_size(&self) -> usize {
        self.header.ext_size
    }

    fn get_ext_start(&self) -> usize {
        self.header.ext_start
    }

    fn get_file(&self) -> &File {
        &self.file
    }

    fn get_header_endianness(&self) -> Endianness {
        self.header.header_endianness
    }
}

impl BluefileReader for Type2000Reader {
    fn new(path: &Path) -> Result<Self> {
        let file = open_file(path)?;
        let header = read_header(&file)?;

        match header.type_code {
            TypeCode::Type2000(_) => Ok(Self {file, header}),
            _ => Err(Error::TypeCodeMismatchError),
        }
    }

    type AdjunctHeader = Type2000Adjunct;

    fn read_adjunct_header(&self) -> Result<Self::AdjunctHeader> {
        let mut file = self.get_file();

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

        let endianness = self.get_header_endianness();
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

    fn get_ext_size(&self) -> usize {
        self.header.ext_size
    }

    fn get_ext_start(&self) -> usize {
        self.header.ext_start
    }

    fn get_file(&self) -> &File {
        &self.file
    }

    fn get_header_endianness(&self) -> Endianness {
        self.header.header_endianness
    }
}

fn open_file(path: &Path) -> Result<File> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path.display().to_string())),
    };
    Ok(file)
}

//fn new_reader(path: &Path) -> Result<Box<dyn BluefileReader>> {
//    let file = open_file(path)?;
//    let header = read_header(&file);
//}

pub struct ExtKeyword {
    pub length: usize,
    pub tag: String,
    pub format: char,
    pub value: Vec<u8>,
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
    let value = v[value_offset..value_offset+value_length].to_vec();

    Ok(ExtKeyword{
        length: key_length,
        tag,
        format,
        value,
    })
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

fn is_blue(v: &[u8]) -> bool {
    v[0] == b'B' && v[1] == b'L' && v[2] == b'U' && v[3] == b'E'
}

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

fn bytes_to_i16(v: &[u8], endianness: Endianness) -> Result<i16> {
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
    let format = Format{mode: data[52], ftype: data[53]};
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
        format,
        timecode,
        keywords,
    };

    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_type2000_test() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/penny.prm");
        let reader = Type2000Reader::new(d.as_path()).unwrap();
        let header = &reader.header;

        assert_eq!(header.header_endianness, Endianness::Little);
        assert_eq!(header.data_endianness, Endianness::Little);
        assert_eq!(header.ext_start, 257*512);
        assert_eq!(header.ext_size, 320);
        assert_eq!(header.data_start, 512.0);
        assert_eq!(header.data_size, 131072.0);
        assert_eq!(header.type_code, TypeCode::Type2000(2000));
        assert_eq!(header.format.mode, b'S');
        assert_eq!(header.format.ftype, b'D');
        assert_eq!(header.timecode, 0.0);
        assert_eq!(header.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
        assert_eq!(header.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});

        let adjunct = &reader.read_adjunct_header().unwrap();
        assert_eq!(adjunct.xstart, 0.0);
        assert_eq!(adjunct.xdelta, 1.0);
        assert_eq!(adjunct.xunits, 0);
        assert_eq!(adjunct.subsize, 128);
        assert_eq!(adjunct.ystart, 0.0);
        assert_eq!(adjunct.ydelta, 1.0);
        assert_eq!(adjunct.yunits, 0);

        let ext_keywords = &reader.read_ext_header().unwrap();
        assert_eq!(ext_keywords.len(), 5);

        assert_eq!(ext_keywords[0].tag, "COMMENT".to_string());
        assert_eq!(ext_keywords[0].format, 'A');
        assert_eq!(from_utf8(&ext_keywords[0].value).unwrap(), "Demo data for XRTSURFACE/STAY".to_string());

        assert_eq!(ext_keywords[4].tag, "COMMENT3".to_string());
        assert_eq!(ext_keywords[4].format, 'A');
        assert_eq!(from_utf8(&ext_keywords[4].value).unwrap(), "XRTSURF/STAY/NOLAB/XC=5,PENNY,1.0,255.0,4,128,16,0,10,2".to_string());
    }
}
