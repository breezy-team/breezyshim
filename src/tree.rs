use crate::branch::{Branch, RegularBranch};
use crate::controldir::ControlDir;
use crate::lock::Lock;
use crate::revisionid::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;

import_exception!(breezy.commit, PointlessCommit);
import_exception!(breezy.commit, NoWhoami);
import_exception!(breezy.errors, NotBranchError);
import_exception!(breezy.errors, DependencyNotPresent);
import_exception!(breezy.errors, DivergedBranches);
import_exception!(breezy.transport, NoSuchFile);

pub type Path = std::path::Path;
pub type PathBuf = std::path::PathBuf;

#[derive(Debug, PartialEq)]
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

    pub fn to_string(&self) -> &'static str {
        match self {
            Kind::File => "file",
            Kind::Directory => "directory",
            Kind::Symlink => "symlink",
            Kind::TreeReference => "tree-reference",
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
    fn extract(ob: &pyo3::PyAny) -> pyo3::PyResult<Self> {
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
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let kind: std::borrow::Cow<str> = ob.getattr("kind")?.extract()?;
        match kind.as_ref() {
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

#[derive(Debug)]
pub enum Error {
    NoSuchFile(PathBuf),
    Other(PyErr),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoSuchFile(path) => write!(f, "No such file: {}", path.to_string_lossy()),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<NoSuchFile>(py) {
                return Error::NoSuchFile(e.into_value(py).getattr(py, "path").unwrap().extract(py).unwrap());
            }
            Error::Other(e)
        })
    }
}

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::NoSuchFile(path) => NoSuchFile::new_err(path.to_string_lossy().to_string()),
            Error::Other(e) => e,
        }
    }
}

#[derive(Debug)]
pub enum PullError {
    DivergedBranches,
    Other(PyErr),
}

impl From<PyErr> for PullError {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<DivergedBranches>(py) {
                return PullError::DivergedBranches;
            }
            PullError::Other(e)
        })
    }
}

impl std::fmt::Display for PullError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PullError::DivergedBranches => write!(f, "Diverged branches"),
            PullError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PullError {}

pub trait Tree: ToPyObject {
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, PyErr> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
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

