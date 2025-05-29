//! Functions for cleaning a working tree by removing unknown files.
//!
//! This module provides functionality to clean a working tree by removing
//! unknown files, ignored files, and various detritus files.

use crate::error::Error;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::Path;

/// Clean a working tree by removing unwanted files.
///
/// # Parameters
///
/// * `directory` - The directory containing the working tree to clean
/// * `unknown` - If true, remove unknown files (those not tracked by version control)
/// * `ignored` - If true, remove ignored files (those matching ignore patterns)
/// * `detritus` - If true, remove detritus files (like backup files, etc.)
/// * `dry_run` - If true, only report what would be done without actually removing files
/// * `no_prompt` - If true, don't prompt for confirmation before removing files
///
/// # Returns
///
/// * `Ok(())` on success
/// * `Err` containing any error that occurred during the cleaning process
pub fn clean_tree(
    directory: &Path,
    unknown: bool,
    ignored: bool,
    detritus: bool,
    dry_run: bool,
    no_prompt: bool,
) -> Result<(), Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.clean_tree")?;
        let f = m.getattr("clean_tree")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("directory", directory.to_str().unwrap())?;
        kwargs.set_item("unknown", unknown)?;
        kwargs.set_item("ignored", ignored)?;
        kwargs.set_item("detritus", detritus)?;
        kwargs.set_item("dry_run", dry_run)?;
        kwargs.set_item("no_prompt", no_prompt)?;
        f.call((), Some(&kwargs))?;
        Ok(())
    })
}
