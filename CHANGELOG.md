# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
