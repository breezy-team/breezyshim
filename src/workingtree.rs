//! Working trees in version control systems.
//!
//! This module provides functionality for working with working trees, which are
//! local directories containing the files of a branch that can be edited.
use crate::branch::{Branch, GenericBranch, PyBranch};
use crate::controldir::{ControlDir, GenericControlDir};
use crate::error::Error;
use crate::tree::{MutableTree, PyMutableTree, PyTree, RevisionTree};
use crate::RevisionId;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

/// Trait representing a working tree in a version control system.
///
/// A working tree is a local directory containing the files of a branch that can
/// be edited. This trait provides methods for interacting with working trees
/// across various version control systems.
pub trait WorkingTree: MutableTree {
    /// Get the base directory path of this working tree.
    ///
    /// # Returns
    ///
    /// The absolute path to the root directory of this working tree.
    fn basedir(&self) -> PathBuf;

    /// Get the control directory for this working tree.
    ///
    /// # Returns
    ///
    /// The control directory containing this working tree.
    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = crate::repository::GenericRepository,
            WorkingTree = GenericWorkingTree,
        >,
    >;

    /// Get the branch associated with this working tree.
    ///
    /// # Returns
    ///
    /// The branch that this working tree is tracking.
    fn branch(&self) -> GenericBranch;

    /// Get the user-visible URL for this working tree.
    ///
    /// # Returns
    ///
    /// The URL that can be used to access this working tree.
    fn get_user_url(&self) -> url::Url;

    /// Check if this working tree supports setting the last revision.
    ///
    /// # Returns
    ///
    /// `true` if the working tree supports setting the last revision, `false` otherwise.
    fn supports_setting_file_ids(&self) -> bool;

    /// Add specified files to version control and the working tree.
    ///
    /// # Parameters
    ///
    /// * `files` - The list of file paths to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the files could not be added.
    fn smart_add(&self, files: &[&Path]) -> Result<(), Error>;

    /// Update the working tree to a specific revision.
    ///
    /// # Parameters
    ///
    /// * `revision_id` - The revision to update to, or None for the latest.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the update failed.
    fn update(&self, revision_id: Option<&RevisionId>) -> Result<(), Error>;

    /// Revert changes in the working tree.
    ///
    /// # Parameters
    ///
    /// * `filenames` - Optional list of specific files to revert.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the revert failed.
    fn revert(&self, filenames: Option<&[&Path]>) -> Result<(), Error>;

    /// Create a commit builder for this working tree.
    ///
    /// # Returns
    ///
    /// A new CommitBuilder instance for this working tree.
    fn build_commit(&self) -> CommitBuilder;

    /// Get the basis tree for this working tree.
    ///
    /// # Returns
    ///
    /// The basis tree that this working tree is based on.
    fn basis_tree(&self) -> Result<RevisionTree, Error>;

    /// Check if a path is a control filename in this working tree.
    ///
    /// Control filenames are filenames that are used by the version control system
    /// for its own purposes, like .git or .bzr.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// `true` if the path is a control filename, `false` otherwise.
    fn is_control_filename(&self, path: &Path) -> bool;

    /// Get a revision tree for a specific revision.
    ///
    /// # Parameters
    ///
    /// * `revision_id` - The ID of the revision to get the tree for.
    ///
    /// # Returns
    ///
    /// The revision tree, or an error if it could not be retrieved.
    fn revision_tree(&self, revision_id: &RevisionId) -> Result<Box<RevisionTree>, Error>;

    /// Convert a path to an absolute path relative to the working tree.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to convert.
    ///
    /// # Returns
    ///
    /// The absolute path, or an error if the conversion failed.
    fn abspath(&self, path: &Path) -> Result<PathBuf, Error>;

    /// Convert an absolute path to a path relative to the working tree.
    ///
    /// # Parameters
    ///
    /// * `path` - The absolute path to convert.
    ///
    /// # Returns
    ///
    /// The relative path, or an error if the conversion failed.
    fn relpath(&self, path: &Path) -> Result<PathBuf, Error>;

