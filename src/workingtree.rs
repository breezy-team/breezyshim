//! Working trees in version control systems.
//!
//! This module provides functionality for working with working trees, which are
//! local directories containing the files of a branch that can be edited.
use crate::branch::{GenericBranch, PyBranch};
use crate::controldir::{ControlDir, GenericControlDir};
use crate::error::Error;
use crate::tree::{MutableTree, PyMutableTree, PyTree, RevisionTree};
use crate::RevisionId;
use pyo3::intern;
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

    /// Commit changes in this working tree.
    ///
    /// # Parameters
    ///
    /// * `message` - The commit message.
    /// * `allow_pointless` - Whether to allow commits with no changes.
    /// * `committer` - Optional committer identity.
    /// * `specific_files` - Optional list of specific files to commit.
    ///
    /// # Returns
    ///
    /// The revision ID of the new commit, or an error if the commit failed.
    fn commit(
        &self,
        message: &str,
        allow_pointless: Option<bool>,
        committer: Option<&str>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error>;

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

    /// Get a dictionary of tags mapped to revision IDs.
    ///
    /// # Returns
    ///
    /// A hash map of tag names to revision IDs, or an error if the tags could not be retrieved.
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error>;

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

    /// Get the revision ID of the last commit in this working tree.
    ///
    /// # Returns
    ///
    /// The revision ID of the last commit, or an error if it could not be retrieved.
    fn last_revision(&self) -> Result<RevisionId, Error>;

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
        source: &dyn PyBranch,
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
        source: &dyn PyBranch,
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
}

/// Trait for working trees that wrap Python working tree objects.
///
/// This trait is implemented by working tree types that wrap Python working tree objects.
pub trait PyWorkingTree: PyMutableTree {}

impl<T: ?Sized + PyWorkingTree> WorkingTree for T {
    fn basedir(&self) -> PathBuf {
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
            GenericBranch::from(self.to_object(py).getattr(py, "branch").unwrap())
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn smart_add(&self, files: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            let file_paths: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            self.to_object(py)
                .call_method1(py, "smart_add", (file_paths,))?;
            Ok(())
        })
    }

    fn commit(
        &self,
        message: &str,
        allow_pointless: Option<bool>,
        committer: Option<&str>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(allow_pointless) = allow_pointless {
                kwargs.set_item("allow_pointless", allow_pointless)?;
            }
            if let Some(committer) = committer {
                kwargs.set_item("committer", committer)?;
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
            Ok(result.extract(py)?)
        })
    }

    fn update(&self, revision_id: Option<&RevisionId>) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "update", (revision_id.cloned(),))?;
            Ok(())
        })
    }

    fn revert(&self, filenames: Option<&[&Path]>) -> Result<(), Error> {
        Python::with_gil(|py| {
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
        Python::with_gil(|py| CommitBuilder::from(GenericWorkingTree(self.to_object(py))))
    }

    fn basis_tree(&self) -> Result<RevisionTree, Error> {
        Python::with_gil(|py| {
            let basis_tree = self.to_object(py).call_method0(py, "basis_tree")?;
            Ok(RevisionTree(basis_tree))
        })
    }

    fn is_control_filename(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
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
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method1(
                py,
                "revision_tree",
                (revision_id.clone().into_pyobject(py).unwrap(),),
            )?;
            Ok(Box::new(RevisionTree(tree)))
        })
    }

    /// Get a dictionary of tags mapped to revision IDs.
    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, intern!(py, "get_tag_dict"))?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    /// Convert a path to an absolute path relative to the working tree.
    fn abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "abspath", (path.to_string_lossy().as_ref(),))?
                .extract(py)?)
        })
    }

    /// Convert an absolute path to a path relative to the working tree.
    fn relpath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "relpath", (path.to_string_lossy().as_ref(),))?
                .extract(py)?)
        })
    }

    /// Get the revision ID of the last commit in this working tree.
    fn last_revision(&self) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            let last_revision = self
                .to_object(py)
                .call_method0(py, intern!(py, "last_revision"))?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }

    /// Pull changes from another branch into this working tree.
    fn pull(
        &self,
        source: &dyn PyBranch,
        overwrite: Option<bool>,
        stop_revision: Option<&RevisionId>,
        local: Option<bool>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
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
            self.to_object(py)
                .call_method(py, "pull", (source.to_object(py),), Some(&kwargs))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    /// Merge changes from another branch into this working tree.
    fn merge_from_branch(
        &self,
        source: &dyn PyBranch,
        to_revision: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
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
            self.to_object(py).call_method(
                py,
                "merge_from_branch",
                (source.to_object(py),),
                Some(&kwargs),
            )
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    /// Convert a list of files to relative paths safely.
    fn safe_relpath_files(
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
                        .map(|x| x.to_string_lossy().to_string())
                        .collect::<Vec<_>>(),
                    canonicalize,
                    apply_view,
                ),
            )?;
            Ok(result.extract(py)?)
        })
    }
}

/// A working tree in a version control system.
///
/// A working tree is a local directory containing the files of a branch that can
/// be edited. This struct wraps a Python working tree object and provides access
/// to its functionality.
pub struct GenericWorkingTree(pub PyObject);

impl crate::tree::PyTree for GenericWorkingTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}
impl crate::tree::PyMutableTree for GenericWorkingTree {}

impl PyWorkingTree for GenericWorkingTree {}

impl Clone for GenericWorkingTree {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericWorkingTree(self.0.clone_ref(py)))
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
        Python::with_gil(|py| {
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
            self.1
                .bind(py)
                .set_item("reporter", reporter.to_object(py))
                .unwrap();
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
    Python::with_gil(|py| {
        let m = py.import("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let wt = c.call_method1("open", (path,))?;
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
    Python::with_gil(|py| {
        let m = py.import("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let (wt, p): (Bound<PyAny>, String) = c
            .call_method1("open_containing", (path.to_string_lossy(),))?
            .extract()?;
        Ok((GenericWorkingTree(wt.unbind()), PathBuf::from(p)))
    })
}

/// Implementation of From<PyObject> for GenericWorkingTree.
impl From<PyObject> for GenericWorkingTree {
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
        GenericWorkingTree(obj)
    }
}
