//! A library to help grow the stack when it runs out of space.
//!
//! This is an implementation of manually instrumented segmented stacks where points in a program's
//! control flow are annotated with "maybe grow the stack here". Each point of annotation indicates
//! how far away from the end of the stack it's allowed to be, plus the amount of stack to allocate
//! if it does reach the end.
//!
//! Once a program has reached the end of its stack, a temporary stack on the heap is allocated and
//! is switched to for the duration of a closure.
//!
//! For a set of lower-level primitives, consider the `psm` crate.
//!
//! # Examples
//!
//! ```
//! // Grow the stack if we are within the "red zone" of 32K, and if we allocate
//! // a new stack allocate 1MB of stack space.
//! //
//! // If we're already in bounds, just run the provided closure on current stack.
//! stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
//!     // guaranteed to have at least 32K of stack
//! });
//! ```

#![allow(improper_ctypes)]

#[macro_use]
extern crate cfg_if;
extern crate libc;
#[cfg(windows)]
extern crate winapi;
#[macro_use]
extern crate psm;

use std::cell::Cell;

/// Grows the call stack if necessary.
///
/// This function is intended to be called at manually instrumented points in a program where
/// recursion is known to happen quite a bit. This function will check to see if we're within
/// `red_zone` bytes of the end of the stack, and if so it will allocate a new stack of at least
/// `stack_size` bytes.
///
/// The closure `f` is guaranteed to run on a stack with at least `red_zone` bytes, and it will be
/// run on the current stack if there's space available.
#[inline(always)]
pub fn maybe_grow<R, F: FnOnce() -> R>(red_zone: usize, stack_size: usize, callback: F) -> R {
    // if we can't guess the remaining stack (unsupported on some platforms) we immediately grow
    // the stack and then cache the new stack size (which we do know now because we allocated it.
    let enough_space = remaining_stack().map_or(false, |remaining| remaining >= red_zone);
    if enough_space {
        callback()
    } else {
        grow(stack_size, callback)
    }
}

/// Always creates a new stack for the passed closure to run on.
/// The closure will still be on the same thread as the caller of `grow`.
/// This will allocate a new stack with at least `stack_size` bytes.
pub fn grow<R, F: FnOnce() -> R>(stack_size: usize, callback: F) -> R {
    let mut ret = None;
    let ret_ref = &mut ret;
    _grow(stack_size, move || {
        *ret_ref = Some(callback());
    });
    ret.unwrap()
}

/// Queries the amount of remaining stack as interpreted by this library.
///
/// This function will return the amount of stack space left which will be used
/// to determine whether a stack switch should be made or not.
pub fn remaining_stack() -> Option<usize> {
    let current_ptr = current_stack_ptr();
    get_stack_limit().map(|limit| current_ptr - limit)
}

psm_stack_information! (
    yes {
        fn current_stack_ptr() -> usize {
            psm::stack_pointer() as usize
        }
    }
    no {
        #[inline(always)]
        fn current_stack_ptr() -> usize {
            unsafe {
                let mut x = std::mem::MaybeUninit::<u8>::uninit();
                // Unlikely to be ever exercised. As a fallback we execute a volatile read to a
                // local (to hopefully defeat the optimisations that would make this local a static
                // global) and take its address. This way we get a very approximate address of the
                // current frame.
                x.as_mut_ptr().write_volatile(42);
                x.as_ptr() as usize
            }
        }
    }
);

thread_local! {
    static STACK_LIMIT: Cell<Option<usize>> = Cell::new(unsafe {
        guess_os_stack_limit()
    })
}

#[inline(always)]
fn get_stack_limit() -> Option<usize> {
    STACK_LIMIT.with(|s| s.get())
}

#[inline(always)]
#[allow(unused)]
fn set_stack_limit(l: Option<usize>) {
    STACK_LIMIT.with(|s| s.set(l))
}

