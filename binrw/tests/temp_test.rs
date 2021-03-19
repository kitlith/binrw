use binrw::{derive_binread, io::Cursor, BinReaderExt};

#[derive_binread]
#[derive(Default, Debug, PartialEq)]
struct Test {
    #[br(temp)]
    len: u32,

    #[br(args(len as usize, ()))]
    y: Vec<u8>,
}

#[test]
fn test_temps() {
    let mut x = Cursor::new(b"\0\0\0\x05ABCDE");

    let y: Test = x.read_be().unwrap();

    assert_eq!(
        y,
        Test {
            y: Vec::from(&b"ABCDE"[..])
        }
    );
}
