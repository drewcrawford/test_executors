# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Fixed a race in `SpawnRuntime::spawn_async` that could make a task vanish. It read the clock twice — once to decide whether the task's `poll_after` deadline was still ahead, once to work out how long to sleep — and if the deadline slipped past between those two reads, subtracting the later instant panicked on the spawned thread. The task never ran, and anyone awaiting its observer waited forever. Every deadline check now goes through `checked_duration_since`, which can't race and can't panic.
- **The crate didn't compile for wasm32 at all.** Our runtimes compare a task's `poll_after()` deadline against the current time, and we'd quietly ended up reading those two from different clocks — `poll_after()` hands back a `some_executor::Instant`, while we were calling `now()` on `wasm_lite_std`'s. Same type on native, two unrelated types on wasm32, fifteen compile errors. We now read the clock `poll_after()` is actually measured in, which also means the `sys` module has nothing left to abstract and is gone.
- The wasm32 test tooling was still reaching for `wasm-bindgen-test-runner` even though we'd dropped wasm-bindgen entirely. It can't load these binaries anymore, so `./scripts/wasm32/tests` now hands them to the `wasm_lite` runner instead. Also swept out a leftover `wasm-bindgen-test` dependency in `test_executors_proc` that hadn't done anything for a while.

## [0.4.1] - 2025-12-20

### Changed
- Freshened up our dependencies to their latest versions. Behind the scenes, we upgraded to `some_executor` 0.6.2, `logwise` 0.5.0, and `test_executors_proc` 0.3.5. For our WASM friends, we also pinned exact versions of `wasm-bindgen`, `web-time`, `wasm-bindgen-futures`, and `wasm-bindgen-test` to keep everything playing nicely together.
- Relaxed the minimum supported Rust version (MSRV) from 1.88.0 to 1.85.1, making it easier for more projects to adopt test_executors.

### Added
- Added optional `logwise_internal` feature for internal logging support.

## [0.4.0] - 2025-11-28

### Changed
- Ditched the `blocking_semaphore` dependency in favor of standard library primitives (Condvar + Mutex). Your builds just got a little lighter, and we solved the "wake-while-locked" puzzle using a clever boolean flag that makes wake notifications "sticky." If someone rings the doorbell before you're waiting, you'll still know they stopped by.
- Upgraded to `logwise` 0.4.0 for better logging goodness behind the scenes.

### Removed
- **Breaking**: Waved goodbye to the `pend_forever` module. Turns out the standard library had what we needed all along. If you were using `pend_forever::PendForever`, you'll need to switch to `std::future::pending()` instead—same behavior, fancier address.

### Added
- Expanded documentation to help you navigate the executor landscape more easily.
- Added SPDX license identifier because we like to keep things properly labeled.
- Beefed up the test suite to keep everything running smoothly.
- Introduced AGENTS.md for those curious about our AI-assisted development workflow.

### Fixed
- CI pipeline now knows what it's doing (we gave it a pep talk and some new configs).

## [0.3.5] - 2025-11-27

Previous releases were not documented in this changelog.
