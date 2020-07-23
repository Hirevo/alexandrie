#[cfg(feature = "executor")]
#[test]
fn panic_in_the_middle_of_the_stream() {
    use futures::executor::block_on_stream;
    use futures::stream::{self, StreamExt};

    let stream = stream::iter(vec![Some(10), None, Some(11)]);

    // panic on second element
    let stream_panicking = stream.map(|o| o.unwrap());
    let mut iter = block_on_stream(stream_panicking.catch_unwind());

    assert_eq!(10, iter.next().unwrap().ok().unwrap());
    assert!(iter.next().unwrap().is_err());
    assert!(iter.next().is_none());
}

#[cfg(feature = "executor")]
#[test]
fn no_panic() {
    use futures::executor::block_on_stream;
    use futures::stream::{self, StreamExt};

    let stream = stream::iter(vec![10, 11, 12]);

    let mut iter = block_on_stream(stream.catch_unwind());

    assert_eq!(10, iter.next().unwrap().ok().unwrap());
    assert_eq!(11, iter.next().unwrap().ok().unwrap());
    assert_eq!(12, iter.next().unwrap().ok().unwrap());
    assert!(iter.next().is_none());
}
