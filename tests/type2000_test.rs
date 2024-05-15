use std::path::PathBuf;
use std::str::from_utf8;

use bluefile::bluefile::{BluefileReader, ExtKeyword, TypeCode};
use bluefile::data_type::{DataValue, Format, Rank};
use bluefile::endian::Endianness;
use bluefile::header::HeaderKeyword;
use bluefile::type2000::Type2000Reader;

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
    assert_eq!(header.data_type.rank, Rank::Scalar);
    assert_eq!(header.data_type.format, Format::Double);
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

    let ext_reader = (&reader).get_ext_iter().unwrap();
    let ext_keywords: Vec<ExtKeyword> = ext_reader.collect();
    assert_eq!(ext_keywords.len(), 5);

    assert_eq!(ext_keywords[0].tag, "COMMENT".to_string());
    assert_eq!(ext_keywords[0].format, 'A');
    assert_eq!(from_utf8(&ext_keywords[0].value).unwrap(), "Demo data for XRTSURFACE/STAY".to_string());

    assert_eq!(ext_keywords[4].tag, "COMMENT3".to_string());
    assert_eq!(ext_keywords[4].format, 'A');
    assert_eq!(from_utf8(&ext_keywords[4].value).unwrap(), "XRTSURF/STAY/NOLAB/XC=5,PENNY,1.0,255.0,4,128,16,0,10,2".to_string());

    let data_reader = (&reader).get_data_iter().unwrap();
    let mut frame_count = 0;

    for frame in data_reader {
        assert_eq!(frame.frame.len(), 128);
        frame_count += 1;

        for item in frame.frame {
            match item.value {
                DataValue::SD(_) => continue,
                _ => panic!("Expected Scalar Double, but got {:?}", item),
            }
        }
    }

    assert_eq!(frame_count, 128);
}
