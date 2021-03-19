//! A module for [`Counted<T>`], a series of items to parse of type T.

use crate::alloc::vec::Vec;
use crate::io::{Read, Seek};
#[cfg(feature = "debug_template")]
use crate::binary_template;
use crate::{BinRead, BinResult, Endian};
use core::fmt;
use typemap_core::{Contains};

/// A type for counted data.
///
/// ## Example
///
/// ```rust
/// # use binrw::{*, io::*};
/// use binrw::helpers::Counted;
///
/// #[derive(BinRead)]
/// struct MyList {
///     #[br(parse_with = Punctuated::separated)]
///     #[br(args(3, ()))]
///     x: Counted<u16>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x03\0\x02\0\x01");
/// # let y: MyList = x.read_be().unwrap();
/// # assert_eq!(*y.x, vec![3, 2, 1]);
/// ```
pub struct Counted<T> {
    data: Vec<T>,
}

impl<Opts: Contains<Endian>, C: Copy + 'static, B: BinRead<Opts, Args = C>> BinRead<Opts>
for Counted<B>
{
    type Args = (usize, B::Args);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &Opts,
        (count, args): Self::Args,
    ) -> BinResult<Self> {
        #[cfg(feature = "debug_template")]
            let options = {
            let pos = reader.seek(crate::SeekFrom::Current(0))?;
            let type_name = core::any::type_name::<B>().rsplitn(1, "::").nth(0).unwrap();

            // this is a massive hack. I'm so sorry
            let type_name = if type_name.starts_with("binread::file_ptr::FilePtr<") {
                // Extract the backing type name from file pointers
                type_name
                    .trim_start_matches("binread::file_ptr::FilePtr<")
                    .split(",")
                    .nth(0)
                    .unwrap()
            } else {
                type_name
            };

            // TODO: this is the only reason this impl needs Contains<Endian>
            binary_template::write_vec(*options.get::<Endian>(), pos, type_name, count);

            typemap_core::Ty::new(options::DontOutputTemplate(true), options)
        };

        let data: BinResult<_> = (0..count)
            .map(|_| B::read_options(reader, &options, args))
            .collect();

        Ok(Counted { data: data? })
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &Opts, (_, args): Self::Args) -> BinResult<()>
        where
            R: Read + Seek,
    {
        for val in self.iter_mut() {
            val.after_parse(reader, ro, args)?;
        }

        Ok(())
    }
}

impl<T> Counted<T> {
    pub fn into_values(self) -> Vec<T> {
        self.data
    }
}

impl<T: fmt::Debug> fmt::Debug for Counted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl<T> core::ops::Deref for Counted<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> core::ops::DerefMut for Counted<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}