    /// Pull changes from another branch into this working tree.
    ///
    /// # Parameters
    ///
    /// * `source` - The branch to pull from.
    /// * `overwrite` - Whether to overwrite diverged changes.
    /// * `stop_revision` - The revision to stop pulling at.
    /// * `local` - Whether to only pull locally accessible revisions.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the pull could not be completed.
    fn pull(
        &self,
        source: &dyn Branch,
        overwrite: Option<bool>,
        stop_revision: Option<&RevisionId>,
        local: Option<bool>,
    ) -> Result<(), Error>;

    /// Merge changes from another branch into this working tree.
    ///
    /// # Parameters
    ///
    /// * `source` - The branch to merge from.
    /// * `to_revision` - The revision to merge up to.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the merge could not be completed.
    fn merge_from_branch(
        &self,
        source: &dyn Branch,
        to_revision: Option<&RevisionId>,
    ) -> Result<(), Error>;

    /// Convert a list of files to relative paths safely.
    ///
    /// This function takes a list of file paths and converts them to paths relative
    /// to the working tree, with various safety checks.
    ///
    /// # Parameters
    ///
    /// * `file_list` - The list of file paths to convert.
    /// * `canonicalize` - Whether to canonicalize the paths first.
    /// * `apply_view` - Whether to apply the view (if any) to the paths.
    ///
    /// # Returns
    ///
    /// A list of converted paths, or an error if the conversion failed.
    fn safe_relpath_files(
        &self,
        file_list: &[&Path],
        canonicalize: bool,
        apply_view: bool,
    ) -> Result<Vec<PathBuf>, Error>;

    /// Add conflicts to the working tree.
    fn add_conflicts(&self, conflicts: &[crate::tree::Conflict]) -> Result<(), Error>;

    /// Add a parent tree.
    fn add_parent_tree(
        &self,
        parent_id: &RevisionId,
        parent_tree: &crate::tree::RevisionTree,
    ) -> Result<(), Error>;

    /// Add a parent tree ID.
    fn add_parent_tree_id(&self, parent_id: &RevisionId) -> Result<(), Error>;

    /// Add a pending merge.
    fn add_pending_merge(&self, revision_id: &RevisionId) -> Result<(), Error>;

    /// Auto-resolve conflicts.
    fn auto_resolve(&self) -> Result<(), Error>;

    /// Check the state of the working tree.
    fn check_state(&self) -> Result<(), Error>;

    /// Get the canonical path for a file.
    fn get_canonical_path(&self, path: &Path) -> Result<PathBuf, Error>;

    /// Get canonical paths for multiple files.
    fn get_canonical_paths(&self, paths: &[&Path]) -> Result<Vec<PathBuf>, Error>;

    /// Get the configuration stack.
    fn get_config_stack(&self) -> Result<Py<PyAny>, Error>;

    /// Get reference information.
    fn get_reference_info(&self, path: &Path) -> Result<Option<(String, PathBuf)>, Error>;

    /// Get the shelf manager.
    fn get_shelf_manager(&self) -> Result<Py<PyAny>, Error>;

    /// Get ignored files.
    fn ignored_files(&self) -> Result<Vec<PathBuf>, Error>;

    /// Check if the working tree is locked.
    fn is_locked(&self) -> bool;

    /// Get merge-modified files.
    fn merge_modified(&self) -> Result<Vec<PathBuf>, Error>;

    /// Move files within the working tree.
    fn move_files(&self, from_paths: &[&Path], to_dir: &Path) -> Result<(), Error>;

    /// Set conflicts in the working tree.
    fn set_conflicts(&self, conflicts: &[crate::tree::Conflict]) -> Result<(), Error>;

    /// Set the last revision.
    fn set_last_revision(&self, revision_id: &RevisionId) -> Result<(), Error>;

    /// Set merge-modified files.
    fn set_merge_modified(&self, files: &[&Path]) -> Result<(), Error>;

    /// Set pending merges.
    fn set_pending_merges(&self, revision_ids: &[RevisionId]) -> Result<(), Error>;

    /// Set reference information.
    fn set_reference_info(
        &self,
        path: &Path,
        location: &str,
        file_id: Option<&str>,
    ) -> Result<(), Error>;

    /// Subsume a tree into this working tree.
    fn subsume(&self, other: &dyn PyWorkingTree) -> Result<(), Error>;

    /// Store uncommitted changes.
    fn store_uncommitted(&self) -> Result<String, Error>;

