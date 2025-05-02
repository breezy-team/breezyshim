//! Convenience functions for automated operations on a VCS tree
#[cfg(feature = "dirty-tracker")]
use crate::dirty_tracker::{DirtyTreeTracker, State as DirtyTrackerState};
use crate::error::Error;
use crate::tree::{PyTree, WorkingTree};
use pyo3::prelude::*;

/// Reset a tree with a dirty tracker.
///
/// This function resets a working tree to match a basis tree, but only if the
/// dirty tracker indicates that the tree is dirty. If the tree is clean, the
/// function does nothing.
///
/// # Parameters
///
/// * `local_tree` - The working tree to reset.
/// * `basis_tree` - The basis tree to reset to, or None to use the working tree's basis tree.
/// * `subpath` - The path within the tree to reset, or None to reset the entire tree.
/// * `dirty_tracker` - The dirty tracker to use, or None to ignore dirty tracking.
///
/// # Returns
///
/// `Ok(())` on success, or an error if the tree could not be reset.
#[cfg(feature = "dirty-tracker")]
pub fn reset_tree_with_dirty_tracker(
    local_tree: &WorkingTree,
    basis_tree: Option<&dyn PyTree>,
    subpath: Option<&std::path::Path>,
    dirty_tracker: Option<&mut DirtyTreeTracker>,
) -> Result<(), Error> {
    if let Some(dirty_tracker) = dirty_tracker {
        if dirty_tracker.state() == DirtyTrackerState::Clean {
            return Ok(());
        }
        // TODO: Only reset those files that are dirty
    }
    reset_tree(local_tree, basis_tree, subpath)
}

/// Reset a tree to match a basis tree.
///
/// This function resets a working tree to match a basis tree, discarding any
/// uncommitted changes in the working tree.
///
/// # Parameters
///
/// * `local_tree` - The working tree to reset.
/// * `basis_tree` - The basis tree to reset to, or None to use the working tree's basis tree.
/// * `subpath` - The path within the tree to reset, or None to reset the entire tree.
///
/// # Returns
///
/// `Ok(())` on success, or an error if the tree could not be reset.
pub fn reset_tree(
    local_tree: &WorkingTree,
    basis_tree: Option<&dyn PyTree>,
    subpath: Option<&std::path::Path>,
) -> Result<(), Error> {
    Python::with_gil(|py| {
        let workspace_m = py.import_bound("breezy.workspace")?;
        let reset_tree = workspace_m.getattr("reset_tree")?;
        let local_tree: PyObject = local_tree.to_object(py);
        let basis_tree: Option<PyObject> = basis_tree.map(|o| o.to_object(py));
        reset_tree.call1((local_tree, basis_tree, subpath))?;
        Ok(())
    })
}

/// Check if a tree is clean.
///
/// This function checks if a working tree is clean, meaning it has no uncommitted
/// changes compared to a basis tree.
///
/// # Parameters
///
/// * `local_tree` - The working tree to check.
/// * `basis_tree` - The basis tree to compare against.
/// * `subpath` - The path within the tree to check.
///
/// # Returns
///
/// `Ok(())` if the tree is clean, or an error if the tree is dirty or the check failed.
pub fn check_clean_tree(
    local_tree: &WorkingTree,
    basis_tree: &dyn PyTree,
    subpath: &std::path::Path,
) -> Result<(), Error> {
    Python::with_gil(|py| {
        let workspace_m = py.import_bound("breezy.workspace")?;
        let check_clean_tree = workspace_m.getattr("check_clean_tree")?;
        let local_tree: PyObject = local_tree.to_object(py).clone_ref(py);
        let basis_tree: PyObject = basis_tree.to_object(py).clone_ref(py);
        check_clean_tree.call1((local_tree, basis_tree, subpath.to_path_buf()))?;
        Ok(())
    })
}
