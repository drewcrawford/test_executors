// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
This crate provides extremely simple, yet useful, async executors. They are primarily useful for writing unit tests
without bringing in a full-blown executor such as [tokio](https://tokio.rs).

![logo](../../../art/logo.png)

# Quick Start

```
use test_executors::{spin_on, sleep_on};

// Run a simple async function
let result = spin_on(async {
    42
});
assert_eq!(result, 42);

// Run an async function that sleeps
let result = sleep_on(async {
    // Your async code here
    "Hello, async!"
});
assert_eq!(result, "Hello, async!");
```

# Available Executors

The crate provides three main executors:

* [`spin_on`] - Polls a future in a busy loop on the current thread. Best for CPU-bound tasks or when latency is critical.
* [`sleep_on`] - Polls a future on the current thread, sleeping between polls. Best for I/O-bound tasks to avoid burning CPU.
* [`spawn_on`] - Spawns a future on a new thread, polling it there. Best for fire-and-forget tasks.

# Platform Support

## Native Platforms
All executors work as described above on native platforms (Linux, macOS, Windows, etc.).

## WebAssembly Support
This crate has special support for `wasm32` targets:
- The `async_test` macro automatically adapts to use `wasm-bindgen-test` on WASM
- `spawn_local` uses `wasm_bindgen_futures::spawn_local` on WASM targets

# Features

## `async_test` Macro
The [`async_test`] macro allows you to write async tests that work on both native and WASM targets:

```
use test_executors::async_test;

#[async_test]
async fn my_test() {
    let value = async { 42 }.await;
    assert_eq!(value, 42);
}
```

## Integration with `some_executor`
This crate implements the [some_executor](https://crates.io/crates/some_executor) trait for all executors,
allowing them to be used in executor-agnostic code:

```
use test_executors::aruntime::SpinRuntime;
use some_executor::SomeExecutor;

let mut runtime = SpinRuntime::new();
// Use runtime with some_executor traits
```

# Utilities

The crate also provides utility functions and types:
- [`poll_once`] and [`poll_once_pin`] - Poll a future exactly once
- [`spawn_local`] - Platform-aware spawning that works on both native and WASM
- [`pend_forever::PendForever`] - A future that is always pending (useful for testing)

*/

pub mod aruntime;
mod noop_waker;
pub mod pend_forever;
mod sys;

use crate::noop_waker::new_context;
use blocking_semaphore::one::Semaphore;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub use test_executors_proc::async_test;

extern crate self as test_executors;

/// Blocks the calling thread until a future is ready, using a spinloop.
///
/// This executor continuously polls the future in a tight loop without yielding the thread.
/// It's the most responsive executor but also the most CPU-intensive.
///
/// # When to Use
/// - When you need minimal latency
/// - For CPU-bound async tasks
/// - In tests where you want deterministic behavior
/// - When the future is expected to complete quickly
///
/// # Example
/// ```
/// use test_executors::spin_on;
///
/// let result = spin_on(async {
///     // Simulate some async work
///     let value = async { 21 }.await;
///     value * 2
/// });
/// assert_eq!(result, 42);
/// ```
///
/// # Performance Note
/// This executor will consume 100% CPU while waiting. For I/O-bound tasks or
/// long-running futures, consider using [`sleep_on`] instead.
pub fn spin_on<F: Future>(mut future: F) -> F::Output {
    //we inherit the parent dlog::context here.
    let mut context = new_context();
    let mut future = unsafe { Pin::new_unchecked(&mut future) };
    loop {
        if let Poll::Ready(val) = future.as_mut().poll(&mut context) {
            return val;
        }
        std::hint::spin_loop();
    }
}

struct SimpleWakeShared {
    semaphore: Semaphore,
}

static CONDVAR_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |ctx| {
        let ctx = unsafe { Arc::from_raw(ctx as *const SimpleWakeShared) };
        let ctx2 = ctx.clone();
        std::mem::forget(ctx);
        RawWaker::new(Arc::into_raw(ctx2) as *const (), &CONDVAR_WAKER_VTABLE)
    },
    |ctx| {
        let ctx = unsafe { Arc::from_raw(ctx as *const SimpleWakeShared) };
        logwise::trace_sync!("waking");
        ctx.semaphore.signal_if_needed();
    },
    |ctx| {
        let ctx = unsafe { Arc::from_raw(ctx as *const SimpleWakeShared) };
        logwise::trace_sync!("waking (by ref)");
        ctx.semaphore.signal_if_needed();
        std::mem::forget(ctx);
    },
    |ctx| {
        let ctx = unsafe { Arc::from_raw(ctx as *const SimpleWakeShared) };
        std::mem::drop(ctx);
    },
);
/// Blocks the calling thread until a future is ready, sleeping between polls.
///
/// This executor uses a condition variable to sleep the thread when the future
/// returns `Poll::Pending`, waking up only when the waker is triggered.
/// This is more CPU-efficient than [`spin_on`] but may have higher latency.
///
/// # When to Use
/// - For I/O-bound async tasks
/// - When you want to avoid burning CPU cycles
/// - For longer-running futures
/// - In tests that involve actual async I/O or timers
///
/// # Example
/// ```
/// use test_executors::sleep_on;
/// use std::future::Future;
/// use std::pin::Pin;
/// use std::task::{Context, Poll};
///
/// # struct Counter {
/// #     count: u32,
/// # }
/// #
/// # impl Future for Counter {
/// #     type Output = u32;
/// #     
/// #     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
/// #         self.count += 1;
/// #         if self.count >= 3 {
/// #             Poll::Ready(self.count)
/// #         } else {
/// #             cx.waker().wake_by_ref();
/// #             Poll::Pending
/// #         }
/// #     }
/// # }
/// let result = sleep_on(Counter { count: 0 });
/// assert_eq!(result, 3);
/// ```
///
/// # Implementation Details
/// The executor will properly handle spurious wakeups and re-poll the future
/// as needed. The waker implementation uses a semaphore to signal readiness.
pub fn sleep_on<F: Future>(mut future: F) -> F::Output {
    //we inherit the parent dlog::context here.
    let shared = Arc::new(SimpleWakeShared {
        semaphore: Semaphore::new(false),
    });
    let local = shared.clone();
    let raw_waker = RawWaker::new(Arc::into_raw(shared) as *const (), &CONDVAR_WAKER_VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut context = Context::from_waker(&waker);
    /*
    per docs,
    any calls to notify_one or notify_all which happen logically
    after the mutex is unlocked are candidates to wake this thread

    ergo, the lock must be locked when polling.
     */
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        logwise::trace_sync!("polling future");
        if let Poll::Ready(val) = future.as_mut().poll(&mut context) {
            logwise::trace_sync!("future is ready");
            return val;
        }
        logwise::trace_sync!("future is not ready");
        local.semaphore.wait();
        logwise::trace_sync!("woken");
    }
}

/// Spawns a future on a new thread and returns immediately without waiting for completion.
///
/// This function creates a new OS thread with the given name and runs the future on that
/// thread using [`sleep_on`]. The calling thread returns immediately, making this useful
/// for fire-and-forget tasks.
///
/// # Parameters
/// - `thread_name`: The name to give to the spawned thread (must be a static string)
/// - `future`: The future to execute on the new thread
///
/// # Requirements
/// - The future must be `Send` because it will be moved to another thread
/// - The future must be `'static` because the spawned thread may outlive the caller
///
/// # Example
/// ```
/// use test_executors::spawn_on;
/// use std::sync::{Arc, Mutex};
/// use std::time::Duration;
///
/// let data = Arc::new(Mutex::new(Vec::new()));
/// let data_clone = data.clone();
///
/// spawn_on("worker", async move {
///     // Simulate some async work
///     data_clone.lock().unwrap().push(42);
/// });
///
/// // Give the spawned thread time to complete
/// std::thread::sleep(Duration::from_millis(50));
///
/// // Check the result
/// assert_eq!(*data.lock().unwrap(), vec![42]);
/// ```
///
/// # Panics
/// Panics if the thread cannot be spawned (e.g., due to resource exhaustion).
///
/// # See Also
/// - [`spawn_local`] for a platform-aware version that works on WASM
pub fn spawn_on<F: Future + Send + 'static>(thread_name: &'static str, future: F) {
    let prior_context = logwise::context::Context::current();
    let new_context = logwise::context::Context::new_task(Some(prior_context), thread_name.to_string());
    std::thread::Builder::new()
        .name(thread_name.to_string())
        .spawn(move || {
            let pushed_id = new_context.context_id();
            logwise::context::Context::set_current(new_context);

            sleep_on(future);
            logwise::context::Context::pop(pushed_id);
        })
        .expect("Cant spawn thread");
}

