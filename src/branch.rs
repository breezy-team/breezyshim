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
pub struct BranchFormat(Py<PyAny>);

impl Clone for BranchFormat {
    fn clone(&self) -> Self {
        Python::attach(|py| BranchFormat(self.0.clone_ref(py)))
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
        Python::attach(|py| {
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
    /// Get a reference to self as Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
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
    /// Bind this branch to another branch.
    ///
    /// Binding a branch means that commits to this branch will also be made
    /// to the master branch.
    ///
    /// # Parameters
    ///
    /// * `other` - The branch to bind to.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the branch could not be bound.
    fn bind(&self, other: &dyn Branch) -> Result<(), Error>;
    /// Unbind this branch from any master branch.
    ///
    /// After unbinding, commits will only be made to this branch.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the branch could not be unbound.
    fn unbind(&self) -> Result<(), Error>;
    /// Get the location of the branch this branch is bound to.
    ///
    /// # Returns
    ///
    /// The URL of the bound branch as a string, or None if not bound.
    fn get_bound_location(&self) -> Option<String>;
    /// Get the location this branch used to be bound to.
    ///
    /// # Returns
    ///
    /// The URL of the old bound branch as a string, or None if there was no previous binding.
    fn get_old_bound_location(&self) -> Option<String>;
    /// Check if this branch is locked.
    ///
    /// # Returns
    ///
    /// `true` if the branch is locked, `false` otherwise.
    fn is_locked(&self) -> bool;
    /// Get the current lock mode of the branch.
    ///
    /// # Returns
    ///
    /// 'r' for read lock, 'w' for write lock, or None if not locked.
    fn peek_lock_mode(&self) -> Option<char>;
    /// Get the revision ID for a given revision number.
    ///
    /// # Parameters
    ///
    /// * `revno` - The revision number.
    ///
    /// # Returns
    ///
    /// The revision ID corresponding to the revision number.
    fn get_rev_id(&self, revno: u32) -> Result<RevisionId, Error>;
    /// Convert a revision ID to its revision number.
    ///
    /// # Parameters
    ///
    /// * `revision_id` - The revision ID to convert.
    ///
    /// # Returns
    ///
    /// The revision number, or an error if the revision ID is not in the branch.
    fn revision_id_to_revno(&self, revision_id: &RevisionId) -> Result<u32, Error>;
    /// Check whether a revision number corresponds to a real revision.
    ///
    /// # Parameters
    ///
    /// * `revno` - The revision number to check.
    ///
    /// # Returns
    ///
    /// `true` if the revision number corresponds to a real revision, `false` otherwise.
    fn check_real_revno(&self, revno: u32) -> bool;
    /// Get information about the last revision.
    ///
    /// # Returns
    ///
    /// A tuple containing the revision number and revision ID of the last revision.
    fn last_revision_info(&self) -> (u32, RevisionId);
    /// Set the last revision information for this branch.
    ///
    /// # Parameters
    ///
    /// * `revno` - The revision number.
    /// * `revision_id` - The revision ID.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the information could not be set.
    fn set_last_revision_info(&self, revno: u32, revision_id: &RevisionId) -> Result<(), Error>;
    /// Get the URL this branch is stacked on.
    ///
    /// # Returns
    ///
    /// The URL of the stacked-on branch, or an error if not stacked.
    fn get_stacked_on_url(&self) -> Result<String, Error>;
    /// Set the URL this branch is stacked on.
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to stack on.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if stacking could not be set.
    fn set_stacked_on_url(&self, url: &str) -> Result<(), Error>;
    /// Copy revisions from another branch into this branch.
    ///
    /// # Parameters
    ///
    /// * `from_branch` - The branch to fetch revisions from.
    /// * `last_revision` - The last revision to fetch, or None to fetch all.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the fetch failed.
    fn fetch(
        &self,
        from_branch: &dyn Branch,
        last_revision: Option<&RevisionId>,
    ) -> Result<(), Error>;
    /// Update this branch to match the master branch.
    ///
    /// This is used when the branch is bound to synchronize changes.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the update failed.
    fn update(&self) -> Result<(), Error>;
    /// Set the location to push this branch to.
    ///
    /// # Parameters
    ///
    /// * `location` - The push location URL.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the location could not be set.
    fn set_push_location(&self, location: &str) -> Result<(), Error>;
    /// Set the public branch location.
    ///
    /// # Parameters
    ///
    /// * `location` - The public branch URL.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the location could not be set.
    fn set_public_branch(&self, location: &str) -> Result<(), Error>;
    /// Check if this branch is configured to only allow appending revisions.
    ///
    /// # Returns
    ///
    /// `true` if only appending is allowed, `false` otherwise.
    fn get_append_revisions_only(&self) -> bool;
    /// Set whether this branch should only allow appending revisions.
    ///
    /// # Parameters
    ///
    /// * `value` - Whether to only allow appending.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the setting could not be changed.
    fn set_append_revisions_only(&self, value: bool) -> Result<(), Error>;
}

/// Trait for branches that wrap Python branch objects.
///
/// This trait is implemented by branch types that wrap Breezy's Python branch objects.
pub trait PyBranch: Branch + Send + std::any::Any {
    /// Get the underlying Python object.
    fn to_object(&self, py: Python<'_>) -> Py<PyAny>;
}

impl dyn PyBranch {
    /// Get a reference to self as a Branch trait object.
    pub fn as_branch(&self) -> &dyn Branch {
        self
    }
}

impl<T: PyBranch> Branch for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn format(&self) -> BranchFormat {
        Python::attach(|py| BranchFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn vcs_type(&self) -> VcsType {
        self.repository().vcs_type()
    }

    fn revno(&self) -> u32 {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "revno")
                .unwrap()
                .extract(py)
                .unwrap()
        })
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

    fn tags(&self) -> Result<crate::tags::Tags, crate::error::Error> {
        Python::attach(|py| {
            Ok(crate::tags::Tags::from(
                self.to_object(py).getattr(py, "tags")?,
            ))
        })
    }

    fn repository(&self) -> GenericRepository {
        Python::attach(|py| {
            GenericRepository::new(self.to_object(py).getattr(py, "repository").unwrap())
        })
    }

    fn last_revision(&self) -> RevisionId {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, intern!(py, "last_revision"))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn name(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .getattr(py, "name")
                .unwrap()
                .extract::<Option<String>>(py)
                .unwrap()
        })
    }

    fn basis_tree(&self) -> Result<crate::tree::RevisionTree, crate::error::Error> {
        Python::attach(|py| {
            Ok(crate::tree::RevisionTree(
                self.to_object(py).call_method0(py, "basis_tree")?,
            ))
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
        Python::attach(|py| {
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
        Python::attach(|py| {
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
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_parent")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn set_parent(&mut self, parent: &str) {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "set_parent", (parent,))
                .unwrap();
        })
    }

    fn get_public_branch(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_public_branch")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_push_location(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_push_location")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_submit_branch(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_submit_branch")
                .unwrap()
                .extract(py)
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

    fn get_config(&self) -> crate::config::BranchConfig {
        Python::attach(|py| {
            crate::config::BranchConfig::new(
                self.to_object(py).call_method0(py, "get_config").unwrap(),
            )
        })
    }

    fn get_config_stack(&self) -> crate::config::ConfigStack {
        Python::attach(|py| {
            crate::config::ConfigStack::new(
                self.to_object(py)
                    .call_method0(py, "get_config_stack")
                    .unwrap(),
            )
        })
    }

    fn sprout(&self, to_controldir: &dyn PyControlDir, to_branch_name: &str) -> Result<(), Error> {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "generate_revision_history",
                (last_revision.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(())
        })
    }

    fn bind(&self, other: &dyn Branch) -> Result<(), Error> {
        Python::attach(|py| {
            // Try to downcast to concrete PyBranch types
            if let Some(gb) = other.as_any().downcast_ref::<GenericBranch>() {
                self.to_object(py)
                    .call_method1(py, "bind", (gb.to_object(py),))?;
            } else if let Some(mb) = other.as_any().downcast_ref::<MemoryBranch>() {
                self.to_object(py)
                    .call_method1(py, "bind", (mb.to_object(py),))?;
            } else {
                return Err(Error::Other(pyo3::exceptions::PyTypeError::new_err(
                    "Branch must be a PyBranch",
                )));
            }
            Ok(())
        })
    }

    fn unbind(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "unbind")?;
            Ok(())
        })
    }

    fn get_bound_location(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_bound_location")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_old_bound_location(&self) -> Option<String> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_old_bound_location")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn is_locked(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "is_locked")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn peek_lock_mode(&self) -> Option<char> {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method0(py, "peek_lock_mode")
                .unwrap();
            if result.is_none(py) {
                None
            } else {
                let mode: String = result.extract(py).unwrap();
                mode.chars().next()
            }
        })
    }