psm_stack_manipulation! {
    yes {
        struct StackRestoreGuard {
            new_stack: *mut libc::c_void,
            stack_bytes: usize,
            old_stack_limit: Option<usize>,
        }

        impl Drop for StackRestoreGuard {
            fn drop(&mut self) {
                unsafe {
                    // FIXME: check the error code and decide what to do with it.
                    // Perhaps a debug_assertion?
                    libc::munmap(self.new_stack, self.stack_bytes);
                }
                set_stack_limit(self.old_stack_limit);
            }
        }

        fn _grow<F: FnOnce()>(stack_size: usize, callback: F) {
            // Calculate a number of pages we want to allocate for the new stack.
            // For maximum portability we want to produce a stack that is aligned to a page and has
            // a size that’s a multiple of page size. Furthermore we want to allocate two extras pages
            // for the stack guard. To achieve that we do our calculations in number of pages and
            // convert to bytes last.
            // FIXME: consider caching the page size.
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) } as usize;
            let requested_pages = stack_size
                .checked_add(page_size - 1)
                .expect("unreasonably large stack requested") / page_size;
            let stack_pages = std::cmp::max(1, requested_pages) + 2;
            let stack_bytes = stack_pages.checked_mul(page_size)
                .expect("unreasonably large stack requesteed");

            // Next, there are a couple of approaches to how we allocate the new stack. We take the
            // most obvious path and use `mmap`. We also `mprotect` a guard page into our
            // allocation.
            //
            // We use a guard pattern to ensure we deallocate the allocated stack when we leave
            // this function and also try to uphold various safety invariants required by `psm`
            // (such as not unwinding from the callback we pass to it).
            //
            // Other than that this code has no meaningful gotchas.
            unsafe {
                let new_stack = libc::mmap(
                    std::ptr::null_mut(),
                    stack_bytes,
                    libc::PROT_NONE,
                    libc::MAP_PRIVATE |
                    libc::MAP_ANON,
                    -1, // Some implementations assert fd = -1 if MAP_ANON is specified
                    0
                );
                if new_stack == libc::MAP_FAILED {
                    panic!("unable to allocate stack")
                }
                let guard = StackRestoreGuard {
                    new_stack,
                    stack_bytes,
                    old_stack_limit: get_stack_limit(),
                };
                let above_guard_page = new_stack.add(page_size);
                #[cfg(not(target_os = "openbsd"))]
                let result = libc::mprotect(
                    above_guard_page,
                    stack_bytes - page_size,
                    libc::PROT_READ | libc::PROT_WRITE
                );
                #[cfg(target_os = "openbsd")]
                let result = if libc::mmap(
                        above_guard_page,
                        stack_bytes - page_size,
                        libc::PROT_READ | libc::PROT_WRITE,
                        libc::MAP_FIXED | libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_STACK,
                        -1,
                        0) == above_guard_page {
                    0
                } else {
                    -1
                };
                if result == -1 {
                    drop(guard);
                    panic!("unable to set stack permissions")
                }
                set_stack_limit(Some(above_guard_page as usize));
                let panic = psm::on_stack(above_guard_page as *mut _, stack_size, move || {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).err()
                });
                drop(guard);
                if let Some(p) = panic {
                    std::panic::resume_unwind(p);
                }
            }
        }
    }

    no {
        #[cfg(not(windows))]
        fn _grow<F: FnOnce()>(stack_size: usize, callback: F) {
            drop(stack_size);
            callback();
        }
    }
}

