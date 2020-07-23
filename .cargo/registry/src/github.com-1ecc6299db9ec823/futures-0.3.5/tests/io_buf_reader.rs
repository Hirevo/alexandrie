#[cfg(any(feature = "std", feature = "executor"))]
macro_rules! run_fill_buf {
    ($reader:expr) => {{
        use futures_test::task::noop_context;
        use futures::task::Poll;
        use std::pin::Pin;

        let mut cx = noop_context();
        loop {
            if let Poll::Ready(x) = Pin::new(&mut $reader).poll_fill_buf(&mut cx) {
                break x;
            }
        }
    }};
}

#[cfg(any(feature = "std", feature = "executor"))]
mod util {
    use futures::future::Future;
    pub fn run<F: Future + Unpin>(mut f: F) -> F::Output {
        use futures_test::task::noop_context;
        use futures::task::Poll;
        use futures::future::FutureExt;

        let mut cx = noop_context();
        loop {
            if let Poll::Ready(x) = f.poll_unpin(&mut cx) {
                return x;
            }
        }
    }
}

#[cfg(feature = "std")]
mod maybe_pending {
    use futures::task::{Context,Poll};
    use std::{cmp,io};
    use std::pin::Pin;
    use futures::io::{AsyncRead,AsyncBufRead};

    pub struct MaybePending<'a> {
        inner: &'a [u8],
        ready_read: bool,
        ready_fill_buf: bool,
    }

    impl<'a> MaybePending<'a> {
        pub fn new(inner: &'a [u8]) -> Self {
            Self { inner, ready_read: false, ready_fill_buf: false }
        }
    }

    impl AsyncRead for MaybePending<'_> {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
            -> Poll<io::Result<usize>>
        {
            if self.ready_read {
                self.ready_read = false;
                Pin::new(&mut self.inner).poll_read(cx, buf)
            } else {
                self.ready_read = true;
                Poll::Pending
            }
        }
    }

    impl AsyncBufRead for MaybePending<'_> {
        fn poll_fill_buf(mut self: Pin<&mut Self>, _: &mut Context<'_>)
            -> Poll<io::Result<&[u8]>>
        {
            if self.ready_fill_buf {
                self.ready_fill_buf = false;
                if self.inner.is_empty() { return Poll::Ready(Ok(&[])) }
                let len = cmp::min(2, self.inner.len());
                Poll::Ready(Ok(&self.inner[0..len]))
            } else {
                self.ready_fill_buf = true;
                Poll::Pending
            }
        }

        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            self.inner = &self.inner[amt..];
        }
    }
}

#[cfg(feature = "executor")]
#[test]
fn test_buffered_reader() {
    use futures::executor::block_on;
    use futures::io::{AsyncReadExt, BufReader};

    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, inner);

    let mut buf = [0, 0, 0];
    let nread = block_on(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 3);
    assert_eq!(buf, [5, 6, 7]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0, 0];
    let nread = block_on(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 2);
    assert_eq!(buf, [0, 1]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0];
    let nread = block_on(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [2]);
    assert_eq!(reader.buffer(), [3]);

    let mut buf = [0, 0, 0];
    let nread = block_on(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [3, 0, 0]);
    assert_eq!(reader.buffer(), []);

    let nread = block_on(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [4, 0, 0]);
    assert_eq!(reader.buffer(), []);

    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 0);
}

#[cfg(feature = "executor")]
#[test]
fn test_buffered_reader_seek() {
    use futures::executor::block_on;
    use futures::io::{AsyncSeekExt, AsyncBufRead, BufReader, Cursor, SeekFrom};
    use std::pin::Pin;
    use util::run;

    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, Cursor::new(inner));

    assert_eq!(block_on(reader.seek(SeekFrom::Start(3))).ok(), Some(3));
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[0, 1][..]));
    assert_eq!(run(reader.seek(SeekFrom::Current(i64::min_value()))).ok(), None);
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[0, 1][..]));
    assert_eq!(block_on(reader.seek(SeekFrom::Current(1))).ok(), Some(4));
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[1, 2][..]));
    Pin::new(&mut reader).consume(1);
    assert_eq!(block_on(reader.seek(SeekFrom::Current(-2))).ok(), Some(3));
}

#[cfg(feature = "executor")]
#[test]
fn test_buffered_reader_seek_underflow() {
    use futures::executor::block_on;
    use futures::io::{AsyncSeekExt, AsyncBufRead, AllowStdIo, BufReader, SeekFrom};
    use std::io;

    // gimmick reader that yields its position modulo 256 for each byte
    struct PositionReader {
        pos: u64
    }
    impl io::Read for PositionReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let len = buf.len();
            for x in buf {
                *x = self.pos as u8;
                self.pos = self.pos.wrapping_add(1);
            }
            Ok(len)
        }
    }
    impl io::Seek for PositionReader {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            match pos {
                SeekFrom::Start(n) => {
                    self.pos = n;
                }
                SeekFrom::Current(n) => {
                    self.pos = self.pos.wrapping_add(n as u64);
                }
                SeekFrom::End(n) => {
                    self.pos = u64::max_value().wrapping_add(n as u64);
                }
            }
            Ok(self.pos)
        }
    }

    let mut reader = BufReader::with_capacity(5, AllowStdIo::new(PositionReader { pos: 0 }));
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[0, 1, 2, 3, 4][..]));
    assert_eq!(block_on(reader.seek(SeekFrom::End(-5))).ok(), Some(u64::max_value()-5));
    assert_eq!(run_fill_buf!(reader).ok().map(|s| s.len()), Some(5));
    // the following seek will require two underlying seeks
    let expected = 9_223_372_036_854_775_802;
    assert_eq!(block_on(reader.seek(SeekFrom::Current(i64::min_value()))).ok(), Some(expected));
    assert_eq!(run_fill_buf!(reader).ok().map(|s| s.len()), Some(5));
    // seeking to 0 should empty the buffer.
    assert_eq!(block_on(reader.seek(SeekFrom::Current(0))).ok(), Some(expected));
    assert_eq!(reader.get_ref().get_ref().pos, expected);
}

