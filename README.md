# test_executors

![logo](art/logo.png)

This crate provides extremely simple, yet useful, async executors. They are primarily useful for writing unit tests
without bringing in a full-blown executor such as [tokio](https://tokio.rs).

## Quick Start

```rust
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

## Available Executors

The crate provides three main executors:

### `spin_on`
Polls a future in a busy loop on the current thread. Best for CPU-bound tasks or when latency is critical.

**When to Use:**
- When you need minimal latency
- For CPU-bound async tasks
- In tests where you want deterministic behavior
- When the future is expected to complete quickly

**Performance Note:** This executor will consume 100% CPU while waiting. For I/O-bound tasks or long-running futures, consider using `sleep_on` instead.

```rust
use test_executors::spin_on;

let result = spin_on(async {
    // Simulate some async work
    let value = async { 21 }.await;
    value * 2
});
assert_eq!(result, 42);
```

### `sleep_on`
Polls a future on the current thread, sleeping between polls. Best for I/O-bound tasks to avoid burning CPU.

**When to Use:**
- For I/O-bound async tasks
- When you want to avoid burning CPU cycles
- For longer-running futures
- In tests that involve actual async I/O or timers

**Implementation Details:** The executor will properly handle spurious wakeups and re-poll the future as needed. The waker implementation uses a semaphore to signal readiness.

```rust
use test_executors::sleep_on;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Counter {
    count: u32,
}

impl Future for Counter {
    type Output = u32;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        if self.count >= 3 {
            Poll::Ready(self.count)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

let result = sleep_on(Counter { count: 0 });
assert_eq!(result, 3);
```

### `spawn_on`
Spawns a future on a new thread, polling it there. Best for fire-and-forget tasks.

This function creates a new OS thread with the given name and runs the future on that thread using `sleep_on`. The calling thread returns immediately, making this useful for fire-and-forget tasks.

**Requirements:**
- The future must be `Send` because it will be moved to another thread
- The future must be `'static` because the spawned thread may outlive the caller

```rust
use test_executors::spawn_on;
use std::sync::{Arc, Mutex};
use std::time::Duration;

let data = Arc::new(Mutex::new(Vec::new()));
let data_clone = data.clone();

spawn_on("worker", async move {
    // Simulate some async work
    data_clone.lock().unwrap().push(42);
});

// Give the spawned thread time to complete
std::thread::sleep(Duration::from_millis(50));

// Check the result
assert_eq!(*data.lock().unwrap(), vec![42]);
```

## Platform Support

### Native Platforms
All executors work as described above on native platforms (Linux, macOS, Windows, etc.).

### WebAssembly Support
This crate has special support for `wasm32` targets:
- The `async_test` macro automatically adapts to use `wasm-bindgen-test` on WASM
- `spawn_local` uses `wasm_bindgen_futures::spawn_local` on WASM targets

## Features

### `async_test` Macro
The `async_test` macro allows you to write async tests that work on both native and WASM targets:

```rust
use test_executors::async_test;

#[async_test]
async fn my_test() {
    let value = async { 42 }.await;
    assert_eq!(value, 42);
}
```

### Integration with `some_executor`
This crate implements the [some_executor](https://crates.io/crates/some_executor) trait for all executors,
allowing them to be used in executor-agnostic code:

```rust
use test_executors::aruntime::SpinRuntime;
use some_executor::SomeExecutor;

let mut runtime = SpinRuntime::new();
// Use runtime with some_executor traits
```

## Utilities

The crate also provides utility functions and types:

### `spawn_local`
Platform-aware spawning that works on both native and WASM platforms.

This function automatically selects the appropriate executor based on the target platform:
- On native platforms: Uses `sleep_on` to run the future on the current thread
- On `wasm32` targets: Uses `wasm_bindgen_futures::spawn_local` to integrate with the browser's event loop

```rust
use test_executors::spawn_local;

spawn_local(async {
    // This will run correctly on both native and WASM platforms
    println!("Hello from async!");
}, "example_task");
```

**Platform Behavior:**
- **Native Platforms:** The future is executed immediately on the current thread using `sleep_on`. This blocks until the future completes.
- **WebAssembly:** The future is scheduled to run on the browser's event loop and this function returns immediately.

### `poll_once` and `poll_once_pin`
Poll a future exactly once - useful for testing futures or implementing custom executors.

#### `poll_once`
Polls a pinned future exactly once and returns the result.

```rust
use test_executors::poll_once;
use std::task::Poll;

let mut future = std::future::pending::<()>();
let result = poll_once(std::pin::Pin::new(&mut future));
assert_eq!(result, Poll::Pending);
```

#### `poll_once_pin`
Polls a future exactly once after pinning it (convenience function that takes ownership).

```rust
use test_executors::poll_once_pin;
use std::task::Poll;

let future = std::future::pending::<()>();
let result = poll_once_pin(future);
assert_eq!(result, Poll::Pending);
```