    /// Restore uncommitted changes.
    fn restore_uncommitted(&self) -> Result<(), Error>;

    /// Extract the working tree to a directory.
    fn extract(&self, dest: &Path, format: Option<&str>) -> Result<(), Error>;

    /// Clone the working tree.
    fn clone(
        &self,
        dest: &Path,
        revision_id: Option<&RevisionId>,
    ) -> Result<GenericWorkingTree, Error>;

    /// Get a control transport.
    fn control_transport(&self) -> Result<crate::transport::Transport, Error>;

    /// Get the control URL.
    fn control_url(&self) -> url::Url;

    /// Copy content into this working tree.
    fn copy_content_into(
        &self,
        source: &dyn PyTree,
        revision_id: Option<&RevisionId>,
    ) -> Result<(), Error>;

    /// Flush any pending changes.
    fn flush(&self) -> Result<(), Error>;

    /// Check if the working tree requires a rich root.
    fn requires_rich_root(&self) -> bool;

    /// Reset the state of the working tree.
    fn reset_state(&self, revision_ids: Option<&[RevisionId]>) -> Result<(), Error>;

    /// Reference a parent tree.
    fn reference_parent(
        &self,
        path: &Path,
        branch: &dyn Branch,
        revision_id: Option<&RevisionId>,
    ) -> Result<(), Error>;

    /// Check if the working tree supports merge-modified tracking.
    fn supports_merge_modified(&self) -> bool;

    /// Break the lock on the working tree.
    fn break_lock(&self) -> Result<(), Error>;

    /// Get the physical lock status.
    fn get_physical_lock_status(&self) -> Result<bool, Error>;
}

/// Trait for working trees that wrap Python working tree objects.
///
/// This trait is implemented by working tree types that wrap Python working tree objects.
pub trait PyWorkingTree: PyMutableTree + WorkingTree {}

impl dyn PyWorkingTree {
    /// Get a reference to self as a WorkingTree trait object.
    pub fn as_working_tree(&self) -> &dyn WorkingTree {
        self
    }
}

impl<T: ?Sized + PyWorkingTree> WorkingTree for T {
    fn basedir(&self) -> PathBuf {
        Python::attach(|py| {
            let path: String = self
                .to_object(py)
                .getattr(py, "basedir")
                .unwrap()
                .extract(py)
                .unwrap();
            PathBuf::from(path)
        })
    }

    fn controldir(
        &self,
    ) -> Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = crate::repository::GenericRepository,
            WorkingTree = GenericWorkingTree,
        >,
    > {
        Python::attach(|py| {
            let controldir = self.to_object(py).getattr(py, "controldir").unwrap();
            Box::new(GenericControlDir::new(controldir))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = crate::repository::GenericRepository,
                        WorkingTree = GenericWorkingTree,
                    >,
                >
        })
    }

    fn branch(&self) -> GenericBranch {
        Python::attach(|py| GenericBranch::from(self.to_object(py).getattr(py, "branch").unwrap()))
    }

    fn get_user_url(&self) -> url::Url {
        Python::attach(|py| {
            let url: String = self
                .to_object(py)
                .getattr(py, "user_url")
                .unwrap()
                .extract(py)
                .unwrap();
            url.parse().unwrap()
        })
    }

    fn supports_setting_file_ids(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn smart_add(&self, files: &[&Path]) -> Result<(), Error> {
        Python::attach(|py| {
            let file_paths: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "smart_add", (file_paths,))?;
            Ok(())
        })
    }

