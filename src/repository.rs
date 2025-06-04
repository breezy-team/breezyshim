//! Repository handling
//!
//! A repository is a collection of revisions and their associated data.
use crate::branch::GenericBranch;
use crate::controldir::{ControlDir, GenericControlDir};
use crate::delta::TreeDelta;
use crate::foreign::VcsType;
use crate::graph::Graph;
use crate::location::AsLocation;
use crate::lock::Lock;
use crate::revisionid::RevisionId;
use crate::tree::RevisionTree;
use chrono::DateTime;
use chrono::TimeZone;
use pyo3::exceptions::PyStopIteration;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Represents the format of a repository.
///
/// Different repository formats have different capabilities, such as
/// support for content hash keys (CHKs).
pub struct RepositoryFormat(PyObject);

impl Clone for RepositoryFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| RepositoryFormat(self.0.clone_ref(py)))
    }
}

impl RepositoryFormat {
    /// Check if this repository format supports content hash keys (CHKs).
    ///
    /// # Returns
    ///
    /// `true` if the format supports CHKs, `false` otherwise
    pub fn supports_chks(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .getattr(py, "supports_chks")
                .and_then(|attr| attr.extract(py))
                .unwrap_or(false)
        })
    }
}

/// Trait for repository operations.
///
/// This trait defines the operations that can be performed on a repository,
/// such as fetching revisions, getting a revision tree, or looking up revisions.
pub trait Repository {
    /// Get the version control system type for this repository.
    fn vcs_type(&self) -> VcsType;

    /// Get the user-facing URL for this repository.
    fn get_user_url(&self) -> url::Url;

    /// Get a transport for the user-facing URL.
    fn user_transport(&self) -> crate::transport::Transport;

    /// Get a transport for the control directory.
    fn control_transport(&self) -> crate::transport::Transport;

    /// Fetch revisions from another repository.
    ///
    /// # Arguments
    ///
    /// * `other_repository` - The repository to fetch from
    /// * `stop_revision` - Optional revision to stop fetching at
    fn fetch<R: PyRepository>(
        &self,
        other_repository: &R,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error>;

    /// Get a revision tree for a specific revision.
    ///
    /// # Arguments
    ///
    /// * `revid` - The revision ID to get the tree for
    fn revision_tree(&self, revid: &RevisionId) -> Result<RevisionTree, crate::error::Error>;

    /// Get the revision graph for this repository.
    fn get_graph(&self) -> Graph;

    /// Get the control directory for this repository.
    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >;

    /// Get the repository format.
    fn format(&self) -> RepositoryFormat;

    /// Iterate over revisions with the given IDs.
    ///
    /// # Arguments
    ///
    /// * `revision_ids` - The revision IDs to iterate over
    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> impl Iterator<Item = (RevisionId, Option<Revision>)>;

    /// Get revision deltas for the given revisions.
    ///
    /// # Arguments
    ///
    /// * `revs` - The revisions to get deltas for
    /// * `specific_files` - Optional list of specific files to get deltas for
    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> impl Iterator<Item = TreeDelta>;

    /// Get a specific revision.
    ///
    /// # Arguments
    ///
    /// * `revision_id` - The revision ID to get
    fn get_revision(&self, revision_id: &RevisionId) -> Result<Revision, crate::error::Error>;

    /// Look up a Bazaar revision ID.
    ///
    /// # Arguments
    ///
    /// * `revision_id` - The revision ID to look up
    fn lookup_bzr_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<(Vec<u8>,), crate::error::Error>;

    /// Look up a foreign revision ID.
    ///
    /// # Arguments
    ///
    /// * `foreign_revid` - The foreign revision ID to look up
    fn lookup_foreign_revision_id(
        &self,
        foreign_revid: &[u8],
    ) -> Result<RevisionId, crate::error::Error>;

    /// Lock the repository for reading.
    fn lock_read(&self) -> Result<Lock, crate::error::Error>;

    /// Lock the repository for writing.
    fn lock_write(&self) -> Result<Lock, crate::error::Error>;
}

/// Trait for types that can be converted to Python repository objects.
///
/// This trait is implemented by types that represent Breezy repositories
/// and can be converted to Python objects.
pub trait PyRepository: std::any::Any {
    /// Get the underlying Python object for this repository.
    fn to_object(&self, py: Python) -> PyObject;
}

/// Generic wrapper for a Python repository object.
///
/// This struct provides a Rust interface to a Breezy repository object.
pub struct GenericRepository(PyObject);

impl Clone for GenericRepository {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericRepository(self.0.clone_ref(py)))
    }
}

