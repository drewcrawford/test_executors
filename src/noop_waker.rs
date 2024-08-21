/*!
A waker which does not do anything.  Primarily useful for testing.
*/

use std::sync::OnceLock;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE),
    |_| (),
    |_| (),
    |_| (),
);



//error: `Waker::from_raw` is not yet stable as a const fn

static NOOP_WAKER: OnceLock<Waker> = OnceLock::new();

fn noop_waker() -> &'static Waker {
    NOOP_WAKER.get_or_init(|| {
        let raw = RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE);
        unsafe { Waker::from_raw(raw) }
    })
}
/**
Creates a new context that has no effect.
*/
pub fn new_context() -> Context<'static> {
    Context::from_waker(noop_waker())
}

