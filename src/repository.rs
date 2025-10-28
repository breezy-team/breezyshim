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
use std::collections::HashMap;

/// Represents the format of a repository.
///
/// Different repository formats have different capabilities, such as
/// support for content hash keys (CHKs).
pub struct RepositoryFormat(Py<PyAny>);

impl Clone for RepositoryFormat {
    fn clone(&self) -> Self {
        Python::attach(|py| RepositoryFormat(self.0.clone_ref(py)))
    }
}

impl RepositoryFormat {
    /// Check if this repository format supports content hash keys (CHKs).
    ///
    /// # Returns
    ///
    /// `true` if the format supports CHKs, `false` otherwise
    pub fn supports_chks(&self) -> bool {
        Python::attach(|py| {
            self.0
                .getattr(py, "supports_chks")
                .and_then(|attr| attr.extract(py))
                .unwrap_or(false)
        })
    }
}

/// Represents the lock status of a repository.
#[derive(Debug, Clone)]
pub struct LockStatus {
    /// Whether the repository is locked.
    pub is_locked: bool,
    /// The holder of the lock, if any.
    pub lock_holder: Option<String>,
}

/// Statistics about a repository.
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    /// Number of revisions in the repository.
    pub revision_count: u32,
    /// Number of files in the repository.
    pub file_count: u32,
    /// Committer statistics, if requested.
    pub committers: Option<HashMap<String, u32>>,
}

/// Trait for repository operations.
///
/// This trait defines the operations that can be performed on a repository,
/// such as fetching revisions, getting a revision tree, or looking up revisions.
pub trait Repository {
    /// Get a reference to the underlying Any type for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

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
    fn fetch(
        &self,
        other_repository: &dyn Repository,
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
    ) -> Box<dyn Iterator<Item = (RevisionId, Option<Revision>)>>;

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
    ) -> Box<dyn Iterator<Item = TreeDelta>>;

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

    /// Check if the repository has a specific revision.
    ///
    /// # Arguments
    ///
    /// * `revision_id` - The revision ID to check for
    fn has_revision(&self, revision_id: &RevisionId) -> Result<bool, crate::error::Error>;

    /// Get all revision IDs in the repository.
    fn all_revision_ids(&self) -> Result<Vec<RevisionId>, crate::error::Error>;

    /// Check if the repository is shared (can be used by multiple branches).
    fn is_shared(&self) -> Result<bool, crate::error::Error>;

    /// Get the signature text for a revision.
    ///
    /// # Arguments
    ///
    /// * `revision_id` - The revision ID to get the signature for
    fn get_signature_text(&self, revision_id: &RevisionId) -> Result<String, crate::error::Error>;

    /// Check if a revision has a signature.
    ///
    /// # Arguments
    ///
    /// * `revision_id` - The revision ID to check
    fn has_signature_for_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<bool, crate::error::Error>;

    /// Pack the repository to optimize storage.
    ///
    /// # Arguments
    ///
    /// * `hint` - Optional list of revision IDs to focus on
    /// * `clean_obsolete_packs` - Whether to clean obsolete packs
    fn pack(
        &self,
        hint: Option<&[RevisionId]>,
        clean_obsolete_packs: bool,
    ) -> Result<(), crate::error::Error>;

    /// Start a write group for batch operations.
    fn start_write_group(&self) -> Result<(), crate::error::Error>;

    /// Commit a write group.
    fn commit_write_group(&self) -> Result<(), crate::error::Error>;

    /// Abort a write group.
    fn abort_write_group(&self) -> Result<(), crate::error::Error>;

    /// Check if a write group is active.
    fn is_in_write_group(&self) -> bool;

    /// Get parent revision IDs for given revisions.
    ///
    /// # Arguments
    ///
    /// * `revision_ids` - The revision IDs to get parents for
    fn get_parent_map(
        &self,
        revision_ids: &[RevisionId],
    ) -> Result<std::collections::HashMap<RevisionId, Vec<RevisionId>>, crate::error::Error>;

    /// Get missing revision IDs between this repository and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other repository to compare with
    /// * `revision_id` - Optional revision to stop at
    fn missing_revision_ids(
        &self,
        other: &dyn Repository,
        revision_id: Option<&RevisionId>,
    ) -> Result<Vec<RevisionId>, crate::error::Error>;

    /// Find branches that use this repository.
    fn find_branches(&self) -> Result<Vec<GenericBranch>, crate::error::Error>;

    /// Get physical lock status.
    fn get_physical_lock_status(&self) -> Result<LockStatus, crate::error::Error>;

    /// Add a fallback repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository to add as a fallback
    fn add_fallback_repository(
        &self,
        repository: &dyn Repository,
    ) -> Result<(), crate::error::Error>;

    /// Get ancestry of revisions.
    ///
    /// # Arguments
    ///
    /// * `revision_ids` - The revision IDs to get ancestry for
    /// * `topo_sorted` - Whether to sort topologically
    fn get_ancestry(
        &self,
        revision_ids: &[RevisionId],
        topo_sorted: bool,
    ) -> Result<Vec<RevisionId>, crate::error::Error>;

    /// Gather statistics about the repository.
    ///
    /// # Arguments
    ///
    /// * `committers` - Whether to gather committer statistics
    /// * `log` - Whether to log progress
    fn gather_stats(
        &self,
        committers: Option<bool>,
        log: Option<bool>,
    ) -> Result<RepositoryStats, crate::error::Error>;

    /// Get file graph for specific files.
    fn get_file_graph(&self) -> Result<Graph, crate::error::Error>;
}

