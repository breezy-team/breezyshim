//! Generation of unified diffs between trees.
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::io::Write;

/// Generate a unified diff between two trees and write it to the provided writer.
///
/// # Arguments
/// * `tree1` - First tree to compare
/// * `tree2` - Second tree to compare
/// * `w` - Writer to write the diff to
/// * `old_label` - Optional label for the old tree
/// * `new_label` - Optional label for the new tree
///
/// # Returns
/// Result with empty success value or error
pub fn show_diff_trees<T: crate::tree::PyTree, U: crate::tree::PyTree>(
    tree1: &T,
    tree2: &U,
    mut w: impl Write,
    old_label: Option<&str>,
    new_label: Option<&str>,
) -> Result<(), crate::error::Error> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = py.import("breezy.diff")?;
        let f = m.getattr("show_diff_trees")?;

        let o = py.import("io")?.call_method0("BytesIO")?;

        let kwargs = PyDict::new(py);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::workingtree::WorkingTree;
    use std::io::Cursor;

    #[test]
    fn test_show_diff_trees_empty() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree1 = wt.basis_tree().unwrap();
        let tree2 = wt.basis_tree().unwrap();

        let mut output = Vec::new();
        let result = show_diff_trees(&tree1, &tree2, &mut output, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_diff_trees_with_labels() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree1 = wt.basis_tree().unwrap();
        let tree2 = wt.basis_tree().unwrap();

        let mut output = Vec::new();
        let result = show_diff_trees(&tree1, &tree2, &mut output, Some("old"), Some("new"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_diff_trees_cursor() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree1 = wt.basis_tree().unwrap();
        let tree2 = wt.basis_tree().unwrap();

        let mut cursor = Cursor::new(Vec::new());
        let result = show_diff_trees(&tree1, &tree2, &mut cursor, None, None);
        assert!(result.is_ok());
    }
}
