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
This crate has special support for `wasm32` targets:
- The `async_test` macro automatically adapts to use `wasm-bindgen-test` on WASM
- `spawn_local` uses `wasm_bindgen_futures::spawn_local` on WASM targets

# Features

## `async_test` Macro
The `async_test` macro allows you to write async tests that work on both native and WASM targets:

```rust
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

## Documentation

For detailed API documentation including all function signatures, examples, and implementation details, see the [docs.rs documentation](https://docs.rs/test_executors).

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
