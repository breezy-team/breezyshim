//! Trees
use crate::branch::Branch;
use crate::error::Error;
use crate::lock::Lock;
use crate::revisionid::RevisionId;
use pyo3::prelude::*;

pub type Path = std::path::Path;
pub type PathBuf = std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Kind {
    File,
    Directory,
    Symlink,
    TreeReference,
}

impl Kind {
    pub fn marker(&self) -> &'static str {
        match self {
            Kind::File => "",
            Kind::Directory => "/",
            Kind::Symlink => "@",
            Kind::TreeReference => "+",
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kind::File => write!(f, "file"),
            Kind::Directory => write!(f, "directory"),
            Kind::Symlink => write!(f, "symlink"),
            Kind::TreeReference => write!(f, "tree-reference"),
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(Kind::File),
            "directory" => Ok(Kind::Directory),
            "symlink" => Ok(Kind::Symlink),
            "tree-reference" => Ok(Kind::TreeReference),
            n => {
                return Err(format!("Invalid kind: {}", n));
            }
        }
    }
}

impl pyo3::ToPyObject for Kind {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        match self {
            Kind::File => "file".to_object(py),
            Kind::Directory => "directory".to_object(py),
            Kind::Symlink => "symlink".to_object(py),
            Kind::TreeReference => "tree-reference".to_object(py),
        }
    }
}

impl pyo3::FromPyObject<'_> for Kind {
    fn extract_bound(ob: &Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let s: String = ob.extract()?;
        match s.as_str() {
            "file" => Ok(Kind::File),
            "directory" => Ok(Kind::Directory),
            "symlink" => Ok(Kind::Symlink),
            "tree-reference" => Ok(Kind::TreeReference),
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid kind: {}",
                s
            ))),
        }
    }
}

pub enum TreeEntry {
    File {
        executable: bool,
        kind: Kind,
        revision: Option<RevisionId>,
        size: u64,
    },
    Directory {
        revision: Option<RevisionId>,
    },
    Symlink {
        revision: Option<RevisionId>,
        symlink_target: String,
    },
    TreeReference {
        revision: Option<RevisionId>,
        reference_revision: RevisionId,
    },
}

impl FromPyObject<'_> for TreeEntry {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let kind: String = ob.getattr("kind")?.extract()?;
        match kind.as_str() {
            "file" => {
                let executable: bool = ob.getattr("executable")?.extract()?;
                let kind: Kind = ob.getattr("kind")?.extract()?;
                let size: u64 = ob.getattr("size")?.extract()?;
                let revision: Option<RevisionId> = ob.getattr("revision")?.extract()?;
                Ok(TreeEntry::File {
                    executable,
                    kind,
                    size,
                    revision,
                })
            }
            "directory" => {
                let revision: Option<RevisionId> = ob.getattr("revision")?.extract()?;
                Ok(TreeEntry::Directory { revision })
            }
            "symlink" => {
                let revision: Option<RevisionId> = ob.getattr("revision")?.extract()?;
                let symlink_target: String = ob.getattr("symlink_target")?.extract()?;
                Ok(TreeEntry::Symlink {
                    revision,
                    symlink_target,
                })
            }
            "tree-reference" => {
                let revision: Option<RevisionId> = ob.getattr("revision")?.extract()?;
                let reference_revision: RevisionId = ob.getattr("reference_revision")?.extract()?;
                Ok(TreeEntry::TreeReference {
                    revision,
                    reference_revision,
                })
            }
            kind => panic!("Invalid kind: {}", kind),
        }
    }
}

pub trait Tree: ToPyObject {
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    fn get_file(&self, path: &Path) -> Result<Box<dyn std::io::Read>, Error> {
        Python::with_gil(|py| {
            let f = self.to_object(py).call_method1(py, "get_file", (path,))?;

            let f = pyo3_filelike::PyBinaryFile::from(f);

            Ok(Box::new(f) as Box<dyn std::io::Read>)
        })
    }