    fn update(&self, revision_id: Option<&RevisionId>) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "update", (revision_id.cloned(),))?;
            Ok(())
        })
    }

    fn revert(&self, filenames: Option<&[&Path]>) -> Result<(), Error> {
        Python::attach(|py| {
            let file_paths = filenames.map(|files| {
                files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            });
            self.to_object(py)
                .call_method1(py, "revert", (file_paths,))?;
            Ok(())
        })
    }

    fn build_commit(&self) -> CommitBuilder {
        Python::attach(|py| CommitBuilder::from(GenericWorkingTree(self.to_object(py))))
    }

    fn basis_tree(&self) -> Result<RevisionTree, Error> {
        Python::attach(|py| {
            let basis_tree = self.to_object(py).call_method0(py, "basis_tree")?;
            Ok(RevisionTree(basis_tree))
        })
    }

    fn is_control_filename(&self, path: &Path) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(
                    py,
                    "is_control_filename",
                    (path.to_string_lossy().as_ref(),),
                )
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Get a revision tree for a specific revision.
    fn revision_tree(&self, revision_id: &RevisionId) -> Result<Box<RevisionTree>, Error> {
        Python::attach(|py| {
            let tree = self.to_object(py).call_method1(
                py,
                "revision_tree",
                (revision_id.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(Box::new(RevisionTree(tree)))
        })
    }

    /// Convert a path to an absolute path relative to the working tree.
    fn abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "abspath", (path.to_string_lossy().as_ref(),))?
                .extract(py)?)
        })
    }

    /// Convert an absolute path to a path relative to the working tree.
    fn relpath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "relpath", (path.to_string_lossy().as_ref(),))?
                .extract(py)?)
        })
    }

    /// Pull changes from another branch into this working tree.
    fn pull(
        &self,
        source: &dyn Branch,
        overwrite: Option<bool>,
        stop_revision: Option<&RevisionId>,
        local: Option<bool>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new(py);
                if let Some(overwrite) = overwrite {
                    kwargs.set_item("overwrite", overwrite).unwrap();
                }
                if let Some(stop_revision) = stop_revision {
                    kwargs
                        .set_item(
                            "stop_revision",
                            stop_revision.clone().into_pyobject(py).unwrap(),
                        )
                        .unwrap();
                }
                if let Some(local) = local {
                    kwargs.set_item("local", local).unwrap();
                }
                kwargs
            };
            // Try to cast to a concrete type that implements PyBranch
            let py_obj =
                if let Some(generic_branch) = source.as_any().downcast_ref::<GenericBranch>() {
                    generic_branch.to_object(py)
                } else if let Some(py_branch) = source
                    .as_any()
                    .downcast_ref::<crate::branch::MemoryBranch>()
                {
                    py_branch.to_object(py)
                } else {
                    return Err(Error::Other(
                        PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                            "Branch must be a PyBranch implementation for pull operation",
                        ),
                    ));
                };
            self.to_object(py)
                .call_method(py, "pull", (py_obj,), Some(&kwargs))?;
            Ok(())
        })
    }

    /// Merge changes from another branch into this working tree.
    fn merge_from_branch(
        &self,
        source: &dyn Branch,
        to_revision: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new(py);
                if let Some(to_revision) = to_revision {
                    kwargs
                        .set_item(
                            "to_revision",
                            to_revision.clone().into_pyobject(py).unwrap(),
                        )
                        .unwrap();
                }
                kwargs
            };
            // Try to cast to a concrete type that implements PyBranch
            let py_obj =
                if let Some(generic_branch) = source.as_any().downcast_ref::<GenericBranch>() {
                    generic_branch.to_object(py)
                } else if let Some(py_branch) = source
                    .as_any()
                    .downcast_ref::<crate::branch::MemoryBranch>()
                {
                    py_branch.to_object(py)
                } else {
                    return Err(Error::Other(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Branch must be a PyBranch implementation for merge_from_branch operation"
                )));
                };
            self.to_object(py)
                .call_method(py, "merge_from_branch", (py_obj,), Some(&kwargs))?;
            Ok(())
        })
    }

    /// Convert a list of files to relative paths safely.
    fn safe_relpath_files(
        &self,
        file_list: &[&Path],
        canonicalize: bool,
        apply_view: bool,
    ) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method1(
                py,
                "safe_relpath_files",
                (
                    file_list
                        .iter()
                        .map(|x| x.to_string_lossy().to_string())
                        .collect::<Vec<_>>(),
                    canonicalize,
                    apply_view,
                ),
            )?;
            Ok(result.extract(py)?)
        })
    }

    fn add_conflicts(&self, conflicts: &[crate::tree::Conflict]) -> Result<(), Error> {
        Python::attach(|py| {
            let conflicts_py: Vec<Py<PyAny>> = conflicts
                .iter()
                .map(|c| {
                    let dict = pyo3::types::PyDict::new(py);
                    dict.set_item("path", c.path.to_string_lossy().to_string())
                        .unwrap();
                    dict.set_item("typestring", &c.conflict_type).unwrap();
                    if let Some(ref msg) = c.message {
                        dict.set_item("message", msg).unwrap();
                    }
                    dict.into_any().unbind()
                })
                .collect();
            self.to_object(py)
                .call_method1(py, "add_conflicts", (conflicts_py,))?;
            Ok(())
        })
    }

    fn add_parent_tree(
        &self,
        parent_id: &RevisionId,
        parent_tree: &crate::tree::RevisionTree,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "add_parent_tree",
                (
                    parent_id.clone().into_pyobject(py).unwrap(),
                    parent_tree.to_object(py),
                ),
            )?;
            Ok(())
        })
    }

    fn add_parent_tree_id(&self, parent_id: &RevisionId) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "add_parent_tree_id",
                (parent_id.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(())
        })
    }

    fn add_pending_merge(&self, revision_id: &RevisionId) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "add_pending_merge",
                (revision_id.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(())
        })
    }

    fn auto_resolve(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "auto_resolve")?;
            Ok(())
        })
    }

    fn check_state(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "check_state")?;
            Ok(())
        })
    }

    fn get_canonical_path(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "get_canonical_path", (path.to_string_lossy().as_ref(),))?
                .extract(py)?)
        })
    }

    fn get_canonical_paths(&self, paths: &[&Path]) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            let path_strings: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            Ok(self
                .to_object(py)
                .call_method1(py, "get_canonical_paths", (path_strings,))?
                .extract(py)?)
        })
    }

    fn get_config_stack(&self) -> Result<Py<PyAny>, Error> {
        Python::attach(|py| Ok(self.to_object(py).call_method0(py, "get_config_stack")?))
    }

    fn get_reference_info(&self, path: &Path) -> Result<Option<(String, PathBuf)>, Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method1(
                py,
                "get_reference_info",
                (path.to_string_lossy().as_ref(),),
            )?;
            if result.is_none(py) {
                Ok(None)
            } else {
                let tuple: (String, String) = result.extract(py)?;
                Ok(Some((tuple.0, PathBuf::from(tuple.1))))
            }
        })
    }

    fn get_shelf_manager(&self) -> Result<Py<PyAny>, Error> {
        Python::attach(|py| Ok(self.to_object(py).call_method0(py, "get_shelf_manager")?))
    }

    fn ignored_files(&self) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, "ignored_files")?
                .extract(py)?)
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

    fn merge_modified(&self) -> Result<Vec<PathBuf>, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, "merge_modified")?
                .extract(py)?)
        })
    }

    fn move_files(&self, from_paths: &[&Path], to_dir: &Path) -> Result<(), Error> {
        Python::attach(|py| {
            let from_strings: Vec<String> = from_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py).call_method1(
                py,
                "move",
                (from_strings, to_dir.to_string_lossy().as_ref()),
            )?;
            Ok(())
        })
    }

    fn set_conflicts(&self, conflicts: &[crate::tree::Conflict]) -> Result<(), Error> {
        Python::attach(|py| {
            let conflicts_py: Vec<Py<PyAny>> = conflicts
                .iter()
                .map(|c| {
                    let dict = pyo3::types::PyDict::new(py);
                    dict.set_item("path", c.path.to_string_lossy().to_string())
                        .unwrap();
                    dict.set_item("typestring", &c.conflict_type).unwrap();
                    if let Some(ref msg) = c.message {
                        dict.set_item("message", msg).unwrap();
                    }
                    dict.into_any().unbind()
                })
                .collect();
            self.to_object(py)
                .call_method1(py, "set_conflicts", (conflicts_py,))?;
            Ok(())
        })
    }

    fn set_last_revision(&self, revision_id: &RevisionId) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "set_last_revision",
                (revision_id.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(())
        })
    }

    fn set_merge_modified(&self, files: &[&Path]) -> Result<(), Error> {
        Python::attach(|py| {
            let file_strings: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "set_merge_modified", (file_strings,))?;
            Ok(())
        })
    }

    fn set_pending_merges(&self, revision_ids: &[RevisionId]) -> Result<(), Error> {
        Python::attach(|py| {
            let revision_ids_py: Vec<Py<PyAny>> = revision_ids
                .iter()
                .map(|id| id.clone().into_pyobject(py).unwrap().unbind())
                .collect();
            self.to_object(py)
                .call_method1(py, "set_pending_merges", (revision_ids_py,))?;
            Ok(())
        })
    }

    fn set_reference_info(
        &self,
        path: &Path,
        location: &str,
        file_id: Option<&str>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(file_id) = file_id {
                kwargs.set_item("file_id", file_id)?;
            }
            self.to_object(py).call_method(
                py,
                "set_reference_info",
                (path.to_string_lossy().as_ref(), location),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn subsume(&self, other: &dyn PyWorkingTree) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "subsume", (other.to_object(py),))?;
            Ok(())
        })
    }

    fn store_uncommitted(&self) -> Result<String, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, "store_uncommitted")?
                .extract(py)?)
        })
    }

    fn restore_uncommitted(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "restore_uncommitted")?;
            Ok(())
        })
    }

    fn extract(&self, dest: &Path, format: Option<&str>) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(format) = format {
                kwargs.set_item("format", format)?;
            }
            self.to_object(py).call_method(
                py,
                "extract",
                (dest.to_string_lossy().as_ref(),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn clone(
        &self,
        dest: &Path,
        revision_id: Option<&RevisionId>,
    ) -> Result<GenericWorkingTree, Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(revision_id) = revision_id {
                kwargs.set_item(
                    "revision_id",
                    revision_id.clone().into_pyobject(py).unwrap(),
                )?;
            }
            let result = self.to_object(py).call_method(
                py,
                "clone",
                (dest.to_string_lossy().as_ref(),),
                Some(&kwargs),
            )?;
            Ok(GenericWorkingTree(result))
        })
    }

    fn control_transport(&self) -> Result<crate::transport::Transport, Error> {
        Python::attach(|py| {
            let transport = self.to_object(py).getattr(py, "control_transport")?;
            Ok(crate::transport::Transport::new(transport))
        })
    }

    fn control_url(&self) -> url::Url {
        Python::attach(|py| {
            let url: String = self
                .to_object(py)
                .getattr(py, "control_url")
                .unwrap()
                .extract(py)
                .unwrap();
            url.parse().unwrap()
        })
    }

    fn copy_content_into(
        &self,
        source: &dyn PyTree,
        revision_id: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(revision_id) = revision_id {
                kwargs.set_item(
                    "revision_id",
                    revision_id.clone().into_pyobject(py).unwrap(),
                )?;
            }
            self.to_object(py).call_method(
                py,
                "copy_content_into",
                (source.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn flush(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "flush")?;
            Ok(())
        })
    }

    fn requires_rich_root(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "requires_rich_root")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn reset_state(&self, revision_ids: Option<&[RevisionId]>) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(revision_ids) = revision_ids {
                let revision_ids_py: Vec<Py<PyAny>> = revision_ids
                    .iter()
                    .map(|id| id.clone().into_pyobject(py).unwrap().unbind())
                    .collect();
                kwargs.set_item("revision_ids", revision_ids_py)?;
            }
            self.to_object(py)
                .call_method(py, "reset_state", (), Some(&kwargs))?;
            Ok(())
        })
    }

    fn reference_parent(
        &self,
        path: &Path,
        branch: &dyn Branch,
        revision_id: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(revision_id) = revision_id {
                kwargs.set_item(
                    "revision_id",
                    revision_id.clone().into_pyobject(py).unwrap(),
                )?;
            }
            // Try to cast to a concrete type that implements PyBranch
            let py_obj =
                if let Some(generic_branch) = branch.as_any().downcast_ref::<GenericBranch>() {
                    generic_branch.to_object(py)
                } else if let Some(py_branch) = branch
                    .as_any()
                    .downcast_ref::<crate::branch::MemoryBranch>()
                {
                    py_branch.to_object(py)
                } else {
                    return Err(Error::Other(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Branch must be a PyBranch implementation for reference_parent operation"
                )));
                };
            self.to_object(py).call_method(
                py,
                "reference_parent",
                (path.to_string_lossy().as_ref(), py_obj),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn supports_merge_modified(&self) -> bool {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "supports_merge_modified")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn break_lock(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "break_lock")?;
            Ok(())
        })
    }

    fn get_physical_lock_status(&self) -> Result<bool, Error> {
        Python::attach(|py| {
            Ok(self
                .to_object(py)
                .call_method0(py, "get_physical_lock_status")?
                .extract(py)?)
        })
    }
}

