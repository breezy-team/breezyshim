use crate::error::Error;
use crate::tree::Tree;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

pub fn get_tarballs(
    orig_dir: &Path,
    tree: &dyn Tree,
    package: &str,
    version: &str,
    locations: &[&Path],
) -> Result<Vec<PathBuf>, Error> {
    Python::with_gil(|py| {
        let m = PyModule::import_bound(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let get_tarballs = m.getattr("get_tarballs").unwrap();
        Ok(get_tarballs
            .call1((
                orig_dir,
                tree.to_object(py),
                package,
                version,
                locations.to_vec(),
            ))?
            .extract()?)
    })
}
