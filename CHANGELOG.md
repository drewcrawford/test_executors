# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-08-16

### Removed
- **`#[async_test]` is gone, and `test_executors_proc` with it.** Use [`#[wasm_lite::wasm_lite_test]`](https://docs.rs/wasm_lite/latest/wasm_lite/attr.wasm_lite_test.html) instead: it registers a libtest `#[test]` off wasm32 and a browser-driven test on it, so one attribute covers both targets, and it takes an `async fn` directly. Migration is usually a one-line swap.

  The reason is that our version was the worse of the two and getting worse. It silently dropped `#[should_panic]` and `#[ignore]` — they landed on the inner async function, which is not the test, so an ignored test ran anyway and a correct `should_panic` test reported as failing. It quietly accepted and discarded arguments like `(worker)`. It renamed your test to `async_test_<name>`. And because its wasm32 half was a forwarder, every capability wasm_lite grew had to be re-plumbed through it before anyone here could use it. On the wasm32 side there was nothing left that was ours; on the native side the value was `sleep_on`, which is still right here and still exported.

  `test_executors_proc` existed only to hold that one macro, so it is retired rather than left as an empty shell.

### Changed
- **The crate's own tests moved to `#[wasm_lite::wasm_lite_test]`**, including the ones that used the `#[cfg_attr(not(target_arch = "wasm32"), test)]` pairing — one attribute now does both. `wasm_lite` and `wasm_lite_std` are dev-dependencies on every target rather than wasm32 only, since the native half of an `async fn` test is `wasm_lite_std::block_on`.
- **`wasm_lite` and `wasm_lite_std` moved to crates.io 0.1.2.** Both entry-point behaviours above landed after 0.1.1 and were unpublished when this crate first adopted them, so the manifest pointed at a sibling checkout via `[patch.crates-io]`. They're published now, so the patch is gone and the dependencies are ordinary registry requirements again.
- **Documented what the executors actually do on wasm32.** The README and crate docs claimed `#[async_test]` adapted to `wasm-bindgen-test`, which stopped being true in 0.5.0. They now say the useful thing instead: `spin_on`, `sleep_on` and `spawn_on` are native-only in practice, because the browser main thread has no condition variable to wait on, no thread for `std` to spawn, and no tolerance for a loop that never yields to the event loop.

## [0.5.0] - 2026-08-15

### Changed
- Building from a checkout now patches `some_executor`, `logwise`, `wasm_lite` and `wasm_lite_std` to the sibling directories next door. This is a development convenience with something sharp behind it: `wasm_lite` exports `#[no_mangle]` symbols, and our graph could end up holding two copies of it — `logwise` reaches it by path, while we and `some_executor` took it from the registry. That doesn't duplicate quietly; it fails to link on wasm32 with "duplicate symbol". One copy now, everywhere. No version requirement moved, and `[patch]` never reaches anyone depending on us from crates.io.
- `wasm_lite_std` and `wasm_lite` now come from crates.io at 0.1.1 instead of a path next door. They were unpublished when we first adopted them, so the manifest pointed at a sibling checkout — which meant nobody without that exact directory layout could build the crate, CI included. They're published now, so we just depend on them like anything else.
- `wasm_lite_std` moved to the wasm32-only dependency table, where it belonged all along: `spawn_local` is its one caller and it's wasm32-only. Native builds now don't compile it at all. Nice side effect — it declares an MSRV of 1.95.0, so while it sat in the shared table it quietly dragged the whole crate up with it, and our advertised 1.85.1 was fiction on every platform. The declared floor is now 1.95.0 rather than 1.85.1 — moving `wasm_lite_std` does not actually buy back the older toolchain, because `some_executor` 0.7 requires 1.95.0 and we depend on it unconditionally.

### Fixed
- Fixed a race in `SpawnRuntime::spawn_async` that could make a task vanish. It read the clock twice — once to decide whether the task's `poll_after` deadline was still ahead, once to work out how long to sleep — and if the deadline slipped past between those two reads, subtracting the later instant panicked on the spawned thread. The task never ran, and anyone awaiting its observer waited forever. Every deadline check now goes through `checked_duration_since`, which can't race and can't panic.
- **The crate didn't compile for wasm32 at all.** Our runtimes compare a task's `poll_after()` deadline against the current time, and we'd quietly ended up reading those two from different clocks — `poll_after()` hands back a `some_executor::Instant`, while we were calling `now()` on `wasm_lite_std`'s. Same type on native, two unrelated types on wasm32, fifteen compile errors. We now read the clock `poll_after()` is actually measured in, which also means the `sys` module has nothing left to abstract and is gone.
- With wasm32 building again, the doctests could finally run there — and seven of them promptly hung the browser. `spin_on`, `sleep_on` and `spawn_on` all block the calling thread, which is exactly what the wasm32 main thread won't tolerate, so those examples now run on native only. The `#[async_test]` example needed a nudge of its own: it registers through a custom wasm section that rustdoc's merged doctest bundle never drives, so it opts out of the merge. The whole wasm32 suite is green again.
- CI was still installing `wasm-bindgen-cli` for the wasm32 job, left over from before we dropped wasm-bindgen. It now installs `wasm_lite_cli`, which supplies the `wasm_lite` binary our test runner actually invokes.
- The wasm32 test tooling was still reaching for `wasm-bindgen-test-runner` even though we'd dropped wasm-bindgen entirely. It can't load these binaries anymore, so `./scripts/wasm32/tests` now hands them to the `wasm_lite` runner instead. Also swept out a leftover `wasm-bindgen-test` dependency in `test_executors_proc` that hadn't done anything for a while.
- `#[async_test]` now targets `wasm_lite` rather than wasm-bindgen, and its `cfg` gating actually gates — the attributes were being emitted in a position where they selected nothing, so both test entry points were compiled on every target.
- `resolve()` in the proc macro treated `FoundCrate::Itself` as the crate's own name when it means `crate`, and looked up `wasm-bindgen-test` under the wrong package name. Either one produced a macro expansion that referred to a path that wasn't there.
- `#[async_test]` shipped with no documentation. The long write-up meant for it was sitting one line too high in the file, so it landed on the private helper underneath instead — which meant docs.rs showed the macro bare and the explanation went nowhere. Put back where it belongs.
- The logo in the crate docs pointed at a relative path, which resolves locally and to nothing on docs.rs. It's an absolute URL now; the README keeps the repo-relative one, which is the right form for each.
- The README's license links pointed at `LICENSE-APACHE` and `LICENSE-MIT`, neither of which exists — the files are `.md`.

### Added
- `test_executors_proc` has a crate-level doc header explaining why the macro emits two different test entry points instead of one, and that you want `test_executors` rather than this crate directly.
- The crate docs carry a License section, so the licensing terms are visible on docs.rs and not only in the repo.

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