/// Spawns a future in a platform-appropriate way without waiting for completion.
///
/// This function automatically selects the appropriate executor based on the target platform:
/// - On native platforms (Linux, macOS, Windows, etc.): Uses [`sleep_on`] to run the future
///   on the current thread
/// - On `wasm32` targets: Uses `wasm_bindgen_futures::spawn_local` to integrate with the
///   browser's event loop
///
/// # Parameters
/// - `future`: The future to execute
/// - `_debug_label`: A label for debugging purposes (used for logging context on WASM)
///
/// # Example
/// ```
/// use test_executors::spawn_local;
///
/// spawn_local(async {
///     // This will run correctly on both native and WASM platforms
///     println!("Hello from async!");
/// }, "example_task");
/// ```
///
/// # Platform Behavior
/// ## Native Platforms
/// The future is executed immediately on the current thread using [`sleep_on`].
/// This blocks until the future completes.
///
/// ## WebAssembly
/// The future is scheduled to run on the browser's event loop and this function
/// returns immediately. The future will run when the JavaScript runtime is ready.
///
/// # Note
/// Unlike [`spawn_on`], this function does not require the future to be `Send`
/// since it may run on the current thread.
pub fn spawn_local<F: Future + 'static>(future: F, _debug_label: &'static str) {
    #[cfg(not(target_arch = "wasm32"))]
    sleep_on(future);
    #[cfg(target_arch = "wasm32")]
    {
        let c = logwise::context::Context::current();
        let new_context = logwise::context::Context::new_task(Some(c), _debug_label.to_string());
        wasm_bindgen_futures::spawn_local(async move {
            logwise::context::ApplyContext::new(new_context, future).await;
        });
    }
}

