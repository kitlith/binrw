use crate::Endian;
use rpds::HashTrieMap;
use std::any::{TypeId, Any};

pub trait OptionsCollection {
    #[must_use]
    fn get<T: 'static>(&self) -> Option<&T>;
    fn insert<T: 'static>(&mut self, value: T) -> bool;
    #[must_use]
    fn contains<T: 'static>(&self) -> bool;
}

impl OptionsCollection for () {
    fn get<T: 'static>(&self) -> Option<&T> {
        None
    }

    fn insert<T: 'static>(&mut self, _value: T) -> bool {
        false
    }

    fn contains<T: 'static>(&self) -> bool {
        false
    }
}

impl OptionsCollection for HashTrieMap<TypeId, Box<dyn Any>> {
    fn get<T: 'static>(&self) -> Option<&T> {
        self.get(&TypeId::of::<T>())
            .map(AsRef::as_ref)
            .and_then(Any::downcast_ref)
    }

    fn insert<T: 'static>(&mut self, value: T) -> bool {
        self.insert_mut(TypeId::of::<T>(), Box::new(value));
        true
    }

    fn contains<T: 'static>(&self) -> bool {
        self.contains_key(&TypeId::of::<T>())
    }
}

#[derive(Clone, Default)]
pub struct OptionsNode<V: 'static, R: OptionsCollection> {
    val: V,
    rest: R
}

impl<V: 'static, R: OptionsCollection> OptionsCollection for OptionsNode<V, R> {
    fn get<T: 'static>(&self) -> Option<&T> {
        Any::downcast_ref::<T>(&self.val).or_else(|| self.rest.get::<T>())
    }

    fn insert<T: 'static>(&mut self, value: T) -> bool {
        if let Some(val) = Any::downcast_mut(&mut self.val) {
            *val = value;
            true
        } else {
            self.rest.insert(value)
        }
    }

    fn contains<T: 'static>(&self) -> bool {
        if TypeId::of::<T>() == TypeId::of::<V>() {
            true
        } else {
            self.rest.contains::<T>()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct VecCount(pub Option<usize>);
#[derive(Debug, PartialEq, Clone, Default)]
pub struct FileOffset(pub u64);

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
type BasicReadOptions<Rest> = OptionsNode<Endian, OptionsNode<VecCount, OptionsNode<FileOffset, Rest>>>;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DontOutputTemplate(pub bool);
#[derive(Debug, PartialEq, Clone, Default)]
pub struct VariableName(pub Option<&'static str>);

#[cfg(feature = "debug_template")]
type TemplateReadOptions<Rest> = OptionsNode<DontOutputTemplate, OptionsNode<VariableName, Rest>>;

#[cfg(feature = "debug_template")]
pub type ReadOptions<Rest = ()> = BasicReadOptions<TemplateReadOptions<Rest>>;
#[cfg(not(feature = "debug_template"))]
pub type ReadOptions<Rest = ()> = BasicReadOptions<Rest>;

pub trait ReadOptionsExt<Rest> {
    fn endian(&self) -> Endian;
    fn count(&self) -> Option<usize>;
    fn offset(&self) -> u64;
    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self);
    #[cfg(feature = "debug_template")]
    fn variable_name(&self);
}

impl<Rest: OptionsCollection> ReadOptionsExt<Rest> for ReadOptions<Rest> {
    fn endian(&self) -> Endian {
        *self.get::<Endian>().unwrap()
    }

    fn count(&self) -> Option<usize> {
        self.get::<VecCount>().unwrap().0
    }

    fn offset(&self) -> u64 {
        self.get::<FileOffset>().unwrap().0
    }

    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self) -> bool {
        self.get::<DontOutputTemplate>().unwrap().0
    }

    #[cfg(feature = "debug_template")]
    fn variable_name(&self) -> Option<&'static str> {
        self.get::<VariableName>().unwrap().0
    }
}