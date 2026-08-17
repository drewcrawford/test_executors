# test_executors

This crate provides extremely simple, yet useful, async executors. They are primarily useful for writing unit tests
without bringing in a full-blown executor such as [tokio](https://tokio.rs).

![logo](art/logo.png)

# Quick Start

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

# Available Executors

The crate provides three main executors:

* `spin_on` - Polls a future in a busy loop on the current thread. Best for CPU-bound tasks or when latency is critical.
* `sleep_on` - Polls a future on the current thread, sleeping between polls. Best for I/O-bound tasks to avoid burning CPU.
* `spawn_on` - Spawns a future on a new thread, polling it there. Best for fire-and-forget tasks.

# Platform Support

## Native Platforms
All executors work as described above on native platforms (Linux, macOS, Windows, etc.).

## WebAssembly Support
`spawn_local` hands the future to the browser event loop via
`wasm_lite_std::spawn_local` on wasm32, and runs it on the calling thread
everywhere else.

The three blocking executors are native-only in practice. The browser main
thread may not block: `sleep_on` waits on a condition variable, which is
unavailable there; `spawn_on` needs a thread `std` cannot spawn on that target;
and `spin_on` never yields to the event loop, so anything waiting on it is
waiting forever. Use them on native, or from a Web Worker.

# Writing tests

This crate no longer supplies a test attribute. It used to export
`#[async_test]`; use
[`#[wasm_lite::wasm_lite_test]`](https://docs.rs/wasm_lite/latest/wasm_lite/attr.wasm_lite_test.html)
instead, which does the same job better and in one place:

```rust
#[wasm_lite::wasm_lite_test]
async fn my_test() {
    let value = async { 42 }.await;
    assert_eq!(value, 42);
}
```

That registers an ordinary libtest `#[test]` off wasm32 and a browser-driven
test on it, so one attribute covers both targets. Unlike the attribute this
crate used to ship, it honours `#[should_panic]` and `#[ignore]`, supports
running the body on a Web Worker, and fails a test whose future panics, hangs,
or is dropped.

What remains here is the executors themselves, for driving a future from
somewhere that is not a test entry point.

## Integration with `some_executor`
This crate implements the [some_executor](https://crates.io/crates/some_executor) trait for all executors,
allowing them to be used in executor-agnostic code:

```rust
use test_executors::aruntime::SpinRuntime;
use some_executor::SomeExecutor;

let mut runtime = SpinRuntime::new();
// Use runtime with some_executor traits
```

# Utilities

The crate also provides utility functions and types:
- `poll_once` and `poll_once_pin` - Poll a future exactly once
- `spawn_local` - Platform-aware spawning that works on both native and WASM

# License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE.md) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT.md) or <http://opensource.org/licenses/MIT>)

at your option.
