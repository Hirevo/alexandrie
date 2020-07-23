// Implementation is based on
// - https://github.com/hyperium/http/blob/master/src/extensions.rs
// - https://github.com/kardeiz/type-map/blob/master/src/lib.rs

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};

/// A type to store extra data inside `Request` and `Response`.
///
/// Store and retrieve values by
/// [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html). This allows
/// storing arbitrary data that implements `Sync + Send + 'static`. This is
/// useful when for example implementing middleware that needs to send values.
#[derive(Default)]
pub struct Extensions {
    map: Option<HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>>,
}

impl Extensions {
    /// Create an empty `Extensions`.
    #[inline]
    pub(crate) fn new() -> Self {
        Self { map: None }
    }

    /// Insert a value into this `Extensions`.
    ///
    /// If a value of this type already exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .get_or_insert_with(Default::default)
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| (boxed as Box<dyn Any>).downcast().ok().map(|boxed| *boxed))
    }

    /// Check if container contains value for type
    pub fn contains<T: 'static>(&self) -> bool {
        self.map
            .as_ref()
            .and_then(|m| m.get(&TypeId::of::<T>()))
            .is_some()
    }

    /// Get a reference to a value previously inserted on this `Extensions`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .as_ref()
            .and_then(|m| m.get(&TypeId::of::<T>()))
            .and_then(|boxed| (&**boxed as &(dyn Any)).downcast_ref())
    }

    /// Get a mutable reference to a value previously inserted on this `Extensions`.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .as_mut()
            .and_then(|m| m.get_mut(&TypeId::of::<T>()))
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any)).downcast_mut())
    }

    /// Remove a value from this `Extensions`.
    ///
    /// If a value of this type exists, it will be returned.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .as_mut()
            .and_then(|m| m.remove(&TypeId::of::<T>()))
            .and_then(|boxed| (boxed as Box<dyn Any>).downcast().ok().map(|boxed| *boxed))
    }

    /// Clear the `Extensions` of all inserted values.
    #[inline]
    pub fn clear(&mut self) {
        self.map = None;
    }
}

impl fmt::Debug for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extensions").finish()
    }
}

// With TypeIds as keys, there's no need to hash them. So we simply use an identy hasher.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extensions() {
        #[derive(Debug, PartialEq)]
        struct MyType(i32);

        let mut map = Extensions::new();

        map.insert(5i32);
        map.insert(MyType(10));

        assert_eq!(map.get(), Some(&5i32));
        assert_eq!(map.get_mut(), Some(&mut 5i32));

        assert_eq!(map.remove::<i32>(), Some(5i32));
        assert!(map.get::<i32>().is_none());

        assert_eq!(map.get::<bool>(), None);
        assert_eq!(map.get(), Some(&MyType(10)));
    }
}
