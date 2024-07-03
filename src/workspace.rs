use crate::dirty_tracker::DirtyTracker;
use crate::error::Error;
use crate::tree::{Tree, WorkingTree};
use pyo3::prelude::*;

pub fn reset_tree(
    local_tree: &WorkingTree,
    basis_tree: Option<&dyn Tree>,
    subpath: Option<&std::path::Path>,
    dirty_tracker: Option<&DirtyTracker>,
) -> Result<(), Error> {
    Python::with_gil(|py| {
        let workspace_m = py.import_bound("breezy.workspace")?;
        let reset_tree = workspace_m.getattr("reset_tree")?;
        let local_tree: PyObject = local_tree.to_object(py);
        let basis_tree: Option<PyObject> = basis_tree.map(|o| o.to_object(py));
        let dirty_tracker: Option<PyObject> = dirty_tracker.map(|dt| dt.to_object(py));
        reset_tree.call1((local_tree, basis_tree, subpath, dirty_tracker))?;
        Ok(())
    })
}

pub fn check_clean_tree(
    local_tree: &WorkingTree,
    basis_tree: &dyn Tree,
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
