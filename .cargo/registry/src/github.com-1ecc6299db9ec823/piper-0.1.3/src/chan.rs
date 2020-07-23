use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::isize;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::process;
use std::ptr;
use std::sync::atomic::{self, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use crossbeam_utils::Backoff;
use futures_util::future;
use futures_sink::Sink;
use futures_util::stream::Stream;

use crate::event::{Event, EventListener};

/// Creates a bounded multi-producer multi-consumer channel.
///
/// This channel has a buffer that holds at most `cap` messages at a time. If `cap` is zero, no
/// messages can be stored in the channel, which means send operations must pair with receive
/// operations in order to pass messages over.
///
/// Senders and receivers can be cloned. When all senders associated with a channel are dropped,
/// remaining messages can still be received, but after that receive operations will return
/// [`None`]. On the other hand, when all receivers are dropped, further send operation block
/// forever.
///
/// # Examples
///
/// ```
/// use smol::{Task, Timer};
/// use std::time::Duration;
///
/// # smol::run(async {
/// // Create a channel that can hold 1 message at a time.
/// let (s, r) = piper::chan(1);
///
/// // Sending completes immediately because there is enough space in the channel.
/// s.send(1).await;
///
/// let t = Task::spawn(async move {
///     // This send operation is blocked because the channel is full.
///     // It will be able to complete only after the first message is received.
///     s.send(2).await;
/// });
///
/// // Sleep for a second and then receive both messages.
/// Timer::after(Duration::from_secs(1)).await;
/// assert_eq!(r.recv().await, Some(1));
/// assert_eq!(r.recv().await, Some(2));
/// # })
/// ```
pub fn chan<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel::with_capacity(cap));
    let s = Sender {
        channel: channel.clone(),
        buffer: VecDeque::new(),
        listener: None,
    };
    let r = Receiver {
        channel,
        listener: None,
    };
    (s, r)
}

/// The sending side of a channel.
///
/// This struct is created by the [`chan`] function. See its documentation for more.
///
/// Senders can be cloned and implement [`Sink`].
///
/// # Examples
///
/// ```
/// use smol::Task;
///
/// # smol::run(async {
/// let (s1, r) = piper::chan(100);
/// let s2 = s1.clone();
///
/// let t1 = Task::spawn(async move { s1.send(1).await });
/// let t2 = Task::spawn(async move { s2.send(2).await });
///
/// let msg1 = r.recv().await.unwrap();
/// let msg2 = r.recv().await.unwrap();
/// assert_eq!(msg1 + msg2, 3);
/// # })
/// ```
pub struct Sender<T> {
    /// The inner channel.
    channel: Arc<Channel<T>>,

    /// The sink buffer.
    ///
    /// Messages sent into this sender as a sink are first buffered, and then they get sent into
    /// the channel when the sink is flushed.
    buffer: VecDeque<T>,

    /// Listens for a receive or handoff event that unblocks this sink.
    listener: Option<EventListener>,
}

impl<T> Unpin for Sender<T> {}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Sender<T> {}

impl<T> Sender<T> {
    /// Sends a message into the channel.
    ///
    /// If the channel is full, this method waits until there is space for a message in the
    /// channel. If there are no receivers on this channel, sending waits forever.
    ///
    /// # Examples
    ///
    /// ```
    /// use smol::Task;
    ///
    /// # smol::run(async {
    /// let (s, r) = piper::chan(1);
    ///
    /// let t = Task::spawn(async move {
    ///     s.send(1).await;
    ///     s.send(2).await;
    /// });
    ///
    /// assert_eq!(r.recv().await, Some(1));
    /// assert_eq!(r.recv().await, Some(2));
    /// assert_eq!(r.recv().await, None);
    /// # })
    /// ```
    pub async fn send(&self, msg: T) {
        self.channel.send(msg).await
    }

    /// Returns the channel capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// let (s, _) = piper::chan::<i32>(5);
    /// assert_eq!(s.capacity(), 5);
    /// ```
    pub fn capacity(&self) -> usize {
        // If this channel does handoff, the capacity is 0, even though the internal buffer's
        // capacity is 1.
        if self.channel.handoff.is_some() {
            0
        } else {
            self.channel.cap
        }
    }

