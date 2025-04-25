//! Inventory trees
use crate::error::Error;
use crate::tree::Path;
use pyo3::prelude::*;

/// Trait for trees that have an inventory and can be modified.
///
/// Inventory trees are trees that track file identifiers, which is a feature
/// specific to Bazaar trees.
pub trait MutableInventoryTree: crate::tree::PyMutableTree {
    /// Add files to the tree with explicit file identifiers.
    ///
    /// # Parameters
    ///
    /// * `paths` - The paths of the files to add.
    /// * `file_ids` - The file identifiers to assign to the files.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the files could not be added.
    fn add(&self, paths: &[&Path], file_ids: &[crate::bazaar::FileId]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "add", (paths.to_vec(), file_ids.to_vec()))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }
}

impl MutableInventoryTree for crate::tree::WorkingTree {}