/// Polls a pinned future exactly once and returns the result.
///
/// This function is useful for testing futures or implementing custom executors.
/// It creates a no-op waker that does nothing when `wake()` is called.
///
/// # Parameters
/// - `future`: A pinned mutable reference to the future to poll
///
/// # Returns
/// - `Poll::Ready(value)` if the future completed on this poll
/// - `Poll::Pending` if the future is not yet ready
///
/// # Example
/// ```
/// use test_executors::{poll_once, pend_forever::PendForever};
/// use std::task::Poll;
///
/// let mut future = PendForever;
/// let result = poll_once(std::pin::Pin::new(&mut future));
/// assert_eq!(result, Poll::Pending);
/// ```
///
/// # Testing Example
/// ```
/// use test_executors::poll_once;
/// use std::future::Future;
/// use std::pin::Pin;
/// use std::task::{Context, Poll};
///
/// struct CounterFuture {
///     count: u32,
/// }
///
/// impl Future for CounterFuture {
///     type Output = u32;
///     
///     fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
///         self.count += 1;
///         if self.count >= 3 {
///             Poll::Ready(self.count)
///         } else {
///             Poll::Pending
///         }
///     }
/// }
///
/// let mut future = CounterFuture { count: 0 };
/// let mut pinned = std::pin::pin!(future);
///
/// assert_eq!(poll_once(pinned.as_mut()), Poll::Pending);
/// assert_eq!(poll_once(pinned.as_mut()), Poll::Pending);
/// assert_eq!(poll_once(pinned.as_mut()), Poll::Ready(3));
/// ```
///
/// # See Also
/// - [`poll_once_pin`] for a version that takes ownership and pins the future for you
pub fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    let mut context = new_context();
    future.poll(&mut context)
}

/// Polls a future exactly once after pinning it.
///
/// This is a convenience function that takes ownership of the future, pins it,
/// and polls it once. Unlike [`poll_once`], you don't need to pin the future yourself.
///
/// # Parameters
/// - `future`: The future to poll (takes ownership)
///
/// # Returns
/// - `Poll::Ready(value)` if the future completed on this poll
/// - `Poll::Pending` if the future is not yet ready
///
/// # Example
/// ```
/// use test_executors::{poll_once_pin, pend_forever::PendForever};
/// use std::task::Poll;
///
/// let future = PendForever;
/// let result = poll_once_pin(future);
/// assert_eq!(result, Poll::Pending);
/// ```
///
/// # Comparison with `poll_once`
/// ```
/// use test_executors::{poll_once, poll_once_pin};
/// use std::task::Poll;
///
/// // Using poll_once_pin (takes ownership)
/// let future1 = async { 42 };
/// assert_eq!(poll_once_pin(future1), Poll::Ready(42));
///
/// // Using poll_once (borrows)
/// let mut future2 = async { 42 };
/// let mut pinned = std::pin::pin!(future2);
/// assert_eq!(poll_once(pinned.as_mut()), Poll::Ready(42));
/// ```
///
/// # Limitations
/// Since this function takes ownership of the future, you cannot poll it again
/// after calling this function. If you need to poll a future multiple times,
/// use [`poll_once`] instead.
pub fn poll_once_pin<F: Future>(future: F) -> Poll<F::Output> {
    let mut context = new_context();
    let pinned = std::pin::pin!(future);
    pinned.poll(&mut context)
}

#[cfg(test)]
mod tests {
    use crate::pend_forever::PendForever;
    use std::future::Future;
    use std::task::Poll;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    fn test_sleep_reentrant() {
        struct F(bool);
        impl Future for F {
            type Output = ();
            fn poll(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if !self.0 {
                    self.0 = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }
        }
        let f = F(false);
        super::sleep_on(f);
    }

    #[crate::async_test]
    async fn hello_world() {
        let f = async { "hello world" };
        assert_eq!(f.await, "hello world");
    }

    #[test]
    fn poll_once_test() {
        let f = PendForever;
        let mut pinned = std::pin::pin!(f);
        let result = super::poll_once(pinned.as_mut());
        assert_eq!(result, Poll::Pending);

        let result2 = super::poll_once(pinned.as_mut());
        assert_eq!(result2, Poll::Pending);
    }
}
