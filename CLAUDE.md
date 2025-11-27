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
./scripts/wasm32/tests    # WASM tests only (uses wasm-bindgen-test-runner)
```

### Manual Commands
```bash
# Run all tests (native)
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run WASM tests (requires nightly + wasm32-unknown-unknown target)
./scripts/wasm32/tests
# Or manually:
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER="wasm-bindgen-test-runner" \
  cargo +nightly test --target wasm32-unknown-unknown
```

## Architecture Overview

The crate consists of two packages:

### Main Crate (`test_executors`)
Located in the root directory, this provides the core functionality:

1. **Core Executors** (src/lib.rs):
   - `spin_on`: Busy-loop executor for minimal latency, uses 100% CPU
   - `sleep_on`: Condition variable-based executor using blocking_semaphore for CPU efficiency
   - `spawn_on`: Thread-spawning executor for parallel execution on new OS thread

2. **Runtime Module** (src/aruntime.rs):
   - Provides `SpinRuntime`, `SleepRuntime`, and `SpawnRuntime` structs
   - Implements the `some_executor::SomeExecutor` trait for all runtimes
   - Global executor management via `set_global_test_runtime()` and `get_test_runtime()`
   - Each runtime wraps its corresponding executor for trait-based usage

3. **Utility Modules**:
   - `noop_waker.rs`: Provides a no-op waker that does nothing when wake() is called
   - `sys.rs`: Platform-specific time abstractions (different implementations for native vs WASM)

### Proc Macro Crate (`test_executors_proc`)
Located in test_executors_proc/, provides the `#[async_test]` attribute macro:
- On native platforms: Wraps test in `sleep_on` executor
- On WASM targets: Uses `wasm_bindgen_test` for browser integration
- Automatically handles platform differences transparently

## Key Design Decisions

- **Waker Implementation**: `spin_on` uses a no-op waker, while `sleep_on` uses a condition variable (Mutex + Condvar with "sticky" wake flag) for efficient blocking
- **Platform Abstraction**: `spawn_local` automatically chooses between native thread blocking and WASM event loop integration
- **Logging Context**: All executors preserve logwise context across async boundaries using `logwise::context::Context`
- **some_executor Integration**: All runtimes implement the SomeExecutor trait to enable executor-agnostic async code
- **Testing Focus**: Designed for unit tests without heavyweight runtime dependencies like tokio
- **Rust Edition 2024**: This crate requires Rust 1.88.0+ (edition 2024)

## CI/CD Pipeline

The project uses GitHub Actions (`.github/workflows/ci.yaml`) with a matrix build:

**Native target (stable):**
- `cargo fmt --check`
- `cargo check`
- `cargo clippy --no-deps`
- `cargo doc`
- `cargo test`

**WASM target (nightly):**
- `cargo +nightly fmt --check`
- `cargo +nightly check --target wasm32-unknown-unknown`
- `cargo +nightly clippy --no-deps --target wasm32-unknown-unknown`
- `cargo +nightly doc --target wasm32-unknown-unknown`
- `cargo +nightly test --target wasm32-unknown-unknown`

All warnings are treated as errors via `RUSTFLAGS="-D warnings"`