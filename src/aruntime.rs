use std::fmt::Display;
use std::future::Future;
use aruntime::{ARuntime, ARuntimeObjSafe, RuntimeHint};
use priority::Priority;

/**
A runtime based on [spin_on]
*/
#[derive(Debug, Copy, Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct SpinRuntime;

impl SpinRuntime {
    pub const fn new() -> Self {
        Self
    }
}




impl ARuntime for SpinRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::spin_on(f);
    }
}
impl ARuntimeObjSafe for SpinRuntime {
    fn spawn_detached_objsafe(&mut self, _priority: Priority, _runtime_hint: RuntimeHint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        let f= Box::into_pin(f);
        crate::spin_on(f);
    }
}

/**
A runtime based on [sleep_on]
*/
#[derive(Debug, Copy, Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct SleepRuntime;
impl SleepRuntime {
    pub const fn new() -> Self {
        Self
    }
}
impl ARuntime for SleepRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::sleep_on(f);
    }
}

impl ARuntimeObjSafe for SleepRuntime {
    fn spawn_detached_objsafe(&mut self, _priority: Priority, _runtime_hint: RuntimeHint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        let f= Box::into_pin(f);
        crate::sleep_on(f);
    }
}


/**
A runtime based on [spawn_on]
*/
#[derive(Debug, Copy, Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct SpawnRuntime;
impl SpawnRuntime {
    pub const fn new() -> Self {
        Self
    }
}
impl ARuntime for SpawnRuntime {
    fn spawn_detached<F: Future + Send + 'static>(&mut self, _priority: priority::Priority, _runtime_hint: aruntime::RuntimeHint, f: F) {
        crate::spawn_on(f);
    }
}

impl ARuntimeObjSafe for SpawnRuntime {
    fn spawn_detached_objsafe(&mut self, _priority: Priority, _runtime_hint: RuntimeHint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        let f= Box::into_pin(f);
        crate::spawn_on(f);
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

#[cfg(test)] mod test {
    #[test] fn assert_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<super::SpinRuntime>();
        assert_send_sync::<super::SleepRuntime>();
        assert_send_sync::<super::SpawnRuntime>();
    }
}


