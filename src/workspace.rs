//! Convenience functions for automated operations on a VCS tree
#[cfg(feature = "dirty-tracker")]
use crate::dirty_tracker::{DirtyTreeTracker, State as DirtyTrackerState};
use crate::error::Error;
use crate::tree::{PyTree, Tree, WorkingTree};
use pyo3::prelude::*;

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
