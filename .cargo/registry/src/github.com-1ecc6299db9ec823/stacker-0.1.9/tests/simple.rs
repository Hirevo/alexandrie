extern crate stacker;

const RED_ZONE: usize = 100 * 1024; // 100k
const STACK_PER_RECURSION: usize = 1 * 1024 * 1024; // 1MB

pub fn ensure_sufficient_stack<R, F: FnOnce() -> R + std::panic::UnwindSafe>(f: F) -> R {
    stacker::maybe_grow(RED_ZONE, STACK_PER_RECURSION, f)
}

#[inline(never)]
fn recurse(n: usize) {
    let x = [42u8; 50000];
    if n != 0 {
        ensure_sufficient_stack(|| recurse(n - 1));
    }
    drop(x);
}

#[test]
fn foo() {
    recurse(10000);
}
