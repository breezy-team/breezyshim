use crate::error::Error;
use crate::tree::MutableTree;
use pyo3::prelude::*;

pub fn release(local_tree: &dyn MutableTree, subpath: &std::path::Path) -> Result<String, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.plugins.debian.release")?;
        let release = m.getattr("release")?;
        let result = release.call1((local_tree.to_object(py), subpath))?;
        Ok(result.extract()?)
    })
}