#[cfg(feature = "executor")]
#[test]
fn test_short_reads() {
    use futures::executor::block_on;
    use futures::io::{AsyncReadExt, AllowStdIo, BufReader};
    use std::io;

    /// A dummy reader intended at testing short-reads propagation.
    struct ShortReader {
        lengths: Vec<usize>,
    }

    impl io::Read for ShortReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            if self.lengths.is_empty() {
                Ok(0)
            } else {
                Ok(self.lengths.remove(0))
            }
        }
    }

    let inner = ShortReader { lengths: vec![0, 1, 2, 0, 1, 0] };
    let mut reader = BufReader::new(AllowStdIo::new(inner));
    let mut buf = [0, 0];
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 0);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 1);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 2);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 0);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 1);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 0);
    assert_eq!(block_on(reader.read(&mut buf)).unwrap(), 0);
}

#[cfg(feature = "std")]
#[test]
fn maybe_pending() {
    use futures::io::{AsyncReadExt, BufReader};
    use util::run;
    use maybe_pending::MaybePending;

    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, MaybePending::new(inner));

    let mut buf = [0, 0, 0];
    let nread = run(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 3);
    assert_eq!(buf, [5, 6, 7]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0, 0];
    let nread = run(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 2);
    assert_eq!(buf, [0, 1]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0];
    let nread = run(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [2]);
    assert_eq!(reader.buffer(), [3]);

    let mut buf = [0, 0, 0];
    let nread = run(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [3, 0, 0]);
    assert_eq!(reader.buffer(), []);

    let nread = run(reader.read(&mut buf));
    assert_eq!(nread.unwrap(), 1);
    assert_eq!(buf, [4, 0, 0]);
    assert_eq!(reader.buffer(), []);

    assert_eq!(run(reader.read(&mut buf)).unwrap(), 0);
}

#[cfg(feature = "std")]
#[test]
fn maybe_pending_buf_read() {
    use futures::io::{AsyncBufReadExt, BufReader};
    use util::run;
    use maybe_pending::MaybePending;

    let inner = MaybePending::new(&[0, 1, 2, 3, 1, 0]);
    let mut reader = BufReader::with_capacity(2, inner);
    let mut v = Vec::new();
    run(reader.read_until(3, &mut v)).unwrap();
    assert_eq!(v, [0, 1, 2, 3]);
    v.clear();
    run(reader.read_until(1, &mut v)).unwrap();
    assert_eq!(v, [1]);
    v.clear();
    run(reader.read_until(8, &mut v)).unwrap();
    assert_eq!(v, [0]);
    v.clear();
    run(reader.read_until(9, &mut v)).unwrap();
    assert_eq!(v, []);
}

// https://github.com/rust-lang/futures-rs/pull/1573#discussion_r281162309
#[cfg(feature = "std")]
#[test]
fn maybe_pending_seek() {
    use futures::io::{AsyncBufRead, AsyncSeek, AsyncSeekExt, AsyncRead, BufReader,
        Cursor, SeekFrom
    };
    use futures::task::{Context,Poll};
    use std::io;
    use std::pin::Pin;
    use util::run;
    pub struct MaybePendingSeek<'a> {
        inner: Cursor<&'a [u8]>,
        ready: bool,
    }

    impl<'a> MaybePendingSeek<'a> {
        pub fn new(inner: &'a [u8]) -> Self {
            Self { inner: Cursor::new(inner), ready: true }
        }
    }

    impl AsyncRead for MaybePendingSeek<'_> {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
            -> Poll<io::Result<usize>>
        {
            Pin::new(&mut self.inner).poll_read(cx, buf)
        }
    }

    impl AsyncBufRead for MaybePendingSeek<'_> {
        fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
            -> Poll<io::Result<&[u8]>>
        {
            let this: *mut Self = &mut *self as *mut _;
            Pin::new(&mut unsafe { &mut *this }.inner).poll_fill_buf(cx)
        }

        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            Pin::new(&mut self.inner).consume(amt)
        }
    }

    impl AsyncSeek for MaybePendingSeek<'_> {
        fn poll_seek(mut self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom)
            -> Poll<io::Result<u64>>
        {
            if self.ready {
                self.ready = false;
                Pin::new(&mut self.inner).poll_seek(cx, pos)
            } else {
                self.ready = true;
                Poll::Pending
            }
        }
    }

    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, MaybePendingSeek::new(inner));

    assert_eq!(run(reader.seek(SeekFrom::Current(3))).ok(), Some(3));
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[0, 1][..]));
    assert_eq!(run(reader.seek(SeekFrom::Current(i64::min_value()))).ok(), None);
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[0, 1][..]));
    assert_eq!(run(reader.seek(SeekFrom::Current(1))).ok(), Some(4));
    assert_eq!(run_fill_buf!(reader).ok(), Some(&[1, 2][..]));
    Pin::new(&mut reader).consume(1);
    assert_eq!(run(reader.seek(SeekFrom::Current(-2))).ok(), Some(3));
}
