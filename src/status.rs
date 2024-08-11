use crate::tree::WorkingTree;
use pyo3::prelude::*;

pub fn show_tree_status(wt: &dyn WorkingTree) -> crate::Result<()> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.status")?;
        let f = m.getattr("show_tree_status")?;
        f.call1((&wt.to_object(py),))?;
        Ok(())
    })
}
