use std::fs::File;
use std::io::Result;
use std::io::Error;
use std::io::Read;
use std::path::Path;

const HEADER_SIZE: usize = 512;

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
pub struct Header {
    pub header_rep: Endianness,
    pub data_rep: Endianness,
    pub ext_start: u32,  // in 512 byte blocks
    pub ext_size: u32,  // in bytes
    pub data_start: f64,  // in bytes
    pub data_size: f64,  // in bytes
    pub type_code: TypeCode,
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
        Err(Error::other("Invalid endianness"))
    }
}

fn bytes_to_u32(v: &[u8], endianness: Endianness) -> Result<u32> {
    let b: [u8; 4] = match v.try_into() {
        Ok(x) => x,
        Err(e) => return Err(Error::other(e)),
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
        Err(e) => return Err(Error::other(e)),
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
        let msg = format!("Unknown file type code {t}");
        Err(Error::other(msg))
    }
}

pub fn parse_header(data: &[u8]) -> Result<Header> {
    if !is_blue(&data[0..4]) {
        return Err(Error::other("Not a BLUE file"));
    }

    let header_rep = parse_endianness(&data[4..8])?;
    let data_rep = parse_endianness(&data[8..12])?;
    let ext_start = bytes_to_u32(&data[24..28], header_rep)?;
    let ext_size = bytes_to_u32(&data[28..32], header_rep)?;
    let data_start = bytes_to_f64(&data[32..40], header_rep)?;
    let data_size = bytes_to_f64(&data[40..48], header_rep)?;
    let type_code = parse_type_code(&data[48..52], header_rep)?;
    Ok(Header{
        header_rep,
        data_rep,
        ext_start,
        ext_size,
        data_start,
        data_size,
        type_code,
    })
}

pub fn read_header(path: &Path) -> Result<Header> {
    let mut file = File::open(path)?;
    let mut data = vec![0_u8; HEADER_SIZE];
    let n = file.read(&mut data)?;

    if n < HEADER_SIZE {
        let msg = format!("Only read {n} bytes from {}", path.display());
        return Err(Error::other(msg));
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
        assert_eq!(header.as_ref().unwrap().header_rep, Endianness::Little);
        assert_eq!(header.as_ref().unwrap().data_rep, Endianness::Little);
        assert_eq!(header.as_ref().unwrap().ext_start, 257);
        assert_eq!(header.as_ref().unwrap().ext_size, 320);
        assert_eq!(header.as_ref().unwrap().data_start, 512.0);
        assert_eq!(header.as_ref().unwrap().data_size, 131072.0);
        assert_eq!(header.as_ref().unwrap().type_code, TypeCode::T2000);
    }
}
