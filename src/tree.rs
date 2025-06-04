//! Trees
use crate::error::Error;
use crate::lock::Lock;
use crate::revisionid::RevisionId;
use pyo3::intern;
use pyo3::prelude::*;

/// Type alias for std::path::Path.
pub type Path = std::path::Path;
/// Type alias for std::path::PathBuf.
pub type PathBuf = std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Eq)]
/// Kind of object in a tree.
pub enum Kind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Reference to another tree.
    TreeReference,
}

impl Kind {
    /// Get a marker string for this kind of tree object.
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
            n => Err(format!("Invalid kind: {}", n)),
        }
    }
}

impl<'py> pyo3::IntoPyObject<'py> for Kind {
    type Target = pyo3::PyAny;
    type Output = pyo3::Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = match self {
            Kind::File => "file",
            Kind::Directory => "directory",
            Kind::Symlink => "symlink",
            Kind::TreeReference => "tree-reference",
        };
        Ok(pyo3::types::PyString::new(py, s).into_any())
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

/// A tree entry, representing different types of objects in a tree.
pub enum TreeEntry {
    /// A regular file entry.
    File {
        /// Whether the file is executable.
        executable: bool,
        /// The kind of file.
        kind: Kind,
        /// The revision ID that introduced this file, if known.
        revision: Option<RevisionId>,
        /// The size of the file in bytes.
        size: u64,
    },
    /// A directory entry.
    Directory {
        /// The revision ID that introduced this directory, if known.
        revision: Option<RevisionId>,
    },
    /// A symbolic link entry.
    Symlink {
        /// The revision ID that introduced this symlink, if known.
        revision: Option<RevisionId>,
        /// The target path of the symbolic link.
        symlink_target: String,
    },
    /// A reference to another tree.
    TreeReference {
        /// The revision ID that introduced this reference, if known.
        revision: Option<RevisionId>,
        /// The revision ID this reference points to.
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

/// The core tree interface that provides access to content and metadata.
///
/// A tree represents a structured collection of files that can be
/// read, modified, and compared, depending on the implementation.
pub trait Tree {
    /// Get a dictionary of tags and their revision IDs.
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error>;
    /// Get a file from the tree as a readable stream.
    fn get_file(&self, path: &Path) -> Result<Box<dyn std::io::Read>, Error>;
    /// Get the contents of a file from the tree as a byte vector.
    fn get_file_text(&self, path: &Path) -> Result<Vec<u8>, Error>;
    /// Get the contents of a file as a vector of lines (byte vectors).
    fn get_file_lines(&self, path: &Path) -> Result<Vec<Vec<u8>>, Error>;
    /// Lock the tree for read operations.
    fn lock_read(&self) -> Result<Lock, Error>;

    /// Check if a file exists in the tree at the specified path.
    fn has_filename(&self, path: &Path) -> bool;

    /// Get the target of a symbolic link.
    fn get_symlink_target(&self, path: &Path) -> Result<PathBuf, Error>;

    /// Get the IDs of the parent revisions of this tree.
    fn get_parent_ids(&self) -> Result<Vec<RevisionId>, Error>;
    /// Check if a path is ignored by version control.
    fn is_ignored(&self, path: &Path) -> Option<String>;
    /// Get the kind of object at the specified path (file, directory, symlink, etc.).
    fn kind(&self, path: &Path) -> Result<Kind, Error>;
    /// Check if a path is under version control.
    fn is_versioned(&self, path: &Path) -> bool;

    /// Iterate through the changes between this tree and another tree.
    ///
    /// # Arguments
    /// * `other` - The other tree to compare against
    /// * `specific_files` - Optional list of specific files to check
    /// * `want_unversioned` - Whether to include unversioned files
    /// * `require_versioned` - Whether to require files to be versioned
    fn iter_changes(
        &self,
        other: &dyn PyTree,
        specific_files: Option<&[&Path]>,
        want_unversioned: Option<bool>,
        require_versioned: Option<bool>,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeChange, Error>>>, Error>;

    /// Check if this tree supports versioned directories.
    fn has_versioned_directories(&self) -> bool;

    /// Get a preview of transformations that would be applied to this tree.
    fn preview_transform(&self) -> Result<crate::transform::TreeTransform, Error>;

    /// List files in the tree, optionally recursively.
    ///
    /// # Arguments
    /// * `include_root` - Whether to include the root directory
    /// * `from_dir` - Starting directory (if not the root)
    /// * `recursive` - Whether to recurse into subdirectories
    /// * `recurse_nested` - Whether to recurse into nested trees
    fn list_files(
        &self,
        include_root: Option<bool>,
        from_dir: Option<&Path>,
        recursive: Option<bool>,
        recurse_nested: Option<bool>,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, bool, Kind, TreeEntry), Error>>>, Error>;

    /// Iterate through entries in a directory.
    ///
    /// # Arguments
    /// * `path` - Path to the directory to list
    fn iter_child_entries(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, Kind, TreeEntry), Error>>>, Error>;
}

/// Trait for Python tree objects that can be converted to and from Python objects.
///
/// This trait is implemented by all tree types that wrap Python objects.
pub trait PyTree: std::any::Any {
    /// Get the underlying Python object for this tree.
    fn to_object(&self, py: Python) -> PyObject;
}

impl<T: PyTree + ?Sized> Tree for T {
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, intern!(py, "get_tag_dict"))?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    fn get_file(&self, path: &Path) -> Result<Box<dyn std::io::Read>, Error> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            let f = self
                .to_object(py)
                .call_method1(py, "get_file", (path_str,))?;

            let f = pyo3_filelike::PyBinaryFile::from(f);

            Ok(Box::new(f) as Box<dyn std::io::Read>)
        })
    }

