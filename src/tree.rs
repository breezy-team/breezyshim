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

/// Result of walking directories in a tree.
#[derive(Debug)]
pub struct WalkdirResult {
    /// The path relative to the tree root.
    pub relpath: PathBuf,
    /// The absolute path.
    pub abspath: PathBuf,
    /// The kind of the entry.
    pub kind: Kind,
    /// The stat information for the entry.
    pub stat: Option<std::fs::Metadata>,
    /// Whether the entry is versioned.
    pub versioned: bool,
}

/// Summary of path content.
#[derive(Debug)]
pub struct PathContentSummary {
    /// The kind of the content.
    pub kind: Kind,
    /// The size in bytes (for files).
    pub size: Option<u64>,
    /// Whether the file is executable.
    pub executable: Option<bool>,
    /// The SHA1 hash (for files).
    pub sha1: Option<String>,
    /// The target (for symlinks).
    pub target: Option<String>,
}

/// Search rule for path matching.
#[derive(Debug)]
pub struct SearchRule {
    /// The pattern to match.
    pub pattern: String,
    /// The type of rule.
    pub rule_type: SearchRuleType,
}

/// Type of search rule.
#[derive(Debug)]
pub enum SearchRuleType {
    /// Include the matched paths.
    Include,
    /// Exclude the matched paths.
    Exclude,
}

/// Represents a conflict in a tree.
#[derive(Debug)]
pub struct Conflict {
    /// The path involved in the conflict.
    pub path: PathBuf,
    /// The type of conflict.
    pub conflict_type: String,
    /// Additional information about the conflict.
    pub message: Option<String>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Conflict {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let path: String = ob.getattr("path")?.extract()?;
        let conflict_type: String = ob.getattr("typestring")?.extract()?;
        let message: Option<String> = ob.getattr("message").ok().and_then(|m| m.extract().ok());

        Ok(Conflict {
            path: PathBuf::from(path),
            conflict_type,
            message,
        })
    }
}

/// Represents a tree reference.
#[derive(Debug)]
pub struct TreeReference {
    /// The path where the reference should be added.
    pub path: PathBuf,
    /// The kind of reference.
    pub kind: Kind,
    /// The reference revision.
    pub reference_revision: Option<RevisionId>,
}