/// A working tree in a version control system.
///
/// A working tree is a local directory containing the files of a branch that can
/// be edited. This struct wraps a Python working tree object and provides access
/// to its functionality.
pub struct GenericWorkingTree(pub Py<PyAny>);

impl crate::tree::PyTree for GenericWorkingTree {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}
impl crate::tree::PyMutableTree for GenericWorkingTree {}

impl PyWorkingTree for GenericWorkingTree {}

impl Clone for GenericWorkingTree {
    fn clone(&self) -> Self {
        Python::attach(|py| GenericWorkingTree(self.0.clone_ref(py)))
    }
}

impl<'py> IntoPyObject<'py> for GenericWorkingTree {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

/// A builder for creating commits in a working tree.
///
/// This struct provides a fluent interface for setting the parameters of a commit
/// and then creating it.
pub struct CommitBuilder(GenericWorkingTree, Py<pyo3::types::PyDict>);

impl From<GenericWorkingTree> for CommitBuilder {
    /// Create a new CommitBuilder from a WorkingTree.
    ///
    /// # Parameters
    ///
    /// * `wt` - The working tree to create commits in.
    ///
    /// # Returns
    ///
    /// A new CommitBuilder instance.
    fn from(wt: GenericWorkingTree) -> Self {
        Python::attach(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            CommitBuilder(wt, kwargs.into())
        })
    }
}

