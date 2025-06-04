//! Branches are the primary way to interact with the history of a project in Breezy.
//!
//! A branch is a named sequence of revisions. Each revision is a snapshot of the project at a
//! particular point in time. Revisions are linked together in a chain, forming a history of the
//! project. The branch itself is a pointer to the most recent revision in the chain.
//! Branches can be pushed to and pulled from other branches, allowing changes to be shared between
//! different branches.
//!
//! Breezy supports several different types of branches, each with different capabilities and
//! constraints.
use crate::controldir::{ControlDir, GenericControlDir, PyControlDir};
use crate::error::Error;
use crate::foreign::VcsType;
use crate::lock::Lock;
use crate::repository::{GenericRepository, PyRepository, Repository};
use crate::revisionid::RevisionId;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Format of a branch in a version control system.
///
/// This struct represents the format of a branch, which defines its capabilities
/// and constraints.
#[derive(Debug)]
pub struct BranchFormat(PyObject);

impl Clone for BranchFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| BranchFormat(self.0.clone_ref(py)))
    }
}

impl BranchFormat {
    /// Check if this branch format supports stacking.
    ///
    /// Stacking allows a branch to reference revisions in another branch
    /// without duplicating their storage.
    ///
    /// # Returns
    ///
    /// `true` if the branch format supports stacking, `false` otherwise.
    pub fn supports_stacking(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "supports_stacking")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

/// Trait representing a branch in a version control system.
///
/// A branch is a named sequence of revisions. Each revision is a snapshot of the project
/// at a particular point in time. This trait provides methods for interacting with
/// branches across various version control systems.
pub trait Branch {
    /// Get the format of this branch.
    ///
    /// # Returns
    ///
    /// The format of this branch.
    fn format(&self) -> BranchFormat;
    /// Get the type of version control system for this branch.
    ///
    /// # Returns
    ///
    /// The version control system type.
    fn vcs_type(&self) -> VcsType;
    /// Get the revision number of the last revision in this branch.
    ///
    /// # Returns
    ///
    /// The revision number.
    fn revno(&self) -> u32;
    /// Lock the branch for reading.
    ///
    /// This method acquires a read lock on the branch, which allows reading from the
    /// branch but prevents others from writing to it.
    ///
    /// # Returns
    ///
    /// A lock object that will release the lock when dropped, or an error if the
    /// lock could not be acquired.
    fn lock_read(&self) -> Result<Lock, crate::error::Error>;
    /// Lock the branch for writing.
    ///
    /// This method acquires a write lock on the branch, which allows writing to the
    /// branch but prevents others from reading from or writing to it.
    ///
    /// # Returns
    ///
    /// A lock object that will release the lock when dropped, or an error if the
    /// lock could not be acquired.
    fn lock_write(&self) -> Result<Lock, crate::error::Error>;
    /// Get the tags for this branch.
    ///
    /// Tags are names associated with specific revisions in the branch.
    ///
    /// # Returns
    ///
    /// The tags object for this branch, or an error if the tags could not be retrieved.
    fn tags(&self) -> Result<crate::tags::Tags, crate::error::Error>;
    /// Get the repository associated with this branch.
    ///
    /// # Returns
    ///
    /// The repository containing this branch.
    fn repository(&self) -> GenericRepository;
    /// Get the last revision in this branch.
    ///
    /// # Returns
    ///
    /// The revision ID of the last revision in this branch.
    fn last_revision(&self) -> RevisionId;
    /// Get the name of this branch.
    ///
    /// # Returns
    ///
    /// The name of this branch, or None if it doesn't have a name.
    fn name(&self) -> Option<String>;
    /// Get the basis tree for this branch.
    ///
    /// The basis tree is the tree corresponding to the last revision in this branch.
    ///
    /// # Returns
    ///
    /// The basis tree, or an error if it could not be retrieved.
    fn basis_tree(&self) -> Result<crate::tree::RevisionTree, crate::error::Error>;
    /// Get the user-visible URL for this branch.
    ///
    /// # Returns
    ///
    /// The URL that can be used to access this branch.
    fn get_user_url(&self) -> url::Url;
    /// Get the control directory for this branch.
    ///
    /// # Returns
    ///
    /// The control directory containing this branch.
    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = crate::repository::GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >;

