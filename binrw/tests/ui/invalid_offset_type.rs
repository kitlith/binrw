use binrw::{BinRead, RelFilePtr};

#[derive(BinRead)]
struct Test {
    a: u8,
    #[br(offset = a)]
    b: RelFilePtr<u8, u8>,
    #[br(offset_after = d)]
    c: RelFilePtr<u8, u8>,
    d: u8,
}

fn main() {}
