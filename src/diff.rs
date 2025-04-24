//! Generation of unified diffs between trees.
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::io::Write;

pub fn show_diff_trees<T: crate::tree::PyTree, U: crate::tree::PyTree>(
    tree1: &T,
    tree2: &U,
    mut w: impl Write,
    old_label: Option<&str>,
    new_label: Option<&str>,
) -> Result<(), crate::error::Error> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = py.import_bound("breezy.diff")?;
        let f = m.getattr("show_diff_trees")?;

        let o = py.import_bound("io")?.call_method0("BytesIO")?;

        let kwargs = PyDict::new_bound(py);
        if let Some(old_label) = old_label {
            kwargs.set_item("old_label", old_label)?;
        }

        if let Some(new_label) = new_label {
            kwargs.set_item("new_label", new_label)?;
        }

        f.call(
            (tree1.to_object(py), tree2.to_object(py), &o),
            Some(&kwargs),
        )?;

        let s = o.call_method0("getvalue")?.extract::<Vec<u8>>()?;

        w.write_all(&s)?;

        Ok(())
    })?;

    Ok(())
}