    /// Returns `true` if the channel is empty.
    ///
    /// If the channel's capacity is zero, it is always empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # smol::run(async {
    /// let (s, r) = piper::chan(1);
    ///
    /// assert!(s.is_empty());
    /// s.send(0).await;
    /// assert!(!s.is_empty());
    /// # })
    /// ```
    pub fn is_empty(&self) -> bool {
        if self.capacity() == 0 {
            true
        } else {
            self.channel.is_empty()
        }
    }

    /// Returns `true` if the channel is full.
    ///
    /// If the channel's capacity is zero, it is always full.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// # smol::run(async {
    /// let (s, r) = piper::chan(1);
    ///
    /// assert!(!s.is_full());
    /// s.send(0).await;
    /// assert!(s.is_full());
    /// #
    /// # })
    /// ```
    pub fn is_full(&self) -> bool {
        if self.capacity() == 0 {
            true
        } else {
            self.channel.is_full()
        }
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # smol::run(async {
    /// let (s, r) = piper::chan(2);
    /// assert_eq!(s.len(), 0);
    ///
    /// s.send(1).await;
    /// s.send(2).await;
    /// assert_eq!(s.len(), 2);
    /// # })
    /// ```
    pub fn len(&self) -> usize {
        self.channel.len()
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Decrement the sender count and disconnect the channel if it drops down to zero.
        if self.channel.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.disconnect();
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        let count = self.channel.sender_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of sender clones are leaked.
        if count > isize::MAX as usize {
            process::abort();
        }

        Sender {
            channel: self.channel.clone(),
            buffer: VecDeque::new(),
            listener: None,
        }
    }
}

impl<T> Sink<T> for Sender<T> {
    type Error = std::convert::Infallible;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.buffer.is_empty() {
            Poll::Ready(Ok(()))
        } else {
            // If there are messages in the buffer, we should encourage the user to flush the sink
            // rather than sending more messages into the sink.
            Poll::Pending
        }
    }