    fn get_file_text(&self, path: &Path) -> Result<Vec<u8>, Error> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            let text = self
                .to_object(py)
                .call_method1(py, "get_file_text", (path_str,))?;
            text.extract(py).map_err(|e| e.into())
        })
    }

    fn get_file_lines(&self, path: &Path) -> Result<Vec<Vec<u8>>, Error> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            let lines = self
                .to_object(py)
                .call_method1(py, "get_file_lines", (path_str,))?;
            lines.extract(py).map_err(|e| e.into())
        })
    }

    fn lock_read(&self) -> Result<Lock, Error> {
        Python::with_gil(|py| {
            let lock = self
                .to_object(py)
                .call_method0(py, intern!(py, "lock_read"))?;
            Ok(Lock::from(lock))
        })
    }

    fn has_filename(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, intern!(py, "has_filename"), (path_str,))
                .and_then(|result| result.extract(py))
                .unwrap_or(false)
        })
    }

    fn get_symlink_target(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            let target = self
                .to_object(py)
                .call_method1(py, "get_symlink_target", (path_str,))?;
            target.extract(py).map_err(|e| e.into())
        })
    }

    fn get_parent_ids(&self) -> Result<Vec<RevisionId>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, intern!(py, "get_parent_ids"))
                .unwrap()
                .extract(py)?)
        })
    }

    fn is_ignored(&self, path: &Path) -> Option<String> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "is_ignored", (path_str,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn kind(&self, path: &Path) -> Result<Kind, Error> {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "kind", (path_str,))
                .unwrap()
                .extract(py)
                .map_err(|e| e.into())
        })
    }

    fn is_versioned(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "is_versioned", (path_str,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn iter_changes(
        &self,
        other: &dyn PyTree,
        specific_files: Option<&[&Path]>,
        want_unversioned: Option<bool>,
        require_versioned: Option<bool>,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeChange, Error>>>, Error> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
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
                        let next = match self.0.call_method0(py, intern!(py, "__next__")) {
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

            Ok(Box::new(TreeChangeIter(self.to_object(py).call_method(
                py,
                "iter_changes",
                (other.to_object(py),),
                Some(&kwargs),
            )?))
                as Box<dyn Iterator<Item = Result<TreeChange, Error>>>)
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
            let kwargs = pyo3::types::PyDict::new(py);
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
                        let next = match self.0.call_method0(py, intern!(py, "__next__")) {
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

            Ok(Box::new(ListFilesIter(self.to_object(py).call_method(
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
                        let next = match self.0.call_method0(py, intern!(py, "__next__")) {
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

            let path_str = path.to_string_lossy().to_string();
            Ok(
                Box::new(IterChildEntriesIter(self.to_object(py).call_method1(
                    py,
                    "iter_child_entries",
                    (path_str,),
                )?))
                    as Box<dyn Iterator<Item = Result<(PathBuf, Kind, TreeEntry), Error>>>,
            )
        })
    }
}

/// A generic tree implementation that wraps any Python tree object.
pub struct GenericTree(PyObject);

impl<'py> IntoPyObject<'py> for GenericTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl From<PyObject> for GenericTree {
    fn from(obj: PyObject) -> Self {
        GenericTree(obj)
    }
}

impl PyTree for GenericTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

/// Trait for trees that support modification operations.
pub trait MutableTree: Tree {
    /// Add specified files to version control.
    fn add(&self, files: &[&Path]) -> Result<(), Error>;
    /// Lock the tree for write operations.
    fn lock_write(&self) -> Result<Lock, Error>;
    /// Write bytes to a file in the tree without atomic guarantees.
    fn put_file_bytes_non_atomic(&self, path: &Path, data: &[u8]) -> Result<(), Error>;
    /// Check if the tree has any uncommitted changes.
    fn has_changes(&self) -> std::result::Result<bool, Error>;
    /// Create a directory in the tree.
    fn mkdir(&self, path: &Path) -> Result<(), Error>;
    /// Remove specified files from version control and from the filesystem.
    fn remove(&self, files: &[&std::path::Path]) -> Result<(), Error>;
    /// Get this object as a reference to the Tree trait.
    fn as_tree(&self) -> &dyn Tree
    where
        Self: Sized;
}

/// A tree that can be modified.
pub trait PyMutableTree: PyTree {}

impl<T: PyMutableTree + ?Sized> MutableTree for T {
    fn add(&self, files: &[&Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::with_gil(|py| -> Result<(), PyErr> {
            let path_strings: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "add", (path_strings,))?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn lock_write(&self) -> Result<Lock, Error> {
        Python::with_gil(|py| {
            let lock = self
                .to_object(py)
                .call_method0(py, intern!(py, "lock_write"))?;
            Ok(Lock::from(lock))
        })
    }

    fn put_file_bytes_non_atomic(&self, path: &Path, data: &[u8]) -> Result<(), Error> {
        assert!(path.is_relative());
        Python::with_gil(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "put_file_bytes_non_atomic", (path_str, data))?;
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
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py).call_method1(py, "mkdir", (path_str,))?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn remove(&self, files: &[&std::path::Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::with_gil(|py| -> Result<(), PyErr> {
            let path_strings: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "remove", (path_strings,))?;
            Ok(())
        })
        .map_err(|e| e.into())
    }

    fn as_tree(&self) -> &dyn Tree
    where
        Self: Sized,
    {
        self
    }
}

/// A read-only tree at a specific revision.
pub struct RevisionTree(pub PyObject);

impl<'py> IntoPyObject<'py> for RevisionTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl PyTree for RevisionTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl Clone for RevisionTree {
    fn clone(&self) -> Self {
        Python::with_gil(|py| RevisionTree(self.0.clone_ref(py)))
    }
}

impl RevisionTree {
    /// Get the repository this revision tree belongs to.
    pub fn repository(&self) -> crate::repository::GenericRepository {
        Python::with_gil(|py| {
            let repository = self.to_object(py).getattr(py, "_repository").unwrap();
            crate::repository::GenericRepository::new(repository)
        })
    }

    /// Get the revision ID of this tree.
    pub fn get_revision_id(&self) -> RevisionId {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_revision_id")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Get the parent revision IDs of this tree.
    pub fn get_parent_ids(&self) -> Vec<RevisionId> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, intern!(py, "get_parent_ids"))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a change to a file in a tree.
pub struct TreeChange {
    /// The path of the file, as (old_path, new_path).
    pub path: (Option<PathBuf>, Option<PathBuf>),
    /// Whether the content of the file changed.
    pub changed_content: bool,
    /// Whether the file is versioned, as (old_versioned, new_versioned).
    pub versioned: (Option<bool>, Option<bool>),
    /// The name of the file, as (old_name, new_name).
    pub name: (Option<std::ffi::OsString>, Option<std::ffi::OsString>),
    /// The kind of the file, as (old_kind, new_kind).
    pub kind: (Option<Kind>, Option<Kind>),
    /// Whether the file is executable, as (old_executable, new_executable).
    pub executable: (Option<bool>, Option<bool>),
    /// Whether the file was copied rather than just changed/renamed.
    pub copied: bool,
}

impl<'py> IntoPyObject<'py> for TreeChange {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("path", &self.path).unwrap();
        dict.set_item("changed_content", self.changed_content)
            .unwrap();
        dict.set_item("versioned", self.versioned).unwrap();
        dict.set_item("name", &self.name).unwrap();
        dict.set_item("kind", self.kind.clone()).unwrap();
        dict.set_item("executable", self.executable).unwrap();
        dict.set_item("copied", self.copied).unwrap();
        Ok(dict.into_any())
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

/// An in-memory tree implementation.
pub struct MemoryTree(pub PyObject);

impl<'py> IntoPyObject<'py> for MemoryTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<B: crate::branch::PyBranch> From<&B> for MemoryTree {
    fn from(branch: &B) -> Self {
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

impl PyTree for MemoryTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl PyMutableTree for MemoryTree {}

pub use crate::workingtree::WorkingTree;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::{create_standalone_workingtree, ControlDirFormat};
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_remove() {
        let env = crate::testing::TestEnv::new();
        let wt =
            create_standalone_workingtree(std::path::Path::new("."), &ControlDirFormat::default())
                .unwrap();
        let path = std::path::Path::new("foo");
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
        std::mem::drop(env);
    }
}
