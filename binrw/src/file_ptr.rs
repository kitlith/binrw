//! A wrapper type for representing a layer of indirection within a file.
//!
//! A `RelFilePtr<P, T>` is composed of two types: a pointer type `P` and a value type `T` where
//! the pointer type describes an offset to read the value type from. Once read from the file
//! it can be dereferenced to yield the value it points to.
//!
//! ## Example
//! ```rust
//! use binrw::{prelude::*, io::Cursor, AbsFilePtr};
//!
//! #[derive(BinRead)]
//! struct Test {
//!     pointer: AbsFilePtr<u32, u8>
//! }
//!
//! let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
//! assert_eq!(test.pointer.ptr, 8);
//! assert_eq!(*test.pointer, 0xFF);
//! ```
//!
//! Example data mapped out:
//! ```hex
//!           [pointer]           [value]
//! 00000000: 0000 0008 0000 0000 ff                   ............
//! ```
//!
//! Use `offset` to change what the pointer is relative to (default: beginning of reader).
use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::{
    io::{Read, Seek, SeekFrom},
    BinRead,
    ReadOptions,
    BinResult
};

pub struct AbsPlacement<BR: BinRead> {
    value: Option<BR> // if after_parse is removed, this can be not an option anymore
}

impl<T> AbsPlacement<T> {
    fn deref_impl(&self, ty: &'static str) -> &T {
        match self.inner.value.as_ref() {
            Some(x) => x,
            None => panic!("Deref'd {0} before reading (make sure to use {0}::after_parse first)", ty)
        }
    }

    fn deref_mut_impl(&self, ty: &'static str) -> &mut T {
        match self.inner.value.as_mut() {
            Some(x) => x,
            None => panic!("Deref'd {0} before reading (make sure to use {0}::after_parse first)", ty)
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn parse<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: T::Args
    ) -> BinResult<T> where T: BinRead
    {
        let mut place: Self = Self::read_options(reader, options, args)?;
        let saved_pos = reader.seek(SeekFrom::Current(0))?;
        place.after_parse(reader, options, args)?;
        reader.seek(SeekFrom::Start(saved_pos))?;
        Ok(place.into_inner())
    }
}

/// ## Panics
/// Will panic if the AbsPlacement has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<T> Deref for AbsPlacement<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.deref_impl("AbsPlacement")
    }
}

/// ## Panics
/// Will panic if the AbsPlacement has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<T> DerefMut for AbsPlacement<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.deref_mut_impl("AbsPlacement")
    }
}

// TODO: when named args is in place
// struct PlacementArgs<T: BinRead> {
//     offset: u64,
//     inner: T::Args
// }

impl<BR: BinRead> BinRead for AbsPlacement<BR> {
    // type Args = PlacementArgs;
    type Args = (u64, BR::Args);

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        // could either precalc offset and store in struct, or do it in after_parse
        // gonna do the latter for now, former might be useful for unifying placement/fileptr
        Ok(AbsPlacement { value: None })
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: BR::Args)-> BinResult<()>
        where R: Read + Seek,
    {
        let before = reader.seek(SeekFrom::Current(0))?;
        reader.seek(SeekFrom::Start(args.0))?;

        let mut inner: BR = BinRead::read_options(reader, ro, args.1)?;

        inner.after_parse(reader, ro, args.1)?;

        self.value = Some(inner);

        reader.seek(SeekFrom::Start(before))?;
        Ok(())
    }
}

// Thought: combine AbsPlacement and RelPlacement via a parameter that says "this must always be 0"? maybe ()?
// TODO: when derive can access members of ReadOptions, derive this.
pub struct RelPlacement<BR: BinRead> {
    inner: AbsPlacement<BR>
}

/// ## Panics
/// Will panic if the RelPlacement has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<T> Deref for RelPlacement<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref_impl("RelPlacement")
    }
}

/// ## Panics
/// Will panic if the RelPlacement has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<T> DerefMut for RelPlacement<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut_impl("RelPlacement")
    }
}

