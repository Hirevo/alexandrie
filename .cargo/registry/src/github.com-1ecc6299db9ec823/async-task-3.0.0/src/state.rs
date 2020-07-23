/// Set if the task is scheduled for running.
///
/// A task is considered to be scheduled whenever its `Task` reference exists. It therefore also
/// begins in scheduled state at the moment of creation.
///
/// This flag can't be set when the task is completed. However, it can be set while the task is
/// running, in which case it will be rescheduled as soon as polling finishes.
pub(crate) const SCHEDULED: usize = 1 << 0;

/// Set if the task is running.
///
/// A task is in running state while its future is being polled.
///
/// This flag can't be set when the task is completed. However, it can be in scheduled state while
/// it is running, in which case it will be rescheduled as soon as polling finishes.
pub(crate) const RUNNING: usize = 1 << 1;

/// Set if the task has been completed.
///
/// This flag is set when polling returns `Poll::Ready`. The output of the future is then stored
/// inside the task until it becomes closed. In fact, `JoinHandle` picks up the output by marking
/// the task as closed.
///
/// This flag can't be set when the task is scheduled or running.
pub(crate) const COMPLETED: usize = 1 << 2;

/// Set if the task is closed.
///
/// If a task is closed, that means it's either canceled or its output has been consumed by the
/// `JoinHandle`. A task becomes closed when:
///
/// 1. It gets canceled by `Task::cancel()`, `Task::drop()`, or `JoinHandle::cancel()`.
/// 2. Its output gets awaited by the `JoinHandle`.
/// 3. It panics while polling the future.
/// 4. It is completed and the `JoinHandle` gets dropped.
pub(crate) const CLOSED: usize = 1 << 3;

/// Set if the `JoinHandle` still exists.
///
/// The `JoinHandle` is a special case in that it is only tracked by this flag, while all other
/// task references (`Task` and `Waker`s) are tracked by the reference count.
pub(crate) const HANDLE: usize = 1 << 4;

/// Set if the `JoinHandle` is awaiting the output.
///
/// This flag is set while there is a registered awaiter of type `Waker` inside the task. When the
/// task gets closed or completed, we need to wake the awaiter. This flag can be used as a fast
/// check that tells us if we need to wake anyone without acquiring the lock inside the task.
pub(crate) const AWAITER: usize = 1 << 5;

/// Set if an awaiter is being registered.
///
/// This flag is set when `JoinHandle` is polled and we are registering a new awaiter.
pub(crate) const REGISTERING: usize = 1 << 6;

/// Set if the awaiter is being notified.
///
/// This flag is set when notifying the awaiter. If an awaiter is concurrently registered and
/// notified, whichever side came first will take over the reposibility of resolving the race.
pub(crate) const NOTIFYING: usize = 1 << 7;

/// A single reference.
///
/// The lower bits in the state contain various flags representing the task state, while the upper
/// bits contain the reference count. The value of `REFERENCE` represents a single reference in the
/// total reference count.
///
/// Note that the reference counter only tracks the `Task` and `Waker`s. The `JoinHandle` is
/// tracked separately by the `HANDLE` flag.
pub(crate) const REFERENCE: usize = 1 << 8;
