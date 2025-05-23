//! Working trees in version control systems.
//!
//! This module provides functionality for working with working trees, which are
//! local directories containing the files of a branch that can be edited.
use crate::branch::{GenericBranch, PyBranch};
use crate::controldir::{ControlDir, GenericControlDir};
use crate::error::Error;
use crate::tree::RevisionTree;
use crate::RevisionId;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

/// A working tree in a version control system.
///
/// A working tree is a local directory containing the files of a branch that can
/// be edited. This struct wraps a Python working tree object and provides access
/// to its functionality.
pub struct WorkingTree(pub PyObject);

impl crate::tree::PyTree for WorkingTree {}
impl crate::tree::PyMutableTree for WorkingTree {}

impl Clone for WorkingTree {
    fn clone(&self) -> Self {
        Python::with_gil(|py| WorkingTree(self.0.clone_ref(py)))
    }
}

impl ToPyObject for WorkingTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

/// A builder for creating commits in a working tree.
///
/// This struct provides a fluent interface for setting the parameters of a commit
/// and then creating it.
pub struct CommitBuilder(WorkingTree, Py<pyo3::types::PyDict>);

impl From<WorkingTree> for CommitBuilder {
    /// Create a new CommitBuilder from a WorkingTree.
    ///
    /// # Parameters
    ///
    /// * `wt` - The working tree to create commits in.
    ///
    /// # Returns
    ///
    /// A new CommitBuilder instance.
    fn from(wt: WorkingTree) -> Self {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new_bound(py);
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
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
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
        let specific_files: Vec<PathBuf> = specific_files.iter().map(|x| x.to_path_buf()).collect();
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
            self.1.bind(py).set_item("reporter", reporter).unwrap();
        });
        self
    }

    /// Create the commit.
    ///
    /// # Returns
    ///
    /// The revision ID of the new commit, or an error if the commit could not be created.
    pub fn commit(self) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            Ok(self
                .0
                .to_object(py)
                .call_method_bound(py, "commit", (), Some(self.1.bind(py)))?
                .extract(py)
                .unwrap())
        })
    }
}

