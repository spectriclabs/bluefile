use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use crate::bluefile::{
    ADJUNCT_HEADER_OFFSET,
    ADJUNCT_HEADER_SIZE,
    BluefileReader,
    TypeCode,
};
use crate::data_type::{bytes_to_data_value, DataType, DataValue};
use crate::endian::Endianness;
use crate::error::Error;
use crate::header::{Header, read_header};
use crate::result::Result;
use crate::util::{
    bytes_to_f64,
    bytes_to_i32,
    open_file,
};

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

pub struct Type2000DataIter {
    reader: BufReader<File>,
    consumed: usize,
    offset: usize,
    size: usize,
    endianness: Endianness,
    data_type: DataType,
    adjunct: Type2000Adjunct,
    buf: Vec<u8>,
}

impl Type2000DataIter {
    fn new(path: PathBuf, offset: usize, size: usize, endianness: Endianness, data_type: DataType, adjunct: Type2000Adjunct) -> Result<Self> {
        let file = open_file(&path)?;
        let mut reader = BufReader::new(file);

        match reader.seek(SeekFrom::Start(offset as u64)) {
            Ok(x) => x,
            Err(_) => return Err(Error::DataSeekError),
        };

        let buf = vec![0_u8; data_type.size()];

        Ok(Type2000DataIter{
            reader,
            consumed: 0,
            offset,
            size,
            endianness,
            data_type,
            adjunct,
            buf,
        })
    }
}

impl Iterator for Type2000DataIter {
    type Item = DataValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed >= self.size {
            return None;
        }

        self.consumed += match self.reader.read_exact(&mut self.buf) {
            Ok(_) => self.data_type.size(),
            Err(_) => return None,
        };

        let value = bytes_to_data_value(&self.data_type, self.endianness, &self.buf).expect("Bytes must convert to expected DataType");
        Some(value)
    }
}

pub struct Type2000Reader {
    ext_path: PathBuf,
    data_path: PathBuf,
    header: Header,
    adj_header: Type2000Adjunct,
}

impl BluefileReader for Type2000Reader {
    type AdjHeader = Type2000Adjunct;
    type DataIter = Type2000DataIter;

    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        let mut file = open_file(&path_buf)?;
        let header = read_header(&file)?;

        match header.type_code {
            TypeCode::Type2000(x) => x,
            _ => return Err(Error::TypeCodeMismatchError),
        };

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

        let adj_header = Type2000Adjunct{
            xstart,
            xdelta,
            xunits,
            subsize,
            ystart,
            ydelta,
            yunits,
        };

        // TODO: Add support for detatched header path
        Ok(Self {
            ext_path: path_buf.clone(),
            data_path: path_buf.clone(),
            header,
            adj_header,
        })
    }

    fn get_header(&self) -> Header {
        self.header.clone()
    }

    fn get_ext_size(&self) -> usize {
        self.header.ext_size
    }

    fn get_ext_start(&self) -> usize {
        self.header.ext_start
    }

    fn get_ext_path(&self) -> PathBuf {
        self.ext_path.clone()
    }

    fn get_adj_header(&self) -> Self::AdjHeader {
        self.adj_header.clone()
    }

    fn get_data_start(&self) -> usize {
        self.header.data_start as usize
    }

    fn get_data_size(&self) -> usize {
        self.header.data_size as usize
    }

    fn get_data_path(&self) -> PathBuf {
        self.data_path.clone()
    }

    fn get_data_iter(&self) -> Result<Self::DataIter> {
        Type2000DataIter::new(
            self.get_data_path(),
            self.get_data_start(),
            self.get_data_size(),
            self.get_data_endianness(),
            self.header.data_type.clone(),
            self.get_adj_header().clone(),
        )
    }

    fn get_header_endianness(&self) -> Endianness {
        self.header.header_endianness
    }

    fn get_data_endianness(&self) -> Endianness {
        self.header.data_endianness
    }
}