impl CommitBuilder {
    /// Set the committer for this commit.
    ///
    /// # Parameters
    ///
    /// * `committer` - The committer's name and email.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn committer(self, committer: &str) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("committer", committer).unwrap();
        });
        self
    }

    /// Set the commit message.
    ///
    /// # Parameters
    ///
    /// * `message` - The commit message.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn message(self, message: &str) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("message", message).unwrap();
        });
        self
    }

    /// Specify which files to include in this commit.
    ///
    /// # Parameters
    ///
    /// * `specific_files` - The paths of files to include in this commit.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn specific_files(self, specific_files: &[&Path]) -> Self {
        let specific_files: Vec<String> = specific_files
            .iter()
            .map(|x| x.to_string_lossy().to_string())
            .collect();
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("specific_files", specific_files)
                .unwrap();
        });
        self
    }

    /// Allow pointless commits.
    ///
    /// # Parameters
    ///
    /// * `allow_pointless` - Whether to allow commits that don't change any files.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn allow_pointless(self, allow_pointless: bool) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("allow_pointless", allow_pointless)
                .unwrap();
        });
        self
    }

    /// Set a reporter for this commit.
    ///
    /// # Parameters
    ///
    /// * `reporter` - The commit reporter to use.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn reporter(self, reporter: &dyn crate::commit::PyCommitReporter) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("reporter", reporter.to_object(py))
                .unwrap();
        });
        self
    }

    /// Set the timestamp for this commit.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - The timestamp for the commit.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn timestamp(self, timestamp: f64) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("timestamp", timestamp).unwrap();
        });
        self
    }

    /// Set a revision property for this commit.
    ///
    /// Revision properties are key-value pairs that can be attached to commits
    /// to store additional metadata beyond the standard commit fields.
    ///
    /// # Parameters
    ///
    /// * `key` - The property key (name).
    /// * `value` - The property value as a string.
    ///
    /// # Returns
    ///
    /// Self for method chaining, or an error if the operation failed.
    pub fn set_revprop(self, key: &str, value: &str) -> Result<Self, Error> {
        Python::attach(|py| {
            // Get or create the revprops dictionary
            if self.1.bind(py).get_item("revprops")?.is_none() {
                let new_revprops = pyo3::types::PyDict::new(py);
                self.1.bind(py).set_item("revprops", new_revprops)?;
            }

            // Now get the revprops dictionary and set the property value
            let revprops = self.1.bind(py).get_item("revprops")?.ok_or_else(|| {
                Error::Other(pyo3::PyErr::new::<pyo3::exceptions::PyAssertionError, _>(
                    "revprops should exist after setting it",
                ))
            })?;

            let revprops_dict = revprops.cast::<pyo3::types::PyDict>().map_err(|_| {
                Error::Other(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "revprops is not a dictionary",
                ))
            })?;

            revprops_dict.set_item(key, value)?;
            Ok(self)
        })
    }

    /// Create the commit.
    ///
    /// # Returns
    ///
    /// The revision ID of the new commit, or an error if the commit could not be created.
    pub fn commit(self) -> Result<RevisionId, Error> {
        Python::attach(|py| {
            Ok(self
                .0
                .to_object(py)
                .call_method(py, "commit", (), Some(self.1.bind(py)))?
                .extract(py)
                .unwrap())
        })
    }
}