#[derive(Debug)]
/// Represents a revision in a version control repository.
///
/// A revision contains metadata about a specific version of the code,
/// such as the revision ID, parent revisions, commit message, committer,
/// and timestamp.
#[derive(Clone)]
pub struct Revision {
    /// The unique identifier for this revision.
    pub revision_id: RevisionId,
    /// The IDs of the parent revisions (usually one, but can be multiple for merges).
    pub parent_ids: Vec<RevisionId>,
    /// The commit message for this revision.
    pub message: String,
    /// The name and email of the person who committed this revision.
    pub committer: String,
    /// The timestamp when this revision was committed (in seconds since the Unix epoch).
    pub timestamp: f64,
    /// The timezone offset for the timestamp, in seconds east of UTC.
    pub timezone: i32,
}

impl Revision {
    /// Get the commit timestamp as a DateTime object.
    ///
    /// # Returns
    ///
    /// A DateTime object representing the commit timestamp with its timezone
    pub fn datetime(&self) -> DateTime<chrono::FixedOffset> {
        let tz = chrono::FixedOffset::east_opt(self.timezone).unwrap();
        tz.timestamp_opt(self.timestamp as i64, 0).unwrap()
    }
}

impl<'py> IntoPyObject<'py> for Revision {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("message", self.message).unwrap();
        kwargs.set_item("committer", self.committer).unwrap();
        kwargs.set_item("timestamp", self.timestamp).unwrap();
        kwargs.set_item("timezone", self.timezone).unwrap();
        kwargs.set_item("revision_id", self.revision_id).unwrap();
        kwargs
            .set_item(
                "parent_ids",
                self.parent_ids.into_iter().collect::<Vec<_>>(),
            )
            .unwrap();
        Ok(py
            .import("breezy.revision")
            .unwrap()
            .getattr("Revision")
            .unwrap()
            .call((), Some(&kwargs))
            .unwrap())
    }
}

impl FromPyObject<'_> for Revision {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Revision {
            revision_id: ob.getattr("revision_id")?.extract()?,
            parent_ids: ob.getattr("parent_ids")?.extract()?,
            message: ob.getattr("message")?.extract()?,
            committer: ob.getattr("committer")?.extract()?,
            timestamp: ob.getattr("timestamp")?.extract()?,
            timezone: ob.getattr("timezone")?.extract()?,
        })
    }
}

/// Iterator over revisions in a repository.
///
/// This struct provides an iterator interface for accessing revisions
/// in a repository, returning pairs of revision IDs and revision objects.
pub struct RevisionIterator(PyObject);

impl Iterator for RevisionIterator {
    type Item = (RevisionId, Option<Revision>);

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(
            |py| match self.0.call_method0(py, intern!(py, "__next__")) {
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Ok(o) => o.extract(py).ok(),
                Err(_) => None,
            },
        )
    }
}

/// Iterator over tree deltas in a repository.
///
/// This struct provides an iterator interface for accessing tree deltas
/// in a repository, which represent changes between revisions.
pub struct DeltaIterator(PyObject);

impl Iterator for DeltaIterator {
    type Item = TreeDelta;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(
            |py| match self.0.call_method0(py, intern!(py, "__next__")) {
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Ok(o) => o.extract(py).ok(),
                Err(_) => None,
            },
        )
    }
}

impl<'py> IntoPyObject<'py> for GenericRepository {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl PyRepository for GenericRepository {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl GenericRepository {
    /// Create a new GenericRepository from a Python object.
    ///
    /// # Arguments
    ///
    /// * `obj` - The Python object representing a Breezy repository
    ///
    /// # Returns
    ///
    /// A new GenericRepository wrapping the provided Python object
    pub fn new(obj: PyObject) -> Self {
        GenericRepository(obj)
    }
}

impl From<PyObject> for GenericRepository {
    fn from(obj: PyObject) -> Self {
        GenericRepository(obj)
    }
}

impl<T: PyRepository> Repository for T {
    fn vcs_type(&self) -> VcsType {
        Python::with_gil(|py| {
            if self.to_object(py).getattr(py, "_git").is_ok() {
                VcsType::Git
            } else {
                VcsType::Bazaar
            }
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "user_url")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
                .parse()
                .unwrap()
        })
    }