    /// Push this branch to a remote branch.
    ///
    /// # Parameters
    ///
    /// * `remote_branch` - The remote branch to push to.
    /// * `overwrite` - Whether to overwrite the remote branch if it has diverged.
    /// * `stop_revision` - The revision to stop pushing at, or None to push all revisions.
    /// * `tag_selector` - A function that selects which tags to push, or None to push all tags.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the push failed.
    fn push(
        &self,
        remote_branch: &dyn PyBranch,
        overwrite: bool,
        stop_revision: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(), crate::error::Error>;

    /// Pull from a source branch into this branch.
    ///
    /// # Parameters
    ///
    /// * `source_branch` - The branch to pull from.
    /// * `overwrite` - Whether to overwrite this branch if it has diverged from the source.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the pull failed.
    fn pull(&self, source_branch: &dyn PyBranch, overwrite: Option<bool>) -> Result<(), Error>;
    /// Get the parent branch location.
    ///
    /// # Returns
    ///
    /// The parent branch location as a string, or None if there is no parent branch.
    fn get_parent(&self) -> Option<String>;
    /// Set the parent branch location.
    ///
    /// # Parameters
    ///
    /// * `parent` - The new parent branch location.
    fn set_parent(&mut self, parent: &str);
    /// Get the public branch location.
    ///
    /// # Returns
    ///
    /// The public branch location as a string, or None if there is no public branch.
    fn get_public_branch(&self) -> Option<String>;
    /// Get the push location for this branch.
    ///
    /// # Returns
    ///
    /// The push location as a string, or None if there is no push location.
    fn get_push_location(&self) -> Option<String>;
    /// Get the submit branch location.
    ///
    /// # Returns
    ///
    /// The submit branch location as a string, or None if there is no submit branch.
    fn get_submit_branch(&self) -> Option<String>;
    /// Get a transport for accessing this branch's user files.
    ///
    /// # Returns
    ///
    /// A transport for accessing this branch's user files.
    fn user_transport(&self) -> crate::transport::Transport;
    /// Get the configuration for this branch.
    ///
    /// # Returns
    ///
    /// The branch configuration.
    fn get_config(&self) -> crate::config::BranchConfig;
    /// Get the configuration stack for this branch.
    ///
    /// # Returns
    ///
    /// The configuration stack for this branch, which includes branch-specific,
    /// repository-specific, and global configuration.
    fn get_config_stack(&self) -> crate::config::ConfigStack;

    /// Create a new branch from this branch.
    ///
    /// # Parameters
    ///
    /// * `to_controldir` - The control directory to create the new branch in.
    /// * `to_branch_name` - The name of the new branch.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the branch could not be created.
    fn sprout(&self, to_controldir: &dyn PyControlDir, to_branch_name: &str) -> Result<(), Error>;
    /// Create a checkout of this branch.
    ///
    /// # Parameters
    ///
    /// * `to_location` - The location to create the checkout at.
    ///
    /// # Returns
    ///
    /// The working tree for the checkout, or an error if the checkout could not be created.
    fn create_checkout(
        &self,
        to_location: &std::path::Path,
    ) -> Result<crate::workingtree::GenericWorkingTree, Error>;
    /// Generate the revision history for this branch.
    ///
    /// # Parameters
    ///
    /// * `last_revision` - The last revision to include in the history.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the history could not be generated.
    fn generate_revision_history(&self, last_revision: &RevisionId) -> Result<(), Error>;
}

/// Trait for branches that wrap Python branch objects.
///
/// This trait is implemented by branch types that wrap Breezy's Python branch objects.
pub trait PyBranch: Send + std::any::Any {
    /// Get the underlying Python object.
    fn to_object(&self, py: Python<'_>) -> PyObject;
}

impl<T: PyBranch> Branch for T {
    fn format(&self) -> BranchFormat {
        Python::with_gil(|py| BranchFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn vcs_type(&self) -> VcsType {
        self.repository().vcs_type()
    }

    fn revno(&self) -> u32 {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "revno")
                .unwrap()
                .extract(py)
                .unwrap()
        })
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

