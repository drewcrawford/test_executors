// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration-test coverage for async test entry points on both targets.
//!
//! This crate no longer supplies the attribute — `#[wasm_lite::wasm_lite_test]`
//! registers a libtest `#[test]` off wasm32 and a browser-driven test on it, so
//! one spelling covers both and there is nothing here for `test_executors` to
//! wrap. What is worth keeping is the coverage: an async body driven to
//! completion from an integration-test target, which is where the old
//! `#[async_test]` had its wasm32 bugs.

#[wasm_lite::wasm_lite_test]
async fn simple_async_test() {
    let result = async { 42 }.await;
    assert_eq!(result, 42);
}

#[wasm_lite::wasm_lite_test]
async fn async_test_with_assertion() {
    let value = async {
        // Simulate some async work
        "hello from async test"
    }
    .await;

    assert_eq!(value, "hello from async test");
}
