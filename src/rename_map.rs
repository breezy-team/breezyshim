//! Detect renames between two trees based on file contents.
use crate::tree::{PyMutableTree, PyTree};
use pyo3::prelude::*;

/// Guess file renames between two trees based on file contents.
///
/// This function detects files that were renamed between the source tree
/// and the target tree by comparing file contents, and updates the
/// target tree to reflect these renames.
///
/// # Arguments
///
/// * `from_tree` - The source tree to detect renames from
/// * `mutable_tree` - The target tree to apply renames to
///
/// # Returns
///
/// `Ok(())` on success, or an error if the operation fails
pub fn guess_renames<T: PyTree, U: PyMutableTree>(
    from_tree: &T,
    mutable_tree: &U,
) -> Result<(), crate::error::Error> {
    pyo3::Python::with_gil(|py| -> Result<(), pyo3::PyErr> {
        let m = py.import_bound("breezy.rename_map")?;
        let rename_map = m.getattr("RenameMap")?;
        rename_map.call_method1(
            "guess_renames",
            (from_tree.to_object(py), mutable_tree.to_object(py)),
        )?;
        Ok(())
    })
    .map_err(Into::into)
}