    fn start_send(mut self: Pin<&mut Self>, msg: T) -> Result<(), Self::Error> {
        // Sending simply involves pushing the message into the sink's buffer.
        self.buffer.push_back(msg);
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            // If this sink is blocked on an event, first make sure it is unblocked.
            if let Some(listener) = self.listener.as_mut() {
                futures_util::ready!(Pin::new(listener).poll(cx));
                self.listener = None;
            }

            loop {
                // Get the next message from the buffer.
                let msg = match self.buffer.pop_front() {
                    None => return Poll::Ready(Ok(())),
                    Some(msg) => msg,
                };

                // Attempt to send the message.
                match self.channel.try_send(msg) {
                    Ok(stamp) => {
                        // The sink is not blocked on an event - drop the listener.
                        self.listener = None;

                        // If this is a zero-capacity channel, we need to wait for the receiver to
                        // confirm the message was received.
                        if let Some(h) = &self.channel.handoff {
                            // The internal buffer's capacity is 1, which means the pointer to the
                            // buffer matches the pointer to the first (and only) slot in it.
                            let slot_stamp = unsafe { &(*self.channel.buffer).stamp };

                            // If the stamp didn't change, the message was not received yet.
                            if slot_stamp.load(Ordering::SeqCst) == stamp {
                                // Listen for a handoff event.
                                let listener = h.listen();

                                // Check the stamp again.
                                if slot_stamp.load(Ordering::SeqCst) == stamp {
                                    // Now we're really blocked on handoff - store this listener
                                    // and go back to the outer loop.
                                    self.listener = Some(listener);
                                    break;
                                }
                            }
                        }
                        // Continue the inner loop to send the next message...
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        // The sink is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Pending;
                    }
                    Err(TrySendError::Full(m)) => {
                        // The buffer is full - put the message back into the buffer.
                        self.buffer.push_front(m);

                        // Listen for a receive event.
                        match self.listener.as_mut() {
                            None => {
                                // Create a listener and try sending the message again.
                                self.listener = Some(self.channel.sink_ops.listen());
                            }
                            Some(_) => {
                                // Go back to the outer loop to poll the listener.
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Sender { .. }")
    }
}

/// The receiving side of a channel.
///
/// This struct is created by the [`chan`] function. See its documentation for more.
///
/// Receivers can be cloned and implement [`Stream`]. Note if a message is sent into a channel and
/// there are multiple receivers, only one of them will receive the message.
///
/// # Examples
///
/// ```
/// use smol::{Task, Timer};
/// use std::time::Duration;
///
/// # smol::run(async {
/// let (s, r) = piper::chan(100);
///
/// let t = Task::spawn((async move {
///     s.send(1).await;
///     Timer::after(Duration::from_secs(1)).await;
///     s.send(2).await;
/// });
///
/// assert_eq!(r.recv().await, Some(1)); // Received immediately.
/// assert_eq!(r.recv().await, Some(2)); // Received after 1 second.
/// #
/// # })
/// ```
pub struct Receiver<T> {
    /// The inner channel.
    channel: Arc<Channel<T>>,

    /// Listens for a send or disconnect event that unblocks this stream.
    listener: Option<EventListener>,
}

impl<T> Unpin for Receiver<T> {}

unsafe impl<T: Send> Send for Receiver<T> {}
unsafe impl<T: Send> Sync for Receiver<T> {}

impl<T> Receiver<T> {
    /// TODO
    pub fn try_recv(&self) -> Option<T> {
        self.channel.try_recv().ok()
    }

    /// Receives a message from the channel.
    ///
    /// If the channel is empty and still has senders, this method will wait until a message is
    /// sent into the channel or until all senders get dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # async_std::task::block_on(async {
    /// #
    /// use async_std::sync::channel;
    /// use async_std::task;
    ///
    /// let (s, r) = channel(1);
    ///
    /// task::spawn(async move {
    ///     s.send(1).await;
    ///     s.send(2).await;
    /// });
    ///
    /// assert_eq!(r.recv().await, Some(1));
    /// assert_eq!(r.recv().await, Some(2));
    /// assert_eq!(r.recv().await, None);
    /// #
    /// # })
    /// ```
    pub async fn recv(&self) -> Option<T> {
        self.channel.recv().await
    }

    /// Returns the channel capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::sync::channel;
    ///
    /// let (_, r) = channel::<i32>(5);
    /// assert_eq!(r.capacity(), 5);
    /// ```
    pub fn capacity(&self) -> usize {
        // If this channel does handoff, the capacity is 0, even though the internal buffer's
        // capacity is 1.
        if self.channel.handoff.is_some() {
            0
        } else {
            self.channel.cap
        }
    }

    /// Returns `true` if the channel is empty.
    ///
    /// If the channel's capacity is zero, it is always empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # async_std::task::block_on(async {
    /// #
    /// use async_std::sync::channel;
    ///
    /// let (s, r) = channel(1);
    ///
    /// assert!(r.is_empty());
    /// s.send(0).await;
    /// assert!(!r.is_empty());
    /// #
    /// # })
    /// ```
    pub fn is_empty(&self) -> bool {
        if self.capacity() == 0 {
            true
        } else {
            self.channel.is_empty()
        }
    }

    /// Returns `true` if the channel is full.
    ///
    /// If the channel's capacity is zero, it is always full.
    ///
    /// # Examples
    ///
    /// ```
    /// # async_std::task::block_on(async {
    /// #
    /// use async_std::sync::channel;
    ///
    /// let (s, r) = channel(1);
    ///
    /// assert!(!r.is_full());
    /// s.send(0).await;
    /// assert!(r.is_full());
    /// #
    /// # })
    /// ```
    pub fn is_full(&self) -> bool {
        if self.capacity() == 0 {
            true
        } else {
            self.channel.is_full()
        }
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # async_std::task::block_on(async {
    /// #
    /// use async_std::sync::channel;
    ///
    /// let (s, r) = channel(2);
    /// assert_eq!(r.len(), 0);
    ///
    /// s.send(1).await;
    /// s.send(2).await;
    /// assert_eq!(r.len(), 2);
    /// #
    /// # })
    /// ```
    pub fn len(&self) -> usize {
        self.channel.len()
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Decrement the receiver count and disconnect the channel if it drops down to zero.
        if self.channel.receiver_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.disconnect();
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Receiver<T> {
        let count = self.channel.receiver_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of receiver clones are leaked.
        if count > isize::MAX as usize {
            process::abort();
        }

        Receiver {
            channel: self.channel.clone(),
            listener: None,
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // If this stream is blocked on an event, first make sure it is unblocked.
            if let Some(listener) = self.listener.as_mut() {
                futures_util::ready!(Pin::new(listener).poll(cx));
                self.listener = None;
            }

            loop {
                // Attempt to receive a message.
                match self.channel.try_recv() {
                    Ok(msg) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(Some(msg));
                    }
                    Err(TryRecvError::Disconnected) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(None);
                    }
                    Err(TryRecvError::Empty) => {}
                }

                // Listen for a send event.
                match self.listener.as_mut() {
                    None => {
                        // Create a listener and try sending the message again.
                        self.listener = Some(self.channel.next_ops.listen());
                    }
                    Some(_) => {
                        // Go back to the outer loop to poll the listener.
                        break;
                    }
                }
            }
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Receiver { .. }")
    }
}

/// A slot in a channel.
struct Slot<T> {
    /// The current stamp.
    stamp: AtomicUsize,

    /// The message in this slot.
    msg: UnsafeCell<T>,
}

/// Bounded channel based on a preallocated array.
struct Channel<T> {
    /// The head of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit in the head is always zero.
    ///
    /// Messages are popped from the head of the channel.
    head: AtomicUsize,

    /// The tail of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit indicates that the channel is disconnected.
    ///
    /// Messages are pushed into the tail of the channel.
    tail: AtomicUsize,

    /// The buffer holding slots.
    buffer: *mut Slot<T>,

    /// The channel capacity.
    cap: usize,

    /// A stamp with the value of `{ lap: 1, mark: 0, index: 0 }`.
    one_lap: usize,

    /// If this bit is set in the tail, that means either all senders were dropped or all receivers
    /// were dropped.
    mark_bit: usize,

    /// Send operations waiting while the channel is full.
    send_ops: Event,

    /// Sink operations waiting while the channel is full.
    sink_ops: Event,

    /// TODO
    handoff: Option<Event>,

    /// Receive operations waiting while the channel is empty and not disconnected.
    recv_ops: Event,

    /// Stream operations while the channel is empty and not disconnected.
    next_ops: Event,

    /// The number of currently active `Sender`s.
    sender_count: AtomicUsize,

    /// The number of currently active `Receivers`s.
    receiver_count: AtomicUsize,

    /// Indicates that dropping a `Channel<T>` may drop values of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Unpin for Channel<T> {}

unsafe impl<T: Send> Send for Channel<T> {}
unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Channel<T> {
    /// Creates a bounded channel of capacity `cap`.
    fn with_capacity(cap: usize) -> Self {
        let handoff = if cap == 0 { Some(Event::new()) } else { None };
        let cap = cap.max(1);

        // Compute constants `mark_bit` and `one_lap`.
        let mark_bit = (cap + 1).next_power_of_two();
        let one_lap = mark_bit * 2;

        // Head is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let head = 0;
        // Tail is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let tail = 0;

        // Allocate a buffer of `cap` slots.
        let buffer = {
            let mut v = Vec::<Slot<T>>::with_capacity(cap);
            let ptr = v.as_mut_ptr();
            mem::forget(v);
            ptr
        };

        // Initialize stamps in the slots.
        for i in 0..cap {
            unsafe {
                // Set the stamp to `{ lap: 0, mark: 0, index: i }`.
                let slot = buffer.add(i);
                ptr::write(&mut (*slot).stamp, AtomicUsize::new(i));
            }
        }

        Channel {
            buffer,
            cap,
            one_lap,
            mark_bit,
            head: AtomicUsize::new(head),
            tail: AtomicUsize::new(tail),
            send_ops: Event::new(),
            sink_ops: Event::new(),
            handoff,
            recv_ops: Event::new(),
            next_ops: Event::new(),
            sender_count: AtomicUsize::new(1),
            receiver_count: AtomicUsize::new(1),
            _marker: PhantomData,
        }
    }

    /// Attempts to send a message.
    fn try_send(&self, msg: T) -> Result<usize, TrySendError<T>> {
        let backoff = Backoff::new();
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            // Extract mark bit from the tail and unset it.
            //
            // If the mark bit was set (which means all receivers have been dropped), we will still
            // send the message into the channel if there is enough capacity. The message will get
            // dropped when the channel is dropped (which means when all senders are also dropped).
            let mark_bit = tail & self.mark_bit;
            tail ^= mark_bit;

            // Deconstruct the tail.
            let index = tail & (self.mark_bit - 1);
            let lap = tail & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            let slot = unsafe { &*self.buffer.add(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                let new_tail = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    tail + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail | mark_bit,
                    new_tail | mark_bit,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Write the message into the slot and update the stamp.
                        unsafe { slot.msg.get().write(msg) };
                        let stamp = tail + 1;
                        slot.stamp.store(stamp, Ordering::Release);

                        // Wake a blocked receive operation.
                        self.recv_ops.notify_one();

                        // Wake all blocked streams.
                        self.next_ops.notify_all();

                        return Ok(stamp);
                    }
                    Err(t) => {
                        tail = t;
                        backoff.spin();
                    }
                }
            } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
                full_fence();
                let head = self.head.load(Ordering::Relaxed);

                // If the head lags one lap behind the tail as well...
                if head.wrapping_add(self.one_lap) == tail {
                    // ...then the channel is full.

                    // Check if the channel is disconnected.
                    if mark_bit != 0 {
                        return Err(TrySendError::Disconnected(msg));
                    } else {
                        return Err(TrySendError::Full(msg));
                    }
                }

                backoff.spin();
                tail = self.tail.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    async fn send(&self, mut msg: T) {
        let mut listener = None;

        let stamp = loop {
            match self.try_send(msg) {
                Ok(stamp) => break stamp,
                Err(TrySendError::Disconnected(_)) => return future::pending().await,
                Err(TrySendError::Full(m)) => msg = m,
            }

            match listener.take() {
                None => listener = Some(self.send_ops.listen()),
                Some(l) => {
                    l.await;
                    if self.cap > 1 {
                        self.send_ops.notify_one();
                    }
                }
            }
        };

        let listener = match &self.handoff {
            None => return,
            Some(h) => {
                let slot_stamp = unsafe { &(*self.buffer).stamp };

                if slot_stamp.load(Ordering::SeqCst) != stamp {
                    return;
                }

                let listener = h.listen();

                if slot_stamp.load(Ordering::SeqCst) != stamp {
                    return;
                }

                listener
            }
        };
        listener.await;
    }

    /// Attempts to receive a message.
    fn try_recv(&self) -> Result<T, TryRecvError> {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            let slot = unsafe { &*self.buffer.add(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Read the message from the slot and update the stamp.
                        let msg = unsafe { slot.msg.get().read() };
                        let stamp = head.wrapping_add(self.one_lap);
                        slot.stamp.store(stamp, Ordering::Release);

                        // Wake a blocked send operation.
                        self.send_ops.notify_one();

                        // Wake all blocked sinks.
                        self.sink_ops.notify_all();

                        // Notify send operations waiting for handoff.
                        if let Some(h) = &self.handoff {
                            h.notify_all();
                        }

                        return Ok(msg);
                    }
                    Err(h) => {
                        head = h;
                        backoff.spin();
                    }
                }
            } else if stamp == head {
                full_fence();
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if (tail & !self.mark_bit) == head {
                    // If the channel is disconnected...
                    if tail & self.mark_bit != 0 {
                        return Err(TryRecvError::Disconnected);
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return Err(TryRecvError::Empty);
                    }
                }

                backoff.spin();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    async fn recv(&self) -> Option<T> {
        let mut listener = None;

        loop {
            match self.try_recv() {
                Ok(msg) => return Some(msg),
                Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {}
            }

            match listener.take() {
                None => listener = Some(self.recv_ops.listen()),
                Some(l) => {
                    l.await;
                    if self.cap > 1 {
                        self.recv_ops.notify_one();
                    }
                }
            }
        }
    }

    /// Returns the current number of messages inside the channel.
    fn len(&self) -> usize {
        loop {
            // Load the tail, then load the head.
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            // If the tail didn't change, we've got consistent values to work with.
            if self.tail.load(Ordering::SeqCst) == tail {
                let hix = head & (self.mark_bit - 1);
                let tix = tail & (self.mark_bit - 1);

                return if hix < tix {
                    tix - hix
                } else if hix > tix {
                    self.cap - hix + tix
                } else if (tail & !self.mark_bit) == head {
                    0
                } else {
                    self.cap
                };
            }
        }
    }

    /// Returns `true` if the channel is empty.
    fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);

        // Is the tail equal to the head?
        //
        // Note: If the head changes just before we load the tail, that means there was a moment
        // when the channel was not empty, so it is safe to just return `false`.
        (tail & !self.mark_bit) == head
    }

    /// Returns `true` if the channel is full.
    fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::SeqCst);
        let head = self.head.load(Ordering::SeqCst);

        // Is the head lagging one lap behind tail?
        //
        // Note: If the tail changes just before we load the head, that means there was a moment
        // when the channel was not full, so it is safe to just return `false`.
        head.wrapping_add(self.one_lap) == tail & !self.mark_bit
    }

    /// Disconnects the channel and wakes up all blocked operations.
    fn disconnect(&self) {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);

        if tail & self.mark_bit == 0 {
            // Notify everyone blocked on this channel.
            self.send_ops.notify_all();
            self.sink_ops.notify_all();
            self.recv_ops.notify_all();
            self.next_ops.notify_all();
            if let Some(h) = &self.handoff {
                h.notify_all();
            }
        }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        // Get the index of the head.
        let hix = self.head.load(Ordering::Relaxed) & (self.mark_bit - 1);

        // Loop over all slots that hold a message and drop them.
        for i in 0..self.len() {
            // Compute the index of the next slot holding a message.
            let index = if hix + i < self.cap {
                hix + i
            } else {
                hix + i - self.cap
            };

            unsafe {
                self.buffer.add(index).drop_in_place();
            }
        }

        // Finally, deallocate the buffer, but don't run any destructors.
        unsafe {
            Vec::from_raw_parts(self.buffer, 0, self.cap);
        }
    }
}

/// An error returned from the `try_send()` method.
enum TrySendError<T> {
    /// The channel is full but not disconnected.
    Full(T),

    /// The channel is full and disconnected.
    Disconnected(T),
}

/// An error returned from the `try_recv()` method.
enum TryRecvError {
    /// The channel is empty but not disconnected.
    Empty,

    /// The channel is empty and disconnected.
    Disconnected,
}

/// Equivalent to `atomic::fence(Ordering::SeqCst)`, but in some cases faster.
#[inline]
fn full_fence() {
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        // HACK(stjepang): On x86 architectures there are two different ways of executing
        // a `SeqCst` fence.
        //
        // 1. `atomic::fence(SeqCst)`, which compiles into a `mfence` instruction.
        // 2. `_.compare_and_swap(_, _, SeqCst)`, which compiles into a `lock cmpxchg` instruction.
        //
        // Both instructions have the effect of a full barrier, but empirical benchmarks have shown
        // that the second one is sometimes a bit faster.
        //
        // The ideal solution here would be to use inline assembly, but we're instead creating a
        // temporary atomic variable and compare-and-exchanging its value. No sane compiler to
        // x86 platforms is going to optimize this away.
        let a = AtomicUsize::new(0);
        a.compare_and_swap(0, 1, Ordering::SeqCst);
    } else {
        atomic::fence(Ordering::SeqCst);
    }
}
