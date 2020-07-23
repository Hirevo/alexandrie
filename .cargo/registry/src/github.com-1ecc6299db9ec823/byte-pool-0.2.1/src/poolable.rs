use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// The trait required to be able to use a type in `BytePool`.
pub trait Poolable {
    fn capacity(&self) -> usize;
    fn alloc(size: usize) -> Self;
}

impl<T: Default + Clone> Poolable for Vec<T> {
    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn alloc(size: usize) -> Self {
        vec![T::default(); size]
    }
}

impl<K, V, S> Poolable for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn alloc(size: usize) -> Self {
        HashMap::with_capacity_and_hasher(size, Default::default())
    }
}

/// A trait allowing for efficient reallocation.
pub trait Realloc {
    fn realloc(&mut self, new_size: usize);
}

impl<T: Default + Clone> Realloc for Vec<T> {
    fn realloc(&mut self, new_size: usize) {
        use std::cmp::Ordering::*;

        assert!(new_size > 0);
        match new_size.cmp(&self.capacity()) {
            Greater => self.resize(new_size, T::default()),
            Less => {
                self.truncate(new_size);
                self.shrink_to_fit();
            }
            Equal => {}
        }
    }
}

impl<K, V, S> Realloc for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn realloc(&mut self, new_size: usize) {
        use std::cmp::Ordering::*;

        assert!(new_size > 0);
        match new_size.cmp(&self.capacity()) {
            Greater => {
                let current = self.capacity();
                let diff = new_size - current;
                self.reserve(diff);
            }
            Less => {
                self.shrink_to_fit();
            }
            Equal => {}
        }
    }
}
