#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use byte_pool::BytePool;

fn touch_obj(buf: &mut [u8], size: usize) {
    assert_eq!(buf.len(), size);
    buf[0] = 1;
    black_box(buf);
}

macro_rules! benches_for_size {
    ($size:expr, $name1:ident, $name2:ident, $name3:ident, $name4:ident, $name5:ident, $name6:ident) => {
        #[bench]
        fn $name1(b: &mut Bencher) {
            b.bytes = $size as u64;

            b.iter(|| {
                // alloc
                let mut buf = vec![0u8; $size];
                touch_obj(&mut buf, $size);
            });
        }

        #[bench]
        fn $name2(b: &mut Bencher) {
            b.bytes = $size as u64;
            let pool = BytePool::<Vec<u8>>::new();

            b.iter(|| {
                // alloc
                let mut buf = pool.alloc($size);
                touch_obj(&mut buf, $size);
            });
        }

        #[bench]
        fn $name3(b: &mut Bencher) {
            b.iter(|| run_vec(10, 1000, $size));
        }

        #[bench]
        fn $name4(b: &mut Bencher) {
            b.iter(|| run_vec(1, 1000, $size));
        }

        #[bench]
        fn $name5(b: &mut Bencher) {
            b.iter(|| run_pool(10, 1000, $size));
        }

        #[bench]
        fn $name6(b: &mut Bencher) {
            b.iter(|| run_pool(1, 1000, $size));
        }
    };
}

fn run_pool(thread: usize, iter: usize, size: usize) {
    use std::sync::Arc;
    let p = Arc::new(BytePool::<Vec<u8>>::new());
    let mut threads = Vec::new();

    for _ in 0..thread {
        let p = p.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..iter {
                let mut v = p.alloc(size);
                v[0] = 1;
                v[size / 4] = 1;
                v[size / 2] = 1;
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}

fn run_vec(thread: usize, iter: usize, size: usize) {
    let mut threads = Vec::new();

    for _ in 0..thread {
        threads.push(std::thread::spawn(move || {
            for _ in 0..iter {
                let mut v = vec![0u8; size];
                v[0] = 1;
                v[size / 4] = 1;
                v[size / 2] = 1;
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}

benches_for_size!(
    256,
    vec_256b,
    pool_256b,
    vec_256b_contention,
    vec_256b_no_contention,
    pool_256b_contention,
    pool_256b_no_contention
);
benches_for_size!(
    1 * 1024,
    vec_1k,
    pool_1k,
    vec_1k_contention,
    vec_1k_no_contention,
    pool_1k_contention,
    pool_1k_no_contention
);
benches_for_size!(
    4 * 1024,
    vec_4k,
    pool_4k,
    vec_4k_contention,
    vec_4k_no_contention,
    pool_4k_contention,
    pool_4k_no_contention
);
benches_for_size!(
    8 * 1024,
    vec_8k,
    pool_8k,
    vec_8k_contention,
    vec_8k_no_contention,
    pool_8k_contention,
    pool_8k_no_contention
);

benches_for_size!(
    1 * 1024 * 1024,
    m1_vec,
    m1_pool,
    m1_vec_contention,
    m1_vec_no_contention,
    m1_pool_contention,
    m1_pool_no_contention
);

#[bench]
fn base_line_vec_mixed(b: &mut Bencher) {
    let mut i = 0;

    b.iter(|| {
        // alternate between two sizes
        let size = if i % 2 == 0 { 1024 } else { 4096 };
        let mut buf = vec![0u8; size];
        touch_obj(&mut buf, size);

        i += 1;
    });
}

#[bench]
fn pool_mixed(b: &mut Bencher) {
    let mut i = 0;

    let pool = BytePool::<Vec<u8>>::new();

    b.iter(|| {
        // alternate between two sizes
        let size = if i % 2 == 0 { 1024 } else { 4096 };
        let mut buf = pool.alloc(size);
        touch_obj(&mut buf, size);

        i += 1;
    });
}

#[bench]
fn base_vec_grow(b: &mut Bencher) {
    let mut size = 16;

    b.iter(|| {
        let mut buf = vec![0u8; size];
        touch_obj(&mut buf, size);

        size = (size * 2).min(4 * 1024);
        buf.resize(size, 0);
        touch_obj(&mut buf, size);
    });
}

#[bench]
fn pool_grow(b: &mut Bencher) {
    let mut size = 16;
    let pool = BytePool::<Vec<u8>>::new();

    b.iter(|| {
        let mut buf = pool.alloc(size);
        touch_obj(&mut buf, size);

        size = (size * 2).min(4 * 1024);
        buf.realloc(size);
        touch_obj(&mut buf, size);
    });
}
