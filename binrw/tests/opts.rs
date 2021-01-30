use binrw::{Endian, Error};
use binrw::io::Cursor;
use binrw::{BinRead, BinReaderExt};

#[derive(BinRead, Debug, PartialEq)]
struct EndianTest {
    #[br(parse_with = Endian::parse_bom, set_opts(Endian = bom))]
    bom: Endian,
    test: u16
}

#[test]
fn test_set_opts() {
    let test: EndianTest = Cursor::new(b"\xFF\xFE\x01\x05").read_ne().unwrap();
    assert_eq!(test, EndianTest { bom: Endian::Little, test: 0x0501 });

    Cursor::new(b"\xFE\xFE\x01\x05").read_ne::<EndianTest>().unwrap_err();
    // match
    // assert_eq!(test, 0);
}