            Ok(Box::new(TreeChangeIter(self.to_object(py).call_method_bound(
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

pub trait MutableTree: Tree {
    fn lock_write(&self) -> Result<Lock, Error> {
        Python::with_gil(|py| {
            let lock = self.to_object(py).call_method0(py, "lock_write").unwrap();
            Ok(Lock::from(lock))
        })
    }

    fn put_file_bytes_non_atomic(&self, path: &Path, data: &[u8]) -> Result<(), Error> {
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
        Python::with_gil(|py| -> Result<(), PyErr> {
            self.to_object(py).call_method1(py, "mkdir", (path,))?;
            Ok(())
        })
        .map_err(|e| e.into())
    }
}

pub struct RevisionTree(pub PyObject);

impl ToPyObject for RevisionTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Tree for RevisionTree {}

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

#[derive(Debug)]
pub enum CommitError {
    PointlessCommit,
    NoWhoami,
    Other(PyErr),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommitError::PointlessCommit => write!(f, "Pointless commit"),
            CommitError::NoWhoami => write!(f, "No whoami"),
            CommitError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for CommitError {}

pub struct WorkingTree(pub PyObject);

impl ToPyObject for WorkingTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

#[derive(Debug)]
pub enum WorkingTreeOpenError {
    NotBranchError(String),
    DependencyNotPresent(String, String),
    Other(PyErr),
}

impl From<PyErr> for WorkingTreeOpenError {
    fn from(err: PyErr) -> Self {
        Python::with_gil(|py| {
            if err.is_instance_of::<NotBranchError>(py) {
                let l = err
                    .into_value(py)
                    .getattr(py, "path")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap();
                WorkingTreeOpenError::NotBranchError(l)
            } else if err.is_instance_of::<DependencyNotPresent>(py) {
                let value = err
                    .into_value(py);
                let l = value
                    .getattr(py, "library")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap();
                let e = value
                    .getattr(py, "error")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap();
                WorkingTreeOpenError::DependencyNotPresent(l, e)
            } else {
                WorkingTreeOpenError::Other(err)
            }
        })
    }
}

impl From<WorkingTreeOpenError> for PyErr {
    fn from(err: WorkingTreeOpenError) -> Self {
        match err {
            WorkingTreeOpenError::NotBranchError(l) => NotBranchError::new_err((l,)),
            WorkingTreeOpenError::DependencyNotPresent(d, e) => {
                DependencyNotPresent::new_err((d, e))
            }
            WorkingTreeOpenError::Other(err) => err,
        }
    }
}

impl std::fmt::Display for WorkingTreeOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WorkingTreeOpenError::NotBranchError(l) => write!(f, "Not branch error: {}", l),
            WorkingTreeOpenError::DependencyNotPresent(d, e) => {
                write!(f, "Dependency not present: {} {}", d, e)
            }
            WorkingTreeOpenError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for WorkingTreeOpenError {}

impl WorkingTree {
    /// Return the base path for this working tree.
    pub fn basedir(&self) -> PathBuf {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "basedir")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Return the branch for this working tree.
    pub fn branch(&self) -> Box<dyn Branch> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch").unwrap();
            Box::new(RegularBranch::new(branch)) as Box<dyn Branch>
        })
    }

    /// Return the control directory for this working tree.
    pub fn controldir(&self) -> ControlDir {
        Python::with_gil(|py| {
            let controldir = self.to_object(py).getattr(py, "controldir").unwrap();
            ControlDir::new(controldir)
        })
    }

    pub fn open(path: &Path) -> Result<WorkingTree, WorkingTreeOpenError> {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.workingtree")?;
            let c = m.getattr("WorkingTree")?;
            let wt = c.call_method1("open", (path,))?;
            Ok(WorkingTree(wt.to_object(py)))
        })
    }

    pub fn open_containing(path: &Path) -> Result<(WorkingTree, PathBuf), WorkingTreeOpenError> {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.workingtree")?;
            let c = m.getattr("WorkingTree")?;
            let (wt, p): (&PyAny, String) =
                c.call_method1("open_containing", (path,))?.extract()?;
            Ok((WorkingTree(wt.to_object(py)), PathBuf::from(p)))
        })
    }

    pub fn basis_tree(&self) -> crate::tree::RevisionTree {
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method0(py, "basis_tree").unwrap();
            RevisionTree(tree)
        })
    }

    pub fn revision_tree(&self, revision_id: &RevisionId) -> Result<Box<RevisionTree>, PyErr> {
        Python::with_gil(|py| {
            let tree = self
                .to_object(py)
                .call_method1(py, "revision_tree", (revision_id.to_object(py),))
                .unwrap();
            Ok(Box::new(RevisionTree(tree)))
        })
    }

    pub fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, PyErr> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
    }

    pub fn abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "abspath", (path,))?
                .extract(py)?)
        })
    }

    pub fn relpath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "relpath", (path,))?
                .extract(py)?)
        })
    }

    pub fn supports_setting_file_ids(&self) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    pub fn smart_add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "smart_add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    pub fn commit(
        &self,
        message: &str,
        allow_pointless: Option<bool>,
        committer: Option<&str>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, CommitError> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new_bound(py);
            if let Some(committer) = committer {
                kwargs.set_item("committer", committer).unwrap();
            }
            if let Some(specific_files) = specific_files {
                kwargs.set_item("specific_files", specific_files).unwrap();
            }
            if let Some(allow_pointless) = allow_pointless {
                kwargs.set_item("allow_pointless", allow_pointless).unwrap();
            }

            let null_commit_reporter = py
                .import_bound("breezy.commit")
                .unwrap()
                .getattr("NullCommitReporter")
                .unwrap()
                .call0()
                .unwrap();
            kwargs.set_item("reporter", null_commit_reporter).unwrap();

            Ok(self
                .to_object(py)
                .call_method_bound(py, "commit", (message,), Some(&kwargs))
                .map_err(|e| {
                    if e.is_instance_of::<PointlessCommit>(py) {
                        CommitError::PointlessCommit
                    } else if e.is_instance_of::<NoWhoami>(py) {
                        CommitError::NoWhoami
                    } else {
                        CommitError::Other(e)
                    }
                })?
                .extract(py)
                .unwrap())
        })
    }

    pub fn last_revision(&self) -> Result<RevisionId, PyErr> {
        Python::with_gil(|py| {
            let last_revision = self.to_object(py).call_method0(py, "last_revision")?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }

    pub fn pull(
        &self,
        source: &dyn crate::branch::Branch,
        overwrite: Option<bool>,
    ) -> Result<(), PullError> {
        Python::with_gil(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new_bound(py);
                if let Some(overwrite) = overwrite {
                    kwargs.set_item("overwrite", overwrite).unwrap();
                }
                kwargs
            };
            self.to_object(py)
                .call_method_bound(py, "pull", (source.to_object(py),), Some(&kwargs))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }
}

impl From<PyObject> for WorkingTree {
    fn from(obj: PyObject) -> Self {
        WorkingTree(obj)
    }
}

impl Tree for WorkingTree {}

impl MutableTree for WorkingTree {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeChange {
    pub path: (Option<PathBuf>, Option<PathBuf>),
    pub changed_content: bool,
    pub versioned: (Option<bool>, Option<bool>),
    pub name: (Option<std::ffi::OsString>, Option<std::ffi::OsString>),
    pub kind: (Option<String>, Option<String>),
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
    fn extract(obj: &PyAny) -> PyResult<Self> {
        fn from_bool(o: &PyAny) -> PyResult<bool> {
            if let Ok(b) = o.extract::<isize>() {
                Ok(b != 0)
            } else {
                o.extract::<bool>()
            }
        }

        fn from_opt_bool_tuple(o: &PyAny) -> PyResult<(Option<bool>, Option<bool>)> {
            let tuple = o.extract::<(Option<&PyAny>, Option<&PyAny>)>()?;
            Ok((
                tuple.0.map(from_bool).transpose()?,
                tuple.1.map(from_bool).transpose()?,
            ))
        }

        let path = obj.getattr("path")?;
        let changed_content = from_bool(obj.getattr("changed_content")?)?;

        let versioned = from_opt_bool_tuple(obj.getattr("versioned")?)?;
        let name = obj.getattr("name")?;
        let kind = obj.getattr("kind")?;
        let executable = from_opt_bool_tuple(obj.getattr("executable")?)?;
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

impl MutableTree for MemoryTree {}
