# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

test_executors is a Rust crate that provides simple async executors primarily for testing purposes. It offers three main executors (spin_on, sleep_on, spawn_on) and integrates with the some_executor ecosystem.

## Common Development Commands

### Using Helper Scripts (Recommended)
The project includes helper scripts in `scripts/` that handle platform-specific flags:

```bash
# Run all checks (fmt, check, clippy, tests, docs) for both native and wasm32
./scripts/check_all

# Individual checks (run both native and wasm32)
./scripts/check       # cargo check
./scripts/clippy      # cargo clippy --no-deps
./scripts/tests       # cargo test
./scripts/docs        # cargo doc
./scripts/fmt         # cargo fmt --check

# Platform-specific scripts
./scripts/native/tests    # Native tests only
./scripts/wasm32/tests    # WASM tests only (drives a real browser via the wasm_lite runner)
```

### Manual Commands
```bash
# Run all tests (native)
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run WASM tests (requires nightly + wasm32-unknown-unknown target + a WebDriver
# browser). `.cargo/config.toml` already points the runner at `wasm_lite run`.
./scripts/wasm32/tests
# Or manually:
cargo +nightly test --target wasm32-unknown-unknown
```

## Architecture Overview

One package. `test_executors_proc` used to sit alongside it holding the
`#[async_test]` attribute; both are gone as of 0.6.0 — see "Writing tests" below.

1. **Core Executors** (src/lib.rs):
   - `spin_on`: Busy-loop executor for minimal latency, uses 100% CPU
   - `sleep_on`: Condition-variable executor (`Mutex` + `Condvar`) that sleeps between polls
   - `spawn_on`: Thread-spawning executor for parallel execution on new OS thread

2. **Runtime Module** (src/aruntime.rs):
   - Provides `SpinRuntime`, `SleepRuntime`, and `SpawnRuntime` structs
   - Implements the `some_executor::SomeExecutor` trait for all runtimes
   - Global executor management via `set_global_test_runtime()` and `get_test_runtime()`
   - Each runtime wraps its corresponding executor for trait-based usage

3. **Utility Modules**:
   - `noop_waker.rs`: Provides a no-op waker that does nothing when wake() is called

## Writing tests

This crate supplies executors, not test entry points. Its own tests use
`#[wasm_lite::wasm_lite_test]`, which registers a libtest `#[test]` off wasm32
and a browser-driven test on it — one attribute, both targets, `async fn`
included. Do not reintroduce a wrapper attribute here: the one it used to have
(`#[async_test]`, removed in 0.6.0) silently dropped `#[should_panic]` and
`#[ignore]`, and every feature wasm_lite added had to be re-plumbed through it.

`wasm_lite`/`wasm_lite_std` are dev-dependencies by **path** for now, because the
entry-point behaviour used here landed after wasm_lite 0.1.1 and is not published
yet. Restore registry requirements once it is.

## Key Design Decisions

- **Waker Implementation**: `spin_on` uses a no-op waker, while `sleep_on` uses a condition variable (Mutex + Condvar with "sticky" wake flag) for efficient blocking
- **Platform Abstraction**: `spawn_local` automatically chooses between native thread blocking and WASM event loop integration
- **Logging Context**: `spawn_on` and `spawn_local` create a named `logwise::context::Context` task for the work they start. `spin_on`/`sleep_on` do not — they inherit whatever context the calling thread already has, and only emit trace records
- **some_executor Integration**: All runtimes implement the SomeExecutor trait to enable executor-agnostic async code
- **Testing Focus**: Designed for unit tests without heavyweight runtime dependencies like tokio
- **Rust Edition 2024**: This crate requires Rust 1.95.0+ (edition 2024), the floor `some_executor` 0.7 imposes

## CI/CD Pipeline

The project uses GitHub Actions (`.github/workflows/ci.yaml`), building for native and wasm32.

All warnings are treated as errors via `RUSTFLAGS="-D warnings"`. Use `./scripts/check_all` locally to run the same checks as CI.