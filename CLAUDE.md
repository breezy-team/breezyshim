# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name`
- Run with features: `cargo test --features debian,launchpad`
- Lint/format: `cargo fmt` and `cargo clippy`

## Code Style Guidelines
- Use 4-space indentation
- Follow Rust naming conventions: snake_case for functions/variables, PascalCase for types
- Use doc comments `//!` for modules and `///` for public items
- Organize imports: std first, then external crates, then internal modules
- Use the `wrapped_py!` macro for Python object wrappers
- Error handling: Use crate::Result<T> type for functions that can fail
- Feature flags in `#[cfg(feature = "...")]` for optional functionality
- Tests in a `#[cfg(test)] mod tests {}` block with the `#[test]` attribute
- Use the `serial_test::serial` attribute for tests that can't run in parallel