/// Represents a change in the inventory.
#[derive(Debug)]
pub struct InventoryDelta {
    /// The old path (None if new).
    pub old_path: Option<PathBuf>,
    /// The new path (None if deleted).
    pub new_path: Option<PathBuf>,
    /// The file ID.
    pub file_id: String,
    /// The entry details.
    pub entry: Option<TreeEntry>,
}

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

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for Kind {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, pyo3::PyAny>) -> PyResult<Self> {
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
#[derive(Debug)]
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

impl<'a, 'py> FromPyObject<'a, 'py> for TreeEntry {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
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

impl<'py> IntoPyObject<'py> for TreeEntry {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = pyo3::types::PyDict::new(py);
        match self {
            TreeEntry::File {
                executable,
                kind: _,
                revision,
                size,
            } => {
                dict.set_item("kind", "file").unwrap();
                dict.set_item("executable", executable).unwrap();
                dict.set_item("size", size).unwrap();
                dict.set_item("revision", revision).unwrap();
            }
            TreeEntry::Directory { revision } => {
                dict.set_item("kind", "directory").unwrap();
                dict.set_item("revision", revision).unwrap();
            }
            TreeEntry::Symlink {
                revision,
                symlink_target,
            } => {
                dict.set_item("kind", "symlink").unwrap();
                dict.set_item("revision", revision).unwrap();
                dict.set_item("symlink_target", symlink_target).unwrap();
            }
            TreeEntry::TreeReference {
                revision,
                reference_revision,
            } => {
                dict.set_item("kind", "tree-reference").unwrap();
                dict.set_item("revision", revision).unwrap();
                dict.set_item("reference_revision", reference_revision)
                    .unwrap();
            }
        }
        Ok(dict.into_any())
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

    /// Get the size of a file in bytes.
    fn get_file_size(&self, path: &Path) -> Result<u64, Error>;

    /// Get the SHA1 hash of a file's contents.
    fn get_file_sha1(
        &self,
        path: &Path,
        stat_value: Option<&std::fs::Metadata>,
    ) -> Result<String, Error>;

    /// Get the modification time of a file.
    fn get_file_mtime(&self, path: &Path) -> Result<u64, Error>;

    /// Check if a file is executable.
    fn is_executable(&self, path: &Path) -> Result<bool, Error>;

    /// Get the stored kind of a file (as opposed to the actual kind on disk).
    fn stored_kind(&self, path: &Path) -> Result<Kind, Error>;

    /// Check if the tree supports content filtering.
    fn supports_content_filtering(&self) -> bool;

    /// Check if the tree supports file IDs.
    fn supports_file_ids(&self) -> bool;

    /// Check if the tree supports rename tracking.
    fn supports_rename_tracking(&self) -> bool;

    /// Check if the tree supports symbolic links.
    fn supports_symlinks(&self) -> bool;

    /// Check if the tree supports tree references.
    fn supports_tree_reference(&self) -> bool;

    /// Get unversioned files in the tree.
    fn unknowns(&self) -> Result<Vec<PathBuf>, Error>;

    /// Get all versioned paths in the tree.
    fn all_versioned_paths(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<PathBuf, Error>>>, Error>;

    /// Get conflicts in the tree.
    fn conflicts(&self) -> Result<Vec<Conflict>, Error>;

    /// Get extra (unversioned) files in the tree.
    fn extras(&self) -> Result<Vec<PathBuf>, Error>;

    /// Filter out versioned files from a list of paths.
    fn filter_unversioned_files(&self, paths: &[&Path]) -> Result<Vec<PathBuf>, Error>;

    /// Walk directories in the tree.
    fn walkdirs(
        &self,
        prefix: Option<&Path>,
    ) -> Result<Box<dyn Iterator<Item = Result<WalkdirResult, Error>>>, Error>;

    /// Check if a file kind is versionable.
    fn versionable_kind(&self, kind: &Kind) -> bool;

    /// Get file content summary for a path.
    fn path_content_summary(&self, path: &Path) -> Result<PathContentSummary, Error>;

    /// Iterate through file bytes.
    fn iter_files_bytes(
        &self,
        paths: &[&Path],
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, Vec<u8>), Error>>>, Error>;

    /// Iterate through entries by directory.
    fn iter_entries_by_dir(
        &self,
        specific_files: Option<&[&Path]>,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, TreeEntry), Error>>>, Error>;

    /// Get file verifier information.
    fn get_file_verifier(
        &self,
        path: &Path,
        stat_value: Option<&std::fs::Metadata>,
    ) -> Result<(String, Vec<u8>), Error>;

    /// Get the reference revision for a tree reference.
    fn get_reference_revision(&self, path: &Path) -> Result<RevisionId, Error>;

    /// Create an archive of the tree.
    fn archive(
        &self,
        format: &str,
        name: &str,
        root: Option<&str>,
        subdir: Option<&Path>,
        force_mtime: Option<f64>,
        recurse_nested: bool,
    ) -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>>>, Error>;

    /// Annotate a file with revision information.
    fn annotate_iter(
        &self,
        path: &Path,
        default_revision: Option<&RevisionId>,
    ) -> Result<Box<dyn Iterator<Item = Result<(RevisionId, Vec<u8>), Error>>>, Error>;

    /// Check if a path is a special path (e.g., control directory).
    fn is_special_path(&self, path: &Path) -> bool;

    /// Iterate through search rules.
    fn iter_search_rules(
        &self,
        paths: &[&Path],
    ) -> Result<Box<dyn Iterator<Item = Result<SearchRule, Error>>>, Error>;
}

/// Trait for Python tree objects that can be converted to and from Python objects.
///
/// This trait is implemented by all tree types that wrap Python objects.
pub trait PyTree: Tree + std::any::Any {
    /// Get the underlying Python object for this tree.
    fn to_object(&self, py: Python) -> Py<PyAny>;
}

impl dyn PyTree {
    /// Get a reference to self as a Tree trait object.
    pub fn as_tree(&self) -> &dyn Tree {
        self
    }
}

impl<T: PyTree + ?Sized> Tree for T {
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::attach(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, intern!(py, "get_tag_dict"))?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    fn get_file(&self, path: &Path) -> Result<Box<dyn std::io::Read>, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let f = self
                .to_object(py)
                .call_method1(py, "get_file", (path_str,))?;

            let f = pyo3_filelike::PyBinaryFile::from(f);

            Ok(Box::new(f) as Box<dyn std::io::Read>)
        })
    }

    fn get_file_text(&self, path: &Path) -> Result<Vec<u8>, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let text = self
                .to_object(py)
                .call_method1(py, "get_file_text", (path_str,))?;
            text.extract(py).map_err(Into::into)
        })
    }

    fn get_file_lines(&self, path: &Path) -> Result<Vec<Vec<u8>>, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let lines = self
                .to_object(py)
                .call_method1(py, "get_file_lines", (path_str,))?;
            lines.extract(py).map_err(Into::into)
        })
    }

    fn lock_read(&self) -> Result<Lock, Error> {
        Python::attach(|py| {
            let lock = self
                .to_object(py)
                .call_method0(py, intern!(py, "lock_read"))?;
            Ok(Lock::from(lock))
        })
    }

    fn has_filename(&self, path: &Path) -> bool {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, intern!(py, "has_filename"), (path_str,))
                .and_then(|result| result.extract(py))
                .unwrap_or(false)
        })
    }

    fn get_symlink_target(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let target = self
                .to_object(py)
                .call_method1(py, "get_symlink_target", (path_str,))?;
            target.extract(py).map_err(Into::into)
        })
    }

    fn get_parent_ids(&self) -> Result<Vec<RevisionId>, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, intern!(py, "get_parent_ids"))
                .unwrap()
                .extract(py)?)
        })
    }

    fn is_ignored(&self, path: &Path) -> Option<String> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "is_ignored", (path_str,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn kind(&self, path: &Path) -> Result<Kind, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "kind", (path_str,))
                .unwrap()
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn is_versioned(&self, path: &Path) -> bool {
        Python::attach(|py| {
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
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(specific_files) = specific_files {
                kwargs.set_item(
                    "specific_files",
                    specific_files
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect::<Vec<_>>(),
                )?;
            }
            if let Some(want_unversioned) = want_unversioned {
                kwargs.set_item("want_unversioned", want_unversioned)?;
            }
            if let Some(require_versioned) = require_versioned {
                kwargs.set_item("require_versioned", require_versioned)?;
            }
            struct TreeChangeIter(pyo3::Py<PyAny>);

            impl Iterator for TreeChangeIter {
                type Item = Result<TreeChange, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
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
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "has_versioned_directories")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn preview_transform(&self) -> Result<crate::transform::TreeTransform, Error> {
        Python::attach(|py| {
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
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(include_root) = include_root {
                kwargs.set_item("include_root", include_root)?;
            }
            if let Some(from_dir) = from_dir {
                kwargs.set_item("from_dir", from_dir.to_string_lossy().to_string())?;
            }
            if let Some(recursive) = recursive {
                kwargs.set_item("recursive", recursive)?;
            }
            if let Some(recurse_nested) = recurse_nested {
                kwargs.set_item("recurse_nested", recurse_nested)?;
            }
            struct ListFilesIter(pyo3::Py<PyAny>);

            impl Iterator for ListFilesIter {
                type Item = Result<(PathBuf, bool, Kind, TreeEntry), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
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
        Python::attach(|py| {
            struct IterChildEntriesIter(pyo3::Py<PyAny>);

            impl Iterator for IterChildEntriesIter {
                type Item = Result<(PathBuf, Kind, TreeEntry), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
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

    fn get_file_size(&self, path: &Path) -> Result<u64, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let size = self
                .to_object(py)
                .call_method1(py, "get_file_size", (path_str,))?;
            size.extract(py).map_err(Into::into)
        })
    }

    fn get_file_sha1(
        &self,
        path: &Path,
        _stat_value: Option<&std::fs::Metadata>,
    ) -> Result<String, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let sha1 = self
                .to_object(py)
                .call_method1(py, "get_file_sha1", (path_str,))?;
            sha1.extract(py).map_err(Into::into)
        })
    }

    fn get_file_mtime(&self, path: &Path) -> Result<u64, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let mtime = self
                .to_object(py)
                .call_method1(py, "get_file_mtime", (path_str,))?;
            mtime.extract(py).map_err(Into::into)
        })
    }

    fn is_executable(&self, path: &Path) -> Result<bool, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let result = self
                .to_object(py)
                .call_method1(py, "is_executable", (path_str,))?;
            result.extract(py).map_err(Into::into)
        })
    }

    fn stored_kind(&self, path: &Path) -> Result<Kind, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "stored_kind", (path_str,))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn supports_content_filtering(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_content_filtering")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn supports_file_ids(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn supports_rename_tracking(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_rename_tracking")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn supports_symlinks(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_symlinks")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn supports_tree_reference(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_tree_reference")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn unknowns(&self) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            let unknowns = self.to_object(py).call_method0(py, "unknowns")?;
            unknowns.extract(py).map_err(Into::into)
        })
    }

    fn all_versioned_paths(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<PathBuf, Error>>>, Error> {
        Python::attach(|py| {
            struct AllVersionedPathsIter(pyo3::Py<PyAny>);

            impl Iterator for AllVersionedPathsIter {
                type Item = Result<PathBuf, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
                        }
                    })
                }
            }

            Ok(Box::new(AllVersionedPathsIter(
                self.to_object(py).call_method0(py, "all_versioned_paths")?,
            ))
                as Box<dyn Iterator<Item = Result<PathBuf, Error>>>)
        })
    }

    fn conflicts(&self) -> Result<Vec<Conflict>, Error> {
        Python::attach(|py| {
            let conflicts = self.to_object(py).call_method0(py, "conflicts")?;
            conflicts.extract(py).map_err(Into::into)
        })
    }

    fn extras(&self) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            let extras = self.to_object(py).call_method0(py, "extras")?;
            extras.extract(py).map_err(Into::into)
        })
    }

    fn filter_unversioned_files(&self, paths: &[&Path]) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            let path_strings: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let result =
                self.to_object(py)
                    .call_method1(py, "filter_unversioned_files", (path_strings,))?;
            result.extract(py).map_err(Into::into)
        })
    }

    fn walkdirs(
        &self,
        prefix: Option<&Path>,
    ) -> Result<Box<dyn Iterator<Item = Result<WalkdirResult, Error>>>, Error> {
        Python::attach(|py| {
            struct WalkdirsIter(pyo3::Py<PyAny>);

            impl Iterator for WalkdirsIter {
                type Item = Result<WalkdirResult, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            let tuple = match next
                                .extract::<(String, String, String, Option<Py<PyAny>>, bool)>(py)
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e.into())),
                            };

                            Some(Ok(WalkdirResult {
                                relpath: PathBuf::from(tuple.0),
                                abspath: PathBuf::from(tuple.1),
                                kind: tuple.2.parse().unwrap(),
                                stat: None, // TODO: convert Python stat to Rust metadata
                                versioned: tuple.4,
                            }))
                        }
                    })
                }
            }

            let prefix_str = prefix.map(|p| p.to_string_lossy().to_string());
            Ok(Box::new(WalkdirsIter(self.to_object(py).call_method1(
                py,
                "walkdirs",
                (prefix_str,),
            )?))
                as Box<dyn Iterator<Item = Result<WalkdirResult, Error>>>)
        })
    }

    fn versionable_kind(&self, kind: &Kind) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "versionable_kind", (kind.clone(),))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn path_content_summary(&self, path: &Path) -> Result<PathContentSummary, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let summary =
                self.to_object(py)
                    .call_method1(py, "path_content_summary", (path_str,))?;

            let summary_bound = summary.bind(py);
            let kind: String = summary_bound.get_item("kind")?.extract()?;
            let size: Option<u64> = summary_bound
                .get_item("size")
                .ok()
                .map(|v| v.extract().expect("size should be u64"));
            let executable: Option<bool> = summary_bound
                .get_item("executable")
                .ok()
                .map(|v| v.extract().expect("executable should be bool"));
            let sha1: Option<String> = summary_bound
                .get_item("sha1")
                .ok()
                .map(|v| v.extract().expect("sha1 should be string"));
            let target: Option<String> = summary_bound
                .get_item("target")
                .ok()
                .map(|v| v.extract().expect("target should be string"));

            Ok(PathContentSummary {
                kind: kind.parse().unwrap(),
                size,
                executable,
                sha1,
                target,
            })
        })
    }

    fn iter_files_bytes(
        &self,
        paths: &[&Path],
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, Vec<u8>), Error>>>, Error> {
        Python::attach(|py| {
            struct IterFilesBytesIter(pyo3::Py<PyAny>);

            impl Iterator for IterFilesBytesIter {
                type Item = Result<(PathBuf, Vec<u8>), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
                        }
                    })
                }
            }

            let path_strings: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            Ok(Box::new(IterFilesBytesIter(self.to_object(py).call_method1(
                py,
                "iter_files_bytes",
                (path_strings,),
            )?))
                as Box<
                    dyn Iterator<Item = Result<(PathBuf, Vec<u8>), Error>>,
                >)
        })
    }

    fn iter_entries_by_dir(
        &self,
        specific_files: Option<&[&Path]>,
    ) -> Result<Box<dyn Iterator<Item = Result<(PathBuf, TreeEntry), Error>>>, Error> {
        Python::attach(|py| {
            struct IterEntriesByDirIter(pyo3::Py<PyAny>);

            impl Iterator for IterEntriesByDirIter {
                type Item = Result<(PathBuf, TreeEntry), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
                        }
                    })
                }
            }

            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(specific_files) = specific_files {
                let path_strings: Vec<String> = specific_files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                kwargs.set_item("specific_files", path_strings)?;
            }

            Ok(
                Box::new(IterEntriesByDirIter(self.to_object(py).call_method(
                    py,
                    "iter_entries_by_dir",
                    (),
                    Some(&kwargs),
                )?))
                    as Box<dyn Iterator<Item = Result<(PathBuf, TreeEntry), Error>>>,
            )
        })
    }

    fn get_file_verifier(
        &self,
        path: &Path,
        _stat_value: Option<&std::fs::Metadata>,
    ) -> Result<(String, Vec<u8>), Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let result = self
                .to_object(py)
                .call_method1(py, "get_file_verifier", (path_str,))?;
            result.extract(py).map_err(Into::into)
        })
    }

    fn get_reference_revision(&self, path: &Path) -> Result<RevisionId, Error> {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            let rev = self
                .to_object(py)
                .call_method1(py, "get_reference_revision", (path_str,))?;
            rev.extract(py).map_err(Into::into)
        })
    }

    fn archive(
        &self,
        format: &str,
        name: &str,
        root: Option<&str>,
        subdir: Option<&Path>,
        force_mtime: Option<f64>,
        recurse_nested: bool,
    ) -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>>>, Error> {
        Python::attach(|py| {
            struct ArchiveIter(pyo3::Py<PyAny>);

            impl Iterator for ArchiveIter {
                type Item = Result<Vec<u8>, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
                        }
                    })
                }
            }

            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("format", format)?;
            kwargs.set_item("name", name)?;
            if let Some(root) = root {
                kwargs.set_item("root", root)?;
            }
            if let Some(subdir) = subdir {
                kwargs.set_item("subdir", subdir.to_string_lossy().to_string())?;
            }
            if let Some(force_mtime) = force_mtime {
                kwargs.set_item("force_mtime", force_mtime)?;
            }
            kwargs.set_item("recurse_nested", recurse_nested)?;

            Ok(Box::new(ArchiveIter(self.to_object(py).call_method(
                py,
                "archive",
                (),
                Some(&kwargs),
            )?))
                as Box<dyn Iterator<Item = Result<Vec<u8>, Error>>>)
        })
    }

    fn annotate_iter(
        &self,
        path: &Path,
        default_revision: Option<&RevisionId>,
    ) -> Result<Box<dyn Iterator<Item = Result<(RevisionId, Vec<u8>), Error>>>, Error> {
        Python::attach(|py| {
            struct AnnotateIter(pyo3::Py<PyAny>);

            impl Iterator for AnnotateIter {
                type Item = Result<(RevisionId, Vec<u8>), Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            Some(next.extract(py).map_err(Into::into))
                        }
                    })
                }
            }

            let path_str = path.to_string_lossy().to_string();
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(default_revision) = default_revision {
                kwargs.set_item(
                    "default_revision",
                    default_revision.clone().into_pyobject(py).unwrap(),
                )?;
            }

            Ok(Box::new(AnnotateIter(self.to_object(py).call_method(
                py,
                "annotate_iter",
                (path_str,),
                Some(&kwargs),
            )?))
                as Box<
                    dyn Iterator<Item = Result<(RevisionId, Vec<u8>), Error>>,
                >)
        })
    }

    fn is_special_path(&self, path: &Path) -> bool {
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "is_special_path", (path_str,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn iter_search_rules(
        &self,
        paths: &[&Path],
    ) -> Result<Box<dyn Iterator<Item = Result<SearchRule, Error>>>, Error> {
        Python::attach(|py| {
            struct IterSearchRulesIter(pyo3::Py<PyAny>);

            impl Iterator for IterSearchRulesIter {
                type Item = Result<SearchRule, Error>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::attach(|py| {
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
                            let tuple = match next.extract::<(String, String)>(py) {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e.into())),
                            };

                            let rule_type = match tuple.1.as_str() {
                                "include" => SearchRuleType::Include,
                                "exclude" => SearchRuleType::Exclude,
                                _ => {
                                    return Some(Err(Error::Other(PyErr::new::<
                                        pyo3::exceptions::PyValueError,
                                        _,
                                    >(
                                        "Unknown search rule type"
                                    ))))
                                }
                            };

                            Some(Ok(SearchRule {
                                pattern: tuple.0,
                                rule_type,
                            }))
                        }
                    })
                }
            }

            let path_strings: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            Ok(
                Box::new(IterSearchRulesIter(self.to_object(py).call_method1(
                    py,
                    "iter_search_rules",
                    (path_strings,),
                )?)) as Box<dyn Iterator<Item = Result<SearchRule, Error>>>,
            )
        })
    }
}

