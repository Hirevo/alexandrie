use bitflags::bitflags;
use std::io;
use std::ops::Drop;
use std::os::raw::c_int;
use std::os::windows::io::{AsRawHandle, AsRawSocket, RawHandle};
use std::ptr;
use std::time::Duration;
use wepoll_sys as sys;

/// An enum for the various `EPOLL_CTL_*` constants.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Operation {
    /// An socket to monitor should be added.
    Add = sys::EPOLL_CTL_ADD,

    /// An socket that is being monitored should be removed.
    Delete = sys::EPOLL_CTL_DEL,

    /// An socket that is being monitored should be modified.
    Modify = sys::EPOLL_CTL_MOD,
}

bitflags! {
    /// An enum for the various `EPOLL*` flags, such as `EPOLLOUT`.`
    ///
    /// The following events are not supported by wepoll:
    ///
    /// * EPOLLET
    /// * EPOLLEXCLUSIVE
    /// * EPOLLWAKEUP
    pub struct EventFlag: u32 {
        const ERR = sys::EPOLLERR;
        const HUP = sys::EPOLLHUP;
        const IN = sys::EPOLLIN;
        const MSG = sys::EPOLLMSG;
        const ONESHOT = sys::EPOLLONESHOT;
        const OUT = sys::EPOLLOUT;
        const PRI = sys::EPOLLPRI;
        const RDBAND = sys::EPOLLRDBAND;
        const RDHUP = sys::EPOLLRDHUP;
        const RDNORM = sys::EPOLLRDNORM;
        const WRBAND = sys::EPOLLWRBAND;
        const WRNORM = sys::EPOLLWRNORM;
    }
}

/// A single epoll event.
pub struct Event {
    raw: sys::epoll_event,
}

impl Event {
    /// Creates a new epoll event.
    pub fn new(flags: EventFlag, data: u64) -> Self {
        Event {
            raw: sys::epoll_event {
                events: flags.bits(),
                data: sys::epoll_data { u64: data },
            },
        }
    }

    /// Returns the flags of this event.
    pub fn flags(&self) -> EventFlag {
        EventFlag::from_bits_truncate(self.raw.events)
    }

    /// Returns the user data that is associated with this event.
    pub fn data(&self) -> u64 {
        unsafe { self.raw.data.u64 }
    }
}

/// A collection of events produced by wepoll.
pub struct Events {
    raw: Vec<sys::epoll_event>,
}

impl Events {
    /// Creates an `Events` collection with enough space for `amount` events.
    pub fn with_capacity(amount: usize) -> Self {
        Events {
            raw: Vec::with_capacity(amount),
        }
    }

    /// Returns the amount of events in `self`.
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns the maximum amount of events that can be stored in `self`.
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Returns an iterator over the events in `self`.
    pub fn iter(&self) -> Iter {
        Iter {
            events: &self,
            index: 0,
        }
    }

    /// Clears all events by setting the internal length to 0.
    ///
    /// The associated memory won't be dropped until it is either overwritten,
    /// or `self` is dropped.
    pub fn clear(&mut self) {
        unsafe { self.raw.set_len(0) };
    }
}

unsafe impl Sync for Events {}
unsafe impl Send for Events {}

/// An iterator over epoll events.
pub struct Iter<'a> {
    events: &'a Events,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.index == self.events.len() {
            return None;
        }

        let ev = &self.events.raw[self.index];
        let event =
            Event::new(EventFlag::from_bits_truncate(ev.events), unsafe {
                ev.data.u64
            });

        self.index += 1;

        Some(event)
    }
}

/// An epoll instance.
///
/// The Epoll type acts as a wrapper around wepoll's `HANDLE` type, and
/// automatically closed it upon being dropped.
///
/// Whereas epoll on Linux supports arbitrary file descriptors, wepoll (and thus
/// this wrapper) only supports Windows sockets.
pub struct Epoll {
    handle: sys::HANDLE,
}

