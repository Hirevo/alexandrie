use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use crossbeam_queue::SegQueue;
use stable_deref_trait::StableDeref;

use crate::poolable::{Poolable, Realloc};

/// A pool of byte slices, that reuses memory.
#[derive(Debug)]
pub struct BytePool<T = Vec<u8>>
where
    T: Poolable,
{
    list_large: SegQueue<T>,
    list_small: SegQueue<T>,
}

/// The size at which point values are allocated in the small list, rather
// than the big.
const SPLIT_SIZE: usize = 4 * 1024;

/// The value returned by an allocation of the pool.
/// When it is dropped the memory gets returned into the pool, and is not zeroed.
/// If that is a concern, you must clear the data yourself.
pub struct Block<'a, T: Poolable = Vec<u8>> {
    data: mem::ManuallyDrop<T>,
    pool: &'a BytePool<T>,
}

impl<T: Poolable + fmt::Debug> fmt::Debug for Block<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Block").field("data", &self.data).finish()
    }
}

impl<T: Poolable> Default for BytePool<T> {
    fn default() -> Self {
        BytePool::<T> {
            list_large: SegQueue::new(),
            list_small: SegQueue::new(),
        }
    }
}

impl<T: Poolable> BytePool<T> {
    /// Constructs a new pool.
    pub fn new() -> Self {
        BytePool::default()
    }

    /// Allocates a new `Block`, which represents a fixed sice byte slice.
    /// If `Block` is dropped, the memory is _not_ freed, but rather it is returned into the pool.
    /// The returned `Block` contains arbitrary data, and must be zeroed or overwritten,
    /// in cases this is needed.
    pub fn alloc(&self, size: usize) -> Block<'_, T> {
        assert!(size > 0, "Can not allocate empty blocks");

        // check the last 4 blocks
        let list = if size < SPLIT_SIZE {
            &self.list_small
        } else {
            &self.list_large
        };
        if let Ok(el) = list.pop() {
            if el.capacity() == size {
                // found one, reuse it
                return Block::new(el, self);
            } else {
                // put it back
                list.push(el);
            }
        }

        // allocate a new block
        let data = T::alloc(size);
        Block::new(data, self)
    }

    fn push_raw_block(&self, block: T) {
        if block.capacity() < SPLIT_SIZE {
            self.list_small.push(block);
        } else {
            self.list_large.push(block);
        }
    }
}

impl<'a, T: Poolable> Drop for Block<'a, T> {
    fn drop(&mut self) {
        let data = mem::ManuallyDrop::into_inner(unsafe { ptr::read(&self.data) });
        self.pool.push_raw_block(data);
    }
}

impl<'a, T: Poolable> Block<'a, T> {
    fn new(data: T, pool: &'a BytePool<T>) -> Self {
        Block {
            data: mem::ManuallyDrop::new(data),
            pool,
        }
    }

    /// Returns the amount of bytes this block has.
    pub fn size(&self) -> usize {
        self.data.capacity()
    }
}

impl<'a, T: Poolable + Realloc> Block<'a, T> {
    /// Resizes a block to a new size.
    pub fn realloc(&mut self, new_size: usize) {
        self.data.realloc(new_size);
    }
}

impl<'a, T: Poolable> Deref for Block<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<'a, T: Poolable> DerefMut for Block<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.deref_mut()
    }
}

// Safe because Block is just a wrapper around `T`.
unsafe impl<'a, T: StableDeref + Poolable> StableDeref for Block<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics_vec_u8() {
        let pool: BytePool<Vec<u8>> = BytePool::new();

        for i in 0..100 {
            let mut block_1k = pool.alloc(1 * 1024);
            let mut block_4k = pool.alloc(4 * 1024);

            for el in block_1k.deref_mut() {
                *el = i as u8;
            }

            for el in block_4k.deref_mut() {
                *el = i as u8;
            }

            for el in block_1k.deref() {
                assert_eq!(*el, i as u8);
            }

            for el in block_4k.deref() {
                assert_eq!(*el, i as u8);
            }
        }
    }

    #[test]
    fn realloc() {
        let pool: BytePool<Vec<u8>> = BytePool::new();

        let mut buf = pool.alloc(10);

        let _slice: &[u8] = &buf;

        assert_eq!(buf.capacity(), 10);
        for i in 0..10 {
            buf[i] = 1;
        }

        buf.realloc(512);
        assert_eq!(buf.capacity(), 512);
        for el in buf.iter().take(10) {
            assert_eq!(*el, 1);
        }

        buf.realloc(5);
        assert_eq!(buf.capacity(), 5);
        for el in buf.iter() {
            assert_eq!(*el, 1);
        }
    }

    #[test]
    fn multi_thread() {
        let pool = std::sync::Arc::new(BytePool::<Vec<u8>>::new());

        let pool1 = pool.clone();
        let h1 = std::thread::spawn(move || {
            for _ in 0..100 {
                let mut buf = pool1.alloc(64);
                buf[10] = 10;
            }
        });

        let pool2 = pool.clone();
        let h2 = std::thread::spawn(move || {
            for _ in 0..100 {
                let mut buf = pool2.alloc(64);
                buf[10] = 10;
            }
        });

        h1.join().unwrap();
        h2.join().unwrap();

        // two threads allocating in parallel will need 2 buffers
        assert!(pool.list_small.len() <= 2);
    }

    #[test]
    fn basics_vec_usize() {
        let pool: BytePool<Vec<usize>> = BytePool::new();

        for i in 0..100 {
            let mut block_1k = pool.alloc(1 * 1024);
            let mut block_4k = pool.alloc(4 * 1024);

            for el in block_1k.deref_mut() {
                *el = i;
            }

            for el in block_4k.deref_mut() {
                *el = i;
            }

            for el in block_1k.deref() {
                assert_eq!(*el, i);
            }

            for el in block_4k.deref() {
                assert_eq!(*el, i);
            }
        }
    }

    #[test]
    fn basics_hash_map() {
        use std::collections::HashMap;
        let pool: BytePool<HashMap<String, String>> = BytePool::new();

        let mut map = pool.alloc(4);
        for i in 0..4 {
            map.insert(format!("hello_{}", i), "world".into());
        }
        for i in 0..4 {
            assert_eq!(
                map.get(&format!("hello_{}", i)).unwrap(),
                &"world".to_string()
            );
        }
        drop(map);

        for i in 0..100 {
            let mut block_1k = pool.alloc(1 * 1024);
            let mut block_4k = pool.alloc(4 * 1024);

            for el in block_1k.deref_mut() {
                *el.1 = i.to_string();
            }

            for el in block_4k.deref_mut() {
                *el.1 = i.to_string();
            }

            for el in block_1k.deref() {
                assert_eq!(*el.0, i.to_string());
                assert_eq!(*el.1, i.to_string());
            }

            for el in block_4k.deref() {
                assert_eq!(*el.0, i.to_string());
                assert_eq!(*el.1, i.to_string());
            }
        }
    }
}
