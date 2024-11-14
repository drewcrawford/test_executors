# test_executors

![logo](art/logo.png)

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