    fn get_rev_id(&self, revno: u32) -> Result<RevisionId, Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "get_rev_id", (revno,))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn revision_id_to_revno(&self, revision_id: &RevisionId) -> Result<u32, Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "revision_id_to_revno", (revision_id.clone(),))?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn check_real_revno(&self, revno: u32) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "check_real_revno", (revno,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn last_revision_info(&self) -> (u32, RevisionId) {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "last_revision_info")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn set_last_revision_info(&self, revno: u32, revision_id: &RevisionId) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "set_last_revision_info",
                (revno, revision_id.clone()),
            )?;
            Ok(())
        })
    }

    fn get_stacked_on_url(&self) -> Result<String, Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_stacked_on_url")?
                .extract(py)
                .map_err(Into::into)
        })
    }

    fn set_stacked_on_url(&self, url: &str) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "set_stacked_on_url", (url,))?;
            Ok(())
        })
    }

    fn fetch(
        &self,
        from_branch: &dyn Branch,
        last_revision: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(rev) = last_revision {
                kwargs.set_item("last_revision", rev.clone())?;
            }

            // Try to downcast to concrete PyBranch types
            if let Some(gb) = from_branch.as_any().downcast_ref::<GenericBranch>() {
                self.to_object(py)
                    .call_method(py, "fetch", (gb.to_object(py),), Some(&kwargs))?;
            } else if let Some(mb) = from_branch.as_any().downcast_ref::<MemoryBranch>() {
                self.to_object(py)
                    .call_method(py, "fetch", (mb.to_object(py),), Some(&kwargs))?;
            } else {
                return Err(Error::Other(pyo3::exceptions::PyTypeError::new_err(
                    "Branch must be a PyBranch",
                )));
            }
            Ok(())
        })
    }

    fn update(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "update")?;
            Ok(())
        })
    }

    fn set_push_location(&self, location: &str) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "set_push_location", (location,))?;
            Ok(())
        })
    }

    fn set_public_branch(&self, location: &str) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "set_public_branch", (location,))?;
            Ok(())
        })
    }

    fn get_append_revisions_only(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "get_append_revisions_only")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn set_append_revisions_only(&self, value: bool) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "set_append_revisions_only", (value,))?;
            Ok(())
        })
    }
}

