use std::fs::File;
use std::path::PathBuf;
use std::str::from_utf8;

use bluefile::{
    DataType,
    Endianness,
    HeaderKeyword,
    read_ext_header,
    read_type2000_adjunct_header,
    read_header,
    TypeCode,
};

#[test]
fn read_type2000_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/penny.prm");
    let file = File::open(&d).unwrap();
    let header = read_header(&file).unwrap();

    assert_eq!(header.header_endianness, Endianness::Little);
    assert_eq!(header.data_endianness, Endianness::Little);
    assert_eq!(header.ext_start, 257*512);
    assert_eq!(header.ext_size, 320);
    assert_eq!(header.data_start, 512.0);
    assert_eq!(header.data_size, 131072.0);
    assert_eq!(header.type_code, 2000 as TypeCode);
    assert_eq!(header.data_type, DataType{rank: b'S', format: b'D'});
    assert_eq!(header.timecode, 0.0);
    assert_eq!(header.keywords[0], HeaderKeyword{name: "VER".to_string(), value: "1.1".to_string()});
    assert_eq!(header.keywords[1], HeaderKeyword{name: "IO".to_string(), value: "X-Midas".to_string()});

    let adjunct = read_type2000_adjunct_header(&file, &header).unwrap();
    assert_eq!(adjunct.xstart, 0.0);
    assert_eq!(adjunct.xdelta, 1.0);
    assert_eq!(adjunct.xunits, 0);
    assert_eq!(adjunct.subsize, 128);
    assert_eq!(adjunct.ystart, 0.0);
    assert_eq!(adjunct.ydelta, 1.0);
    assert_eq!(adjunct.yunits, 0);

    let ext_keywords = read_ext_header(&file, &header).unwrap();
    assert_eq!(ext_keywords.len(), 5);

    assert_eq!(ext_keywords[0].tag, "COMMENT".to_string());
    assert_eq!(ext_keywords[0].value.format, 'A');
    assert_eq!(from_utf8(&ext_keywords[0].value.raw_value).unwrap(), "Demo data for XRTSURFACE/STAY".to_string());

    assert_eq!(ext_keywords[4].tag, "COMMENT3".to_string());
    assert_eq!(ext_keywords[4].value.format, 'A');
    assert_eq!(from_utf8(&ext_keywords[4].value.raw_value).unwrap(), "XRTSURF/STAY/NOLAB/XC=5,PENNY,1.0,255.0,4,128,16,0,10,2".to_string());
}
