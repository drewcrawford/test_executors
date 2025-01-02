// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
This crate provides extremely simple, yet useful, async executors.  They are primarily useful for writing unit tests
without bringing in a full-blown executor such as [tokio](https://tokio.rs).

The executors are:
* spin_on: polls a future in a busyloop on the current thread.
* sleep_on: polls a future on the current thread, sleeping between polls.
* spawn_on: spawns a future on a new thread, polling it there.

# some_executor

This crate implements the [some_executor](https://crates.io/crates/some_executor) trait for all executors, allowing them
to be used in executor-agnostic code.

# `async_test`
This crate provides a macro, `async_test`, allowing tests to be used with async functions, including support
for wasm32 targets.

*/

/*!
Blocks the calling thread until a future is ready.
*/

mod noop_waker;
pub mod aruntime;
pub mod pend_forever;
mod sys;

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use blocking_semaphore::one::Semaphore;
use crate::noop_waker::new_context;

pub use test_executors_proc::async_test;

extern crate self as test_executors;

/**
Blocks the calling thread until a future is ready.

This implementation uses a spinloop.
*/
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
    |ctx|{
        let ctx = unsafe{Arc::from_raw(ctx as *const SimpleWakeShared)};
        let ctx2 = ctx.clone();
        std::mem::forget(ctx);
        RawWaker::new(Arc::into_raw(ctx2) as *const (), &CONDVAR_WAKER_VTABLE)
    },
    |ctx| {
        let ctx = unsafe{Arc::from_raw(ctx as *const SimpleWakeShared)};
        logwise::trace_sync!("waking");
        ctx.semaphore.signal_if_needed();
    },
    |ctx| {
        let ctx = unsafe{Arc::from_raw(ctx as *const SimpleWakeShared)};
        logwise::trace_sync!("waking (by ref)");
        ctx.semaphore.signal_if_needed();
        std::mem::forget(ctx);
    },
    |ctx| {
        let ctx = unsafe{Arc::from_raw(ctx as *const SimpleWakeShared)};
        std::mem::drop(ctx);
    },
);
/**
Blocks the calling thread until a future is ready.

This implementation uses a condvar to sleep the thread.
*/
pub fn sleep_on<F: Future>(mut future: F) -> F::Output {
    //we inherit the parent dlog::context here.
    let shared = Arc::new(SimpleWakeShared{semaphore: Semaphore::new(false)});
    let local = shared.clone();
    let raw_waker = RawWaker::new(Arc::into_raw(shared) as *const (), &CONDVAR_WAKER_VTABLE);
    let waker = unsafe{Waker::from_raw(raw_waker)};
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

/**
A function that spawns the given future and does not wait for it to complete.
*/
pub fn spawn_on<F: Future + Send + 'static>(thread_name: &'static str, future: F) {
        let prior_context = logwise::context::Context::current();
        let new_context = logwise::context::Context::new_task(Some(prior_context), thread_name);
        std::thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                let pushed_id = new_context.context_id();
                logwise::context::Context::set_current(new_context);

                sleep_on(future);
                logwise::context::Context::pop(pushed_id);
            }).expect("Cant spawn thread");
}

/**
Spawns the future in a platform-appropriate way.

On most platforms, this uses [sleep_on].
On wasm32, it uses [wasm_bindgen_futures::spawn_local].
*/
pub fn spawn_local<F: Future + 'static>(future: F, debug_label: &'static str) {
    #[cfg(not(target_arch = "wasm32"))]
    sleep_on(future);
    #[cfg(target_arch = "wasm32")] {
        let c = logwise::context::Context::current();
        let new_context = logwise::context::Context::new_task(Some(c), debug_label);
        wasm_bindgen_futures::spawn_local(async move { logwise::context::ApplyContext::new(new_context, future).await; });
    }

}

/**
Poll the given future once.

# Example
```
use test_executors::pend_forever::PendForever;
let mut future = PendForever;
let result = test_executors::poll_once(std::pin::Pin::new(&mut future));
```
*/
pub fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    let mut context = new_context();
    future.poll(&mut context)
}

/**
Poll the given future once.

This is a convenience function that pins the future for you.

# Example
```
use test_executors::pend_forever::PendForever;
let mut future = PendForever;
let result = test_executors::poll_once_pin(future);
```

The main drawback of this function is that by transferring ownership of the future to the function, you lose the ability to poll the future again.
*/
pub fn poll_once_pin<F: Future>(future: F) -> Poll<F::Output> {
    let mut context = new_context();
    let pinned = std::pin::pin!(future);
    let output = pinned.poll(&mut context);
    output
}

#[cfg(test)] mod tests {
    use std::future::Future;
    use std::task::Poll;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[test] fn test_sleep_reentrant() {
        struct F(bool);
        impl Future for F {
            type Output = ();
            fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
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



    #[crate::async_test] async fn hello_world() {
        let f = async {
            "hello world"
        };
        assert_eq!(f.await, "hello world");
    }
}