    fn tags(&self) -> Result<crate::tags::Tags, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(crate::tags::Tags::from(
                self.to_object(py).getattr(py, "tags")?,
            ))
        })
    }

    fn repository(&self) -> GenericRepository {
        Python::with_gil(|py| {
            GenericRepository::new(self.to_object(py).getattr(py, "repository").unwrap())
        })
    }

    fn last_revision(&self) -> RevisionId {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, intern!(py, "last_revision"))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn name(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "name")
                .unwrap()
                .extract::<Option<String>>(py)
                .unwrap()
        })
    }

    fn basis_tree(&self) -> Result<crate::tree::RevisionTree, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(crate::tree::RevisionTree(
                self.to_object(py).call_method0(py, "basis_tree")?,
            ))
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
            let url = self
                .to_object(py)
                .getattr(py, "user_url")
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            url.parse::<url::Url>().unwrap()
        })
    }

    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = crate::repository::GenericRepository,
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
                        Repository = crate::repository::GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >
        })
    }

    fn push(
        &self,
        remote_branch: &dyn PyBranch,
        overwrite: bool,
        stop_revision: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("overwrite", overwrite)?;
            if let Some(stop_revision) = stop_revision {
                kwargs.set_item("stop_revision", stop_revision.clone())?;
            }
            if let Some(tag_selector) = tag_selector {
                kwargs.set_item("tag_selector", py_tag_selector(py, tag_selector)?)?;
            }
            self.to_object(py).call_method(
                py,
                "push",
                (&remote_branch.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn pull(&self, source_branch: &dyn PyBranch, overwrite: Option<bool>) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            if let Some(overwrite) = overwrite {
                kwargs.set_item("overwrite", overwrite)?;
            }
            self.to_object(py).call_method(
                py,
                "pull",
                (&source_branch.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn get_parent(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_parent")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn set_parent(&mut self, parent: &str) {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "set_parent", (parent,))
                .unwrap();
        })
    }

    fn get_public_branch(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_public_branch")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_push_location(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_push_location")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_submit_branch(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_submit_branch")
                .unwrap()
                .extract(py)
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

    fn get_config(&self) -> crate::config::BranchConfig {
        Python::with_gil(|py| {
            crate::config::BranchConfig::new(
                self.to_object(py).call_method0(py, "get_config").unwrap(),
            )
        })
    }

    fn get_config_stack(&self) -> crate::config::ConfigStack {
        Python::with_gil(|py| {
            crate::config::ConfigStack::new(
                self.to_object(py)
                    .call_method0(py, "get_config_stack")
                    .unwrap(),
            )
        })
    }

    fn sprout(&self, to_controldir: &dyn PyControlDir, to_branch_name: &str) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("name", to_branch_name)?;
            self.to_object(py).call_method(
                py,
                "sprout",
                (to_controldir.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn create_checkout(
        &self,
        to_location: &std::path::Path,
    ) -> Result<crate::workingtree::GenericWorkingTree, Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(
                    py,
                    "create_checkout",
                    (to_location.to_string_lossy().to_string(),),
                )
                .map(crate::workingtree::GenericWorkingTree)
                .map_err(|e| e.into())
        })
    }

    fn generate_revision_history(&self, last_revision: &RevisionId) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method1(
                py,
                "generate_revision_history",
                (last_revision.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(())
        })
    }
}

/// A generic branch that can represent any type of branch.
///
/// This struct wraps a Python branch object and provides access to it through
/// the Branch trait.
pub struct GenericBranch(PyObject);

impl Clone for GenericBranch {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericBranch(self.0.clone_ref(py)))
    }
}

impl PyBranch for GenericBranch {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl<'py> IntoPyObject<'py> for GenericBranch {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl FromPyObject<'_> for GenericBranch {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericBranch(ob.clone().unbind()))
    }
}

impl<'py> From<Bound<'py, PyAny>> for GenericBranch {
    fn from(ob: Bound<PyAny>) -> Self {
        GenericBranch(ob.unbind())
    }
}

