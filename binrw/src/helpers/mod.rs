use crate::{BinResult, ReadOptions, io::{Read, Seek}};
use crate::alloc::{vec::Vec, vec};

mod file_ptr;
pub use file_ptr::*;

mod punctuated;
pub use punctuated::*;

/// A helper for more efficiently mass-reading bytes
///
///## Example:
///
/// ```rust
/// # use binrw::{BinRead, helpers::read_bytes, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct BunchaBytes {
///     #[br(args(5), parse_with = read_bytes)]
///     data: Vec<u8>
/// }
///
/// # let mut x = Cursor::new(b"\0\x01\x02\x03\x04");
/// # let x: BunchaBytes = x.read_be().unwrap();
/// # assert_eq!(x.data, &[0, 1, 2, 3, 4]);
/// ```
pub fn read_bytes<R: Read + Seek>(reader: &mut R, _options: &ReadOptions, (count,): (usize,)) -> BinResult<Vec<u8>> {
    let mut buf = vec![0; count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}