/// A generic branch that can represent any type of branch.
///
/// This struct wraps a Python branch object and provides access to it through
/// the Branch trait.
pub struct GenericBranch(Py<PyAny>);

impl Clone for GenericBranch {
    fn clone(&self) -> Self {
        Python::attach(|py| GenericBranch(self.0.clone_ref(py)))
    }
}

impl PyBranch for GenericBranch {
    fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
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

impl<'a, 'py> FromPyObject<'a, 'py> for GenericBranch {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericBranch(ob.to_owned().unbind()))
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
pub struct MemoryBranch(Py<PyAny>);

impl Clone for MemoryBranch {
    fn clone(&self) -> Self {
        Python::attach(|py| MemoryBranch(self.0.clone_ref(py)))
    }
}

impl PyBranch for MemoryBranch {
    fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
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
        Python::attach(|py| {
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
) -> PyResult<Py<PyAny>> {
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
#[deprecated(
    since = "0.7.7",
    note = "Use `open_as_generic` instead to avoid unnecessary boxing"
)]
pub fn open(url: &url::Url) -> Result<Box<dyn Branch>, Error> {
    open_as_generic(url).map(|b| Box::new(b) as Box<dyn Branch>)
}

/// Open a branch at the specified URL, returning a GenericBranch.
///
/// This is similar to `open`, but returns a `GenericBranch` directly
/// instead of boxing it as a trait object. This is more efficient and allows the caller
/// to use `GenericBranch`-specific methods and traits like `Clone`.
///
/// # Parameters
///
/// * `url` - The URL of the branch to open.
///
/// # Returns
///
/// The opened branch, or an error if the branch could not be opened.
pub fn open_as_generic(url: &url::Url) -> Result<GenericBranch, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open", (url.to_string(),))?;
        Ok(GenericBranch::from(r))
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
#[deprecated(
    since = "0.7.7",
    note = "Use `open_containing_as_generic` instead to avoid unnecessary boxing"
)]
pub fn open_containing(url: &url::Url) -> Result<(Box<dyn Branch>, String), Error> {
    open_containing_as_generic(url).map(|(b, p)| (Box::new(b) as Box<dyn Branch>, p))
}