/// A generic tree implementation that wraps any Python tree object.
pub struct GenericTree(Py<PyAny>);

impl<'py> IntoPyObject<'py> for GenericTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl From<Py<PyAny>> for GenericTree {
    fn from(obj: Py<PyAny>) -> Self {
        GenericTree(obj)
    }
}

impl PyTree for GenericTree {
    fn to_object(&self, py: Python) -> Py<PyAny> {
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

    /// Add a tree reference.
    fn add_reference(&self, reference: &TreeReference) -> Result<(), Error>;

    /// Copy a file or directory to a new location.
    fn copy_one(&self, from_path: &Path, to_path: &Path) -> Result<(), Error>;

    /// Get the last revision ID.
    fn last_revision(&self) -> Result<RevisionId, Error>;

    /// Lock the tree for write operations.
    fn lock_tree_write(&self) -> Result<Lock, Error>;

    /// Set the parent IDs for this tree.
    fn set_parent_ids(&self, parent_ids: &[RevisionId]) -> Result<(), Error>;

    /// Set the parent trees for this tree.
    fn set_parent_trees(&self, parent_trees: &[(RevisionId, RevisionTree)]) -> Result<(), Error>;

    /// Apply a delta to the tree.
    fn apply_inventory_delta(&self, delta: Vec<InventoryDelta>) -> Result<(), Error>;

    /// Commit changes in the tree.
    fn commit(
        &self,
        message: &str,
        committer: Option<&str>,
        timestamp: Option<f64>,
        allow_pointless: Option<bool>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error>;
}

/// A tree that can be modified.
pub trait PyMutableTree: PyTree + MutableTree {}

impl dyn PyMutableTree {
    /// Get a reference to self as a MutableTree trait object.
    pub fn as_mutable_tree(&self) -> &dyn MutableTree {
        self
    }
}

impl<T: PyMutableTree + ?Sized> MutableTree for T {
    fn add(&self, files: &[&Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::attach(|py| -> Result<(), PyErr> {
            let path_strings: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "add", (path_strings,))?;
            Ok(())
        })
        .map_err(Into::into)
    }

    fn lock_write(&self) -> Result<Lock, Error> {
        Python::attach(|py| {
            let lock = self
                .to_object(py)
                .call_method0(py, intern!(py, "lock_write"))?;
            Ok(Lock::from(lock))
        })
    }

    fn put_file_bytes_non_atomic(&self, path: &Path, data: &[u8]) -> Result<(), Error> {
        assert!(path.is_relative());
        Python::attach(|py| {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "put_file_bytes_non_atomic", (path_str, data))?;
            Ok(())
        })
    }

    fn has_changes(&self) -> std::result::Result<bool, Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "has_changes")?
                .extract::<bool>(py)
                .map_err(Into::into)
        })
    }

    fn mkdir(&self, path: &Path) -> Result<(), Error> {
        assert!(path.is_relative());
        Python::attach(|py| -> Result<(), PyErr> {
            let path_str = path.to_string_lossy().to_string();
            self.to_object(py).call_method1(py, "mkdir", (path_str,))?;
            Ok(())
        })
        .map_err(Into::into)
    }

    fn remove(&self, files: &[&std::path::Path]) -> Result<(), Error> {
        for f in files {
            assert!(f.is_relative());
        }
        Python::attach(|py| -> Result<(), PyErr> {
            let path_strings: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "remove", (path_strings,))?;
            Ok(())
        })
        .map_err(Into::into)
    }

    fn add_reference(&self, reference: &TreeReference) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("path", reference.path.to_string_lossy().to_string())?;
            kwargs.set_item("kind", reference.kind.clone())?;
            if let Some(ref rev) = reference.reference_revision {
                kwargs.set_item("reference_revision", rev.clone().into_pyobject(py).unwrap())?;
            }
            self.to_object(py)
                .call_method(py, "add_reference", (), Some(&kwargs))?;
            Ok(())
        })
    }

    fn copy_one(&self, from_path: &Path, to_path: &Path) -> Result<(), Error> {
        assert!(from_path.is_relative());
        assert!(to_path.is_relative());
        Python::attach(|py| {
            let from_str = from_path.to_string_lossy().to_string();
            let to_str = to_path.to_string_lossy().to_string();
            self.to_object(py)
                .call_method1(py, "copy_one", (from_str, to_str))?;
            Ok(())
        })
    }

    fn last_revision(&self) -> Result<RevisionId, Error> {
        Python::attach(|py| {
            let last_revision = self
                .to_object(py)
                .call_method0(py, intern!(py, "last_revision"))?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }

    fn lock_tree_write(&self) -> Result<Lock, Error> {
        Python::attach(|py| {
            let lock = self.to_object(py).call_method0(py, "lock_tree_write")?;
            Ok(Lock::from(lock))
        })
    }

    fn set_parent_ids(&self, parent_ids: &[RevisionId]) -> Result<(), Error> {
        Python::attach(|py| {
            let parent_ids_py: Vec<Py<PyAny>> = parent_ids
                .iter()
                .map(|id| id.clone().into_pyobject(py).unwrap().unbind())
                .collect();
            self.to_object(py)
                .call_method1(py, "set_parent_ids", (parent_ids_py,))?;
            Ok(())
        })
    }

    fn set_parent_trees(&self, parent_trees: &[(RevisionId, RevisionTree)]) -> Result<(), Error> {
        Python::attach(|py| {
            let parent_trees_py: Vec<(Py<PyAny>, Py<PyAny>)> = parent_trees
                .iter()
                .map(|(id, tree)| {
                    (
                        id.clone().into_pyobject(py).unwrap().unbind(),
                        tree.to_object(py),
                    )
                })
                .collect();
            self.to_object(py)
                .call_method1(py, "set_parent_trees", (parent_trees_py,))?;
            Ok(())
        })
    }

    fn apply_inventory_delta(&self, delta: Vec<InventoryDelta>) -> Result<(), Error> {
        Python::attach(|py| {
            let delta_py: Vec<Py<PyAny>> = delta
                .into_iter()
                .map(|d| {
                    let tuple = pyo3::types::PyTuple::new(
                        py,
                        vec![
                            d.old_path
                                .map(|p| p.to_string_lossy().to_string())
                                .into_pyobject(py)
                                .unwrap()
                                .into_any(),
                            d.new_path
                                .map(|p| p.to_string_lossy().to_string())
                                .into_pyobject(py)
                                .unwrap()
                                .into_any(),
                            d.file_id.into_pyobject(py).unwrap().into_any(),
                            d.entry.into_pyobject(py).unwrap().into_any(),
                        ],
                    )
                    .unwrap();
                    tuple.into_any().unbind()
                })
                .collect();
            self.to_object(py)
                .call_method1(py, "apply_inventory_delta", (delta_py,))?;
            Ok(())
        })
    }

    fn commit(
        &self,
        message: &str,
        committer: Option<&str>,
        timestamp: Option<f64>,
        allow_pointless: Option<bool>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(committer) = committer {
                kwargs.set_item("committer", committer)?;
            }
            if let Some(timestamp) = timestamp {
                kwargs.set_item("timestamp", timestamp)?;
            }
            if let Some(allow_pointless) = allow_pointless {
                kwargs.set_item("allow_pointless", allow_pointless)?;
            }
            if let Some(specific_files) = specific_files {
                let file_paths: Vec<String> = specific_files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                kwargs.set_item("specific_files", file_paths)?;
            }
            let result = self
                .to_object(py)
                .call_method(py, "commit", (message,), Some(&kwargs))?;
            result.extract(py).map_err(Into::into)
        })
    }
}

