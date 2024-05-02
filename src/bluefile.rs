use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use crate::endian::Endianness;
use crate::error::Error;
use crate::result::Result;
use crate::util::{bytes_to_i16, bytes_to_i32, open_file};

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

pub struct ExtHeaderReader {
    reader: BufReader<File>,
    consumed: usize,
    offset: usize,
    size: usize,
    endianness: Endianness,
}

impl ExtHeaderReader {
    fn new(path: PathBuf, offset: usize, size: usize, endianness: Endianness) -> Result<Self> {
        let file = open_file(&path)?;
        let mut reader = BufReader::new(file);

        match reader.seek(SeekFrom::Start(offset as u64)) {
            Ok(x) => x,
            Err(_) => return Err(Error::ExtHeaderSeekError),
        };
        Ok(ExtHeaderReader{
            reader,
            consumed: 0,
            offset,
            size,
            endianness,
        })
    }
}

impl Iterator for ExtHeaderReader {
    type Item = ExtKeyword;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed >= self.size {
            return None;
        }

        let mut key_length_buf = vec![0_u8; EXT_KEYWORD_LENGTH];
        self.consumed += match self.reader.read(&mut key_length_buf) {
            Ok(x) => x,
            Err(_) => return None,
        };

        // entire length of keyword block: tag, data, kwhdr & padding
        let key_length = bytes_to_i32(&key_length_buf, self.endianness).unwrap() as usize;
        let mut key_buf = vec![0_u8; key_length-EXT_KEYWORD_LENGTH];
        self.consumed += match self.reader.read(&mut key_buf) {
            Ok(x) => x,
            Err(_) => return None,
        };
        let keyword = parse_ext_keyword(&key_buf, key_length, self.endianness).unwrap();
        Some(keyword)

    }
}

pub trait BluefileReader {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized;
    fn get_ext_size(&self) -> usize;
    fn get_ext_start(&self) -> usize;
    fn get_ext_path(&self) -> PathBuf;
    fn get_data_path(&self) -> PathBuf;
    fn get_header_endianness(&self) -> Endianness;
    fn get_data_endianness(&self) -> Endianness;

    fn read_ext_header(&self) -> Result<ExtHeaderReader> {
        ExtHeaderReader::new(
            self.get_ext_path(),
            self.get_ext_start(),
            self.get_ext_size(),
            self.get_header_endianness(),
        )
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
