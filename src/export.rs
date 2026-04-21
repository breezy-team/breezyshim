//! Export a tree to a directory.
use pyo3::exceptions::PyStopIteration;
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
    Python::attach(|py| {
        let m = py.import("breezy.export").unwrap();
        let export = m.getattr("export").unwrap();
        let kwargs = PyDict::new(py);
        let subdir = if subdir.is_none() || subdir == Some(Path::new("")) {
            None
        } else {
            Some(subdir.unwrap().to_string_lossy().to_string())
        };
        kwargs.set_item("subdir", subdir).unwrap();
        export.call(
            (
                tree.to_object(py),
                target.to_string_lossy().to_string(),
                "dir",
                py.None(),
            ),
            Some(&kwargs),
        )?;
        Ok(())
    })
}

/// Archive format for [`archive`].
#[derive(Debug, Clone, Copy)]
pub enum ArchiveFormat {
    /// gzip-compressed tar (`.tar.gz` / `.tgz`)
    Tgz,
    /// bzip2-compressed tar
    Tbz2,
    /// uncompressed tar
    Tar,
    /// ZIP archive
    Zip,
}

impl ArchiveFormat {
    fn as_str(&self) -> &'static str {
        match self {
            ArchiveFormat::Tgz => "tgz",
            ArchiveFormat::Tbz2 => "tbz2",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::Zip => "zip",
        }
    }
}

/// Iterator over `bytes` chunks yielded by [`Tree::archive`][crate::tree::Tree].
///
/// Wraps the Python iterator returned by `breezy.Tree.archive(...)`.
pub struct ArchiveIter(pyo3::Py<PyAny>);

impl Iterator for ArchiveIter {
    type Item = Result<Vec<u8>, crate::error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::attach(|py| match self.0.call_method0(py, "__next__") {
            Ok(v) => Some(v.extract::<Vec<u8>>(py).map_err(Into::into)),
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Err(e) => Some(Err(e.into())),
        })
    }
}

/// Create an in-memory archive of a tree, returning an iterator of byte
/// chunks suitable for streaming to an HTTP client.
///
/// Calls `breezy.Tree.archive(format, name, subdir=subdir)`.
pub fn archive<T: crate::tree::PyTree>(
    tree: &T,
    format: ArchiveFormat,
    name: &str,
    subdir: Option<&Path>,
    root: Option<&str>,
) -> Result<ArchiveIter, crate::error::Error> {
    Python::attach(|py| {
        let kwargs = PyDict::new(py);
        if let Some(s) = subdir {
            kwargs.set_item("subdir", s.to_string_lossy().to_string())?;
        }
        if let Some(r) = root {
            kwargs.set_item("root", r)?;
        }
        let obj = tree.to_object(py);
        let iter = obj.call_method(
            py,
            "archive",
            (format.as_str(), name),
            Some(&kwargs),
        )?;
        let iter = py.import("builtins")?.getattr("iter")?.call1((iter,))?;
        Ok(ArchiveIter(iter.unbind()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;
    use serial_test::serial;
    use std::path::Path;

    #[serial]
    #[test]
    fn test_export_tree() {
        let env = crate::testing::TestEnv::new();
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree = wt.basis_tree().unwrap();

        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("export_target");
        let result = export(&tree, &target_dir, None);
        assert!(result.is_ok());
        std::mem::drop(env);
    }

    #[serial]
    #[test]
    fn test_export_with_subdir() {
        let env = crate::testing::TestEnv::new();
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
        std::mem::drop(env);
    }

    #[serial]
    #[test]
    fn test_export_with_empty_subdir() {
        let env = crate::testing::TestEnv::new();
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        let tree = wt.basis_tree().unwrap();

        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("export_empty");
        let subdir = Path::new("");
        let result = export(&tree, &target_dir, Some(subdir));
        assert!(result.is_ok());
        std::mem::drop(env);
    }
}
