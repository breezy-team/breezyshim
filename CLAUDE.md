# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust shim around the Breezy Python API that provides Rust bindings for version control operations. The crate wraps Python objects using PyO3 to expose Breezy's functionality to Rust code while the main Breezy project is being ported to Rust.

## Build and Test Commands

- `cargo build` - Build the project
- `cargo build --features debian` - Build with Debian-specific features
- `cargo test` - Run tests (basic features)
- `cargo test --features debian` - Run tests with Debian features
- `cargo clippy` - Run linter
- `cargo doc` - Generate documentation

## Core Architecture

**Python-Rust Bridge Pattern:**
- All core types (`Branch`, `Tree`, `Transport`, `Forge`) are thin wrappers around `PyObject`
- Use `Python::with_gil(|py| ...)` for all Python interactions
- Clone operations use `clone_ref()` method for Python object references

**Key Abstractions:**
- `Branch`: Represents a named sequence of revisions with format and stacking support
- `Tree`: File system objects with `Kind` enum (File/Directory/Symlink/TreeReference)
- `Transport`: URL-based content access with automatic local/remote detection
- `Forge`: Code hosting service integration for merge proposals and repository management

**VCS Backend Organization:**
- Each VCS has dedicated modules: `git.rs`, `bazaar/`, `mercurial.rs`, `subversion.rs`, etc.
- Generic types (`GenericBranch`, `GenericRepository`) provide VCS-agnostic interfaces
- `VcsType` enum identifies different version control systems

**Error Handling:**
- Comprehensive Python exception mapping using `import_exception!` macro
- Centralized error conversion from Python to Rust in `error.rs`
- Covers transport, format, permission, forge, and VCS-specific errors

**Initialization:**
- `breezyshim::init()` must be called before using the library (unless auto-initialize feature enabled)
- Checks minimum Breezy version (3.3.6+) and loads Git/Bazaar support
- Thread-safe initialization using `std::sync::Once`

## Feature Flags

- `debian`: Enables Debian packaging integration
- `dirty-tracker`: File change tracking functionality  
- `auto-initialize`: Automatically calls `init()` at startup
- `launchpad`: Launchpad integration support
- `sqlx`: PostgreSQL database support

## Development Notes

- Minimum supported Breezy version: 3.3.6
- Requires Breezy Python package installed: `python -m pip install breezy`
- For Ubuntu development: also install `bzr devscripts libapt-pkg-dev`
- Uses serial_test for tests that require single-threaded execution