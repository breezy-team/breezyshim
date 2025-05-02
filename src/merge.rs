//! Tree merging.
use crate::branch::PyBranch;
use crate::graph::Graph;
use crate::hooks::HookDict;
use crate::transform::TreeTransform;
use crate::tree::PyTree;
use crate::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::types::PyDict;

import_exception!(breezy.errors, UnrelatedBranches);

/// Errors that can occur during merge operations.
pub enum Error {
    /// Error indicating that the branches being merged are unrelated.
    ///
    /// This occurs when the branches have no common ancestor.
    UnrelatedBranches,
}

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<UnrelatedBranches>(py) {
                Error::UnrelatedBranches
            } else {
                panic!("unexpected error: {:?}", e)
            }
        })
    }
}

/// Represents a merge operation between two branches.
///
/// This struct provides methods to configure and perform merges between branches,
/// including finding the base revision, setting merge parameters, and executing the merge.
pub struct Merger(PyObject);

/// Types of merge algorithms that can be used.
pub enum MergeType {
    /// Three-way merge algorithm.
    ///
    /// This is the standard merge algorithm that uses a common base revision
    /// and the two branches to be merged.
    Merge3,
}

impl From<PyObject> for Merger {
    fn from(obj: PyObject) -> Self {
        Merger(obj)
    }
}

impl Merger {
    /// Create a new merger for merging into a tree.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch to merge from
    /// * `this_tree` - The tree to merge into
    /// * `revision_graph` - The graph of revisions to use for finding common ancestors
    ///
    /// # Returns
    ///
    /// A new Merger object
    pub fn new<T: PyTree, B: PyBranch>(branch: &B, this_tree: &T, revision_graph: &Graph) -> Self {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.merge").unwrap();
            let cls = m.getattr("Merger").unwrap();
            let kwargs = PyDict::new_bound(py);
            kwargs
                .set_item("this_tree", this_tree.to_object(py))
                .unwrap();
            kwargs
                .set_item("revision_graph", revision_graph.to_object(py))
                .unwrap();
            let merger = cls.call((branch.to_object(py),), Some(&kwargs)).unwrap();
            Merger(merger.into())
        })
    }

    /// Find the base revision for the merge.
    ///
    /// # Returns
    ///
    /// The base revision ID if found, or None if the branches are unrelated
    pub fn find_base(&self) -> Result<Option<RevisionId>, crate::error::Error> {
        Python::with_gil(|py| match self.0.call_method0(py, "find_base") {
            Ok(_py_obj) => Ok(self
                .0
                .getattr(py, "base_rev_id")
                .unwrap()
                .extract(py)
                .unwrap()),
            Err(err) => {
                if err.is_instance_of::<UnrelatedBranches>(py) {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        })
        .map_err(Into::into)
    }

    /// Set the other revision to merge.
    ///
    /// # Arguments
    ///
    /// * `other_revision` - The revision ID to merge
    /// * `other_branch` - The branch containing the revision
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if the operation fails
    pub fn set_other_revision<B: PyBranch>(
        &mut self,
        other_revision: &RevisionId,
        other_branch: &B,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(
                py,
                "set_other_revision",
                (other_revision.clone(), other_branch.to_object(py)),
            )?;
            Ok(())
        })
    }

    /// Set the base revision for the merge.
    ///
    /// # Arguments
    ///
    /// * `base_revision` - The base revision ID to use
    /// * `base_branch` - The branch containing the base revision
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if the operation fails
    pub fn set_base_revision<B: PyBranch>(
        &mut self,
        base_revision: &RevisionId,
        base_branch: &B,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(
                py,
                "set_base_revision",
                (base_revision.clone(), base_branch.to_object(py)),
            )?;
            Ok(())
        })
    }

    /// Set the merge algorithm to use.
    ///
    /// # Arguments
    ///
    /// * `merge_type` - The merge algorithm to use
    pub fn set_merge_type(&mut self, merge_type: MergeType) {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.merge").unwrap();
            let merge_type = match merge_type {
                MergeType::Merge3 => m.getattr("Merge3Merger").unwrap(),
            };
            self.0.setattr(py, "merge_type", merge_type).unwrap();
        })
    }

    /// Create a submerger to execute the merge.
    ///
    /// # Returns
    ///
    /// A Submerger object that can perform the actual merge
    pub fn make_merger(&self) -> Result<Submerger, crate::error::Error> {
        Python::with_gil(|py| {
            let merger = self.0.call_method0(py, "make_merger")?;
            Ok(Submerger(merger))
        })
    }

    /// Create a merger from specific revision IDs.
    ///
    /// # Arguments
    ///
    /// * `other_tree` - The tree to merge from
    /// * `other_branch` - The branch containing the revision to merge
    /// * `other` - The revision ID to merge
    /// * `tree_branch` - The branch containing the tree to merge into
    ///
    /// # Returns
    ///
    /// A new Merger object, or an error if the operation fails
    pub fn from_revision_ids<T: PyTree, B1: PyBranch, B2: PyBranch>(
        other_tree: &T,
        other_branch: &B1,
        other: &RevisionId,
        tree_branch: &B2,
    ) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.merge").unwrap();
            let cls = m.getattr("Merger").unwrap();
            let kwargs = PyDict::new_bound(py);
            kwargs
                .set_item("other_branch", other_branch.to_object(py))
                .unwrap();
            kwargs.set_item("other", other.to_object(py)).unwrap();
            kwargs
                .set_item("tree_branch", tree_branch.to_object(py))
                .unwrap();
            let merger = cls.call_method(
                "from_revision_ids",
                (other_tree.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(Merger(merger.into()))
        })
    }
}

/// Performs the actual merge operation.
///
/// This struct is created by the Merger.make_merger() method and provides
/// methods to execute the merge and create transformations.
pub struct Submerger(PyObject);

impl Submerger {
    /// Create a preview transformation of the merge.
    ///
    /// This allows inspecting the changes that would be made by the merge
    /// without actually applying them to the working tree.
    ///
    /// # Returns
    ///
    /// A TreeTransform object representing the merge changes
    pub fn make_preview_transform(&self) -> Result<TreeTransform, crate::error::Error> {
        Python::with_gil(|py| {
            let transform = self
                .0
                .call_method0(py, "make_preview_transform")?
                .to_object(py);
            Ok(TreeTransform::from(transform))
        })
    }
}

lazy_static::lazy_static! {
    /// Hooks that are called during merge operations.
    pub static ref MERGE_HOOKS: HookDict = HookDict::new("breezy.merge", "Merger", "hooks");
}
