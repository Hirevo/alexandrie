use core::any::Any;
use core::pin::Pin;
use std::panic::{catch_unwind, UnwindSafe, AssertUnwindSafe};

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use pin_project::pin_project;

/// Future for the [`catch_unwind`](super::FutureExt::catch_unwind) method.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct CatchUnwind<Fut>(#[pin] Fut);

impl<Fut> CatchUnwind<Fut> where Fut: Future + UnwindSafe {
    pub(super) fn new(future: Fut) -> CatchUnwind<Fut> {
        CatchUnwind(future)
    }
}

impl<Fut> Future for CatchUnwind<Fut>
    where Fut: Future + UnwindSafe,
{
    type Output = Result<Fut::Output, Box<dyn Any + Send>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let f = self.project().0;
        catch_unwind(AssertUnwindSafe(|| f.poll(cx)))?.map(Ok)
    }
}
