use std::fs::File;
use std::path::PathBuf;

use bluefile::{
    DataType,
    Endianness,
    Header,
    read_header,
};

#[test]
fn read_bad_header_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/bad_header.tmp");
    let file = File::open(&d).unwrap();
    let _header: Header = match read_header(&file) {
        Ok(_) => panic!("This header should have produced an error"),
        Err(_) => Header{
            header_endianness: Endianness::Little,
            data_endianness: Endianness::Little,
            ext_start: 0,
            ext_size: 0,
            data_start: 0.0,
            data_size: 0.0,
            type_code: 1000,
            data_type: DataType{format: 0, rank: 0},
            timecode: 0.0,
            keywords: vec![],
        },
    };
}
