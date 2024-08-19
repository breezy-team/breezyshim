//! Status reporting functions.
use crate::tree::WorkingTree;
use pyo3::prelude::*;

pub fn show_tree_status(wt: &WorkingTree) -> crate::Result<()> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.status")?;
        let f = m.getattr("show_tree_status")?;
        f.call1((&wt.0,))?;
        Ok(())
    })
}
