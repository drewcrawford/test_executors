// SPDX-License-Identifier: MIT OR Apache-2.0

use std::any::Any;
use std::convert::Infallible;
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use some_executor::{DynExecutor, SomeExecutor, SomeExecutorExt};
use some_executor::observer::{ExecutorNotified, Observer, ObserverNotified};
use some_executor::task::Task;

/**
A runtime based on [crate::spin_on]
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpinRuntime;

impl SpinRuntime {
    pub const fn new() -> Self {
        Self
    }
}

impl SomeExecutorExt for SpinRuntime {}
impl SomeExecutor for SpinRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output>>(&mut self, task: Task<F, Notifier>) -> Observer<F::Output, Self::ExecutorNotifier>
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

    fn spawn_async<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Future<Output=Observer<F::Output, Self::ExecutorNotifier>> + Send + 'static
    where
        Self: Sized,
        F::Output: Send,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        async move {
            while spawned.poll_after() > crate::sys::time::Instant::now() {
                std::hint::spin_loop()
            }
            crate::spin_on(spawned);
            observer
        }
    }


    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Observer<Box<dyn Any + 'static + Send>, Box<dyn ExecutorNotified + 'static + Send>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());

        let (spawned, observer) = task.spawn_objsafe(self);
        while spawned.poll_after() > crate::sys::time::Instant::now() {
            std::hint::spin_loop()
        }
        crate::spin_on(spawned);
        observer
    }


    fn clone_box(&self) -> Box<DynExecutor> {
        Box::new(*self)
    }

    fn executor_notifier(&mut self) -> Option<Self::ExecutorNotifier> {
        None
    }
}

/**
A runtime based on [crate::sleep_on]
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SleepRuntime;
impl SleepRuntime {
    pub const fn new() -> Self {
        Self
    }
}
impl SomeExecutorExt for SleepRuntime {

}

impl SomeExecutor for SleepRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output>>(&mut self, task: Task<F, Notifier>) -> Observer<F::Output, Self::ExecutorNotifier>
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

    fn spawn_async<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Future<Output=Observer<F::Output, Self::ExecutorNotifier>> + Send + 'static
    where
        Self: Sized,
        F::Output: Send,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        async move {
            let now = crate::sys::time::Instant::now();
            if spawned.poll_after() > now {
                let dur = spawned.poll_after() - now;
                std::thread::sleep(dur);
            }
            crate::sleep_on(spawned);
            observer
        }
    }

    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Observer<Box<dyn Any + 'static + Send>, Box<dyn ExecutorNotified + 'static + Send>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn_objsafe(self);
        let now = crate::sys::time::Instant::now();
        if spawned.poll_after() > now {
            let dur = now.duration_since(spawned.poll_after());
            std::thread::sleep(dur);
        }
        crate::sleep_on(spawned);
        observer
    }

    fn clone_box(&self) -> Box<DynExecutor> {
        Box::new(*self)
    }

    fn executor_notifier(&mut self) -> Option<Self::ExecutorNotifier> {
        None
    }
}


/**
A runtime based on [crate::spawn_on]
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpawnRuntime;
impl SpawnRuntime {
    pub const fn new() -> Self {
        Self
    }
}
impl SomeExecutorExt for SpawnRuntime {
}


impl SomeExecutor for SpawnRuntime {
    type ExecutorNotifier = Infallible;

    fn spawn<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> Observer<F::Output, Self::ExecutorNotifier>
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

    fn spawn_async<F: Future + Send + 'static, Notifier: ObserverNotified<F::Output> + Send>(&mut self, task: Task<F, Notifier>) -> impl Future<Output=Observer<F::Output, Self::ExecutorNotifier>> + Send + 'static
    where
        Self: Sized,
        F::Output: Send,
    {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn(self);
        async move {
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

    fn spawn_objsafe(&mut self, task: Task<Pin<Box<dyn Future<Output=Box<dyn Any + 'static + Send>> + 'static + Send>>, Box<dyn ObserverNotified<dyn Any + Send> + Send>>) -> Observer<Box<dyn Any + 'static + Send>, Box<dyn ExecutorNotified + 'static + Send>> {
        logwise::info_sync!("spawned future: {label}", label=task.label());
        let (spawned, observer) = task.spawn_objsafe(self);
        std::thread::spawn(move || {
            if spawned.poll_after() > crate::sys::time::Instant::now() {
                let dur = crate::sys::time::Instant::now().duration_since(spawned.poll_after());
                std::thread::sleep(dur);
            }
            crate::sleep_on(spawned);
        });
        observer
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

/**
Sets a truntime as the global runtime.
*/
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


