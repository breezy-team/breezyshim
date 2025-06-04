//! Status reporting functions.
use crate::workingtree::PyWorkingTree;
use pyo3::prelude::*;

/// Display the status of a working tree.
///
/// This function prints the status of the working tree to stdout,
/// showing which files have been modified, added, or removed.
///
/// # Arguments
///
/// * `wt` - The working tree to show the status for
///
/// # Returns
///
/// `Ok(())` on success, or an error if the operation fails
pub fn show_tree_status(wt: &dyn PyWorkingTree) -> crate::Result<()> {
    Python::with_gil(|py| {
        let m = py.import("breezy.status")?;
        let f = m.getattr("show_tree_status")?;
        f.call1((&wt.to_object(py),))?;
        Ok(())
    })
}
