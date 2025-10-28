//! Detect renames between two trees based on file contents.
use crate::tree::{PyMutableTree, PyTree};
use pyo3::prelude::*;

/// Guess file renames between two trees based on file contents.
///
/// This function detects files that were renamed between the source tree
/// and the target tree by comparing file contents, and updates the
/// target tree to reflect these renames.
///
/// # Arguments
///
/// * `from_tree` - The source tree to detect renames from
/// * `mutable_tree` - The target tree to apply renames to
///
/// # Returns
///
/// `Ok(())` on success, or an error if the operation fails
pub fn guess_renames(
    from_tree: &dyn PyTree,
    mutable_tree: &dyn PyMutableTree,
) -> Result<(), crate::error::Error> {
    pyo3::Python::attach(|py| -> Result<(), pyo3::PyErr> {
        let m = py.import("breezy.rename_map")?;
        let rename_map = m.getattr("RenameMap")?;
        rename_map.call_method1(
            "guess_renames",
            (from_tree.to_object(py), mutable_tree.to_object(py)),
        )?;
        Ok(())
    })
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;
    use std::path::Path;

    #[test]
    fn test_guess_renames() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();

        // Add some content to make rename detection meaningful
        std::fs::write(tmp_dir.path().join("file1.txt"), "content1").unwrap();
        wt.add(&[Path::new("file1.txt")]).unwrap();
        wt.build_commit()
            .message("Initial commit")
            .commit()
            .unwrap();

        let from_tree = wt.basis_tree().unwrap();

        // Create a second working tree and simulate a rename
        let tmp_dir2 = tempfile::tempdir().unwrap();
        let wt2 = create_standalone_workingtree(tmp_dir2.path(), "2a").unwrap();
        std::fs::write(tmp_dir2.path().join("file2.txt"), "content1").unwrap();
        wt2.add(&[Path::new("file2.txt")]).unwrap();

        let result = guess_renames(&from_tree, &wt2);
        assert!(result.is_ok());
    }
}
