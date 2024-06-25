use pyo3::prelude::*;
use crate::tree::{MutableTree, Tree};

pub fn guess_renames(from_tree: &dyn Tree, mutable_tree: &dyn MutableTree) -> pyo3::PyResult<()> {
    pyo3::Python::with_gil(|py| {
        let m = py.import_bound("breezy.rename_map")?;
        let rename_map = m.getattr("RenameMap")?;
        rename_map.call_method1(
            "guess_renames",
            (from_tree.to_object(py), mutable_tree.to_object(py)),
        )?;
        Ok(())
    })
}
