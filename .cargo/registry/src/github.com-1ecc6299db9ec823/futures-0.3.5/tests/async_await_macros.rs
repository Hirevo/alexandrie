#![recursion_limit="128"]

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn poll_and_pending() {
    use futures::{pending, pin_mut, poll};
    use futures::executor::block_on;
    use futures::task::Poll;

    let pending_once = async { pending!() };
    block_on(async {
        pin_mut!(pending_once);
        assert_eq!(Poll::Pending, poll!(&mut pending_once));
        assert_eq!(Poll::Ready(()), poll!(&mut pending_once));
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn join() {
    use futures::{pin_mut, poll, join};
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use futures::task::Poll;

    let (tx1, rx1) = oneshot::channel::<i32>();
    let (tx2, rx2) = oneshot::channel::<i32>();

    let fut = async {
        let res = join!(rx1, rx2);
        assert_eq!((Ok(1), Ok(2)), res);
    };

    block_on(async {
        pin_mut!(fut);
        assert_eq!(Poll::Pending, poll!(&mut fut));
        tx1.send(1).unwrap();
        assert_eq!(Poll::Pending, poll!(&mut fut));
        tx2.send(2).unwrap();
        assert_eq!(Poll::Ready(()), poll!(&mut fut));
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select() {
    use futures::select;
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    let (tx1, rx1) = oneshot::channel::<i32>();
    let (_tx2, rx2) = oneshot::channel::<i32>();
    tx1.send(1).unwrap();
    let mut ran = false;
    block_on(async {
        select! {
            res = rx1.fuse() => {
                assert_eq!(Ok(1), res);
                ran = true;
            },
            _ = rx2.fuse() => unreachable!(),
        }
    });
    assert!(ran);
}

#[cfg(all(feature = "alloc", feature = "executor", feature = "async-await"))]
#[test]
fn select_biased() {
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use futures::future::FutureExt;
    use futures::select_biased;

    let (tx1, rx1) = oneshot::channel::<i32>();
    let (_tx2, rx2) = oneshot::channel::<i32>();
    tx1.send(1).unwrap();
    let mut ran = false;
    block_on(async {
        select_biased! {
            res = rx1.fuse() => {
                assert_eq!(Ok(1), res);
                ran = true;
            },
            _ = rx2.fuse() => unreachable!(),
        }
    });
    assert!(ran);
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_streams() {
    use futures::select;
    use futures::channel::mpsc;
    use futures::executor::block_on;
    use futures::sink::SinkExt;
    use futures::stream::StreamExt;

    let (mut tx1, rx1) = mpsc::channel::<i32>(1);
    let (mut tx2, rx2) = mpsc::channel::<i32>(1);
    let mut rx1 = rx1.fuse();
    let mut rx2 = rx2.fuse();
    let mut ran = false;
    let mut total = 0;
    block_on(async {
        let mut tx1_opt;
        let mut tx2_opt;
        select! {
            _ = rx1.next() => panic!(),
            _ = rx2.next() => panic!(),
            default => {
                tx1.send(2).await.unwrap();
                tx2.send(3).await.unwrap();
                tx1_opt = Some(tx1);
                tx2_opt = Some(tx2);
            }
            complete => panic!(),
        }
        loop {
            select! {
                // runs first and again after default
                x = rx1.next() => if let Some(x) = x { total += x; },
                // runs second and again after default
                x = rx2.next()  => if let Some(x) = x { total += x; },
                // runs third
                default => {
                    assert_eq!(total, 5);
                    ran = true;
                    drop(tx1_opt.take().unwrap());
                    drop(tx2_opt.take().unwrap());
                },
                // runs last
                complete => break,
            };
        }
    });
    assert!(ran);
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_can_move_uncompleted_futures() {
    use futures::select;
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    let (tx1, rx1) = oneshot::channel::<i32>();
    let (tx2, rx2) = oneshot::channel::<i32>();
    tx1.send(1).unwrap();
    tx2.send(2).unwrap();
    let mut ran = false;
    let mut rx1 = rx1.fuse();
    let mut rx2 = rx2.fuse();
    block_on(async {
        select! {
            res = rx1 => {
                assert_eq!(Ok(1), res);
                assert_eq!(Ok(2), rx2.await);
                ran = true;
            },
            res = rx2 => {
                assert_eq!(Ok(2), res);
                assert_eq!(Ok(1), rx1.await);
                ran = true;
            },
        }
    });
    assert!(ran);
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_nested() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future;

    let mut outer_fut = future::ready(1);
    let mut inner_fut = future::ready(2);
    let res = block_on(async {
        select! {
            x = outer_fut => {
                select! {
                    y = inner_fut => x + y,
                }
            }
        }
    });
    assert_eq!(res, 3);
}

#[cfg(all(feature = "async-await", feature = "std"))]
#[test]
fn select_size() {
    use futures::select;
    use futures::future;

    let fut = async {
        let mut ready = future::ready(0i32);
        select! {
            _ = ready => {},
        }
    };
    assert_eq!(::std::mem::size_of_val(&fut), 24);

    let fut = async {
        let mut ready1 = future::ready(0i32);
        let mut ready2 = future::ready(0i32);
        select! {
            _ = ready1 => {},
            _ = ready2 => {},
        }
    };
    assert_eq!(::std::mem::size_of_val(&fut), 40);
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_on_non_unpin_expressions() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    // The returned Future is !Unpin
    let make_non_unpin_fut = || { async {
        5
    }};

    let res = block_on(async {
        let select_res;
        select! {
            value_1 = make_non_unpin_fut().fuse() => { select_res = value_1 },
            value_2 = make_non_unpin_fut().fuse() => { select_res = value_2 },
        };
        select_res
    });
    assert_eq!(res, 5);
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_on_non_unpin_expressions_with_default() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    // The returned Future is !Unpin
    let make_non_unpin_fut = || { async {
        5
    }};

    let res = block_on(async {
        let select_res;
        select! {
            value_1 = make_non_unpin_fut().fuse() => { select_res = value_1 },
            value_2 = make_non_unpin_fut().fuse() => { select_res = value_2 },
            default => { select_res = 7 },
        };
        select_res
    });
    assert_eq!(res, 5);
}

#[cfg(all(feature = "async-await", feature = "std"))]
#[test]
fn select_on_non_unpin_size() {
    use futures::select;
    use futures::future::FutureExt;

    // The returned Future is !Unpin
    let make_non_unpin_fut = || { async {
        5
    }};

    let fut = async {
        let select_res;
        select! {
            value_1 = make_non_unpin_fut().fuse() => { select_res = value_1 },
            value_2 = make_non_unpin_fut().fuse() => { select_res = value_2 },
        };
        select_res
    };

    assert_eq!(32, std::mem::size_of_val(&fut));
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_can_be_used_as_expression() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future;

    block_on(async {
        let res = select! {
            x = future::ready(7) => { x },
            y = future::ready(3) => { y + 1 },
        };
        assert!(res == 7 || res == 4);
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_with_default_can_be_used_as_expression() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future::{FutureExt, poll_fn};
    use futures::task::{Context, Poll};

    fn poll_always_pending<T>(_cx: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }

    block_on(async {
        let res = select! {
            x = poll_fn(poll_always_pending::<i32>).fuse() => x,
            y = poll_fn(poll_always_pending::<i32>).fuse() => { y + 1 },
            default => 99,
        };
        assert_eq!(res, 99);
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_with_complete_can_be_used_as_expression() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future;

    block_on(async {
        let res = select! {
            x = future::pending::<i32>() => { x },
            y = future::pending::<i32>() => { y + 1 },
            default => 99,
            complete => 237,
        };
        assert_eq!(res, 237);
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_on_mutable_borrowing_future_with_same_borrow_in_block() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    async fn require_mutable(_: &mut i32) {}
    async fn async_noop() {}

    block_on(async {
        let mut value = 234;
        select! {
            x = require_mutable(&mut value).fuse() => { },
            y = async_noop().fuse() => {
                value += 5;
            },
        }
    });
}

#[cfg(all(feature = "async-await", feature = "std", feature = "executor"))]
#[test]
fn select_on_mutable_borrowing_future_with_same_borrow_in_block_and_default() {
    use futures::select;
    use futures::executor::block_on;
    use futures::future::FutureExt;

    async fn require_mutable(_: &mut i32) {}
    async fn async_noop() {}

    block_on(async {
        let mut value = 234;
        select! {
            x = require_mutable(&mut value).fuse() => { },
            y = async_noop().fuse() => {
                value += 5;
            },
            default => {
                value += 27;
            },
        }
    });
}

#[cfg(feature = "async-await")]
#[test]
fn join_size() {
    use futures::join;
    use futures::future;

    let fut = async {
        let ready = future::ready(0i32);
        join!(ready)
    };
    assert_eq!(::std::mem::size_of_val(&fut), 16);

    let fut = async {
        let ready1 = future::ready(0i32);
        let ready2 = future::ready(0i32);
        join!(ready1, ready2)
    };
    assert_eq!(::std::mem::size_of_val(&fut), 28);
}

#[cfg(feature = "async-await")]
#[test]
fn try_join_size() {
    use futures::try_join;
    use futures::future;

    let fut = async {
        let ready = future::ready(Ok::<i32, i32>(0));
        try_join!(ready)
    };
    assert_eq!(::std::mem::size_of_val(&fut), 16);

    let fut = async {
        let ready1 = future::ready(Ok::<i32, i32>(0));
        let ready2 = future::ready(Ok::<i32, i32>(0));
        try_join!(ready1, ready2)
    };
    assert_eq!(::std::mem::size_of_val(&fut), 28);
}

#[cfg(feature = "async-await")]
#[test]
fn join_doesnt_require_unpin() {
    use futures::join;

    let _ = async {
        join!(async {}, async {})
    };
}

#[cfg(feature = "async-await")]
#[test]
fn try_join_doesnt_require_unpin() {
    use futures::try_join;

    let _ = async {
        try_join!(
            async { Ok::<(), ()>(()) },
            async { Ok::<(), ()>(()) },
        )
    };
}