/// Trait for types that can be converted to Python repository objects.
///
/// This trait is implemented by types that represent Breezy repositories
/// and can be converted to Python objects.
pub trait PyRepository: Repository + std::any::Any {
    /// Get the underlying Python object for this repository.
    fn to_object(&self, py: Python) -> Py<PyAny>;
}

impl dyn PyRepository {
    /// Get a reference to self as a Repository trait object.
    pub fn as_repository(&self) -> &dyn Repository {
        self
    }
}

/// Generic wrapper for a Python repository object.
///
/// This struct provides a Rust interface to a Breezy repository object.
pub struct GenericRepository(Py<PyAny>);

impl Clone for GenericRepository {
    fn clone(&self) -> Self {
        Python::attach(|py| GenericRepository(self.0.clone_ref(py)))
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
    /// Revision properties as key-value pairs.
    pub properties: std::collections::HashMap<String, String>,
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

    /// Get the revision properties for this revision.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing the revision properties as key-value pairs
    pub fn get_properties(&self) -> &std::collections::HashMap<String, String> {
        &self.properties
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

        // Add properties if they exist
        if !self.properties.is_empty() {
            let py_properties = pyo3::types::PyDict::new(py);
            for (key, value) in self.properties {
                py_properties.set_item(key, value).unwrap();
            }
            kwargs.set_item("properties", py_properties).unwrap();
        }

        Ok(py
            .import("breezy.revision")
            .unwrap()
            .getattr("Revision")
            .unwrap()
            .call((), Some(&kwargs))
            .unwrap())
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Revision {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Extract properties if they exist
        let mut properties = std::collections::HashMap::new();

        if let Ok(py_properties) = ob.getattr("properties") {
            if !py_properties.is_none() {
                if let Ok(py_dict) = py_properties.cast::<pyo3::types::PyDict>() {
                    for (key, value) in py_dict.iter() {
                        let key_str: String = key.extract()?;
                        let value_str: String = value.extract()?;
                        properties.insert(key_str, value_str);
                    }
                }
            }
        }

        Ok(Revision {
            revision_id: ob.getattr("revision_id")?.extract()?,
            parent_ids: ob.getattr("parent_ids")?.extract()?,
            message: ob.getattr("message")?.extract()?,
            committer: ob.getattr("committer")?.extract()?,
            timestamp: ob.getattr("timestamp")?.extract()?,
            timezone: ob.getattr("timezone")?.extract()?,
            properties,
        })
    }
}

/// Iterator over revisions in a repository.
///
/// This struct provides an iterator interface for accessing revisions
/// in a repository, returning pairs of revision IDs and revision objects.
pub struct RevisionIterator(Py<PyAny>);

impl Iterator for RevisionIterator {
    type Item = (RevisionId, Option<Revision>);

    fn next(&mut self) -> Option<Self::Item> {
        Python::attach(
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
pub struct DeltaIterator(Py<PyAny>);

impl Iterator for DeltaIterator {
    type Item = TreeDelta;

    fn next(&mut self) -> Option<Self::Item> {
        Python::attach(
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
    fn to_object(&self, py: Python) -> Py<PyAny> {
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
    pub fn new(obj: Py<PyAny>) -> Self {
        GenericRepository(obj)
    }
}

impl From<Py<PyAny>> for GenericRepository {
    fn from(obj: Py<PyAny>) -> Self {
        GenericRepository(obj)
    }
}

impl<T: PyRepository> Repository for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn vcs_type(&self) -> VcsType {
        Python::attach(|py| {
            if self.to_object(py).getattr(py, "_git").is_ok() {
                VcsType::Git
            } else {
                VcsType::Bazaar
            }
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::attach(|py| {
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
        Python::attach(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "user_transport").unwrap(),
            )
        })
    }

    fn control_transport(&self) -> crate::transport::Transport {
        Python::attach(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "control_transport").unwrap(),
            )
        })
    }

    fn fetch(
        &self,
        other_repository: &dyn Repository,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            // Try to get the Python object from the other repository
            let other_py = if let Some(py_repo) = other_repository
                .as_any()
                .downcast_ref::<GenericRepository>()
            {
                py_repo.to_object(py)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Repository must be a PyRepository",
                )
                .into());
            };

            self.to_object(py).call_method1(
                py,
                "fetch",
                (
                    other_py,
                    stop_revision.map(|r| r.clone().into_pyobject(py).unwrap().unbind()),
                ),
            )?;
            Ok(())
        })
    }

    fn revision_tree(&self, revid: &RevisionId) -> Result<RevisionTree, crate::error::Error> {
        Python::attach(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "revision_tree", (revid.clone(),))?;
            Ok(RevisionTree(o))
        })
    }

    fn get_graph(&self) -> Graph {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
        Python::attach(|py| RepositoryFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> Box<dyn Iterator<Item = (RevisionId, Option<Revision>)>> {
        Python::attach(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "iter_revisions", (revision_ids,))
                .unwrap();
            Box::new(RevisionIterator(o))
        })
    }

    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> Box<dyn Iterator<Item = TreeDelta>> {
        Python::attach(|py| {
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
            Box::new(DeltaIterator(o))
        })
    }

    fn get_revision(&self, revision_id: &RevisionId) -> Result<Revision, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "get_revision", (revision_id.clone(),))?
                .extract(py)
        })
        .map_err(Into::into)
    }

    // TODO: This should really be on ForeignRepository
    fn lookup_bzr_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<(Vec<u8>,), crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_bzr_revision_id", (revision_id.clone(),))?
                .extract::<(Vec<u8>, Py<PyAny>)>(py)
        })
        .map_err(Into::into)
        .map(|(v, _m)| (v,))
    }

    fn lookup_foreign_revision_id(
        &self,
        foreign_revid: &[u8],
    ) -> Result<RevisionId, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_foreign_revision_id", (foreign_revid,))?
                .extract(py)
        })
        .map_err(Into::into)
    }

    fn lock_read(&self) -> Result<Lock, crate::error::Error> {
        Python::attach(|py| {
            Ok(Lock::from(
                self.to_object(py)
                    .call_method0(py, intern!(py, "lock_read"))?,
            ))
        })
    }

    fn lock_write(&self) -> Result<Lock, crate::error::Error> {
        Python::attach(|py| {
            Ok(Lock::from(
                self.to_object(py)
                    .call_method0(py, intern!(py, "lock_write"))?,
            ))
        })
    }

    fn has_revision(&self, revision_id: &RevisionId) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "has_revision", (revision_id.clone(),))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn all_revision_ids(&self) -> Result<Vec<RevisionId>, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "all_revision_ids")?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn is_shared(&self) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "is_shared")?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn get_signature_text(&self, revision_id: &RevisionId) -> Result<String, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "get_signature_text", (revision_id.clone(),))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn has_signature_for_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "has_signature_for_revision_id", (revision_id.clone(),))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn pack(
        &self,
        hint: Option<&[RevisionId]>,
        clean_obsolete_packs: bool,
    ) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            let hint_py = hint.map(|h| h.to_vec());
            self.to_object(py)
                .call_method1(py, "pack", (hint_py, clean_obsolete_packs))?;
            Ok(())
        })
    }

    fn start_write_group(&self) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "start_write_group")?;
            Ok(())
        })
    }

    fn commit_write_group(&self) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "commit_write_group")?;
            Ok(())
        })
    }

    fn abort_write_group(&self) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "abort_write_group")?;
            Ok(())
        })
    }

    fn is_in_write_group(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "is_in_write_group")
                .and_then(|r| r.extract(py))
                .unwrap_or(false)
        })
    }

    fn get_parent_map(
        &self,
        revision_ids: &[RevisionId],
    ) -> Result<HashMap<RevisionId, Vec<RevisionId>>, crate::error::Error> {
        Python::attach(|py| {
            let result =
                self.to_object(py)
                    .call_method1(py, "get_parent_map", (revision_ids.to_vec(),))?;

            let dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .expect("get_parent_map should return a dict");
            let mut map = HashMap::new();

            for (key, value) in dict.iter() {
                let rev_id: RevisionId = key.extract()?;
                let parents: Vec<RevisionId> = value.extract()?;
                map.insert(rev_id, parents);
            }

            Ok(map)
        })
    }

    fn missing_revision_ids(
        &self,
        other: &dyn Repository,
        revision_id: Option<&RevisionId>,
    ) -> Result<Vec<RevisionId>, crate::error::Error> {
        Python::attach(|py| {
            // Try to get the Python object from the other repository
            let other_py = if let Some(py_repo) = other.as_any().downcast_ref::<GenericRepository>()
            {
                py_repo.to_object(py)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Repository must be a PyRepository",
                )
                .into());
            };

            self.to_object(py)
                .call_method1(py, "missing_revision_ids", (other_py, revision_id.cloned()))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn find_branches(&self) -> Result<Vec<GenericBranch>, crate::error::Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method0(py, "find_branches")?;

            // find_branches returns a generator, so we need to convert it to a list
            let list_module = py.import("builtins")?;
            let list_result = list_module.call_method1("list", (result,))?;

            let list = list_result
                .cast::<pyo3::types::PyList>()
                .expect("list() should return a list");
            let mut branches = Vec::new();

            for item in list.iter() {
                branches.push(GenericBranch::from(item));
            }

            Ok(branches)
        })
    }

    fn get_physical_lock_status(&self) -> Result<LockStatus, crate::error::Error> {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method0(py, "get_physical_lock_status")?;

            if result.is_none(py) {
                return Ok(LockStatus {
                    is_locked: false,
                    lock_holder: None,
                });
            }

            // The result is typically a tuple (is_locked, lock_info)
            if let Ok(tuple) = result.cast_bound::<pyo3::types::PyTuple>(py) {
                if tuple.len() >= 2 {
                    let is_locked = tuple.get_item(0)?.extract::<bool>()?;
                    let lock_info = tuple.get_item(1)?;

                    let lock_holder = if lock_info.is_none() {
                        None
                    } else if let Ok(info_dict) = lock_info.cast::<pyo3::types::PyDict>() {
                        info_dict
                            .get_item("user")?
                            .and_then(|u| u.extract::<String>().ok())
                    } else {
                        lock_info.extract::<String>().ok()
                    };

                    return Ok(LockStatus {
                        is_locked,
                        lock_holder,
                    });
                }
            }

            // Fallback: try to extract as bool
            let is_locked = result.extract::<bool>(py)?;
            Ok(LockStatus {
                is_locked,
                lock_holder: None,
            })
        })
    }

    fn add_fallback_repository(
        &self,
        repository: &dyn Repository,
    ) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            // Try to get the Python object from the repository
            let repo_py =
                if let Some(py_repo) = repository.as_any().downcast_ref::<GenericRepository>() {
                    py_repo.to_object(py)
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Repository must be a PyRepository",
                    )
                    .into());
                };

            self.to_object(py)
                .call_method1(py, "add_fallback_repository", (repo_py,))?;
            Ok(())
        })
    }

    fn get_ancestry(
        &self,
        revision_ids: &[RevisionId],
        topo_sorted: bool,
    ) -> Result<Vec<RevisionId>, crate::error::Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "get_ancestry", (revision_ids.to_vec(), topo_sorted))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn gather_stats(
        &self,
        committers: Option<bool>,
        log: Option<bool>,
    ) -> Result<RepositoryStats, crate::error::Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(c) = committers {
                kwargs.set_item("committers", c)?;
            }
            if let Some(l) = log {
                kwargs.set_item("log", l)?;
            }

            let result = self
                .to_object(py)
                .call_method(py, "gather_stats", (), Some(&kwargs))?;

            let stats_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .expect("gather_stats should return a dict");

            let revision_count = stats_dict
                .get_item("revisions")?
                .and_then(|v| v.extract::<u32>().ok())
                .unwrap_or(0);

            let file_count = stats_dict
                .get_item("files")?
                .and_then(|v| v.extract::<u32>().ok())
                .unwrap_or(0);

            let committers = if let Some(committers_dict) = stats_dict.get_item("committers")? {
                if !committers_dict.is_none() {
                    let dict = committers_dict
                        .cast::<pyo3::types::PyDict>()
                        .expect("committers should be a dict");
                    let mut map = HashMap::new();
                    for (key, value) in dict.iter() {
                        let name: String = key.extract()?;
                        let count: u32 = value.extract()?;
                        map.insert(name, count);
                    }
                    Some(map)
                } else {
                    None
                }
            } else {
                None
            };

            Ok(RepositoryStats {
                revision_count,
                file_count,
                committers,
            })
        })
    }

    fn get_file_graph(&self) -> Result<Graph, crate::error::Error> {
        Python::attach(|py| {
            Ok(Graph::from(
                self.to_object(py).call_method0(py, "get_file_graph")?,
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
    Python::attach(|py| {
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
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;
    use std::path::Path;

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

    #[test]
    fn test_has_revision() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test with null revision
        let null_revid = RevisionId::null();
        let has_null = repo.has_revision(&null_revid).unwrap();
        assert!(has_null);

        // Test with non-existent revision
        let fake_revid = RevisionId::from("fake-revision-id".as_bytes());
        let has_fake = repo.has_revision(&fake_revid).unwrap();
        assert!(!has_fake);
    }

    #[test]
    fn test_all_revision_ids() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let revision_ids = repo.all_revision_ids().unwrap();
        // New repository should have no revisions
        assert_eq!(revision_ids.len(), 0);
    }

    #[test]
    fn test_is_shared() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let _is_shared = repo.is_shared().unwrap();
        // Just test that the method works
    }

    #[test]
    fn test_write_group_operations() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test initial state
        assert!(!repo.is_in_write_group());

        // Acquire write lock first
        let _lock = repo.lock_write().unwrap();

        // Start a write group
        repo.start_write_group().unwrap();
        assert!(repo.is_in_write_group());

        // Abort the write group
        repo.abort_write_group().unwrap();
        assert!(!repo.is_in_write_group());
    }

    #[test]
    fn test_get_parent_map() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test with empty list
        let parent_map = repo.get_parent_map(&[]).unwrap();
        assert!(parent_map.is_empty());

        // Test with null revision
        let null_revid = RevisionId::null();
        let parent_map = repo.get_parent_map(&[null_revid]).unwrap();
        assert_eq!(parent_map.len(), 1);
    }

    #[test]
    fn test_find_branches() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let branches = repo.find_branches().unwrap();
        // A new standalone workingtree should have at least one branch
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_get_physical_lock_status() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let status = repo.get_physical_lock_status().unwrap();
        // Repository should not be locked initially
        assert!(!status.is_locked);
        assert!(status.lock_holder.is_none());
    }

    #[test]
    fn test_gather_stats() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let stats = repo.gather_stats(None, None).unwrap();
        // New repository should have 0 revisions
        assert_eq!(stats.revision_count, 0);
    }

    #[test]
    fn test_get_file_graph() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        let _graph = repo.get_file_graph().unwrap();
        // Just test that the method works
    }

    #[test]
    fn test_pack() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Test pack without hints
        repo.pack(None, false).unwrap();

        // Test pack with empty hints
        repo.pack(Some(&[]), true).unwrap();
    }

    #[test]
    fn test_commit_with_revision_properties() {
        let td = tempfile::tempdir().unwrap();
        let wt = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Create a file to commit
        let test_file = td.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        // Use add() with relative path instead of smart_add to avoid path issues
        wt.add(&[Path::new("test.txt")]).unwrap();

        // Test CommitBuilder with revision properties
        let test_key = "test-property";
        let test_value = "test-value-data";
        let test_key2 = "deb-pristine-delta-foo.tar.gz";
        let test_value2 = "binary-delta-data-here";

        let revision_id = wt
            .build_commit()
            .message("Test commit with properties")
            .committer("Test User <test@example.com>")
            .set_revprop(test_key, test_value)
            .unwrap()
            .set_revprop(test_key2, test_value2)
            .unwrap()
            .commit()
            .unwrap();

        // Retrieve the revision and check properties
        let revision = repo.get_revision(&revision_id).unwrap();
        let properties = revision.get_properties();

        // Check that our properties are present with correct values
        assert!(
            properties.contains_key(test_key),
            "Property '{}' not found",
            test_key
        );
        assert!(
            properties.contains_key(test_key2),
            "Property '{}' not found",
            test_key2
        );

        // Verify the values match what we set
        let retrieved_value = properties.get(test_key).unwrap();
        let retrieved_value2 = properties.get(test_key2).unwrap();

        assert_eq!(
            retrieved_value, test_value,
            "Property '{}' value mismatch",
            test_key
        );
        assert_eq!(
            retrieved_value2, test_value2,
            "Property '{}' value mismatch",
            test_key2
        );
    }

    #[test]
    fn test_revision_properties_empty() {
        let td = tempfile::tempdir().unwrap();
        let wt = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();

        // Create a file to commit
        let test_file = td.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        // Use add() with relative path instead of smart_add to avoid path issues
        wt.add(&[Path::new("test.txt")]).unwrap();

        // Create a commit without revision properties
        let revision_id = wt
            .build_commit()
            .message("Test commit without properties")
            .committer("Test User <test@example.com>")
            .commit()
            .unwrap();

        // Retrieve the revision and check properties
        let revision = repo.get_revision(&revision_id).unwrap();
        let properties = revision.get_properties();

        // Breezy automatically adds a "branch-nick" property
        // Just check that it exists and no other custom properties are present
        assert!(
            properties.contains_key("branch-nick"),
            "Expected branch-nick property"
        );
        assert!(!properties.contains_key("test-property"));
        assert!(!properties.contains_key("deb-pristine-delta-foo.tar.gz"));
    }
}