impl<BR> BinRead for RelPlacement<BR> where BR: BinRead {
    type Args = (u64, BR::Args);

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, mut args: Self::Args) -> BinResult<Self> {
        // TODO: when ReadOptions can be extended with additional context members,
        //  add an extra type argument so that there can be multiple kinds of relative offsets
        args.0 += options.offset;
        Ok(RelPlacement { inner: AbsPlacement::read_options(reader, options, args)? })
    }

    fn after_parse<R: Read + Seek>(&mut self, reader: &mut R, options: &ReadOptions, mut args: Self::Args) -> BinResult<()> {
        args.0 += options.offset;
        self.inner.after_parse(reader, options, args)
    }
}

/// A wrapper type for representing a layer of indirection within a file.
///
/// A `AbsFilePtr<P, T>` is composed of two types: a pointer type `P` and a value type `T` where
/// the pointer type describes and offset to read the value type from. Once read from the file
/// it can be dereferenced to yeild the value it points to.
///
/// ## Example
/// ```rust
/// use binrw::{prelude::*, io::Cursor, AbsFilePtr};
///
/// #[derive(BinRead)]
/// struct Test {
///     pointer: AbsFilePtr<u32, u8>
/// }
///
/// let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
/// assert_eq!(test.pointer.ptr, 8);
/// assert_eq!(*test.pointer, 0xFF);
/// ```
///
/// Example data mapped out:
/// ```hex
///           [pointer]           [value]
/// 00000000: 0000 0008 0000 0000 ff                   ............
/// ```
///
/// Use `offset` to change what the pointer is relative to (default: beginning of reader).
#[derive(BinRead)]
#[br(import_tuple(args: BR::Args))]
pub struct AbsFilePtr<Ptr: IntoSeekFrom, BR: BinRead> {
    pub ptr: Ptr,
    // TODO: mark struct saying that after_parse should be passed through
    #[br(args(ptr, args))]
    inner: AbsPlacement<BR>
}

/// A wrapper type for representing a layer of indirection within a file.
///
/// A `AbsFilePtr<P, T>` is composed of two types: a pointer type `P` and a value type `T` where
/// the pointer type describes and offset to read the value type from. Once read from the file
/// it can be dereferenced to yeild the value it points to.
///
/// ## Example
/// ```rust
/// use binrw::{prelude::*, io::Cursor, AbsFilePtr};
///
/// #[derive(BinRead)]
/// struct Test {
///     pointer: AbsFilePtr<u32, u8>
/// }
///
/// let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
/// assert_eq!(test.pointer.ptr, 8);
/// assert_eq!(*test.pointer, 0xFF);
/// ```
///
/// Example data mapped out:
/// ```hex
///           [pointer]           [value]
/// 00000000: 0000 0008 0000 0000 ff                   ............
/// ```
#[derive(BinRead)]
#[br(import_tuple(args: BR::Args))]
pub struct RelFilePtr<Ptr: IntoSeekFrom, BR: BinRead> {
    pub ptr: Ptr,
    // TODO: mark struct saying that after_parse should be passed through
    #[br(args(ptr, args))]
    inner: RelPlacement<BR>
}

/// Type alias for 8-bit relative pointers
pub type RelFilePtr8<T> = RelFilePtr<u8, T>;
/// Type alias for 16-bit relative pointers
pub type RelFilePtr16<T> = RelFilePtr<u16, T>;
/// Type alias for 32-bit relative pointers
pub type RelFilePtr32<T> = RelFilePtr<u32, T>;
/// Type alias for 64-bit relative pointers
pub type RelFilePtr64<T> = RelFilePtr<u64, T>;
/// Type alias for 128-bit relative pointers
pub type RelFilePtr128<T> = RelFilePtr<u128, T>;

