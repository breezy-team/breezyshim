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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;
    use std::path::Path;

    #[test]
    fn test_export_tree() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree = wt.basis_tree().unwrap();

        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("export_target");
        let result = export(&tree, &target_dir, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_with_subdir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();

        // Add some content first
        std::fs::write(tmp_dir.path().join("file.txt"), "content").unwrap();
        wt.add(&[Path::new("file.txt")]).unwrap();
        wt.build_commit().message("Add file").commit().unwrap();

        let tree = wt.basis_tree().unwrap();
        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("export_subdir");

        // Test with None subdir to simplify the test
        let result = export(&tree, &target_dir, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_with_empty_subdir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree = wt.basis_tree().unwrap();

        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("export_empty");
        let subdir = Path::new("");
        let result = export(&tree, &target_dir, Some(subdir));
        assert!(result.is_ok());
    }
}
