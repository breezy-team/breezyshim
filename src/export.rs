use crate::tree::Tree;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::Path;

pub fn export(
    tree: &dyn Tree,
    target: &std::path::Path,
    subdir: Option<&std::path::Path>,
) -> Result<(), crate::error::Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.export").unwrap();
        let export = m.getattr("export").unwrap();
        let kwargs = PyDict::new_bound(py);
        let subdir = if subdir.is_none() || subdir == Some(Path::new("")) {
            None
        } else {
            Some(subdir)
        };
        kwargs.set_item("subdir", subdir).unwrap();
        export.call(
            (tree.to_object(py), target, "dir", py.None()),
            Some(&kwargs),
        )?;
        Ok(())
    })
}
