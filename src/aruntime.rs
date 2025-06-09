// SPDX-License-Identifier: MIT OR Apache-2.0

//! Async runtime implementations for test executors.
//!
//! This module provides three runtime implementations that integrate with the
//! [`some_executor`] ecosystem, making them suitable for use in executor-agnostic code.
//!
//! # Available Runtimes
//!
//! - [`SpinRuntime`] - Polls futures in a busy loop (highest performance, highest CPU usage)
//! - [`SleepRuntime`] - Polls futures with sleeping between polls (balanced performance)
//! - [`SpawnRuntime`] - Spawns each future on a new OS thread (best for parallel execution)
//!
//! # Example
//!
//! ```
//! use test_executors::aruntime::SpinRuntime;
//! use some_executor::{SomeExecutor, task::{Task, Configuration}};
//!
//! # test_executors::spin_on(async {
//! let mut runtime = SpinRuntime::new();
//! let task = Task::without_notifications(
//!     "example".to_string(),
//!     async { 42 },
//!     Configuration::default()
//! );
//! let observer = runtime.spawn(task);
//! if let some_executor::observer::FinishedObservation::Ready(value) = observer.await {
//!     assert_eq!(value, 42);
//! }
//! # });
//! ```
//!
//! # Integration with Global Executor
//!
//! You can set a runtime as the global executor using [`set_global_test_runtime`]:
//!
//! ```
//! use test_executors::aruntime;
//!
//! aruntime::set_global_test_runtime();
//! // Now some_executor::global_executor::spawn() will use SpawnRuntime
//! ```

use std::any::Any;
use std::convert::Infallible;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use some_executor::{DynExecutor, SomeExecutor, SomeExecutorExt};
use some_executor::observer::{FinishedObservation, Observer, ObserverNotified};
use some_executor::task::Task;

/// A runtime that polls futures in a busy loop using [`crate::spin_on`].
///
/// This runtime provides the lowest latency but highest CPU usage. It continuously
/// polls futures without yielding the thread, making it ideal for CPU-bound tasks
/// or scenarios where minimal latency is critical.
///
/// # Characteristics
/// - **Latency**: Minimal - responds immediately to future readiness
/// - **CPU Usage**: Maximum - continuously burns CPU cycles
/// - **Blocking**: Yes - blocks the calling thread
/// - **Concurrency**: No - executes one future at a time
///
/// # Example
///
/// ```
/// use test_executors::aruntime::SpinRuntime;
/// use some_executor::{SomeExecutor, task::{Task, Configuration}};
///
/// # test_executors::spin_on(async {
/// let mut runtime = SpinRuntime::new();
/// let task = Task::without_notifications(
///     "example".to_string(),
///     async { 42 },
///     Configuration::default()
/// );
/// let observer = runtime.spawn(task);
/// if let some_executor::observer::FinishedObservation::Ready(value) = observer.await {
///     assert_eq!(value, 42);
/// }
/// # });
/// ```
///
/// # When to Use
/// - For CPU-bound async tasks
/// - When you need minimal latency
/// - In tests where deterministic behavior is important
/// - For short-lived futures that complete quickly
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpinRuntime;

impl SpinRuntime {
    /// Creates a new `SpinRuntime`.
    ///
    /// # Example
    ///
    /// ```
    /// use test_executors::aruntime::SpinRuntime;
    ///
    /// let runtime = SpinRuntime::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl SomeExecutorExt for SpinRuntime {}
impl SomeExecutor for SpinRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output>>(&mut self, task: Task<F, Notifier>) -> impl Observer<Value=F::Output>
    where
        Self: Sized,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        while task.poll_after() > crate::sys::time::Instant::now() {
            std::hint::spin_loop()
        }
        let (spawned, observer) = task.spawn(self);
        crate::spin_on(spawned);
        observer
    }


    async fn spawn_async<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Observer<Value=F::Output> where
        Self: Sized,
        F::Output: Send + Unpin,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        while spawned.poll_after() > crate::sys::time::Instant::now() {
            std::hint::spin_loop()
        }
        crate::spin_on(spawned);
        observer
    }


    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());

        let (spawned, observer) = task.spawn_objsafe(self);
        while spawned.poll_after() > crate::sys::time::Instant::now() {
            std::hint::spin_loop()
        }
        crate::spin_on(spawned);
        Box::new(observer)
    }

    fn spawn_objsafe_async<'s>(&'s mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Future<Output=Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>>> + 's> {
        #[allow(clippy::async_yields_async)]
        Box::new(async {
            Self::spawn_objsafe(self, task)
        })
    }


    fn clone_box(&self) -> Box<DynExecutor> {
        Box::new(*self)
    }

    fn executor_notifier(&mut self) -> Option<Self::ExecutorNotifier> {
        None
    }
}

