use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::str::from_utf8;

use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;
use crate::util::{bytes_to_i16, bytes_to_i32};

pub(crate) const ADJUNCT_HEADER_OFFSET: usize = 256;
pub(crate) const ADJUNCT_HEADER_SIZE: usize = 256;
const EXT_KEYWORD_LENGTH: usize = 4;


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
