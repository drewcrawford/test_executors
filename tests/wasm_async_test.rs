// Test file to reproduce async_test bug on wasm32

use test_executors::async_test;

#[async_test]
async fn simple_async_test() {
    let result = async { 42 }.await;
    assert_eq!(result, 42);
}

#[async_test]
async fn async_test_with_assertion() {
    let value = async {
        // Simulate some async work
        "hello from async test"
    }
    .await;

    assert_eq!(value, "hello from async test");
}