/// A runtime that polls futures with sleeping between polls using [`crate::sleep_on`].
///
/// This runtime provides a balance between responsiveness and CPU efficiency. It uses
/// a condition variable to sleep when futures return `Poll::Pending`, waking only when
/// the waker is triggered.
///
/// # Characteristics
/// - **Latency**: Moderate - wakes on waker notification
/// - **CPU Usage**: Low - sleeps when waiting
/// - **Blocking**: Yes - blocks the calling thread
/// - **Concurrency**: No - executes one future at a time
///
/// # Example
///
/// ```
/// use test_executors::aruntime::SleepRuntime;
/// use some_executor::{SomeExecutor, task::{Task, Configuration}};
///
/// # test_executors::spin_on(async {
/// let mut runtime = SleepRuntime::new();
/// let task = Task::without_notifications(
///     "io_task".to_string(),
///     async { "completed".to_string() },
///     Configuration::default()
/// );
/// let observer = runtime.spawn(task);
/// if let some_executor::observer::FinishedObservation::Ready(value) = observer.await {
///     assert_eq!(value, "completed");
/// }
/// # });
/// ```
///
/// # When to Use
/// - For I/O-bound async tasks
/// - When you want to avoid burning CPU cycles
/// - For longer-running futures
/// - In tests that involve actual async I/O or timers
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SleepRuntime;
impl SleepRuntime {
    /// Creates a new `SleepRuntime`.
    ///
    /// # Example
    ///
    /// ```
    /// use test_executors::aruntime::SleepRuntime;
    ///
    /// let runtime = SleepRuntime::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}
impl SomeExecutorExt for SleepRuntime {

}

impl SomeExecutor for SleepRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output>>(&mut self, task: Task<F, Notifier>) -> impl Observer<Value=F::Output>
    where
        Self: Sized,
        F::Output: Send,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        let now = crate::sys::time::Instant::now();
        if spawned.poll_after() > now {
            let dur = now.duration_since(spawned.poll_after());
            std::thread::sleep(dur);
        }
        crate::sleep_on(spawned);
        observer
    }

    async fn spawn_async<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Observer<Value=F::Output>
    where
        Self: Sized,
        F::Output: Send + Unpin,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        let now = crate::sys::time::Instant::now();
        if spawned.poll_after() > now {
            let dur = spawned.poll_after() - now;
            std::thread::sleep(dur);
        }
        crate::sleep_on(spawned);
        observer
    }

    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn_objsafe(self);
        let now = crate::sys::time::Instant::now();
        if spawned.poll_after() > now {
            let dur = now.duration_since(spawned.poll_after());
            std::thread::sleep(dur);
        }
        crate::sleep_on(spawned);
        Box::new(observer)
    }

    fn spawn_objsafe_async<'s>(&'s mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Future<Output=Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>>> + 's> {
        #[allow(clippy::async_yields_async)]
        Box::new(async {
            Self::spawn_objsafe(self, task)
        })
    }

    fn clone_box(&self) -> Box<DynExecutor> {
        Box::new(*self)
    }

    fn executor_notifier(&mut self) -> Option<Self::ExecutorNotifier> {
        None
    }
}


