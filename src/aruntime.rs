use std::future::Future;
use aruntime::ARuntime;

/**
A runtime based on [spin_on]
*/
pub struct SpinRuntime;
impl ARuntime for SpinRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::spin_on(f);
    }
}

/**
A runtime based on [sleep_on]
*/
pub struct SleepRuntime;
impl ARuntime for SleepRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::sleep_on(f);
    }
}


/**
A runtime based on [spawn_on]
*/
pub struct SpawnRuntime;
impl ARuntime for SpawnRuntime {
    fn spawn_detached<F: Future + Send + 'static>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::spawn_on(f);
    }
}