use crate::{BinResult, ReadOptions, io::{Read, Seek}};
use crate::alloc::{vec::Vec, vec};

mod file_ptr;
pub use file_ptr::*;

mod punctuated;
pub use punctuated::*;
use crate::options::Options;
use crate::binread::options::ReadOptionsExt;

/// A helper for more efficiently mass-reading bytes
///
///## Example:
///
/// ```rust
/// # use binrw::{BinRead, helpers::read_bytes, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct BunchaBytes {
///     #[br(count = 5)]
///     data: Vec<u8>
/// }
///
/// # let mut x = Cursor::new(b"\0\x01\x02\x03\x04");
/// # let x: BunchaBytes = x.read_be().unwrap();
/// # assert_eq!(x.data, &[0, 1, 2, 3, 4]);
/// ```
pub fn read_bytes<R: Read + Seek>(reader: &mut R, options: &Options, _: ()) -> BinResult<Vec<u8>> {
    let count = match options.count() {
        Some(x) => x,
        None => panic!("Missing count for read_bytes")
    };
    let mut buf = vec![0; count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}