impl WorkingTree {
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
    pub fn is_control_filename(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "is_control_filename", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Return the base path for this working tree.
    ///
    /// # Returns
    ///
    /// The base directory path of this working tree.
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
    ///
    /// # Returns
    ///
    /// The branch associated with this working tree.
    pub fn branch(&self) -> GenericBranch {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch").unwrap();
            GenericBranch::from(branch)
        })
    }

    /// Return the control directory for this working tree.
    ///
    /// # Returns
    ///
    /// The control directory containing this working tree.
    pub fn controldir(&self) -> Box<dyn ControlDir> {
        Python::with_gil(|py| {
            let controldir = self.to_object(py).getattr(py, "controldir").unwrap();
            Box::new(GenericControlDir::new(controldir)) as Box<dyn ControlDir>
        })
    }

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
    pub fn open(path: &Path) -> Result<WorkingTree, Error> {
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
    pub fn open_containing(path: &Path) -> Result<(WorkingTree, PathBuf), Error> {
        open_containing(path)
    }

    /// Get the basis tree for this working tree.
    ///
    /// The basis tree is the tree of the last revision, which is the state
    /// of the tree before any uncommitted changes.
    ///
    /// # Returns
    ///
    /// The basis tree, or an error if it could not be retrieved.
    pub fn basis_tree(&self) -> Result<crate::tree::RevisionTree, Error> {
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method0(py, "basis_tree")?;
            Ok(RevisionTree(tree))
        })
    }

    /// Get a revision tree for a specific revision.
    ///
    /// # Parameters
    ///
    /// * `revision_id` - The ID of the revision to get the tree for.
    ///
    /// # Returns
    ///
    /// The revision tree, or an error if it could not be retrieved.
    pub fn revision_tree(&self, revision_id: &RevisionId) -> Result<Box<RevisionTree>, Error> {
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method1(
                py,
                "revision_tree",
                (revision_id.to_object(py),),
            )?;
            Ok(Box::new(RevisionTree(tree)))
        })
    }

    /// Get a dictionary of tags mapped to revision IDs.
    ///
    /// # Returns
    ///
    /// A hash map of tag names to revision IDs, or an error if the tags could not be retrieved.
    pub fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    /// Convert a path to an absolute path relative to the working tree.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to convert.
    ///
    /// # Returns
    ///
    /// The absolute path, or an error if the conversion failed.
    pub fn abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "abspath", (path,))?
                .extract(py)?)
        })
    }

    /// Convert an absolute path to a path relative to the working tree.
    ///
    /// # Parameters
    ///
    /// * `path` - The absolute path to convert.
    ///
    /// # Returns
    ///
    /// The relative path, or an error if the conversion failed.
    pub fn relpath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "relpath", (path,))?
                .extract(py)?)
        })
    }

    /// Check if this working tree supports setting file IDs.
    ///
    /// # Returns
    ///
    /// `true` if this working tree supports setting file IDs, `false` otherwise.
    pub fn supports_setting_file_ids(&self) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Add files to version control.
    ///
    /// # Parameters
    ///
    /// * `paths` - The paths of files to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the files could not be added.
    pub fn add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    /// Add files to version control, recursively adding subdirectories.
    ///
    /// This is similar to `add`, but smarter - it will recursively add
    /// subdirectories and handle ignored files appropriately.
    ///
    /// # Parameters
    ///
    /// * `paths` - The paths of files to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the files could not be added.
    pub fn smart_add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "smart_add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    /// Create a commit builder for this working tree.
    ///
    /// # Returns
    ///
    /// A new CommitBuilder instance for this working tree.
    pub fn build_commit(&self) -> CommitBuilder {
        CommitBuilder::from(self.clone())
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
        allow_pointless: Option<bool>,
        committer: Option<&str>,
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

        builder.commit()
    }

    /// Get the revision ID of the last commit in this working tree.
    ///
    /// # Returns
    ///
    /// The revision ID of the last commit, or an error if it could not be retrieved.
    pub fn last_revision(&self) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            let last_revision = self.to_object(py).call_method0(py, "last_revision")?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }

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
    pub fn pull<B: PyBranch>(
        &self,
        source: &B,
        overwrite: Option<bool>,
        stop_revision: Option<&RevisionId>,
        local: Option<bool>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new_bound(py);
                if let Some(overwrite) = overwrite {
                    kwargs.set_item("overwrite", overwrite).unwrap();
                }
                if let Some(stop_revision) = stop_revision {
                    kwargs
                        .set_item("stop_revision", stop_revision.to_object(py))
                        .unwrap();
                }
                if let Some(local) = local {
                    kwargs.set_item("local", local).unwrap();
                }
                kwargs
            };
            self.to_object(py)
                .call_method_bound(py, "pull", (source.to_object(py),), Some(&kwargs))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

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
    pub fn merge_from_branch<B: PyBranch>(
        &self,
        source: &B,
        to_revision: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new_bound(py);
                if let Some(to_revision) = to_revision {
                    kwargs
                        .set_item("to_revision", to_revision.to_object(py))
                        .unwrap();
                }
                kwargs
            };
            self.to_object(py).call_method_bound(
                py,
                "merge_from_branch",
                (source.to_object(py),),
                Some(&kwargs),
            )
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    /// Update the working tree to a different revision.
    ///
    /// # Parameters
    ///
    /// * `revision` - The revision to update to, or None for the latest revision.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the update could not be completed.
    pub fn update(&self, revision: Option<&RevisionId>) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new_bound(py);
                kwargs.set_item("revision", revision.to_object(py)).unwrap();
                kwargs
            };
            self.to_object(py)
                .call_method_bound(py, "update", (), Some(&kwargs))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

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
    pub fn safe_relpath_files(
        &self,
        file_list: &[&Path],
        canonicalize: bool,
        apply_view: bool,
    ) -> Result<Vec<PathBuf>, Error> {
        Python::with_gil(|py| {
            let result = self.to_object(py).call_method1(
                py,
                "safe_relpath_files",
                (
                    file_list
                        .iter()
                        .map(|x| x.to_path_buf())
                        .collect::<Vec<_>>(),
                    canonicalize,
                    apply_view,
                ),
            )?;
            Ok(result.extract(py)?)
        })
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
pub fn open(path: &Path) -> Result<WorkingTree, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let wt = c.call_method1("open", (path,))?;
        Ok(WorkingTree(wt.to_object(py)))
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
pub fn open_containing(path: &Path) -> Result<(WorkingTree, PathBuf), Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let (wt, p): (Bound<PyAny>, String) =
            c.call_method1("open_containing", (path,))?.extract()?;
        Ok((WorkingTree(wt.to_object(py)), PathBuf::from(p)))
    })
}

/// Implementation of From<PyObject> for WorkingTree.
impl From<PyObject> for WorkingTree {
    /// Create a new WorkingTree from a Python object.
    ///
    /// # Parameters
    ///
    /// * `obj` - The Python object representing a working tree.
    ///
    /// # Returns
    ///
    /// A new WorkingTree instance.
    fn from(obj: PyObject) -> Self {
        WorkingTree(obj)
    }
}
