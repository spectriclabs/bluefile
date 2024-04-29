use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;

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
    header: Header,
    adj_header: Type2000Adjunct,
}

impl BluefileReader for Type2000Reader {
    fn new(path: &PathBuf) -> Result<Self> {
        let mut file = open_file(path)?;
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
            ext_path: path.clone(),
            data_path: path.clone(),
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

    fn get_ext_path(&self) -> &PathBuf {
        &self.ext_path
    }

    fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }

    fn get_header_endianness(&self) -> Endianness {
        self.header.header_endianness
    }

    fn get_data_endianness(&self) -> Endianness {
        self.header.data_endianness
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    use crate::header::HeaderKeyword;
    use crate::bluefile::ExtKeyword;

    #[test]
    fn read_type2000_test() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/penny.prm");
        let reader = Type2000Reader::new(&d).unwrap();
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

        let adjunct = &reader.adj_header;
        assert_eq!(adjunct.xstart, 0.0);
        assert_eq!(adjunct.xdelta, 1.0);
        assert_eq!(adjunct.xunits, 0);
        assert_eq!(adjunct.subsize, 128);
        assert_eq!(adjunct.ystart, 0.0);
        assert_eq!(adjunct.ydelta, 1.0);
        assert_eq!(adjunct.yunits, 0);

        let ext_reader = (&reader).read_ext_header().unwrap();
        let ext_keywords: Vec<ExtKeyword> = ext_reader.collect();
        assert_eq!(ext_keywords.len(), 5);

        assert_eq!(ext_keywords[0].tag, "COMMENT".to_string());
        assert_eq!(ext_keywords[0].format, 'A');
        assert_eq!(from_utf8(&ext_keywords[0].value).unwrap(), "Demo data for XRTSURFACE/STAY".to_string());

        assert_eq!(ext_keywords[4].tag, "COMMENT3".to_string());
        assert_eq!(ext_keywords[4].format, 'A');
        assert_eq!(from_utf8(&ext_keywords[4].value).unwrap(), "XRTSURF/STAY/NOLAB/XC=5,PENNY,1.0,255.0,4,128,16,0,10,2".to_string());
    }
}