/// Find and open a branch containing the specified URL, returning a GenericBranch.
///
/// This is similar to `open_containing`, but returns a `GenericBranch` directly
/// instead of boxing it as a trait object. This is more efficient and allows the caller
/// to use `GenericBranch`-specific methods and traits like `Clone`.
///
/// # Parameters
///
/// * `url` - The URL to find a branch for.
///
/// # Returns
///
/// A tuple containing the opened branch and the relative path from the branch to
/// the specified URL, or an error if no branch could be found.
pub fn open_containing_as_generic(url: &url::Url) -> Result<(GenericBranch, String), Error> {
    Python::attach(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();

        let (b, p): (Bound<PyAny>, String) = c
            .call_method1("open_containing", (url.to_string(),))?
            .extract()?;

        Ok((GenericBranch(b.unbind()), p))
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
#[deprecated(
    since = "0.7.7",
    note = "Use `open_from_transport_as_generic` instead to avoid unnecessary boxing"
)]
pub fn open_from_transport(
    transport: &crate::transport::Transport,
) -> Result<Box<dyn Branch>, Error> {
    open_from_transport_as_generic(transport).map(|b| Box::new(b) as Box<dyn Branch>)
}

/// Open a branch from a transport, returning a GenericBranch.
///
/// This is similar to `open_from_transport`, but returns a `GenericBranch` directly
/// instead of boxing it as a trait object. This is more efficient and allows the caller
/// to use `GenericBranch`-specific methods and traits like `Clone`.
///
/// # Parameters
///
/// * `transport` - The transport to use for accessing the branch.
///
/// # Returns
///
/// The opened branch, or an error if the branch could not be opened.
pub fn open_from_transport_as_generic(
    transport: &crate::transport::Transport,
) -> Result<GenericBranch, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open_from_transport", (transport.as_pyobject(),))?;
        Ok(GenericBranch(r.unbind()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_clone() {
        crate::init();
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience_as_generic(
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
        crate::init();
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience_as_generic(
            &url,
            None,
            &crate::controldir::ControlDirFormat::default(),
        )
        .unwrap();
        let branch = MemoryBranch::new(&branch.repository(), None, &RevisionId::null());

        assert_eq!(branch.last_revision(), RevisionId::null());
    }
}
