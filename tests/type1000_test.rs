use std::path::PathBuf;
use std::str::from_utf8;

use bluefile::bluefile::{BluefileReader, TypeCode};
use bluefile::data_type::{DataValue, Format, Rank};
use bluefile::endian::Endianness;
use bluefile::header::HeaderKeyword;
use bluefile::type1000::Type1000Reader;

#[test]
fn read_type1000_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/sin.tmp");
    let reader = Type1000Reader::new(&d).unwrap();
    let header = &reader.get_header();

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

    let adjunct = &reader.get_adj_header();
    assert_eq!(adjunct.xstart, 0.0);
    assert_eq!(adjunct.xdelta, 1.0);
    assert_eq!(adjunct.xunits, 0);

    let data_reader = (&reader).get_data_iter().unwrap();
    let mut count = 0;

    for value in data_reader {
        count += 1;

        match value {
            DataValue::SD(_) => continue,
            _ => panic!("Expected Scalar Double, but got {:?}", value),
        };
    }

    let bytes_per_element = 8;
    let data_size = 32768;
    assert_eq!(count, data_size / bytes_per_element);
}

#[test]
fn read_type1000_complex_test() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/pulse_cx.tmp");
    let reader = Type1000Reader::new(&d).unwrap();
    let header = &reader.get_header();

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

    let adjunct = &reader.get_adj_header();
    assert_eq!(adjunct.xstart, 0.0);
    assert_eq!(adjunct.xdelta, 1.0);
    assert_eq!(adjunct.xunits, 1);

    let data_reader = (&reader).get_data_iter().unwrap();
    let mut count = 0;

    for value in data_reader {
        count += 1;

        match value {
            DataValue::CF(_) => continue,
            _ => panic!("Expected Complex Float, but got {:?}", value),
        };
    }

    let bytes_per_element = 8;
    let data_size = 1600;
    assert_eq!(count, data_size / bytes_per_element);
}