    fn user_transport(&self) -> crate::transport::Transport {
        Python::with_gil(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "user_transport").unwrap(),
            )
        })
    }

    fn control_transport(&self) -> crate::transport::Transport {
        Python::with_gil(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "control_transport").unwrap(),
            )
        })
    }

    fn fetch<R: PyRepository>(
        &self,
        other_repository: &R,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method1(
                py,
                "fetch",
                (
                    other_repository.to_object(py),
                    stop_revision.map(|r| r.clone().into_pyobject(py).unwrap().unbind()),
                ),
            )?;
            Ok(())
        })
    }

    fn revision_tree(&self, revid: &RevisionId) -> Result<RevisionTree, crate::error::Error> {
        Python::with_gil(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "revision_tree", (revid.clone(),))?;
            Ok(RevisionTree(o))
        })
    }

    fn get_graph(&self) -> Graph {
        Python::with_gil(|py| {
            Graph::from(self.to_object(py).call_method0(py, "get_graph").unwrap())
        })
    }

    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    > {
        Python::with_gil(|py| {
            Box::new(GenericControlDir::new(
                self.to_object(py).getattr(py, "controldir").unwrap(),
            ))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >
        })
    }

    fn format(&self) -> RepositoryFormat {
        Python::with_gil(|py| RepositoryFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> impl Iterator<Item = (RevisionId, Option<Revision>)> {
        Python::with_gil(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "iter_revisions", (revision_ids,))
                .unwrap();
            RevisionIterator(o)
        })
    }

    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> impl Iterator<Item = TreeDelta> {
        Python::with_gil(|py| {
            let revs = revs
                .iter()
                .map(|r| r.clone().into_pyobject(py).unwrap())
                .collect::<Vec<_>>();
            let specific_files = specific_files
                .map(|files| files.iter().map(|f| f.to_path_buf()).collect::<Vec<_>>());
            let o = self
                .to_object(py)
                .call_method1(py, "get_revision_deltas", (revs, specific_files))
                .unwrap();
            DeltaIterator(o)
        })
    }

    fn get_revision(&self, revision_id: &RevisionId) -> Result<Revision, crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "get_revision", (revision_id.clone(),))?
                .extract(py)
        })
        .map_err(|e| e.into())
    }

    // TODO: This should really be on ForeignRepository
    fn lookup_bzr_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<(Vec<u8>,), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_bzr_revision_id", (revision_id.clone(),))?
                .extract::<(Vec<u8>, PyObject)>(py)
        })
        .map_err(|e| e.into())
        .map(|(v, _m)| (v,))
    }

    fn lookup_foreign_revision_id(
        &self,
        foreign_revid: &[u8],
    ) -> Result<RevisionId, crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_foreign_revision_id", (foreign_revid,))?
                .extract(py)
        })
        .map_err(|e| e.into())
    }

    fn lock_read(&self) -> Result<Lock, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(Lock::from(
                self.to_object(py)
                    .call_method0(py, intern!(py, "lock_read"))?,
            ))
        })
    }

    fn lock_write(&self) -> Result<Lock, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(Lock::from(
                self.to_object(py)
                    .call_method0(py, intern!(py, "lock_write"))?,
            ))
        })
    }
}

/// Open a repository at the specified location.
///
/// # Arguments
///
/// * `base` - The location to open the repository at
///
/// # Returns
///
/// A GenericRepository object, or an error if the operation fails
///
/// # Examples
///
/// ```no_run
/// use breezyshim::repository::open;
/// let repo = open("https://code.launchpad.net/brz").unwrap();
/// ```
pub fn open(base: impl AsLocation) -> Result<GenericRepository, crate::error::Error> {
    Python::with_gil(|py| {
        let o = py
            .import("breezy.repository")?
            .getattr("Repository")?
            .call_method1("open", (base.as_location(),))?;
        Ok(GenericRepository::new(o.into()))
    })
}

#[cfg(test)]
mod repository_tests {
    use super::{GenericRepository, Repository};
    use crate::controldir::ControlDirFormat;
    use crate::foreign::VcsType;
    use crate::revisionid::RevisionId;

    #[test]
    fn test_simple() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let _repo = crate::repository::open(td.path()).unwrap();
    }

    #[test]
    fn test_clone() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _repo2 = repo.clone();
    }

    #[test]
    fn test_repository_format() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let format = repo.format();
        let _supports_chks = format.supports_chks();
    }

    #[test]
    fn test_repository_format_clone() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let format = repo.format();
        let _format2 = format.clone();
    }

    #[test]
    fn test_vcs_type() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let vcs_type = repo.vcs_type();
        assert!(matches!(vcs_type, VcsType::Bazaar | VcsType::Git));
    }

    #[test]
    fn test_user_url() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _url = repo.get_user_url();
    }

    #[test]
    fn test_transports() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _user_transport = repo.user_transport();
        let _control_transport = repo.control_transport();
    }

    #[test]
    fn test_controldir() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _controldir = repo.controldir();
    }

    #[test]
    fn test_get_graph() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _graph = repo.get_graph();
    }

    #[test]
    fn test_revision_tree() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Try to get revision tree for null revision
        let null_revid = RevisionId::null();
        let _tree = repo.revision_tree(&null_revid).unwrap();
    }

    #[test]
    fn test_iter_revisions() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test with empty list
        let revisions = vec![];
        let mut iter = repo.iter_revisions(revisions);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_lock_operations() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test read lock
        let read_lock = repo.lock_read();
        assert!(read_lock.is_ok());

        // Test write lock
        let write_lock = repo.lock_write();
        assert!(!write_lock.is_ok());
    }
}
