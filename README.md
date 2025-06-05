# Rust wrapper for Breezy

This crate contains a rust wrapper for the Breezy API, which is written in
Python.

Breezy itself is being ported to Rust, but until that port has completed, this
crate allows access to the most important Breezy APIs via Rust.

The Rust API here will follow the Breezy 4.0 Rust API as much as possible,
to make porting easier.

## prelude

This crate provides a prelude module that re-exports the most important
types and traits from the Breezy API. This allows you to use the Breezy API
without having to import each type and trait individually.

```rust
use breezyshim::prelude::*;
```
