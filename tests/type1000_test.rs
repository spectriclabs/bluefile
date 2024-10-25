use std::fs::File;
use std::path::PathBuf;
use std::str::from_utf8;

use bluefile::bluefile::{TypeCode};
use bluefile::data_type::{Format, Rank};
use bluefile::endian::Endianness;
use bluefile::header::{
    HeaderKeyword,
    read_type1000_adjunct_header,
    read_header,
};

#[test]
fn read_type1000_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/sin.tmp");
    let file = File::open(&d).unwrap();
    let header = read_header(&file).unwrap();

    assert_eq!(header.header_endianness, Endianness::Little);
    assert_eq!(header.data_endianness, Endianness::Little);
    assert_eq!(header.ext_start, 0);
    assert_eq!(header.ext_size, 0);
    assert_eq!(header.data_start, 512.0);
    assert_eq!(header.data_size, 32768.0);
    assert_eq!(header.type_code, TypeCode::Type1000(1000));
    assert_eq!(header.data_type.rank, Rank::Scalar);
    assert_eq!(header.data_type.format, Format::Double);
    assert_eq!(header.timecode, 0.0);
    assert_eq!(header.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
    assert_eq!(header.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});

    let adjunct = read_type1000_adjunct_header(&file, &header).unwrap();
    assert_eq!(adjunct.xstart, 0.0);
    assert_eq!(adjunct.xdelta, 1.0);
    assert_eq!(adjunct.xunits, 0);
}

#[test]
fn read_type1000_complex_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/pulse_cx.tmp");
    let file = File::open(&d).unwrap();
    let header = read_header(&file).unwrap();

    assert_eq!(header.header_endianness, Endianness::Little);
    assert_eq!(header.data_endianness, Endianness::Little);
    assert_eq!(header.ext_start, 0);
    assert_eq!(header.ext_size, 0);
    assert_eq!(header.data_start, 512.0);
    assert_eq!(header.data_size, 1600.0);
    assert_eq!(header.type_code, TypeCode::Type1000(1000));
    assert_eq!(header.data_type.rank, Rank::Complex);
    assert_eq!(header.data_type.format, Format::Float);
    assert_eq!(header.timecode, 0.0);
    assert_eq!(header.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
    assert_eq!(header.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});

    let adjunct = read_type1000_adjunct_header(&file, &header).unwrap();
    assert_eq!(adjunct.xstart, 0.0);
    assert_eq!(adjunct.xdelta, 1.0);
    assert_eq!(adjunct.xunits, 1);
}
