# Changelog

## 0.2.0

- Renamed project to **Nexus**.
- Added library crate (`src/lib.rs`) with a production-oriented `Kernel`.
- Implemented bounded mailboxes, send/handle timeouts, and structured errors.
- Added graceful shutdown with task tracking.
- Replaced `println!` with structured logging (`tracing`).
- Added CI (fmt, clippy, tests, release build).
- Added integration tests and Dockerfile.
