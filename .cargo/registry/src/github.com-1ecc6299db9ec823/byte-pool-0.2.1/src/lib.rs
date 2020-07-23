//! Pool of byte slices.
//!
//! # Example
//! ```rust
//! use byte_pool::BytePool;
//!
//! // Create a pool
//! let pool = BytePool::<Vec<u8>>::new();
//!
//! // Allocate a buffer with capacity 1024.
//! let mut buf = pool.alloc(1024);
//!
//! // write some data into it
//! for i in 0..100 {
//!   buf[i] = 12;
//! }
//!
//! // Check that we actually wrote sth.
//! assert_eq!(buf[55], 12);
//!
//! // Returns the underlying memory to the pool.
//! drop(buf);
//!
//! // Frees all memory in the pool.
//! drop(pool);
//! ```

mod pool;
mod poolable;

pub use pool::{Block, BytePool};
pub use poolable::{Poolable, Realloc};
