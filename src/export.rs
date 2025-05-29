//! Export a tree to a directory.
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::Path;

/// Export a tree to a directory.
///
/// # Arguments
/// * `tree` - Tree to export
/// * `target` - Target directory path
/// * `subdir` - Optional subdirectory within the tree to export
///
/// # Returns
/// Result with empty success value or error
pub fn export<T: crate::tree::PyTree>(
    tree: &T,
    target: &std::path::Path,
    subdir: Option<&std::path::Path>,
) -> Result<(), crate::error::Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.export").unwrap();
        let export = m.getattr("export").unwrap();
        let kwargs = PyDict::new(py);
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