impl Epoll {
    /// Creates a new `Epoll`.
    ///
    /// Flags and/or a size can not be provided, as wepoll does not support
    /// either.
    ///
    /// # Examples
    ///
    /// ```
    /// use wepoll_binding::Epoll;
    ///
    /// let epoll = Epoll::new().expect("Failed to create a new Epoll");
    /// ```
    pub fn new() -> io::Result<Epoll> {
        let handle = unsafe { sys::epoll_create(1) };

        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        Ok(Epoll { handle })
    }

    /// Waits for events to be produced.
    ///
    /// `poll` blocks the current thread until one or more events are produced,
    /// at which point they will be stored in the `events` slice.
    ///
    /// If a timeout is given, this method will return when it expires. When
    /// this happens, the number of produced events may be less than the
    /// capacity of the `Events` type.
    ///
    /// Timeouts internally use a resolution of 1 millisecond, so a timeout
    /// smaller than this value may be rounded up.
    ///
    /// When the timeout is 0, this method won't block and report any events
    /// that are already waiting to be processed.
    pub fn poll(
        &self,
        events: &mut Events,
        timeout: Option<Duration>,
    ) -> io::Result<usize> {
        let timeout_ms = if let Some(duration) = timeout {
            duration.as_millis() as c_int
        } else {
            -1
        };

        let received = unsafe {
            sys::epoll_wait(
                self.handle,
                events.raw.as_mut_ptr(),
                events.capacity() as c_int,
                timeout_ms,
            )
        };

        if received == -1 {
            return Err(io::Error::last_os_error());
        }

        unsafe { events.raw.set_len(received as usize) };

        Ok(received as usize)
    }

    /// Registers a raw socket with `self`.
    ///
    /// Registering an already registered socket will produce an error. If you
    /// want to update an existing registration, use `Epoll::reregister()`
    /// instead.
    ///
    /// The `data` argument will be included in any events produced by the
    /// registration.
    ///
    /// # Examples
    ///
    /// ```
    /// use wepoll_binding::{Epoll, EventFlag};
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let poll = Epoll::new().unwrap();
    ///
    /// poll
    ///   .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
    ///   .unwrap();
    /// ```
    pub fn register<T: AsRawSocket>(
        &self,
        socket: &T,
        flags: EventFlag,
        data: u64,
    ) -> io::Result<()> {
        self.register_or_reregister(socket, flags, data, Operation::Add)
    }

    /// Re-registers a raw socket with `self`.
    ///
    /// Attempting to re-register a socket that is not registered will result in
    /// an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use wepoll_binding::{Epoll, EventFlag};
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let poll = Epoll::new().unwrap();
    ///
    /// poll
    ///   .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
    ///   .unwrap();
    ///
    /// poll
    ///   .reregister(&socket, EventFlag::IN | EventFlag::ONESHOT, 42)
    ///   .unwrap();
    /// ```
    pub fn reregister<T: AsRawSocket>(
        &self,
        socket: &T,
        flags: EventFlag,
        data: u64,
    ) -> io::Result<()> {
        self.register_or_reregister(socket, flags, data, Operation::Modify)
    }

