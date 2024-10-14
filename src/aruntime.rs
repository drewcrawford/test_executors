// SPDX-License-Identifier: MIT OR Apache-2.0
use std::fmt::Display;
use std::future::Future;
use std::time::Instant;
use some_executor::{Hint, SomeExecutor, SomeExecutorObjSafe};
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




impl SomeExecutor for SpinRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, label: &'static str, _priority: priority::Priority, _runtime_hint: Hint, f: F) {
        logwise::info_sync!("spawned future: {label}", label=label);
        crate::spin_on(f);
    }
    fn spawn_detached_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_detached(label, priority, runtime_hint, f);
        }
    }

    fn spawn_after<F: Future + Send + 'static>(&mut self, label: &'static str, _priority: Priority, _runtime_hint: Hint, time: Instant, f: F) {
        let now = Instant::now();
        if now < time {
            let dur = time - now;
            std::thread::sleep(dur);
        }
        assert!(Instant::now() >= time);
        logwise::info_sync!("spawned future: {label}", label=label);
        crate::spin_on(f);
    }
    fn spawn_after_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, time: Instant, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_after(label, priority, runtime_hint, time, f);
        }
    }
    fn to_objsafe_runtime(self) -> Box<dyn SomeExecutorObjSafe> {
        Box::new(self)
    }
}
impl SomeExecutorObjSafe for SpinRuntime {
    fn spawn_detached_objsafe(&self, label: &'static str, _priority: Priority, _runtime_hint: Hint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        logwise::info_sync!("spawned future: {label}", label=label);
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
impl SomeExecutor for SleepRuntime {
    fn spawn_detached<F: Future + Send>(&mut self, label: &'static str, _priority: priority::Priority, _runtime_hint: Hint, f: F) {
        logwise::info_sync!("spawned future: {label}", label=label);
        crate::sleep_on(f);
    }
    fn spawn_after<F: Future + Send + 'static>(&mut self, label: &'static str, _priority: Priority, _runtime_hint: Hint, time: Instant, f: F) {
        let now = Instant::now();
        if now < time {
            let dur = time - now;
            std::thread::sleep(dur);
        }
        assert!(Instant::now() >= time);
        logwise::info_sync!("spawned future: {label}", label=label);
        crate::sleep_on(f);
    }
    fn spawn_detached_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_detached(label, priority, runtime_hint, f);
        }
    }

    fn spawn_after_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, time: Instant, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_after(label, priority, runtime_hint, time, f);
        }
    }

    fn to_objsafe_runtime(self) -> Box<dyn SomeExecutorObjSafe> {
        Box::new(self)
    }

}

impl SomeExecutorObjSafe for SleepRuntime {
    fn spawn_detached_objsafe(&self, label: &'static str, _priority: Priority, _runtime_hint: Hint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        logwise::info_sync!("spawned future: {label}", label=label);
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
impl SomeExecutor for SpawnRuntime {

    fn spawn_detached<F: Future + Send + 'static>(&mut self,label: &'static str, _priority: priority::Priority, _runtime_hint: Hint, f: F) {
        let block = async move {
            logwise::info_async!("spawned future: {label}", label=label);
            f.await;
        };
        crate::spawn_on(label, block);
    }

    fn to_objsafe_runtime(self) -> Box<dyn SomeExecutorObjSafe> {
        Box::new(self)
    }

    fn spawn_after<F: Future + Send + 'static>(&mut self, label: &'static str, _priority: Priority, _runtime_hint: Hint, time: Instant, f: F) {
        crate::spawn_on(label, async move {
            if Instant::now() < time {
                let dur = time - Instant::now();
                std::thread::sleep(dur);
            }
            assert!(Instant::now() >= time);
            logwise::info_async!("spawned future: {label}", label=label);
            f.await;
        })
    }
    fn spawn_after_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, time: Instant, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_after(label, priority, runtime_hint, time, f);
        }
    }
    fn spawn_detached_async<F: Future + Send + 'static>(&mut self, label: &'static str, priority: Priority, runtime_hint: Hint, f: F) -> impl Future<Output=()> {
        async move {
            self.spawn_detached(label, priority, runtime_hint, f);
        }
    }
}

impl SomeExecutorObjSafe for SpawnRuntime {
    fn spawn_detached_objsafe(&self, label: &'static str, _priority: Priority, _runtime_hint: Hint, f: Box<dyn Future<Output=()> + Send + 'static>) {
        logwise::info_sync!("spawned future: {label}", label=label);
        let f= Box::into_pin(f);
        crate::spawn_on(label, f);
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

/**
Sets a truntime as the global runtime.
*/
pub fn set_global_test_runtime() {
    some_executor::set_global_runtime(SpawnRuntime.to_objsafe_runtime())
}
#[cfg(test)] mod test {
    #[test] fn assert_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<super::SpinRuntime>();
        assert_send_sync::<super::SleepRuntime>();
        assert_send_sync::<super::SpawnRuntime>();
    }
}


