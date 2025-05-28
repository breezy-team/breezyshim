//! Detect renames between two trees based on file contents.
use crate::tree::{PyMutableTree, PyTree};
use pyo3::prelude::*;

pub fn guess_renames<T: PyTree, U: PyMutableTree>(
    from_tree: &T,
    mutable_tree: &U,
) -> Result<(), crate::error::Error> {
    pyo3::Python::with_gil(|py| -> Result<(), pyo3::PyErr> {
        let m = py.import("breezy.rename_map")?;
        let rename_map = m.getattr("RenameMap")?;
        rename_map.call_method1(
            "guess_renames",
            (from_tree, mutable_tree),
        )?;
        Ok(())
    })
    .map_err(Into::into)
}
