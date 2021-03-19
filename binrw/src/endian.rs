//! An enum to represent what endianness to read as

use crate::alloc::string::String;
use crate::io::{Read, Seek, SeekFrom};
use crate::{BinRead, BinResult};

/// An enum to represent what endianness to read as
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endian {
    Big,
    Little,
    Native,
}

use typemap_core::{Ty, TyEnd};
pub use Endian::{Big as BE, Little as LE, Native as NE};

impl Endian {
    pub fn from_be_bom(bom: u16) -> Option<Self> {
        match bom {
            0xFEFF => Some(Self::Big),
            0xFFFE => Some(Self::Little),
            _ => None,
        }
    }

    pub fn from_le_bom(bom: u16) -> Option<Self> {
        match bom {
            0xFEFF => Some(Self::Little),
            0xFFFE => Some(Self::Big),
            _ => None,
        }
    }

    pub fn parse_bom<R: Read + Seek, Opts>(
        reader: &mut R,
        _: &Opts,
        _: (),
    ) -> BinResult<Self> {
        let pos = reader.seek(SeekFrom::Current(0))?;

        let options = Ty::new(Endian::Big, TyEnd);

        let bom = u16::read_options(reader, &options, ())?;

        Endian::from_be_bom(bom).ok_or_else(|| crate::Error::BadMagic {
            pos,
            found: Box::new(bom),
        })
    }
}

impl Into<String> for &Endian {
    fn into(self) -> String {
        String::from(match self {
            Endian::Big => "Big",
            Endian::Little => "Little",
            Endian::Native => "Native",
        })
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Native
    }
}
