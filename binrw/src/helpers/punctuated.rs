//! A module for [`Punctuated<T, P>`](Punctuated), a series of items to parse of type T separated
//! by punction of type `P`.

use crate::alloc::vec::Vec;
use crate::io::{Read, Seek};
use crate::{BinRead, BinResult};
use core::fmt;

/// A type for seperated data. Since parsing for this type is ambiguous, you must manually specify
/// a parser using the `parse_with` attribute.
///
/// ## Example
///
/// ```rust
/// # use binrw::{*, io::*};
/// use binrw::helpers::Punctuated;
///
/// #[derive(BinRead)]
/// struct MyList {
///     #[br(parse_with = Punctuated::separated)]
///     #[br(args(3, ()))]
///     x: Punctuated<u16, u8>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x03\0\0\x02\x01\0\x01");
/// # let y: MyList = x.read_be().unwrap();
/// # assert_eq!(*y.x, vec![3, 2, 1]);
/// # assert_eq!(y.x.seperators, vec![0, 1]);
/// ```
pub struct Punctuated<T, P> {
    data: Vec<T>,
    pub seperators: Vec<P>,
}

impl<T, P> Punctuated<T, P> {
    /// A parser for values seperated by another value, with no trailing punctuation.
    ///
    /// Requires a specified count.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use binrw::{*, io::*};
    /// use binrw::helpers::Punctuated;
    ///
    /// #[derive(BinRead)]
    /// struct MyList {
    ///     #[br(parse_with = Punctuated::separated)]
    ///     #[br(args(3, ()))]
    ///     x: Punctuated<u16, u8>,
    /// }
    ///
    /// # let mut x = Cursor::new(b"\0\x03\0\0\x02\x01\0\x01");
    /// # let y: MyList = x.read_be().unwrap();
    /// # assert_eq!(*y.x, vec![3, 2, 1]);
    /// # assert_eq!(y.x.seperators, vec![0, 1]);
    /// ```
    pub fn separated<R: Read + Seek, Opts>(
        reader: &mut R,
        options: &Opts,
        (count, args): (usize, T::Args),
    ) -> BinResult<Self>
    where
        T: BinRead<Opts>,
        T::Args: Copy + 'static,
        P: BinRead<Opts, Args = ()>,
    {
        let mut data = Vec::with_capacity(count);
        let mut seperators = Vec::with_capacity(count.max(1) - 1);

        for i in 0..count {
            data.push(T::read_options(reader, options, args)?);
            if i + 1 != count {
                seperators.push(P::read_options(reader, options, ())?);
            }
        }

        Ok(Self { data, seperators })
    }

    /// A parser for values seperated by another value, with trailing punctuation.
    ///
    /// Requires a specified count.
    pub fn separated_trailing<R: Read + Seek, Opts>(
        reader: &mut R,
        options: &Opts,
        (count, args): (usize, T::Args),
    ) -> BinResult<Self>
    where
        T: BinRead<Opts>,
        T::Args: Copy + 'static,
        P: BinRead<Opts, Args = ()>,
    {
        let mut data = Vec::with_capacity(count);
        let mut seperators = Vec::with_capacity(count);

        for _ in 0..count {
            data.push(T::read_options(reader, options, args)?);
            seperators.push(P::read_options(reader, options, ())?);
        }

        Ok(Self { data, seperators })
    }

    /// Convert into a `Vec` of the values without the separators
    pub fn into_values(self) -> Vec<T> {
        let Self { data, .. } = self;

        data
    }
}

impl<T: fmt::Debug, P> fmt::Debug for Punctuated<T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl<T, P> core::ops::Deref for Punctuated<T, P> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, P> core::ops::DerefMut for Punctuated<T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