/// A read-only tree at a specific revision.
pub struct RevisionTree(pub Py<PyAny>);

impl<'py> IntoPyObject<'py> for RevisionTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl PyTree for RevisionTree {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

impl Clone for RevisionTree {
    fn clone(&self) -> Self {
        Python::attach(|py| RevisionTree(self.0.clone_ref(py)))
    }
}

impl RevisionTree {
    /// Get the repository this revision tree belongs to.
    pub fn repository(&self) -> crate::repository::GenericRepository {
        Python::attach(|py| {
            let repository = self.to_object(py).getattr(py, "_repository").unwrap();
            crate::repository::GenericRepository::new(repository)
        })
    }

    /// Get the revision ID of this tree.
    pub fn get_revision_id(&self) -> RevisionId {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_revision_id")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Get the parent revision IDs of this tree.
    pub fn get_parent_ids(&self) -> Vec<RevisionId> {
        Python::attach(|py| {
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
        dict.set_item(
            "path",
            (
                self.path
                    .0
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                self.path
                    .1
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
            ),
        )
        .unwrap();
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

impl<'a, 'py> FromPyObject<'a, 'py> for TreeChange {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
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
pub struct MemoryTree(pub Py<PyAny>);

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
        Python::attach(|py| {
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
    fn to_object(&self, py: Python) -> Py<PyAny> {
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