/// Type alias for 8-bit absolute pointers
pub type AbsFilePtr8<T> = AbsFilePtr<u8, T>;
/// Type alias for 16-bit absolute pointers
pub type AbsFilePtr16<T> = AbsFilePtr<u16, T>;
/// Type alias for 32-bit absolute pointers
pub type AbsFilePtr32<T> = AbsFilePtr<u32, T>;
/// Type alias for 64-bit absolute pointers
pub type AbsFilePtr64<T> = AbsFilePtr<u64, T>;
/// Type alias for 128-bit absolute pointers
pub type AbsFilePtr128<T> = AbsFilePtr<u128, T>;

// impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, BR: BinRead> BinRead for RelFilePtr<Ptr, BR> {
//     type Args = BR::Args;
//
//     fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
//         Ok(RelFilePtr{
//             ptr: Ptr::read_options(reader, options, ())?,
//             value: None
//         })
//     }
//
//     fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: BR::Args)-> BinResult<()>
//         where R: Read + Seek,
//     {
//         let relative_to = ro.offset;
//         let before = reader.seek(SeekFrom::Current(0))?;
//         reader.seek(SeekFrom::Start(relative_to))?;
//         reader.seek(self.ptr.into_seek_from())?;
//
//         let mut inner: BR = BinRead::read_options(reader, ro, args)?;
//
//         inner.after_parse(reader, ro, args)?;
//
//         self.value = Some(inner);
//
//         reader.seek(SeekFrom::Start(before))?;
//         Ok(())
//     }
// }

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, BR: BinRead> AbsFilePtr<Ptr, BR> {
    /// Custom parser designed for use with the `parse_with` attribute ([example](crate::attribute#custom-parsers))
    /// that reads a [`AbsFilePtr`](AbsFilePtr) then immediately dereferences it into an owned value
    pub fn parse<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: BR::Args
    ) -> BinResult<BR>
    {
        let mut ptr: Self = Self::read_options(reader, options, args)?;
        let saved_pos = reader.seek(SeekFrom::Current(0))?;
        ptr.after_parse(reader, options, args)?;
        reader.seek(SeekFrom::Start(saved_pos))?;
        Ok(ptr.into_inner())
    }

    /// Consume the pointer and return the inner type
    ///
    /// # Panics
    ///
    /// Will panic if the file pointer hasn't been properly postprocessed
    pub fn into_inner(self) -> BR {
        self.value.unwrap()
    }
}

/// Used to allow any convert any type castable to i64 into a [`SeekFrom::Current`](io::SeekFrom::Current)
pub trait IntoSeekFrom: Copy {
    fn into_seek_from(self) -> SeekFrom;
}

macro_rules! impl_into_seek_from {
    ($($t:ty),*) => {
        $(
            impl IntoSeekFrom for $t {
                fn into_seek_from(self) -> SeekFrom {
                    SeekFrom::Current(self as i64)
                }
            }
        )*
    };
}

impl_into_seek_from!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

/// ## Panics
/// Will panic if the AbsFilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr, T> Deref for AbsFilePtr<Ptr, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref_impl("AbsFilePtr")
    }
}

/// ## Panics
/// Will panic if the AbsFilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr, T> DerefMut for AbsFilePtr<Ptr, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut_impl("AbsFilePtr")
    }
}

/// ## Panics
/// Will panic if the RelFilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr, T> Deref for RelFilePtr<Ptr, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.inner.deref_impl("RelFilePtr")
    }
}

/// ## Panics
/// Will panic if the RelFilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr, T> DerefMut for RelFilePtr<Ptr, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.inner.deref_mut_impl("RelFilePtr")
    }
}

impl<Ptr, BR> fmt::Debug for AbsFilePtr<Ptr, BR>
    where Ptr: BinRead<Args = ()> + IntoSeekFrom,
          BR: BinRead + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.value {
            fmt::Debug::fmt(value, f)
        } else {
            write!(f, "UnreadPointer")
        }
    }
}

impl<Ptr, BR> PartialEq<AbsFilePtr<Ptr, BR>> for AbsFilePtr<Ptr, BR>
    where Ptr: BinRead<Args = ()> + IntoSeekFrom,
          BR: BinRead + PartialEq,
{

    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
