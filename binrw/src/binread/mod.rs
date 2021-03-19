use core::any::{Any, TypeId};
use typemap_core::{Contains, Ty, TyEnd};

#[cfg(feature = "debug_template")]
use crate::binary_template;
use crate::io::{Read, Seek};
pub use crate::options::{self, ReadOptionsExt};
use crate::{BinResult, Endian};

mod impls;

/// A `BinRead` trait allows reading a structure from anything that implements [`io::Read`](io::Read) and [`io::Seek`](io::Seek)
/// BinRead is implemented on the type to be read out of the given reader
pub trait BinRead<Opts>: Sized {
    /// The type of arguments needed to be supplied in order to read this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will allow [`read`](BinRead::read) to be used.
    type Args: Any + Copy;

    /// Read the type from the reader
    fn read_options<R>(reader: &mut R, options: &Opts, args: Self::Args) -> BinResult<Self>
    where
        R: Read + Seek;

    fn after_parse<R>(&mut self, _: &mut R, _: &Opts, _: Self::Args) -> BinResult<()>
    where
        R: Read + Seek,
    {
        Ok(())
    }

    /// The default arguments to be used when using the [`read`](BinRead::read) shortcut method.
    /// Override this for any type that optionally requries arguments
    fn args_default() -> Option<Self::Args> {
        // Trick to effectively get specialization on stable, should constant-folded away
        // Returns `Some(())` if Self::Args == (), otherwise returns `None`
        if TypeId::of::<Self::Args>() == TypeId::of::<()>() {
            Some(unsafe { core::mem::MaybeUninit::uninit().assume_init() })
        } else {
            None
        }
    }
}

pub trait BinReadExt: BinRead<Ty<Endian, TyEnd>> {
    /// Read the type from the reader while assuming no arguments have been passed
    ///
    /// # Panics
    /// Panics if there is no [`args_default`](BinRead::args_default) implementation
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        let args = match Self::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented"),
        };

        Self::read_options(reader, &Ty::new(Endian::default(), TyEnd), args)
    }

    /// Read the type from the reader using the specified arguments
    fn read_args<R>(reader: &mut R, args: Self::Args) -> BinResult<Self>
    where
        R: Read + Seek,
    {
        Self::read_options(reader, &Ty::new(Endian::default(), TyEnd), args)
    }
}

impl<T: BinRead<Ty<Endian, TyEnd>>> BinReadExt for T {}

/// An extension trait for [`io::Read`](io::Read) to provide methods for reading a value directly
///
/// ## Example
/// ```rust
/// use binrw::prelude::*; // BinReadExt is in the prelude
/// use binrw::endian::LE;
/// use std::io::Cursor;
///
/// fn main() {
///     let mut reader = Cursor::new(b"\x07\0\0\0\xCC\0\0\x05");
///     let x: u32 = reader.read_le().unwrap();
///     let y: u16 = reader.read_type(LE).unwrap();
///     let z = reader.read_be::<u16>().unwrap();
///
///     assert_eq!((x, y, z), (7u32, 0xCCu16, 5u16));
/// }
/// ```
pub trait BinReaderExt: Read + Seek + Sized {
    /// Read the given type from the reader using the given endianness.
    fn read_type<T: BinRead<Ty<Endian, TyEnd>>>(&mut self, endian: Endian) -> BinResult<T> {
        let args = match T::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented"),
        };

        let options = Ty::new(endian, TyEnd);

        let mut res = T::read_options(self, &options, args)?;
        res.after_parse(self, &options, args)?;

        Ok(res)
    }

    /// Read the given type from the reader with big endian byteorder
    fn read_be<T: BinRead<Ty<Endian, TyEnd>>>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Big)
    }

    /// Read the given type from the reader with little endian byteorder
    fn read_le<T: BinRead<Ty<Endian, TyEnd>>>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Little)
    }

    /// Read the given type from the reader with the native byteorder
    fn read_ne<T: BinRead<Ty<Endian, TyEnd>>>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Native)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
