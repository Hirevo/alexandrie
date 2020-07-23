// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub struct Thunk<'a, A = (), R = ()> {
    invoke: Box<dyn Invoke<A, R> + Send + 'a>,
}

impl<'a, R> Thunk<'a, (), R> {
    pub fn new<F>(func: F) -> Thunk<'a, (), R>
    where
        F: FnOnce() -> R + Send + 'a,
    {
        Thunk::with_arg(move |()| func())
    }
}

impl<'a, A, R> Thunk<'a, A, R> {
    pub fn with_arg<F>(func: F) -> Thunk<'a, A, R>
    where
        F: FnOnce(A) -> R + Send + 'a,
    {
        Thunk {
            invoke: Box::<F>::new(func),
        }
    }

    pub fn invoke(self, arg: A) -> R {
        self.invoke.invoke(arg)
    }
}

#[doc(hidden)]
pub trait Invoke<A = (), R = ()> {
    fn invoke(self: Box<Self>, arg: A) -> R;
}

impl<A, R, F> Invoke<A, R> for F
where
    F: FnOnce(A) -> R,
{
    fn invoke(self: Box<F>, arg: A) -> R {
        let f = *self;
        f(arg)
    }
}