    fn get_file_text(&self, path: &Path) -> Result<Vec<u8>, Error> {
        Python::with_gil(|py| {
            let text = self
                .to_object(py)
                .call_method1(py, "get_file_text", (path,))?;
            text.extract(py).map_err(|e| e.into())
        })
    }

    fn get_file_lines(&self, path: &Path) -> Result<Vec<Vec<u8>>, Error> {
        Python::with_gil(|py| {
            let lines = self
                .to_object(py)
                .call_method1(py, "get_file_lines", (path,))?;
            lines.extract(py).map_err(|e| e.into())
        })
    }

    fn lock_read(&self) -> Result<Lock, Error> {
        Python::with_gil(|py| {
            let lock = self.to_object(py).call_method0(py, "lock_read")?;
            Ok(Lock::from(lock))
        })
    }

    fn has_filename(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "has_filename", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_symlink_target(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            let target = self
                .to_object(py)
                .call_method1(py, "get_symlink_target", (path,))?;
            target.extract(py).map_err(|e| e.into())
        })
    }

    fn get_parent_ids(&self) -> Result<Vec<RevisionId>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, "get_parent_ids")
                .unwrap()
                .extract(py)?)
        })
    }

    fn is_ignored(&self, path: &Path) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "is_ignored", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn kind(&self, path: &Path) -> Result<Kind, Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "kind", (path,))
                .unwrap()
                .extract(py)
                .map_err(|e| e.into())
        })
    }

    fn is_versioned(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "is_versioned", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn iter_changes(
        &self,
        other: &dyn Tree,
        specific_files: Option<&[&Path]>,
        want_unversioned: Option<bool>,
        require_versioned: Option<bool>,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeChange, Error>>>, Error> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new_bound(py);
            if let Some(specific_files) = specific_files {
                kwargs.set_item("specific_files", specific_files)?;
            }
            if let Some(want_unversioned) = want_unversioned {
                kwargs.set_item("want_unversioned", want_unversioned)?;
            }
            if let Some(require_versioned) = require_versioned {
                kwargs.set_item("require_versioned", require_versioned)?;
            }
            struct TreeChangeIter(pyo3::PyObject);

            impl Iterator for TreeChangeIter {
                type Item = Result<TreeChange, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::with_gil(|py| {
                        let next = match self.0.call_method0(py, "__next__") {
                            Ok(v) => v,
                            Err(e) => {
                                if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                                    return None;
                                }
                                return Some(Err(e.into()));
                            }
                        };

                        if next.is_none(py) {
                            None
                        } else {
                            Some(next.extract(py).map_err(|e| e.into()))
                        }
                    })
                }
            }

            Ok(
                Box::new(TreeChangeIter(self.to_object(py).call_method_bound(
                    py,
                    "iter_changes",
                    (other.to_object(py),),
                    Some(&kwargs),
                )?)) as Box<dyn Iterator<Item = Result<TreeChange, Error>>>,
            )
        })
    }

    fn has_versioned_directories(&self) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "has_versioned_directories")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn preview_transform(&self) -> Result<crate::transform::TreeTransform, Error> {
        Python::with_gil(|py| {
            let transform = self.to_object(py).call_method0(py, "preview_transform")?;
            Ok(crate::transform::TreeTransform::from(transform))
        })
    }

    fn list_files(
        &self,
        include_root: Option<bool>,
        from_dir: Option<&Path>,
        recursive: Option<bool>,
        recurse_nested: Option<bool>,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, bool, Kind, TreeEntry), Error>>>, Error>
    {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new_bound(py);
            if let Some(include_root) = include_root {
                kwargs.set_item("include_root", include_root)?;
            }
            if let Some(from_dir) = from_dir {
                kwargs.set_item("from_dir", from_dir)?;
            }
            if let Some(recursive) = recursive {
                kwargs.set_item("recursive", recursive)?;
            }
            if let Some(recurse_nested) = recurse_nested {
                kwargs.set_item("recurse_nested", recurse_nested)?;
            }
            struct ListFilesIter(pyo3::PyObject);

            impl Iterator for ListFilesIter {
                type Item = Result<(PathBuf, bool, Kind, TreeEntry), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::with_gil(|py| {
                        let next = match self.0.call_method0(py, "__next__") {
                            Ok(v) => v,
                            Err(e) => {
                                if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                                    return None;
                                }
                                return Some(Err(e.into()));
                            }
                        };

                        if next.is_none(py) {
                            None
                        } else {
                            Some(next.extract(py).map_err(|e| e.into()))
                        }
                    })
                }
            }

            Ok(Box::new(ListFilesIter(self.to_object(py).call_method_bound(
                py,
                "list_files",
                (),
                Some(&kwargs),
            )?))
                as Box<
                    dyn Iterator<Item = Result<(PathBuf, bool, Kind, TreeEntry), Error>>,
                >)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    fn iter_child_entries(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, Kind, TreeEntry), Error>>>, Error> {
        Python::with_gil(|py| {
            struct IterChildEntriesIter(pyo3::PyObject);

            impl Iterator for IterChildEntriesIter {
                type Item = Result<(PathBuf, Kind, TreeEntry), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::with_gil(|py| {
                        let next = match self.0.call_method0(py, "__next__") {
                            Ok(v) => v,
                            Err(e) => {
                                if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                                    return None;
                                }
                                return Some(Err(e.into()));
                            }
                        };

                        if next.is_none(py) {
                            None
                        } else {
                            Some(next.extract(py).map_err(|e| e.into()))
                        }
                    })
                }
            }

            Ok(
                Box::new(IterChildEntriesIter(self.to_object(py).call_method1(
                    py,
                    "iter_child_entries",
                    (path,),
                )?))
                    as Box<dyn Iterator<Item = Result<(PathBuf, Kind, TreeEntry), Error>>>,
            )
        })
    }
}

