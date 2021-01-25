use crate::Endian;
use rpds::HashTrieMap;
use std::any::{TypeId, Any};
use std::mem::transmute;

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
#[non_exhaustive]
#[derive(Default, Clone)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    pub offset: u64,

    #[cfg(feature = "debug_template")]
    pub dont_output_to_template: bool,
    #[cfg(feature = "debug_template")]
    pub variable_name: Option<&'static str>,

    ext: HashTrieMap<TypeId, Box<dyn Any>>
}

#[repr(transparent)]
#[derive(Debug, PartialEq)]
struct VecCount(pub usize);
#[repr(transparent)]
#[derive(Debug, PartialEq)]
struct FileOffset(pub u64);

impl ReadOptions {
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let ty = TypeId::of::<T>();
        if ty == TypeId::of::<Endian>() {
            Any::downcast_ref(&self.endian)
        } else if ty == TypeId::of::<VecCount>() {
            // we store as usize, not as VecCount, so we need to transmute.
            // this should be safe because VecCount is repr(transparent)
            // TODO: get rid of this by storing as VecCount
            self.count.as_ref().map(|c| unsafe { transmute(c) })
        } else if ty == TypeId::of::<FileOffset>() {
            // we store as u64, not as FileOffset, so we need to transmute
            // this should be safe because FileOffset is repr(transparent)
            // TODO: get rid of this by storing as FileOffset
            unsafe { Some(transmute(&self.offset )) }
        } else {
            self.ext
                .get(&ty)
                .map(AsRef::as_ref)
                .and_then(Any::downcast_ref)
        }
    }

    pub fn insert_mut<T: 'static>(&mut self, value: T) {
        let ty = TypeId::of::<T>();
        if ty == TypeId::of::<Endian>() {
            self.endian = *Any::downcast_ref(&value).unwrap();
        } else if ty == TypeId::of::<VecCount>() {
            self.count = Any::downcast_ref::<VecCount>(&value).map(|c| c.0);
        } else if ty == TypeId::of::<FileOffset>() {
            self.offset = Any::downcast_ref::<FileOffset>(&value).unwrap().0;
        } else {
            self.ext.insert_mut(ty, Box::new(value))
        }
    }

    #[must_use]
    pub fn insert<T: 'static>(&self, value: T) -> ReadOptions {
        let mut new = self.clone();

        new.insert_mut(value);

        new
    }

    pub fn remove_mut<T: 'static>(&mut self) -> bool {
        let ty = TypeId::of::<T>();
        if ty == TypeId::of::<Endian>() {
            self.endian = Endian::default();
            true
        } else if ty == TypeId::of::<VecCount>() {
            self.count = None;
            true
        } else if ty == TypeId::of::<FileOffset>() {
            self.offset = 0;
            true
        } else {
            self.ext.remove_mut(&ty)
        }
    }

    #[must_use]
    pub fn remove<T: 'static>(&self) -> ReadOptions {
        let mut new = self.clone();

        new.remove_mut::<T>();

        new
    }

    // this function in particular plays fast and loose with whether a option is "present"
    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        let ty = TypeId::of::<T>();
        if ty == TypeId::of::<Endian>() {
            true
        } else if ty == TypeId::of::<VecCount>() {
            self.count.is_some()
        } else if ty == TypeId::of::<FileOffset>() {
            self.offset != 0
        } else {
            self.ext.contains_key(&ty)
        }
    }
}

#[cfg(test)]
mod test {
    use super::ReadOptions;
    use super::{FileOffset, VecCount};
    use crate::Endian;

    #[test]
    fn read_existing() {
        let mut test = ReadOptions::default();
        for val in [Endian::Big, Endian::Little, Endian::Native].into_iter() {
            test.endian = *val;

            assert_eq!(val, test.get::<Endian>().unwrap());
        }

        for val in [Some(0), None, Some(0xff), Some(1337)].into_iter() {
            test.count = *val;

            assert_eq!(val.map(VecCount).as_ref(), test.get::<VecCount>());
        }

        for val in [0, 0xff, 1337].into_iter() {
            test.offset = *val;

            assert_eq!(Some(&FileOffset(*val)), test.get::<FileOffset>());
        }
    }

    #[test]
    fn write_existing() {
        let mut test = ReadOptions::default();
        for val in [Endian::Big, Endian::Little, Endian::Native].into_iter() {
            test.insert_mut(*val);

            assert_eq!(*val, test.endian);
        }

        assert_eq!(test.count, None);

        for val in [0, 0xff, 1337].into_iter() {
            test.insert_mut(VecCount(*val));

            assert_eq!(Some(*val), test.count);
        }

        for val in [0, 0xff, 1337].into_iter() {
            test.insert_mut(FileOffset(*val));

            assert_eq!(*val, test.offset);
        }
    }
}