impl GenericWorkingTree {
    /// Open a working tree at the specified path.
    ///
    /// This method is deprecated, use the module-level `open` function instead.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to the working tree.
    ///
    /// # Returns
    ///
    /// The working tree, or an error if it could not be opened.
    #[deprecated = "Use ::open instead"]
    pub fn open(path: &Path) -> Result<GenericWorkingTree, Error> {
        open(path)
    }

    /// Open a working tree containing the specified path.
    ///
    /// This method is deprecated, use the module-level `open_containing` function instead.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to look for a containing working tree.
    ///
    /// # Returns
    ///
    /// A tuple containing the working tree and the relative path, or an error
    /// if no containing working tree could be found.
    #[deprecated = "Use ::open_containing instead"]
    pub fn open_containing(path: &Path) -> Result<(GenericWorkingTree, PathBuf), Error> {
        open_containing(path)
    }

    /// Create a commit with the specified parameters.
    ///
    /// This method is deprecated, use the `build_commit` method instead.
    ///
    /// # Parameters
    ///
    /// * `message` - The commit message.
    /// * `allow_pointless` - Whether to allow commits that don't change any files.
    /// * `committer` - The committer's name and email.
    /// * `specific_files` - The paths of files to include in this commit.
    ///
    /// # Returns
    ///
    /// The revision ID of the new commit, or an error if the commit could not be created.
    #[deprecated = "Use build_commit instead"]
    pub fn commit(
        &self,
        message: &str,
        committer: Option<&str>,
        timestamp: Option<f64>,
        allow_pointless: Option<bool>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error> {
        let mut builder = self.build_commit().message(message);

        if let Some(specific_files) = specific_files {
            builder = builder.specific_files(specific_files);
        }

        if let Some(allow_pointless) = allow_pointless {
            builder = builder.allow_pointless(allow_pointless);
        }

        if let Some(committer) = committer {
            builder = builder.committer(committer);
        }

        if let Some(timestamp) = timestamp {
            builder = builder.timestamp(timestamp);
        }

        builder.commit()
    }
}

/// Open a working tree at the specified path.
///
/// # Parameters
///
/// * `path` - The path of the working tree to open.
///
/// # Returns
///
/// The working tree, or an error if it could not be opened.
pub fn open(path: &Path) -> Result<GenericWorkingTree, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let wt = c.call_method1("open", (path.to_string_lossy().to_string(),))?;
        Ok(GenericWorkingTree(wt.unbind()))
    })
}

/// Open a working tree containing the specified path.
///
/// This function searches for a working tree containing the specified path
/// and returns both the working tree and the path relative to the working tree.
///
/// # Parameters
///
/// * `path` - The path to look for a containing working tree.
///
/// # Returns
///
/// A tuple containing the working tree and the relative path, or an error
/// if no containing working tree could be found.
pub fn open_containing(path: &Path) -> Result<(GenericWorkingTree, PathBuf), Error> {
    Python::attach(|py| {
        let m = py.import("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let (wt, p): (Bound<PyAny>, String) = c
            .call_method1("open_containing", (path.to_string_lossy(),))?
            .extract()?;
        Ok((GenericWorkingTree(wt.unbind()), PathBuf::from(p)))
    })
}

/// Implementation of From<Py<PyAny>> for GenericWorkingTree.
impl From<Py<PyAny>> for GenericWorkingTree {
    /// Create a new WorkingTree from a Python object.
    ///
    /// # Parameters
    ///
    /// * `obj` - The Python object representing a working tree.
    ///
    /// # Returns
    ///
    /// A new WorkingTree instance.
    fn from(obj: Py<PyAny>) -> Self {
        GenericWorkingTree(obj)
    }
}
