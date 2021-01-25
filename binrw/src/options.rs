use rpds::HashTrieMap;
use std::any::{TypeId, Any};

// NOTE: loosely based off of https://github.com/SergioBenitez/state and https://github.com/kardeiz/type-map
//  but setup using a persistent data structure instead

// TODO: no-op hasher since TypeId is already unique
// TODO: manually implement Debug, if relevant?
// TODO: some way of requiring inserted objects to implement certain traits?
#[derive(Clone, Debug, Default)]
pub struct Options(HashTrieMap<TypeId, Box<dyn Any>>);

#[allow(dead_code)]
impl Options {
    #[must_use]
    pub fn new() -> Options {
        Options(HashTrieMap::new())
    }

    #[must_use]
    pub fn new_with_degree(degree: u8) -> Options {
        Options(HashTrieMap::new_with_degree(degree))
    }

    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(AsRef::as_ref)
            .and_then(Any::downcast_ref)
    }

    #[must_use]
    pub fn insert<T: 'static>(&self, value: T) -> Options {
        Options(self.0.insert(TypeId::of::<T>(), Box::new(value)))
    }

    pub fn insert_mut<T: 'static>(&mut self, value: T) {
        self.0.insert_mut(TypeId::of::<T>(), Box::new(value))
    }

    #[must_use]
    pub fn remove<T: 'static>(&self) -> Options {
        Options(self.0.remove(&TypeId::of::<T>()))
    }

    #[must_use]
    pub fn remove_mut<T: 'static>(&mut self) -> bool {
        self.0.remove_mut(&TypeId::of::<T>())
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}