cfg_if! {
    if #[cfg(windows)] {
        use std::ptr;
        use std::io;

        use winapi::shared::basetsd::*;
        use winapi::shared::minwindef::{LPVOID, BOOL};
        use winapi::shared::ntdef::*;
        use winapi::um::fibersapi::*;
        use winapi::um::memoryapi::*;
        use winapi::um::processthreadsapi::*;
        use winapi::um::winbase::*;

        // Make sure the libstacker.a (implemented in C) is linked.
        // See https://github.com/rust-lang/rust/issues/65610
        #[link(name="stacker")]
        extern {
            fn __stacker_get_current_fiber() -> PVOID;
        }

        struct FiberInfo<F> {
            callback: std::mem::MaybeUninit<F>,
            panic: Option<Box<dyn std::any::Any + Send + 'static>>,
            parent_fiber: LPVOID,
        }

        unsafe extern "system" fn fiber_proc<F: FnOnce()>(data: LPVOID) {
            // This function is the entry point to our inner fiber, and as argument we get an
            // instance of `FiberInfo`. We will set-up the "runtime" for the callback and execute
            // it.
            let data = &mut *(data as *mut FiberInfo<F>);
            let old_stack_limit = get_stack_limit();
            set_stack_limit(guess_os_stack_limit());
            let callback = data.callback.as_ptr();
            data.panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback.read())).err();

            // Restore to the previous Fiber
            set_stack_limit(old_stack_limit);
            SwitchToFiber(data.parent_fiber);
            return;
        }

        fn _grow<F: FnOnce()>(stack_size: usize, callback: F) {
            // Fibers (or stackful coroutines) is the only official way to create new stacks on the
            // same thread on Windows. So in order to extend the stack we create fiber and switch
            // to it so we can use it's stack. After running `callback` within our fiber, we switch
            // back to the current stack and destroy the fiber and its associated stack.
            unsafe {
                let was_fiber = IsThreadAFiber() == TRUE as BOOL;
                let mut data = FiberInfo {
                    callback: std::mem::MaybeUninit::new(callback),
                    panic: None,
                    parent_fiber: {
                        if was_fiber {
                            // Get a handle to the current fiber. We need to use a C implementation
                            // for this as GetCurrentFiber is an header only function.
                            __stacker_get_current_fiber()
                        } else {
                            // Convert the current thread to a fiber, so we are able to switch back
                            // to the current stack. Threads coverted to fibers still act like
                            // regular threads, but they have associated fiber data. We later
                            // convert it back to a regular thread and free the fiber data.
                            ConvertThreadToFiber(ptr::null_mut())
                        }
                    },
                };

                if data.parent_fiber.is_null() {
                    panic!("unable to convert thread to fiber: {}", io::Error::last_os_error());
                }

                let fiber = CreateFiber(
                    stack_size as SIZE_T,
                    Some(fiber_proc::<F>),
                    &mut data as *mut FiberInfo<F> as *mut _,
                );
                if fiber.is_null() {
                    panic!("unable to allocate fiber: {}", io::Error::last_os_error());
                }

                // Switch to the fiber we created. This changes stacks and starts executing
                // fiber_proc on it. fiber_proc will run `callback` and then switch back to run the
                // next statement.
                SwitchToFiber(fiber);
                DeleteFiber(fiber);

                // Clean-up.
                if !was_fiber {
                    if ConvertFiberToThread() == 0 {
                        // FIXME: Perhaps should not panic here?
                        panic!("unable to convert back to thread: {}", io::Error::last_os_error());
                    }
                }
                if let Some(p) = data.panic {
                    std::panic::resume_unwind(p);
                }
            }
        }

        #[inline(always)]
        fn get_thread_stack_guarantee() -> usize {
            let min_guarantee = if cfg!(target_pointer_width = "32") {
                0x1000
            } else {
                0x2000
            };
            let mut stack_guarantee = 0;
            unsafe {
                // Read the current thread stack guarantee
                // This is the stack reserved for stack overflow
                // exception handling.
                // This doesn't return the true value so we need
                // some further logic to calculate the real stack
                // guarantee. This logic is what is used on x86-32 and
                // x86-64 Windows 10. Other versions and platforms may differ
                SetThreadStackGuarantee(&mut stack_guarantee)
            };
            std::cmp::max(stack_guarantee, min_guarantee) as usize + 0x1000
        }

        #[inline(always)]
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            // Query the allocation which contains our stack pointer in order
            // to discover the size of the stack
            //
            // FIXME: we could read stack base from the TIB, specifically the 3rd element of it.
            type QueryT = winapi::um::winnt::MEMORY_BASIC_INFORMATION;
            let mut mi = std::mem::MaybeUninit::<QueryT>::uninit();
            VirtualQuery(
                psm::stack_pointer() as *const _,
                mi.as_mut_ptr(),
                std::mem::size_of::<QueryT>() as SIZE_T,
            );
            Some(mi.assume_init().AllocationBase as usize + get_thread_stack_guarantee() + 0x1000)
        }
    } else if #[cfg(any(target_os = "linux", target_os="solaris", target_os = "netbsd"))] {
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
            assert_eq!(libc::pthread_attr_init(attr.as_mut_ptr()), 0);
            assert_eq!(libc::pthread_getattr_np(libc::pthread_self(),
                                                attr.as_mut_ptr()), 0);
            let mut stackaddr = std::ptr::null_mut();
            let mut stacksize = 0;
            assert_eq!(libc::pthread_attr_getstack(
                attr.as_ptr(), &mut stackaddr, &mut stacksize
            ), 0);
            assert_eq!(libc::pthread_attr_destroy(attr.as_mut_ptr()), 0);
            Some(stackaddr as usize)
        }
    } else if #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))] {
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
            assert_eq!(libc::pthread_attr_init(attr.as_mut_ptr()), 0);
            assert_eq!(libc::pthread_attr_get_np(libc::pthread_self(), attr.as_mut_ptr()), 0);
            let mut stackaddr = std::ptr::null_mut();
            let mut stacksize = 0;
            assert_eq!(libc::pthread_attr_getstack(
                attr.as_ptr(), &mut stackaddr, &mut stacksize
            ), 0);
            assert_eq!(libc::pthread_attr_destroy(attr.as_mut_ptr()), 0);
            Some(stackaddr as usize)
        }
    } else if #[cfg(target_os = "openbsd")] {
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            let mut stackinfo = std::mem::MaybeUninit::<libc::stack_t>::uninit();
            assert_eq!(libc::pthread_stackseg_np(libc::pthread_self(), stackinfo.as_mut_ptr()), 0);
            Some(stackinfo.assume_init().ss_sp as usize - stackinfo.assume_init().ss_size)
        }
    } else if #[cfg(target_os = "macos")] {
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            Some(libc::pthread_get_stackaddr_np(libc::pthread_self()) as usize -
                libc::pthread_get_stacksize_np(libc::pthread_self()) as usize)
        }
    } else {
        // fallback for other platforms is to always increase the stack if we're on
        // the root stack. After we increased the stack once, we know the new stack
        // size and don't need this pessimization anymore
        #[inline(always)]
        unsafe fn guess_os_stack_limit() -> Option<usize> {
            None
        }
    }
}