/// A runtime that spawns each future on a new OS thread using [`crate::spawn_on`].
///
/// This runtime provides true parallelism by running each future on its own thread.
/// It returns immediately after spawning, making it suitable for fire-and-forget tasks
/// or when you need parallel execution.
///
/// # Characteristics
/// - **Latency**: Low - returns immediately after spawning
/// - **CPU Usage**: Efficient - only uses CPU when futures are ready
/// - **Blocking**: No - doesn't block the calling thread
/// - **Concurrency**: Yes - can run multiple futures in parallel
///
/// # Example
///
/// ```
/// use test_executors::aruntime::SpawnRuntime;
/// use some_executor::{SomeExecutor, task::{Task, Configuration}};
/// use std::sync::{Arc, Mutex};
///
/// # test_executors::spin_on(async {
/// let mut runtime = SpawnRuntime::new();
/// let results = Arc::new(Mutex::new(Vec::new()));
/// let results_clone = results.clone();
///
/// let task = Task::without_notifications(
///     "parallel_task".to_string(),
///     async move {
///         results_clone.lock().unwrap().push(42);
///     },
///     Configuration::default()
/// );
/// let observer = runtime.spawn(task);
/// 
/// // The task runs in parallel
/// observer.await;
/// assert_eq!(results.lock().unwrap().len(), 1);
/// # });
/// ```
///
/// # When to Use
/// - For tasks that should run in parallel
/// - When you don't want to block the calling thread
/// - For fire-and-forget operations
/// - When you need true concurrency
///
/// # Thread Safety
/// The futures spawned must be `Send` since they will be moved to another thread.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpawnRuntime;
impl SpawnRuntime {
    /// Creates a new `SpawnRuntime`.
    ///
    /// # Example
    ///
    /// ```
    /// use test_executors::aruntime::SpawnRuntime;
    ///
    /// let runtime = SpawnRuntime::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}
impl SomeExecutorExt for SpawnRuntime {
}


impl SomeExecutor for SpawnRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Observer<Value=F::Output>
    where
        Self: Sized,
        F::Output: Send,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        std::thread::spawn(move || {
            if spawned.poll_after() > crate::sys::time::Instant::now() {
                let dur = crate::sys::time::Instant::now().duration_since(spawned.poll_after());
                std::thread::sleep(dur);
            }
            crate::sleep_on(spawned);
        });
        observer
    }

    fn spawn_async<'s, F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&'s mut self, task: Task<F, Notifier>) -> impl Future<Output=impl Observer<Value=F::Output>> + Send + 's
    where
        Self: Sized,
        F::Output: Send + Unpin,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        #[allow(clippy::async_yields_async)]
        async move {
            let (spawned, observer) = task.spawn(self);
            std::thread::spawn(move || {
                if spawned.poll_after() > crate::sys::time::Instant::now() {
                    let dur = spawned.poll_after() - crate::sys::time::Instant::now();
                    std::thread::sleep(dur);
                }
                crate::sleep_on(spawned);
            });
            observer
        }
    }

    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn_objsafe(self);
        std::thread::spawn(move || {
            if spawned.poll_after() > crate::sys::time::Instant::now() {
                let dur = crate::sys::time::Instant::now().duration_since(spawned.poll_after());
                std::thread::sleep(dur);
            }
            crate::sleep_on(spawned);
        });
        Box::new(observer)
    }

    fn spawn_objsafe_async<'s>(&'s mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Box<dyn Future<Output=Box<dyn Observer<Value=Box<dyn Any + Send>, Output=FinishedObservation<Box<dyn Any + Send>>>>> + 's> {
        #[allow(clippy::async_yields_async)]
        Box::new(async {
            Self::spawn_objsafe(self, task)
        })
    }

    fn clone_box(&self) -> Box<DynExecutor> {
        Box::new(*self)
    }

    fn executor_notifier(&mut self) -> Option<Self::ExecutorNotifier> {
        None
    }
}

//boilerplate

impl Display for SpinRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpinRuntime")
    }
}

impl Display for SleepRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SleepRuntime")
    }
}

impl Display for SpawnRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpawnRuntime")
    }
}

impl Default for SpinRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SleepRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SpawnRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Sets a [`SpawnRuntime`] as the global executor for the `some_executor` ecosystem.
///
/// This function configures a `SpawnRuntime` instance as the global executor,
/// allowing code that uses `some_executor::global_executor::spawn()` to automatically
/// use this runtime for task execution.
///
/// # Example
///
/// ```
/// use test_executors::aruntime;
///
/// // Set the global runtime
/// aruntime::set_global_test_runtime();
///
/// // Now the global executor is available for use
/// // via some_executor::global_executor::global_executor()
/// ```
///
/// # Note
/// This function uses `SpawnRuntime`, which creates a new thread for each spawned
/// future. This provides good parallelism but may have higher overhead for many
/// small tasks.
pub fn set_global_test_runtime() {
    let as_dyn = Box::new(SpawnRuntime) as Box<DynExecutor>;
    some_executor::global_executor::set_global_executor(as_dyn)
}
#[cfg(test)]
mod test {
    #[test]
    fn assert_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<super::SpinRuntime>();
        assert_send_sync::<super::SleepRuntime>();
        assert_send_sync::<super::SpawnRuntime>();
    }
}


