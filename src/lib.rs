/*!
A simple runtime that is useful for testing.

It blocks the current thread until the future completes.
*/

/*!
Blocks the calling thread until a future is ready.
*/

mod noop_waker;
pub mod aruntime;

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use crate::noop_waker::new_context;

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
    }
}

struct SimpleWakeShared {
    condvar: Condvar,
    mutex: std::sync::Mutex<()>,
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
        dlog::perfwarn!("truntime mutex", {
            let guard = ctx.mutex.lock().unwrap();
            ctx.condvar.notify_all();
            drop(guard);
        });

    },
    |ctx| {
        let ctx = unsafe{Arc::from_raw(ctx as *const SimpleWakeShared)};
        dlog::perfwarn!("truntime mutex", {
            let guard = ctx.mutex.lock().unwrap();
            ctx.condvar.notify_one();
            drop(guard);
        });
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
    let shared = Arc::new(SimpleWakeShared{condvar: Condvar::new(), mutex: std::sync::Mutex::new(())});
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
    let mut guard = local.mutex.lock().unwrap();

    loop {
        if let Poll::Ready(val) = future.as_mut().poll(&mut context) {
            return val;
        }
        guard = local.condvar.wait(guard).unwrap();
    }
}

/**
A function that spawns the given future and does not wait for it to complete.
*/
pub fn spawn_on<F: Future + Send + 'static>(future: F) {
    let prior_context = dlog::context::Context::current_clone();

    std::thread::spawn(move || {
        let pushed_id = if let Some(prior_context) = prior_context {
            let new_context = dlog::context::Context::new_task(Some(prior_context));
            let new_context_id = new_context.context_id();
            dlog::context::Context::set_current(new_context);
            Some(new_context_id)
        }
        else {
            None
        };
        sleep_on(future);
        if let Some(pushed_id) = pushed_id {
            dlog::context::Context::pop(pushed_id);
        }
    });
}

/**
Poll the given future once.
*/
pub fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    let mut context = new_context();
    future.poll(&mut context)
}