    /// Deregisters a raw socket from `self`.
    ///
    /// Attempting to deregister an unregistered socket will produce an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use wepoll_binding::{Epoll, EventFlag};
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let poll = Epoll::new().unwrap();
    ///
    /// poll
    ///   .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
    ///   .unwrap();
    ///
    /// poll.deregister(&socket).unwrap();
    /// ```
    pub fn deregister<T: AsRawSocket>(&self, socket: &T) -> io::Result<()> {
        let result = unsafe {
            sys::epoll_ctl(
                self.handle,
                Operation::Delete as c_int,
                socket.as_raw_socket() as sys::SOCKET,
                ptr::null_mut(),
            )
        };

        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    fn register_or_reregister<T: AsRawSocket>(
        &self,
        socket: &T,
        flags: EventFlag,
        data: u64,
        operation: Operation,
    ) -> io::Result<()> {
        let mut event = Event::new(flags, data);

        let result = unsafe {
            sys::epoll_ctl(
                self.handle,
                operation as c_int,
                socket.as_raw_socket() as sys::SOCKET,
                &mut event.raw,
            )
        };

        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

unsafe impl Sync for Epoll {}
unsafe impl Send for Epoll {}

impl Drop for Epoll {
    fn drop(&mut self) {
        if unsafe { sys::epoll_close(self.handle) } == -1 {
            panic!("epoll_close() failed: {}", io::Error::last_os_error());
        }
    }
}

impl AsRawHandle for Epoll {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream, UdpSocket};

    #[test]
    fn test_event_new() {
        let event = Event::new(EventFlag::IN, 42);

        assert_eq!(event.flags(), EventFlag::IN);
        assert_eq!(event.data(), 42);
    }

    #[test]
    fn test_events_with_capacity() {
        let events = Events::with_capacity(2);

        assert_eq!(events.raw.capacity(), 2);
    }

    #[test]
    fn test_events_len() {
        let events = Events::with_capacity(1);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_events_capacity() {
        let events = Events::with_capacity(1);

        assert_eq!(events.capacity(), 1);
    }

    #[test]
    fn test_events_clear() {
        let mut events = Events::with_capacity(1);

        unsafe { events.raw.set_len(1) };

        events.clear();

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_poll_poll_without_timeout() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let mut events = Events::with_capacity(1);

        epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .unwrap();

        assert_eq!(epoll.poll(&mut events, None).unwrap(), 1);
        assert_eq!(events.len(), 1);

        let event = events.iter().next().unwrap();

        assert_eq!(event.data(), 42);
        assert_eq!(event.flags(), EventFlag::OUT);
    }

    #[test]
    fn test_poll_in_and_rdhup() {
        let epoll = Epoll::new().unwrap();
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let socket = TcpStream::connect(l.local_addr().unwrap()).unwrap();
        let mut s1 = l.incoming().next().unwrap().unwrap();
        s1.write_all(b"hello").unwrap();
        let mut events = Events::with_capacity(1);

        epoll
            .register(&socket, EventFlag::IN | EventFlag::RDHUP, 42)
            .unwrap();

        assert_eq!(epoll.poll(&mut events, None).unwrap(), 1);
        let event = events.iter().next().unwrap();
        assert_eq!(event.flags(), EventFlag::IN);

        s1.shutdown(std::net::Shutdown::Write).unwrap();

        assert_eq!(epoll.poll(&mut events, None).unwrap(), 1);
        let event = events.iter().next().unwrap();
        assert_eq!(event.flags(), EventFlag::IN | EventFlag::RDHUP);
    }

    #[test]
    fn test_poll_poll_with_timeout() {
        let epoll = Epoll::new().unwrap();
        let mut events = Events::with_capacity(1);

        epoll
            .poll(&mut events, Some(Duration::from_millis(5)))
            .unwrap();

        assert_eq!(events.len(), 0);
        assert!(events.iter().next().is_none());
    }

    #[test]
    fn test_poll_register_valid() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .is_ok());
    }

    #[test]
    fn test_poll_register_already_registered() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .is_ok());

        assert!(epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 45)
            .is_err());
    }

    #[test]
    fn test_poll_reregister_invalid() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll
            .reregister(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .is_err());
    }

    #[test]
    fn test_poll_reregister_already_registered() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .is_ok());

        assert!(epoll
            .reregister(&socket, EventFlag::OUT | EventFlag::ONESHOT, 45)
            .is_ok());
    }

    #[test]
    fn test_poll_deregister_invalid() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll.deregister(&socket).is_err());
    }

    #[test]
    fn test_poll_deregister_already_registered() {
        let epoll = Epoll::new().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        assert!(epoll
            .register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42)
            .is_ok());

        assert!(epoll.deregister(&socket).is_ok());
    }
}
