use std::fs::File;
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
use crate::endian::Endianness;
use crate::error::Error;
use crate::header::{Header, read_header};
use crate::result::Result;
use crate::util::{
    bytes_to_f64,
    bytes_to_i32,
    open_file,
};

pub struct Type2000Adjunct {
    pub xstart: f64,
    pub xdelta: f64,
    pub xunits: i32,
    pub subsize: i32,
    pub ystart: f64,
    pub ydelta: f64,
    pub yunits: i32,
}

pub struct Type2000Reader {
    ext_path: PathBuf,
    data_path: PathBuf,
    pub header: Header,
    pub adj_header: Type2000Adjunct,
}

impl BluefileReader for Type2000Reader {
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

    fn get_ext_size(&self) -> usize {
        self.header.ext_size
    }

    fn get_ext_start(&self) -> usize {
        self.header.ext_start
    }

    fn get_ext_path(&self) -> PathBuf {
        self.ext_path.clone()
    }

    fn get_data_path(&self) -> PathBuf {
        self.data_path.clone()
    }

    fn get_header_endianness(&self) -> Endianness {
        self.header.header_endianness
    }

    fn get_data_endianness(&self) -> Endianness {
        self.header.data_endianness
    }
}