impl From<Py<PyAny>> for GenericBranch {
    fn from(gb: Py<PyAny>) -> Self {
        GenericBranch(gb)
    }
}

/// A branch that exists only in memory.
///
/// Memory branches are not backed by a persistent storage and are primarily
/// used for testing or temporary operations.
pub struct MemoryBranch(PyObject);

impl Clone for MemoryBranch {
    fn clone(&self) -> Self {
        Python::with_gil(|py| MemoryBranch(self.0.clone_ref(py)))
    }
}

impl PyBranch for MemoryBranch {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl MemoryBranch {
    /// Create a new MemoryBranch.
    ///
    /// # Parameters
    ///
    /// * `repository` - The repository to use for this memory branch.
    /// * `revno` - Optional revision number to use as the last revision.
    /// * `revid` - The revision ID to use as the last revision.
    ///
    /// # Returns
    ///
    /// A new MemoryBranch instance.
    pub fn new<R: PyRepository>(repository: &R, revno: Option<u32>, revid: &RevisionId) -> Self {
        Python::with_gil(|py| {
            let mb_cls = py
                .import("breezy.memorybranch")
                .unwrap()
                .getattr("MemoryBranch")
                .unwrap();

            let o = mb_cls
                .call1((repository.to_object(py), (revno, revid.clone())))
                .unwrap();

            MemoryBranch(o.unbind())
        })
    }
}

pub(crate) fn py_tag_selector(
    py: Python,
    tag_selector: Box<dyn Fn(String) -> bool>,
) -> PyResult<PyObject> {
    #[pyclass(unsendable)]
    struct PyTagSelector(Box<dyn Fn(String) -> bool>);

    #[pymethods]
    impl PyTagSelector {
        fn __call__(&self, tag: String) -> bool {
            (self.0)(tag)
        }
    }
    Ok(PyTagSelector(tag_selector)
        .into_pyobject(py)
        .unwrap()
        .unbind()
        .into())
}

/// Open a branch at the specified URL.
///
/// # Parameters
///
/// * `url` - The URL of the branch to open.
///
/// # Returns
///
/// The opened branch, or an error if the branch could not be opened.
pub fn open(url: &url::Url) -> Result<Box<dyn Branch>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open", (url.to_string(),))?;
        Ok(Box::new(GenericBranch::from(r)) as Box<dyn Branch>)
    })
}

/// Find and open a branch containing the specified URL.
///
/// This function searches for a branch containing the specified URL and returns
/// the branch and the relative path from the branch to the specified URL.
///
/// # Parameters
///
/// * `url` - The URL to find a branch for.
///
/// # Returns
///
/// A tuple containing the opened branch and the relative path from the branch to
/// the specified URL, or an error if no branch could be found.
pub fn open_containing(url: &url::Url) -> Result<(Box<dyn Branch>, String), Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();

        let (b, p): (Bound<PyAny>, String) = c
            .call_method1("open_containing", (url.to_string(),))?
            .extract()?;

        Ok((Box::new(GenericBranch(b.unbind())) as Box<dyn Branch>, p))
    })
}

/// Open a branch from a transport.
///
/// # Parameters
///
/// * `transport` - The transport to use for accessing the branch.
///
/// # Returns
///
/// The opened branch, or an error if the branch could not be opened.
pub fn open_from_transport(
    transport: &crate::transport::Transport,
) -> Result<Box<dyn Branch>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open_from_transport", (transport.as_pyobject(),))?;
        Ok(Box::new(GenericBranch(r.unbind())) as Box<dyn Branch>)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_clone() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience(
            &url,
            None,
            &crate::controldir::ControlDirFormat::default(),
        )
        .unwrap();

        assert_eq!(branch.revno(), 0);
        assert_eq!(branch.last_revision(), RevisionId::null());
    }

    #[test]
    fn test_create_and_clone_memory() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience(
            &url,
            None,
            &crate::controldir::ControlDirFormat::default(),
        )
        .unwrap();
        let branch = MemoryBranch::new(&branch.repository(), None, &RevisionId::null());

        assert_eq!(branch.last_revision(), RevisionId::null());
    }
}