pub struct GenericTree(PyObject);

impl ToPyObject for GenericTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl From<PyObject> for GenericTree {
    fn from(obj: PyObject) -> Self {
        GenericTree(obj)
    }
}

impl Tree for GenericTree {}

pub trait MutableTree: Tree {
    fn add(&self, files: &[&Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::with_gil(|py| -> Result<(), PyErr> {
            self.to_object(py).call_method1(
                py,
                "add",
                (files.iter().map(|p| p.to_path_buf()).collect::<Vec<_>>(),),
            )?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn lock_write(&self) -> Result<Lock, Error> {
        Python::with_gil(|py| {
            let lock = self.to_object(py).call_method0(py, "lock_write")?;
            Ok(Lock::from(lock))
        })
    }

    fn put_file_bytes_non_atomic(&self, path: &Path, data: &[u8]) -> Result<(), Error> {
        assert!(path.is_relative());
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "put_file_bytes_non_atomic", (path, data))?;
            Ok(())
        })
    }

    fn has_changes(&self) -> std::result::Result<bool, Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "has_changes")?
                .extract::<bool>(py)
                .map_err(|e| e.into())
        })
    }

    fn mkdir(&self, path: &Path) -> Result<(), Error> {
        assert!(path.is_relative());
        Python::with_gil(|py| -> Result<(), PyErr> {
            self.to_object(py).call_method1(py, "mkdir", (path,))?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn remove(&self, files: &[&std::path::Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::with_gil(|py| -> Result<(), PyErr> {
            self.to_object(py).call_method1(
                py,
                "remove",
                (files.iter().map(|p| p.to_path_buf()).collect::<Vec<_>>(),),
            )?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn as_tree(&self) -> &dyn Tree;
}

pub struct RevisionTree(pub PyObject);

impl ToPyObject for RevisionTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Tree for RevisionTree {}

impl Clone for RevisionTree {
    fn clone(&self) -> Self {
        Python::with_gil(|py| RevisionTree(self.0.clone_ref(py)))
    }
}

impl RevisionTree {
    pub fn repository(&self) -> crate::repository::Repository {
        Python::with_gil(|py| {
            let repository = self.to_object(py).getattr(py, "_repository").unwrap();
            crate::repository::Repository::new(repository)
        })
    }

    pub fn get_revision_id(&self) -> RevisionId {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_revision_id")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn get_parent_ids(&self) -> Vec<RevisionId> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_parent_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeChange {
    pub path: (Option<PathBuf>, Option<PathBuf>),
    pub changed_content: bool,
    pub versioned: (Option<bool>, Option<bool>),
    pub name: (Option<std::ffi::OsString>, Option<std::ffi::OsString>),
    pub kind: (Option<Kind>, Option<Kind>),
    pub executable: (Option<bool>, Option<bool>),
    pub copied: bool,
}

impl ToPyObject for TreeChange {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("path", &self.path).unwrap();
        dict.set_item("changed_content", self.changed_content)
            .unwrap();
        dict.set_item("versioned", self.versioned).unwrap();
        dict.set_item("name", &self.name).unwrap();
        dict.set_item("kind", &self.kind).unwrap();
        dict.set_item("executable", self.executable).unwrap();
        dict.set_item("copied", self.copied).unwrap();
        dict.into()
    }
}

impl FromPyObject<'_> for TreeChange {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        fn from_bool(o: &Bound<PyAny>) -> PyResult<bool> {
            if let Ok(b) = o.extract::<isize>() {
                Ok(b != 0)
            } else {
                o.extract::<bool>()
            }
        }

        fn from_opt_bool_tuple(o: &Bound<PyAny>) -> PyResult<(Option<bool>, Option<bool>)> {
            let tuple = o.extract::<(Option<Bound<PyAny>>, Option<Bound<PyAny>>)>()?;
            Ok((
                tuple.0.map(|o| from_bool(&o.as_borrowed())).transpose()?,
                tuple.1.map(|o| from_bool(&o.as_borrowed())).transpose()?,
            ))
        }

        let path = obj.getattr("path")?;
        let changed_content = from_bool(&obj.getattr("changed_content")?)?;

        let versioned = from_opt_bool_tuple(&obj.getattr("versioned")?)?;
        let name = obj.getattr("name")?;
        let kind = obj.getattr("kind")?;
        let executable = from_opt_bool_tuple(&obj.getattr("executable")?)?;
        let copied = obj.getattr("copied")?;

        Ok(TreeChange {
            path: path.extract()?,
            changed_content,
            versioned,
            name: name.extract()?,
            kind: kind.extract()?,
            executable,
            copied: copied.extract()?,
        })
    }
}

pub struct MemoryTree(pub PyObject);

impl ToPyObject for MemoryTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl From<&dyn Branch> for MemoryTree {
    fn from(branch: &dyn Branch) -> Self {
        Python::with_gil(|py| {
            MemoryTree(
                branch
                    .to_object(py)
                    .call_method0(py, "create_memorytree")
                    .unwrap()
                    .extract(py)
                    .unwrap(),
            )
        })
    }
}

impl Tree for MemoryTree {}

impl MutableTree for MemoryTree {
    fn as_tree(&self) -> &dyn Tree {
        self
    }
}

pub use crate::workingtree::WorkingTree;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::{create_standalone_workingtree, ControlDirFormat};

    #[test]
    fn test_remove() {
        let td = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let path = td.path().join("foo");
        std::fs::write(&path, b"").unwrap();
        wt.add(&[(std::path::Path::new("foo"))]).unwrap();
        wt.build_commit()
            .message("Initial commit")
            .reporter(&crate::commit::NullCommitReporter::new())
            .commit()
            .unwrap();
        assert!(wt.has_filename(&path));
        wt.remove(&[Path::new("foo")]).unwrap();
        assert!(!wt.is_versioned(&path));
        std::mem::drop(td);
    }
}
