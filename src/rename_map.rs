//! Detect renames between two trees based on file contents.
use crate::tree::{MutableTree, Tree};
use pyo3::prelude::*;

pub fn guess_renames(
    from_tree: &dyn Tree,
    mutable_tree: &dyn MutableTree,
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
